//! Windows cleaner: C:\Windows\Temp (needs administrator permission, deletes permanently).

use std::path::PathBuf;

use super::windows_dir;
use crate::cleaners::simple::{always, files_info, SimpleFiles};
use crate::util::{children_of, Candidate};
use crate::{Cleaner, CleanerGroup, Risk};

fn dir() -> PathBuf {
    windows_dir().join("Temp")
}

fn collect() -> Vec<Candidate> {
    children_of(&dir(), None, Some("system temporary file"))
}

pub fn cleaner() -> Box<dyn Cleaner> {
    Box::new(SimpleFiles {
        info: files_info(
            "windows-temp",
            "Windows temporary files",
            "Deletes files in C:\\Windows\\Temp. Permanent; needs administrator permission. Files in use are skipped.",
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
