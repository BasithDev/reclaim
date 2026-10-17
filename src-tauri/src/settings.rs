//! App settings: the Settings struct (mirrors src/lib/types.ts) and JSON load/save
//! at app_config_dir/settings.json. Missing or unreadable files yield the defaults.

use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Settings {
    /// "system" | "light" | "dark"
    pub theme: String,
    pub excluded_paths: Vec<String>,
    pub permanent_delete: bool,
    pub confirm_before_clean: bool,
    pub cross_filesystems: bool,
    pub default_scan_root: Option<String>,
    pub save_snapshots: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            theme: "system".into(),
            excluded_paths: Vec::new(),
            permanent_delete: true,
            confirm_before_clean: true,
            cross_filesystems: false,
            default_scan_root: None,
            save_snapshots: true,
        }
    }
}

impl Settings {
    pub fn load(path: &Path) -> Self {
        std::fs::read_to_string(path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }

    pub fn save(&self, path: &Path) -> Result<(), String> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let json = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
        std::fs::write(path, json).map_err(|e| e.to_string())
    }
}
