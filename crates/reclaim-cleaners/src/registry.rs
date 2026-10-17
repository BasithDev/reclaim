//! The list of cleaners compiled for this OS.

use crate::Cleaner;

/// Every cleaner for the current OS whose is_available() is true, in display order.
pub fn all_cleaners() -> Vec<Box<dyn Cleaner>> {
    let mut list: Vec<Box<dyn Cleaner>> = Vec::new();
    #[cfg(target_os = "linux")]
    list.extend(crate::cleaners::linux::all());
    #[cfg(windows)]
    list.extend(crate::cleaners::windows::all());
    list.extend(crate::cleaners::common::all());
    list.into_iter().filter(|c| c.is_available()).collect()
}

pub fn find_cleaner(id: &str) -> Option<Box<dyn Cleaner>> {
    all_cleaners().into_iter().find(|c| c.info().id == id)
}
