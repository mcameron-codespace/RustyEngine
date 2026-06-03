# AGENTS.md

## Project overview
- This repository is a Rust engine prototype named `rustyengine`.
- The main architecture and roadmap are documented in [docs/design/Rusty-Engine-Design.md](docs/design/Rusty-Engine-Design.md) and [docs/design/Rusty-Engine-Implementation-Plan.md](docs/design/Rusty-Engine-Implementation-Plan.md).

## Working conventions
- Prefer small, focused changes in the relevant subsystem (`src/config/`, `src/input/`, `src/audio/`, `src/renderer/`, `src/physics/`, `src/network/`, `src/gameplay/`, `src/ecs/`, etc.).
- Keep the existing modular layout and avoid introducing unrelated framework changes.
- Use the shared config parser path in `src/config/parser.rs` when touching configuration behavior.

## Build and test
- Build and test with `cargo test` from the repository root.
- If you need to validate only config-related work, run the narrowest relevant test target available after changes.
- Current verified baseline: 29/29 tests passing, `cargo check` green.

## Key directories
- `src/config/`: configuration defaults, parser helpers, and config tests.
- `src/input/`: input mapping/action abstractions and analog curve shaping.
- `src/audio/`: spatial audio, collision-to-audio bridge, and `AudioSystem`.
- `src/renderer/`: wgpu render pipeline scaffolding and `RenderCommand` queue.
- `src/physics/`: Rapier3D simulation loop, interpolation helpers, and physics state.
- `src/network/`: mock UDP transport and prediction/rollback buffers.
- `src/commander/`: typed `CommandBuffer<T>` and central `CommandDispatcher` for decoupled producer/consumer communication.
- `src/gameplay/`: terrain generation, vehicle dynamics, and `GameplaySystem` command consumption.
- `src/ecs/`: modular Entity Component System abstractions and world management.
- `src/ui/`: immediate-mode UI primitives and boundary tests.

## Notes for agents
- The typed command buffer infrastructure is defined in `src/commander/`, but the main runtime (`main.rs`) is still a placeholder and not yet integrating subsystems.
- Keep command wiring additive and non-breaking; do not change behavior outside the targeted subsystem in a single batch.
- Prefer linking to the existing design docs instead of duplicating their contents.
