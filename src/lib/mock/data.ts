// Reclaim mock backend data: a believable Linux dev machine (disks, a ~52 GB home tree, cleaners, snapshots).
// Used by mock/index.ts (browser dev with VITE_MOCK=1) and by tests. Pure data + small helpers, no side effects.
import type { CleanerInfo, DeletedOpenFile, DiskInfo, FileCategory, FileEntry, Settings, SnapshotMeta } from "../types";

export const GB = 1024 ** 3;
export const MB = 1024 ** 2;
export const HOME = "/home/basith";
const DAY = 86_400_000;
export const NOW = Date.now();

// ---------- Disks ----------
export const DISKS: DiskInfo[] = [
  { id: "nvme0n1p5", name: "Ubuntu (nvme0n1p5)", mountPoint: "/", fileSystem: "ext4", totalBytes: 134 * GB, usedBytes: Math.round(134 * GB * 0.85), availableBytes: Math.round(134 * GB * 0.15), isRemovable: false, isSystem: true, kind: "ssd" },
  { id: "nvme0n1p3", name: "Windows (nvme0n1p3)", mountPoint: "/mnt/windows", fileSystem: "ntfs", totalBytes: 476 * GB, usedBytes: Math.round(476 * GB * 0.61), availableBytes: Math.round(476 * GB * 0.39), isRemovable: false, isSystem: false, kind: "ssd" },
  { id: "sda1", name: "BACKUP_USB", mountPoint: "/media/basith/BACKUP_USB", fileSystem: "exfat", totalBytes: 58 * GB, usedBytes: Math.round(58 * GB * 0.4), availableBytes: Math.round(58 * GB * 0.6), isRemovable: true, isSystem: false, kind: "unknown" },
];

// ---------- Scan tree ----------
export interface MNode { name: string; path: string; isDir: boolean; leafBytes: number; leafFiles: number; children: MNode[]; mtime: number }

const node = (parent: string, name: string, kind: "dir" | "file", bytes: number, files: number, children: MNode[], age: number): MNode => ({
  name, path: `${parent}/${name}`, isDir: kind === "dir", leafBytes: bytes, leafFiles: files, children, mtime: NOW - age * DAY,
});
/** Directory with children; paths are fixed up recursively by `place`. */
type Spec = { n: string; gb?: number; mb?: number; files?: number; age?: number; kids?: Spec[]; file?: boolean };
const d = (n: string, kids: Spec[], age = 2): Spec => ({ n, kids, age });
const leaf = (n: string, gb: number, files: number, age = 5): Spec => ({ n, gb, files, age });
const f = (n: string, mb: number, age = 10): Spec => ({ n, mb, file: true, age });

function place(parent: string, s: Spec): MNode {
  const bytes = (s.gb ?? 0) * GB + (s.mb ?? 0) * MB;
  if (s.kids) {
    const path = `${parent}/${s.n}`;
    return { name: s.n, path, isDir: true, leafBytes: 0, leafFiles: 0, mtime: NOW - (s.age ?? 2) * DAY, children: s.kids.map((k) => place(path, k)) };
  }
  return node(parent, s.n, s.file ? "file" : "dir", bytes, s.file ? 1 : s.files ?? 1, [], s.age ?? 5);
}

