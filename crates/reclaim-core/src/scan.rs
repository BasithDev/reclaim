//! Parallel directory scanner.
//!
//! Walks a tree measuring allocated size (platform::allocated_size), counts hard
//! links once (platform::file_id), never follows symlinks/junctions, stays on one
//! filesystem unless asked, reports progress through a callback and can be cancelled.
//! The finished tree is kept in memory (`ScanResult`) for drill-down queries.
//!
//! Storage: one arena `Vec<Node>`. The children of a directory are pushed as one contiguous
//! block (`first_child .. first_child + child_count`), always after the parent, so sizes
//! can be aggregated bottom-up with a single reverse pass over the arena.

use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering::Relaxed};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use crate::{
    platform, CoreError, FileCategory, FileEntry, Result, ScanOptions, ScanProgress, ScanSummary, TreeNode, TypeStat,
};

const CATEGORIES: [FileCategory; 11] = [
    FileCategory::Video,
    FileCategory::Audio,
    FileCategory::Image,
    FileCategory::Document,
    FileCategory::Archive,
    FileCategory::DiskImage,
    FileCategory::Code,
    FileCategory::Executable,
    FileCategory::Database,
    FileCategory::Log,
    FileCategory::Other,
];
const NCAT: usize = CATEGORIES.len();

fn cat_index(c: FileCategory) -> usize {
    CATEGORIES.iter().position(|x| *x == c).unwrap_or(NCAT - 1)
}

/// One entry in the arena (file or directory).
struct Node {
    name: Box<str>,
    size: u64,
    apparent: u64,
    first_child: u32,
    child_count: u32,
    parent: u32,
    /// Modification time in Unix seconds; 0 = unknown.
    mtime: u32,
    /// Files in this subtree (1 for a file node).
    files: u32,
    /// Directories below this node (not counting itself).
    dirs: u32,
    is_dir: bool,
    category: u8,
}

/// A finished scan: summary plus the in-memory tree. Cheap to share behind an Arc.
pub struct ScanResult {
    pub summary: ScanSummary,
    nodes: Vec<Node>,
    /// Per category: arena indices of the largest files, biggest first.
    top: Vec<Vec<u32>>,
    top_limit: usize,
}

fn mtime_secs(meta: &fs::Metadata) -> u32 {
    meta.modified()
        .ok()
        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .map(|d| d.as_secs().min(u32::MAX as u64) as u32)
        .unwrap_or(0)
}

fn extension_of(name: &str) -> Option<String> {
    let (stem, ext) = name.rsplit_once('.')?;
    if stem.is_empty() || ext.is_empty() || ext.len() > 16 {
        return None;
    }
    Some(ext.to_lowercase())
}

impl ScanResult {
    fn path_of(&self, idx: u32) -> PathBuf {
        let mut chain = Vec::new();
        let mut cur = idx;
        while cur != 0 {
            chain.push(cur);
            cur = self.nodes[cur as usize].parent;
        }
        let mut p = PathBuf::from(&*self.nodes[0].name);
        for i in chain.iter().rev() {
            p.push(&*self.nodes[*i as usize].name);
        }
        p
    }

