# **Technical Specification & Design Document: Project "rustyengine"**

## **A Rust Engine Prototype with Modular Subsystems**

## **1\. Executive Summary & Current Status**

Project "rustyengine" is currently a modular Rust engine prototype rather than a finished cross-platform game engine. The repository already contains subsystem scaffolding for configuration, input, audio, rendering, physics, networking, gameplay, and UI, and the current test suite passes with `cargo test` (17 tests, 0 failures). The design goals are still the same as the original vision — safe, modular, and testable Rust architecture — but the present codebase is best understood as an active compile-fix / prototype phase.

### **What is implemented today**

1. **Prototype subsystems:** `src/config/`, `src/input/`, `src/audio/`, `src/renderer/`, `src/physics/`, `src/network/`, `src/gameplay/`, and `src/ui/` are present and exercising core logic through tests.
2. **Verified baseline:** The repository currently passes `cargo test`, which confirms the available subsystem logic is stable enough for incremental development.
3. **Runtime entry point:** The top-level executable is still a placeholder (`main` prints `Hello, world!`), so the engine is not yet wired into a full real-time window/render loop.

### **Target Applications**

1. **Survival Game:** Demands high entity counts (foliage, items, wildlife), large static and dynamic world streaming, collision detection for dense environments, persistent world state, and client-side prediction under high-latency networking.  
2. **Sim Racing Game:** Demands deterministic, high-frequency physics tick rates (![][image1] to ![][image2]), extremely low input-to-render latency, complex rigid-body constraints (suspension, tire-to-surface interaction), local/LAN multiplayer, and high-fidelity spatial audio for engine, tire, and ambient sounds.

## **2\. Engine Architecture Overview**

rustyengine is currently structured as a collection of independent modules and tests that can be assembled into a future application loop. The long-term architecture remains aspirational, but the present repository state is closer to a subsystem toolkit than a finished engine runtime.

                           \+------------------------+  
                           |    Main Loop (Winit)   |  
                           \+-----------+------------+  
                                       |  
                  \+--------------------+--------------------+  
                  |                                         |  
        \+---------v---------+                     \+---------v---------+  
        |   System Physics  |                     |  System Render    |  
        |   (120Hz-400Hz)   |                     |     (Variable)    |  
        |  \[Rapier3D Sim\]   |                     | \[wgpu (Vulkan/GL)\]|  
        \+---------+---------+                     \+---------+---------+  
                  |                                         |  
                  \+--------------------+--------------------+  
                                       |  
                     \+-----------------v-----------------+  
                     |            Custom ECS             |  
                     |  (Entities, Components, Systems)  |  
                     \+--------+-----------------+--------+  
                              |                 |  
                    \+---------v---------+     \+-v-----------------+  
                    |   System Audio    |     | System Networking |  
                    |  \[Kira Spatial\]   |     |  \[Renet Netcode\]  |  
                    \+-------------------+     \+-------------------+

### **Core Architecture Principles**

* **Prototype First, Then Integration:** The repository is currently validating logic and subsystem boundaries through tests before the runtime loop is fully connected.  
* **Explicit Subsystem Boundaries:** Configuration, input, audio, renderer, physics, networking, gameplay, and UI are intentionally isolated modules.  
* **Safe Rust Foundations:** The code uses standard Rust abstractions and test coverage to keep the design approachable while the full runtime is still being wired up.  
* **Incremental Runtime Construction:** The future application loop, render backend, and deterministic simulation stack should be built on top of the existing modular pieces rather than replacing them.

### **Current Project Snapshot**

* The current implementation status is best described as a working prototype with subsystem tests and scaffolding.
* The main runtime path is still incomplete, so the document below should be read as a design target and implementation roadmap rather than a fully deployed engine specification.
* The next practical milestone is to connect the existing modules into a real engine loop, then validate the integration path with the same test suite that currently passes.

## **3\. Module Specifications**

### **Module A: Custom Safe Sparse-Set ECS**

We implement a safe, custom **Sparse-Set** ECS optimized for ![][image3] lookup, linear cache-friendly iteration, and secure dynamic borrowing without unsafe pointer tricks.

#### **Data Structures**

* **Entity:** A 64-bit wrapper containing an index (u32) and a generation (u32) to prevent stale entity handle reuse.  
* **Sparse Set Storage:** Maps sparse entity indices to packed component arrays.  
  * dense: A packed contiguous vector Vec\<T\> of components.  
  * dense\_entities: A vector tracking which Entity owns each component in dense.  
  * sparse: A vector Vec\<Option\<usize\>\> where the entity index points to its index in the dense list.

use std::any::{Any, TypeId};  
use std::cell::{Ref, RefCell, RefMut};  
use std::collections::HashMap;

\#\[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)\]  
pub struct Entity {  
    pub id: u32,  
    pub generation: u32,  
}

pub trait ComponentStoreTrait: Any {  
    fn as\_any(\&self) \-\> \&dyn Any;  
    fn as\_any\_mut(\&mut self) \-\> \&mut dyn Any;  
    fn remove\_entity(\&mut self, entity: Entity);  
}

pub struct ComponentStore\<T\> {  
    dense: Vec\<T\>,  
    dense\_entities: Vec\<Entity\>,  
    sparse: Vec\<Option\<usize\>\>,  
}

impl\<T: Any\> ComponentStore\<T\> {  
    pub fn with\_capacity(capacity: usize) \-\> Self {  
        Self {  
            dense: Vec::with\_capacity(capacity),  
            dense\_entities: Vec::with\_capacity(capacity),  
            sparse: Vec::with\_capacity(capacity),  
        }  
    }

    pub fn insert(\&mut self, entity: Entity, component: T) {  
        let index \= entity.id as usize;  
        if index \>= self.sparse.len() {  
            self.sparse.resize(index \+ 1, None);  
        }

        if let Some(dense\_idx) \= self.sparse\[index\] {  
            self.dense\[dense\_idx\] \= component;  
        } else {  
            let dense\_idx \= self.dense.len();  
            self.dense.push(component);  
            self.dense\_entities.push(entity);  
            self.sparse\[index\] \= Some(dense\_idx);  
        }  
    }

    pub fn remove(\&mut self, entity: Entity) \-\> Option\<T\> {  
        let index \= entity.id as usize;  
        if index \>= self.sparse.len() {  
            return None;  
        }

        let dense\_idx \= self.sparse\[index\]?;  
        self.sparse\[index\] \= None;

        let component \= self.dense.swap\_remove(dense\_idx);  
        self.dense\_entities.swap\_remove(dense\_idx);

        if dense\_idx \< self.dense.len() {  
            let swapped\_entity \= self.dense\_entities\[dense\_idx\];  
            self.sparse\[swapped\_entity.id as usize\] \= Some(dense\_idx);  
        }

        Some(component)  
    }

    pub fn get(\&self, entity: Entity) \-\> Option\<\&T\> {  
        let dense\_idx \= self.sparse.get(entity.id as usize)?.as\_ref()?;  
        Some(\&self.dense\[\*dense\_idx\])  
    }

    pub fn get\_mut(\&mut self, entity: Entity) \-\> Option\<\&mut T\> {  
        let dense\_idx \= self.sparse.get(entity.id as usize)?.as\_ref()?;  
        Some(\&mut self.dense\[\*dense\_idx\])  
    }

