//! Data types that cross the cleaner/UI boundary (serialized to JSON for the UI).
//! Mirrored exactly in src/lib/types.ts; see docs/CONTRACT.md.

use serde::{Deserialize, Serialize};

/// Static description of a cleaner, shown as a row on the Cleanup page.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CleanerInfo {
    /// Stable kebab-case id, e.g. "journald-logs", "node-modules".
    pub id: String,
    pub name: String,
    /// One or two plain-language sentences: what it is and what cleaning does.
    pub description: String,
    pub group: CleanerGroup,
    pub risk: Risk,
    pub requires_admin: bool,
    pub method: CleanMethod,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum CleanerGroup {
    System,
    Developer,
    Applications,
    User,
}

/// Safe: regenerated automatically. Moderate: may cost a re-download/rebuild.
/// Careful: user data might be inside; preview must be reviewed.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum Risk {
    Safe,
    Moderate,
    Careful,
}

/// How cleaning works: moving files to trash/deleting them, or running the tool's own command.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum CleanMethod {
    Files,
    Command,
}

/// What a cleaner would remove right now.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CleanupPreview {
    pub cleaner_id: String,
    pub total_bytes: u64,
    pub item_count: u64,
    /// Largest items first, capped at 200 (total_bytes/item_count still cover all).
    pub items: Vec<PreviewItem>,
    /// For CleanMethod::Command: the exact command that will run, shown to the user.
    pub command: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PreviewItem {
    pub path: String,
    pub size_bytes: u64,
    pub modified_ms: Option<i64>,
    /// Short note, e.g. "node_modules of project foo" or "snap firefox rev 4793".
    pub label: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CleanOptions {
    /// false = move to Trash/Recycle Bin (default), true = delete permanently.
    #[serde(default)]
    pub permanent: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CleanupResult {
    pub cleaner_id: String,
    pub success: bool,
    pub freed_bytes: u64,
    pub items_removed: u64,
    /// Human-readable problems (skipped files, partial failures).
    pub errors: Vec<String>,
}

/// A deleted file still held open by a process: its space is not freed until the process closes it.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeletedOpenFile {
    pub pid: u32,
    pub process_name: String,
    pub path: String,
    pub size_bytes: u64,
}
