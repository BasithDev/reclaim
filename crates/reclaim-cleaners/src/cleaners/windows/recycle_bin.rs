//! Windows cleaner: empties the Recycle Bin with PowerShell's Clear-RecycleBin.

use crate::cleaners::simple::command_info;
use crate::safety::run_command;
use crate::{CleanOptions, Cleaner, CleanerGroup, CleanerInfo, CleanupPreview, CleanupResult, Result, Risk};

const CMD: &str = "powershell -NoProfile -Command Clear-RecycleBin -Force";

/// Fixed scripts: no user-controlled text is ever placed in them.
const MEASURE: &str = "$i = @((New-Object -ComObject Shell.Application).NameSpace(0xA).Items()); \
     $s = 0; foreach ($x in $i) { $s += [int64]$x.Size }; \"$($i.Count) $s\"";
const CLEAR: &str = "Clear-RecycleBin -Force -ErrorAction SilentlyContinue";

pub struct RecycleBin;

/// (item count, total bytes) of the Recycle Bin.
fn measure() -> Result<(u64, u64)> {
    let out = run_command("powershell", &["-NoProfile", "-NonInteractive", "-Command", MEASURE], false)?;
    let mut parts = out.split_whitespace();
    let count = parts.next().and_then(|t| t.parse().ok()).unwrap_or(0);
    let bytes = parts.next().and_then(|t| t.parse().ok()).unwrap_or(0);
    Ok((count, bytes))
}

impl Cleaner for RecycleBin {
    fn info(&self) -> CleanerInfo {
        command_info(
            "recycle-bin",
            "Recycle Bin",
            "Permanently empties the Recycle Bin. Items in it cannot be restored afterwards.",
            CleanerGroup::User,
            Risk::Careful,
            false,
        )
    }

    fn is_available(&self) -> bool {
        which::which("powershell").is_ok()
    }

    fn preview(&self) -> Result<CleanupPreview> {
        let (count, bytes) = measure()?;
        Ok(CleanupPreview {
            cleaner_id: "recycle-bin".into(),
            total_bytes: bytes,
            item_count: count,
            items: Vec::new(),
            command: Some(CMD.into()),
        })
    }

    fn clean(&self, _options: &CleanOptions) -> Result<CleanupResult> {
        let (count, bytes) = measure().unwrap_or((0, 0));
        run_command("powershell", &["-NoProfile", "-NonInteractive", "-Command", CLEAR], false)?;
        Ok(CleanupResult {
            cleaner_id: "recycle-bin".into(),
            success: true,
            freed_bytes: bytes,
            items_removed: count,
            errors: Vec::new(),
        })
    }
}
