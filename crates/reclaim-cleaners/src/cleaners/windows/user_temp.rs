//! Windows cleaner: files in the user's %TEMP% folder that are older than one day.

use std::path::PathBuf;

use crate::cleaners::simple::{files_info, never, SimpleFiles};
use crate::util::{children_of, days, home, Candidate};
use crate::{Cleaner, CleanerGroup, Risk};

/// %TEMP%, only if it is inside the user profile (never an arbitrary custom location).
fn temp_dir() -> Option<PathBuf> {
    let t = std::env::temp_dir();
    let h = home()?;
    (t.starts_with(&h) && t.is_dir()).then_some(t)
}

fn collect() -> Vec<Candidate> {
    temp_dir().map(|t| children_of(&t, Some(days(1)), Some("temporary file"))).unwrap_or_default()
}

pub fn cleaner() -> Box<dyn Cleaner> {
    Box::new(SimpleFiles {
        info: files_info(
            "user-temp",
            "User temporary files",
            "Removes files in your %TEMP% folder that are more than a day old. Files in use are skipped.",
            CleanerGroup::User,
            Risk::Safe,
            false,
        ),
        available: || temp_dir().is_some(),
        roots: || temp_dir().into_iter().collect(),
        collect,
        needs_admin: never,
        force_permanent: false,
        post_clean: None,
    })
}
