//! Reclaim Tauri host: wires reclaim-core and reclaim-cleaners to the UI.
//! Registers every command listed in docs/CONTRACT.md, holds app state
//! (running/finished scans, snapshot store, settings) and emits scan events.

mod commands;
mod settings;
mod state;

use commands::{app, cleaners, history, scan};
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let config = app.path().app_config_dir()?;
            let data = app.path().app_data_dir()?;
            std::fs::create_dir_all(&data).ok();
            app.manage(state::AppState::new(config.join("settings.json"), data.join("snapshots.db")));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            app::get_app_info,
            app::list_disks,
            app::get_settings,
            app::save_settings,
            scan::start_scan,
            scan::cancel_scan,
            scan::get_scan_summary,
            scan::get_tree,
            scan::get_largest_files,
            cleaners::list_cleaners,
            cleaners::preview_cleaner,
            cleaners::run_cleaners,
            cleaners::remove_path,
            cleaners::get_deleted_open_files,
            history::list_snapshots,
            history::delete_snapshot,
            history::diff_snapshots,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Reclaim");
}
