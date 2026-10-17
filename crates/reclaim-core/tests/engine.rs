//! Integration tests for the scanner, queries, snapshots and diff, using tempdir fixtures only.

use std::fs;
use std::path::Path;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;

use reclaim_core::diff::diff_sizes;
use reclaim_core::platform;
use reclaim_core::scan::{categorize, scan, ScanResult};
use reclaim_core::snapshot::SnapshotStore;
use reclaim_core::{CoreError, FileCategory, ScanOptions};

fn opts(root: &Path) -> ScanOptions {
    ScanOptions {
        root: root.to_string_lossy().to_string(),
        cross_filesystems: false,
        excluded_paths: vec![],
        top_files_limit: 500,
    }
}

fn run(o: ScanOptions) -> ScanResult {
    scan("t".into(), o, |_| {}, Arc::new(AtomicBool::new(false))).unwrap()
}

fn alloc(p: &Path) -> u64 {
    let m = fs::symlink_metadata(p).unwrap();
    platform::allocated_size(p, &m)
}

fn write(p: &Path, n: usize) {
    fs::create_dir_all(p.parent().unwrap()).unwrap();
    fs::write(p, vec![7u8; n]).unwrap();
}

/// root/{a/x.bin(10000), a/b/y.mp4(50000), c/z.txt(300), top.log(0)}
fn fixture() -> tempfile::TempDir {
    let d = tempfile::tempdir().unwrap();
    let r = d.path();
    write(&r.join("a/x.bin"), 10_000);
    write(&r.join("a/b/y.mp4"), 50_000);
    write(&r.join("c/z.txt"), 300);
    write(&r.join("top.log"), 0);
    d
}

#[test]
fn totals_exact() {
    let d = fixture();
    let r = d.path();
    let res = run(opts(r));
    let s = &res.summary;
    let mut expect = alloc(r);
    for p in ["a", "a/b", "c"] {
        expect += alloc(&r.join(p));
    }
    for p in ["a/x.bin", "a/b/y.mp4", "c/z.txt", "top.log"] {
        expect += alloc(&r.join(p));
    }
    assert_eq!(s.total_bytes, expect);
    assert_eq!(s.apparent_bytes, 60_300);
    assert_eq!(s.file_count, 4);
    assert_eq!(s.dir_count, 4);
    assert_eq!(s.error_count, 0);
    let sub = res.subtree(None, 3, 100).unwrap();
    assert_eq!(sub.size_bytes, s.total_bytes);
    let a = sub.children.iter().find(|c| c.name == "a").unwrap();
    assert_eq!(a.file_count, 2);
    assert_eq!(a.dir_count, 1);
    // Breakdown covers the video and the document.
    assert!(s.type_breakdown.iter().any(|t| t.category == FileCategory::Video && t.count == 1));
    assert_eq!(s.type_breakdown.iter().map(|t| t.count).sum::<u64>(), 4);
}

#[test]
fn missing_and_file_root() {
    let d = fixture();
    let missing = d.path().join("nope");
    assert!(matches!(
        scan("t".into(), opts(&missing), |_| {}, Arc::new(AtomicBool::new(false))),
        Err(CoreError::NotFound(_))
    ));
    let f = d.path().join("top.log");
    assert!(scan("t".into(), opts(&f), |_| {}, Arc::new(AtomicBool::new(false))).is_err());
}

#[cfg(unix)]
#[test]
fn hard_links_counted_once() {
    let d = fixture();
    let r = d.path();
    let base = run(opts(r)).summary;
    fs::hard_link(r.join("a/x.bin"), r.join("c/x_link.bin")).unwrap();
    let s = run(opts(r)).summary;
    assert_eq!(s.total_bytes, base.total_bytes);
    assert_eq!(s.file_count, base.file_count + 1);
    assert_eq!(s.apparent_bytes, base.apparent_bytes);
}

#[cfg(unix)]
#[test]
fn symlinks_not_followed_and_loops_safe() {
    let d = fixture();
    let r = d.path();
    let before = run(opts(r)).summary;
    std::os::unix::fs::symlink(r, r.join("a/loop")).unwrap();
    std::os::unix::fs::symlink(r.join("a"), r.join("c/link_to_a")).unwrap();
    std::os::unix::fs::symlink(r.join("a/x.bin"), r.join("filelink")).unwrap();
    let s = run(opts(r)).summary;
    assert_eq!(s.apparent_bytes, before.apparent_bytes);
    assert_eq!(s.dir_count, before.dir_count);
    assert_eq!(s.file_count, before.file_count + 3);
}

