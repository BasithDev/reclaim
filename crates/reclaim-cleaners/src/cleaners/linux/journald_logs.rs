//! Linux cleaner: vacuums the systemd journal down to 200 MB with `journalctl --vacuum-size`.

use crate::safety::run_command;
use crate::util::parse_size;
use crate::{CleanOptions, Cleaner, CleanerGroup, CleanerInfo, CleanupPreview, CleanupResult, Result, Risk};

const KEEP: u64 = 200 * 1024 * 1024;
const CMD: &str = "pkexec journalctl --vacuum-size=200M";

pub struct JournaldLogs;

/// Current journal size from `journalctl --disk-usage`.
fn usage() -> Result<u64> {
    let out = run_command("journalctl", &["--disk-usage"], false)?;
    let after = out.split("take up").nth(1).unwrap_or("");
    Ok(after.split_whitespace().next().and_then(|t| parse_size(t, 1024.0)).unwrap_or(0))
}

impl Cleaner for JournaldLogs {
    fn info(&self) -> CleanerInfo {
        crate::cleaners::simple::command_info(
            "journald-logs",
            "System journal logs",
            "Trims the systemd journal to 200 MB, discarding the oldest log entries. Needs administrator permission.",
            CleanerGroup::System,
            Risk::Safe,
            true,
        )
    }

    fn is_available(&self) -> bool {
        which::which("journalctl").is_ok()
    }

    fn preview(&self) -> Result<CleanupPreview> {
        let total = usage()?.saturating_sub(KEEP);
        Ok(CleanupPreview {
            cleaner_id: "journald-logs".into(),
            total_bytes: total,
            item_count: u64::from(total > 0),
            items: Vec::new(),
            command: Some(CMD.into()),
        })
    }

    fn clean(&self, _options: &CleanOptions) -> Result<CleanupResult> {
        let before = usage().unwrap_or(0);
        run_command("journalctl", &["--vacuum-size=200M"], true)?;
        let after = usage().unwrap_or(before);
        Ok(CleanupResult {
            cleaner_id: "journald-logs".into(),
            success: true,
            freed_bytes: before.saturating_sub(after),
            items_removed: 1,
            errors: Vec::new(),
        })
    }
}
