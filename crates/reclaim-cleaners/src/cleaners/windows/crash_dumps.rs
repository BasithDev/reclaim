//! Windows cleaner: crash dump files in %LOCALAPPDATA%\CrashDumps and C:\Windows\Minidump.

use std::path::{Path, PathBuf};

use super::windows_dir;
use crate::cleaners::simple::{files_info, SimpleFiles};
use crate::util::{children_of, Candidate};
use crate::{Cleaner, CleanerGroup, Risk};

fn user_dir() -> Option<PathBuf> {
    dirs::cache_dir().map(|l| l.join("CrashDumps"))
}

fn system_dir() -> PathBuf {
    windows_dir().join("Minidump")
}

fn dirs_list() -> Vec<PathBuf> {
    user_dir().into_iter().chain(std::iter::once(system_dir())).filter(|p| p.is_dir()).collect()
}

fn collect() -> Vec<Candidate> {
    dirs_list().iter().flat_map(|d| children_of(d, None, Some("crash dump"))).collect()
}

/// Only the system Minidump folder needs elevation.
fn needs_admin(p: &Path) -> bool {
    p.starts_with(windows_dir())
}

pub fn cleaner() -> Box<dyn Cleaner> {
    Box::new(SimpleFiles {
        info: files_info(
            "crash-dumps",
            "Crash dumps",
            "Removes crash dump files. Your own dumps go to the Recycle Bin; system minidumps are deleted permanently and need administrator permission.",
            CleanerGroup::System,
            Risk::Safe,
            true,
        ),
        available: || !dirs_list().is_empty(),
        roots: dirs_list,
        collect,
        needs_admin,
        force_permanent: false,
        post_clean: None,
    })
}
