//! Windows cleaner: GPU shader caches (DirectX, NVIDIA, AMD) under %LOCALAPPDATA%.

use std::path::PathBuf;

use crate::cleaners::simple::{files_info, never, SimpleFiles};
use crate::util::{children_of, Candidate};
use crate::{Cleaner, CleanerGroup, Risk};

fn dirs_list() -> Vec<PathBuf> {
    let Some(l) = dirs::cache_dir() else { return Vec::new() };
    ["D3DSCache", r"NVIDIA\DXCache", r"NVIDIA\GLCache", r"AMD\DxCache", r"AMD\GLCache"]
        .iter()
        .map(|rel| l.join(rel))
        .filter(|p| p.is_dir())
        .collect()
}

fn collect() -> Vec<Candidate> {
    dirs_list().iter().flat_map(|d| children_of(d, None, Some("shader cache"))).collect()
}

pub fn cleaner() -> Box<dyn Cleaner> {
    Box::new(SimpleFiles {
        info: files_info(
            "shader-cache",
            "GPU shader caches",
            "Clears DirectX, NVIDIA and AMD shader caches. Games rebuild them, so the first launch may stutter briefly.",
            CleanerGroup::Applications,
            Risk::Safe,
            false,
        ),
        available: || !dirs_list().is_empty(),
        roots: dirs_list,
        collect,
        needs_admin: never,
        force_permanent: false,
        post_clean: None,
    })
}