    pub fn iter(\&self) \-\> impl Iterator\<Item \= (\&Entity, \&T)\> {  
        self.dense\_entities.iter().zip(self.dense.iter())  
    }  
}

impl\<T: Any\> ComponentStoreTrait for ComponentStore\<T\> {  
    fn as\_any(\&self) \-\> \&dyn Any { self }  
    fn as\_any\_mut(\&mut self) \-\> \&mut dyn Any { self }  
    fn remove\_entity(\&mut self, entity: Entity) {  
        self.remove(entity);  
    }  
}

#### **Unified World & Safe Runtime Queries**

To manage multiple component types safely without compiler aliasing errors, we wrap each ComponentStore in a runtime borrow-checked container:

pub struct World {  
    entities: Vec\<u32\>, // Generation tracking  
    free\_entities: Vec\<u32\>,  
    stores: HashMap\<TypeId, RefCell\<Box\<dyn ComponentStoreTrait\>\>\>,  
}

impl World {  
    pub fn new() \-\> Self {  
        Self {  
            entities: Vec::new(),  
            free\_entities: Vec::new(),  
            stores: HashMap::new(),  
        }  
    }

    pub fn create\_entity(\&mut self) \-\> Entity {  
        if let Some(id) \= self.free\_entities.pop() {  
            let gen \= self.entities\[id as usize\];  
            Entity { id, generation: gen }  
        } else {  
            let id \= self.entities.len() as u32;  
            self.entities.push(1);  
            Entity { id, generation: 1 }  
        }  
    }

    pub fn register\_component\<T: Any\>(\&mut self, capacity: usize) {  
        let store \= Box::new(ComponentStore::\<T\>::with\_capacity(capacity));  
        self.stores.insert(TypeId::of::\<T\>(), RefCell::new(store));  
    }

    pub fn borrow\_store\<T: Any\>(\&self) \-\> Ref\<'\_, ComponentStore\<T\>\> {  
        let cell \= self.stores.get(\&TypeId::of::\<T\>()).expect("Component not registered");  
        Ref::map(cell.borrow(), |store| {  
            store.as\_any().downcast\_ref::\<ComponentStore\<T\>\>().unwrap()  
        })  
    }

    pub fn borrow\_store\_mut\<T: Any\>(\&self) \-\> RefMut\<'\_, ComponentStore\<T\>\> {  
        let cell \= self.stores.get(\&TypeId::of::\<T\>()).expect("Component not registered");  
        Ref::map\_mut(cell.borrow\_mut(), |store| {  
            store.as\_any\_mut().downcast\_mut::\<ComponentStore\<T\>\>().unwrap()  
        })  
    }  
}

* **Query Intersection Optimization:** When querying entities that have multiple components (e.g., Transform and Velocity), our systems query the smallest dense store first, then verify matching indices in the secondary sparse components. This maximizes CPU cache efficiency and avoids ![][image4] checks.

### **Module B: wgpu Cross-Platform Renderer**

wgpu provides a safe, idiomatic Rust graphics API acting as a translation layer. It runs on Vulkan by default, but seamlessly falls back to OpenGL (via GLES 3.0 / GL 4.2 backends) or Metal/DirectX when required.

       \+---------------------------------------------------------+  
       |                  wgpu Rendering Wrapper                 |  
       \+---------------------------------------------------------+  
       |   wgpu::Instance \-\> Selects Adapter (Vulkan Pref)       |  
       |   Fallbacks configured: Vulkan \-\> DX12 \-\> Metal \-\> GL   |  
       \+---------------------------------------------------------+  
       |   wgpu::Device & Queue (Resource and command creation)  |  
       |   wgpu::Surface & SurfaceConfiguration (Swapchain)      |  
       |   wgpu::RenderPipeline & BindGroupLayouts               |  
       \+---------------------------------------------------------+

#### **Core Components**

1. **Instance Configuration:** Configured to favor Vulkan backends but allow OpenGL ES 3.0 / GL fallback:  
   let instance \= wgpu::Instance::new(wgpu::InstanceDescriptor {  
       backends: wgpu::Backends::VULKAN | wgpu::Backends::GL,  
       dx12\_shader\_compiler: Default::default(),  
       flags: wgpu::InstanceFlags::default(),  
       gles\_minor\_version: wgpu::Gles3MinorVersion::default(),  
   });

2. **Pipelines and Shaders:** Built using **WGSL** (WebGPU Shading Language), compiling on-the-fly to SPIR-V or GLSL under the hood.  
3. **GPU-Driven Culling & Dynamic Indirect Drawing:**  
   To render high-density forests (Survival) or massive racetrack assets (Racing) without stalling the CPU, we leverage compute shaders and indirect drawing.  
   * Mesh instance transforms are uploaded to a contiguous wgpu::Buffer marked with Storage usage.  
   * A compute shader evaluates the camera view frustum against each instance bounding sphere.  
   * Visited instances write their indices directly into a wgpu::Buffer marked with INDIRECT usage.  
   * A single render pass executes draw\_indexed\_indirect, completing culling and rendering pipelines entirely on the GPU.

### **Module C: 3D Spatial Audio via Kira**

We run audio processing in a decoupled runtime environment using kira. Kira's thread-safe manager communicates with the main game thread using command queues, preventing rendering frame-time drops from inducing audio stutters.

  \+-------------------------------------------------------------+  
  |                        Audio Engine                         |  
  \+-------------------------------------------------------------+  
  |                         Kira Thread                         |  
  |  \[Active Sound Streams\] \-\> \[Spatial Mixers\] \-\> Output Hardware  
  |                                 ^  
  |                          Command Channel  
  |                                 |  
  |                        Physics Loop Thread                  |  
  |  \[Collision Event\] \-\> Calculate Impulse \-\> Play Spatially   |  
  \+-------------------------------------------------------------+

#### **Physical-Audio Contact Bridge (Rapier3D \-\> Kira)**

We bridge spatial audio triggering directly with the physics step to ensure real-world feedback from rigid body dynamics:

1. **Contact Interceptor:** During the fixed physics loop update, we drain the Rapier3D contact-force events channel.  
2. **Impulse Calculation:** For each contact event, we retrieve the dynamic impulse:  
   ![][image5]  
3. **Dynamic Mix Modulation:** If the impulse ![][image6] breaches a material sound threshold, we query the collided entities' material components (e.g., Rubber, Asphalt, Metal).  
4. **Trigger Command:** We dispatch a command to play the matching sample, scaling its volume and pitch proportionally to the kinetic energy of the impact:  
   ![][image7]

### **Module D: Physics Integration (Rapier3D)**

Rapier3D runs deterministically inside the simulation loop.

#### **Implementation Details**

* **Frequency Mapping:** Sim racing requires ![][image8] updates. The physics loop maintains a strict accumulator to feed Rapier PhysicsPipeline cycles.  
* **Visual Frame Interpolation:** To eliminate temporal aliasing, components maintain previous\_transform and current\_transform. Render interpolation computes ![][image9] based on time remainder:  
  ![][image10]![][image11]

#### **Pacejka Magic Formula Tire Simulation**

