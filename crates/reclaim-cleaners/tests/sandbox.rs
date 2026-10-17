//! Sandbox tests: every test runs inside a temp HOME with PATH shims; nothing outside the tempdir is touched.
//! Tests mutate process env, so they are all #[serial]. No real pkexec/snap/docker/... is ever run.

#![cfg(target_os = "linux")]

use std::collections::BTreeSet;
use std::fs;
use std::os::unix::fs::{symlink, PermissionsExt};
use std::path::{Path, PathBuf};
use std::time::Duration;

use filetime::{set_file_mtime, FileTime};
use reclaim_cleaners::safety::{self, is_protected};
use reclaim_cleaners::{all_cleaners, find_cleaner, CleanError, CleanOptions, CleanupPreview};
use serial_test::serial;
use tempfile::TempDir;
use walkdir::WalkDir;

// ------------------------------------------------------------------------------------------
// Harness
// ------------------------------------------------------------------------------------------

const SHIM: &str = r##"#!/bin/sh
name=${0##*/}
d=${0%/*}
printf '%s' "$name" >> "$SHIM_LOG"
for a in "$@"; do printf ' %s' "$a" >> "$SHIM_LOG"; done
printf '\n' >> "$SHIM_LOG"
f="$d/$name.${1}_${2}.out"
[ -f "$f" ] || f="$d/$name.out"
if [ -f "$f" ]; then while IFS= read -r l; do printf '%s\n' "$l"; done < "$f"; fi
rc=0
[ -f "$d/$name.rc" ] && read rc < "$d/$name.rc"
exit "$rc"
"##;

/// Logs, honours pkexec.rc (e.g. 126 = cancelled), and only really runs `rm` on sandbox paths.
const PKEXEC: &str = r##"#!/bin/sh
d=${0%/*}
printf 'pkexec' >> "$SHIM_LOG"
for a in "$@"; do printf ' %s' "$a" >> "$SHIM_LOG"; done
printf '\n' >> "$SHIM_LOG"
rc=0
[ -f "$d/pkexec.rc" ] && read rc < "$d/pkexec.rc"
[ "$rc" = 0 ] || exit "$rc"
if [ "$1" = rm ]; then
  shift
  seen=0
  for a in "$@"; do
    if [ "$seen" = 1 ]; then
      case "$a" in "$SANDBOX"/*) ;; *) exit 99 ;; esac
    fi
    [ "$a" = -- ] && seen=1
  done
  exec /bin/rm "$@"
fi
exit 0
"##;

struct Sb {
    _td: TempDir,
    root: PathBuf,
    home: PathBuf,
    bin: PathBuf,
    log: PathBuf,
}

impl Sb {
    fn new() -> Sb {
        let base = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/test-sandbox");
        fs::create_dir_all(&base).unwrap();
        let td = tempfile::Builder::new().prefix("sb").tempdir_in(&base).unwrap();
        let root = td.path().canonicalize().unwrap();
        let (home, bin) = (root.join("home"), root.join("bin"));
        fs::create_dir_all(&home).unwrap();
        fs::create_dir_all(&bin).unwrap();
        let log = root.join("calls.log");
        fs::write(&log, "").unwrap();
        std::env::set_var("HOME", &home);
        std::env::set_var("XDG_CACHE_HOME", home.join(".cache"));
        std::env::set_var("XDG_CONFIG_HOME", home.join(".config"));
        std::env::set_var("XDG_DATA_HOME", home.join(".local/share"));
        std::env::set_var("XDG_STATE_HOME", home.join(".local/state"));
        for v in ["XDG_DOWNLOAD_DIR", "CARGO_HOME", "GRADLE_USER_HOME", "TMPDIR"] {
            std::env::remove_var(v);
        }
        std::env::set_var("PATH", &bin);
        std::env::set_var("SHIM_LOG", &log);
        std::env::set_var("SANDBOX", &root);
        let sb = Sb { _td: td, root, home, bin, log };
        sb.script("pkexec", PKEXEC);
        for n in ["journalctl", "snap", "apt-get", "flatpak", "docker", "pnpm", "go", "npm"] {
            sb.script(n, SHIM);
        }
        sb
    }
    fn script(&self, name: &str, body: &str) {
        let p = self.bin.join(name);
        fs::write(&p, body).unwrap();
        fs::set_permissions(&p, fs::Permissions::from_mode(0o755)).unwrap();
    }
    fn canned(&self, name: &str, key: &str, out: &str) {
        fs::write(self.bin.join(format!("{name}.{key}.out")), out).unwrap();
    }
    fn rc(&self, name: &str, code: i32) {
        fs::write(self.bin.join(format!("{name}.rc")), code.to_string()).unwrap();
    }
    fn calls(&self) -> Vec<String> {
        fs::read_to_string(&self.log).unwrap().lines().map(str::to_string).collect()
    }
    fn calls_of(&self, prefix: &str) -> Vec<String> {
        self.calls().into_iter().filter(|l| l.starts_with(prefix)).collect()
    }
    fn clear_log(&self) {
        fs::write(&self.log, "").unwrap();
    }
    fn file(&self, rel: &str, bytes: usize) -> PathBuf {
        let p = self.home.join(rel);
        fs::create_dir_all(p.parent().unwrap()).unwrap();
        fs::write(&p, vec![b'x'; bytes]).unwrap();
        p
    }
}

fn age_tree(path: &Path, days: u64) {
    let t = FileTime::from_unix_time(
        (std::time::SystemTime::now() - Duration::from_secs(days * 86_400))
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64,
        0,
    );
    let entries: Vec<_> = WalkDir::new(path).contents_first(true).into_iter().flatten().collect();
    for e in entries {
        set_file_mtime(e.path(), t).unwrap();
    }
}

fn fingerprint(root: &Path) -> BTreeSet<String> {
    WalkDir::new(root)
        .follow_links(false)
        .into_iter()
        .flatten()
        .map(|e| {
            let md = e.metadata().unwrap();
            format!("{} {} {:?} {}", e.path().display(), md.len(), md.modified().ok(), md.file_type().is_symlink())
        })
        .collect()
}

fn opts(permanent: bool) -> CleanOptions {
    CleanOptions { permanent }
}

/// Populates the sandbox home with junk and with things that must survive. Returns the keep list.
fn populate(sb: &Sb) -> Vec<PathBuf> {
    let mut keep = Vec::new();
    // Trash (files + metadata)
    sb.file(".local/share/Trash/files/t1.txt", 5000);
    sb.file(".local/share/Trash/info/t1.txt.trashinfo", 100);
    // thumbnails, generic caches
    sb.file(".cache/thumbnails/large/a.png", 5000);
    sb.file(".cache/foo/bar.bin", 5000);
    sb.file(".cache/pip/http/x", 5000);
    sb.file(".cache/yarn/v6/pkg", 5000);
    keep.push(sb.file(".cache/mozilla/firefox/prof/cookies.sqlite", 100));
    sb.file(".cache/mozilla/firefox/prof/cache2/entries/e1", 5000);
    sb.file(".cache/google-chrome/Default/Cache/Cache_Data/f", 5000);
    keep.push(sb.file(".config/google-chrome/Default/Cookies", 100));
    keep.push(sb.file(".config/google-chrome/Default/History", 100));
    sb.file(".config/google-chrome/Default/GPUCache/g", 5000);
    // dev caches
    sb.file(".cargo/registry/cache/idx/c.crate", 5000);
    keep.push(sb.file(".cargo/registry/src/idx/code.rs", 100));
    sb.file(".cargo/git/db/repo/pack", 5000);
    sb.file(".gradle/caches/modules/m", 5000);
    keep.push(sb.file(".gradle/gradle.properties", 50));
    sb.file(".npm/_cacache/content-v2/a", 5000);
    keep.push(sb.file(".npmrc", 50));
    // editors
    sb.file(".config/Code/CachedData/h/x", 5000);
    sb.file(".config/Code/logs/l.log", 5000);
    keep.push(sb.file(".config/Code/User/settings.json", 50));
    // downloads
    let old = sb.file("Downloads/old.zip", 5000);
    set_file_mtime(&old, FileTime::from_unix_time(1_000_000_000, 0)).unwrap();
    sb.file("Downloads/olddir/f.bin", 5000);
    age_tree(&sb.home.join("Downloads/olddir"), 200);
    keep.push(sb.file("Downloads/new.zip", 5000));
    // node projects
    sb.file("work/oldproj/package.json", 50);
    sb.file("work/oldproj/src/a.js", 50);
    sb.file("work/oldproj/node_modules/dep/index.js", 5000);
    age_tree(&sb.home.join("work/oldproj"), 60);
    keep.push(sb.file("work/newproj/package.json", 50));
    keep.push(sb.file("work/newproj/node_modules/dep/index.js", 5000));
    keep.push(sb.file("work/nopkg/node_modules/dep/index.js", 5000));
    age_tree(&sb.home.join("work/nopkg"), 90);
    // rust projects
    sb.file("work/oldrs/Cargo.toml", 50);
    sb.file("work/oldrs/src/main.rs", 50);
    sb.file("work/oldrs/target/CACHEDIR.TAG", 50);
    sb.file("work/oldrs/target/debug/bin", 5000);
    age_tree(&sb.home.join("work/oldrs"), 60);
    keep.push(sb.file("work/newrs/Cargo.toml", 50));
    keep.push(sb.file("work/newrs/target/CACHEDIR.TAG", 50));
    sb.file("work/foreign/Cargo.toml", 50);
    keep.push(sb.file("work/foreign/target/out.bin", 5000)); // no CACHEDIR.TAG: not a cargo target
    age_tree(&sb.home.join("work/foreign"), 90);
    keep
}

/// Ids of cleaners that only touch the sandbox home (no system paths, no external tools).
const FILE_IDS: &[&str] = &[
    "trash",
    "thumbnails",
    "user-cache",
    "pip-cache",
    "yarn-cache",
    "cargo-cache",
    "gradle-cache",
    "npm-cache",
    "browser-caches",
    "vscode-caches",
    "old-downloads",
    "node-modules",
    "rust-targets",
];

fn preview_paths(p: &CleanupPreview) -> Vec<PathBuf> {
    p.items.iter().map(|i| PathBuf::from(&i.path)).collect()
}

// ------------------------------------------------------------------------------------------
// is_protected
// ------------------------------------------------------------------------------------------

#[test]
#[serial]
fn protected_table() {
    let sb = Sb::new();
    let h = sb.home.clone();
    let must_block: Vec<PathBuf> = [
        "/", "/bin", "/boot", "/etc", "/etc/passwd", "/lib", "/lib64", "/usr", "/usr/bin/ls", "/sbin", "/proc", "/proc/1",
        "/sys", "/dev", "/dev/null", "/run", "/var", "/var/lib", "/var/lib/dpkg", "/var/lib/dpkg/status", "/var/log",
        "/var/log/journal", "/var/log/journal/x", "/root", "/home", "/tmp", "/mnt", "/media", "/opt", "/snap",
        "/boot/efi/EFI", "relative/path", "../x", "",
    ]
    .iter()
    .map(PathBuf::from)
    .chain([
        h.clone(),
        h.parent().unwrap().to_path_buf(),
        h.join("Documents"),
        h.join(".cache"),
        h.join(".config"),
        h.join(".local"),
        h.join(".local/share"),
        h.join(".ssh"),
        h.join(".ssh/id_rsa"),
        h.join(".gnupg/pubring.kbx"),
        h.join("Downloads"),
        h.join("x/../.ssh"),
    ])
    .collect();
    for p in &must_block {
        assert!(is_protected(p), "must be protected: {}", p.display());
    }
    let must_allow: Vec<PathBuf> = [
        "/var/log/syslog.1",
        "/var/log/apt/history.log.1.gz",
        "/var/cache/apt/archives/x.deb",
        "/var/lib/systemd/coredump/core.1",
        "/var/tmp/abc",
        "/tmp/abc",
        "/home/someone/Downloads/x.iso",
        "/opt/app/data",
    ]
    .iter()
    .map(PathBuf::from)
    .chain([
        h.join(".cache/pip"),
        h.join(".cache/pip/http/x"),
        h.join("Downloads/old.zip"),
        h.join("work/proj/node_modules"),
        h.join(".local/share/Trash/files/a"),
        h.join(".cargo/registry/cache"),
    ])
    .collect();
    for p in &must_allow {
        assert!(!is_protected(p), "must be allowed: {}", p.display());
    }
}

#[test]
#[serial]
fn protected_resolves_symlinked_parent() {
    let sb = Sb::new();
    // home/escape -> /etc : a path through the link is /etc, which is protected.
    symlink("/etc", sb.home.join("escape")).unwrap();
    assert!(is_protected(&sb.home.join("escape/passwd")));
    assert!(is_protected(&sb.home.join("escape/ssh/ssh_config")));
}

// ------------------------------------------------------------------------------------------
// remove_path / remove_path_in
// ------------------------------------------------------------------------------------------

#[test]
#[serial]
fn remove_permanent_and_trash() {
    let sb = Sb::new();
    let a = sb.file("proj/a.txt", 10_000);
    let freed = safety::remove_path(&a, true).unwrap();
    assert!(freed >= 10_000);
    assert!(!a.exists());

    let b = sb.file("proj/dir/b.txt", 10_000);
    let dir = b.parent().unwrap().to_path_buf();
    let freed = safety::remove_path(&dir, false).unwrap();
    assert!(freed >= 10_000);
    assert!(!dir.exists());
    let trashed = sb.home.join(".local/share/Trash/files/dir/b.txt");
    assert!(trashed.exists(), "directory should be in the sandbox Trash");
}

#[test]
#[serial]
fn remove_refuses_protected_and_missing() {
    let sb = Sb::new();
    let docs = sb.home.join("Documents");
    fs::create_dir_all(&docs).unwrap();
    fs::write(docs.join("thesis.txt"), "important").unwrap();
    for p in [sb.home.clone(), docs.clone(), sb.home.join(".ssh")] {
        match safety::remove_path(&p, true) {
            Err(CleanError::Protected(_)) => {}
            other => panic!("expected Protected for {}, got {other:?}", p.display()),
        }
    }
    assert!(docs.join("thesis.txt").exists());
    assert!(matches!(safety::remove_path(&sb.home.join("nope/x"), true), Err(CleanError::Io { .. })));
    assert!(matches!(safety::remove_path(Path::new("relative"), true), Err(CleanError::Protected(_))));
}

#[test]
#[serial]
fn symlinks_are_removed_as_links_never_followed() {
    let sb = Sb::new();
    let target = sb.file("data/keep/important.txt", 100);
    let link = sb.home.join("proj/link");
    fs::create_dir_all(link.parent().unwrap()).unwrap();
    symlink(target.parent().unwrap(), &link).unwrap();
    safety::remove_path(&link, true).unwrap();
    assert!(fs::symlink_metadata(&link).is_err());
    assert!(target.exists(), "link target must be untouched");

    // A symlink to a directory nested inside a tree being removed.
    let tree = sb.home.join("proj/tree");
    fs::create_dir_all(&tree).unwrap();
    symlink(target.parent().unwrap(), tree.join("inner")).unwrap();
    fs::write(tree.join("f"), "x").unwrap();
    safety::remove_path(&tree, true).unwrap();
    assert!(!tree.exists());
    assert!(target.exists());

    // Same through the Trash path.
    let link2 = sb.home.join("proj/link2");
    symlink(target.parent().unwrap(), &link2).unwrap();
    safety::remove_path(&link2, false).unwrap();
    assert!(target.exists());
}

#[test]
#[serial]
fn remove_path_in_enforces_roots() {
    let sb = Sb::new();
    let root = sb.home.join("proj/root");
    let inside = sb.file("proj/root/a/b.txt", 100);
    let outside = sb.file("proj/other/c.txt", 100);
    let roots = vec![root.clone()];
    assert!(matches!(safety::remove_path_in(&outside, &roots, true), Err(CleanError::Protected(_))));
    assert!(matches!(safety::remove_path_in(&root, &roots, true), Err(CleanError::Protected(_))));
    assert!(outside.exists() && root.exists());
    // A link inside the root pointing outside: operating "through" it must be refused.
    symlink(sb.home.join("proj/other"), root.join("esc")).unwrap();
    assert!(safety::remove_path_in(&root.join("esc/c.txt"), &roots, true).is_err());
    assert!(outside.exists());
    safety::remove_path_in(&inside, &roots, true).unwrap();
    assert!(!inside.exists());
}

// ------------------------------------------------------------------------------------------
// File cleaners
// ------------------------------------------------------------------------------------------

#[test]
#[serial]
fn preview_never_modifies_anything() {
    let sb = Sb::new();
    populate(&sb);
    sb.canned("journalctl", "--disk-usage_", "Archived and active journals take up 1.2G in the file system.\n");
    sb.canned("snap", "list_--all", "Name Version Rev Tracking Publisher Notes\nfirefox 1 100 latest/stable moz disabled\n");
    sb.canned("flatpak", "uninstall_--unused", "1. org.x  1  < 10.0 MB\n");
    sb.canned("docker", "system_df", "Containers\t2MB (100%)\n");
    sb.canned("docker", "images_-f", "10MB\n");
    let before = fingerprint(&sb.root);
    let cleaners = all_cleaners();
    assert!(cleaners.len() >= 13);
    for c in cleaners {
        let id = c.info().id;
        let _ = c.preview();
        // preview may only append to the shim log, which is outside home; compare home only.
        assert_eq!(
            fingerprint(&sb.home),
            before.iter().filter(|l| l.starts_with(&sb.home.display().to_string())).cloned().collect(),
            "preview of {id} changed the sandbox home"
        );
    }
    for l in sb.calls() {
        assert!(!l.starts_with("pkexec"), "preview must never elevate: {l}");
    }
}

#[test]
#[serial]
fn clean_removes_exactly_the_preview_set() {
    let sb = Sb::new();
    let keep = populate(&sb);
    let mut all_removed: Vec<PathBuf> = Vec::new();
    for id in FILE_IDS {
        let c = find_cleaner(id).unwrap_or_else(|| panic!("cleaner {id} should be available in the sandbox"));
        let p = c.preview().unwrap();
        assert!(p.total_bytes > 0 && p.item_count > 0, "{id}: preview found nothing");
        assert_eq!(p.items.len() as u64, p.item_count.min(200), "{id}");
        let paths = preview_paths(&p);
        for path in &paths {
            assert!(path.starts_with(&sb.home), "{id}: preview path outside sandbox: {}", path.display());
        }
        let r = c.clean(&opts(true)).unwrap();
        assert!(r.success, "{id}: {:?}", r.errors);
        assert!(r.errors.is_empty(), "{id}: {:?}", r.errors);
        assert_eq!(r.items_removed, p.item_count, "{id}");
        assert!(r.freed_bytes > 0, "{id}");
        for path in &paths {
            assert!(fs::symlink_metadata(path).is_err(), "{id}: {} should be gone", path.display());
        }
        all_removed.extend(paths);
        // Second run finds nothing left.
        assert_eq!(c.preview().unwrap().item_count, 0, "{id}: nothing should remain");
    }
    for k in &keep {
        assert!(k.exists(), "must survive: {}", k.display());
    }
    // Spot-check the specific expectations from the spec.
    let removed: Vec<String> = all_removed.iter().map(|p| p.strip_prefix(&sb.home).unwrap().display().to_string()).collect();
    for want in [
        "Downloads/old.zip",
        "Downloads/olddir",
        "work/oldproj/node_modules",
        "work/oldrs/target",
        ".config/google-chrome/Default/GPUCache",
        ".cache/mozilla/firefox/prof/cache2",
        ".config/Code/CachedData",
    ] {
        assert!(removed.iter().any(|r| r == want), "expected {want} in {removed:?}");
    }
    assert!(sb.home.join("work/oldproj/package.json").exists() && sb.home.join("work/oldrs/src/main.rs").exists());
    assert!(sb.calls_of("pkexec").is_empty());
}

#[test]
#[serial]
fn clean_default_moves_to_sandbox_trash() {
    let sb = Sb::new();
    populate(&sb);
    let c = find_cleaner("pip-cache").unwrap();
    let p = c.preview().unwrap();
    let r = c.clean(&opts(false)).unwrap();
    assert!(r.success && r.items_removed == p.item_count);
    assert!(!sb.home.join(".cache/pip/http").exists());
    assert!(sb.home.join(".local/share/Trash/files/http").exists());
}

#[test]
#[serial]
fn idle_detection_for_projects() {
    let sb = Sb::new();
    populate(&sb);
    let nm: Vec<String> = preview_paths(&find_cleaner("node-modules").unwrap().preview().unwrap())
        .iter()
        .map(|p| p.strip_prefix(&sb.home).unwrap().display().to_string())
        .collect();
    assert_eq!(nm, vec!["work/oldproj/node_modules"], "recent and package.json-less projects must be skipped");

    let rt: Vec<String> = preview_paths(&find_cleaner("rust-targets").unwrap().preview().unwrap())
        .iter()
        .map(|p| p.strip_prefix(&sb.home).unwrap().display().to_string())
        .collect();
    assert_eq!(rt, vec!["work/oldrs/target"]);

    // Touching a source file makes the old project active again.
    fs::write(sb.home.join("work/oldproj/src/a.js"), "changed").unwrap();
    assert_eq!(find_cleaner("node-modules").unwrap().preview().unwrap().item_count, 0);
    // A fresh edit in the old Rust project too.
    fs::write(sb.home.join("work/oldrs/src/main.rs"), "fn main(){}").unwrap();
    assert_eq!(find_cleaner("rust-targets").unwrap().preview().unwrap().item_count, 0);
}

#[test]
#[serial]
fn node_modules_not_descended_and_symlink_ignored() {
    let sb = Sb::new();
    sb.file("work/p/package.json", 10);
    sb.file("work/p/node_modules/a/node_modules/inner.js", 100);
    sb.file("work/p/node_modules/a/package.json", 10);
    age_tree(&sb.home.join("work/p"), 100);
    // A symlinked node_modules is never a candidate.
    sb.file("work/q/package.json", 10);
    let real = sb.file("elsewhere/x.js", 100);
    symlink(real.parent().unwrap(), sb.home.join("work/q/node_modules")).unwrap();
    age_tree(&sb.home.join("work/q"), 100);
    let p = find_cleaner("node-modules").unwrap().preview().unwrap();
    assert_eq!(p.item_count, 1);
    assert!(p.items[0].path.ends_with("work/p/node_modules"));
}

#[test]
#[serial]
fn old_downloads_fall_back_without_user_dirs() {
    let sb = Sb::new();
    populate(&sb);
    let c = find_cleaner("old-downloads").expect("Downloads should be found without user-dirs.dirs");
    assert_eq!(c.preview().unwrap().item_count, 2);
}

#[test]
#[serial]
fn elevated_removal_is_one_prompt_and_checked() {
    let sb = Sb::new();
    let a = sb.file("sys/log/a.gz", 1000);
    let b = sb.file("sys/log/b.1", 1000);
    let c = sb.file("sys/log/sub/c.old", 1000);
    let keep = sb.file("sys/other/keep", 1000);
    let roots = vec![sb.home.join("sys/log")];
    let r = safety::remove_paths_elevated(&[a.clone(), b.clone(), c.clone(), keep.clone()], &roots).unwrap();
    assert_eq!(r.removed, 3, "{:?}", r.errors);
    assert!(r.errors.iter().any(|e| e.contains("refused")), "outside-root path must be refused");
    assert!(!a.exists() && !b.exists() && !c.exists());
    assert!(keep.exists());
    let calls = sb.calls_of("pkexec");
    assert_eq!(calls.len(), 1, "exactly one elevated invocation: {calls:?}");
    assert!(calls[0].starts_with("pkexec rm -rf --one-file-system -- "), "{}", calls[0]);
    // Protected paths never reach pkexec.
    sb.clear_log();
    let r = safety::remove_paths_elevated(&[PathBuf::from("/etc"), sb.home.clone()], &[PathBuf::from("/")]).unwrap();
    assert_eq!(r.removed, 0);
    assert!(sb.calls_of("pkexec").is_empty());
}

#[test]
#[serial]
fn elevation_cancel_maps_to_denied() {
    let sb = Sb::new();
    let a = sb.file("sys/log/a.gz", 1000);
    sb.rc("pkexec", 126);
    let roots = vec![sb.home.join("sys/log")];
    assert!(matches!(safety::remove_paths_elevated(&[a.clone()], &roots), Err(CleanError::ElevationDenied)));
    assert!(a.exists());
    assert!(matches!(safety::run_command("journalctl", &["--x"], true), Err(CleanError::ElevationDenied)));
    sb.rc("pkexec", 127);
    assert!(matches!(safety::run_command("journalctl", &["--x"], true), Err(CleanError::ElevationDenied)));
}

// ------------------------------------------------------------------------------------------
// Command cleaners (all through shims)
// ------------------------------------------------------------------------------------------

#[test]
#[serial]
fn run_command_timeout_and_errors() {
    let sb = Sb::new();
    sb.script("slow", "#!/bin/sh\nread x < /dev/zero 2>/dev/null\nwhile :; do :; done\n");
    let t = std::time::Instant::now();
    let e = safety::run_command_timeout("slow", &[], false, Duration::from_millis(600)).unwrap_err();
    assert!(t.elapsed() < Duration::from_secs(5));
    assert!(e.to_string().contains("timed out"), "{e}");
    sb.rc("go", 3);
    assert!(matches!(safety::run_command("go", &[], false), Err(CleanError::Command { .. })));
    assert!(matches!(safety::run_command("definitely-not-installed", &[], false), Err(CleanError::Command { .. })));
    // Arguments are passed verbatim, never through a shell.
    sb.rc("go", 0);
    sb.clear_log();
    safety::run_command("go", &["a b", "$(touch /nonexistent-marker)", ";rm"], false).unwrap();
    assert_eq!(sb.calls_of("go"), vec!["go a b $(touch /nonexistent-marker) ;rm"]);
    assert!(!Path::new("/nonexistent-marker").exists());
}

#[test]
#[serial]
fn journald_command_flow() {
    let sb = Sb::new();
    sb.canned("journalctl", "--disk-usage_", "Archived and active journals take up 1.2G in the file system.\n");
    let c = find_cleaner("journald-logs").unwrap();
    let p = c.preview().unwrap();
    assert_eq!(p.total_bytes, (1.2f64 * 1024.0 * 1024.0 * 1024.0) as u64 - 200 * 1024 * 1024);
    assert_eq!(p.command.as_deref(), Some("pkexec journalctl --vacuum-size=200M"));
    assert_eq!(sb.calls(), vec!["journalctl --disk-usage"]);
    sb.clear_log();
    c.clean(&opts(false)).unwrap();
    let calls = sb.calls();
    assert_eq!(calls.iter().filter(|l| l.starts_with("pkexec")).count(), 1);
    assert!(calls.contains(&"pkexec journalctl --vacuum-size=200M".to_string()), "{calls:?}");
    // Cancelled prompt.
    sb.rc("pkexec", 126);
    assert!(matches!(c.clean(&opts(false)), Err(CleanError::ElevationDenied)));
}

#[test]
#[serial]
fn snap_batches_into_one_elevation() {
    let sb = Sb::new();
    sb.canned(
        "snap",
        "list_--all",
        "Name Version Rev Tracking Publisher Notes\n\
         firefox 1 100 latest/stable mozilla disabled\n\
         firefox 2 101 latest/stable mozilla -\n\
         core22 3 200 latest/stable canonical base,disabled\n\
         evil;rm 4 5 x y disabled\n\
         bad 1 x1 x y disabled\n",
    );
    let c = find_cleaner("snap-old-revisions").unwrap();
    let p = c.preview().unwrap();
    assert_eq!(p.item_count, 2, "invalid names/revisions and active revisions are excluded");
    sb.clear_log();
    c.clean(&opts(false)).unwrap();
    let pk = sb.calls_of("pkexec");
    assert_eq!(pk.len(), 1, "one password prompt for all revisions: {pk:?}");
    assert!(pk[0].starts_with("pkexec sh -c "), "{}", pk[0]);
    assert!(pk[0].contains("snap remove \"$1\" --revision=\"$2\""));
    assert!(pk[0].ends_with(" sh firefox 100 core22 200"), "{}", pk[0]);
    sb.rc("pkexec", 126);
    assert!(matches!(c.clean(&opts(false)), Err(CleanError::ElevationDenied)));
}

#[test]
#[serial]
fn apt_and_flatpak_commands() {
    let sb = Sb::new();
    if let Some(c) = find_cleaner("apt-cache") {
        sb.clear_log();
        c.clean(&opts(false)).unwrap();
        assert_eq!(sb.calls(), vec!["pkexec apt-get clean"]);
    } else {
        eprintln!("apt-cache unavailable on this host; skipped");
    }
    sb.canned("flatpak", "uninstall_--unused", "ID  Branch  Op  Size\n 1. org.a.Platform 23.08 r < 142.3 MB\n 2. org.b.Locale 23.08 r < 50.0 MB\n");
    let c = find_cleaner("flatpak-unused").unwrap();
    let p = c.preview().unwrap();
    assert_eq!((p.total_bytes, p.item_count), (192_300_000, 2));
    assert_eq!(sb.calls_of("flatpak"), vec!["flatpak uninstall --unused --assumeno"]);
    sb.clear_log();
    c.clean(&opts(false)).unwrap();
    assert!(sb.calls().contains(&"flatpak uninstall --unused -y".to_string()));
    assert!(sb.calls_of("pkexec").is_empty(), "flatpak runs unprivileged");
}

#[test]
#[serial]
fn docker_pnpm_go_commands() {
    let sb = Sb::new();
    sb.canned(
        "docker",
        "system_df",
        "Images\t1GB (50%)\nContainers\t2MB (100%)\nLocal Volumes\t5GB (100%)\nBuild Cache\t10MB (100%)\n",
    );
    sb.canned("docker", "images_-f", "100MB\n");
    sb.canned("docker", "system_prune", "Deleted Containers:\nabc\nTotal reclaimed space: 112MB\n");
    let c = find_cleaner("docker").unwrap();
    let p = c.preview().unwrap();
    assert_eq!(p.total_bytes, 112_000_000, "volumes and in-use images are not counted");
    sb.clear_log();
    let r = c.clean(&opts(false)).unwrap();
    assert_eq!(r.freed_bytes, 112_000_000);
    assert!(sb.calls().contains(&"docker system prune -f".to_string()));
    assert!(!sb.calls().iter().any(|l| l.contains("--volumes") || l.contains(" -a")));

    let store = sb.root.join("pnpmstore");
    fs::create_dir_all(&store).unwrap();
    sb.canned("pnpm", "store_path", &format!("{}\n", store.display()));
    let c = find_cleaner("pnpm-store").unwrap();
    assert_eq!(c.preview().unwrap().command.as_deref(), Some("pnpm store prune"));
    sb.clear_log();
    c.clean(&opts(false)).unwrap();
    assert!(sb.calls().contains(&"pnpm store prune".to_string()));

    let gocache = sb.root.join("gocache");
    fs::create_dir_all(&gocache).unwrap();
    fs::write(gocache.join("f"), vec![1u8; 20_000]).unwrap();
    sb.canned("go", "env_GOCACHE", &format!("{}\n", gocache.display()));
    let c = find_cleaner("go-build-cache").unwrap();
    assert!(c.preview().unwrap().total_bytes >= 20_000);
    sb.clear_log();
    c.clean(&opts(false)).unwrap();
    assert!(sb.calls().contains(&"go clean -cache".to_string()));
    assert!(gocache.join("f").exists(), "go's own command does the removal; the shim does nothing");
}

#[test]
#[serial]
fn registry_only_lists_available() {
    let sb = Sb::new();
    // Empty home, no shims for docker/pnpm/go/flatpak/snap/journalctl/apt removed:
    for n in ["docker", "pnpm", "go", "flatpak", "snap", "journalctl", "apt-get"] {
        fs::remove_file(sb.bin.join(n)).unwrap();
    }
    let ids: Vec<String> = all_cleaners().iter().map(|c| c.info().id).collect();
    for gone in ["docker", "pnpm-store", "go-build-cache", "flatpak-unused", "snap-old-revisions", "journald-logs", "apt-cache"] {
        assert!(!ids.contains(&gone.to_string()), "{gone} should be unavailable: {ids:?}");
    }
    for absent in ["trash", "thumbnails", "gradle-cache", "old-downloads"] {
        assert!(!ids.contains(&absent.to_string()), "{absent} has no folder in an empty home");
    }
    let unique: BTreeSet<_> = ids.iter().collect();
    assert_eq!(unique.len(), ids.len());
}

// ------------------------------------------------------------------------------------------
// Insights
// ------------------------------------------------------------------------------------------

#[test]
#[serial]
fn deleted_open_file_is_reported() {
    let sb = Sb::new();
    let p = sb.file("held/big.bin", 1_000_000);
    let f = fs::File::open(&p).unwrap();
    fs::remove_file(&p).unwrap();
    let found = reclaim_cleaners::insights::deleted_open_files().unwrap();
    let mine: Vec<_> = found.iter().filter(|d| d.pid == std::process::id() && d.path.ends_with("held/big.bin")).collect();
    assert_eq!(mine.len(), 1, "{found:?}");
    assert!(mine[0].size_bytes >= 1_000_000);
    assert!(!mine[0].process_name.is_empty());
    drop(f);
    let after = reclaim_cleaners::insights::deleted_open_files().unwrap();
    assert!(!after.iter().any(|d| d.pid == std::process::id() && d.path.ends_with("held/big.bin")));
}
