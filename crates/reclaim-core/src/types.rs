//! Data types that cross the engine/UI boundary (serialized to JSON for the UI).
//! Mirrored exactly in src/lib/types.ts; see docs/CONTRACT.md. All JSON is camelCase,
//! sizes are bytes (u64), timestamps are Unix milliseconds (i64), paths are strings.

use serde::{Deserialize, Serialize};

/// One mounted drive or partition.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiskInfo {
    pub id: String,
    pub name: String,
    pub mount_point: String,
    pub file_system: String,
    pub total_bytes: u64,
    pub used_bytes: u64,
    pub available_bytes: u64,
    pub is_removable: bool,
    /// True for the OS drive ("/" on Linux, the system drive on Windows).
    pub is_system: bool,
    pub kind: DiskKind,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum DiskKind {
    Ssd,
    Hdd,
    Unknown,
}

/// What to scan and how.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanOptions {
    pub root: String,
    /// Descend into other mounted filesystems below root. Default false.
    #[serde(default)]
    pub cross_filesystems: bool,
    /// Absolute paths to skip entirely.
    #[serde(default)]
    pub excluded_paths: Vec<String>,
    /// How many of the largest files to keep. Default 500.
    #[serde(default = "default_top_files")]
    pub top_files_limit: usize,
}

fn default_top_files() -> usize {
    500
}

/// Emitted repeatedly while a scan runs (about every 100 ms).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanProgress {
    pub scan_id: String,
    pub files_scanned: u64,
    pub dirs_scanned: u64,
    pub bytes_counted: u64,
    pub errors: u64,
    pub current_path: String,
    pub elapsed_ms: u64,
}

/// Totals for a finished scan (the full tree stays in the backend).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanSummary {
    pub scan_id: String,
    pub root: String,
    /// Allocated (real on-disk) bytes, hard links counted once.
    pub total_bytes: u64,
    /// Sum of file lengths as reported (can differ for sparse/compressed files).
    pub apparent_bytes: u64,
    pub file_count: u64,
    pub dir_count: u64,
    pub error_count: u64,
    pub started_at_ms: i64,
    pub finished_at_ms: i64,
    pub duration_ms: u64,
    pub type_breakdown: Vec<TypeStat>,
    /// Id of the snapshot saved for this scan, if saving succeeded.
    pub snapshot_id: Option<i64>,
}

/// A directory (or file) node returned for drill-down views.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TreeNode {
    pub path: String,
    pub name: String,
    pub is_dir: bool,
    pub size_bytes: u64,
    pub apparent_bytes: u64,
    pub file_count: u64,
    pub dir_count: u64,
    pub modified_ms: Option<i64>,
    /// Sorted by size_bytes descending; truncated to the requested depth/limit.
    pub children: Vec<TreeNode>,
    /// Number of children left out because of max_children (they are summed into `other_bytes`).
    pub hidden_children: u64,
    pub other_bytes: u64,
}

/// One file in the "largest files" list.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileEntry {
    pub path: String,
    pub name: String,
    pub size_bytes: u64,
    pub modified_ms: Option<i64>,
    pub extension: Option<String>,
    pub category: FileCategory,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
pub enum FileCategory {
    Video,
    Audio,
    Image,
    Document,
    Archive,
    DiskImage,
    Code,
    Executable,
    Database,
    Log,
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TypeStat {
    pub category: FileCategory,
    pub bytes: u64,
    pub count: u64,
}

/// A saved scan, used for the History page.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SnapshotMeta {
    pub id: i64,
    pub root: String,
    pub taken_at_ms: i64,
    pub total_bytes: u64,
    pub file_count: u64,
    /// Used bytes of the disk holding root at snapshot time (None if unknown).
    pub disk_used_bytes: Option<u64>,
    pub disk_total_bytes: Option<u64>,
}

/// One directory whose size changed between two snapshots.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GrowthEntry {
    pub path: String,
    pub before_bytes: u64,
    pub after_bytes: u64,
    /// after - before (negative = shrank).
    pub delta_bytes: i64,
    /// True if the path did not exist in the older snapshot.
    pub is_new: bool,
}