For realistic racing dynamics, vehicle tires use Pacejka's lateral and longitudinal slip calculations:

                \[Car Chassis Rigid Body (Rapier3D)\]  
                     |                      |  
            \[Front Left Raycast\]   \[Front Right Raycast\]  
                     |                      |  
            Calculate Slip Ratio   Calculate Slip Angle  
                     \\                      /  
                   \[Apply Pacejka Magic Formula\]  
                                |  
                     Apply Lateral/Longitudinal   
                     Forces back to Chassis body

1. **Suspension Raycast:** Every physics tick, we perform a downward shapecast/raycast to calculate suspension compression, wheel position, and velocity vector.  
2. **Slip Assessment:** We calculate the longitudinal slip ratio ![][image12] and lateral slip angle ![][image9] relative to tire orientation.  
3. **Pacejka Equation Execution:**  
   ![][image13]  
   Where ![][image14] is the calculated slip variable, and ![][image15] are coefficients representing track-to-rubber friction thresholds.  
4. **Force Application:** We translate ![][image16] and ![][image17] to world-space force vectors and apply them as direct external impulses back into the parent chassis rigid body.

### **Module E: Networking Architecture via Renet**

rustyengine uses an authoritative client/server architecture as a design target for future networking work, while the current repository focuses on subsystem logic and test coverage.

 Client Input   \+------------------------+      Authoritative State  
 \+------------\> | Client Prediction Loop | \<------------------------+  
                \+-----------+------------+                          |  
                            |                                       |  
                            v                                       |  
                \+------------------------+                          |  
                |   Verify State Tick    |                          |  
                \+-----------+------------+                          |  
                            |                                       |  
                       Error? | Yes                                 |  
                            v                                       |  
                \+------------------------+                          |  
                |  Rollback to T\_server  |                          |  
                |  Re-simulate Inputs    |                          |  
                \+------------------------+                          |  
                            |                                       |  
                            \+---------------------------------------+

#### **Client-Side Rollback & Prediction Buffer**

To support competitive racing and lag-free survival character controls under erratic latencies, renet states are handled via a rollback prediction loop:

1. **Input Registry:** The client records player inputs and active player rigid body states into a local circular buffer keyed by game tick:  
   struct PredictedFrame {  
       tick: u32,  
       inputs: PlayerInputs,  
       position: rapier3d::na::Isometry3\<f32\>,  
       velocity: rapier3d::dynamics::Velocity,  
   }

2. **Server Package Arrival:** The client receives a signed network snapshot from the server detailing historical state at frame ![][image18].  
3. **Error Evaluation:** The local client compares its buffered state at ![][image18] with the server's absolute state. If the positional delta exceeds a precision envelope:  
   * **State Override:** The client immediately overrides its current ECS entity states with the authoritative server values for tick ![][image18].  
   * **Re-Simulation:** The engine fast-forwards the simulation from ![][image18] up to the present frame ![][image19], running the physics engine in sub-stepped, head-less loops applying the cached inputs saved inside the prediction buffer.

## **4\. Multi-Threading Strategy**

To maintain maximum engine speed, we enforce structural multi-threading across thread bounds:

\[ Thread 1: Event & Window Loop \] \-\> Winit event listener, captures mouse/keyboard, handles raw OS buffers  
  | (Cross-beam channels)  
  v  
\[ Thread 2: Logic & Simulation (Targeting fixed step, e.g., 60-400Hz) \]  
  \-\> Custom ECS Systems (movement, game state, networks)  
  \-\> Rapier3D Physics Pipeline  
  \-\> Renet Network Manager (transmitting/parsing packets)  
  | (Double-buffered spatial data exchange)  
  v  
\[ Thread 3: Render Loop (Variable Framerate) \] \-\> Receives double-buffered visual states, draws using wgpu

### **Pre-Allocated Fixed-Memory Game Loop**

To keep heap allocations entirely out of our high-frequency loop threads, rustyengine should implement a strict memory design as part of the future runtime integration work:

1. **Startup Warm-up:** During initialization, all dynamic lists, entities, component vectors, and particle arrays call .reserve() or with\_capacity() to pre-allocate memory chunks on the system heap.  
2. **Double-Buffered Render Sync:** Frame data passed from the simulation thread to the render thread utilizes pre-allocated double buffers. Swap operations toggle pointers rather than performing structural copies.  
3. **Transient Arena Allocator (bumpalo):** Frame-level allocations that cannot be statically sized (such as immediate-mode debug lines, network serialization arrays, and particle update queues) use a pre-allocated bump allocation region. This region is cleared instantly at the end of the frame with a single pointer reset:  
   // Pre-allocated transient arena  
   let frame\_arena \= bumpalo::Bump::with\_capacity(16 \* 1024 \* 1024); // 16MB frame buffer

## **5\. Architectural Requirements for Targets**

### **A. Survival Game Architecture**

* **Asynchronous Chunking:** The quadtree-based terrain division streams data dynamically on separate thread-pool workers, preparing vertices in the background before submitting them to wgpu buffers.  
* **Sparse Set Optimization:** Low-density entities (such as dynamic loot nodes) do not waste memory, as empty spaces are represented cleanly as None inside sparse set buffers.  
* **Dynamic Lighting & Shadow Mapping:** Handled safely in wgpu through pre-allocated shadow map arrays and binding groups representing localized campfires, flashlights, or day-night sun positions.

### **B. Sim Racing Game Architecture**

* **High Frequency Physics System:** The physics loop and input poll ticks are scheduled at a super-high frequency (![][image20]).  
* **Deterministic Input Queries:** Raw input drivers (DirectInput/XInput) can be handled inside custom simulation systems, feeding immediately into vehicle dynamics updates.  
* **Kira Dynamic Audio Integration:** Sound nodes scale their pitch dynamic parameters dynamically based on car telemetry (RPM, exhaust backfire, wind turbulence, and tire slip friction rates).

## **6\. Implementation Roadmap**

The roadmap below reflects the current repository state rather than the original long-term fantasy scope.

Phase 1: Foundation Complete (modular crate layout, config parsing, input/audio/physics/gameplay/UI scaffolding, and passing tests)  
   |  
Phase 2: Runtime Wiring (connect the modules into a real application loop and window context)  
   |  
Phase 3: Renderer Integration (complete the wgpu-based rendering path and validate the first real draw path)  
   |  
Phase 4: Simulation Loop (add deterministic fixed-step simulation and real-time state exchange)  
   |  
Phase 5: Audio & Networking (complete the spatial/audio and rollback/network primitives already started in the codebase)  
   |  
Phase 6: Gameplay Features (terrain streaming, racing dynamics, and richer world logic once the runtime path is stable)