const SPECS: Spec[] = [
  d(".cache", [leaf("pip", 3.1, 8420, 3), leaf("google-chrome", 2.2, 5310, 0), leaf("mozilla", 1.6, 3290, 1), leaf("JetBrains", 1.1, 2210, 12), leaf("yarn", 0.8, 11250, 40), leaf("thumbnails", 0.4, 9100, 6), leaf("ms-playwright", 0.9, 1800, 25)]),
  d(".cargo", [leaf("registry", 2.9, 41200, 4), leaf("git", 0.9, 8900, 9), leaf("bin", 0.3, 24, 30)]),
  d(".rustup", [leaf("toolchains", 1.8, 14100, 20)]),
  d(".npm", [leaf("_cacache", 1.2, 31200, 2)]),
  d(".local", [d("share", [leaf("Trash", 1.3, 214, 1), leaf("flatpak", 0.7, 5400, 15)])]),
  leaf(".config", 0.6, 3300, 1),
  d("snap", [leaf("firefox", 2.1, 4300, 9), leaf("code", 1.8, 6100, 4), leaf("spotify", 1.4, 900, 33)]),
  d("Downloads", [
    f("ubuntu-24.04.2-desktop-amd64.iso", 5900, 95), f("blender-4.2.1-linux-x64.tar.xz", 1280, 120), f("Windows11_23H2.iso", 3400, 210),
    leaf("old-installers", 1.1, 38, 140), f("dataset-v2.zip", 720, 61), leaf("misc", 0.6, 112, 20),
  ]),
  d("Videos", [f("conference-talk-recording.mp4", 2300, 18), f("trip-2024-raw.mkv", 3100, 75), leaf("screencasts", 2.4, 41, 7)]),
  d("Projects", [
    d("reclaim", [leaf("target", 3.2, 21500, 0), leaf("node_modules", 0.9, 24300, 0), leaf("src-tauri", 0.2, 340, 0), leaf(".git", 0.1, 1200, 0)], 0),
    d("webapp", [leaf("node_modules", 1.4, 41800, 62), leaf(".next", 0.8, 3100, 62), leaf("src", 0.05, 480, 62)], 62),
    d("ml-experiments", [leaf("venv", 1.9, 19800, 48), f("model-checkpoint.pt", 980, 48), leaf("data", 0.7, 90, 48)], 48),
    leaf("old-rust-target", 1.5, 9800, 95),
  ], 0),
  d("Pictures", [leaf("Photos", 2.6, 1840, 30), leaf("Screenshots", 0.6, 960, 3)]),
  d("Documents", [leaf("Work", 1.2, 640, 4), leaf("Taxes", 0.2, 80, 200), leaf("Books", 0.5, 120, 90)]),
  d("Music", [leaf("Library", 1.2, 310, 80)]),
];

export function buildTree(): MNode {
  return { name: "basith", path: HOME, isDir: true, leafBytes: 0, leafFiles: 0, mtime: NOW, children: SPECS.map((s) => place(HOME, s)) };
}

export const sizeOf = (n: MNode): number => n.leafBytes + n.children.reduce((s, c) => s + sizeOf(c), 0);
export const filesOf = (n: MNode): number => n.leafFiles + n.children.reduce((s, c) => s + filesOf(c), 0);
export const dirsOf = (n: MNode): number => (n.isDir ? 1 : 0) + n.children.reduce((s, c) => s + dirsOf(c), 0);

export function find(root: MNode, path: string | null): MNode | null {
  if (!path || path === root.path) return root;
  if (!path.startsWith(root.path + "/")) return null;
  for (const c of root.children) { const r = find(c, path); if (r) return r; }
  return null;
}
export function remove(root: MNode, path: string): number {
  const parent = find(root, path.slice(0, path.lastIndexOf("/")));
  const i = parent?.children.findIndex((c) => c.path === path) ?? -1;
  if (!parent || i < 0) return 0;
  const [gone] = parent.children.splice(i, 1);
  return sizeOf(gone);
}

// ---------- Largest files ----------
const EXT_CAT: Record<string, FileCategory> = {
  mp4: "video", mkv: "video", iso: "diskImage", img: "diskImage", zip: "archive", xz: "archive", gz: "archive", tar: "archive",
  mp3: "audio", flac: "audio", jpg: "image", png: "image", raw: "image", pdf: "document", epub: "document", pt: "database",
  db: "database", sqlite: "database", log: "log", so: "executable", AppImage: "executable", rs: "code", rlib: "code", json: "code",
};
const extOf = (name: string) => (name.includes(".") ? name.split(".").pop()! : null);
export const categoryOf = (name: string): FileCategory => EXT_CAT[extOf(name) ?? ""] ?? "other";

