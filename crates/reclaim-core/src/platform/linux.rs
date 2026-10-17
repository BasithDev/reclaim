//! Linux (and other Unix) implementations of the platform API in platform/mod.rs.
//! Uses `std::os::unix::fs::MetadataExt`; no unsafe code or libc needed.

use std::fs::Metadata;
use std::os::unix::fs::MetadataExt;
use std::path::Path;

/// Real bytes on disk: st_blocks is always in 512-byte units.
pub fn allocated_size(_path: &Path, meta: &Metadata) -> u64 {
    meta.blocks().saturating_mul(512)
}

/// (device, inode) for non-directories with more than one hard link.
pub fn file_id(_path: &Path, meta: &Metadata) -> Option<(u64, u64)> {
    if meta.nlink() > 1 && !meta.is_dir() {
        Some((meta.dev(), meta.ino()))
    } else {
        None
    }
}

/// st_dev of the filesystem holding the entry.
pub fn device_id(_path: &Path, meta: &Metadata) -> u64 {
    meta.dev()
}

/// Virtual kernel filesystems that are never worth scanning.
pub fn is_skipped_path(path: &Path) -> bool {
    ["/proc", "/sys", "/dev", "/run"].iter().any(|p| path == Path::new(p))
}

/// Symlinks are never followed.
pub fn is_link_like(meta: &Metadata) -> bool {
    meta.file_type().is_symlink()
}

/// Case-sensitive file name comparison on Linux.
pub fn names_equal(a: &str, b: &str) -> bool {
    a == b
}

/// True for the OS drive mount ("/").
pub fn is_system_mount(mount_point: &str) -> bool {
    mount_point == "/"
}

/// Mounts that are implementation details (snap, containers, kernel) rather than user disks.
pub fn is_hidden_mount(mount_point: &str) -> bool {
    const PREFIXES: [&str; 8] = [
        "/snap", "/var/snap", "/var/lib/snapd", "/var/lib/docker", "/var/lib/containers", "/proc", "/sys", "/run",
    ];
    PREFIXES.iter().any(|p| Path::new(mount_point).starts_with(p)) || Path::new(mount_point).starts_with("/dev")
}