    fn display_name(&self, idx: u32) -> String {
        if idx == 0 {
            Path::new(&*self.nodes[0].name)
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| self.nodes[0].name.to_string())
        } else {
            self.nodes[idx as usize].name.to_string()
        }
    }

    fn children(&self, idx: u32) -> std::ops::Range<u32> {
        let n = &self.nodes[idx as usize];
        n.first_child..n.first_child + n.child_count
    }

    fn find(&self, path: &str) -> Option<u32> {
        let rel = Path::new(path).strip_prefix(&*self.nodes[0].name).ok()?;
        let mut cur = 0u32;
        for comp in rel.components() {
            if matches!(comp, std::path::Component::CurDir) {
                continue;
            }
            let name = comp.as_os_str().to_string_lossy();
            if !self.nodes[cur as usize].is_dir {
                return None;
            }
            cur = self.children(cur).find(|c| platform::names_equal(&self.nodes[*c as usize].name, &name))?;
        }
        Some(cur)
    }

    fn build_tree(&self, idx: u32, path: PathBuf, depth: u32, max_children: usize) -> TreeNode {
        let n = &self.nodes[idx as usize];
        let mut node = TreeNode {
            path: path.to_string_lossy().to_string(),
            name: self.display_name(idx),
            is_dir: n.is_dir,
            size_bytes: n.size,
            apparent_bytes: n.apparent,
            file_count: n.files as u64,
            dir_count: n.dirs as u64,
            modified_ms: (n.mtime != 0).then(|| n.mtime as i64 * 1000),
            children: Vec::new(),
            hidden_children: 0,
            other_bytes: 0,
        };
        if !n.is_dir || depth == 0 {
            return node;
        }
        let mut kids: Vec<u32> = self.children(idx).collect();
        let by_size = |a: &u32, b: &u32| self.nodes[*b as usize].size.cmp(&self.nodes[*a as usize].size);
        if kids.len() > max_children {
            if max_children > 0 {
                kids.select_nth_unstable_by(max_children - 1, by_size);
            }
            node.hidden_children = (kids.len() - max_children) as u64;
            node.other_bytes = kids[max_children..].iter().map(|k| self.nodes[*k as usize].size).sum();
            kids.truncate(max_children);
        }
        kids.sort_unstable_by(by_size);
        node.children = kids
            .into_iter()
            .map(|k| self.build_tree(k, path.join(&*self.nodes[k as usize].name), depth - 1, max_children))
            .collect();
        node
    }

    /// The node at `path` (None = scan root) with children down to `depth` levels
    /// (0 = node only), each level capped at `max_children` largest entries.
    pub fn subtree(&self, path: Option<&str>, depth: u32, max_children: usize) -> Option<TreeNode> {
        let idx = match path {
            None => 0,
            Some(p) => self.find(p)?,
        };
        Some(self.build_tree(idx, self.path_of(idx), depth, max_children))
    }

    /// Largest files overall, optionally filtered by category, biggest first.
    pub fn largest_files(&self, limit: usize, category: Option<FileCategory>) -> Vec<FileEntry> {
        let mut idxs: Vec<u32> = match category {
            Some(c) => self.top[cat_index(c)].clone(),
            None => self.top.iter().flatten().copied().collect(),
        };
        idxs.sort_unstable_by(|a, b| self.nodes[*b as usize].size.cmp(&self.nodes[*a as usize].size));
        idxs.truncate(limit.min(self.top_limit));
        idxs.into_iter()
            .map(|i| {
                let n = &self.nodes[i as usize];
                FileEntry {
                    path: self.path_of(i).to_string_lossy().to_string(),
                    name: n.name.to_string(),
                    size_bytes: n.size,
                    modified_ms: (n.mtime != 0).then(|| n.mtime as i64 * 1000),
                    extension: extension_of(&n.name),
                    category: CATEGORIES[n.category as usize],
                }
            })
            .collect()
    }

    /// Every directory down to `max_depth` below root with its size; used for snapshots.
    pub fn directory_sizes(&self, max_depth: u32) -> Vec<(String, u64)> {
        let mut out = Vec::new();
        let mut stack: Vec<(u32, PathBuf, u32)> = vec![(0, self.path_of(0), 0)];
        while let Some((idx, path, depth)) = stack.pop() {
            out.push((path.to_string_lossy().to_string(), self.nodes[idx as usize].size));
            if depth >= max_depth {
                continue;
            }
            for c in self.children(idx) {
                let n = &self.nodes[c as usize];
                if n.is_dir {
                    stack.push((c, path.join(&*n.name), depth + 1));
                }
            }
        }
        out
    }
}

type LargeHeap = BinaryHeap<Reverse<(u64, u32)>>;

/// State shared by all walker tasks.
struct Ctx<'a> {
    scan_id: String,
    root_dev: u64,
    cross: bool,
    excluded: HashSet<PathBuf>,
    cancel: &'a AtomicBool,
    nodes: Mutex<Vec<Node>>,
    links: Mutex<HashSet<(u64, u64)>>,
    files: AtomicU64,
    dirs: AtomicU64,
    bytes: AtomicU64,
    apparent: AtomicU64,
    errors: AtomicU64,
    cat_bytes: [AtomicU64; NCAT],
    cat_count: [AtomicU64; NCAT],
    thresholds: [AtomicU64; NCAT],
    heaps: Mutex<Vec<LargeHeap>>,
    top_limit: usize,
    last_emit: Mutex<Instant>,
    start: Instant,
    on_progress: &'a (dyn Fn(ScanProgress) + Send + Sync),
}

