//! Cleaner: Yarn package cache.

use std::path::PathBuf;

use crate::cleaners::simple::{files_info, never, SimpleFiles};
use crate::util::{children_of, Candidate};
use crate::{Cleaner, CleanerGroup, Risk};

/// Existing cache folders whose contents can be removed.
fn dirs_list() -> Vec<PathBuf> {
    let mut v: Vec<PathBuf> = Vec::new();
    if cfg!(windows) {
        if let Some(c) = dirs::cache_dir() {
            v.push(c.join("Yarn").join("Cache"));
        }
    } else {
        if let Some(c) = dirs::cache_dir() {
            v.push(c.join("yarn"));
        }
        if let Some(h) = dirs::home_dir() {
            v.push(h.join(".yarn").join("berry").join("cache"));
        }
    }
    v.into_iter().filter(|p| p.is_dir()).collect()
}

fn collect() -> Vec<Candidate> {
    dirs_list().iter().flat_map(|d| children_of(d, None, Some("Yarn package cache"))).collect()
}

pub fn cleaner() -> Box<dyn Cleaner> {
    Box::new(SimpleFiles {
        info: files_info(
            "yarn-cache",
            "Yarn cache",
            "Clears Yarn's package cache. Packages are downloaded again when needed.",
            CleanerGroup::Developer,
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
