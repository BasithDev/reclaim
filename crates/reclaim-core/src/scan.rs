//! Parallel directory scanner.
//!
//! Walks a tree measuring allocated size (platform::allocated_size), counts hard
//! links once (platform::file_id), never follows symlinks/junctions, stays on one
//! filesystem unless asked, reports progress through a callback and can be cancelled.
//! The finished tree is kept in memory (`ScanResult`) for drill-down queries.

use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use crate::{FileCategory, FileEntry, Result, ScanOptions, ScanProgress, ScanSummary, TreeNode};

/// A finished scan: summary plus the in-memory tree. Cheap to share behind an Arc.
pub struct ScanResult {
    pub summary: ScanSummary,
    // A1: private tree storage (e.g. an arena of nodes) goes here.
}

impl ScanResult {
    /// The node at `path` (None = scan root) with children down to `depth` levels
    /// (0 = node only), each level capped at `max_children` largest entries.
    pub fn subtree(&self, path: Option<&str>, depth: u32, max_children: usize) -> Option<TreeNode> {
        let _ = (path, depth, max_children);
        todo!("A1: implement")
    }

    /// Largest files overall, optionally filtered by category, biggest first.
    pub fn largest_files(&self, limit: usize, category: Option<FileCategory>) -> Vec<FileEntry> {
        let _ = (limit, category);
        todo!("A1: implement")
    }

    /// Every directory down to `max_depth` below root with its size; used for snapshots.
    pub fn directory_sizes(&self, max_depth: u32) -> Vec<(String, u64)> {
        let _ = max_depth;
        todo!("A1: implement")
    }
}

/// Runs a scan to completion (blocking; call from a background thread).
/// `on_progress` is throttled to about every 100 ms. Returns Err(Cancelled) if `cancel` is set.
pub fn scan(
    scan_id: String,
    options: ScanOptions,
    on_progress: impl Fn(ScanProgress) + Send + Sync,
    cancel: Arc<AtomicBool>,
) -> Result<ScanResult> {
    let _ = (scan_id, options, on_progress, cancel);
    todo!("A1: implement")
}

/// Maps a file extension (lowercase, without dot) to a category.
pub fn categorize(extension: Option<&str>) -> FileCategory {
    let _ = extension;
    todo!("A1: implement")
}
