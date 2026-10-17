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

## Download
Get the latest installer from **[Releases](https://github.com/BasithDev/reclaim/releases/latest)**:
- **Ubuntu / Debian:** `Reclaim_*_amd64.deb` → `sudo apt install ./Reclaim_*_amd64.deb`
- **Other Linux:** `Reclaim_*_amd64.AppImage` → `chmod +x` it and run it
- **Windows:** `Reclaim_*_x64-setup.exe` → run it (SmartScreen: *More info → Run anyway*, the app is unsigned)

## Releasing (maintainers)
Bump `version` in `src-tauri/tauri.conf.json` and `Cargo.toml`, then push a tag: `git tag v0.2.0 && git push origin v0.2.0`.
The `release` workflow builds both platforms and publishes the Release.

## Build
Requires Rust, Node + pnpm, and the [Tauri prerequisites](https://tauri.app/start/prerequisites/).
```bash
pnpm install
pnpm tauri dev     # run
pnpm tauri build   # installers
```

## License
MIT
