<!-- Guide for AI working in reclaim-core (the scanning engine). -->
# reclaim-core

Pure Rust library: no Tauri, no UI. Public API fixed by the skeleton signatures; see `../../docs/CONTRACT.md`.
- `types.rs`: serde types sent to the UI (mirror: `src/lib/types.ts`).
- `disks.rs`: list drives. `scan.rs`: parallel walker + in-memory arena tree + queries.
  `snapshot.rs`: SQLite history. `diff.rs`: growth between snapshots.
- `platform/{linux,windows}.rs`: allocated size, file ids (hard links), device ids, skipped paths, link
  detection. Only this folder may contain OS-specific filesystem code.
- Never follow symlinks/junctions. Count hard links once. Permission errors are counted, never fatal.
- Memory matters: millions of entries. Use an arena (Vec + indices), not `Rc` trees.
