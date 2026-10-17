//! Windows implementations of the platform API in platform/mod.rs, built on windows-sys.
//! Cannot be compiled on Linux; keep the unsafe blocks small and well-commented.

use std::fs::{Metadata, OpenOptions};
use std::os::windows::ffi::OsStrExt;
use std::os::windows::fs::{MetadataExt, OpenOptionsExt};
use std::os::windows::io::AsRawHandle;
use std::path::Path;

use windows_sys::Win32::Foundation::HANDLE;
use windows_sys::Win32::Storage::FileSystem::{
    GetCompressedFileSizeW, GetFileInformationByHandle, BY_HANDLE_FILE_INFORMATION, FILE_ATTRIBUTE_COMPRESSED,
    FILE_ATTRIBUTE_REPARSE_POINT, FILE_ATTRIBUTE_SPARSE_FILE, FILE_FLAG_BACKUP_SEMANTICS,
};

const CLUSTER: u64 = 4096;

/// Real bytes on disk. Plain files are rounded up to a 4 KiB cluster (no syscall);
/// compressed or sparse files ask GetCompressedFileSizeW. Directories count as 0.
pub fn allocated_size(path: &Path, meta: &Metadata) -> u64 {
    if meta.is_dir() {
        return 0;
    }
    let len = meta.len();
    let attrs = meta.file_attributes();
    if attrs & (FILE_ATTRIBUTE_COMPRESSED | FILE_ATTRIBUTE_SPARSE_FILE) == 0 {
        return len.div_ceil(CLUSTER) * CLUSTER;
    }
    let wide: Vec<u16> = path.as_os_str().encode_wide().chain(std::iter::once(0)).collect();
    let mut high: u32 = 0;
    // SAFETY: `wide` is a valid NUL-terminated UTF-16 string and `high` is a valid out pointer.
    let low = unsafe { GetCompressedFileSizeW(wide.as_ptr(), &mut high) };
    if low == u32::MAX && std::io::Error::last_os_error().raw_os_error().unwrap_or(0) != 0 {
        return len;
    }
    ((high as u64) << 32) | low as u64
}

/// (volume serial, file index) when the file has more than one hard link.
pub fn file_id(path: &Path, meta: &Metadata) -> Option<(u64, u64)> {
    if meta.is_dir() {
        return None;
    }
    // Open with no access rights just to query attributes; backup semantics allows odd files.
    let file = OpenOptions::new()
        .access_mode(0)
        .share_mode(7)
        .custom_flags(FILE_FLAG_BACKUP_SEMANTICS)
        .open(path)
        .ok()?;
    // SAFETY: zeroed BY_HANDLE_FILE_INFORMATION is a valid plain-data struct and the handle is open.
    let mut info: BY_HANDLE_FILE_INFORMATION = unsafe { std::mem::zeroed() };
    let ok = unsafe { GetFileInformationByHandle(file.as_raw_handle() as HANDLE, &mut info) };
    if ok == 0 || info.nNumberOfLinks <= 1 {
        return None;
    }
    let index = ((info.nFileIndexHigh as u64) << 32) | info.nFileIndexLow as u64;
    Some((info.dwVolumeSerialNumber as u64, index))
}

/// Mount points are reparse points (never followed), so all scanned entries share one identity.
pub fn device_id(_path: &Path, _meta: &Metadata) -> u64 {
    0
}

/// Locations that are always access-denied and never useful.
pub fn is_skipped_path(path: &Path) -> bool {
    path.file_name()
        .map(|n| n.to_string_lossy().eq_ignore_ascii_case("System Volume Information"))
        .unwrap_or(false)
}

/// Symlinks, and junctions / mount points (directories that are reparse points).
/// Non-directory reparse files (OneDrive placeholders, dedup) are treated as regular files.
pub fn is_link_like(meta: &Metadata) -> bool {
    meta.file_type().is_symlink() || (meta.is_dir() && meta.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0)
}

/// Windows file names are case-insensitive.
pub fn names_equal(a: &str, b: &str) -> bool {
    a.eq_ignore_ascii_case(b) || a.to_lowercase() == b.to_lowercase()
}

/// True for the drive Windows is installed on (%SystemDrive%, default C:).
pub fn is_system_mount(mount_point: &str) -> bool {
    let sys = std::env::var("SystemDrive").unwrap_or_else(|_| "C:".to_string());
    let m = mount_point.trim_end_matches(['\\', '/']);
    m.eq_ignore_ascii_case(sys.trim_end_matches(['\\', '/']))
}

/// No hidden mounts on Windows.
pub fn is_hidden_mount(_mount_point: &str) -> bool {
    false
}
