# AGENTS.md

## Project
- Rust engine prototype (`edition = "2024"`, single crate, **no `lib.rs`** — modules declared in `src/main.rs`).
- Architecture: `docs/design/Rusty-Engine-Design.md`. Development plans live in `.hermes/plans/`.

## Build & test
- `cargo test` from root. All tests are inline `#[cfg(test)]` unit tests (no `tests/` dir, no dev-dependencies).
- Baseline: **29 tests passing**, `cargo check` green. No CI, no rustfmt/clippy config.

## Architecture highlights
- **`main.rs` is a placeholder** — it prints `"Hello, world!"` and declares all 10 subsystems as `pub mod`. No runtime wiring exists yet.
- **Commander** (`src/commander/`): `CommandBuffer<T>` bounded ring buffer + `CommandDispatcher` (holds audio/render/gameplay buffers). The decoupling mechanism for producer/consumer communication.
- **Config** (`src/config/parser.rs`): custom INI parser. The file `config/engine-defaults.ini` is embedded as a string constant for first-run generation.
- **ECS** (`src/ecs/`): sparse-set with runtime borrow-checking via `RefCell`. Not a framework — hand-rolled.
- **Physics** (`src/physics/mod.rs`): `SimulationLoop` with accumulator-based fixed timestep; `Transform` stores dual `previous_`/`current_` state for interpolation.
- Key dependency versions: wgpu 0.19, rapier3d 0.18, kira 0.8, winit 0.29, nalgebra 0.32.

## Conventions
- Touch config behavior only through `src/config/parser.rs`.
- Keep command wiring additive and non-breaking within a single batch.
- Link to design docs instead of duplicating their contents.
