//! Snapshot commands: list, delete and diff saved scans (History page).

use crate::state::AppState;
use reclaim_core::{GrowthEntry, SnapshotMeta};
use tauri::State;

#[tauri::command]
pub fn list_snapshots(state: State<'_, AppState>) -> Result<Vec<SnapshotMeta>, String> {
    state.with_snapshots(|s| s.list())
}

#[tauri::command]
pub fn delete_snapshot(id: i64, state: State<'_, AppState>) -> Result<(), String> {
    state.with_snapshots(|s| s.delete(id))
}

#[tauri::command]
pub async fn diff_snapshots(
    older_id: i64,
    newer_id: i64,
    limit: u32,
    state: State<'_, AppState>,
) -> Result<Vec<GrowthEntry>, String> {
    let state = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        state.with_snapshots(|s| s.diff(older_id, newer_id, limit as usize))
    })
    .await
    .map_err(|e| e.to_string())?
}
