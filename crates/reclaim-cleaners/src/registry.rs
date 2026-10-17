//! The list of cleaners compiled for this OS.

use crate::Cleaner;

/// Every cleaner for the current OS whose is_available() is true, in display order.
pub fn all_cleaners() -> Vec<Box<dyn Cleaner>> {
    todo!("A2: implement")
}

pub fn find_cleaner(id: &str) -> Option<Box<dyn Cleaner>> {
    all_cleaners().into_iter().find(|c| c.info().id == id)
}
