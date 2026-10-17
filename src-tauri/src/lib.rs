//! Reclaim Tauri host: wires reclaim-core and reclaim-cleaners to the UI.
//! Registers every command listed in docs/CONTRACT.md, holds app state
//! (running/finished scans, snapshot store, settings) and emits scan events.
//! A4: split commands into modules under src-tauri/src/commands/ as they grow.

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .run(tauri::generate_context!())
        .expect("error while running Reclaim");
}
