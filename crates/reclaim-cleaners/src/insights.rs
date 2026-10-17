//! Read-only diagnostics that explain "missing" space but are not cleaners.

use crate::{DeletedOpenFile, Result};

/// Deleted files still held open by running processes (Linux: /proc/*/fd).
/// Returns an empty list on Windows, where the OS does not allow this situation.
pub fn deleted_open_files() -> Result<Vec<DeletedOpenFile>> {
    todo!("A2: implement")
}
