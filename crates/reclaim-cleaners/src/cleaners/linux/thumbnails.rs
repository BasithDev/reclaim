//! Linux cleaner: clears the freedesktop thumbnail cache (~/.cache/thumbnails).

use std::path::PathBuf;

use crate::cleaners::simple::{files_info, never, SimpleFiles};
use crate::util::{children_of, Candidate};
use crate::{Cleaner, CleanerGroup, Risk};

fn dirs_list() -> Vec<PathBuf> {
    let mut v = Vec::new();
    if let Some(c) = dirs::cache_dir() {
        v.push(c.join("thumbnails"));
    }
    if let Some(h) = dirs::home_dir() {
        v.push(h.join(".thumbnails"));
    }
    v.into_iter().filter(|p| p.is_dir()).collect()
}

fn collect() -> Vec<Candidate> {
    dirs_list().iter().flat_map(|d| children_of(d, None, Some("thumbnails"))).collect()
}

pub fn cleaner() -> Box<dyn Cleaner> {
    Box::new(SimpleFiles {
        info: files_info(
            "thumbnails",
            "Thumbnail cache",
            "Removes cached image previews. They are regenerated when you browse folders again.",
            CleanerGroup::User,
            Risk::Safe,
            false,
        ),
        available: || !dirs_list().is_empty(),
        roots: dirs_list,
        collect,
        needs_admin: never,
        force_permanent: false,
        post_clean: None,
    })
}
