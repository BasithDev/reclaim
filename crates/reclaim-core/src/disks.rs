//! Lists mounted drives/partitions with capacity and usage.
//! Skips pseudo filesystems (proc, sysfs, tmpfs, squashfs snap mounts, overlay, ...).

use crate::{DiskInfo, Result};

/// All real disks/partitions visible to the user, system drive first.
pub fn list_disks() -> Result<Vec<DiskInfo>> {
    todo!("A1: implement")
}

/// The disk that contains `path` (longest matching mount point), if any.
pub fn disk_for_path(path: &str) -> Result<Option<DiskInfo>> {
    let _ = path;
    todo!("A1: implement")
}
