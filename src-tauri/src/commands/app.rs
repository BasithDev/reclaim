//! App-level commands: app info, disks, settings.

use crate::settings::Settings;
use crate::state::AppState;
use reclaim_core::DiskInfo;
use serde::Serialize;
use tauri::State;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppInfo {
    version: String,
    os: &'static str,
    arch: &'static str,
    home_dir: String,
}

#[tauri::command]
pub fn get_app_info() -> AppInfo {
    AppInfo {
        version: env!("CARGO_PKG_VERSION").to_string(),
        os: match std::env::consts::OS {
            "linux" => "linux",
            "windows" => "windows",
            "macos" => "macos",
            _ => "other",
        },
        arch: std::env::consts::ARCH,
        home_dir: dirs::home_dir().map(|p| p.to_string_lossy().into_owned()).unwrap_or_default(),
    }
}

#[tauri::command]
pub async fn list_disks() -> Result<Vec<DiskInfo>, String> {
    tauri::async_runtime::spawn_blocking(|| reclaim_core::disks::list_disks().map_err(|e| e.to_string()))
        .await
        .map_err(|e| e.to_string())?
}

#[tauri::command]
pub fn get_settings(state: State<'_, AppState>) -> Settings {
    state.settings.lock().unwrap_or_else(|p| p.into_inner()).clone()
}

#[tauri::command]
pub fn save_settings(settings: Settings, state: State<'_, AppState>) -> Result<(), String> {
    settings.save(&state.settings_path)?;
    *state.settings.lock().unwrap_or_else(|p| p.into_inner()) = settings;
    Ok(())
}
