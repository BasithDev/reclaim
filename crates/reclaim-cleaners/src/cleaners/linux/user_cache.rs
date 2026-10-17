//! Linux cleaner: the rest of ~/.cache, skipping thumbnails and caches owned by other cleaners.

use std::path::PathBuf;

use crate::cleaners::simple::{files_info, never, SimpleFiles};
use crate::util::{children_of, Candidate};
use crate::{Cleaner, CleanerGroup, Risk};

/// Cache folders handled by other cleaners (or that must never be touched).
const EXCLUDED: &[&str] = &[
    "thumbnails",
    "pip",
    "yarn",
    "go-build",
    "mozilla",
    "google-chrome",
    "google-chrome-beta",
    "chromium",
    "BraveSoftware",
    "microsoft-edge",
    "Code",
    "Cursor",
    "doc",
];

fn root() -> Option<PathBuf> {
    dirs::cache_dir().filter(|p| p.is_dir())
}

fn collect() -> Vec<Candidate> {
    let Some(r) = root() else { return Vec::new() };
    children_of(&r, None, Some("application cache"))
        .into_iter()
        .filter(|c| {
            let name = c.path.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
            !EXCLUDED.contains(&name.as_str())
        })
        .collect()
}

pub fn cleaner() -> Box<dyn Cleaner> {
    Box::new(SimpleFiles {
        info: files_info(
            "user-cache",
            "Application caches (~/.cache)",
            "Clears per-app cache folders in ~/.cache. Apps rebuild them, which can make the next start slower.",
            CleanerGroup::User,
            Risk::Moderate,
            false,
        ),
        available: || root().is_some(),
        roots: || root().into_iter().collect(),
        collect,
        needs_admin: never,
        force_permanent: false,
        post_clean: None,
    })
}
