//! Cleaner: pip download cache.

use std::path::PathBuf;

use crate::cleaners::simple::{files_info, never, SimpleFiles};
use crate::util::{children_of, Candidate};
use crate::{Cleaner, CleanerGroup, Risk};

/// Existing cache folders whose contents can be removed.
fn dirs_list() -> Vec<PathBuf> {
    let mut v: Vec<PathBuf> = Vec::new();
    if let Some(c) = dirs::cache_dir() {
        v.push(if cfg!(windows) { c.join("pip").join("Cache") } else { c.join("pip") });
    }
    v.into_iter().filter(|p| p.is_dir()).collect()
}

fn collect() -> Vec<Candidate> {
    dirs_list().iter().flat_map(|d| children_of(d, None, Some("pip download cache"))).collect()
}

pub fn cleaner() -> Box<dyn Cleaner> {
    Box::new(SimpleFiles {
        info: files_info(
            "pip-cache",
            "pip cache",
            "Clears pip's cache of downloaded packages and built wheels.",
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
