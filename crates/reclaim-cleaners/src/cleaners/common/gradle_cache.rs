//! Cleaner: Gradle caches.

use std::path::PathBuf;

use crate::cleaners::simple::{files_info, never, SimpleFiles};
use crate::util::{children_of, Candidate};
use crate::{Cleaner, CleanerGroup, Risk};

/// Existing cache folders whose contents can be removed.
fn dirs_list() -> Vec<PathBuf> {
    let mut v: Vec<PathBuf> = Vec::new();
    let gradle_home = std::env::var_os("GRADLE_USER_HOME")
        .map(PathBuf::from)
        .filter(|p| p.is_absolute())
        .or_else(|| dirs::home_dir().map(|h| h.join(".gradle")));
    if let Some(g) = gradle_home {
        v.push(g.join("caches"));
    }
    v.into_iter().filter(|p| p.is_dir()).collect()
}

fn collect() -> Vec<Candidate> {
    dirs_list().iter().flat_map(|d| children_of(d, None, Some("Gradle caches"))).collect()
}

pub fn cleaner() -> Box<dyn Cleaner> {
    Box::new(SimpleFiles {
        info: files_info(
            "gradle-cache",
            "Gradle caches",
            "Removes Gradle dependency and build caches. The next build downloads and rebuilds them, which can be slow.",
            CleanerGroup::Developer,
            Risk::Moderate,
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