[image1]: <data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAFIAAAAZCAYAAACis3k0AAAD9klEQVR4Xu2YW4hNURjHx8zkGh5cpuZy9j4zU6NJaVyjNOOeB+FBklLIJRNPUojkBTUppVAe3EI8kVtIbqVkppAmMxgxY8hkaEoh8fvmrDWzfGefM2c24mH/69/e6/996ztr/fdea+99srIiRIgQIRyKi4tjHHK0LvB9vxLe9zyvwxxX6hyQi74T3oC1sKawsHCATrKgTn9yfgSwTuIcTwXEfhQVFY3Wtf45ysvL+zK4Cia1g2Mbgxyvc2Kx2DjiDbCqoKCgkPZpM6mNbp6Z+AlOc8UkeJ72DTcnCOT5pl4rzdyA+G0Tn6tjoUChyUxiVzwe93QsLKj5Gl6C12SwQUZKjMnMse3S0tJ+aE3wM+cjRCM+X/qXlJSMtHmMtVw0YpOsFoT8/PzhxqjnOiZAv2zqVOlYaFAwTsF9DPKADFTHw4Jam1IYmY3eARtZ+kOtSP5BM/nOJc6YjnL+rrtbJ2Spf4dblP4LrJHUaNAxwV8x0oK7Mo/Ce/iBk3C6jvcWaYwUM95LjP2u1Iq094pGv/XSZiyPaNd3d+vKayd2Vesu/qmRFnKXiAn8yDm4GClb52SCNEbKRRujL5aYY4ycIW3OP4qZbo7R33oplqxFGCP53ZmieYk7vt5LbE0fjFbhdO8dZN+i+GqKXOQHl2cFbNrpkM5IDfLi8Au8S7OP0WSiQUa2wGatu3D2yLRURi6CtxhvvmmPI+cbvNxV+DchS/EMfJ2XlzdIB1Ohl0YeI/+VPMEd7XMKI5thk9ZdhLwjV8MNpplD/B5sYUzDbE5Y5FJ4GcWewDu+85TNBI6RE3TMBXWXkPdBv8uJseiPXU2A1gprte4ipJGLGMNUOUffLHG4tKtTbyHvgRRaS5GnfuLJOVnnZIJMjDTLpykoB70ONgbon+A1rbsIY6SFrCBiXxnbFWnLKrGvZJkih86rKPIAbnXf38LAGsmDZaKOCahfRPyZvQsEtKcxuQXm/DBs7+7RadBAY8BuV9dw9sikCyFIY6QsaZm/fG35InC+jTnMVnnJKCsrG0xytSm+Qh4yOicMrJF+wMuzn/hKuQ+XuDr5W+k3T845zpL+7seC1DIGje3ulQzHyBc6JjBzTTISbYvpV201xnGkx32eQpV0ugAXZpmn5Z8CA9gug4qZ1xkVO0Ks3Uu8Ytz0Eq8bbWZyo2we7Rq412kfJ37ItlNBnrzGkDc6JkC/rsdGe4qXePWRT9BOL+QigvY/+eWXMYxJL81EhLLf3OK4TuLc8UOcWBJlj3bKZWPcGvT98LDc5aI58V8gd7quZ9jjnxYcz5r2Qy9xgYWyH8uHQ8o/SiJE+A9h9kh7G/fES7J0dI0IESJEiBAhwl/GT8t8hf8XAH1+AAAAAElFTkSuQmCC>

[image2]: <data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAD0AAAAZCAYAAACCXybJAAADWElEQVR4Xu2XS0hVURSGb2Zl0aSHZb7OFW+U9iCzBkEROCgKokKiSe8QpEGNDAcVDbJJSCEaZElFhr3MBkWRkYNUCKIsaNCDiApqIA1yEA2ivqVry7rbe5WuGEHnh8Xd61//Xo+zPQ8jkRAh/i8UFhbOCoKgNzc3N+bH8vPztxO7i3Vj1woKCgJfE41Gs4g1YR3oH2MVvsYCXQ32K4GV5+XlLUzAi7X4eUYFEjZIYpqf7/H7sB65KOIzzBH8L84XZGdnz2TfC9GKTyyP9Ru4o06TDOjOSV3y7vdj7F+rw3b6sVGDK7ucxD/8oTnR2XDfaWiLkafBfcSqHMG6VoY2Gmm4Er63tLR0guV9oDulg+3xY9RdqbF2PzZqkPQRVicFGHSe4fcqt9jTP6Che8Z/T/NXPU2Z7OWCrrK8Dzc0vzv92JgNTbGtJK2hwEEtPnjS+PXCyZ+r3QN3E+tDmxEduJdl3xmrIV+pNnzI8j7++tCxWGwSCbu4J6ckGbpNuSy7D+0V4TnFQmyZNlZvNTwQFwmP9rzlfaQyNH6H8r3BwF/pE/VvWF1CUKg60Hsp0dCs7ysXNzTcZS1SRGy1ruOGJl+x8s2W9+GGHsH8oXvYt5vlOPWvYz+lF6sbAk45E2FXRDcmGhr/tnL+0M3Cc5pz+V2hjcUNjV+key9a3ocbOvpnJ/3Srdm3UTWHrSYhEDXaK5Nk6EuacI7jlG8RPicnZ4Y8+FTTYDXuXUvek5b3kcrQLqe8Kll/IP4cN91qhgBRidyXlks0tGsIK/C0t4RnmSaFVdNkNfoaFH7YE0hlaAf4Vom7NwTrEl8zCDfgMPZOdNKI+kvtfvzOYODW6Ae61/itnmad7l1veR9m6F1+bLih4co1VqdUOuuOONFIMMUHT5qi0+C+8bvDccXFxRPheuEOOA7/GPYqos8H5aqwz2PxcaLPo0/Y28zMzKnCyeczfpvVjQg2nJYCFFpgeZraBN/Ncrz4xCvwn8FnGE0GXJdoxaeByXL6aLc5TTK4uugrE8T6P3Cwhx5/R/kyw9ViZ60uKWhsgwyhScS+Rr2vKx5Wa+BPBAMPtjqGmm7jAuHYV02+C2gasc2+xiJI8R8OaizRdR/WrvZUOGof9+uECBEiRIgQIf59/AaErG2QXgpDBQAAAABJRU5ErkJggg==>

[image3]: <data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAACsAAAAaCAYAAAAue6XIAAAC4klEQVR4Xu2WTYhNYRjHrxkfIURyud8fo9ENoy7ZSINIElaSlGShUIpENlYkG4sZZaFJXSnR5GsjQ4ksyEoU0awwLIyPZkFp/B7znts7f/fOPerexdT91dO55/9/zvM+73vOee+JRJqMA4rF4qR0On09n88n1atGKpWaTdxoa2ubqV5DodHzmUxmm+oB+FnVDJpdR9zlZ6t6DYFGdhK9qre3t89gAp14F4lB9QPwesg7rnpN7JZw8QniCgVucnxAPGb2xyKVZ9+C/57cjb7I+TL0T1aHeEV89X0fcldabiKRmKpeVWhoFxe+4cIjnLYEeiwWm4v2nOhjMlO8SyLJZHIL+lt+TvB1H/zbYzVr4PcTB1WvCIlniY80u0g9A28tMcyETopeIrp8TQnTLHUvkXNf9X8gaa81ks1mN6gXYCtKzm/ima9z/rrWioRp1vXwTfVR2EqSNEQ8Uc8nGo1OtwkRnwMtl8vNMo0aW/1cJUyzLNRqV2u+emVI6HJJ+9TzCYoRjzytwzTz/FwlZLNLrRbvwGL1AlpJGHBNVNwHA5jMGZdXfj4pvMI03uIlfq7imh3zFnOXUlaLcTrV+wtv+TTXwDCnE9UPsFuTHnlUfvmNeSvb4ecrYZqNx+MJq2V/EuqVIeGFW52qexx+t5v1qI3b/lpds2Eeg++q+9jtt1ocl6tXhoRTlkRsUs9A3+9mfEG94KXD26yej2v2h+o+1FhltViAeeqVcY/CO+Kl/yYWCoXJFDiN/pM4HKmy6eP1k3dIdR9y7hFDNjn1Ahh7d9rbaarCbYwy4DniYWbkL7bE8SrHo8QCzffB7yEuq85jNcfq4Q3airn4Yo3bKmo+ejdj3lK9rjDAHlsR+0RU73+gzlNih+p1xX34DLBa29ULC3dhITU+RMbYkeoGAx0g7qgeFvsusGdW9UZhn4m9DLpejVpw3RqipHpDsX2aQa+xT8bUq4a9hFzTZzuSek2ajHf+AI8lyAtaeSvZAAAAAElFTkSuQmCC>

