//! A reusable Cleaner for "remove these files/folders" junk sources, so each cleaner file only
//! describes where its junk lives. All removal goes through `safety`.

use std::path::{Path, PathBuf};

use crate::safety;
use crate::util::{preview_from, push_error, result_for, Candidate};
use crate::{CleanError, CleanOptions, Cleaner, CleanerInfo, CleanupPreview, CleanupResult, Result};

pub struct SimpleFiles {
    pub info: CleanerInfo,
    /// Cheap availability check (a folder exists, a tool is installed).
    pub available: fn() -> bool,
    /// Folders the cleaner may remove things from. Candidates must lie strictly inside one.
    pub roots: fn() -> Vec<PathBuf>,
    /// Finds what to remove right now.
    pub collect: fn() -> Vec<Candidate>,
    /// Which candidates need an elevated, permanent removal.
    pub needs_admin: fn(&Path) -> bool,
    /// Always delete permanently (Trash contents, logs): moving to Trash would be pointless.
    pub force_permanent: bool,
    /// Optional follow-up after a clean (for example, removing orphaned metadata).
    pub post_clean: Option<fn()>,
}

pub fn never(_: &Path) -> bool {
    false
}

pub fn always(_: &Path) -> bool {
    true
}

impl Cleaner for SimpleFiles {
    fn info(&self) -> CleanerInfo {
        self.info.clone()
    }

    fn is_available(&self) -> bool {
        (self.available)()
    }

    fn preview(&self) -> Result<CleanupPreview> {
        Ok(preview_from(&self.info.id, (self.collect)(), None))
    }

    fn clean(&self, options: &CleanOptions) -> Result<CleanupResult> {
        let roots = (self.roots)();
        let permanent = self.force_permanent || options.permanent;
        let (admin, user): (Vec<Candidate>, Vec<Candidate>) =
            (self.collect)().into_iter().partition(|c| (self.needs_admin)(&c.path));
        let (mut freed, mut removed) = (0u64, 0u64);
        let mut errors = Vec::new();
        for c in &user {
            match safety::remove_path_in(&c.path, &roots, permanent) {
                Ok(b) => {
                    freed = freed.saturating_add(b);
                    removed += 1;
                }
                Err(e) => push_error(&mut errors, e.to_string()),
            }
        }
        if !admin.is_empty() {
            let paths: Vec<PathBuf> = admin.into_iter().map(|c| c.path).collect();
            match safety::remove_paths_elevated(&paths, &roots) {
                Ok(r) => {
                    freed = freed.saturating_add(r.freed);
                    removed += r.removed;
                    for e in r.errors {
                        push_error(&mut errors, e);
                    }
                }
                Err(CleanError::ElevationDenied) if removed == 0 => return Err(CleanError::ElevationDenied),
                Err(CleanError::ElevationDenied) => {
                    push_error(&mut errors, "administrator permission was cancelled; system items were skipped".into())
                }
                Err(e) => push_error(&mut errors, e.to_string()),
            }
        }
        if let Some(f) = self.post_clean {
            f();
        }
        Ok(result_for(&self.info.id, freed, removed, errors))
    }
}

/// Builds a CleanerInfo with `CleanMethod::Files`.
pub fn files_info(
    id: &str,
    name: &str,
    description: &str,
    group: crate::CleanerGroup,
    risk: crate::Risk,
    requires_admin: bool,
) -> CleanerInfo {
    CleanerInfo {
        id: id.into(),
        name: name.into(),
        description: description.into(),
        group,
        risk,
        requires_admin,
        method: crate::CleanMethod::Files,
    }
}

/// Builds a CleanerInfo with `CleanMethod::Command`.
pub fn command_info(
    id: &str,
    name: &str,
    description: &str,
    group: crate::CleanerGroup,
    risk: crate::Risk,
    requires_admin: bool,
) -> CleanerInfo {
    CleanerInfo { method: crate::CleanMethod::Command, ..files_info(id, name, description, group, risk, requires_admin) }
}
