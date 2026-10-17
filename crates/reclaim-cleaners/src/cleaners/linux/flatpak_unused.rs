//! Linux cleaner: removes unused Flatpak runtimes with `flatpak uninstall --unused`.

use crate::safety::{run_command, run_command_timeout, LONG_TIMEOUT};
use crate::util::parse_size;
use crate::{CleanOptions, Cleaner, CleanerGroup, CleanerInfo, CleanupPreview, CleanupResult, Result, Risk};

const CMD: &str = "flatpak uninstall --unused -y";

pub struct FlatpakUnused;

/// Runs a read-only dry run (`--assumeno`) and sums the "< 142.3 MB" size column. Returns (bytes, items).
fn dry_run() -> Result<(u64, u64)> {
    // --assumeno answers "no" to the confirmation, so nothing is uninstalled.
    let out = match run_command("flatpak", &["uninstall", "--unused", "--assumeno"], false) {
        Ok(o) => o,
        // flatpak exits non-zero when the answer was "no"; the table is what we want.
        Err(crate::CleanError::Command { message, .. }) => message,
        Err(e) => return Err(e),
    };
    let (mut bytes, mut items) = (0u64, 0u64);
    for line in out.lines() {
        if let Some(rest) = line.split('<').nth(1) {
            if let Some(b) = parse_size(rest.trim(), 1000.0) {
                bytes = bytes.saturating_add(b);
                items += 1;
            }
        }
    }
    Ok((bytes, items))
}

impl Cleaner for FlatpakUnused {
    fn info(&self) -> CleanerInfo {
        crate::cleaners::simple::command_info(
            "flatpak-unused",
            "Unused Flatpak runtimes",
            "Uninstalls Flatpak runtimes and extensions no installed app needs.",
            CleanerGroup::System,
            Risk::Safe,
            false,
        )
    }

    fn is_available(&self) -> bool {
        which::which("flatpak").is_ok()
    }

    fn preview(&self) -> Result<CleanupPreview> {
        let (bytes, items) = dry_run().unwrap_or((0, 0));
        Ok(CleanupPreview {
            cleaner_id: "flatpak-unused".into(),
            total_bytes: bytes,
            item_count: items,
            items: Vec::new(),
            command: Some(CMD.into()),
        })
    }

    fn clean(&self, _options: &CleanOptions) -> Result<CleanupResult> {
        let (bytes, items) = dry_run().unwrap_or((0, 0));
        run_command_timeout("flatpak", &["uninstall", "--unused", "-y"], false, LONG_TIMEOUT)?;
        Ok(CleanupResult {
            cleaner_id: "flatpak-unused".into(),
            success: true,
            freed_bytes: bytes,
            items_removed: items,
            errors: Vec::new(),
        })
    }
}
