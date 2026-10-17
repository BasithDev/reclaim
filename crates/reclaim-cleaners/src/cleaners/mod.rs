//! All cleaner implementations, grouped by the OS they run on, plus the shared file-cleaner scaffold.

pub mod common;
#[cfg(target_os = "linux")]
pub mod linux;
pub mod simple;
#[cfg(windows)]
pub mod windows;