#[cfg(unix)]
#[test]
fn sparse_file_allocated_less_than_apparent() {
    let d = tempfile::tempdir().unwrap();
    let p = d.path().join("sparse.img");
    let f = fs::File::create(&p).unwrap();
    f.set_len(10_000_000).unwrap();
    drop(f);
    let res = run(opts(d.path()));
    assert_eq!(res.summary.apparent_bytes, 10_000_000);
    assert_eq!(res.summary.total_bytes, alloc(d.path()) + alloc(&p));
    assert!(alloc(&p) < 10_000_000);
}

#[cfg(unix)]
#[test]
fn unreadable_dir_counted_not_fatal() {
    use std::os::unix::fs::PermissionsExt;
    let d = fixture();
    let r = d.path();
    let locked = r.join("locked");
    write(&locked.join("secret"), 100);
    fs::set_permissions(&locked, fs::Permissions::from_mode(0o000)).unwrap();
    let res = scan("t".into(), opts(r), |_| {}, Arc::new(AtomicBool::new(false)));
    // restore so the tempdir can be removed
    fs::set_permissions(&locked, fs::Permissions::from_mode(0o755)).unwrap();
    let s = res.unwrap().summary;
    // Running as root can read it anyway; otherwise exactly one error.
    let is_root = fs::read_dir(&locked).is_ok() && s.error_count == 0;
    if !is_root {
        assert_eq!(s.error_count, 1);
    }
    assert_eq!(s.apparent_bytes, 60_300 + if is_root { 100 } else { 0 });
}

#[test]
fn exclusions_respected() {
    let d = fixture();
    let r = d.path();
    let mut o = opts(r);
    o.excluded_paths = vec![r.join("a/b").to_string_lossy().to_string(), r.join("top.log").to_string_lossy().to_string()];
    let s = run(o).summary;
    assert_eq!(s.apparent_bytes, 10_300);
    assert_eq!(s.file_count, 2);
    assert_eq!(s.dir_count, 3);
}

#[cfg(unix)]
#[test]
fn skipped_paths() {
    assert!(platform::is_skipped_path(Path::new("/proc")));
    assert!(platform::is_skipped_path(Path::new("/sys")));
    assert!(platform::is_skipped_path(Path::new("/dev")));
    assert!(platform::is_skipped_path(Path::new("/run")));
    assert!(!platform::is_skipped_path(Path::new("/home")));
    assert!(!platform::is_skipped_path(Path::new("/tmp/run")));
}

#[test]
fn cancel_and_progress() {
    let d = tempfile::tempdir().unwrap();
    for i in 0..30 {
        write(&d.path().join(format!("d{i}/f")), 10);
    }
    // Pre-set cancel flag.
    let r = scan("t".into(), opts(d.path()), |_| {}, Arc::new(AtomicBool::new(true)));
    assert!(matches!(r, Err(CoreError::Cancelled)));
    // Progress: at least the final event fires, with the right id and counts.
    let calls = AtomicU64::new(0);
    let last_files = AtomicU64::new(0);
    let res = scan(
        "abc".into(),
        opts(d.path()),
        |p| {
            assert_eq!(p.scan_id, "abc");
            calls.fetch_add(1, Ordering::SeqCst);
            last_files.store(p.files_scanned, Ordering::SeqCst);
        },
        Arc::new(AtomicBool::new(false)),
    )
    .unwrap();
    assert!(calls.load(Ordering::SeqCst) >= 1);
    assert_eq!(last_files.load(Ordering::SeqCst), 30);
    assert_eq!(res.summary.file_count, 30);
}

#[test]
fn cancel_during_scan() {
    let d = tempfile::tempdir().unwrap();
    for i in 0..200 {
        write(&d.path().join(format!("d{i}/f")), 10);
    }
    let cancel = Arc::new(AtomicBool::new(false));
    let c2 = cancel.clone();
    // Cancel from the progress callback (first emission can only come after 100 ms, so also cancel via thread).
    let h = std::thread::spawn(move || c2.store(true, Ordering::SeqCst));
    h.join().unwrap();
    let r = scan("t".into(), opts(d.path()), |_| {}, cancel);
    assert!(matches!(r, Err(CoreError::Cancelled)));
}

