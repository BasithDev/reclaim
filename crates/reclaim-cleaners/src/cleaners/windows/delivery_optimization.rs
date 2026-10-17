//! Windows cleaner: the Delivery Optimization cache (peer-to-peer update downloads).

use std::path::PathBuf;

use super::windows_dir;
use crate::cleaners::simple::{always, files_info, SimpleFiles};
use crate::util::{children_of, Candidate};
use crate::{Cleaner, CleanerGroup, Risk};

fn dir() -> PathBuf {
    windows_dir()
        .join("ServiceProfiles")
        .join("NetworkService")
        .join("AppData")
        .join("Local")
        .join("Microsoft")
        .join("Windows")
        .join("DeliveryOptimization")
        .join("Cache")
}

fn collect() -> Vec<Candidate> {
    children_of(&dir(), None, Some("Delivery Optimization cache"))
}

pub fn cleaner() -> Box<dyn Cleaner> {
    Box::new(SimpleFiles {
        info: files_info(
            "delivery-optimization",
            "Delivery Optimization cache",
            "Deletes cached update files Windows shares with other PCs. Permanent; needs administrator permission.",
            CleanerGroup::System,
            Risk::Safe,
            true,
        ),
        // The folder is only readable by SYSTEM/admin; if it cannot be listed, it is hidden from the list.
        available: || dir().is_dir(),
        roots: || vec![dir()],
        collect,
        needs_admin: always,
        force_permanent: true,
        post_clean: None,
    })
}
