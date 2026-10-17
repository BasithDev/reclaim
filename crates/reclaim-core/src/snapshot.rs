//! SQLite-backed history of scans: totals plus per-directory sizes (to a fixed depth),
//! so two snapshots of the same root can be diffed to show what grew.

use std::path::Path;
use std::sync::Mutex;

use rusqlite::{params, Connection};

use crate::scan::ScanResult;
use crate::{CoreError, GrowthEntry, Result, SnapshotMeta};

const SNAPSHOT_DEPTH: u32 = 6;

fn db_err(e: rusqlite::Error) -> CoreError {
    CoreError::Store(e.to_string())
}

pub struct SnapshotStore {
    conn: Mutex<Connection>,
}

impl SnapshotStore {
    /// Opens or creates the database file (parent directories are created).
    pub fn open(db_path: &Path) -> Result<Self> {
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| CoreError::Io { path: parent.to_string_lossy().to_string(), source: e })?;
        }
        let conn = Connection::open(db_path).map_err(db_err)?;
        conn.execute_batch(
            "PRAGMA journal_mode = WAL;
             PRAGMA foreign_keys = ON;
             CREATE TABLE IF NOT EXISTS snapshots (
                 id INTEGER PRIMARY KEY AUTOINCREMENT,
                 root TEXT NOT NULL,
                 taken_at_ms INTEGER NOT NULL,
                 total_bytes INTEGER NOT NULL,
                 file_count INTEGER NOT NULL,
                 disk_used_bytes INTEGER,
                 disk_total_bytes INTEGER
             );
             CREATE TABLE IF NOT EXISTS dir_sizes (
                 snapshot_id INTEGER NOT NULL REFERENCES snapshots(id) ON DELETE CASCADE,
                 path TEXT NOT NULL,
                 bytes INTEGER NOT NULL,
                 PRIMARY KEY (snapshot_id, path)
             ) WITHOUT ROWID;",
        )
        .map_err(db_err)?;
        Ok(Self { conn: Mutex::new(conn) })
    }

    /// Saves a finished scan (directory sizes to depth 6) and returns its metadata.
    pub fn save(&self, result: &ScanResult) -> Result<SnapshotMeta> {
        let s = &result.summary;
        let disk = crate::disks::disk_for_path(&s.root).ok().flatten();
        let (used, total) = match &disk {
            Some(d) => (Some(d.used_bytes), Some(d.total_bytes)),
            None => (None, None),
        };
        let sizes = result.directory_sizes(SNAPSHOT_DEPTH);
        let mut conn = self.conn.lock().unwrap();
        let tx = conn.transaction().map_err(db_err)?;
        tx.execute(
            "INSERT INTO snapshots (root, taken_at_ms, total_bytes, file_count, disk_used_bytes, disk_total_bytes)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![s.root, s.finished_at_ms, s.total_bytes as i64, s.file_count as i64, used.map(|v| v as i64), total.map(|v| v as i64)],
        )
        .map_err(db_err)?;
        let id = tx.last_insert_rowid();
        {
            let mut stmt = tx
                .prepare("INSERT OR REPLACE INTO dir_sizes (snapshot_id, path, bytes) VALUES (?1, ?2, ?3)")
                .map_err(db_err)?;
            for (path, bytes) in &sizes {
                stmt.execute(params![id, path, *bytes as i64]).map_err(db_err)?;
            }
        }
        tx.commit().map_err(db_err)?;
        Ok(SnapshotMeta {
            id,
            root: s.root.clone(),
            taken_at_ms: s.finished_at_ms,
            total_bytes: s.total_bytes,
            file_count: s.file_count,
            disk_used_bytes: used,
            disk_total_bytes: total,
        })
    }

    /// All snapshots, newest first.
    pub fn list(&self) -> Result<Vec<SnapshotMeta>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn
            .prepare(
                "SELECT id, root, taken_at_ms, total_bytes, file_count, disk_used_bytes, disk_total_bytes
                 FROM snapshots ORDER BY taken_at_ms DESC, id DESC",
            )
            .map_err(db_err)?;
        let rows = stmt
            .query_map([], |r| {
                Ok(SnapshotMeta {
                    id: r.get(0)?,
                    root: r.get(1)?,
                    taken_at_ms: r.get(2)?,
                    total_bytes: r.get::<_, i64>(3)? as u64,
                    file_count: r.get::<_, i64>(4)? as u64,
                    disk_used_bytes: r.get::<_, Option<i64>>(5)?.map(|v| v as u64),
                    disk_total_bytes: r.get::<_, Option<i64>>(6)?.map(|v| v as u64),
                })
            })
            .map_err(db_err)?;
        rows.collect::<std::result::Result<Vec<_>, _>>().map_err(db_err)
    }

    pub fn delete(&self, id: i64) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM dir_sizes WHERE snapshot_id = ?1", params![id]).map_err(db_err)?;
        conn.execute("DELETE FROM snapshots WHERE id = ?1", params![id]).map_err(db_err)?;
        Ok(())
    }

    fn load_sizes(conn: &Connection, id: i64) -> Result<Vec<(String, u64)>> {
        let exists: bool = conn
            .query_row("SELECT COUNT(*) FROM snapshots WHERE id = ?1", params![id], |r| r.get::<_, i64>(0))
            .map_err(db_err)?
            > 0;
        if !exists {
            return Err(CoreError::NotFound(format!("snapshot {id}")));
        }
        let mut stmt = conn.prepare("SELECT path, bytes FROM dir_sizes WHERE snapshot_id = ?1").map_err(db_err)?;
        let rows = stmt
            .query_map(params![id], |r| Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)? as u64)))
            .map_err(db_err)?;
        rows.collect::<std::result::Result<Vec<_>, _>>().map_err(db_err)
    }

    /// Directories that changed between two snapshots; delegates to crate::diff.
    pub fn diff(&self, older_id: i64, newer_id: i64, limit: usize) -> Result<Vec<GrowthEntry>> {
        let (older, newer) = {
            let conn = self.conn.lock().unwrap();
            (Self::load_sizes(&conn, older_id)?, Self::load_sizes(&conn, newer_id)?)
        };
        Ok(crate::diff::diff_sizes(&older, &newer, limit))
    }
}
