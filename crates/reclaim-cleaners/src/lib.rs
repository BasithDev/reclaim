//! reclaim-cleaners: finds reclaimable junk and removes it safely.
//!
//! Each junk source (journald logs, snap revisions, Docker, dev caches, Trash,
//! %TEMP%, ...) is a `Cleaner`. The registry returns only the ones available on
//! this OS. Every removal goes through `safety`, which blocks protected paths and
//! sends files to the Trash/Recycle Bin unless permanent deletion is requested.
//! The public API here is fixed by docs/CONTRACT.md; change both together.

pub mod cleaner;
mod cleaners;
pub mod error;
pub mod insights;
pub mod registry;
pub mod safety;
pub mod types;
mod util;

pub use cleaner::Cleaner;
pub use error::{CleanError, Result};
pub use registry::{all_cleaners, find_cleaner};
pub use types::*;
