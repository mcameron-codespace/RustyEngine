# rustyengine

A modular Rust game engine prototype. The intended stack in the design docs includes ECS style abstractions, `wgpu` rendering, `rapier3d` physics, `kira` audio, and Renet oriented networking, but the current repo focus is clean compiler verified modules and typed command pipeline scaffolding.

## Current state

- Tests pass: `cargo test` is green against the current workspace.
- Binary entry point is still a placeholder in `src/main.rs`.
- Active structural work lives in typed command buffers and subsystem stubs:
  - `src/commander/`: `CommandBuffer<T>` and `CommandDispatcher`
  - `src/audio/`: `AudioSystem`, `PhysicsAudioBridge`, and spatial helpers
  - `src/renderer/`: render context, pipelines, and `RenderCommand`
  - `src/physics/`: simulation loop, interpolation, and physics state
  - `src/network/`: prediction buffer and rollback logic
  - `src/gameplay/`: terrain generation and gameplay commands
  - `src/config/`, `src/input/`, `src/ecs/`, `src/ui/`

## Development

Run tests from the repository root:

```sh
cargo test
```

If you are working only on config-related behavior, run the narrowest relevant test target after changes.

## Notes

- Main architecture and planning live under [docs/design](docs/design).
- The codebase is still in an active compile/narrow refactor phase, so verify the specific area touched before claiming completion.
