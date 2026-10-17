//! Command-layer tests: AppState::run_scan (what start_scan runs) on a tempdir fixture:
//! progress, complete, summary/tree/largest, cancel, keep-last-3, snapshot-after-scan and diff.

use reclaim_core::{CoreError, ScanOptions};
use reclaim_lib::settings::Settings;
use reclaim_lib::state::AppState;
use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

fn write(p: &Path, bytes: usize) {
    std::fs::create_dir_all(p.parent().unwrap()).unwrap();
    std::fs::write(p, vec![7u8; bytes]).unwrap();
}

fn fixture() -> tempfile::TempDir {
    let d = tempfile::tempdir().unwrap();
    write(&d.path().join("big.bin"), 200_000);
    write(&d.path().join("docs/a.txt"), 10_000);
    write(&d.path().join("docs/b.txt"), 20_000);
    write(&d.path().join("media/v.mp4"), 100_000);
    d
}

fn state(dir: &Path, save_snapshots: bool) -> AppState {
    let st = AppState::new(dir.join("cfg/settings.json"), dir.join("data/snapshots.db"));
    st.settings.lock().unwrap().clone_from(&Settings { save_snapshots, ..Settings::default() });
    st
}

fn opts(root: &Path) -> ScanOptions {
    ScanOptions { root: root.to_string_lossy().into(), cross_filesystems: false, excluded_paths: vec![], top_files_limit: 100 }
}

fn scan(st: &AppState, id: &str, root: &Path) -> Result<reclaim_core::ScanSummary, CoreError> {
    let flag = st.register_running(id);
    st.run_scan(id, opts(root), |_| {}, flag)
}

#[test]
fn full_lifecycle_summary_tree_largest() {
    let fx = fixture();
    let app = tempfile::tempdir().unwrap();
    let st = state(app.path(), true);
    let ticks = Arc::new(AtomicUsize::new(0));
    let t = ticks.clone();
    let flag = st.register_running("s1");
    let summary = st.run_scan("s1", opts(fx.path()), move |_| { t.fetch_add(1, Ordering::Relaxed); }, flag).unwrap();

    assert_eq!(summary.scan_id, "s1");
    assert_eq!(summary.file_count, 4);
    assert!(summary.total_bytes >= 330_000);
    assert!(summary.snapshot_id.is_some());
    assert!(st.cancel_flag("s1").is_none(), "running entry cleaned up");

    let got = st.get_scan("s1").unwrap();
    assert_eq!(got.summary.snapshot_id, summary.snapshot_id);

    let tree = got.result.subtree(None, 1, 10).unwrap();
    assert!(tree.is_dir);
    assert_eq!(tree.children[0].name, "big.bin", "children sorted by size desc");
    let docs = got.result.subtree(Some(&fx.path().join("docs").to_string_lossy()), 1, 10).unwrap();
    assert_eq!(docs.children.len(), 2);
    assert!(got.result.subtree(Some("/definitely/not/here"), 0, 1).is_none());

    let files = got.result.largest_files(2, None);
    assert_eq!(files.len(), 2);
    assert_eq!(files[0].name, "big.bin");
    assert!(files[0].size_bytes >= files[1].size_bytes);
    let video = got.result.largest_files(10, Some(reclaim_core::FileCategory::Video));
    assert_eq!(video.len(), 1);
    assert_eq!(video[0].name, "v.mp4");
}

#[test]
fn unknown_scan_id_errors() {
    let app = tempfile::tempdir().unwrap();
    assert!(state(app.path(), true).get_scan("nope").is_err());
}

#[test]
fn cancel_returns_cancelled_and_stores_nothing() {
    let fx = fixture();
    let app = tempfile::tempdir().unwrap();
    let st = state(app.path(), true);
    let flag = st.register_running("c1");
    flag.store(true, Ordering::Relaxed);
    let err = st.run_scan("c1", opts(fx.path()), |_| {}, flag).unwrap_err();
    assert!(matches!(err, CoreError::Cancelled));
    assert!(st.get_scan("c1").is_err());
    assert!(st.cancel_flag("c1").is_none());
    assert!(st.with_snapshots(|s| s.list()).unwrap().is_empty(), "no snapshot for a cancelled scan");
}

#[test]
fn keeps_only_last_three_scans() {
    let fx = fixture();
    let app = tempfile::tempdir().unwrap();
    let st = state(app.path(), false);
    for i in 0..5 {
        scan(&st, &format!("k{i}"), fx.path()).unwrap();
    }
    assert!(st.get_scan("k0").is_err() && st.get_scan("k1").is_err());
    for i in 2..5 {
        assert!(st.get_scan(&format!("k{i}")).is_ok());
    }
}

#[test]
fn snapshots_respect_setting() {
    let fx = fixture();
    let app = tempfile::tempdir().unwrap();
    let off = state(app.path(), false);
    assert!(scan(&off, "a", fx.path()).unwrap().snapshot_id.is_none());
    assert!(off.with_snapshots(|s| s.list()).unwrap().is_empty());

    let on = state(app.path(), true);
    let sm = scan(&on, "b", fx.path()).unwrap();
    let list = on.with_snapshots(|s| s.list()).unwrap();
    assert_eq!(list.len(), 1);
    assert_eq!(Some(list[0].id), sm.snapshot_id);
    assert_eq!(list[0].root, sm.root);
}

#[test]
fn diff_shows_growth_between_scans() {
    let fx = fixture();
    let app = tempfile::tempdir().unwrap();
    let st = state(app.path(), true);
    let first = scan(&st, "d1", fx.path()).unwrap();
    write(&fx.path().join("docs/new.bin"), 500_000);
    let second = scan(&st, "d2", fx.path()).unwrap();
    let (a, b) = (first.snapshot_id.unwrap(), second.snapshot_id.unwrap());
    assert!(b > a);

    let growth = st.with_snapshots(|s| s.diff(a, b, 10)).unwrap();
    assert!(!growth.is_empty());
    let docs = growth.iter().find(|g| g.path.ends_with("docs")).expect("docs reported as grown");
    assert!(docs.delta_bytes >= 400_000);
    assert!(!docs.is_new);

    st.with_snapshots(|s| s.delete(a)).unwrap();
    assert_eq!(st.with_snapshots(|s| s.list()).unwrap().len(), 1);
}

#[test]
fn nonexistent_root_errors_and_cleans_up() {
    let app = tempfile::tempdir().unwrap();
    let st = state(app.path(), true);
    let flag = st.register_running("e1");
    let r = st.run_scan("e1", opts(&app.path().join("missing")), |_| {}, flag);
    assert!(r.is_err());
    assert!(st.cancel_flag("e1").is_none());
}

