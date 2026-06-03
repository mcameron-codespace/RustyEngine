# ECS Gameplay & World Layers (Conceptual Blueprint)

## 1. Environment & Atmosphere Layer
*Goal: Make the world feel alive without hardcoding logic into entities.*

| Component | Data Fields | Purpose |
| :--- | :--- | :--- |
| `TimeOfDay` | `current_time` (float 0.0-24.0), `day_length` (float) | Global singleton component tracking the world's time. |
| `WeatherVolume` | `type` (enum: rain, snow, clear), `density` (float), `bounds` (AABB) | Defines an area where weather effects are active. |
| `AudioZone` | `ambient_track_id` (u32), `volume` (float), `bounds` (AABB) | Triggers background music or ambient SFX when the player enters. |

**Systems:**
- `EnvironmentSystem`: Reads `TimeOfDay`, interpolates the `Light` component's color/intensity (e.g., orange at sunset, dark at night), and updates the skybox.
- `AudioZoneSystem`: Checks player `Transform` against `AudioZone` bounds. Pushes `PlayAudioEvent` or `FadeAudioEvent` to the Event Queue.

## 2. Gameplay & Interaction Layer
*Goal: Define how the player and world objects behave and react.*

| Component | Data Fields | Purpose |
| :--- | :--- | :--- |
| `Health` | `current` (float), `max` (float), `is_invincible` (bool) | Tracks vitality. If `current <= 0`, adds a `Destroy` or `Ragdoll` tag. |
| `Interactable` | `prompt_text` (String), `action_type` (enum: open, pickup, talk), `cooldown` (float) | Marks an entity as something the player can interact with. |
| `Inventory` | `slots` (Vec<ItemID>), `capacity` (u8) | Holds references to collected items (pure data, no UI logic). |

**Systems:**
- `InteractionSystem`: Runs a raycast from the `Camera`'s `Transform`. If it hits an entity with an `Interactable` component, it pushes a `ShowPromptEvent` to the UI layer.
- `DamageSystem`: Listens to `CollisionEvent`s. If a projectile hits an entity with `Health`, it subtracts the impact force from `current`.

## 3. AI & Behavior Layer
*Goal: Drive NPC movement using the existing Physics/Transform systems. No custom AI movement logic!*

| Component | Data Fields | Purpose |
| :--- | :--- | :--- |
| `AIState` | `state` (enum: Idle, Patrol, Chase, Attack), `target_entity` (Option<EntityID>) | The current "brain" state of the NPC. |
| `PatrolPath` | `waypoints` (Vec<vec3>), `current_index` (u8), `speed` (float) | Defines a looping route for the NPC to follow. |

**Systems:**
- `AISystem`: Reads `AIState` and `PatrolPath`. It does **not** move the NPC directly. Instead, it calculates the desired direction and writes directly to the NPC's `RigidBody.velocity` or `Transform.position`. The existing `PhysicsSystem` and `RenderSystem` handle the rest seamlessly.

## 4. Visual Polish (VFX) Layer
*Goal: Add juice to the game without bloating the core renderer.*

| Component | Data Fields | Purpose |
| :--- | :--- | :--- |
| `ParticleEmitter` | `effect_id` (u32), `spawn_rate` (float), `lifespan` (float), `is_active` (bool) | Spawns temporary visual entities (like sparks or dust). |
| `Decal` | `texture_id` (u32), `fade_duration` (float), `age` (float) | Projects a texture onto a surface (e.g., bullet holes, blood). |

**Systems:**
- `VFXSystem`: Reads `ParticleEmitter`. Spawns new, short-lived entities with `Mesh` and `Transform` components. Increments `age` on `Decal`s and tags them for `Destroy` when `age >= fade_duration`.