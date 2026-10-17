//! SQLite-backed history of scans: totals plus per-directory sizes (to a fixed depth),
//! so two snapshots of the same root can be diffed to show what grew.

use std::path::Path;

use crate::scan::ScanResult;
use crate::{GrowthEntry, Result, SnapshotMeta};

pub struct SnapshotStore {
    // A1: rusqlite::Connection (behind a Mutex if needed) goes here.
}

impl SnapshotStore {
    /// Opens or creates the database file (parent directories are created).
    pub fn open(db_path: &Path) -> Result<Self> {
        let _ = db_path;
        todo!("A1: implement")
    }

    /// Saves a finished scan (directory sizes to depth 6) and returns its metadata.
    pub fn save(&self, result: &ScanResult) -> Result<SnapshotMeta> {
        let _ = result;
        todo!("A1: implement")
    }

    /// All snapshots, newest first.
    pub fn list(&self) -> Result<Vec<SnapshotMeta>> {
        todo!("A1: implement")
    }

    pub fn delete(&self, id: i64) -> Result<()> {
        let _ = id;
        todo!("A1: implement")
    }

    /// Directories that changed between two snapshots; delegates to crate::diff.
    pub fn diff(&self, older_id: i64, newer_id: i64, limit: usize) -> Result<Vec<GrowthEntry>> {
        let _ = (older_id, newer_id, limit);
        todo!("A1: implement")
    }
}
