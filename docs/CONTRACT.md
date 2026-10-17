<!-- Reclaim interface contract: the fixed boundaries between engine, cleaners, Tauri host and UI. -->
# Reclaim — Interface Contract

These interfaces are fixed so that four people (agents) can build in parallel. **Don't change a signature
in this file without the manager's approval.** If you need something extra, add it next to the existing API
(a new command or function) and say so in your progress report.

## Rust: reclaim-core (`crates/reclaim-core/src`)
The public API is defined with exact signatures in the skeleton files: `types.rs`, `disks.rs`, `scan.rs`,
`snapshot.rs`, `diff.rs`, and the function list in `platform/mod.rs`. You may add private items and
fields, but not change public signatures.

## Rust: reclaim-cleaners (`crates/reclaim-cleaners/src`)
Fixed in `types.rs`, `cleaner.rs` (trait `Cleaner`), `registry.rs`, `safety.rs` and `insights.rs`.
Individual cleaners go in `src/cleaners/{linux,windows,common}/*.rs`.

## Tauri commands (src-tauri) ↔ `src/lib/api.ts`
JSON is camelCase. Rust command params are snake_case (Tauri maps them). Errors are returned as `String`
(the `Display` text); the UI shows them.

| command | params | returns |
|---|---|---|
| get_app_info | – | AppInfo |
| list_disks | – | DiskInfo[] |
| start_scan | options: ScanOptions | string (scanId); runs in background |
| cancel_scan | scan_id | () |
| get_scan_summary | scan_id | ScanSummary |
| get_tree | scan_id, path: Option<String>, depth: u32, max_children: u32 | TreeNode |
| get_largest_files | scan_id, limit: u32, category: Option<FileCategory> | FileEntry[] |
| list_cleaners | – | CleanerInfo[] |
| preview_cleaner | cleaner_id | CleanupPreview |
| run_cleaners | cleaner_ids: Vec<String>, permanent: bool | CleanupResult[] |
| remove_path | path, permanent: bool | u64 bytes freed (via reclaim_cleaners::safety) |
| get_deleted_open_files | – | DeletedOpenFile[] |
| list_snapshots | – | SnapshotMeta[] |
| delete_snapshot | id: i64 | () |
| diff_snapshots | older_id, newer_id, limit: u32 | GrowthEntry[] |
| get_settings | – | Settings |
| save_settings | settings: Settings | () |

`Settings` is defined in Rust in `src-tauri/src/settings.rs` (A4), matching `src/lib/types.ts`. Defaults:
theme "system", excludedPaths [], permanentDelete false, confirmBeforeClean true, crossFilesystems false,
defaultScanRoot null, saveSnapshots true.

**Events** (emitted by src-tauri, wrapped in `events` in api.ts):
- `scan://progress` → ScanProgress (throttled to about 100 ms)
- `scan://complete` → ScanSummary (after the snapshot is saved)
- `scan://error` → ScanError { scanId, message, cancelled }

## Frontend boundaries
- `src/lib/types.ts`, `src/lib/api.ts`, `src/lib/format.ts`: shared and stable. Only add to them.
- `src/store/ui.ts` (A3): `useUiStore` → `page`, `pagePath`, `setPage(page, path?)`. A3 may add fields
  (sidebar, theme), but these three stay as they are.
- `src/store/scan.ts` (A4): `useScanStore` → `status: "idle"|"scanning"|"done"|"error"`, `progress`,
  `summary`, `scanId`, `error`, `startScan(root?)`, `cancelScan()`. It subscribes to the events once.
  The shell's File menu and shortcuts call `startScan`/`cancelScan`, and the sidebar meter reads it.
- `src/store/settings.ts` (A3): `useSettingsStore` → `settings`, `load()`, `update(partial)` (persists via
  api.saveSettings and applies the theme).
- Pages are default exports from `src/pages/<id>/index.tsx`. The shell imports them by those paths.

## Shared components (A3 builds; everyone uses) — `src/components/index.ts` re-exports all
```ts
Button({ variant?: "primary"|"secondary"|"ghost"|"danger", size?: "sm"|"md"|"lg", icon?: LucideIcon, loading?: boolean, ...buttonProps })
IconButton({ icon: LucideIcon, label: string, variant?, size?, ...buttonProps })
Card({ title?: ReactNode, subtitle?: ReactNode, actions?: ReactNode, padded?: boolean, className?, children })
StatTile({ label: string, value: ReactNode, sub?: ReactNode, icon?: LucideIcon, tone?: "default"|"accent"|"warning"|"danger" })
AnimatedNumber({ value: number, format?: (n: number) => string, duration?: number })
ProgressBar({ value: number /*0..1*/, tone?, indeterminate?: boolean, className? })
RingGauge({ value: number /*0..1*/, size?: number, thickness?: number, label?: ReactNode, sublabel?: ReactNode, tone? })
Badge({ tone?: "neutral"|"accent"|"success"|"warning"|"danger"|"info", children })
Chip({ active?: boolean, color?: string, count?: ReactNode, onClick?, children })
Checkbox({ checked: boolean, indeterminate?: boolean, onChange: (v: boolean) => void, label? })
Switch({ checked: boolean, onChange: (v: boolean) => void, label?, description? })
Tooltip({ content: ReactNode, children })
Skeleton({ className? })
EmptyState({ icon: LucideIcon, title: string, description?: string, action?: ReactNode })
PageHeader({ title: string, description?: string, actions?: ReactNode })
Dialog({ open: boolean, onClose: () => void, title: string, children, footer?: ReactNode, size?: "sm"|"md"|"lg" })
ConfirmDialog({ open, onClose, onConfirm: () => void, title, message: ReactNode, confirmLabel?, tone?: "default"|"danger", loading? })
Tabs({ value, onChange, items: { value: string; label: string; icon?: LucideIcon }[] })
SegmentedControl({ value, onChange, options: { value: string; label: string; icon?: LucideIcon }[] })
DropdownMenu({ trigger: ReactNode, items: ({ label, icon?, shortcut?, onSelect, disabled?, danger? } | "separator")[] })
CategoryDot({ category: FileCategory })
useToast(): { toast(opts: { title: string; description?: string; tone?: "success"|"error"|"info" }): void }
useChartTheme(): { textColor, mutedColor, gridColor, tooltipBg, tooltipBorder, accent, palette: string[] }
categoryMeta: Record<FileCategory, { label: string; color: string; icon: LucideIcon }>
```
`<ToastProvider>` is mounted by the shell.
