//! Linux cleaner: deletes systemd crash dumps in /var/lib/systemd/coredump.

use std::path::PathBuf;

use crate::cleaners::simple::{always, files_info, SimpleFiles};
use crate::util::{children_of, Candidate};
use crate::{Cleaner, CleanerGroup, Risk};

const DIR: &str = "/var/lib/systemd/coredump";

fn collect() -> Vec<Candidate> {
    children_of(std::path::Path::new(DIR), None, Some("core dump"))
}

pub fn cleaner() -> Box<dyn Cleaner> {
    Box::new(SimpleFiles {
        info: files_info(
            "coredumps",
            "Crash dumps",
            "Deletes systemd core dumps left by crashed programs. Permanent. Needs administrator permission.",
            CleanerGroup::System,
            Risk::Safe,
            true,
        ),
        available: || PathBuf::from(DIR).is_dir(),
        roots: || vec![PathBuf::from(DIR)],
        collect,
        needs_admin: always,
        force_permanent: true,
        post_clean: None,
    })
}