impl Ctx<'_> {
    fn progress(&self, path: &Path) -> ScanProgress {
        ScanProgress {
            scan_id: self.scan_id.clone(),
            files_scanned: self.files.load(Relaxed),
            dirs_scanned: self.dirs.load(Relaxed),
            bytes_counted: self.bytes.load(Relaxed),
            errors: self.errors.load(Relaxed),
            current_path: path.to_string_lossy().to_string(),
            elapsed_ms: self.start.elapsed().as_millis() as u64,
        }
    }

    /// Emits a progress event at most every ~100 ms (skipped if another thread is emitting).
    fn tick(&self, path: &Path) {
        if let Ok(mut last) = self.last_emit.try_lock() {
            if last.elapsed() >= Duration::from_millis(100) {
                *last = Instant::now();
                (self.on_progress)(self.progress(path));
            }
        }
    }

    fn offer_large(&self, cat: usize, size: u64, idx: u32) {
        if size == 0 || self.top_limit == 0 || size < self.thresholds[cat].load(Relaxed) {
            return;
        }
        let mut heaps = self.heaps.lock().unwrap();
        let heap = &mut heaps[cat];
        heap.push(Reverse((size, idx)));
        if heap.len() > self.top_limit {
            heap.pop();
        }
        if heap.len() == self.top_limit {
            if let Some(Reverse((min, _))) = heap.peek() {
                self.thresholds[cat].store(*min, Relaxed);
            }
        }
    }
}

fn new_node(name: Box<str>, is_dir: bool) -> Node {
    Node {
        name,
        size: 0,
        apparent: 0,
        first_child: 0,
        child_count: 0,
        parent: 0,
        mtime: 0,
        files: u32::from(!is_dir),
        dirs: 0,
        is_dir,
        category: (NCAT - 1) as u8,
    }
}

/// Reads one directory, registers its children in the arena and spawns tasks for subdirectories.
fn walk_dir<'a, 's>(ctx: &'s Ctx<'a>, scope: &rayon::Scope<'s>, idx: u32, path: PathBuf)
where
    'a: 's,
{
    if ctx.cancel.load(Relaxed) {
        return;
    }
    ctx.dirs.fetch_add(1, Relaxed);
    ctx.tick(&path);
    let rd = match fs::read_dir(&path) {
        Ok(r) => r,
        Err(_) => {
            ctx.errors.fetch_add(1, Relaxed);
            return;
        }
    };

    let mut children: Vec<Node> = Vec::new();
    let mut subdirs: Vec<(usize, PathBuf)> = Vec::new();
    // (local index, category, size) of regular files, offered to the largest-files heaps.
    let mut file_sizes: Vec<(usize, usize, u64)> = Vec::new();
    let mut cat_bytes = [0u64; NCAT];
    let mut cat_count = [0u64; NCAT];
    let (mut n_files, mut bytes, mut apparent) = (0u64, 0u64, 0u64);

    for (count, entry) in rd.enumerate() {
        if count % 512 == 511 {
            if ctx.cancel.load(Relaxed) {
                return;
            }
            ctx.tick(&path);
        }
        let entry = match entry {
            Ok(e) => e,
            Err(_) => {
                ctx.errors.fetch_add(1, Relaxed);
                continue;
            }
        };
        let epath = entry.path();
        if !ctx.excluded.is_empty() && ctx.excluded.contains(&epath) {
            continue;
        }
        let meta = match entry.metadata() {
            Ok(m) => m,
            Err(_) => {
                ctx.errors.fetch_add(1, Relaxed);
                continue;
            }
        };
        let name: Box<str> = entry.file_name().to_string_lossy().into();
        let link = platform::is_link_like(&meta);

        if meta.is_dir() && !link {
            if platform::is_skipped_path(&epath) {
                continue;
            }
            let mut node = new_node(name, true);
            node.size = platform::allocated_size(&epath, &meta);
            node.apparent = 0;
            node.mtime = mtime_secs(&meta);
            bytes += node.size;
            if ctx.cross || platform::device_id(&epath, &meta) == ctx.root_dev {
                subdirs.push((children.len(), epath));
            }
            children.push(node);
        } else {
            let mut node = new_node(name, false);
            node.mtime = mtime_secs(&meta);
            if !link {
                let mut alloc = platform::allocated_size(&epath, &meta);
                let mut app = meta.len();
                if let Some(id) = platform::file_id(&epath, &meta) {
                    if !ctx.links.lock().unwrap().insert(id) {
                        alloc = 0;
                        app = 0;
                    }
                }
                let cat = cat_index(categorize(extension_of(&node.name).as_deref()));
                node.size = alloc;
                node.apparent = app;
                node.category = cat as u8;
                cat_bytes[cat] += alloc;
                cat_count[cat] += 1;
                file_sizes.push((children.len(), cat, alloc));
                bytes += alloc;
                apparent += app;
            }
            n_files += 1;
            children.push(node);
        }
    }

    ctx.files.fetch_add(n_files, Relaxed);
    ctx.bytes.fetch_add(bytes, Relaxed);
    ctx.apparent.fetch_add(apparent, Relaxed);
    for i in 0..NCAT {
        if cat_count[i] > 0 {
            ctx.cat_bytes[i].fetch_add(cat_bytes[i], Relaxed);
            ctx.cat_count[i].fetch_add(cat_count[i], Relaxed);
        }
    }
    if children.is_empty() {
        return;
    }

    for c in children.iter_mut() {
        c.parent = idx;
    }
    let count = children.len() as u32;
    let base = {
        let mut nodes = ctx.nodes.lock().unwrap();
        let base = nodes.len() as u32;
        let dir = &mut nodes[idx as usize];
        dir.first_child = base;
        dir.child_count = count;
        nodes.extend(children);
        base
    };
    for (local, cat, size) in file_sizes {
        ctx.offer_large(cat, size, base + local as u32);
    }
    for (local, p) in subdirs {
        let child = base + local as u32;
        scope.spawn(move |s| walk_dir(ctx, s, child, p));
    }
}

fn now_ms() -> i64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_millis() as i64).unwrap_or(0)
}