const EXTRA: [string, number, number][] = [
  ["Projects/reclaim/target/debug/deps/libreclaim_core-3f9a1.rlib", 410, 0], ["Projects/reclaim/target/debug/reclaim", 188, 0],
  [".cache/mozilla/firefox/cache2/entries/9A3FC1", 96, 1], [".cache/pip/http/wheels/torch-2.4.0-cp312.whl", 780, 3],
  ["Projects/webapp/.next/cache/webpack/client.pack", 310, 62], ["Videos/screencasts/demo-final.mp4", 640, 7],
  [".local/share/Trash/files/backup-2023.tar.gz", 920, 1], ["snap/spotify/common/.cache/offline.bnk", 540, 33],
  ["Pictures/Photos/2024/IMG_8812.raw", 62, 30], ["Pictures/Photos/2024/IMG_8813.raw", 61, 30], ["Documents/Books/rust-programming.pdf", 58, 90],
  [".config/Code/logs/exthost.log", 210, 1], [".npm/_cacache/content-v2/sha512/ab/cd/blob", 140, 2], ["Music/Library/live-set.flac", 480, 80],
  ["Downloads/misc/backup.sqlite", 330, 20], ["Projects/ml-experiments/data/train.db", 410, 48], [".cargo/registry/cache/index.crate", 150, 4],
  ["Downloads/old-installers/Postman-linux-x64.tar.gz", 190, 140], ["Downloads/old-installers/discord.deb", 95, 140], ["Downloads/misc/tool.AppImage", 118, 20],
];

export function buildLargestFiles(root: MNode): FileEntry[] {
  const out: FileEntry[] = [];
  const walk = (n: MNode) => { if (!n.isDir) out.push({ path: n.path, name: n.name, sizeBytes: n.leafBytes, modifiedMs: n.mtime, extension: extOf(n.name), category: categoryOf(n.name) }); n.children.forEach(walk); };
  walk(root);
  for (const [rel, mb, age] of EXTRA) { const name = rel.split("/").pop()!; out.push({ path: `${HOME}/${rel}`, name, sizeBytes: mb * MB, modifiedMs: NOW - age * DAY, extension: extOf(name), category: categoryOf(name) }); }
  // Deterministic filler so the table is long enough to scroll.
  let seed = 7;
  const rnd = () => (seed = (seed * 16807) % 2147483647) / 2147483647;
  const names = ["clip", "export", "render", "archive", "cache", "dump", "session", "backup"];
  const exts = ["mp4", "zip", "log", "png", "pdf", "mp3", "db", "so", "json"];
  const dirs = ["Videos", "Downloads", "Documents", "Pictures", ".cache", "Projects/webapp"];
  for (let i = 0; i < 150; i++) {
    const e = exts[Math.floor(rnd() * exts.length)];
    const nm = `${names[Math.floor(rnd() * names.length)]}-${i}.${e}`;
    out.push({ path: `${HOME}/${dirs[Math.floor(rnd() * dirs.length)]}/${nm}`, name: nm, sizeBytes: Math.round((300 * MB) / (1 + i * 0.35) + rnd() * 20 * MB), modifiedMs: NOW - Math.floor(rnd() * 400) * DAY, extension: e, category: categoryOf(nm) });
  }
  return out.sort((a, b) => b.sizeBytes - a.sizeBytes);
}

export const TYPE_SHARES: [FileCategory, number, number][] = [
  ["video", 0.17, 480], ["archive", 0.15, 2200], ["code", 0.21, 188000], ["diskImage", 0.14, 4], ["image", 0.07, 11800],
  ["database", 0.06, 340], ["executable", 0.05, 2900], ["document", 0.04, 4100], ["audio", 0.03, 410], ["log", 0.02, 1900], ["other", 0.06, 22000],
];

export const SCAN_PATHS = [".cache/pip/http", "Projects/reclaim/target/debug/deps", "Projects/webapp/node_modules/.pnpm", "snap/firefox/common", ".cargo/registry/src", "Downloads", "Videos/screencasts", ".local/share/Trash/files", ".rustup/toolchains/stable/lib", "Pictures/Photos/2024", ".npm/_cacache/content-v2"];