[image4]: <data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAADsAAAAaCAYAAAAJ1SQgAAAD30lEQVR4Xu2Ye2iNcRjHzy7u12KN3d6z7Wg6kpjLf5iQhPmLQu5FCLlf/qBEUiibSFIaRjSXluUSifyDptbURPPPRvtHFKLE5+n83u3ZszOcbWeofevb+77f5/k9z/O7nvc9gUAX/h0Eg8GBnuedho9heUZGxjDr81eQn5/fjYKu5ObmZlpbW5GVlXWCmFPc/Tbua8LhcHdl3wYXNbXoJFDIcWZirtXbA2LehIfknthB7n9wmaRcEtHuwwKlxRckWwDLrN6RoJOTpbMwW+uZmZm5aPV5eXn9tP5HCIVC/Wm8C14gwXWu9+Ajlst2zEnWPxAZ3Tp8Z1iDICUlpS+22/hUumI/kyNF+6Dtgx+c/SMs13YB+S8S56zVBeg3aLPF6r+ErH8avqThZh4TfT0tLW0w2jN4l0J7qCYysnPQX3GboHULfHbDN65DLQojzlj0ymgzRF2LpaOwp7UJaLcc1un9/EvgfAi+JeBwaxNgmyKFkniP0UtgkdaiAZ/72dnZE7l+h/V20Mg7H+7VmgDf0fAwtwnkziGGZ304pUNSm9RobS2A0wpxJtB0a/MhxblCn2id5xq4TmsWqampffB5Jvdcz7vCVmofnotlMLSWnp4+yIsMZoHbs/s9s2d9oNd6kRXZOmQmcfoEH1ubhitYimzwtZycnAGiEaNQ+1owI7PxOyL3XMe4OM+1D89VXJK1FoycGeLr80sg+rkhOW5hL7Z6M+BQ5ApeZW0abglKwodKGyWanRELfI7Amf6zK0y2xFR5dgN+talF7CDGJWKUWl0jCYd3rhNRl4cPgh10fo37k0NlnGjsmZHa1wKfSjmV/WdizXCdvebs67hf39QidhDjFLxj9UZwyvZ2HfgRMEtIg+KGeJGl/k13TM3sKO2vgX96tCLQnroO53Mt4zrC+sQC2p8kzj2rNwMOVZKUonpZmw/sxeJDp3doXV4NXWdbXca0WWLbCWi3UNpKR+ELa48VxCj1frcVvMgJJ0kb95QG+hqxy8hZm39oYZtlbT6wlzAY460OkrG9crnPWGOsIEYFPGb1ZnBL+TWsluXq6/IDTScOoH+FmwKtvDRgq8Vvg9UFbuYb7BuTD9mnbrDa/TJP7S/haqu3ACOfSsKj8EEwctyXuNNtKxxq/TVkVuA5rclbkMRC/yKdge+jDYhbGQ16kNsCd6bINptgbR0KEiyTguUTz9o6C9RQSA3VVu9wuA+Hd8zcPGvrLMjKosMbrR4XkGytF+VLpTMg/16QuyrYykdCPCCfefJbOc0a4gzJW0HesDXEFfI7TeLLvFWlWVu8wGzuhEut3oUudOH/x0/8Cgz6CvMMMQAAAABJRU5ErkJggg==>

[image5]: <data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAmwAAAAwCAYAAACsRiaAAAAD0UlEQVR4Xu3cX4ilYxwH8CGkJHEzF3POnPeckdHcKBNq79jihrTaJXeKiFK0/m+72l2SC7lx40IpsRFF0cpyYxVXNjdTuJMSucDlSuP323ne9p2ns/7MOZODz6eenvf5/Z4z73vm6tv7zjtzcwAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAB2DwWC9rgEA8A/q9/uHckRQO1zG5/UeAABmxOLi4sEIbK/WdQAAtlEEsP05Rxg7UPf+65qmeT3n+O731z0AgJnRBrUIbofq3r9RfI+36tqZxN53y/xw3QMA2JKmac6PcPF9HJ5d97aqE9iernspXzJYWlrqt+vYv2duiuefprjW5/J6Y3xU91LU34xxorM+FdjiOz1yehcAwATyLliMW+r6JCIEHsx5XGCL3g1R35fHvV5vIef5+fkLNu+aHRG8dsc135ahbW5MqMx67Lmpsz6as8AGAExNBIzjo9HoorreKneXzjSeqvenQXkUOhgf2J6N+pXl+MW6P0vi+i4vh2fFNZ+I9R2d3j1ROxbjZM7xO1zMegS198sssAEA05HBq65Nqg1ydWCLEHNx1H5t13H8bbc/bf1+/6oMU3W9FaHrmrimW+t6Gg6HV8Rnj3dr+buKcWe7bjYeJ2/6jlH7oMyPd+sAAFu2TYFt7B22WO/qni/C0sud9raI891X11rR29uUtzprcW1vR//abq0Etq87e3ZGsLu+u6dzh+3Rbh0AYEsWFhZ6EUA+rOuT+oPAthbjp26t1HdkeIvxzKDcncu7cTGej/VLZc8v+Qi1PIr8MXpPxvE7MR7M8/V6vUvKvtei9lisLy0/5+7TZ/rr8prG1K6Lsb66unpuruM8H+e8srJyXkzn5LHABgBMTQlPv+U8Go0uq/uTaMa8dDDY+Huv9Rzj7qxl0Crz2vLy8oV5bW0vA1isj8bndpc9b0Tt3hI4Tz1ibc8V844Ye2N8l+utBLb47JH2WseNOPdXZd9dMb5oOn/bNigvHQz8Ww8AYJaNC2x/Jva+Uua1vLuWwajt5c/LIBTzzWXPkQxiJbCdzFrenSu9Y/nWacw/lPrfDmyT6AS2h+oeAMDMyMeVOUdoOVz3xhkOh1eXoLYn5p/zjcu86xfHL8T4NPdE75v2EWTUvsxgVB6Z5h27nbmv2fiXIZ/E2B/js/y5+YgyjndtPuP2iXO9l3NcywN1DwBgZkRYub07/59EYHsi5wiKN9Y9AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAmPsdUSPY79LbjQQAAAAASUVORK5CYII=>