/// Runs a scan to completion (blocking; call from a background thread).
/// `on_progress` is throttled to about every 100 ms. Returns Err(Cancelled) if `cancel` is set.
pub fn scan(
    scan_id: String,
    options: ScanOptions,
    on_progress: impl Fn(ScanProgress) + Send + Sync,
    cancel: Arc<AtomicBool>,
) -> Result<ScanResult> {
    let root = PathBuf::from(&options.root);
    let root_meta = fs::metadata(&root).map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            CoreError::NotFound(options.root.clone())
        } else {
            CoreError::Io { path: options.root.clone(), source: e }
        }
    })?;
    if !root_meta.is_dir() {
        return Err(CoreError::Other(format!("not a directory: {}", options.root)));
    }

    let started_at_ms = now_ms();
    let start = Instant::now();
    let mut root_node = new_node(options.root.clone().into_boxed_str(), true);
    root_node.size = platform::allocated_size(&root, &root_meta);
    root_node.mtime = mtime_secs(&root_meta);

    let ctx = Ctx {
        scan_id: scan_id.clone(),
        root_dev: platform::device_id(&root, &root_meta),
        cross: options.cross_filesystems,
        excluded: options.excluded_paths.iter().map(PathBuf::from).collect(),
        cancel: &cancel,
        nodes: Mutex::new(vec![root_node]),
        links: Mutex::new(HashSet::new()),
        files: AtomicU64::new(0),
        dirs: AtomicU64::new(0),
        bytes: AtomicU64::new(0),
        apparent: AtomicU64::new(0),
        errors: AtomicU64::new(0),
        cat_bytes: std::array::from_fn(|_| AtomicU64::new(0)),
        cat_count: std::array::from_fn(|_| AtomicU64::new(0)),
        thresholds: std::array::from_fn(|_| AtomicU64::new(0)),
        heaps: Mutex::new((0..NCAT).map(|_| BinaryHeap::new()).collect()),
        top_limit: options.top_files_limit,
        last_emit: Mutex::new(Instant::now()),
        start,
        on_progress: &on_progress,
    };

    // The walk is I/O bound, so use more threads than cores.
    let threads = std::thread::available_parallelism().map(|n| n.get() * 2).unwrap_or(8).clamp(4, 32);
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(threads)
        .thread_name(|i| format!("reclaim-scan-{i}"))
        .build()
        .map_err(|e| CoreError::Other(format!("cannot start scan threads: {e}")))?;
    pool.scope(|s| walk_dir(&ctx, s, 0, root.clone()));

    if cancel.load(Relaxed) {
        return Err(CoreError::Cancelled);
    }
    (ctx.on_progress)(ctx.progress(&root));

    let mut nodes = std::mem::take(&mut *ctx.nodes.lock().unwrap());
    // Bottom-up aggregation: children always have a higher index than their parent.
    for i in (1..nodes.len()).rev() {
        let (p, size, app, files, dirs, is_dir) = {
            let n = &nodes[i];
            (n.parent as usize, n.size, n.apparent, n.files, n.dirs, n.is_dir)
        };
        let parent = &mut nodes[p];
        parent.size += size;
        parent.apparent += app;
        parent.files += files;
        parent.dirs += dirs + u32::from(is_dir);
    }

    let top: Vec<Vec<u32>> = ctx
        .heaps
        .lock()
        .unwrap()
        .iter()
        .map(|h| {
            let mut v: Vec<(u64, u32)> = h.iter().map(|Reverse(x)| *x).collect();
            v.sort_unstable_by(|a, b| b.cmp(a));
            v.into_iter().map(|(_, i)| i).collect()
        })
        .collect();

    let mut type_breakdown: Vec<TypeStat> = (0..NCAT)
        .filter(|i| ctx.cat_count[*i].load(Relaxed) > 0)
        .map(|i| TypeStat {
            category: CATEGORIES[i],
            bytes: ctx.cat_bytes[i].load(Relaxed),
            count: ctx.cat_count[i].load(Relaxed),
        })
        .collect();
    type_breakdown.sort_by_key(|t| Reverse(t.bytes));

    let summary = ScanSummary {
        scan_id,
        root: options.root.clone(),
        total_bytes: nodes[0].size,
        apparent_bytes: nodes[0].apparent,
        file_count: nodes[0].files as u64,
        dir_count: nodes[0].dirs as u64 + 1,
        error_count: ctx.errors.load(Relaxed),
        started_at_ms,
        finished_at_ms: now_ms(),
        duration_ms: start.elapsed().as_millis() as u64,
        type_breakdown,
        snapshot_id: None,
    };
    Ok(ScanResult { summary, nodes, top, top_limit: options.top_files_limit })
}

