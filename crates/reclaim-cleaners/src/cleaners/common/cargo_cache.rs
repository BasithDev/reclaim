//! Cleaner: Cargo registry cache.

use std::path::PathBuf;

use crate::cleaners::simple::{files_info, never, SimpleFiles};
use crate::util::{children_of, Candidate};
use crate::{Cleaner, CleanerGroup, Risk};

/// Existing cache folders whose contents can be removed.
fn dirs_list() -> Vec<PathBuf> {
    let mut v: Vec<PathBuf> = Vec::new();
    let cargo_home = std::env::var_os("CARGO_HOME")
        .map(PathBuf::from)
        .filter(|p| p.is_absolute())
        .or_else(|| dirs::home_dir().map(|h| h.join(".cargo")));
    if let Some(c) = cargo_home {
        v.push(c.join("registry").join("cache"));
        v.push(c.join("git").join("db"));
    }
    v.into_iter().filter(|p| p.is_dir()).collect()
}

fn collect() -> Vec<Candidate> {
    dirs_list().iter().flat_map(|d| children_of(d, None, Some("Cargo registry cache"))).collect()
}

pub fn cleaner() -> Box<dyn Cleaner> {
    Box::new(SimpleFiles {
        info: files_info(
            "cargo-cache",
            "Cargo download cache",
            "Removes downloaded crate archives and git checkouts of dependencies. Cargo fetches them again on the next build.",
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
