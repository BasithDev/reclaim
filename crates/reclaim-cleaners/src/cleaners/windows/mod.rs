//! Windows-only cleaners (temp folders, update caches, Recycle Bin, dumps, GPU caches).

use std::path::PathBuf;

use crate::Cleaner;

mod crash_dumps;
mod delivery_optimization;
mod recycle_bin;
mod shader_cache;
mod thumbnail_cache;
mod user_temp;
mod windows_temp;
mod windows_update_cache;

/// The Windows directory (usually C:\Windows).
pub(crate) fn windows_dir() -> PathBuf {
    std::env::var_os("SystemRoot")
        .or_else(|| std::env::var_os("windir"))
        .map(PathBuf::from)
        .filter(|p| p.is_absolute())
        .unwrap_or_else(|| PathBuf::from(r"C:\Windows"))
}

/// Windows cleaners in display order (availability is filtered by the registry).
pub fn all() -> Vec<Box<dyn Cleaner>> {
    vec![
        user_temp::cleaner(),
        windows_temp::cleaner(),
        Box::new(recycle_bin::RecycleBin),
        windows_update_cache::cleaner(),
        delivery_optimization::cleaner(),
        crash_dumps::cleaner(),
        thumbnail_cache::cleaner(),
        shader_cache::cleaner(),
    ]
}
