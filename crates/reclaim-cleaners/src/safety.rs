//! The only code allowed to delete anything. Enforces the protected-path deny list,
//! defaults to Trash/Recycle Bin, and runs privileged commands via pkexec (Linux) or UAC (Windows).

use std::path::Path;

use crate::Result;

/// True for paths that must never be removed: filesystem roots, OS directories
/// (/, /bin, /boot, /etc, /usr, C:\Windows, C:\Program Files, ...), the user's home itself, etc.
pub fn is_protected(path: &Path) -> bool {
    let _ = path;
    todo!("A2: implement")
}

/// Moves to Trash/Recycle Bin, or deletes when `permanent`. Refuses protected paths.
/// Returns bytes freed (allocated size measured before removal).
pub fn remove_path(path: &Path, permanent: bool) -> Result<u64> {
    let _ = (path, permanent);
    todo!("A2: implement")
}

/// Runs a command, elevated if `admin` (pkexec on Linux, UAC prompt on Windows).
/// Returns stdout. Err(ElevationDenied) if the user cancels the prompt.
pub fn run_command(program: &str, args: &[&str], admin: bool) -> Result<String> {
    let _ = (program, args, admin);
    todo!("A2: implement")
}
