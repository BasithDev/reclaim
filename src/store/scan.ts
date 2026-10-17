// Reclaim: scan state (zustand). Starts/cancels scans and subscribes to the scan:// events once.
// Shell menu/shortcuts and the Dashboard call startScan/cancelScan; see docs/CONTRACT.md.

import { create } from "zustand";
import { api, events } from "../lib/api";
import type { ScanProgress, ScanSummary } from "../lib/types";

export type ScanStatus = "idle" | "scanning" | "done" | "error";

interface ScanState {
  status: ScanStatus;
  progress: ScanProgress | null;
  summary: ScanSummary | null;
  scanId: string | null;
  error: string | null;
  /** Scans the given root, or the default scan root from settings, or the home directory. */
  startScan: (root?: string) => Promise<void>;
  cancelScan: () => Promise<void>;
}

export const useScanStore = create<ScanState>((set, get) => ({
  status: "idle",
  progress: null,
  summary: null,
  scanId: null,
  error: null,

  startScan: async (root) => {
    if (get().status === "scanning") return;
    set({ status: "scanning", progress: null, error: null });
    try {
      const [settings, info] = await Promise.all([api.getSettings(), api.getAppInfo()]);
      const scanId = await api.startScan({
        root: root ?? settings.defaultScanRoot ?? info.homeDir,
        crossFilesystems: settings.crossFilesystems,
        excludedPaths: settings.excludedPaths,
      });
      // Events may already have finished the scan before invoke resolved.
      if (get().status === "scanning") set({ scanId });
    } catch (e) {
      set({ status: "error", error: String(e) });
    }
  },

  cancelScan: async () => {
    const { scanId, status } = get();
    if (status !== "scanning") return;
    if (scanId) {
      try {
        await api.cancelScan(scanId);
      } catch (e) {
        set({ status: "error", error: String(e) });
      }
    } else {
      set({ status: "idle", progress: null });
    }
  },
}));

let subscribed = false;
/** Subscribes to backend events exactly once (guarded so HMR/StrictMode cannot double up). */
function subscribe() {
  if (subscribed) return;
  subscribed = true;
  const s = useScanStore.setState;
  const g = useScanStore.getState;
  const safe = (p: Promise<unknown>) => p.catch(() => undefined); // not running inside Tauri
  safe(events.onScanProgress((p) => {
    const cur = g();
    if (cur.status === "scanning" && (!cur.scanId || cur.scanId === p.scanId)) s({ progress: p, scanId: p.scanId });
  }));
  safe(events.onScanComplete((summary) => s({ status: "done", summary, scanId: summary.scanId, error: null })));
  safe(events.onScanError((e) => {
    if (e.cancelled) s({ status: g().summary ? "done" : "idle", progress: null, error: null });
    else s({ status: "error", error: e.message });
  }));
}
subscribe();