[image6]: <data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAsAAAAZCAYAAADnstS2AAAAoUlEQVR4XmNgGElAXl7eC4gvAPFfIP4Ppc/IycmFoKuFAwUFhV0gxTIyMrrocigAqIATqPAbEN9Gl8MAQCtdQKYC6X50OQwAVNQGVeyDLocBgAqPAvEvcXFxbnQ5FAA0TRAaCnvR5TAAUFEwVHEtuhwGACq6C8Q/VVRU2NHlUABQkSLU1N3ocnAAjAAPoIITQPwdqvg5SAPQ/aHoakfBIAQA6nspbeQOBbsAAAAASUVORK5CYII=>

[image7]: <data:image/png;base64,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>

[image8]: <data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAFIAAAAZCAYAAACis3k0AAAELElEQVR4Xu2Ya4hVVRTHr87kkxQxHZjXufOAibEvNoooiPnI6EOIiPQhMBJSSQjqQ5YilBAliqn5xlBRKvWLimWkCCYSiA8sRDJKyFdJOIYgqMT4W3PWHtase+713mNiH84fFvfs//rvtfdeZ7/OzeUyZMiQoXy0trb2b2xs3BRF0Q3sMrarrq6u3ussWlpaRqL7u76+vtW5quGXYUex09hKNAOdpgf5fH4Amq4EOyN+fr9K8HU1NDQ852M9cWhnP29qaqrhdyb2D3aJBA/x2gD862VAJOJZx0usXTxWS5KwA5SPWk0S0OU1SdcpVif4f1D/y96XCgQaz+z5hEFH3pcGxJtMvGM89jHcW9rpFUbaA2bDWHx3RWMTyfMM4WS2Bo7Y7aobF7gk1NbWPqNt/uZ9AvhDGucF70sNAjYRcA2d3Cgd9f5KQP1FxLtHvHcCJ8taB3XJagPgj2NrRcMLbQs8MXbA/WW1uXip/4stdnwvhEQS46L3CR5LIgNkKRJ4OQ18iU3x/nJAvXelgyR0W+B4HqaJvGG1Atp7Ff5jNO/pwOyM/AnugtUL4Drxfe95iyeayIDm5uahMjAa2S8DherrNcXQ0dHxFHVet/sh5UmayF57mxxKcCcY9KCkRFK+Jcm0dZT/MyqyZAPSJJI+TNN+yoy/gB3Gbio32lSvDHr6ziPINzT4Ri5h0y4HeoJLp1+yPOX34eeqJimRUk5K5FXsiuctzB5Z0lwiZ8v+zp5dq+UONPexQz2BHxGyL+3BLtfU1Az2zlKQxETxG15qeV7SCLgTOT2UiiTyTpFEXomK7LcBKWfkPOxtLVbh/xG7yh4/PGjSoprAcwh2HjvuZ9TDIEuWeufsfhkAv4V4k0I5KZFwf8D9HMoBcNex0563SJnI2czGifIM/4H4sdd6KlWK9vb2fgRaQJBf8vHJOd5rykCfKD60vuC5yjrgRtPpry2XlEjKZ7BfrU55uZce9rxFmkQGkMwx+O7Rp++kLLcOWUFeVwpVVH6TIKewJfb+Vino4IfE2pczBxXlz/S3O2kl7HfR8bsV6wz1BTrLJQGfWt7D7JEFL0IQFU+kLGkZ/218eSF4XsqNZrrTFaKtre1pxAs1+Fw5ZLymEhBjFn04afdTmeWN8UU9EehX68Ds0n5ROPuxgH+cJuj5wCXBJLL7pXjoWAsSCbdY6y0MHP3YLrPU6gqQj68mB7GZOfM1khY0OiqKl558F8v14ZTudfLlstPrA/BtkAFIfcevxFaZ8k76vNlqkiAnr8TDrnmfAP6ItjfVcBOi+GCUa1p3LuQlgs7/6suvbNDoCh1AgZGAj7yegbyC76zR3US320j6Up4Pvw7bKtuCcMbfC/lH+NOC371aPhfFk0BMJkVXqT9KMmT4H0L3yDCNH2bfytLxMTJkyJAhQ4YMjxkPADN2lsRfjD8gAAAAAElFTkSuQmCC>

[image9]: <data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAA0AAAAaCAYAAABsONZfAAAA70lEQVR4XmNgGAVDDsjLyzsC8XogPgLEp4G4FCjMhK4ODuTk5GKAil4oKSmpQflKQP5rBQWFBhAfyK4Bst3hGoAcc6DgL6BCX7ggA1jjDKD4cyCTGUgfNTY2ZoVLAgXOATXeQiiHAKBYJVDuP1BzHJCeBJcAcgxBEkA8BUk9TC4HJAfUfEpZWVkWLgEUCIdKJCKpBwOgeBbUwCR0CSeoE1D8AwJAsTqogRYoEkABDqDEPaCCfpiYrKysFFCsTx4S/L+BciFAdQHAkJWDa4QG7xKgxEYgvQ9ILwSKuULlUoFiJ4D0Di0tLTa4plEwPAEAnfM7B4ktVYkAAAAASUVORK5CYII=>

[image10]: <data:image/png;base64,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>

[image11]: <data:image/png;base64,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>

[image12]: <data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAwAAAAaCAYAAACD+r1hAAAA30lEQVR4XmNgGAWDCsjJyWnLy8s/VlRU1APxxcXFuRUUFDqAYneAcuXo6hmAEtVA/FtFRYVPVlZWB8jeC8S1QPwfqCEVXT1IwzagiaeA2B2oYKeysrIsuhpkwAzU8B6InwI13AI6Sx9dAQoAuRtkNRAvA+I5QHwDpBFIO6KrBQOQG0EalJSU1KBCLCBnAcWegDhAzQIyMjJCcA1AgYUwSRgA8hcB8XUouwSIA5El7wE1zYQLMIANmQkU3wdkMgNt26GlpcUGlgAxoJ71R9YAFDME4hdAfAKIPZHlRsEQBABdTjRiDspi/AAAAABJRU5ErkJggg==>

[image13]: <data:image/png;base64,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>

[image14]: <data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAwAAAAZCAYAAAAFbs/PAAAAs0lEQVR4XmNgGAVDGygoKCTIy8sfB+LzQLwYyDcH4pVycnIrgHxvbIqrgExmGRkZTiD7PxCfk5aWFgZqOAgyCF3DJiDFCGVrgDQAFeaCNINsAIrZo2hABkDJTKgGJXQ5GGABKmgFKgjR0tJiA7LfAPFcmCTQgAAgToSrBko6Qd2cA5TIgJreBpJTUVERBbJ3qqur88I1AAX5QO4EeQ4kCcQuQE3HgHg10IBGkMfhikfB0AUAwBosO78TZnYAAAAASUVORK5CYII=>

