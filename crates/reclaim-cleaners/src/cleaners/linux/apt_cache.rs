//! Linux cleaner: clears downloaded .deb packages from /var/cache/apt/archives with `apt-get clean`.

use std::path::Path;

use crate::safety::run_command_timeout;
use crate::safety::LONG_TIMEOUT;
use crate::util::{candidate, dir_size, preview_from};
use crate::{CleanOptions, Cleaner, CleanerGroup, CleanerInfo, CleanupPreview, CleanupResult, Result, Risk};

const DIR: &str = "/var/cache/apt/archives";

pub struct AptCache;

impl Cleaner for AptCache {
    fn info(&self) -> CleanerInfo {
        crate::cleaners::simple::command_info(
            "apt-cache",
            "APT package cache",
            "Removes downloaded .deb package files with `apt-get clean`. Packages are re-downloaded if needed. Needs administrator permission.",
            CleanerGroup::System,
            Risk::Safe,
            true,
        )
    }

    fn is_available(&self) -> bool {
        which::which("apt-get").is_ok() && Path::new(DIR).is_dir()
    }

    fn preview(&self) -> Result<CleanupPreview> {
        let debs = std::fs::read_dir(DIR)
            .map(|rd| {
                rd.flatten()
                    .filter(|e| e.file_name().to_string_lossy().ends_with(".deb"))
                    .filter_map(|e| candidate(&e.path(), Some("package file".into())))
                    .collect()
            })
            .unwrap_or_default();
        Ok(preview_from("apt-cache", debs, Some("pkexec apt-get clean".into())))
    }

    fn clean(&self, _options: &CleanOptions) -> Result<CleanupResult> {
        let before = dir_size(Path::new(DIR));
        run_command_timeout("apt-get", &["clean"], true, LONG_TIMEOUT)?;
        let after = dir_size(Path::new(DIR));
        Ok(CleanupResult {
            cleaner_id: "apt-cache".into(),
            success: true,
            freed_bytes: before.saturating_sub(after),
            items_removed: 1,
            errors: Vec::new(),
        })
    }
}
