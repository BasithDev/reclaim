<!-- Reclaim: project README for GitHub visitors. -->
# Reclaim

**Find out what is eating your disk, then reclaim the space safely.** A fast disk-usage analyzer and junk
cleaner for **Linux and Windows**, built with Rust + Tauri.

- Visual explorer (treemap / sunburst) and largest-files list
- "What grew since last scan" history
- One-click cleanup of logs, caches, old snap revisions, Docker, `node_modules`, `target/`, temp files…
  with previews first and Trash/Recycle Bin by default
- Detects deleted files that a running process still holds open
- Dark/light theme

> Status: v0.1 in development.

## Build
Requires Rust, Node + pnpm, and the [Tauri prerequisites](https://tauri.app/start/prerequisites/).
```bash
pnpm install
pnpm tauri dev     # run
pnpm tauri build   # installers
```

## License
MIT
