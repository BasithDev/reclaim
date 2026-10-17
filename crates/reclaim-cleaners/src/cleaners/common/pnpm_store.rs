//! Cleaner: `pnpm store prune`, which drops packages in the store that no project references.

use std::path::PathBuf;

use crate::cleaners::simple::command_info;
use crate::safety::{run_command, run_command_timeout, LONG_TIMEOUT};
use crate::util::dir_size;
use crate::{CleanOptions, Cleaner, CleanerGroup, CleanerInfo, CleanupPreview, CleanupResult, PreviewItem, Result, Risk};

const CMD: &str = "pnpm store prune";

pub struct PnpmStore;

fn store_path() -> Option<PathBuf> {
    let out = run_command("pnpm", &["store", "path"], false).ok()?;
    let p = PathBuf::from(out.trim());
    p.is_absolute().then_some(p)
}

impl Cleaner for PnpmStore {
    fn info(&self) -> CleanerInfo {
        command_info(
            "pnpm-store",
            "pnpm store",
            "Runs `pnpm store prune` to remove packages no project uses any more.",
            CleanerGroup::Developer,
            Risk::Safe,
            false,
        )
    }

    fn is_available(&self) -> bool {
        which::which("pnpm").is_ok()
    }

    fn preview(&self) -> Result<CleanupPreview> {
        // The exact amount is only known after pruning; show the store location instead of guessing.
        let items = store_path()
            .map(|p| {
                vec![PreviewItem {
                    path: p.to_string_lossy().into_owned(),
                    size_bytes: 0,
                    modified_ms: None,
                    label: Some("store location; only unreferenced packages are removed, size known after pruning".into()),
                }]
            })
            .unwrap_or_default();
        Ok(CleanupPreview {
            cleaner_id: "pnpm-store".into(),
            total_bytes: 0,
            item_count: 0,
            items,
            command: Some(CMD.into()),
        })
    }

    fn clean(&self, _options: &CleanOptions) -> Result<CleanupResult> {
        let store = store_path();
        let before = store.as_deref().map(dir_size).unwrap_or(0);
        run_command_timeout("pnpm", &["store", "prune"], false, LONG_TIMEOUT)?;
        let after = store.as_deref().map(dir_size).unwrap_or(before);
        Ok(CleanupResult {
            cleaner_id: "pnpm-store".into(),
            success: true,
            freed_bytes: before.saturating_sub(after),
            items_removed: 1,
            errors: Vec::new(),
        })
    }
}
