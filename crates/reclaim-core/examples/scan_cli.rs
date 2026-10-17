//! Read-only CLI: scans a directory and prints a JSON summary (totals, peak RSS, top 10 dirs).
//! Usage: cargo run --release -p reclaim-core --example scan_cli -- <root> [--cross]

use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use reclaim_core::scan::scan;
use reclaim_core::ScanOptions;

/// Peak resident set size in bytes (Linux only; None elsewhere).
fn peak_rss() -> Option<u64> {
    let s = std::fs::read_to_string("/proc/self/status").ok()?;
    let line = s.lines().find(|l| l.starts_with("VmHWM:"))?;
    let kb: u64 = line.split_whitespace().nth(1)?.parse().ok()?;
    Some(kb * 1024)
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let cross = args.iter().any(|a| a == "--cross");
    let Some(root) = args.iter().find(|a| !a.starts_with("--")) else {
        eprintln!("usage: scan_cli <root> [--cross]");
        std::process::exit(2);
    };
    let opts = ScanOptions {
        root: root.clone(),
        cross_filesystems: cross,
        excluded_paths: vec![],
        top_files_limit: 500,
    };
    let res = match scan("cli".into(), opts, |_| {}, Arc::new(AtomicBool::new(false))) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("scan failed: {e}");
            std::process::exit(1);
        }
    };
    let s = &res.summary;
    let top: Vec<serde_json::Value> = res
        .subtree(None, 1, 10)
        .map(|t| t.children)
        .unwrap_or_default()
        .into_iter()
        .filter(|c| c.is_dir)
        .map(|c| serde_json::json!({ "path": c.path, "sizeBytes": c.size_bytes }))
        .collect();
    let out = serde_json::json!({
        "root": s.root,
        "totalBytes": s.total_bytes,
        "apparentBytes": s.apparent_bytes,
        "fileCount": s.file_count,
        "dirCount": s.dir_count,
        "errorCount": s.error_count,
        "durationMs": s.duration_ms,
        "peakRssBytes": peak_rss(),
        "topDirs": top,
    });
    println!("{}", serde_json::to_string_pretty(&out).unwrap());
}
