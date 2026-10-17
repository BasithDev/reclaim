// Reclaim mock backend: installs a fake Tauri IPC layer (mockIPC) so the UI runs in a plain browser/Vitest.
// Enable with VITE_MOCK=1 (see main.tsx). Simulates a ~3 s scan with progress events, cleanups, snapshots, settings.
import { clearMocks, mockIPC, mockWindows } from "@tauri-apps/api/mocks";
import { emit } from "@tauri-apps/api/event";
import type { CleanupPreview, CleanupResult, FileCategory, ScanOptions, Settings, SnapshotMeta, TreeNode } from "../types";
import {
  CLEANERS, DEFAULT_SETTINGS, DELETED_OPEN, DISKS, GB, HOME, NOW, SCAN_PATHS, SNAPSHOTS, SNAP_FOLDERS, TYPE_SHARES,
  buildLargestFiles, buildTree, dirsOf, filesOf, find, remove, sizeOf, type MNode,
} from "./data";

const delay = (min = 120, max = 380) => new Promise<void>((r) => setTimeout(r, min + Math.random() * (max - min)));
const SCAN_MS = 3000;
const TICK_MS = 100;

const activeTimers = new Set<ReturnType<typeof setInterval>>();

export interface MockOptions { latency?: boolean; scanMs?: number }

