//! Cleaner commands: list, preview, run, remove a single path, deleted-but-open files.

use reclaim_cleaners::{
    find_cleaner, insights, safety, CleanOptions, CleanerInfo, CleanupPreview, CleanupResult, DeletedOpenFile,
};

fn join<T>(r: Result<Result<T, String>, tauri::Error>) -> Result<T, String> {
    r.map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn list_cleaners() -> Result<Vec<CleanerInfo>, String> {
    join(
        tauri::async_runtime::spawn_blocking(|| {
            Ok(reclaim_cleaners::all_cleaners().iter().map(|c| c.info()).collect())
        })
        .await,
    )
}

#[tauri::command]
pub async fn preview_cleaner(cleaner_id: String) -> Result<CleanupPreview, String> {
    join(
        tauri::async_runtime::spawn_blocking(move || {
            let c = find_cleaner(&cleaner_id).ok_or_else(|| format!("unknown cleaner: {cleaner_id}"))?;
            c.preview().map_err(|e| e.to_string())
        })
        .await,
    )
}

#[tauri::command]
pub async fn run_cleaners(cleaner_ids: Vec<String>, permanent: bool) -> Result<Vec<CleanupResult>, String> {
    join(
        tauri::async_runtime::spawn_blocking(move || {
            let options = CleanOptions { permanent };
            Ok(cleaner_ids
                .into_iter()
                .map(|id| {
                    let outcome = match find_cleaner(&id) {
                        Some(c) => c.clean(&options).map_err(|e| e.to_string()),
                        None => Err(format!("unknown cleaner: {id}")),
                    };
                    outcome.unwrap_or_else(|msg| CleanupResult {
                        cleaner_id: id,
                        success: false,
                        freed_bytes: 0,
                        items_removed: 0,
                        errors: vec![msg],
                    })
                })
                .collect())
        })
        .await,
    )
}

#[tauri::command]
pub async fn remove_path(path: String, permanent: bool) -> Result<u64, String> {
    join(
        tauri::async_runtime::spawn_blocking(move || {
            safety::remove_path(std::path::Path::new(&path), permanent).map_err(|e| e.to_string())
        })
        .await,
    )
}

#[tauri::command]
pub async fn get_deleted_open_files() -> Result<Vec<DeletedOpenFile>, String> {
    join(
        tauri::async_runtime::spawn_blocking(|| insights::deleted_open_files().map_err(|e| e.to_string())).await,
    )
}