// ---------- Cleaners ----------
export interface MockCleaner extends CleanerInfo { bytes: number; items: [string, number, number | null][]; command?: string; fail?: string }
export const CLEANERS: MockCleaner[] = [
  { id: "journald-logs", name: "Systemd journal", description: "Vacuum the journal down to 200 MB", group: "system", risk: "safe", requiresAdmin: true, method: "command", bytes: 1.4 * GB, items: [], command: "journalctl --vacuum-size=200M" },
  { id: "apt-cache", name: "APT package cache", description: "Downloaded .deb files in /var/cache/apt/archives", group: "system", risk: "safe", requiresAdmin: true, method: "command", bytes: 860 * MB, items: [], command: "apt-get clean" },
  { id: "snap-old-revisions", name: "Old Snap revisions", description: "Disabled revisions of installed snaps", group: "system", risk: "safe", requiresAdmin: true, method: "command", bytes: 2.6 * GB, items: [], command: "snap remove firefox --revision=4336 && snap remove code --revision=179" },
  { id: "rotated-logs", name: "Rotated logs", description: "Compressed and numbered logs in /var/log", group: "system", risk: "safe", requiresAdmin: true, method: "files", bytes: 310 * MB, items: [["/var/log/syslog.2.gz", 84 * MB, 20], ["/var/log/kern.log.1", 61 * MB, 8], ["/var/log/auth.log.3.gz", 12 * MB, 25]] },
  { id: "trash", name: "Trash", description: "Everything in your Trash folder (emptied permanently)", group: "user", risk: "careful", requiresAdmin: false, method: "files", bytes: 1.3 * GB, items: [[`${HOME}/.local/share/Trash/files/backup-2023.tar.gz`, 920 * MB, 1], [`${HOME}/.local/share/Trash/files/old-notes`, 4 * MB, 2]] },
  { id: "thumbnails", name: "Thumbnail cache", description: "Generated image previews, rebuilt on demand", group: "user", risk: "safe", requiresAdmin: false, method: "files", bytes: 410 * MB, items: [[`${HOME}/.cache/thumbnails/large`, 300 * MB, 6], [`${HOME}/.cache/thumbnails/normal`, 110 * MB, 6]] },
  { id: "old-downloads", name: "Old downloads", description: "Files in Downloads untouched for 90+ days. Review before cleaning.", group: "user", risk: "careful", requiresAdmin: false, method: "files", bytes: 10.4 * GB, items: [[`${HOME}/Downloads/ubuntu-24.04.2-desktop-amd64.iso`, 5.9 * GB, 95], [`${HOME}/Downloads/Windows11_23H2.iso`, 3.4 * GB, 210], [`${HOME}/Downloads/blender-4.2.1-linux-x64.tar.xz`, 1.25 * GB, 120]] },
  { id: "node-modules", name: "Stale node_modules", description: "node_modules in projects untouched for 30+ days", group: "developer", risk: "moderate", requiresAdmin: false, method: "files", bytes: 1.4 * GB, items: [[`${HOME}/Projects/webapp/node_modules`, 1.4 * GB, 62]] },
  { id: "rust-targets", name: "Stale Rust target dirs", description: "Cargo target/ folders untouched for 30+ days", group: "developer", risk: "moderate", requiresAdmin: false, method: "files", bytes: 1.5 * GB, items: [[`${HOME}/Projects/old-rust-target/target`, 1.5 * GB, 95]] },
  { id: "pip-cache", name: "pip cache", description: "Python wheels and HTTP cache", group: "developer", risk: "safe", requiresAdmin: false, method: "files", bytes: 3.1 * GB, items: [[`${HOME}/.cache/pip/http`, 2.4 * GB, 3], [`${HOME}/.cache/pip/wheels`, 0.7 * GB, 3]] },
  { id: "npm-cache", name: "npm cache", description: "~/.npm/_cacache", group: "developer", risk: "safe", requiresAdmin: false, method: "command", bytes: 1.2 * GB, items: [], command: "npm cache clean --force" },
  { id: "cargo-cache", name: "Cargo registry cache", description: "Downloaded crates and git checkouts", group: "developer", risk: "safe", requiresAdmin: false, method: "files", bytes: 3.8 * GB, items: [[`${HOME}/.cargo/registry/cache`, 2.9 * GB, 4], [`${HOME}/.cargo/git/db`, 0.9 * GB, 9]] },
  { id: "docker", name: "Docker unused data", description: "Stopped containers, dangling images and build cache", group: "developer", risk: "moderate", requiresAdmin: false, method: "command", bytes: 6.2 * GB, items: [], command: "docker system prune -f", fail: "permission denied while connecting to the Docker daemon socket" },
  { id: "browser-caches", name: "Browser caches", description: "Chrome and Firefox cache (never cookies or history)", group: "applications", risk: "safe", requiresAdmin: false, method: "files", bytes: 3.8 * GB, items: [[`${HOME}/.cache/google-chrome/Default/Cache`, 2.2 * GB, 0], [`${HOME}/.cache/mozilla/firefox`, 1.6 * GB, 1]] },
  { id: "vscode-caches", name: "VS Code caches", description: "CachedData, CachedExtensionVSIXs and logs", group: "applications", risk: "safe", requiresAdmin: false, method: "files", bytes: 740 * MB, items: [[`${HOME}/.config/Code/CachedData`, 420 * MB, 4], [`${HOME}/.config/Code/CachedExtensionVSIXs`, 260 * MB, 20], [`${HOME}/.config/Code/logs`, 60 * MB, 1]] },
  { id: "flatpak-unused", name: "Unused Flatpak runtimes", description: "flatpak uninstall --unused", group: "applications", risk: "safe", requiresAdmin: false, method: "command", bytes: 700 * MB, items: [], command: "flatpak uninstall --unused -y" },
  { id: "user-cache", name: "Other user caches", description: "Remaining ~/.cache folders not covered above", group: "user", risk: "moderate", requiresAdmin: false, method: "files", bytes: 2 * GB, items: [[`${HOME}/.cache/JetBrains`, 1.1 * GB, 12], [`${HOME}/.cache/yarn`, 0.8 * GB, 40], [`${HOME}/.cache/ms-playwright`, 0.9 * GB, 25]] },
];

