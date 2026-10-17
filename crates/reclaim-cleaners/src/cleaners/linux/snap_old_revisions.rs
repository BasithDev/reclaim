//! Linux cleaner: removes disabled (old) snap revisions in one elevated batch.

use std::path::{Path, PathBuf};

use crate::safety::{run_command, run_command_timeout, LONG_TIMEOUT};
use crate::util::{modified_ms, preview_from, push_error, result_for, Candidate};
use crate::{CleanOptions, Cleaner, CleanerGroup, CleanerInfo, CleanupPreview, CleanupResult, Result, Risk};

/// Fixed script: argv is `sh NAME REV NAME REV ...`; nothing is interpolated into the script text.
const SCRIPT: &str =
    r#"rc=0; while [ "$#" -ge 2 ]; do snap remove "$1" --revision="$2" || rc=1; shift 2; done; exit $rc"#;

pub struct SnapOldRevisions;

struct Revision {
    name: String,
    rev: String,
    file: PathBuf,
    size: u64,
}

fn valid_name(n: &str) -> bool {
    !n.is_empty() && n.len() <= 40 && n.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
}

fn valid_rev(r: &str) -> bool {
    !r.is_empty() && r.len() <= 10 && r.chars().all(|c| c.is_ascii_digit())
}

/// Parses `snap list --all` and keeps lines flagged "disabled".
fn disabled_revisions() -> Result<Vec<Revision>> {
    let out = run_command("snap", &["list", "--all"], false)?;
    let mut revs = Vec::new();
    for line in out.lines().skip(1) {
        let cols: Vec<&str> = line.split_whitespace().collect();
        // Name Version Rev Tracking Publisher Notes
        if cols.len() < 6 || !cols[5..].iter().any(|n| n.split(',').any(|x| x == "disabled")) {
            continue;
        }
        let (name, rev) = (cols[0], cols[2]);
        if !valid_name(name) || !valid_rev(rev) {
            continue;
        }
        let file = Path::new("/var/lib/snapd/snaps").join(format!("{name}_{rev}.snap"));
        let size = std::fs::symlink_metadata(&file).map(|m| crate::util::alloc_size(&m)).unwrap_or(0);
        revs.push(Revision { name: name.to_string(), rev: rev.to_string(), file, size });
    }
    Ok(revs)
}

impl Cleaner for SnapOldRevisions {
    fn info(&self) -> CleanerInfo {
        crate::cleaners::simple::command_info(
            "snap-old-revisions",
            "Old snap revisions",
            "Removes disabled older revisions of installed snaps (the current version is kept). One administrator prompt.",
            CleanerGroup::System,
            Risk::Safe,
            true,
        )
    }

    fn is_available(&self) -> bool {
        which::which("snap").is_ok()
    }

    fn preview(&self) -> Result<CleanupPreview> {
        let cands = disabled_revisions()?
            .into_iter()
            .map(|r| Candidate {
                modified_ms: std::fs::symlink_metadata(&r.file).ok().and_then(|m| modified_ms(&m)),
                label: Some(format!("snap {} rev {}", r.name, r.rev)),
                path: r.file,
                size: r.size,
            })
            .collect();
        Ok(preview_from(
            "snap-old-revisions",
            cands,
            Some("pkexec sh -c '<for each disabled revision: snap remove NAME --revision=REV>'".into()),
        ))
    }

    fn clean(&self, _options: &CleanOptions) -> Result<CleanupResult> {
        let revs = disabled_revisions()?;
        if revs.is_empty() {
            return Ok(result_for("snap-old-revisions", 0, 0, Vec::new()));
        }
        let mut args: Vec<&str> = vec!["-c", SCRIPT, "sh"];
        for r in &revs {
            args.push(&r.name);
            args.push(&r.rev);
        }
        let mut errors = Vec::new();
        match run_command_timeout("sh", &args, true, LONG_TIMEOUT) {
            Ok(_) => {}
            Err(crate::CleanError::ElevationDenied) => return Err(crate::CleanError::ElevationDenied),
            Err(e) => push_error(&mut errors, e.to_string()),
        }
        // Verify against what is still disabled.
        let remaining: Vec<(String, String)> =
            disabled_revisions().unwrap_or_default().into_iter().map(|r| (r.name, r.rev)).collect();
        let (mut freed, mut removed) = (0u64, 0u64);
        for r in &revs {
            if !remaining.contains(&(r.name.clone(), r.rev.clone())) {
                freed = freed.saturating_add(r.size);
                removed += 1;
            }
        }
        Ok(result_for("snap-old-revisions", freed, removed, errors))
    }
}
