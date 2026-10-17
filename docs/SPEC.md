<!-- Reclaim product + design spec: what v1 does and how it looks. Read with CONTRACT.md before writing code. -->
# Reclaim v1 — Spec

Reclaim is a desktop app for **Linux and Windows** (not macOS) that answers "what is eating my disk?" and
then safely gets the space back. It's a personal tool in a public repo. It has to be fast, honest about
sizes, and never delete anything it shouldn't.

Stack: Rust engine (`crates/`) + Tauri 2 host (`src-tauri/`) + React 19 / TypeScript / Tailwind v4 UI (`src/`).
Charts: ECharts (`echarts-for-react`). Animation: Motion (`motion/react`). Icons: `lucide-react`. State: `zustand`.

---

## 1. Engine — reclaim-core (A1)

- **Disks:** list real drives/partitions (skip proc/sysfs/tmpfs/devtmpfs/squashfs/overlay/snap mounts), with
  used/free/total, filesystem, removable flag, system flag, SSD/HDD where detectable. The system drive comes first.
  On a dual-boot Linux machine, the Windows NTFS partition shows up as a normal disk.
- **Scanner:** a parallel walk (rayon or jwalk) that:
  - measures **allocated** size (st_blocks*512 / GetCompressedFileSizeW) and apparent size;
  - counts hard-linked files once (tracks (dev, inode) only when nlink > 1);
  - never follows symlinks, junctions or reparse points;
  - stays on the root's filesystem unless `crossFilesystems` is set, and always skips `/proc /sys /dev /run`;
  - honours `excludedPaths`; permission errors are counted, not fatal;
  - throttles progress callbacks to about 100 ms and can be cancelled through an AtomicBool;
  - keeps the result as a compact in-memory tree (arena: Vec of nodes with parent/child indices, names as
    `Box<str>`). It must hold several million entries without trouble. Directory sizes are aggregated bottom-up.
  - keeps the top-N largest files (a min-heap) and a per-category type breakdown.
- **Queries:** `subtree(path, depth, maxChildren)` returns children sorted by size; the leftovers are summed into
  `otherBytes`/`hiddenChildren`. `largest_files(limit, category)`. `directory_sizes(maxDepth)`.
- **Snapshots:** SQLite (`rusqlite` with feature `bundled`). Tables: snapshots (meta + disk used/total at that time)
  and dir_sizes (snapshot_id, path, bytes) to depth 6. `diff` points at the *deepest* folders that explain the
  growth, not every ancestor. For example, if only `~/.cache/pip` grew by 3 GB, report that one folder, not
  `~`, `~/.cache` and `~/.cache/pip`.
- **Platform split:** `platform/linux.rs` (libc / `std::os::unix::fs::MetadataExt`) and `platform/windows.rs`
  (`windows-sys`). Nothing else uses `cfg(target_os)` for filesystem details.
- Suggested crates: `rayon` or `jwalk`, `sysinfo` (disks), `rusqlite` (bundled), `libc` (unix), `windows-sys` (windows).

## 2. Cleaners — reclaim-cleaners (A2)

Each cleaner implements `Cleaner` (info / is_available / preview / clean). `preview` never modifies anything.
`clean` re-evaluates and removes through `safety` only.

**Linux**
| id | what | method | admin | risk |
|---|---|---|---|---|
| journald-logs | systemd journal, vacuum to 200 MB (`journalctl --vacuum-size=200M`; size via `--disk-usage`) | command | yes | safe |
| rotated-logs | `/var/log/**/*.gz`, `*.1`, `*.old` | files (permanent) | yes | safe |
| snap-old-revisions | disabled snap revisions (`snap list --all`, `snap remove NAME --revision=R`) | command | yes | safe |
| apt-cache | `/var/cache/apt/archives/*.deb` (`apt-get clean`) | command | yes | safe |
| flatpak-unused | `flatpak uninstall --unused -y` | command | no | safe |
| coredumps | `/var/lib/systemd/coredump/*` | files | yes | safe |
| trash | `~/.local/share/Trash` (empty it = permanent) | files | no | careful |
| thumbnails | `~/.cache/thumbnails` | files | no | safe |
| user-cache | rest of `~/.cache` (excluding thumbnails and caches owned by other cleaners) | files | no | moderate |

