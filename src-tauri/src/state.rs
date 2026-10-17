//! Shared app state: finished scans (last 3), running scans' cancel flags,
//! the snapshot store and the current settings. Cheap to clone (all Arc).

use crate::settings::Settings;
use reclaim_core::scan::ScanResult;
use reclaim_core::snapshot::SnapshotStore;
use reclaim_core::ScanSummary;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex, MutexGuard};

const KEEP_SCANS: usize = 3;

pub struct FinishedScan {
    pub result: Arc<ScanResult>,
    /// Summary including the snapshot id (the result's own summary does not have it).
    pub summary: ScanSummary,
}

#[derive(Clone)]
pub struct AppState {
    scans: Arc<Mutex<Vec<(String, Arc<FinishedScan>)>>>, // oldest first
    running: Arc<Mutex<HashMap<String, Arc<AtomicBool>>>>,
    pub snapshots: Arc<Mutex<Option<SnapshotStore>>>,
    pub snapshot_error: Arc<Mutex<Option<String>>>,
    pub settings: Arc<Mutex<Settings>>,
    pub settings_path: Arc<PathBuf>,
}

fn lock<T>(m: &Mutex<T>) -> MutexGuard<'_, T> {
    m.lock().unwrap_or_else(|p| p.into_inner())
}

impl AppState {
    pub fn new(settings_path: PathBuf, db_path: PathBuf) -> Self {
        let (store, err) = match SnapshotStore::open(&db_path) {
            Ok(s) => (Some(s), None),
            Err(e) => (None, Some(e.to_string())),
        };
        Self {
            scans: Default::default(),
            running: Default::default(),
            snapshots: Arc::new(Mutex::new(store)),
            snapshot_error: Arc::new(Mutex::new(err)),
            settings: Arc::new(Mutex::new(Settings::load(&settings_path))),
            settings_path: Arc::new(settings_path),
        }
    }

    pub fn register_running(&self, id: &str) -> Arc<AtomicBool> {
        let flag = Arc::new(AtomicBool::new(false));
        lock(&self.running).insert(id.to_string(), flag.clone());
        flag
    }

    pub fn cancel_flag(&self, id: &str) -> Option<Arc<AtomicBool>> {
        lock(&self.running).get(id).cloned()
    }

    pub fn finish_running(&self, id: &str) {
        lock(&self.running).remove(id);
    }

    pub fn store_scan(&self, id: String, scan: FinishedScan) {
        let mut scans = lock(&self.scans);
        scans.push((id, Arc::new(scan)));
        while scans.len() > KEEP_SCANS {
            scans.remove(0);
        }
    }

    pub fn get_scan(&self, id: &str) -> Result<Arc<FinishedScan>, String> {
        lock(&self.scans)
            .iter()
            .find(|(i, _)| i == id)
            .map(|(_, s)| s.clone())
            .ok_or_else(|| format!("scan not found: {id} (only the last {KEEP_SCANS} scans are kept)"))
    }

    pub fn with_snapshots<T>(
        &self,
        f: impl FnOnce(&SnapshotStore) -> reclaim_core::Result<T>,
    ) -> Result<T, String> {
        match lock(&self.snapshots).as_ref() {
            Some(s) => f(s).map_err(|e| e.to_string()),
            None => Err(format!(
                "snapshot store unavailable: {}",
                lock(&self.snapshot_error).clone().unwrap_or_default()
            )),
        }
    }
}
