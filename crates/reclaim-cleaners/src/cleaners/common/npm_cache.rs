//! Cleaner: npm download cache.

use std::path::PathBuf;

use crate::cleaners::simple::{files_info, never, SimpleFiles};
use crate::util::{children_of, Candidate};
use crate::{Cleaner, CleanerGroup, Risk};

/// Existing cache folders whose contents can be removed.
fn dirs_list() -> Vec<PathBuf> {
    let mut v: Vec<PathBuf> = Vec::new();
    if cfg!(windows) {
        if let Some(c) = dirs::cache_dir() {
            v.push(c.join("npm-cache").join("_cacache"));
        }
    } else if let Some(h) = dirs::home_dir() {
        v.push(h.join(".npm").join("_cacache"));
    }
    v.into_iter().filter(|p| p.is_dir()).collect()
}

fn collect() -> Vec<Candidate> {
    dirs_list().iter().flat_map(|d| children_of(d, None, Some("npm download cache"))).collect()
}

pub fn cleaner() -> Box<dyn Cleaner> {
    Box::new(SimpleFiles {
        info: files_info(
            "npm-cache",
            "npm cache",
            "Clears npm's download cache. Packages are downloaded again when needed.",
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
