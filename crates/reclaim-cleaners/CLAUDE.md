<!-- Guide for AI working in reclaim-cleaners (junk detection and safe removal). -->
# reclaim-cleaners

Pure Rust library: no Tauri, no UI. Public API fixed in `types.rs`, `cleaner.rs`, `registry.rs`,
`safety.rs`, `insights.rs`; see `../../docs/CONTRACT.md`. The cleaner catalogue is in `../../docs/SPEC.md` §2.
- One cleaner per file in `src/cleaners/{linux,windows,common}/`, registered in `registry.rs`, gated with `#[cfg]`.
- `preview()` must never modify anything. `clean()` removes only via `safety::remove_path` / `safety::run_command`.
- `safety::is_protected` is the last line of defence. Make it strict and keep it strict.
- External commands: always a timeout, never a shell string built from paths.
