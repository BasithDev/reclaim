//! Lists mounted drives/partitions with capacity and usage.
//! Skips pseudo filesystems (proc, sysfs, tmpfs, squashfs snap mounts, overlay, ...).

use std::path::Path;

use sysinfo::Disks;

use crate::{platform, CoreError, DiskInfo, DiskKind, Result};

/// Filesystem types that never represent a user-visible drive.
const PSEUDO_FS: &[&str] = &[
    "proc", "sysfs", "tmpfs", "devtmpfs", "devpts", "squashfs", "overlay", "overlayfs", "cgroup", "cgroup2",
    "securityfs", "debugfs", "tracefs", "configfs", "fusectl", "pstore", "bpf", "autofs", "mqueue", "hugetlbfs",
    "binfmt_misc", "ramfs", "efivarfs", "nsfs", "rpc_pipefs", "fuse.portal", "fuse.gvfsd-fuse", "fuse.snapfuse",
    "fuse.lxcfs", "iso9660", "9p", "drvfs",
];

/// All real disks/partitions visible to the user, system drive first.
pub fn list_disks() -> Result<Vec<DiskInfo>> {
    let sys = Disks::new_with_refreshed_list();
    let mut out: Vec<DiskInfo> = Vec::new();
    // device name -> index in `out`, so bind mounts / subvolumes of one device appear once.
    let mut seen: Vec<(String, u64, usize)> = Vec::new();

    for d in sys.list() {
        let mount = d.mount_point().to_string_lossy().to_string();
        let fs = d.file_system().to_string_lossy().to_string();
        let total = d.total_space();
        if total == 0 || PSEUDO_FS.contains(&fs.as_str()) || platform::is_hidden_mount(&mount) {
            continue;
        }
        let device = d.name().to_string_lossy().to_string();
        let available = d.available_space().min(total);
        let name = if device.is_empty() || (cfg!(unix) && device.starts_with("/dev/")) {
            mount.clone()
        } else {
            device.clone()
        };
        let info = DiskInfo {
            id: mount.clone(),
            name,
            mount_point: mount.clone(),
            file_system: fs,
            total_bytes: total,
            used_bytes: total - available,
            available_bytes: available,
            is_removable: d.is_removable(),
            is_system: platform::is_system_mount(&mount),
            kind: match d.kind() {
                sysinfo::DiskKind::SSD => DiskKind::Ssd,
                sysinfo::DiskKind::HDD => DiskKind::Hdd,
                _ => DiskKind::Unknown,
            },
        };
        if !device.is_empty() {
            if let Some((_, _, i)) = seen.iter().find(|(n, t, _)| *n == device && *t == total) {
                // Keep the shortest mount point for a device mounted several times.
                if info.mount_point.len() < out[*i].mount_point.len() {
                    out[*i] = info;
                }
                continue;
            }
            seen.push((device, total, out.len()));
        }
        out.push(info);
    }
    out.sort_by(|a, b| b.is_system.cmp(&a.is_system).then_with(|| a.mount_point.cmp(&b.mount_point)));
    Ok(out)
}

/// The disk that contains `path` (longest matching mount point), if any.
pub fn disk_for_path(path: &str) -> Result<Option<DiskInfo>> {
    let disks = list_disks().map_err(|e| CoreError::Other(e.to_string()))?;
    let p = Path::new(path);
    Ok(disks
        .into_iter()
        .filter(|d| p.starts_with(&d.mount_point))
        .max_by_key(|d| Path::new(&d.mount_point).components().count()))
}
