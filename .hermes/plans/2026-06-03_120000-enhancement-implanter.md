# Implementation Plan: Codebase Enhancements

**Date:** 2026-06-03  
**Status:** Draft / Not started  
**Scope:** Non-breaking refactors plus small new behaviors across core, physics, audio, input, networking, terrain, renderer, and config subsystems.  
**Constraints:**  
- No CLI or user-facing behavior changes in this batch; keep runtime entry (`main.rs`) untouched.  
- Additive-only where possible; deprecate names rather than remove them until later.  
- Preserve and add tests so `cargo test` still passes at every step.

---

## 1. Core / ECS hardening

**Suggested order:** start here — low risk and visible benefit.

1. Rename `World::generate_entity()` → `spawn_entity()` with `#[deprecated]` forwarding alias for compatibility.
2. Add `World::alive_entity_count()` and `World::registered_component_count()`.
3. Add `World::clear()` that empties all stores and frees generations/free_list buffers.
4. Add `World::retain_components(ids: &[Entity])` implementation strategy:
   - Keep the first N alive IDs by compacting `generations` once per call.
   - Skip on first pass; track as foundation for future performant reset.

Files likely to change:
- src/ecs/world.rs
- src/ecs/mod.rs

Tests to add:
- `test_spawn_then_destroy_then_spawn_generations`
- `test_alive_count`
- `test_retain_does_not_remove_retained_entities`

---

## 2. Physics validation helper

**Goal:** prevent silent bad-step behavior in `PhysicsState`.

Tasks:
1. Add `fn validate_dt(dt: f32) -> Result<f32, String>` central helper (reject NaN, <= 0.0, > 1.0 unconditionally).
2. Add `PhysicsState::try_step(dt: f32) -> Result<(), String>` that calls validate_dt then `step`.
3. Option A (preferred): derive `PhysicsConfig::dt() -> f32 = 1.0 / tick_rate_hz` to centralize the relation.

Files:
- src/physics/state.rs
- src/physics/mod.rs (maybe add helper module)
- src/config/physics.rs (add `dt()`)

Tests:
- step with dt=0.0 → error
- step with dt=NaN → error
- step with valid dt → succeeds

---

## 3. Input shaping (deadzone / response-curve)

**Goal:** racing-grade steering feel without breaking existing 0/1 semantics.

Tasks:
1. Add `fn smooth_step(x: f32, deadzone: f32, sharpness: f32) -> f32` in `src/input/mod.rs` or new helper module.
2. Add config binding support:
   - Optional `InputBindingConfig { deadzone: f32, curve: CurvePreset }`.
   - CurvePreset: Linear, SCurve, Exponential.
3. Apply in `InputMapper::get_source_value` only when analog input is detected; keep digital path fast.
4. Expose bind-and-shape API: `bind_analog_with_curve(action, source, scale, preset)`.

Files:
- src/input/mod.rs
- src/input/mapper.rs
- src/input/source.rs (expand enum if needed)
- src/config/input.rs

Tests:
- deadzone clamps small values to 0
- curve per preset monotonic between [-1, 1]

---

## 4. Audio bridge impulse accuracy

**Goal:** replace single-number `impulse_magnitude` with per-pair estimation.

Tasks:
1. Change `PhysicsAudioBridge::process_collision_events` signature to accept either:
   - `Fn(handle1, handle2) -> Option<(PhysicsMaterial, [f32;3])>`, OR
   - A `CollisionImpulseProvider` trait that bridges Rapier `RigidBodyHandle`s to materials + positions.
2. Remove hardcoded constants (`max_impulse = 50.0`) and derive scale from the actual collision flags/impulse when possible.
3. Keep backward-compat wrapper accepting scalar magnitude.

Files:
- src/audio/bridge.rs

Tests:
- collision events filter by threshold; zero-threshold plays once.
- provider trait returns None when materials missing → no sound.

---

## 5. Networking protocol hardening

**Goal:** make wire format safer and reduce bandwidth.

