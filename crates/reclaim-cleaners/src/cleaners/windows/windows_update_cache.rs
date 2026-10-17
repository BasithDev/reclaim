//! Windows cleaner: downloaded update files in C:\Windows\SoftwareDistribution\Download.

use std::path::PathBuf;

use super::windows_dir;
use crate::cleaners::simple::{always, files_info, SimpleFiles};
use crate::util::{children_of, Candidate};
use crate::{Cleaner, CleanerGroup, Risk};

fn dir() -> PathBuf {
    windows_dir().join("SoftwareDistribution").join("Download")
}

fn collect() -> Vec<Candidate> {
    children_of(&dir(), None, Some("Windows Update download"))
}

pub fn cleaner() -> Box<dyn Cleaner> {
    Box::new(SimpleFiles {
        info: files_info(
            "windows-update-cache",
            "Windows Update cache",
            "Deletes already-downloaded update files. Windows downloads them again if needed. Permanent; needs administrator permission.",
            CleanerGroup::System,
            Risk::Safe,
            true,
        ),
        available: || dir().is_dir(),
        roots: || vec![dir()],
        collect,
        needs_admin: always,
        force_permanent: true,
        post_clean: None,
    })
}
