//! OS-specific primitives behind one API. Exactly one of linux.rs / windows.rs is compiled.
//! Nothing outside this module may use cfg(target_os) for filesystem details.

#[cfg(unix)]
mod linux;
#[cfg(unix)]
pub use linux::*;

#[cfg(windows)]
mod windows;
#[cfg(windows)]
pub use windows::*;

// Both platform files must export:
//   pub fn allocated_size(path: &std::path::Path, meta: &std::fs::Metadata) -> u64
//       Real bytes on disk (Linux: st_blocks * 512; Windows: GetCompressedFileSizeW).
//   pub fn file_id(path: &std::path::Path, meta: &std::fs::Metadata) -> Option<(u64, u64)>
//       (device, inode) / (volume serial, file index); Some only when link count > 1.
//   pub fn device_id(path: &std::path::Path, meta: &std::fs::Metadata) -> u64
//       Filesystem identity, used to avoid crossing mount points.
//   pub fn is_skipped_path(path: &std::path::Path) -> bool
//       Pseudo/virtual locations never worth scanning (/proc, /sys, /dev, /run, ...).
//   pub fn is_link_like(meta: &std::fs::Metadata) -> bool
//       Symlinks, and on Windows junctions/reparse points.
