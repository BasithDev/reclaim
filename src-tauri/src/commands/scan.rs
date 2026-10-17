//! Scan commands: start/cancel a background scan, query the finished tree and largest files.
//! Emits scan://progress, scan://complete (after the snapshot is saved) and scan://error.

use crate::state::{AppState, FinishedScan};
use reclaim_core::{CoreError, FileCategory, FileEntry, ScanOptions, ScanSummary, TreeNode};
use serde::Serialize;
use std::sync::atomic::Ordering;
use tauri::{AppHandle, Emitter, State};

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ScanErrorPayload {
    scan_id: String,
    message: String,
    cancelled: bool,
}

#[tauri::command]
pub fn start_scan(options: ScanOptions, app: AppHandle, state: State<'_, AppState>) -> Result<String, String> {
    let scan_id = uuid::Uuid::new_v4().to_string();
    let cancel = state.register_running(&scan_id);
    let state = state.inner().clone();
    let id = scan_id.clone();

    tauri::async_runtime::spawn_blocking(move || {
        let progress_app = app.clone();
        let result = reclaim_core::scan::scan(
            id.clone(),
            options,
            move |p| {
                let _ = progress_app.emit("scan://progress", p);
            },
            cancel,
        );
        state.finish_running(&id);
        match result {
            Ok(result) => {
                let mut summary = result.summary.clone();
                let save = state.settings.lock().unwrap_or_else(|p| p.into_inner()).save_snapshots;
                if save {
                    // A failed snapshot must not fail the scan; the History page just misses an entry.
                    if let Ok(meta) = state.with_snapshots(|s| s.save(&result)) {
                        summary.snapshot_id = Some(meta.id);
                    }
                }
                state.store_scan(
                    id.clone(),
                    FinishedScan { result: std::sync::Arc::new(result), summary: summary.clone() },
                );
                let _ = app.emit("scan://complete", summary);
            }
            Err(e) => {
                let cancelled = matches!(e, CoreError::Cancelled);
                let _ = app.emit(
                    "scan://error",
                    ScanErrorPayload { scan_id: id, message: e.to_string(), cancelled },
                );
            }
        }
    });
    Ok(scan_id)
}

#[tauri::command]
pub fn cancel_scan(scan_id: String, state: State<'_, AppState>) -> Result<(), String> {
    if let Some(flag) = state.cancel_flag(&scan_id) {
        flag.store(true, Ordering::Relaxed);
    }
    Ok(())
}

#[tauri::command]
pub fn get_scan_summary(scan_id: String, state: State<'_, AppState>) -> Result<ScanSummary, String> {
    Ok(state.get_scan(&scan_id)?.summary.clone())
}

#[tauri::command]
pub async fn get_tree(
    scan_id: String,
    path: Option<String>,
    depth: u32,
    max_children: u32,
    state: State<'_, AppState>,
) -> Result<TreeNode, String> {
    let scan = state.get_scan(&scan_id)?;
    tauri::async_runtime::spawn_blocking(move || {
        scan.result
            .subtree(path.as_deref(), depth, max_children as usize)
            .ok_or_else(|| format!("path not found in scan: {}", path.unwrap_or_default()))
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn get_largest_files(
    scan_id: String,
    limit: u32,
    category: Option<FileCategory>,
    state: State<'_, AppState>,
) -> Result<Vec<FileEntry>, String> {
    let scan = state.get_scan(&scan_id)?;
    tauri::async_runtime::spawn_blocking(move || scan.result.largest_files(limit as usize, category))
        .await
        .map_err(|e| e.to_string())
}
