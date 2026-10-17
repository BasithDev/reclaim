//! Cleaner: items in the Downloads folder not modified for 90+ days (reviewed by the user first).

use std::path::PathBuf;

use crate::cleaners::simple::{files_info, never, SimpleFiles};
use crate::util::{children_of, days, Candidate};
use crate::{Cleaner, CleanerGroup, Risk};

fn downloads() -> Option<PathBuf> {
    // dirs returns None when ~/.config/user-dirs.dirs is missing; fall back to ~/Downloads.
    dirs::download_dir().or_else(|| dirs::home_dir().map(|h| h.join("Downloads"))).filter(|p| p.is_dir())
}

fn collect() -> Vec<Candidate> {
    let Some(d) = downloads() else { return Vec::new() };
    children_of(&d, Some(days(90)), Some("not modified for 90+ days"))
}

pub fn cleaner() -> Box<dyn Cleaner> {
    Box::new(SimpleFiles {
        info: files_info(
            "old-downloads",
            "Old downloads",
            "Moves files in Downloads that have not changed for 90+ days to the Trash. Review the list: these are your own files.",
            CleanerGroup::User,
            Risk::Careful,
            false,
        ),
        available: || downloads().is_some(),
        roots: || downloads().into_iter().collect(),
        collect,
        needs_admin: never,
        force_permanent: false,
        post_clean: None,
    })
}
