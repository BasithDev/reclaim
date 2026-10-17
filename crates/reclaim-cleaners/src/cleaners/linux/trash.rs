//! Linux cleaner: permanently empties the user's Trash (~/.local/share/Trash).

use std::path::PathBuf;

use crate::cleaners::simple::{files_info, never, SimpleFiles};
use crate::safety::remove_path_in;
use crate::util::{children_of, Candidate};
use crate::{Cleaner, CleanerGroup, Risk};

fn trash_dir() -> Option<PathBuf> {
    dirs::data_dir().map(|d| d.join("Trash"))
}

fn roots() -> Vec<PathBuf> {
    trash_dir().into_iter().collect()
}

fn collect() -> Vec<Candidate> {
    let Some(t) = trash_dir() else { return Vec::new() };
    children_of(&t.join("files"), None, Some("trashed item"))
}

/// Removes the metadata entries (info/, expunged/) left after the files were deleted.
fn post_clean() {
    let Some(t) = trash_dir() else { return };
    for sub in ["info", "expunged"] {
        if let Ok(rd) = std::fs::read_dir(t.join(sub)) {
            for e in rd.flatten() {
                let _ = remove_path_in(&e.path(), std::slice::from_ref(&t), true);
            }
        }
    }
}

pub fn cleaner() -> Box<dyn Cleaner> {
    Box::new(SimpleFiles {
        info: files_info(
            "trash",
            "Trash",
            "Permanently empties your Trash. Items in it cannot be restored afterwards.",
            CleanerGroup::User,
            Risk::Careful,
            false,
        ),
        available: || trash_dir().is_some_and(|t| t.is_dir()),
        roots,
        collect,
        needs_admin: never,
        force_permanent: true,
        post_clean: Some(post_clean),
    })
}
