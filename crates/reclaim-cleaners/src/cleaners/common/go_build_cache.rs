//! Cleaner: `go clean -cache`, which empties the Go build cache.

use std::path::PathBuf;

use crate::cleaners::simple::command_info;
use crate::safety::{run_command, run_command_timeout, LONG_TIMEOUT};
use crate::util::{dir_size, preview_from, Candidate};
use crate::{CleanOptions, Cleaner, CleanerGroup, CleanerInfo, CleanupPreview, CleanupResult, Result, Risk};

const CMD: &str = "go clean -cache";

pub struct GoBuildCache;

fn cache_dir() -> Option<PathBuf> {
    let out = run_command("go", &["env", "GOCACHE"], false).ok()?;
    let p = PathBuf::from(out.trim());
    (p.is_absolute() && p.is_dir()).then_some(p)
}

impl Cleaner for GoBuildCache {
    fn info(&self) -> CleanerInfo {
        command_info(
            "go-build-cache",
            "Go build cache",
            "Runs `go clean -cache`. Go rebuilds what it needs on the next build.",
            CleanerGroup::Developer,
            Risk::Safe,
            false,
        )
    }

    fn is_available(&self) -> bool {
        which::which("go").is_ok()
    }

    fn preview(&self) -> Result<CleanupPreview> {
        let cands = cache_dir()
            .map(|p| {
                vec![Candidate { size: dir_size(&p), path: p, modified_ms: None, label: Some("Go build cache".into()) }]
            })
            .unwrap_or_default();
        let mut pv = preview_from("go-build-cache", cands, Some(CMD.into()));
        pv.item_count = u64::from(pv.total_bytes > 0);
        Ok(pv)
    }

    fn clean(&self, _options: &CleanOptions) -> Result<CleanupResult> {
        let dir = cache_dir();
        let before = dir.as_deref().map(dir_size).unwrap_or(0);
        run_command_timeout("go", &["clean", "-cache"], false, LONG_TIMEOUT)?;
        let after = dir.as_deref().map(dir_size).unwrap_or(before);
        Ok(CleanupResult {
            cleaner_id: "go-build-cache".into(),
            success: true,
            freed_bytes: before.saturating_sub(after),
            items_removed: 1,
            errors: Vec::new(),
        })
    }
}
