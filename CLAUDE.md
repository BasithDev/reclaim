<!-- Reclaim: root guide for AI assistants (Claude Code). Read this first in every session. -->
# Reclaim — AI guide

Reclaim is a desktop disk-usage analyzer and cleaner for **Linux and Windows** (not macOS): Rust engine +
Tauri 2 + React/TypeScript UI. It's built entirely with AI, so keep this file and the docs accurate.

## Read before working
1. `docs/SPEC.md`: what the app does and how it looks.
2. `docs/CONTRACT.md`: the fixed interfaces between engine, cleaners, Tauri host and UI.
3. The `CLAUDE.md` in the folder you're working in.

## Layout
```
crates/reclaim-core/      scanning engine (disks, tree, sizes, snapshots, diff); OS code only in platform/
crates/reclaim-cleaners/  junk detectors + cleaners + safety layer (the only code that deletes anything)
src-tauri/                Tauri host: commands, events, app state, settings
src/                      React UI: shell/, components/, pages/<id>/, store/, lib/ (api, types, format)
docs/                     SPEC, CONTRACT, progress/
```

## Rules
- **Every file starts with a header comment** (1–3 lines) saying what the file is for. No exceptions.
- OS-specific filesystem code lives only in `reclaim-core/src/platform/` and in cleaner modules gated with
  `#[cfg(target_os = ...)]`. Everything must compile on both Linux and Windows.
- Nothing deletes files except `reclaim_cleaners::safety`. Default to Trash/Recycle Bin.
- The UI talks to the backend only through `src/lib/api.ts`. Rust and TS types must stay in sync
  (`types.rs` ↔ `src/lib/types.ts`).
- JSON is camelCase, sizes are bytes, timestamps are Unix ms.
- Use the components in `src/components/` instead of one-off styled elements. Every view needs
  loading/empty/error states.
- pnpm only. Reclaim is its own pnpm root (`pnpm-workspace.yaml` stops pnpm adopting `~`).

## Commands
```bash
pnpm install                 # frontend deps
pnpm tauri dev               # run the app
pnpm tauri build             # release bundles (deb/AppImage on Linux, nsis/msi on Windows)
cargo check --workspace      # compile-check Rust
pnpm exec tsc --noEmit       # type-check UI
```

## Git
Personal repo `github.com/BasithDev/reclaim` (public). Commit as `Basith <abdulbasith7599@gmail.com>`
(set in the repo's local config). Never use a work identity.
