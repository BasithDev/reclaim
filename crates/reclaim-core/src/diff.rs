//! Compares two directory-size listings and reports what grew or shrank.

use crate::GrowthEntry;

/// Changed directories sorted by |delta| descending, at most `limit` entries.
/// Children are reported only where they explain the parent's change, so the
/// list points at the deepest folder that actually grew instead of every ancestor.
pub fn diff_sizes(older: &[(String, u64)], newer: &[(String, u64)], limit: usize) -> Vec<GrowthEntry> {
    let _ = (older, newer, limit);
    todo!("A1: implement")
}
