//! Cleaner: Cargo `target/` folders next to a Cargo.toml in projects untouched for 30+ days.


use super::devscan::find_artifacts;
use crate::cleaners::simple::{files_info, never, SimpleFiles};
use crate::util::{days, home, Candidate};
use crate::{Cleaner, CleanerGroup, Risk};

fn collect() -> Vec<Candidate> {
    find_artifacts(
        "target",
        |p| p.join("Cargo.toml").is_file(),
        // Cargo marks its target dirs; this keeps unrelated folders named "target" safe.
        |t| t.join("CACHEDIR.TAG").is_file() || t.join(".rustc_info.json").is_file(),
        days(30),
        "Cargo target",
    )
}

pub fn cleaner() -> Box<dyn Cleaner> {
    Box::new(SimpleFiles {
        info: files_info(
            "rust-targets",
            "Old Rust build folders",
            "Removes Cargo target/ folders in projects you have not touched for 30+ days. The next cargo build recompiles them.",
            CleanerGroup::Developer,
            Risk::Moderate,
            false,
        ),
        available: || home().is_some(),
        roots: || home().map(|h| vec![h]).unwrap_or_default(),
        collect,
        needs_admin: never,
        force_permanent: false,
        post_clean: None,
    })
}
