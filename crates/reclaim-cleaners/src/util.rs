//! Shared private helpers: allocated-size measurement, age checks, preview/clean plumbing for cleaners.

use std::fs::Metadata;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use walkdir::WalkDir;

use crate::{CleanupPreview, CleanupResult, PreviewItem};

/// Maximum number of items returned in a preview (totals still cover everything).
pub const MAX_PREVIEW_ITEMS: usize = 200;
/// Maximum number of individual error strings reported in a result.
const MAX_ERRORS: usize = 20;

/// One removable thing: a file or a directory (removed as a whole).
#[derive(Debug, Clone)]
pub struct Candidate {
    pub path: PathBuf,
    pub size: u64,
    pub modified_ms: Option<i64>,
    pub label: Option<String>,
}

#[cfg(unix)]
pub fn alloc_size(md: &Metadata) -> u64 {
    use std::os::unix::fs::MetadataExt;
    md.blocks() * 512
}

#[cfg(not(unix))]
pub fn alloc_size(md: &Metadata) -> u64 {
    md.len()
}

/// True for symlinks and (on Windows) junctions / other reparse points. Never followed.
pub fn is_link_like(md: &Metadata) -> bool {
    if md.file_type().is_symlink() {
        return true;
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        if md.file_attributes() & 0x400 != 0 {
            return true;
        }
    }
    false
}

/// Allocated bytes of everything under `path` (files only, symlinks not followed). 0 if missing.
pub fn dir_size(path: &Path) -> u64 {
    let mut total = 0u64;
    for entry in WalkDir::new(path).follow_links(false).into_iter().flatten() {
        if entry.file_type().is_dir() {
            continue;
        }
        if let Ok(md) = entry.metadata() {
            total = total.saturating_add(alloc_size(&md));
        }
    }
    total
}

/// Size of a file or directory tree, never following symlinks.
pub fn path_size(path: &Path) -> u64 {
    match std::fs::symlink_metadata(path) {
        Ok(md) if md.is_dir() && !is_link_like(&md) => dir_size(path),
        Ok(md) => alloc_size(&md),
        Err(_) => 0,
    }
}

pub fn modified_ms(md: &Metadata) -> Option<i64> {
    let t = md.modified().ok()?;
    let d = t.duration_since(SystemTime::UNIX_EPOCH).ok()?;
    Some(d.as_millis() as i64)
}

/// Newest modification time found in a tree (or of the file itself). Walks at most `cap` entries.
pub fn newest_mtime(path: &Path, cap: usize) -> Option<SystemTime> {
    let mut newest: Option<SystemTime> = None;
    for entry in WalkDir::new(path).follow_links(false).into_iter().flatten().take(cap) {
        if let Some(t) = entry.metadata().ok().and_then(|m| m.modified().ok()) {
            if newest.is_none_or(|n| t > n) {
                newest = Some(t);
            }
        }
    }
    newest
}

pub fn is_older_than(t: Option<SystemTime>, age: Duration) -> bool {
    match t {
        Some(t) => SystemTime::now().duration_since(t).map(|d| d >= age).unwrap_or(false),
        None => false,
    }
}

pub fn days(n: u64) -> Duration {
    Duration::from_secs(n * 86_400)
}

/// Builds a candidate for a file or directory. Returns None if it cannot be inspected.
pub fn candidate(path: &Path, label: Option<String>) -> Option<Candidate> {
    let md = std::fs::symlink_metadata(path).ok()?;
    let size = if md.is_dir() && !is_link_like(&md) { dir_size(path) } else { alloc_size(&md) };
    Some(Candidate { path: path.to_path_buf(), size, modified_ms: modified_ms(&md), label })
}

/// Direct children of `root` (not following symlinks). With `min_age`, only children whose newest
/// modification time (for directories: anywhere inside) is at least that old.
pub fn children_of(root: &Path, min_age: Option<Duration>, label: Option<&str>) -> Vec<Candidate> {
    let Ok(rd) = std::fs::read_dir(root) else { return Vec::new() };
    let mut out = Vec::new();
    for entry in rd.flatten() {
        let path = entry.path();
        if let Some(age) = min_age {
            let Ok(md) = std::fs::symlink_metadata(&path) else { continue };
            let t = if md.is_dir() && !is_link_like(&md) {
                newest_mtime(&path, 200_000)
            } else {
                md.modified().ok()
            };
            if !is_older_than(t, age) {
                continue;
            }
        }
        if let Some(c) = candidate(&path, label.map(str::to_string)) {
            out.push(c);
        }
    }
    out
}

/// Turns candidates into a CleanupPreview (largest first, capped).
pub fn preview_from(id: &str, mut cands: Vec<Candidate>, command: Option<String>) -> CleanupPreview {
    let total_bytes = cands.iter().fold(0u64, |a, c| a.saturating_add(c.size));
    let item_count = cands.len() as u64;
    cands.sort_by_key(|c| std::cmp::Reverse(c.size));
    cands.truncate(MAX_PREVIEW_ITEMS);
    CleanupPreview {
        cleaner_id: id.to_string(),
        total_bytes,
        item_count,
        items: cands
            .into_iter()
            .map(|c| PreviewItem {
                path: c.path.to_string_lossy().into_owned(),
                size_bytes: c.size,
                modified_ms: c.modified_ms,
                label: c.label,
            })
            .collect(),
        command,
    }
}

/// Collects errors, keeping the list short.
pub fn push_error(errors: &mut Vec<String>, msg: String) {
    if errors.len() < MAX_ERRORS {
        errors.push(msg);
    } else if errors.len() == MAX_ERRORS {
        errors.push("further errors omitted".to_string());
    }
}

pub fn result_for(id: &str, freed: u64, removed: u64, errors: Vec<String>) -> CleanupResult {
    CleanupResult {
        cleaner_id: id.to_string(),
        success: errors.is_empty() || removed > 0 || freed > 0,
        freed_bytes: freed,
        items_removed: removed,
        errors,
    }
}

/// Parses "1.5G", "142.3 MB", "512K", "10 bytes" into bytes. `base` is 1024 or 1000.
pub fn parse_size(text: &str, base: f64) -> Option<u64> {
    let t = text.trim();
    let split = t.find(|c: char| !(c.is_ascii_digit() || c == '.' || c == ','))?;
    let (num, unit) = t.split_at(split);
    let value: f64 = num.replace(',', ".").parse().ok()?;
    let unit = unit.trim().to_ascii_lowercase();
    if unit.starts_with("byte") {
        return Some(value as u64);
    }
    let mult = match unit.trim_end_matches("ib").trim_end_matches('b') {
        "" => 1.0,
        "k" => base,
        "m" => base * base,
        "g" => base * base * base,
        "t" => base * base * base * base,
        _ => return None,
    };
    Some((value * mult) as u64)
}

pub fn home() -> Option<PathBuf> {
    dirs::home_dir()
}
