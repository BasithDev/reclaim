//! Cleaner: VS Code / Cursor cache folders (CachedData, CachedExtensionVSIXs, logs).

use std::path::PathBuf;

use crate::cleaners::simple::{files_info, never, SimpleFiles};
use crate::util::{candidate, Candidate};
use crate::{Cleaner, CleanerGroup, Risk};

const EDITORS: &[&str] = &["Code", "Code - Insiders", "Cursor", "VSCodium"];
const CACHE_NAMES: &[&str] = &["CachedData", "CachedExtensionVSIXs", "logs"];

fn bases() -> Vec<PathBuf> {
    let Some(cfg) = dirs::config_dir() else { return Vec::new() };
    EDITORS.iter().map(|e| cfg.join(e)).filter(|p| p.is_dir()).collect()
}

fn collect() -> Vec<Candidate> {
    let mut out = Vec::new();
    for base in bases() {
        let editor = base.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
        for n in CACHE_NAMES {
            let p = base.join(n);
            if std::fs::symlink_metadata(&p).is_ok_and(|m| m.is_dir() && !m.file_type().is_symlink()) {
                if let Some(c) = candidate(&p, Some(format!("{editor} {n}"))) {
                    out.push(c);
                }
            }
        }
    }
    out
}

pub fn cleaner() -> Box<dyn Cleaner> {
    Box::new(SimpleFiles {
        info: files_info(
            "vscode-caches",
            "VS Code / Cursor caches",
            "Clears cached code, downloaded extension packages and old logs. Settings and extensions are kept.",
            CleanerGroup::Applications,
            Risk::Safe,
            false,
        ),
        available: || !bases().is_empty(),
        roots: bases,
        collect,
        needs_admin: never,
        force_permanent: false,
        post_clean: None,
    })
}
