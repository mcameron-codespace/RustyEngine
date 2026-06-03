# Plan: Async Logging System + Async Asset Manager

## Goal
Add two engine subsystems to `rustyengine`:
1. A logging system that outputs to both the console and a rolling file log.
2. An asset manager that loads/tracks textures, meshes, and audio files with fully async file I/O.

Both features must avoid blocking the main thread on file I/O and must remain additive/non-breaking to the current crate layout.

## Current Context / Assumptions
- Baseline is 29/29 tests passing in `cargo test`.
- `main.rs` is still a placeholder, but is the crate root: module declarations must be added there.
- Runtime is sync Rust today. No async executor is required for logging.
- Asset loading benefits from async I/O; Tokio is acceptable only for the asset subsystem and its tests.

## Proposed Approach
1. Keep logging async-agnostic: a dedicated `std::thread` plus `crossbeam-channel` for fire-and-forget file writes. Console output stays sync/simple.
2. Use Tokio only for asset I/O, with explicit minimal features: `rt-multi-thread`, `fs`, `io-util`, `sync`, plus `macros` and `rt` in dev-dependencies.
3. Define log levels and small logging macros.
4. Define typed asset handles via a typed enum or generic handle registry. Avoid `Box<dyn Any>` and avoid DashMap.
5. Add graceful shutdown for both subsystems.

## Files Changed / Current State
- `Cargo.toml`: added `thiserror` and `tokio` (runtime + dev runtime)
- `src/main.rs`: added `pub mod logging;` and `pub mod assets;`
- `src/logging/mod.rs`: module declarations
- `src/logging/level.rs`: `LogLevel` enum
- `src/logging/logger.rs`: console + file logger with background thread writer (runtime removed from logger.rs)
- `src/logging/rolling.rs`: size-based rollover helpers
- `src/assets/mod.rs`: module declarations
- `src/assets/types.rs`: `AssetKind`, `AssetPath`, `LoadedAsset<T>`, `AssetHandle`
- `src/assets/manager.rs`: in-progress; first pass had two issues:
  - write removed background `ConsoleOnlyWriter`/`FileSinkWriter` definitions, leaving `Logger::init`/`console_only` tested logic broken
  - asset manager first pass also tripped async/lint warnings
- `src/logging/tests.rs`: basic tests added
- `src/assets/tests.rs`: in-progress tests

## Next Actions
1. Restore/replace logger writer thread implementation in `src/logging/logger.rs` so it compiles with current LogSink API.
2. Simplify/reduce the first-pass `assets/manager.rs` to a minimal unsafe-light compile path from compile-lint blocks.
3. Run `cargo test`, iterate until green before handing to OpenCode.


## Files Likely to Change
- `Cargo.toml`
- `src/main.rs`
- `src/logging/mod.rs`
- `src/logging/level.rs`
- `src/logging/logger.rs`
- `src/logging/rolling.rs`
- `src/assets/mod.rs`
- `src/assets/types.rs`
- `src/assets/manager.rs`
- tests alongside each new module

## Tests / Validation
- Keep 29/29 tests green after each small batch.
- Add dedicated logging tests: creation, write, console output, rotation behavior, oversized-message policy, shutdown flush.
- Add dedicated asset tests: load from disk, cache hit, eviction, async completion.

## Risks, Tradeoffs, and Open Questions
- Tokio is introduced only for assets; logging uses blocking std thread + channel to avoid async overhead where it doesn't help.
- The runtime is still not embedded in the engine loop. Logging and assets can initialize their own runtime/thread in tests, but a real engine loop will eventually need to coordinate this.
- Rolling logic must explicitly handle oversized single writes to avoid infinite roll loops.
- Typed handles sacrifice some generic uniformity; the acceptable compromise is a typed enum over `Box<dyn Any>`.
