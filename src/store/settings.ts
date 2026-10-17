// Reclaim: persisted app settings (zustand). load() reads from the backend, update() saves and applies the theme.
import { create } from "zustand";
import { api } from "../lib/api";
import type { Settings, ThemePreference } from "../lib/types";
import { useUiStore } from "./ui";

export const DEFAULT_SETTINGS: Settings = {
  theme: "system",
  excludedPaths: [],
  permanentDelete: false,
  confirmBeforeClean: true,
  crossFilesystems: false,
  defaultScanRoot: null,
  saveSnapshots: true,
};

const THEME_KEY = "reclaim.theme";
let mql: MediaQueryList | null = null;
let mqlListener: (() => void) | null = null;

/** Applies a theme preference to <html data-theme>, following the OS when "system". */
export function applyTheme(pref: ThemePreference, animate = true) {
  const root = document.documentElement;
  mql ??= window.matchMedia("(prefers-color-scheme: dark)");
  if (mqlListener) mql.removeEventListener("change", mqlListener);
  const set = () => {
    const resolved = pref === "system" ? (mql!.matches ? "dark" : "light") : pref;
    if (animate) {
      root.classList.add("theme-transition");
      window.setTimeout(() => root.classList.remove("theme-transition"), 260);
    }
    root.setAttribute("data-theme", resolved);
    useUiStore.getState().setResolvedTheme(resolved);
  };
  set();
  if (pref === "system") { mqlListener = () => set(); mql.addEventListener("change", mqlListener); } else mqlListener = null;
  try { localStorage.setItem(THEME_KEY, pref); } catch { /* ignore */ }
}

/** Apply the cached theme immediately (before the backend answers) to avoid a flash. */
export function applyCachedTheme() {
  let pref: ThemePreference = "system";
  try { const v = localStorage.getItem(THEME_KEY); if (v === "light" || v === "dark" || v === "system") pref = v; } catch { /* ignore */ }
  applyTheme(pref, false);
}

interface SettingsState {
  settings: Settings;
  loaded: boolean;
  error: string | null;
  load: () => Promise<void>;
  update: (partial: Partial<Settings>) => Promise<void>;
}

export const useSettingsStore = create<SettingsState>((set, get) => ({
  settings: DEFAULT_SETTINGS,
  loaded: false,
  error: null,
  load: async () => {
    try {
      const settings = await api.getSettings();
      set({ settings, loaded: true, error: null });
      applyTheme(settings.theme, false);
    } catch (e) {
      set({ loaded: true, error: String(e) });
    }
  },
  update: async (partial) => {
    const prev = get().settings;
    const next = { ...prev, ...partial };
    set({ settings: next });
    if (partial.theme && partial.theme !== prev.theme) applyTheme(partial.theme);
    try {
      await api.saveSettings(next);
      set({ error: null });
    } catch (e) {
      set({ settings: prev, error: String(e) });
      if (partial.theme) applyTheme(prev.theme);
      throw e;
    }
  },
}));

/** Wording + flag for deleting files from Explorer / Large Files / Dashboard, following Settings › Delete mode. */
export function useDeleteMode() {
  const permanent = useSettingsStore((s) => s.settings.permanentDelete);
  return permanent
    ? { permanent, verb: "Delete", past: "Deleted", question: "Delete permanently?", note: "This cannot be undone." }
    : { permanent, verb: "Move to Trash", past: "Moved to Trash", question: "Move to Trash?", note: "You can restore from the Trash." };
}
