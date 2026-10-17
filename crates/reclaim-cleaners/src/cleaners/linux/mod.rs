//! Linux-only cleaners (system logs, packages, Trash, caches) and their constructor list.

use crate::Cleaner;

mod apt_cache;
mod coredumps;
mod flatpak_unused;
mod journald_logs;
mod rotated_logs;
mod snap_old_revisions;
mod thumbnails;
mod trash;
mod user_cache;

/// Linux cleaners in display order (availability is filtered by the registry).
pub fn all() -> Vec<Box<dyn Cleaner>> {
    vec![
        Box::new(journald_logs::JournaldLogs),
        rotated_logs::cleaner(),
        Box::new(snap_old_revisions::SnapOldRevisions),
        Box::new(apt_cache::AptCache),
        Box::new(flatpak_unused::FlatpakUnused),
        coredumps::cleaner(),
        trash::cleaner(),
        thumbnails::cleaner(),
        user_cache::cleaner(),
    ]
}