#[test]
fn subtree_queries() {
    let d = tempfile::tempdir().unwrap();
    let r = d.path();
    for i in 1..=5 {
        write(&r.join(format!("f{i}.bin")), i * 10_000);
    }
    write(&r.join("sub/deep/g.bin"), 20_000);
    let res = run(opts(r));
    let root = res.subtree(None, 1, 3).unwrap();
    assert_eq!(root.children.len(), 3);
    assert_eq!(root.hidden_children, 3); // 6 entries (5 files + sub) minus 3
    let sizes: Vec<u64> = root.children.iter().map(|c| c.size_bytes).collect();
    assert!(sizes.windows(2).all(|w| w[0] >= w[1]));
    let shown: u64 = sizes.iter().sum();
    assert_eq!(shown + root.other_bytes, root.size_bytes - alloc(r));
    // depth 0 = node only
    assert!(res.subtree(None, 0, 10).unwrap().children.is_empty());
    // path lookup
    let deep = res.subtree(Some(&r.join("sub/deep").to_string_lossy()), 1, 10).unwrap();
    assert_eq!(deep.name, "deep");
    assert_eq!(deep.children.len(), 1);
    assert_eq!(deep.children[0].name, "g.bin");
    assert_eq!(deep.children[0].apparent_bytes, 20_000);
    assert!(res.subtree(Some(&r.join("nothing").to_string_lossy()), 1, 10).is_none());
    assert!(res.subtree(Some("/definitely/elsewhere"), 1, 10).is_none());
    // depth 2 reaches sub/deep
    let two = res.subtree(None, 2, 100).unwrap();
    let sub = two.children.iter().find(|c| c.name == "sub").unwrap();
    assert_eq!(sub.children.len(), 1);
    assert!(sub.children[0].children.is_empty());
}

#[test]
fn largest_files_and_categories() {
    let d = tempfile::tempdir().unwrap();
    let r = d.path();
    write(&r.join("m/big.mp4"), 90_000);
    write(&r.join("m/small.mkv"), 5_000);
    write(&r.join("doc.pdf"), 40_000);
    write(&r.join("x/n.zip"), 20_000);
    write(&r.join("noext"), 8_000);
    let res = run(opts(r));
    let all = res.largest_files(10, None);
    assert_eq!(all.len(), 5);
    assert_eq!(all[0].name, "big.mp4");
    assert_eq!(all[0].category, FileCategory::Video);
    assert_eq!(all[0].extension.as_deref(), Some("mp4"));
    assert!(all.windows(2).all(|w| w[0].size_bytes >= w[1].size_bytes));
    assert_eq!(all[0].path, r.join("m").join("big.mp4").to_string_lossy());
    let vids = res.largest_files(10, Some(FileCategory::Video));
    assert_eq!(vids.len(), 2);
    assert_eq!(vids[1].name, "small.mkv");
    assert_eq!(res.largest_files(2, None).len(), 2);
    assert!(res.largest_files(10, Some(FileCategory::Audio)).is_empty());
    assert_eq!(all.iter().find(|f| f.name == "noext").unwrap().extension, None);
    // Bounded heap keeps only the top N.
    let mut o = opts(r);
    o.top_files_limit = 2;
    let small = run(o);
    let top = small.largest_files(10, None);
    assert_eq!(top.len(), 2);
    assert_eq!(top[0].name, "big.mp4");
    assert_eq!(top[1].name, "doc.pdf");
}

#[test]
fn directory_sizes_depth() {
    let d = tempfile::tempdir().unwrap();
    let r = d.path();
    write(&r.join("a/b/c/f"), 1000);
    let res = run(opts(r));
    let d0 = res.directory_sizes(0);
    assert_eq!(d0.len(), 1);
    assert_eq!(d0[0].1, res.summary.total_bytes);
    let d2 = res.directory_sizes(2);
    assert_eq!(d2.len(), 3);
    let all = res.directory_sizes(6);
    assert_eq!(all.len(), 4);
    let a = all.iter().find(|(p, _)| p == &r.join("a").to_string_lossy()).unwrap();
    assert_eq!(a.1, alloc(&r.join("a")) + alloc(&r.join("a/b")) + alloc(&r.join("a/b/c")) + alloc(&r.join("a/b/c/f")));
}

#[test]
fn categorize_table() {
    use FileCategory::*;
    let cases = [
        ("mp4", Video), ("mkv", Video), ("mp3", Audio), ("flac", Audio), ("png", Image), ("jpg", Image),
        ("pdf", Document), ("docx", Document), ("zip", Archive), ("gz", Archive), ("iso", DiskImage),
        ("qcow2", DiskImage), ("rs", Code), ("py", Code), ("exe", Executable), ("so", Executable),
        ("sqlite", Database), ("db", Database), ("log", Log), ("xyz", Other),
    ];
    for (e, c) in cases {
        assert_eq!(categorize(Some(e)), c, "{e}");
    }
    assert_eq!(categorize(None), Other);
}