**Windows**
| id | what | method | admin | risk |
|---|---|---|---|---|
| user-temp | `%TEMP%` files older than 1 day | files | no | safe |
| windows-temp | `C:\Windows\Temp` | files | yes | safe |
| recycle-bin | empty the Recycle Bin (PowerShell `Clear-RecycleBin -Force`) | command | no | careful |
| windows-update-cache | `C:\Windows\SoftwareDistribution\Download` | files | yes | safe |
| delivery-optimization | `C:\Windows\ServiceProfiles\NetworkService\AppData\Local\Microsoft\Windows\DeliveryOptimization\Cache` | files | yes | safe |
| crash-dumps | `%LOCALAPPDATA%\CrashDumps`, `C:\Windows\Minidump` | files | partial | safe |
| thumbnail-cache | `%LOCALAPPDATA%\Microsoft\Windows\Explorer\thumbcache_*.db` | files | no | safe |
| shader-cache | `%LOCALAPPDATA%\D3DSCache`, NVIDIA/AMD shader caches | files | no | safe |

**Both OSes (developer + apps)**
| id | what | method | risk |
|---|---|---|---|
| docker | `docker system prune -f` (size from `docker system df`) | command | moderate |
| node-modules | `node_modules` dirs under home in projects untouched for 30+ days (don't descend into node_modules) | files | moderate |
| rust-targets | Cargo `target/` dirs (next to a Cargo.toml) untouched for 30+ days | files | moderate |
| npm-cache | `~/.npm/_cacache` (`npm cache clean --force`) | command or files | safe |
| pnpm-store | `pnpm store prune` | command | safe |
| yarn-cache | yarn cache dir | files | safe |
| pip-cache | `~/.cache/pip` / `%LOCALAPPDATA%\pip\Cache` | files | safe |
| cargo-cache | `~/.cargo/registry/cache`, `~/.cargo/git/db` | files | safe |
| gradle-cache | `~/.gradle/caches` | files | moderate |
| go-build-cache | `go clean -cache` | command | safe |
| browser-caches | Chrome/Chromium/Edge/Brave/Firefox **cache** dirs only (never profiles, cookies or history) | files | safe |
| vscode-caches | VS Code / Cursor `CachedData`, `CachedExtensionVSIXs`, `logs` | files | safe |
| old-downloads | files in Downloads not modified for 90+ days (preview must be reviewed) | files | careful |

**Insights:** `deleted_open_files()` on Linux scans `/proc/*/fd` for links ending in " (deleted)" and reports
pid, process name and size. On Windows it returns an empty list.

**Safety (non-negotiable):**
- `is_protected` blocks filesystem roots, OS dirs (`/bin /boot /etc /lib* /usr /var/lib` except known cache
  subdirs, `/proc /sys /dev`, `C:\Windows` except known cache subdirs, `Program Files*`, `ProgramData`
  except caches), the home dir itself and its top-level dirs, and anything outside the cleaner's own roots.
- Default = Trash/Recycle Bin (`trash` crate). Permanent only when asked, or for system files that cannot go
  to Trash (logs, apt cache), which are handled by the command or marked permanent in the description.
- Elevation: Linux `pkexec <cmd>`; Windows `powershell Start-Process -Verb RunAs -Wait`. A cancelled prompt
  returns `ElevationDenied` and is not counted as a failure.
- Never follow symlinks while deleting. Use timeouts on external commands (60 s; docker/snap 5 min).
- Suggested crates: `trash`, `walkdir`, `dirs`, `which`.

## 3. Tauri host — src-tauri (A4)

- Implements every command and event in CONTRACT.md. Long work (scan, preview, clean) runs in
  `tauri::async_runtime::spawn_blocking` so the UI never freezes.
- State: finished scans in `Mutex<HashMap<String, Arc<ScanResult>>>` (keep the last 3); running scans' cancel
  flags; `SnapshotStore` at `app_data_dir/snapshots.db`; settings JSON at `app_config_dir/settings.json`
  (defaults when missing).
- After a scan completes and `saveSnapshots` is on: save a snapshot, then emit `scan://complete`.
- `.github/workflows/build.yml`: build (not test) on ubuntu-22.04 and windows-latest with `tauri-action`,
  uploading artifacts. Triggered on push to main and on manual dispatch.

## 4. UI / design (A3 shell and design system, A4 data pages)

**Feel:** modern, calm, simple to use, but detailed when you look closer. Think Linear / Raycast / macOS
System Settings. Plenty of whitespace, one accent colour, numbers front and centre.

**Design tokens (A3, in `src/styles/global.css` as CSS variables + Tailwind `@theme`):**
- Accent: emerald/teal (`#10b981` family), meaning "space reclaimed". Danger red, warning amber, info sky.
- Light: background `#f7f8fa`, surfaces white, borders `#e5e7eb`, text `#0f172a`/`#475569`.
- Dark: background `#0b0d12`, surfaces `#12151c`/`#181c25`, borders `#232836`, text `#e6e8ee`/`#9aa3b5`.
- Category colours, fixed per FileCategory and used everywhere (charts, chips, legends):
  video violet, audio pink, image amber, document sky, archive orange, diskImage red, code emerald,
  executable slate, database indigo, log lime, other gray.
- Radius 10–14 px, soft shadows in light mode, 1 px borders in dark mode. Font: Inter (`@fontsource-variable/inter`),
  with tabular numerals for all numbers.
- Theme: system / light / dark, set via `data-theme` on `<html>`, with a smooth 200 ms colour transition.

**Shell (A3):**
- Left sidebar (collapsible to icons): logo + "Reclaim", nav items with icons (Dashboard, Explorer, Large
  Files, Cleanup, History, Settings), and a small disk-usage meter at the bottom.
- Top bar: in-app menu bar **File** (New scan, Scan folder…, Cancel scan, Quit), **View** (each page,
  Toggle sidebar, Theme ▸ System/Light/Dark), **Help** (Keyboard shortcuts, About Reclaim). Also the page
  title and a theme toggle button.
- Keyboard shortcuts: `Ctrl+1..6` pages, `Ctrl+N` new scan, `Ctrl+O` scan folder, `Esc` cancel scan,
  `Ctrl+B` toggle sidebar, `Ctrl+Shift+L` cycle theme, `Ctrl+,` settings, `?` shortcuts dialog.
- Page transitions: fade and slide 8 px (Motion `AnimatePresence`).
- Toasts (bottom-right), confirm dialog, About dialog (version, OS, links), shortcuts dialog.

**Shared components (A3, `src/components/`, names/props fixed in CONTRACT.md):** Button, IconButton, Card,
StatTile (label, value, sub, icon, trend), AnimatedNumber, ProgressBar, RingGauge, Badge, Chip, Checkbox,
Switch, Tooltip, Skeleton, EmptyState, PageHeader, Dialog, ConfirmDialog, Toast (`useToast`), Tabs,
SegmentedControl, DropdownMenu, `useChartTheme()` (ECharts theme from current tokens), CategoryDot,
`categoryMeta` (label + colour + icon per FileCategory).

**Pages:**
- **Dashboard (A4):** hero with a big animated ring gauge for the system disk (used/free), Scan button
  (scans `defaultScanRoot` or home) and a "Scan folder…" picker; cards for other disks; while scanning, a live
  panel (files/s, size counted ticking up, current path, cancel button); after a scan, StatTiles (scanned size,
  files, folders, duration), a type-breakdown donut, a top-10 folders bar chart (click → Explorer at that path),
  "Reclaimable" summary (sum of cleaner previews, loaded lazily, click → Cleanup), a deleted-but-open files
  warning banner when present, and "Grew since last scan" (top 5) when a previous snapshot of the same root exists.
- **Explorer (A4):** ECharts treemap of the current folder (2 levels, coloured by category/size), with a
  switch to sunburst; breadcrumb navigation; click a folder to drill down, back button to go up. A side list of
  children with inline size bars and percentages. Row actions: open in file manager, reveal, move to Trash
  (confirm). Before any scan: an EmptyState with a Scan button.
- **Large Files (A4):** table of the largest files with category filter chips (with totals), search box,
  sortable size/modified columns, relative dates, and per-row reveal / move to Trash with a confirm. Multi-select
  with a sticky footer showing "N selected · X GB" and a bulk Trash button.
- **Cleanup (A3):** cleaners grouped (System / Developer / Applications / User). Each row has a checkbox, name,
  description, risk Badge, an admin shield icon, and the size (previewed lazily in parallel, with a Skeleton while
  loading). Rows expand to show preview items (path, size, label) or the exact command. A sticky footer shows
  the selected total and "Clean selected". The confirm dialog lists what will happen (Trash vs permanent, admin
  prompts). Afterwards, a results panel with an animated "Freed X GB" counter and per-cleaner errors.
- **History (A3):** an area chart of disk used over time (snapshot disk_used_bytes) plus a list of snapshots
  (date, root, size, delete). Pick two snapshots (default: the latest two of the same root) to see the growth list:
  diverging bars, +/- deltas, a "new" badge, click → Explorer if that scan is loaded.
- **Settings (A3):** Appearance (theme SegmentedControl), Scanning (default scan root with a picker, cross
  filesystems, excluded paths list with add via folder picker and remove), Cleanup (delete mode Trash/Permanent,
  confirm before clean), History (save snapshots, open data folder), About (version, OS, GitHub link).

**States:** every data view has loading (Skeleton), empty (EmptyState with an action) and error (inline message
+ retry) states. Numbers animate when they change. Respect `prefers-reduced-motion`.
