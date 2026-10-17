//! Contract test: serializes a sample of every DTO that crosses the Rust/TS boundary to
//! src/lib/__tests__/fixtures/contract.json, which src/lib/__tests__/contract.test.ts checks against types.ts.

use reclaim_cleaners::*;
use reclaim_core::*;
use reclaim_lib::commands::app::get_app_info;
use reclaim_lib::commands::scan::ScanErrorPayload;
use reclaim_lib::settings::Settings;
use serde_json::{json, to_value, Value};

const ALL_CATEGORIES: [FileCategory; 11] = [
    FileCategory::Video, FileCategory::Audio, FileCategory::Image, FileCategory::Document, FileCategory::Archive,
    FileCategory::DiskImage, FileCategory::Code, FileCategory::Executable, FileCategory::Database,
    FileCategory::Log, FileCategory::Other,
];

fn tree_node() -> TreeNode {
    let child = TreeNode {
        path: "/r/a".into(), name: "a".into(), is_dir: false, size_bytes: 1, apparent_bytes: 1, file_count: 1,
        dir_count: 0, modified_ms: None, children: vec![], hidden_children: 0, other_bytes: 0,
    };
    TreeNode {
        path: "/r".into(), name: "r".into(), is_dir: true, size_bytes: 2, apparent_bytes: 2, file_count: 1,
        dir_count: 1, modified_ms: Some(1_700_000_000_000), children: vec![child], hidden_children: 3, other_bytes: 9,
    }
}

#[test]
fn write_contract_fixtures() {
    let disk = DiskInfo {
        id: "d".into(), name: "n".into(), mount_point: "/".into(), file_system: "ext4".into(), total_bytes: 10,
        used_bytes: 4, available_bytes: 6, is_removable: false, is_system: true, kind: DiskKind::Ssd,
    };
    let summary = ScanSummary {
        scan_id: "s".into(), root: "/".into(), total_bytes: 1, apparent_bytes: 1, file_count: 1, dir_count: 1,
        error_count: 0, started_at_ms: 1, finished_at_ms: 2, duration_ms: 1,
        type_breakdown: vec![TypeStat { category: FileCategory::Video, bytes: 1, count: 1 }], snapshot_id: Some(3),
    };
    let info = CleanerInfo {
        id: "x".into(), name: "X".into(), description: "d".into(), group: CleanerGroup::System, risk: Risk::Safe,
        requires_admin: false, method: CleanMethod::Files,
    };

    fn names<T: serde::Serialize>(v: &[T]) -> Vec<Value> {
        v.iter().map(|x| to_value(x).unwrap()).collect()
    }
    let enums = json!({
        "DiskKind": names(&[DiskKind::Ssd, DiskKind::Hdd, DiskKind::Unknown]),
        "FileCategory": names(&ALL_CATEGORIES),
        "CleanerGroup": names(&[CleanerGroup::System, CleanerGroup::Developer, CleanerGroup::Applications, CleanerGroup::User]),
        "Risk": names(&[Risk::Safe, Risk::Moderate, Risk::Careful]),
        "CleanMethod": names(&[CleanMethod::Files, CleanMethod::Command]),
    });

    let fixtures = json!({
        "DiskInfo": to_value(&disk).unwrap(),
        "ScanOptions": to_value(ScanOptions { root: "/".into(), cross_filesystems: true, excluded_paths: vec!["/x".into()], top_files_limit: 5 }).unwrap(),
        "ScanProgress": to_value(ScanProgress { scan_id: "s".into(), files_scanned: 1, dirs_scanned: 1, bytes_counted: 1, errors: 0, current_path: "/".into(), elapsed_ms: 1 }).unwrap(),
        "ScanSummary": to_value(&summary).unwrap(),
        "TypeStat": to_value(&summary.type_breakdown[0]).unwrap(),
        "TreeNode": to_value(tree_node()).unwrap(),
        "FileEntry": to_value(FileEntry { path: "/a".into(), name: "a".into(), size_bytes: 1, modified_ms: Some(1), extension: Some("txt".into()), category: FileCategory::Document }).unwrap(),
        "SnapshotMeta": to_value(SnapshotMeta { id: 1, root: "/".into(), taken_at_ms: 1, total_bytes: 1, file_count: 1, disk_used_bytes: Some(1), disk_total_bytes: Some(2) }).unwrap(),
        "GrowthEntry": to_value(GrowthEntry { path: "/a".into(), before_bytes: 1, after_bytes: 2, delta_bytes: 1, is_new: false }).unwrap(),
        "CleanerInfo": to_value(&info).unwrap(),
        "PreviewItem": to_value(PreviewItem { path: "/a".into(), size_bytes: 1, modified_ms: None, label: Some("l".into()) }).unwrap(),
        "CleanupPreview": to_value(CleanupPreview { cleaner_id: "x".into(), total_bytes: 1, item_count: 1, items: vec![PreviewItem { path: "/a".into(), size_bytes: 1, modified_ms: None, label: None }], command: Some("c".into()) }).unwrap(),
        "CleanupResult": to_value(CleanupResult { cleaner_id: "x".into(), success: true, freed_bytes: 1, items_removed: 1, errors: vec!["e".into()] }).unwrap(),
        "DeletedOpenFile": to_value(DeletedOpenFile { pid: 1, process_name: "p".into(), path: "/a".into(), size_bytes: 1 }).unwrap(),
        "Settings": to_value(Settings::default()).unwrap(),
        "AppInfo": to_value(get_app_info()).unwrap(),
        "ScanError": to_value(ScanErrorPayload { scan_id: "s".into(), message: "m".into(), cancelled: true }).unwrap(),
        "enums": enums,
    });

    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../src/lib/__tests__/fixtures/contract.json");
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(&path, serde_json::to_string_pretty(&fixtures).unwrap() + "\n").unwrap();

    // Sanity: JSON is camelCase everywhere (no snake_case keys).
    fn check(v: &Value) {
        match v {
            Value::Object(m) => m.iter().for_each(|(k, v)| { assert!(!k.contains('_'), "snake_case key {k}"); check(v) }),
            Value::Array(a) => a.iter().for_each(check),
            _ => {}
        }
    }
    check(&fixtures);
}