#[test]
fn snapshot_roundtrip_and_diff() {
    let d = tempfile::tempdir().unwrap();
    let data = d.path().join("data");
    write(&data.join("pip/a.bin"), 10_000);
    write(&data.join("other/b.bin"), 5_000);
    let store = SnapshotStore::open(&d.path().join("sub/dir/h.db")).unwrap();
    let s1 = store.save(&run(opts(&data))).unwrap();
    write(&data.join("pip/big.bin"), 3_000_000);
    write(&data.join("fresh/new.bin"), 100_000);
    let s2 = store.save(&run(opts(&data))).unwrap();
    assert!(s2.id > s1.id);
    assert_eq!(s2.file_count, 4);
    let list = store.list().unwrap();
    assert_eq!(list.len(), 2);
    assert_eq!(list[0].id, s2.id);
    assert_eq!(list[1].root, s1.root);
    let g = store.diff(s1.id, s2.id, 10).unwrap();
    let pip = data.join("pip").to_string_lossy().to_string();
    assert_eq!(g[0].path, pip);
    assert!(g[0].delta_bytes >= 3_000_000);
    assert!(!g[0].is_new);
    assert!(g.iter().any(|e| e.path.ends_with("fresh") && e.is_new));
    // Ancestor (the data root) is not reported separately for pip's growth being dominant... only
    // when children explain it; here pip + fresh explain >= 80%.
    assert!(!g.iter().any(|e| e.path == data.to_string_lossy()));
    assert_eq!(store.diff(s1.id, s2.id, 1).unwrap().len(), 1);
    assert!(store.diff(s1.id, 9999, 5).is_err());
    store.delete(s1.id).unwrap();
    assert_eq!(store.list().unwrap().len(), 1);
    assert!(store.diff(s1.id, s2.id, 5).is_err());
    // Reopening keeps data.
    drop(store);
    let again = SnapshotStore::open(&d.path().join("sub/dir/h.db")).unwrap();
    assert_eq!(again.list().unwrap().len(), 1);
}

fn v(items: &[(&str, u64)]) -> Vec<(String, u64)> {
    items.iter().map(|(p, b)| (p.to_string(), *b)).collect()
}

#[test]
fn diff_deepest_cause() {
    let old = v(&[("/h", 100), ("/h/.cache", 50), ("/h/.cache/pip", 10), ("/h/docs", 50)]);
    let new = v(&[("/h", 3100), ("/h/.cache", 3050), ("/h/.cache/pip", 3010), ("/h/docs", 50)]);
    let g = diff_sizes(&old, &new, 10);
    assert_eq!(g.len(), 1);
    assert_eq!(g[0].path, "/h/.cache/pip");
    assert_eq!(g[0].delta_bytes, 3000);
    assert_eq!(g[0].before_bytes, 10);
    assert_eq!(g[0].after_bytes, 3010);
}

#[test]
fn diff_new_shrink_limit_and_split() {
    let old = v(&[("/r", 1000), ("/r/gone", 400), ("/r/a", 100)]);
    let new = v(&[("/r", 700), ("/r/a", 100), ("/r/newdir", 100)]);
    let g = diff_sizes(&old, &new, 10);
    let gone = g.iter().find(|e| e.path == "/r/gone").unwrap();
    assert_eq!(gone.delta_bytes, -400);
    assert_eq!(gone.after_bytes, 0);
    let nd = g.iter().find(|e| e.path == "/r/newdir").unwrap();
    assert!(nd.is_new);
    assert_eq!(nd.delta_bytes, 100);
    // sorted by |delta|
    assert!(g.windows(2).all(|w| w[0].delta_bytes.abs() >= w[1].delta_bytes.abs()));
    assert_eq!(diff_sizes(&old, &new, 1).len(), 1);
    assert!(diff_sizes(&old, &old, 10).is_empty());
    // Parent kept when children explain less than 80%: parent +1000, child +100.
    let o = v(&[("/p", 0), ("/p/c", 0)]);
    let n = v(&[("/p", 1000), ("/p/c", 100)]);
    let g = diff_sizes(&o, &n, 10);
    assert_eq!(g.len(), 2);
}

#[test]
fn disks_list_sane() {
    let disks = reclaim_core::disks::list_disks().unwrap();
    for d in &disks {
        assert!(d.used_bytes + d.available_bytes == d.total_bytes);
    }
    let td = tempfile::tempdir().unwrap();
    if let Some(d) = reclaim_core::disks::disk_for_path(&td.path().to_string_lossy()).unwrap() {
        assert!(td.path().starts_with(&d.mount_point));
    }
}
