//! Cleaner: `docker system prune -f` (stopped containers, dangling images, unused networks, build cache).

use crate::cleaners::simple::command_info;
use crate::safety::{run_command, run_command_timeout, LONG_TIMEOUT};
use crate::util::parse_size;
use crate::{CleanOptions, Cleaner, CleanerGroup, CleanerInfo, CleanupPreview, CleanupResult, Result, Risk};

const CMD: &str = "docker system prune -f";

pub struct Docker;

/// What `docker system prune -f` would reclaim: stopped containers + build cache (from
/// `docker system df`) + dangling images. Volumes are never pruned by this command.
fn reclaimable() -> Result<(u64, u64)> {
    let df = run_command("docker", &["system", "df", "--format", "{{.Type}}\t{{.Reclaimable}}"], false)?;
    let mut bytes = 0u64;
    let mut items = 0u64;
    for line in df.lines() {
        let mut parts = line.split('\t');
        let (kind, rec) = (parts.next().unwrap_or(""), parts.next().unwrap_or(""));
        if kind == "Containers" || kind == "Build Cache" {
            if let Some(b) = rec.split_whitespace().next().and_then(|t| parse_size(t, 1000.0)) {
                bytes = bytes.saturating_add(b);
                if b > 0 {
                    items += 1;
                }
            }
        }
    }
    let dangling = run_command("docker", &["images", "-f", "dangling=true", "--format", "{{.Size}}"], false)
        .unwrap_or_default();
    for line in dangling.lines() {
        if let Some(b) = parse_size(line.trim(), 1000.0) {
            bytes = bytes.saturating_add(b);
            items += 1;
        }
    }
    Ok((bytes, items))
}

impl Cleaner for Docker {
    fn info(&self) -> CleanerInfo {
        command_info(
            "docker",
            "Docker unused data",
            "Runs `docker system prune -f`: removes stopped containers, dangling images, unused networks and build cache. Volumes are kept.",
            CleanerGroup::Developer,
            Risk::Moderate,
            false,
        )
    }

    fn is_available(&self) -> bool {
        which::which("docker").is_ok()
    }

    fn preview(&self) -> Result<CleanupPreview> {
        let (bytes, items) = reclaimable()?;
        Ok(CleanupPreview {
            cleaner_id: "docker".into(),
            total_bytes: bytes,
            item_count: items,
            items: Vec::new(),
            command: Some(CMD.into()),
        })
    }

    fn clean(&self, _options: &CleanOptions) -> Result<CleanupResult> {
        let estimate = reclaimable().map(|r| r.0).unwrap_or(0);
        let out = run_command_timeout("docker", &["system", "prune", "-f"], false, LONG_TIMEOUT)?;
        let freed = out
            .lines()
            .find_map(|l| l.strip_prefix("Total reclaimed space:"))
            .and_then(|t| parse_size(t.trim(), 1000.0))
            .unwrap_or(estimate);
        Ok(CleanupResult {
            cleaner_id: "docker".into(),
            success: true,
            freed_bytes: freed,
            items_removed: 1,
            errors: Vec::new(),
        })
    }
}
