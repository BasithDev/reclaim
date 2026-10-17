//! Compares two directory-size listings and reports what grew or shrank.

use std::collections::HashMap;
use std::path::Path;

use crate::GrowthEntry;

/// Changed directories sorted by |delta| descending, at most `limit` entries.
/// Children are reported only where they explain the parent's change, so the
/// list points at the deepest folder that actually grew instead of every ancestor.
pub fn diff_sizes(older: &[(String, u64)], newer: &[(String, u64)], limit: usize) -> Vec<GrowthEntry> {
    let old: HashMap<&str, u64> = older.iter().map(|(p, b)| (p.as_str(), *b)).collect();
    let new: HashMap<&str, u64> = newer.iter().map(|(p, b)| (p.as_str(), *b)).collect();

    // Every changed path with its delta.
    let mut changed: Vec<(&str, u64, u64, i64)> = Vec::new();
    for (&p, &after) in &new {
        let before = old.get(p).copied().unwrap_or(0);
        if after != before {
            changed.push((p, before, after, after as i64 - before as i64));
        }
    }
    for (&p, &before) in &old {
        if !new.contains_key(p) && before != 0 {
            changed.push((p, before, 0, -(before as i64)));
        }
    }

    // Per parent: total growth / shrinkage contributed by its changed direct children.
    let mut child_sums: HashMap<String, (i64, i64)> = HashMap::new();
    for (p, _, _, d) in &changed {
        if let Some(parent) = Path::new(p).parent() {
            let e = child_sums.entry(parent.to_string_lossy().to_string()).or_insert((0, 0));
            if *d > 0 {
                e.0 += d;
            } else {
                e.1 += -d;
            }
        }
    }

    // Drop a directory when its changed children already explain at least 80% of its delta.
    let mut out: Vec<GrowthEntry> = changed
        .into_iter()
        .filter(|(p, _, _, d)| {
            let explained = match child_sums.get(*p) {
                Some((pos, neg)) => {
                    if *d > 0 {
                        *pos
                    } else {
                        *neg
                    }
                }
                None => 0,
            };
            explained.saturating_mul(5) < d.abs().saturating_mul(4)
        })
        .map(|(p, before, after, d)| GrowthEntry {
            path: p.to_string(),
            before_bytes: before,
            after_bytes: after,
            delta_bytes: d,
            is_new: !old.contains_key(p),
        })
        .collect();
    out.sort_by(|a, b| b.delta_bytes.abs().cmp(&a.delta_bytes.abs()).then_with(|| a.path.cmp(&b.path)));
    out.truncate(limit);
    out
}
