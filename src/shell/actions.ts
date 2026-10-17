// Reclaim: shell-level actions (scan folder picker, cycle theme, quit) used by the menu bar and shortcuts.
import { open } from "@tauri-apps/plugin-dialog";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useScanStore } from "../store/scan";
import { useSettingsStore } from "../store/settings";
import { useUiStore } from "../store/ui";
import type { ThemePreference } from "../lib/types";

export async function scanFolder() {
  try {
    const dir = await open({ directory: true, multiple: false, title: "Choose a folder to scan" });
    if (typeof dir === "string") {
      useUiStore.getState().setPage("dashboard");
      await useScanStore.getState().startScan(dir);
    }
  } catch { /* dialog unavailable outside Tauri */ }
}

export function newScan() {
  useUiStore.getState().setPage("dashboard");
  void useScanStore.getState().startScan();
}

export function setTheme(t: ThemePreference) {
  void useSettingsStore.getState().update({ theme: t }).catch(() => undefined);
}

export function cycleTheme() {
  const order: ThemePreference[] = ["system", "light", "dark"];
  const cur = useSettingsStore.getState().settings.theme;
  setTheme(order[(order.indexOf(cur) + 1) % order.length]);
}

export function quit() {
  getCurrentWindow().close().catch(() => window.close());
}
