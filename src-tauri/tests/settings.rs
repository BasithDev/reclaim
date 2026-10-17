//! Settings tests: defaults match CONTRACT.md, save/load round-trips, bad or partial files fall back safely.

use reclaim_lib::settings::Settings;

#[test]
fn defaults_match_contract() {
    let s = Settings::default();
    assert_eq!(s.theme, "system");
    assert!(s.excluded_paths.is_empty());
    assert!(!s.permanent_delete);
    assert!(s.confirm_before_clean);
    assert!(!s.cross_filesystems);
    assert_eq!(s.default_scan_root, None);
    assert!(s.save_snapshots);
}

#[test]
fn missing_file_gives_defaults() {
    let dir = tempfile::tempdir().unwrap();
    let s = Settings::load(&dir.path().join("nope.json"));
    assert_eq!(s.theme, "system");
}

#[test]
fn save_creates_parent_dirs_and_round_trips() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("a/b/settings.json");
    let s = Settings {
        theme: "dark".into(), excluded_paths: vec!["/x".into()], permanent_delete: true, confirm_before_clean: false,
        cross_filesystems: true, default_scan_root: Some("/data".into()), save_snapshots: false,
    };
    s.save(&path).unwrap();
    let back = Settings::load(&path);
    assert_eq!(back.theme, "dark");
    assert_eq!(back.excluded_paths, vec!["/x".to_string()]);
    assert!(back.permanent_delete && !back.confirm_before_clean && back.cross_filesystems && !back.save_snapshots);
    assert_eq!(back.default_scan_root.as_deref(), Some("/data"));
    let raw = std::fs::read_to_string(&path).unwrap();
    assert!(raw.contains("excludedPaths"), "file uses camelCase keys");
}

#[test]
fn corrupt_file_gives_defaults() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("s.json");
    std::fs::write(&path, "{not json").unwrap();
    assert!(Settings::load(&path).save_snapshots);
}

#[test]
fn partial_file_fills_missing_with_defaults() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("s.json");
    std::fs::write(&path, r#"{"theme":"light"}"#).unwrap();
    let s = Settings::load(&path);
    assert_eq!(s.theme, "light");
    assert!(s.confirm_before_clean && s.save_snapshots);
}
