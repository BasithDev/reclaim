//! Linux cleaner: removes rotated and compressed logs (*.gz, *.1, *.old) under /var/log.

use std::path::PathBuf;

use walkdir::WalkDir;

use crate::cleaners::simple::{always, files_info, SimpleFiles};
use crate::util::{candidate, Candidate};
use crate::{Cleaner, CleanerGroup, Risk};

fn roots() -> Vec<PathBuf> {
    vec![PathBuf::from("/var/log")]
}

fn collect() -> Vec<Candidate> {
    let mut out = Vec::new();
    let walker = WalkDir::new("/var/log")
        .follow_links(false)
        .into_iter()
        .filter_entry(|e| e.file_name() != "journal");
    for entry in walker.flatten() {
        if !entry.file_type().is_file() {
            continue;
        }
        let name = entry.file_name().to_string_lossy();
        if name.ends_with(".gz") || name.ends_with(".1") || name.ends_with(".old") {
            if let Some(c) = candidate(entry.path(), Some("rotated log".into())) {
                out.push(c);
            }
        }
    }
    out
}

pub fn cleaner() -> Box<dyn Cleaner> {
    Box::new(SimpleFiles {
        info: files_info(
            "rotated-logs",
            "Rotated log files",
            "Deletes old rotated and compressed logs in /var/log (*.gz, *.1, *.old). Permanent. Needs administrator permission.",
            CleanerGroup::System,
            Risk::Safe,
            true,
        ),
        available: || PathBuf::from("/var/log").is_dir(),
        roots,
        collect,
        needs_admin: always,
        force_permanent: true,
        post_clean: None,
    })
}
