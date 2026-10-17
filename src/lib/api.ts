// Reclaim: typed wrappers for every Tauri command and event. The UI calls the backend
// ONLY through this file. Each function maps 1:1 to a #[tauri::command] in src-tauri
// (Tauri converts camelCase args here to snake_case params in Rust). See docs/CONTRACT.md.

import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type {
  AppInfo, CleanerInfo, CleanupPreview, CleanupResult, DeletedOpenFile, DiskInfo,
  FileCategory, FileEntry, GrowthEntry, ScanError, ScanOptions, ScanProgress,
  ScanSummary, Settings, SnapshotMeta, TreeNode,
} from "./types";

export const api = {
  getAppInfo: () => invoke<AppInfo>("get_app_info"),
  listDisks: () => invoke<DiskInfo[]>("list_disks"),

  /** Starts a background scan and returns its id; follow it with the scan events below. */
  startScan: (options: ScanOptions) => invoke<string>("start_scan", { options }),
  cancelScan: (scanId: string) => invoke<void>("cancel_scan", { scanId }),
  getScanSummary: (scanId: string) => invoke<ScanSummary>("get_scan_summary", { scanId }),
  /** path null = scan root. depth 0 = node only. */
  getTree: (scanId: string, path: string | null, depth: number, maxChildren: number) =>
    invoke<TreeNode>("get_tree", { scanId, path, depth, maxChildren }),
  getLargestFiles: (scanId: string, limit: number, category: FileCategory | null) =>
    invoke<FileEntry[]>("get_largest_files", { scanId, limit, category }),

  listCleaners: () => invoke<CleanerInfo[]>("list_cleaners"),
  previewCleaner: (cleanerId: string) => invoke<CleanupPreview>("preview_cleaner", { cleanerId }),
  runCleaners: (cleanerIds: string[], permanent: boolean) =>
    invoke<CleanupResult[]>("run_cleaners", { cleanerIds, permanent }),
  /** Moves one file/folder (e.g. from Large Files) to Trash, or deletes it. Returns bytes freed. */
  removePath: (path: string, permanent: boolean) => invoke<number>("remove_path", { path, permanent }),
  getDeletedOpenFiles: () => invoke<DeletedOpenFile[]>("get_deleted_open_files"),

  listSnapshots: () => invoke<SnapshotMeta[]>("list_snapshots"),
  deleteSnapshot: (id: number) => invoke<void>("delete_snapshot", { id }),
  diffSnapshots: (olderId: number, newerId: number, limit: number) =>
    invoke<GrowthEntry[]>("diff_snapshots", { olderId, newerId, limit }),

  getSettings: () => invoke<Settings>("get_settings"),
  saveSettings: (settings: Settings) => invoke<void>("save_settings", { settings }),
};

export const events = {
  onScanProgress: (cb: (p: ScanProgress) => void): Promise<UnlistenFn> =>
    listen<ScanProgress>("scan://progress", (e) => cb(e.payload)),
  onScanComplete: (cb: (s: ScanSummary) => void): Promise<UnlistenFn> =>
    listen<ScanSummary>("scan://complete", (e) => cb(e.payload)),
  onScanError: (cb: (e: ScanError) => void): Promise<UnlistenFn> =>
    listen<ScanError>("scan://error", (e) => cb(e.payload)),
};
