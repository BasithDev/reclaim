<!-- Guide for AI working in src-tauri (the Tauri host process). -->
# src-tauri

Thin glue between the engine crates and the UI. Command list and event names: `../docs/CONTRACT.md`.
- `lib.rs` registers plugins, state and commands. Commands go in `commands/*.rs`, state in `state.rs`,
  settings in `settings.rs`.
- Heavy work runs in `spawn_blocking`. Commands return `Result<T, String>`.
- Capabilities (permissions) live in `capabilities/default.json`.
