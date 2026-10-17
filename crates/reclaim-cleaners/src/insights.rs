//! Read-only diagnostics that explain "missing" space but are not cleaners.

use crate::{DeletedOpenFile, Result};

/// Deleted files still held open by running processes (Linux: /proc/*/fd).
/// Returns an empty list on Windows, where the OS does not allow this situation.
pub fn deleted_open_files() -> Result<Vec<DeletedOpenFile>> {
    #[cfg(target_os = "linux")]
    {
        Ok(linux_scan())
    }
    #[cfg(not(target_os = "linux"))]
    {
        Ok(Vec::new())
    }
}

#[cfg(target_os = "linux")]
fn linux_scan() -> Vec<DeletedOpenFile> {
    use std::collections::HashSet;
    use std::os::unix::fs::MetadataExt;

    let mut out = Vec::new();
    let Ok(procs) = std::fs::read_dir("/proc") else { return out };
    for p in procs.flatten() {
        let Some(pid) = p.file_name().to_string_lossy().parse::<u32>().ok() else { continue };
        let fd_dir = p.path().join("fd");
        // Other users' processes are not readable without privileges; skip them silently.
        let Ok(fds) = std::fs::read_dir(&fd_dir) else { continue };
        let name = std::fs::read_to_string(p.path().join("comm")).map(|s| s.trim().to_string()).unwrap_or_default();
        let mut seen: HashSet<(u64, u64)> = HashSet::new();
        for fd in fds.flatten() {
            let Ok(target) = std::fs::read_link(fd.path()) else { continue };
            let t = target.to_string_lossy();
            let Some(path) = t.strip_suffix(" (deleted)") else { continue };
            // Real files only: skip memfd, shared memory, sockets and device nodes.
            if !path.starts_with('/') || path.starts_with("/memfd:") || path.starts_with("/dev/") {
                continue;
            }
            // stat through the magic link reads the open (deleted) inode.
            let Ok(md) = std::fs::metadata(fd.path()) else { continue };
            if !md.is_file() || !seen.insert((md.dev(), md.ino())) {
                continue;
            }
            let size = md.blocks() * 512;
            if size == 0 {
                continue;
            }
            out.push(DeletedOpenFile { pid, process_name: name.clone(), path: path.to_string(), size_bytes: size });
        }
    }
    out.sort_by_key(|f| std::cmp::Reverse(f.size_bytes));
    out
}