/// Maps a file extension (lowercase, without dot) to a category.
pub fn categorize(extension: Option<&str>) -> FileCategory {
    let Some(ext) = extension else { return FileCategory::Other };
    match ext {
        "mp4" | "mkv" | "avi" | "mov" | "wmv" | "flv" | "webm" | "m4v" | "mpg" | "mpeg" | "ts" | "3gp" | "vob"
        | "m2ts" | "ogv" => FileCategory::Video,
        "mp3" | "flac" | "wav" | "aac" | "ogg" | "m4a" | "wma" | "opus" | "aiff" | "mid" | "midi" => {
            FileCategory::Audio
        }
        "jpg" | "jpeg" | "png" | "gif" | "bmp" | "webp" | "svg" | "tif" | "tiff" | "heic" | "heif" | "raw" | "cr2"
        | "nef" | "arw" | "dng" | "ico" | "psd" | "avif" => FileCategory::Image,
        "pdf" | "doc" | "docx" | "xls" | "xlsx" | "ppt" | "pptx" | "odt" | "ods" | "odp" | "txt" | "rtf" | "md"
        | "epub" | "mobi" | "csv" | "tex" => FileCategory::Document,
        "zip" | "tar" | "gz" | "tgz" | "bz2" | "xz" | "7z" | "rar" | "zst" | "lz4" | "lzma" | "cab" | "jar"
        | "war" | "deb" | "rpm" | "apk" | "whl" | "crate" => FileCategory::Archive,
        "iso" | "img" | "vmdk" | "vdi" | "qcow2" | "vhd" | "vhdx" | "dmg" | "wim" | "ova" | "squashfs" => {
            FileCategory::DiskImage
        }
        "rs" | "py" | "js" | "jsx" | "tsx" | "java" | "c" | "h" | "cpp" | "hpp" | "cc" | "cs" | "go" | "rb"
        | "php" | "swift" | "kt" | "sh" | "bat" | "ps1" | "html" | "css" | "scss" | "json" | "yaml" | "yml"
        | "toml" | "xml" | "sql" | "lua" | "dart" | "vue" | "svelte" | "map" => FileCategory::Code,
        "exe" | "dll" | "so" | "bin" | "msi" | "appimage" | "o" | "a" | "lib" | "sys" | "dylib" | "pyc" | "class"
        | "wasm" | "rlib" => FileCategory::Executable,
        "db" | "sqlite" | "sqlite3" | "mdb" | "accdb" | "mdf" | "ldf" | "ibd" | "rdb" | "parquet" | "bak" => {
            FileCategory::Database
        }
        "log" | "out" | "trace" | "dmp" | "etl" => FileCategory::Log,
        _ => FileCategory::Other,
    }
}
