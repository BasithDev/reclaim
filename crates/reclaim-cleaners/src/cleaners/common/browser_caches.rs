//! Cleaner: web browser cache folders (Chrome, Chromium, Edge, Brave, Firefox). Never profiles,
//! cookies, history or passwords: only a fixed allow-list of cache folder names is ever touched.

use std::path::{Path, PathBuf};

use crate::cleaners::simple::{files_info, never, SimpleFiles};
use crate::util::{candidate, Candidate};
use crate::{Cleaner, CleanerGroup, Risk};

/// The only folder names removed inside a Chromium profile or browser folder.
const CHROMIUM_CACHE_NAMES: &[&str] =
    &["Cache", "Code Cache", "GPUCache", "GrShaderCache", "ShaderCache", "DawnCache"];
/// The only folder names removed inside a Firefox profile.
const FIREFOX_CACHE_NAMES: &[&str] = &["cache2", "startupCache", "jumpListCache"];

/// Browser data folders to look in: (path, is_firefox).
fn bases() -> Vec<(PathBuf, bool)> {
    let mut v = Vec::new();
    if cfg!(windows) {
        if let Some(l) = dirs::cache_dir() {
            for rel in [
                r"Google\Chrome\User Data",
                r"Microsoft\Edge\User Data",
                r"BraveSoftware\Brave-Browser\User Data",
                r"Chromium\User Data",
            ] {
                v.push((l.join(rel), false));
            }
            v.push((l.join(r"Mozilla\Firefox\Profiles"), true));
        }
    } else {
        for root in [dirs::cache_dir(), dirs::config_dir()].into_iter().flatten() {
            for rel in ["google-chrome", "chromium", "BraveSoftware/Brave-Browser", "microsoft-edge"] {
                v.push((root.join(rel), false));
            }
        }
        if let Some(c) = dirs::cache_dir() {
            v.push((c.join("mozilla").join("firefox"), true));
        }
    }
    v.into_iter().filter(|(p, _)| p.is_dir()).collect()
}

fn cache_dirs_in(dir: &Path, names: &[&str], out: &mut Vec<Candidate>, label: &str) {
    for n in names {
        let p = dir.join(n);
        if std::fs::symlink_metadata(&p).is_ok_and(|m| m.is_dir() && !m.file_type().is_symlink()) {
            if let Some(c) = candidate(&p, Some(label.to_string())) {
                out.push(c);
            }
        }
    }
}

fn collect() -> Vec<Candidate> {
    let mut out = Vec::new();
    for (base, firefox) in bases() {
        let label = format!(
            "{} cache",
            base.components()
                .rev()
                .map(|c| c.as_os_str().to_string_lossy().into_owned())
                .find(|n| n != "User Data" && n != "firefox" && n != "Profiles")
                .unwrap_or_default()
        );
        let names = if firefox { FIREFOX_CACHE_NAMES } else { CHROMIUM_CACHE_NAMES };
        if !firefox {
            cache_dirs_in(&base, names, &mut out, &label);
        }
        if let Ok(rd) = std::fs::read_dir(&base) {
            for e in rd.flatten() {
                let is_dir = e.file_type().is_ok_and(|t| t.is_dir() && !t.is_symlink());
                if is_dir {
                    cache_dirs_in(&e.path(), names, &mut out, &label);
                }
            }
        }
    }
    out
}

pub fn cleaner() -> Box<dyn Cleaner> {
    Box::new(SimpleFiles {
        info: files_info(
            "browser-caches",
            "Browser caches",
            "Clears cached web content for Chrome, Chromium, Edge, Brave and Firefox. Logins, history and bookmarks are never touched. Close the browser first.",
            CleanerGroup::Applications,
            Risk::Safe,
            false,
        ),
        available: || !bases().is_empty(),
        roots: || bases().into_iter().map(|(p, _)| p).collect(),
        collect,
        needs_admin: never,
        force_permanent: false,
        post_clean: None,
    })
}
