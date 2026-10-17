// Reclaim: TypeScript mirror of the Rust types that cross the Tauri boundary.
// Source of truth: crates/reclaim-core/src/types.rs, crates/reclaim-cleaners/src/types.rs,
// and the Settings struct in src-tauri. Keep in sync; see docs/CONTRACT.md.
// Sizes are bytes, timestamps are Unix milliseconds.

export type DiskKind = "ssd" | "hdd" | "unknown";

export interface DiskInfo {
  id: string;
  name: string;
  mountPoint: string;
  fileSystem: string;
  totalBytes: number;
  usedBytes: number;
  availableBytes: number;
  isRemovable: boolean;
  isSystem: boolean;
  kind: DiskKind;
}

export interface ScanOptions {
  root: string;
  crossFilesystems?: boolean;
  excludedPaths?: string[];
  topFilesLimit?: number;
}

export interface ScanProgress {
  scanId: string;
  filesScanned: number;
  dirsScanned: number;
  bytesCounted: number;
  errors: number;
  currentPath: string;
  elapsedMs: number;
}

export type FileCategory =
  | "video" | "audio" | "image" | "document" | "archive" | "diskImage"
  | "code" | "executable" | "database" | "log" | "other";

export interface TypeStat {
  category: FileCategory;
  bytes: number;
  count: number;
}

export interface ScanSummary {
  scanId: string;
  root: string;
  totalBytes: number;
  apparentBytes: number;
  fileCount: number;
  dirCount: number;
  errorCount: number;
  startedAtMs: number;
  finishedAtMs: number;
  durationMs: number;
  typeBreakdown: TypeStat[];
  snapshotId: number | null;
}

export interface TreeNode {
  path: string;
  name: string;
  isDir: boolean;
  sizeBytes: number;
  apparentBytes: number;
  fileCount: number;
  dirCount: number;
  modifiedMs: number | null;
  children: TreeNode[];
  hiddenChildren: number;
  otherBytes: number;
}

export interface FileEntry {
  path: string;
  name: string;
  sizeBytes: number;
  modifiedMs: number | null;
  extension: string | null;
  category: FileCategory;
}

export interface SnapshotMeta {
  id: number;
  root: string;
  takenAtMs: number;
  totalBytes: number;
  fileCount: number;
  diskUsedBytes: number | null;
  diskTotalBytes: number | null;
}

export interface GrowthEntry {
  path: string;
  beforeBytes: number;
  afterBytes: number;
  deltaBytes: number;
  isNew: boolean;
}

export type CleanerGroup = "system" | "developer" | "applications" | "user";
export type Risk = "safe" | "moderate" | "careful";
export type CleanMethod = "files" | "command";

export interface CleanerInfo {
  id: string;
  name: string;
  description: string;
  group: CleanerGroup;
  risk: Risk;
  requiresAdmin: boolean;
  method: CleanMethod;
}

export interface PreviewItem {
  path: string;
  sizeBytes: number;
  modifiedMs: number | null;
  label: string | null;
}

export interface CleanupPreview {
  cleanerId: string;
  totalBytes: number;
  itemCount: number;
  items: PreviewItem[];
  command: string | null;
}

export interface CleanupResult {
  cleanerId: string;
  success: boolean;
  freedBytes: number;
  itemsRemoved: number;
  errors: string[];
}

export interface DeletedOpenFile {
  pid: number;
  processName: string;
  path: string;
  sizeBytes: number;
}

export type ThemePreference = "system" | "light" | "dark";

export interface Settings {
  theme: ThemePreference;
  excludedPaths: string[];
  /** Default for cleanups: false = Trash/Recycle Bin. */
  permanentDelete: boolean;
  confirmBeforeClean: boolean;
  crossFilesystems: boolean;
  /** Path scanned by the Dashboard's "Scan" button; null = home directory. */
  defaultScanRoot: string | null;
  /** Save a snapshot after every scan (History page). */
  saveSnapshots: boolean;
}

export interface AppInfo {
  version: string;
  os: "linux" | "windows" | "macos" | "other";
  arch: string;
  homeDir: string;
}

/** Payload of the "scan://error" event. */
export interface ScanError {
  scanId: string;
  message: string;
  cancelled: boolean;
}
