//! The Cleaner trait every junk source implements.

use crate::{CleanOptions, CleanerInfo, CleanupPreview, CleanupResult, Result};

pub trait Cleaner: Send + Sync {
    fn info(&self) -> CleanerInfo;

    /// Cheap check: does this junk source exist on this machine (tool installed, folder present)?
    fn is_available(&self) -> bool;

    /// Measures what would be removed. Must not modify anything. May take seconds.
    fn preview(&self) -> Result<CleanupPreview>;

    /// Removes what preview() reports (re-evaluated now, not cached), via crate::safety.
    fn clean(&self, options: &CleanOptions) -> Result<CleanupResult>;
}