export function installMockBackend(opts: MockOptions = {}) {
  const lat = opts.latency ?? true;
  const wait = (a?: number, b?: number) => (lat ? delay(a, b) : Promise.resolve());
  const scanMs = opts.scanMs ?? SCAN_MS;
  const tree: MNode = buildTree();
  const cleaned = new Set<string>();
  const snapshots: SnapshotMeta[] = SNAPSHOTS.map((s) => ({ ...s }));
  let settings: Settings = { ...DEFAULT_SETTINGS, excludedPaths: [...DEFAULT_SETTINGS.excludedPaths] };
  const scans = new Map<string, { cancelled: boolean; summary?: unknown }>();
  let scanCount = 0;

  const toTree = (n: MNode, depth: number, max: number): TreeNode => {
    const sorted = [...n.children].sort((a, b) => sizeOf(b) - sizeOf(a));
    const shown = depth > 0 ? sorted.slice(0, max) : [];
    const hidden = depth > 0 ? sorted.slice(max) : sorted;
    return {
      path: n.path, name: n.name, isDir: n.isDir, sizeBytes: sizeOf(n), apparentBytes: Math.round(sizeOf(n) * 1.04), fileCount: filesOf(n), dirCount: dirsOf(n),
      modifiedMs: n.mtime, children: shown.map((c) => toTree(c, depth - 1, max)), hiddenChildren: hidden.length, otherBytes: hidden.reduce((s, c) => s + sizeOf(c), 0),
    };
  };

  const preview = (id: string): CleanupPreview => {
    const c = CLEANERS.find((x) => x.id === id);
    if (!c) throw `unknown cleaner: ${id}`;
    if (cleaned.has(id)) return { cleanerId: id, totalBytes: 0, itemCount: 0, items: [], command: c.command ?? null };
    return {
      cleanerId: id, totalBytes: Math.round(c.bytes), itemCount: c.items.length || (c.command ? 1 : 0), command: c.command ?? null,
      items: c.items.map(([path, size, age]) => ({ path, sizeBytes: Math.round(size), modifiedMs: age == null ? null : NOW - age * 86_400_000, label: path.split("/").pop() ?? null })),
    };
  };

  const diff = (olderId: number, newerId: number, limit: number) => {
    const a = snapshots.findIndex((s) => s.id === olderId), b = snapshots.findIndex((s) => s.id === newerId);
    if (a < 0 || b < 0) throw "snapshot not found";
    const idx = (id: number) => SNAPSHOTS.findIndex((s) => s.id === id);
    return Object.entries(SNAP_FOLDERS)
      .map(([p, v]) => ({ path: `${HOME}/${p}`, beforeBytes: Math.round(v[idx(olderId)] * GB), afterBytes: Math.round(v[idx(newerId)] * GB) }))
      .map((e) => ({ ...e, deltaBytes: e.afterBytes - e.beforeBytes, isNew: e.beforeBytes === 0 && e.afterBytes > 0 }))
      .filter((e) => e.deltaBytes !== 0).sort((x, y) => Math.abs(y.deltaBytes) - Math.abs(x.deltaBytes)).slice(0, limit);
  };

  const startScan = (options: ScanOptions) => {
    const scanId = `mock-scan-${++scanCount}`;
    const state: { cancelled: boolean; summary?: unknown } = { cancelled: false };
    scans.set(scanId, state);
    const startedAtMs = Date.now();
    const total = sizeOf(tree), files = filesOf(tree), dirs = dirsOf(tree);
    let t = 0;
    const timer = setInterval(() => {
      t += TICK_MS;
      if (state.cancelled) {
        clearInterval(timer); activeTimers.delete(timer);
        void emit("scan://error", { scanId, message: "cancelled", cancelled: true });
        return;
      }
      const f = Math.min(1, t / scanMs);
      const eased = 1 - (1 - f) ** 2;
      if (f < 1) {
        void emit("scan://progress", {
          scanId, filesScanned: Math.round(files * eased), dirsScanned: Math.round(dirs * eased), bytesCounted: Math.round(total * eased),
          errors: Math.floor(f * 14), currentPath: `${options.root}/${SCAN_PATHS[Math.floor(t / TICK_MS) % SCAN_PATHS.length]}`, elapsedMs: t,
        });
        return;
      }
      clearInterval(timer); activeTimers.delete(timer);
      const finishedAtMs = Date.now();
      const snapshotId = settings.saveSnapshots ? snapshots.length + 1 : null;
      if (snapshotId) snapshots.push({ id: snapshotId, root: options.root, takenAtMs: finishedAtMs, totalBytes: total, fileCount: files, diskUsedBytes: DISKS[0].usedBytes, diskTotalBytes: DISKS[0].totalBytes });
      const summary = {
        scanId, root: options.root, totalBytes: total, apparentBytes: Math.round(total * 1.04), fileCount: files, dirCount: dirs, errorCount: 14,
        startedAtMs, finishedAtMs, durationMs: scanMs,
        typeBreakdown: TYPE_SHARES.map(([category, share, count]) => ({ category: category as FileCategory, bytes: Math.round(total * share), count })),
        snapshotId,
      };
      state.summary = summary;
      void emit("scan://complete", summary);
    }, TICK_MS);
    activeTimers.add(timer);
    return scanId;
  };

  const handler = async (cmd: string, a: Record<string, unknown> = {}): Promise<unknown> => {
    switch (cmd) {
      case "get_app_info": return { version: "0.1.0", os: "linux", arch: "x86_64", homeDir: HOME };
      case "list_disks": await wait(); return DISKS;
      case "start_scan": return startScan(a.options as ScanOptions);
      case "cancel_scan": { const s = scans.get(a.scanId as string); if (s) s.cancelled = true; return null; }
      case "get_scan_summary": { const s = scans.get(a.scanId as string); if (!s?.summary) throw "scan not found"; return s.summary; }
      case "get_tree": {
        await wait(80, 250);
        const n = find(tree, (a.path as string | null) ?? null);
        if (!n) throw `path not found: ${a.path}`;
        return toTree(n, a.depth as number, a.maxChildren as number);
      }
      case "get_largest_files": {
        await wait();
        const cat = a.category as FileCategory | null;
        return buildLargestFiles(tree).filter((f) => !cat || f.category === cat).slice(0, a.limit as number);
      }
      case "list_cleaners": await wait(); return CLEANERS.map(({ bytes: _b, items: _i, command: _c, fail: _f, ...info }) => info);
      case "preview_cleaner": await wait(250, 1800); return preview(a.cleanerId as string);
      case "run_cleaners": {
        await wait(900, 1600);
        return (a.cleanerIds as string[]).map((id): CleanupResult => {
          const c = CLEANERS.find((x) => x.id === id)!;
          if (c.fail) return { cleanerId: id, success: false, freedBytes: 0, itemsRemoved: 0, errors: [c.fail] };
          const p = preview(id);
          cleaned.add(id);
          return { cleanerId: id, success: true, freedBytes: p.totalBytes, itemsRemoved: p.itemCount, errors: id === "trash" ? ["1 item skipped: file is in use"] : [] };
        });
      }
      case "remove_path": { await wait(); const path = a.path as string; return remove(tree, path) || 120 * 1024 ** 2; }
      case "get_deleted_open_files": await wait(); return DELETED_OPEN;
      case "list_snapshots": await wait(); return snapshots;
      case "delete_snapshot": { const i = snapshots.findIndex((s) => s.id === a.id); if (i >= 0) snapshots.splice(i, 1); return null; }
      case "diff_snapshots": await wait(); return diff(a.olderId as number, a.newerId as number, a.limit as number);
      case "get_settings": await wait(60, 160); return settings;
      case "save_settings": settings = a.settings as Settings; return null;
      // Plugins
      case "plugin:dialog|open": await wait(100, 250); return `${HOME}/Projects`;
      case "plugin:dialog|save": return `${HOME}/export.json`;
      case "plugin:dialog|message": case "plugin:dialog|ask": case "plugin:dialog|confirm": return true;
      case "plugin:opener|open_url": case "plugin:opener|open_path": case "plugin:opener|reveal_item_in_dir": return null;
      case "plugin:path|resolve_directory": return `${HOME}/.local/share/dev.reclaim.app`;
      default:
        if (cmd.startsWith("plugin:window|") || cmd.startsWith("plugin:event|")) return null;
        throw `mock backend: unknown command "${cmd}"`;
    }
  };

  mockWindows("main");
  mockIPC(handler as Parameters<typeof mockIPC>[0], { shouldMockEvents: true });
}

export function uninstallMockBackend() {
  activeTimers.forEach((t) => clearInterval(t));
  activeTimers.clear();
  clearMocks();
}
