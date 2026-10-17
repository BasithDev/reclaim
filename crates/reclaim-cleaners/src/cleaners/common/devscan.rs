//! Shared scanner for per-project build artefacts (node_modules, Cargo target) in the home folder.

use std::path::Path;
use std::time::{Duration, SystemTime};

use walkdir::WalkDir;

use crate::util::{candidate, home, is_older_than, Candidate};

/// Folders never descended into while looking for projects.
const SKIP: &[&str] = &[
    "AppData",
    "Application Data",
    "snap",
    "Library",
    "node_modules",
    "target",
    "venv",
    "__pycache__",
    "site-packages",
];

/// Folders ignored while measuring when a project was last worked on.
const ACTIVITY_SKIP: &[&str] =
    &["node_modules", "target", ".git", "dist", "build", ".next", ".venv", "venv", "__pycache__"];

fn project_activity(project: &Path) -> Option<SystemTime> {
    let mut newest: Option<SystemTime> = None;
    let mut bump = |t: Option<SystemTime>| {
        if let Some(t) = t {
            if newest.is_none_or(|n| t > n) {
                newest = Some(t);
            }
        }
    };
    let walker = WalkDir::new(project)
        .follow_links(false)
        .into_iter()
        .filter_entry(|e| e.depth() == 0 || !ACTIVITY_SKIP.contains(&e.file_name().to_string_lossy().as_ref()));
    for e in walker.flatten().take(5000) {
        bump(e.metadata().ok().and_then(|m| m.modified().ok()));
    }
    // Git activity (commits, checkouts) touches these even when no source file changes.
    for f in [".git/index", ".git/HEAD", ".git/FETCH_HEAD"] {
        bump(std::fs::metadata(project.join(f)).ok().and_then(|m| m.modified().ok()));
    }
    newest
}

fn artifact_activity(dir: &Path) -> Option<SystemTime> {
    let mut newest = std::fs::metadata(dir).ok().and_then(|m| m.modified().ok());
    if let Ok(rd) = std::fs::read_dir(dir) {
        for e in rd.flatten() {
            if let Some(t) = e.metadata().ok().and_then(|m| m.modified().ok()) {
                if newest.is_none_or(|n| t > n) {
                    newest = Some(t);
                }
            }
        }
    }
    newest
}

/// Finds `artifact_name` folders under the home dir whose parent satisfies `is_project` and which
/// pass `is_valid`, in projects with no activity for `min_age`. Never descends into matches.
pub fn find_artifacts(
    artifact_name: &str,
    is_project: fn(&Path) -> bool,
    is_valid: fn(&Path) -> bool,
    min_age: Duration,
    label_prefix: &str,
) -> Vec<Candidate> {
    let Some(root) = home() else { return Vec::new() };
    let mut out = Vec::new();
    let mut it = WalkDir::new(&root).max_depth(8).same_file_system(true).follow_links(false).into_iter();
    while let Some(next) = it.next() {
        let Ok(entry) = next else { continue };
        if entry.depth() == 0 || !entry.file_type().is_dir() {
            continue;
        }
        let name = entry.file_name().to_string_lossy();
        if name == artifact_name {
            it.skip_current_dir();
            let Some(project) = entry.path().parent() else { continue };
            if !is_project(project) || !is_valid(entry.path()) {
                continue;
            }
            let last = [project_activity(project), artifact_activity(entry.path())].into_iter().flatten().max();
            if !is_older_than(last, min_age) {
                continue;
            }
            let pname = project.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
            if let Some(c) = candidate(entry.path(), Some(format!("{label_prefix} of project {pname}"))) {
                out.push(c);
            }
        } else if name.starts_with('.') || SKIP.contains(&name.as_ref()) {
            it.skip_current_dir();
        }
    }
    out
}
