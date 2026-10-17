//! reclaim-core: Reclaim's scanning engine.
//!
//! Lists disks, walks a directory tree in parallel to measure real on-disk usage,
//! keeps the tree in memory for drill-down queries, and saves snapshots so the
//! UI can show "what grew since last time". OS differences live in `platform`.
//! The public API here is fixed by docs/CONTRACT.md; change both together.

pub mod diff;
pub mod disks;
pub mod error;
pub mod platform;
pub mod scan;
pub mod snapshot;
pub mod types;

pub use error::{CoreError, Result};
pub use types::*;