[image15]: <data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAF4AAAAaCAYAAAA+G+sUAAAEb0lEQVR4Xu2YXYhVVRTH75gmhaFSEzVfZ75ycrQCh+zzwYzKgvIl6qUetAKNIKOHCvpgTIsIiuyljwcpM3qQqEiy8aE0KR0l8EXFyiIi7QNHixxQhvz9PevUbnXvPfvaPT2dHyzOPeu/19pr77PPPufcSqWkpKSkpKSkOHp6euYnSbIZ+xH7E9tj56esq6vre/t9tY9tBOvnZWwndgD7HNvW2dl5eWtr6zT6+XhoaGiKj6vH4ODgmcRtIs/XVvuhsHY739fd3b3Mx8ai2CSdE+WXfWa5tyfpOI4FWo+Pz4WgdRZ8YehnYvrwjWOH29vbzw21GAYGBs4hdr1yM0lrOc7DPUka5zM5fxfbzQBHXGg0xC63/HdX0YZtXA97LZaOjo6zNAfk2u81aEG7BzvR6MI5BYHfknjU+wUD2qHitTq9Vg/yzSBuF/YbOa7xusA/R7lp+5jXYkls0ZDjAq8JtC9UQ1tb23lei4HY65SfWl/0Wgb6l96XCxM6V4mx571GZ0Om7eX0DK/Xg5gN2AQ5rvdawCTaHFc/XohE8T9pcr2QgbZSY+DC3OS1GIhdbXNwc+br7e2dxfn9+q2VTu73/46IhKAVlnhh5rP98z58P3Pcoo7CmDzIucgG+4bXPLTbwKHF+2Mg/xXqhxqf8VoG2rM2vqVeiyFJn0fj3DFn61wTzfk6+r7Tt20Ikmy0wrTfvoA9YfvxGMmHffsYknSL0YSc7kqOgj4eVz/1VnM2Pmq51Wt52DPqhM3PP6zW1haFriJJxinqPa+ReIF1+qbX6tHX19dpxR32WrOhj09VY7YaPTa+o9gEdZ3v9Tx0sWwsKzMfv6/CdoXtGobXvBvt6q3wmkjS1ybpF3utFhR7gxX7kdc85H3U+2Jh+5uepM+HTV7LQLvDalnvtRiSdAdQ/F/bsC3Ip4M2a+hnTnYeBUmeU2IuwGVeE2gHpfNK1e+1WtD2IsWQe6vXQujzUtq95v2xMNjb1Q/HR7xmTE7S74axhifGIHY39kd/f/9Urwn8reibvT+XJH1wHPR+QbF3aWDVEutOYYIv8f4MYj7Bfufdv8NrQhdaeavp9DuTi7akkvMWRfzrqo+2V3oNWvC/hH6EfNd6Ma9+wdtem/Ln3FGv6K7y/rpkiRN3G+qpTbIH8R/juEPtQh3/bIs7VOujgYENdKVfvdu054caE7IM7Z1a79XEvGUTWnX7y6DNV9gYPyeHfnvV+xDbrg/AUBMx9QvVaXX8azvUHYD2APZNxT4Ic6HxPBWFHbcCfkiCT2062o+NYMsrVVadHlK0+0Xx+gbwegbxM7BhbJSJ3sLxVY5PEXeLbxuCvgqbwN72mv5eSNK76YjVrod/WPso9gG/l9aa1Lz6iX/I8n1nfewN+zD7VVq1i1I4dLxG+7n3NwPdDd0R3wD/hSLrLxQK3+h9zYJJX0z+e72/mRRZf2Fwi97GtvGk9zcJPRhHTue9O5aC6y8OVsuqinuoNQtyL2TiF3l/Mymy/pKSkpKSkpL/iZM9S2MnmB8g6gAAAABJRU5ErkJggg==>

[image16]: <data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAABYAAAAaCAYAAACzdqxAAAABmUlEQVR4Xu2UMSxDQRjHi4YEi0jTtK+v16YvEa0JiTAamA1iIgaxGGwisTGIUQwiFklNRkYmaYhFJBILG2G1YDDU77zvxfUSbR8m6T/55777vv/9797Xu0YiDfwPKKVG0un0BeM7LMNreAxL8FFyARP2+poQs3Iul3PtGhtPUnshbLZrVZFMJttZ+ArP7ZogSu3STtZEJpMZl09dCXKpVMohv6zjeDzeQW33a0WdwGBDjPsl1URuh/mCqQsNDM7EuIL0tmBr6waLu5R/I46MXIH5g6kLDQwm5HRLQS6bzfYw3zNk4YHplnz6cJCLxWKdjuN0m7rQwPAGPhNG7ZoJvmAG3QEclfk0P/CmrfsEol59WgSHds2E67qDaIbQFuGczjHuYz5fIdS7Kv+lXUkbNG+/66u8xhY0T0GLiO/RD1jS8JDDnOiYx+Mp/4lXbV9d0C3ghNs6ZlwMNvk1MErQ51Pl36I7uGZrfoJmTGcZo/l8vlX3mrs+ZotCg1vRh9mbnHqKsRT5i/56ntdGX9cxXIVF/TdgaxqowAdYSmZDafPqNgAAAABJRU5ErkJggg==>

[image17]: <data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAABYAAAAbCAYAAAB4Kn/lAAABZklEQVR4Xu2TvS8EURTFx9rGRyMyGTtfbyZTiKxKR4VGrdD5D1QaEeIP0Ek0BIliQ0+pQwiFbCLRaEWtoRFZv2fvxOwrdney0cic5OTdd+95Z97cmWtZBf4HlFIzYRjesX7CBnyEF/AavkouZcU83xFi1kiSJDBrPHiJ2jthyay1heu6gxz8gLdmTVCm9mAmOyKKogV51c005/u+R35dx47jDFE7+D3RJTDYFuMpSfWR22e/ktXlBgY3YtxCels1tV2DwyOq+UecZ3JV9i9ZXW5gsCi3W0tzcRyPsz/OyPLjL413pafTac627WHP80azutzA8Am+EZbNWhZo5uApPIEb/DWrrDVT9wMKE/q2iM7MmoF+dEc6oEXL6C9lGustKhLzqjnCdW0sfG7T15IeGB1guoN2yxT0DIzv4azEY0Y5H4IgmNRvBCvwi9sPaFPV62RinGByJaNfo2V7rIdWhw9eoIBlfQPckGBXyrUI1QAAAABJRU5ErkJggg==>