Tasks:
1. Add `#[repr(u8)]` to `NetworkChannel`; ensure serialization uses the repr value explicitly.
2. Add `ClientMessage::Heartbeat { tick: u32 }` and `ServerMessage::Heartbeat { tick: u32 }` for liveness.
3. Add delta snapshot concept (optional in first iteration):
   - `ServerMessage::StateDelta { tick, changed_bitfield, packed_positions, packed_velocities }`.
   - Bitfield byte alignment: multiple of 8 entities per delta message.

Files:
- src/network/protocol.rs
- src/network/rollback.rs (consume delta)
- Add tokenizer/serializer helper for compact u8-bitfields

Tests:
- serialize/deserialize roundtrip for each variant
- delta contains correct count and ordering

---

## 6. Terrain generation abstraction

**Goal:** replace sine mock with pluggable heightmap source.

Tasks:
1. Define `trait HeightmapFn { fn height_at(&self, x: f32, z: f32) -> f32; }`.
2. Implement `SineHeightmap` and `FlatHeightmap` wrappers.
3. Add `TerrainGenerator::with_heightmap(chunk_size, resolution, heightmap)`.
4. Make heightmap optional config field in config layer next step.

Files:
- src/gameplay/terrain.rs (new trait)

Tests:
- sine heightmap matches old behavior exactly for known coords

---

## 7. Render context robustness

**Goal:** safer resize and config encapsulation.

Tasks:
1. Add `fn validate_config(&self) -> Result<(), wgpu::Error>` that checks:
   - width/height > 0
   - present_mode supported by current adapter
   - format supported by surface
2. Call from `resize` and after adapter re-enumeration (when `AdapterInfo::backend` changes).
3. Add `RenderContext::recreate_swapchain()` helper for adapter-lost recovery.
4. Replace public field reads with accessors if internal state can no longer stay public.

Files:
- src/renderer/context.rs

Tests:
- resize to zero → result is clamped (still non-zero) and config valid

---

## 8. Config ecosystem improvements

**Goal:** stronger typed access and developer ergonomics.

Tasks:
1. Rename `DEFAULT_ENGINE_DEFAULTS` → `DEFAULT_ENGINE_CONFIG`.
2. Add typed config read:
   - `ConfigSection::get<T: FromStr>(&self, key) -> Result<T, ConfigParseError>`.
   - Move repeat-parse logic from callers into one place.
3. Add duplicate-key warning detection in INI parser (log warning once in debug builds).
4. Add optional TOML parser module feature (`cfg(feature = "toml")`) with same typed-read surface.

Files:
- src/config/parser.rs
- src/config/mod.rs

Tests:
- unknown key handled gracefully or per chosen policy
- duplicate keys warning verified under log capture

---

## 9. Batch dependency audit

**Goal:** add foundational crates for logging and error handling.

Tasks:
1. Add `tracing = { version = "0.1", default-features = false, features = ["std"] }` and a one-line init in a new `src/logging.rs` stub used only behind `debug_assertions` feature guard.
2. Add `thiserror = "1"` and create `src/error.rs` with `EngineError { io, config, physics, audio, render, net, input }` variants, each wrapping source where possible.
3. Add `smallvec = { version = "1.11", features = ["union"] }` as an optional `performance` feature for ECS stores.

Files:
- Cargo.toml
- src/logging.rs
- src/error.rs

---

## 10. Telemetry scaffolding

**Goal:** one-stop timing API to validate latency claims later.

Tasks:
1. Define `struct SubsystemTimer { label: &'static str, samples: ringbuf::RingBuffer<f32> }`.
2. Expose `SubsystemTimer::begin()` / `end()` and aggregate min/avg/max.
3. Instrument Physics and Render from `state.rs` and `context.rs` under a `profiling` feature flag.

Files:
- src/logging.rs or a new module.

---

## Cross-cutting risks and mitigations

1. API churn from deprecations — keep forwarding aliases until the next minor.
2. Unvalidated config values remain plausible — prefer early error over silent clamp where input is user-controlled.
3. Performance regressions in ECS borrow path — add `cargo bench` plan before profiling once Entity insertion rate rises.

---

## Expected verification
- `cargo test --all` passing after every subsystem chunk.
- Optional: `cargo clippy --all-targets -- -D warnings` passes cleanly (add warnings-as-errors once the surface is stable).
- Future: integration harness under `tests/integration_tests.rs` once a real loop exists.