// ---------- Snapshots ----------
/** Folder sizes (GB) at each snapshot, oldest first. Drives both the snapshot list and the growth diff. */
export const SNAP_FOLDERS: Record<string, number[]> = {
  ".cache/pip": [0.4, 1.2, 2.2, 3.1], "Projects/reclaim/target": [0.2, 1.1, 2.4, 3.2], "Downloads": [7.0, 7.2, 10.8, 11.4],
  "snap/code": [1.8, 1.8, 1.8, 1.8], "Videos/screencasts": [2.9, 2.9, 2.6, 2.4], ".cargo/registry": [2.4, 2.6, 2.8, 2.9],
  "Projects/ml-experiments/venv": [0, 0, 1.9, 1.9], ".local/share/Trash": [0.1, 0.1, 0.4, 1.3], "Pictures/Photos": [2.5, 2.5, 2.6, 2.6],
  "Projects/webapp/node_modules": [1.4, 1.4, 1.4, 1.4], ".npm/_cacache": [0.6, 0.9, 1.0, 1.2],
};
const SNAP_TOTAL_GB = [44.8, 47.1, 49.6, 52.1];
const SNAP_DISK_GB = [104.2, 108.6, 111.9, 113.9];
export const SNAPSHOT_AGES_DAYS = [14, 9, 4, 0];
export const SNAPSHOTS: SnapshotMeta[] = SNAP_TOTAL_GB.map((t, i) => ({
  id: i + 1, root: HOME, takenAtMs: NOW - SNAPSHOT_AGES_DAYS[i] * DAY, totalBytes: Math.round(t * GB), fileCount: 596_000 + i * 9_400,
  diskUsedBytes: Math.round(SNAP_DISK_GB[i] * GB), diskTotalBytes: 134 * GB,
}));

// ---------- Misc ----------
export const DELETED_OPEN: DeletedOpenFile[] = [
  { pid: 2841, processName: "firefox", path: "/home/basith/.cache/mozilla/firefox/cache2/doomed/old-profile.sqlite (deleted)", sizeBytes: 1.1 * GB },
  { pid: 9120, processName: "code", path: "/home/basith/.config/Code/logs/exthost-old.log (deleted)", sizeBytes: 420 * MB },
];

export const DEFAULT_SETTINGS: Settings = { theme: "system", excludedPaths: [`${HOME}/.steam`], permanentDelete: false, confirmBeforeClean: true, crossFilesystems: false, defaultScanRoot: null, saveSnapshots: true };