[image18]: <data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAADIAAAAaCAYAAAD1wA/qAAACnklEQVR4Xu2WSWgUQRSGG5cYiRuSYWCYmepZwugICg6KBLeDihAlCIrgxeWieBBEUFAGcogmUdGICoKXYI4aDyIIDoIeDG4QxYNCQHIJIogIokJCGL9nv5aiHUEwh27sH36q6n+vltf1qqodJ0aMGP8PstnsTmNMHU7Bt7AGv6g2CZ/g80jt9Uwmsy44RijA4u7Caj6fX+hrruvel0Wn0+k2X8vlcsvRvqVSqVZfCw2KxeICFnfP1giiWRYMR21dgPY6qIUCLHo/i9tja6TRZk2rq7bO7sxFq9laaCCpw67MsTUWe1oCkbNj6+VyuQl9qa2FGiz2sQTCbi0K2iKDUqk0nyAmCeJZ0BYpkE7bNa16grZIgZ24pIFsCtoiBYIYgd/lCg7aeAhT6P3Yuwl0gHqn9mmHt2l3wV6kmbRv6Ac5Dp+ir6E8gvYZPkgkEvMoz8H3kgX0mUX9MrxC+yJv1mr6uLQfSppTnoXP/+rcase6+cMVK4vCZZ/UKTfS3pZMJlvwH5P3SH0GsO2lOsN4Z20r5QF5TNXeA69pfS323VLH5xjtM1LnJi0afcOwbzD6CFNW/Xl+A4aELFw5roEI34lWKBQyvi/tHfArg9+Rr+t4X7EDfpJJlNexH1T/CRn/12TOz7+DFegftG+1UqnMVt9hOGSNU5N3S4KlPmKP8c/gFybLQrboRK+M9xV3UY4HfQUSSKNUQH+B3kl5ytJeoh2y/QS6a9N7gzJgPwMvkzoTd9Dukv8z6h/91OErLqZ9WH2mGgWCdhTbGGdula9JUPCW32aeE/JQM+76aQ+Eic4b79BVGbxPDr/qK42XmvJHcFLTtdt4KTqI7xJ7HOmH/sbWHO+CkPFvGt1p+smZHTTeBXHBbXABxYgRI0Y08QPbLLoEvpBuQAAAAABJRU5ErkJggg==>

[image19]: <data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAADsAAAAaCAYAAAAJ1SQgAAACvElEQVR4Xu2XXYhNURTHJ/Kdr6Kb+3XO/eBylZf75CsyREkeKEVDI0XKAy8muiIhXkQpyjxIk1K8GF8zeWBSmhTlRSMzZUJRFEWZdP3W3LXNbrvG9fFwjs6//q21/2vtfdc6Z5+zz21oiBAhQoQgI51Or/c8rwK/wqewE35UbQA+IOeuxiupVGqxu0ZoQAPtsJzNZicbzff9DmksmUzONFomk5mH9ikej08zWqiQz+cn0cBNW6PRsdIUfGbrArQnrhYa0FgzDWy0NbbsCt3CZ2yduzwOrdPWQgXZptzdMbZGQ0ekWXmWbb1YLI5Gn2NroQcN3ZdmuetT3Nh/hUKhMJFGB2i0240FEdQ6gx1YdPW6wMQ1uoWPubEggpuygXrXuXpdYPIpbXa5GwsaqNWn1sd/3CwTH8HPcvy4MQH6SuJ3sC3YVqSR2C59xmfLRcL2eHqcMd6F/wHtHLYdu72WJrn4O+ElxofI2azzT+raO+BB/Cty3mstzfpbbbDM+T/+e6G/AhPlSlW8nxwvfDnliD2ngKm8wadLHlpcYvjvpVnxiTd51tmN3oF2FDbKY1JLgyXmdJk5+H0wo/47uE39rfCilSdfe/XdWVO08qVXbVbYK1oul0uZXApsoahb9nwDcl+ZZvE3eVaz4qedY8zVmHsc7SEsKy/D+Zr7hty56sva16x16m/2d6BF3HB1AXr/cM0SWzuU/aPG+DRss3MM0F87a1+3YoPNJhKJJJ+7s4Zm/SXkA0R/ePD85WovMt/OaD3m6mMPyDY18xjfdpt1NfwlrN0bi8UmyBh/GTkL1X87zIW8Klq6+q5YavR/AhZezaLd2MO+vlhU38P4Ana/WvnXdAJ/C4W8wN6DqyS3liZAa/Krf0D2YXcjjUDby/gL9ixs1Ob6JFd/dwFshedLpdIos1aECBEiRAgqvgHerdaInxC1QgAAAABJRU5ErkJggg==>

[image20]: <data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAFIAAAAZCAYAAACis3k0AAAELklEQVR4Xu2YXYhVVRTHTSe1QkyspuZr3/mQgaGimkIKIsFURMFeBiHNR4WCDKlIoQ98iCSJoIIEI0NDqB5EhBGNQF+MdMyPQdNIITStoKmRejCG6fefs/Zt3d09N7mSDnH+sDh7/fd/77P2Ovvr3gkTChQoUODK0d3dPa2trW1LqVQ6HULYRfl9rCfVCa2trQ+h+RT7AuuXn0ga4Ndb/QC2saWl5aZEUwbvnIpmtIodVj3P7VXqRnnv3Wlf1x0KlsQ95Xwl4Xes3evQ9MFdZPCzTbcGO0dxUtSoL2wbxQYlCdup/mJ9HtCVLEkXcBuq1O+3+oVp3bgAAd5pAQ5GjvILxq133F3YMPqXHHdAus7Ozlb51C0x/46o0cwWF5Ofh6amptvsnd+ldQJ8v/UzJ60bF7BE/ogdctxaC3qD4zaIY0k1RQ5/IYl61Wk+Ul/RN2ipj2DrEr4CMZH0cTqtE8Z9IoWurq7bCXBq9EO2B1YEjX8K+zn61YD+GJqTKQ83RN2elPf4XyTSgxm2mIAvE/CqyFG+VYPABrEn0ezjeQB+bW9v741RB/erkhl9x18MOUs2op5EEsfjFpdm/ElsL/aLcfe75tcOnKz38PLPlAxse2Nj4y2xjuXcacEpIe9BNXR0dEynfJDBfBh10uQk8nzIDqVcuD2ypiWJ7NNHjdsN5V40f2L95Y5rgQYrEf+EnaD8StzsqwHN0ylXC7oK0WZACdFMFKerhg1kpLm5eWbU8u7VNrj75FP+IyeR57CzKe9R54xciT1r7qSQHX7nfYy50L2NxjuwFTRapOXF84x/QQRcibrnU/7fQLMnFDTv2CJfX9wSecbr8JeZ7nX5PL/HP+41pruADaS8R52J7CO2R1W2PCjGZeVGtUCDTRwOUzzX3t5+b8gOgzUKSJwGj3Z/rdlq0H1PM6p8dyPADgtqREu8p6dnsvlfunaKZanxWu4a7GHsW68x/jdsb8p71JPICMb6IHWXiXu3fGZkiw7QVFcBOnoj5QQL5BNsOGS/KC6FK/g69Pe2JePNyLFfzjJOs22GuJAl6eu/W44lfLlpxmYk5c3YkNcQ183S5MUd4fbIf3wIIeQnUkv6EHaJupIIyi8zueYnukr4e1w1aLbS0cMkozmtqwYbfDkZAvEssKC/cpyWzrA/peGes7Zz5fOcJ59BBKeZbQl6IHLV4BJZsX1EhJxEwq2zds9ETluSZqnX/edoy046ncZjA9WSCNkX1sHxWNTp9zLcINxruDeUsivRQewd+VGHvxF7y/lb0W6Kfh7cPvxDWifAf676+NGMeyRkVx/9BB2LQR8RDPmPec2gr8zLj2JHsOPYBwTSnepsyX+MfaODBVudasBE+luF5l1sM5oXxaWiiNJV/GkR7IdDyGLXHVKm/Xi01h8lBQoUKFCgQIECBQrUg78AQBSOGN94mcAAAAAASUVORK5CYII=>