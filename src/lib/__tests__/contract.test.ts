// Reclaim: contract test. Checks the JSON produced by Rust (fixtures/contract.json, written by
// src-tauri/tests/contract.rs) has exactly the keys and enum strings declared in src/lib/types.ts.
import { describe, expect, it } from "vitest";
import fixtures from "./fixtures/contract.json";
import type {
  AppInfo, CleanerGroup, CleanerInfo, CleanMethod, CleanupPreview, CleanupResult, DeletedOpenFile, DiskInfo,
  DiskKind, FileCategory, FileEntry, GrowthEntry, PreviewItem, Risk, ScanError, ScanOptions, ScanProgress,
  ScanSummary, Settings, SnapshotMeta, TreeNode, TypeStat,
} from "../types";

function keys<T>() {
  return <const K extends readonly (keyof T)[]>(k: K & ([keyof T] extends [K[number]] ? unknown : never)): K => k;
}

const expected = {
  DiskInfo: keys<DiskInfo>()(["id", "name", "mountPoint", "fileSystem", "totalBytes", "usedBytes", "availableBytes", "isRemovable", "isSystem", "kind"]),
  ScanOptions: keys<Required<ScanOptions>>()(["root", "crossFilesystems", "excludedPaths", "topFilesLimit"]),
  ScanProgress: keys<ScanProgress>()(["scanId", "filesScanned", "dirsScanned", "bytesCounted", "errors", "currentPath", "elapsedMs"]),
  ScanSummary: keys<ScanSummary>()(["scanId", "root", "totalBytes", "apparentBytes", "fileCount", "dirCount", "errorCount", "startedAtMs", "finishedAtMs", "durationMs", "typeBreakdown", "snapshotId"]),
  TypeStat: keys<TypeStat>()(["category", "bytes", "count"]),
  TreeNode: keys<TreeNode>()(["path", "name", "isDir", "sizeBytes", "apparentBytes", "fileCount", "dirCount", "modifiedMs", "children", "hiddenChildren", "otherBytes"]),
  FileEntry: keys<FileEntry>()(["path", "name", "sizeBytes", "modifiedMs", "extension", "category"]),
  SnapshotMeta: keys<SnapshotMeta>()(["id", "root", "takenAtMs", "totalBytes", "fileCount", "diskUsedBytes", "diskTotalBytes"]),
  GrowthEntry: keys<GrowthEntry>()(["path", "beforeBytes", "afterBytes", "deltaBytes", "isNew"]),
  CleanerInfo: keys<CleanerInfo>()(["id", "name", "description", "group", "risk", "requiresAdmin", "method"]),
  PreviewItem: keys<PreviewItem>()(["path", "sizeBytes", "modifiedMs", "label"]),
  CleanupPreview: keys<CleanupPreview>()(["cleanerId", "totalBytes", "itemCount", "items", "command"]),
  CleanupResult: keys<CleanupResult>()(["cleanerId", "success", "freedBytes", "itemsRemoved", "errors"]),
  DeletedOpenFile: keys<DeletedOpenFile>()(["pid", "processName", "path", "sizeBytes"]),
  Settings: keys<Settings>()(["theme", "excludedPaths", "permanentDelete", "confirmBeforeClean", "crossFilesystems", "defaultScanRoot", "saveSnapshots"]),
  AppInfo: keys<AppInfo>()(["version", "os", "arch", "homeDir"]),
  ScanError: keys<ScanError>()(["scanId", "message", "cancelled"]),
} as const;

const enums = {
  DiskKind: ["ssd", "hdd", "unknown"] as const satisfies readonly DiskKind[],
  FileCategory: ["video", "audio", "image", "document", "archive", "diskImage", "code", "executable", "database", "log", "other"] as const satisfies readonly FileCategory[],
  CleanerGroup: ["system", "developer", "applications", "user"] as const satisfies readonly CleanerGroup[],
  Risk: ["safe", "moderate", "careful"] as const satisfies readonly Risk[],
  CleanMethod: ["files", "command"] as const satisfies readonly CleanMethod[],
};

const fx = fixtures as unknown as Record<string, Record<string, unknown>> & { enums: Record<string, string[]> };

describe("Rust <-> TS contract", () => {
  for (const [name, list] of Object.entries(expected)) {
    it(`${name} has exactly the keys of the TS interface`, () => {
      expect(Object.keys(fx[name]).sort()).toEqual([...list].sort());
    });
  }

  it("nested TreeNode children have the same keys", () => {
    const child = (fx.TreeNode.children as Record<string, unknown>[])[0];
    expect(Object.keys(child).sort()).toEqual([...expected.TreeNode].sort());
  });

  it("nested ScanSummary typeBreakdown and preview items have the right keys", () => {
    const tb = (fx.ScanSummary.typeBreakdown as Record<string, unknown>[])[0];
    expect(Object.keys(tb).sort()).toEqual([...expected.TypeStat].sort());
    const item = (fx.CleanupPreview.items as Record<string, unknown>[])[0];
    expect(Object.keys(item).sort()).toEqual([...expected.PreviewItem].sort());
  });

  for (const [name, list] of Object.entries(enums)) {
    it(`${name} enum strings match the TS union`, () => {
      expect([...fx.enums[name]].sort()).toEqual([...list].sort());
    });
  }

  it("Settings defaults match CONTRACT.md", () => {
    expect(fx.Settings).toEqual({
      theme: "system", excludedPaths: [], permanentDelete: true, confirmBeforeClean: true,
      crossFilesystems: false, defaultScanRoot: null, saveSnapshots: true,
    });
  });
});
