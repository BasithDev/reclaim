//! Windows cleaner: Explorer thumbnail cache databases (thumbcache_*.db).

use std::path::PathBuf;

use crate::cleaners::simple::{files_info, never, SimpleFiles};
use crate::util::{candidate, Candidate};
use crate::{Cleaner, CleanerGroup, Risk};

fn dir() -> Option<PathBuf> {
    dirs::cache_dir().map(|l| l.join("Microsoft").join("Windows").join("Explorer")).filter(|p| p.is_dir())
}

fn collect() -> Vec<Candidate> {
    let Some(d) = dir() else { return Vec::new() };
    let Ok(rd) = std::fs::read_dir(&d) else { return Vec::new() };
    rd.flatten()
        .filter(|e| {
            let n = e.file_name().to_string_lossy().to_lowercase();
            n.starts_with("thumbcache_") && n.ends_with(".db")
        })
        .filter_map(|e| candidate(&e.path(), Some("thumbnail cache".into())))
        .collect()
}

pub fn cleaner() -> Box<dyn Cleaner> {
    Box::new(SimpleFiles {
        info: files_info(
            "thumbnail-cache",
            "Thumbnail cache",
            "Removes Explorer's cached picture previews. Windows rebuilds them. Files in use are skipped.",
            CleanerGroup::User,
            Risk::Safe,
            false,
        ),
        available: || dir().is_some(),
        roots: || dir().into_iter().collect(),
        collect,
        needs_admin: never,
        force_permanent: false,
        post_clean: None,
    })
}
