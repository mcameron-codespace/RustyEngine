# rustyengine Enhancement Plan

## Goal
Transform `rustyengine` from a working prototype/subsystem toolkit into a runtime-connected engine with safer ECS APIs, a real command-driven render path, improved audio/physics robustness, stricter config validation, and missing tests — while keeping `cargo test` green after each batch.

## Current Context / Assumptions
- Codebase compiles and all 26 tests pass.
- Subsystems exist (`config`, `input`, `audio`, `renderer`, `physics`, `network`, `gameplay`, `ui`, plus custom ECS).
- `src/main.rs` is still a placeholder.
- Design docs target survival + sim-racing gameplay; implementing all of that scope is not realistic in a single change set. Enhancements below focus on architectural strength, runtime wiring, API hygiene, and incrementally testable steps.

## Proposed Approach
Work in small, build-passing batches:
1. Runtime wiring + main loop scaffold
2. ECS/World usability improvements
3. Renderer lifecycle + command encoder
4. Audio shared math + AudioWorld scaffolding
5. Config validation + bounds checking
6. Thread-safety preview + deprecation cleanup
7. Added tests + final `clippy` sweep

## Step-by-step Plan

### Batch 1: Runtime wiring (main.rs + app loop)
- Add `src/app.rs` with a minimal `EngineApp` struct holding:
  - `World`
  - `SimulationLoop`
  - `RenderContext`
  - optional `PhysicsState` / `AudioWorld` placeholders
- Add a basic `EngineApp::run(window: Arc<Window>) -> Result<(), ...>` that:
  - creates `RenderContext`
  - creates `SimulationLoop` (configurable tick rate from `config::engine`)
  - runs a simple run-loop with event polling, simulation ticking, and a single-frame render stub
- Leave `main.rs` delegating into `EngineApp::run` instead of `println!`.
- Validation: `cargo test` still passes; `cargo build` succeeds.

### Batch 2: ECS usability improvements
- Change read-only query APIs to take `&self`:
  - `borrow_store`
  - `try_borrow_store`
- Add `World::despawn_components(entity)` as an internal compaction helper (reserve-aware).
- Add archetypal query convenience: `World::with<A, B, F>` where the closure gets `(&StoreA, &StoreB)`.
- Validation: `cargo test` covers `World`; add/update tests for read-only borrow and the new query helper.

### Batch 3: Physics / interpolation robustness
- Extract quaternion math into `src/physics/quat.rs` with explicit `normalize` and `slerp` helpers used by `interpolate_rotation`.
- Keep `Transform` stable, but call into helpers instead of inline grid math.
- Add property-style tests: `interpolate_position/rotation` for alpha=0,1, and edge cases (opposite quaternions handled).
- Validation: `cargo test`.

### Batch 4a: Renderer ownership/docs improvements for `context.rs`
- Document `RenderContext` lifetime/ownership contract.
- Add expose helpers only if compile-only safe:
  - `window_size()` returning current configured logical size.
  - `format()` shorthand.
- Keep existing constructor/resize surface behavior unchanged so validation remains low-risk.
- Validation: `cargo build`.

### Batch 5: Audio shared math + AudioWorld
- Add `src/audio/math.rs` re-exporting right-vector calc and attenuation helpers used by both `emitter.rs` and future audio runtime.
- Add `src/audio/world.rs`:
  - `AudioWorld` owns listeners + emitters
  - `update(listener, dt)` returning per-emitter playback requests (volume, panning)
- Wire these into `EngineApp` conceptually; keep default stub for now.
- Validation: `cargo test` on `audio::tests`.

### Batch 6: Config validation
- Add a `config::validate()` convenience that checks:
  - tick rate positivity and bounds
  - window size minimums
  - audio listener defaults
  - optional feature flags consistency
- Update `config::engine` to call validation on load or expose typed errors.
- Validation: `cargo test` on `config` and `parser` modules.

### Batch 7: Deprecation cleanup + warnings sweep
- Remove `World::create_entity` once callers use `spawn_entity`.
- Ensure `MeshViewStub` and similar dead-code warnings are removed regionally.
- Fix new warnings introduced by batches using `cargo fix --tests` as applicable.
- Validation: `cargo test` with warnings-as-errors via `RUSTFLAGS="-D warnings"` if project prefers strict build.

### Batch 8: Testing gaps
- Add tests for:
  - `App`-level simulation -> render interpolation (headless)
  - Audio math helpers + AudioWorld update sequence
  - Config validation errors
  - Render command drain (can use mock device if possible)
  - ECS query helper + `borrow_store(&self)` read-only
- Validation: full test suite, 100% green.

## Files likely to change
- `src/main.rs`
- new: `src/app.rs`, `src/renderer/encoder.rs`, `src/physics/quat.rs`, `src/audio/math.rs`, `src/audio/world.rs`
- `src/ecs/world.rs`
- `src/ecs/mod.rs`
- `src/renderer/context.rs`
- `src/renderer/bridge.rs`
- `src/audio/emitter.rs`
- `src/config/engine.rs`
- `src/config/mod.rs`
- `Cargo.toml` (only if new optional features are needed)

## Tests / validation
- Canonical pass: `cargo test`
- Optional strict build: `RUSTFLAGS="-D warnings" cargo build`
- After each batch: run `cargo test` and confirm green before moving on.

## Risks, tradeoffs, and open questions
- Runtime wiring touches many modules at once; keep it behind feature-gated modules or a single `app` crate target to avoid regressing existing unit tests.
- Changing `borrow_store` to `&self` is a breaking public API change; acceptable for a prototype-phase crate.
- Renderer command drain cannot be fully unit-tested without a mock GPU backend; we may need minimal gpu-test scaffolding or compile-only tests.
- Removing `create_entity` could break external tests/crates depending on it; verify first.
