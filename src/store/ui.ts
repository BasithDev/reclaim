// Reclaim: global UI state (current page, sidebar, dialogs, resolved theme). Owned by A3; other agents
// may read it and call setPage() for navigation. See docs/CONTRACT.md.

import { create } from "zustand";

export type PageId = "dashboard" | "explorer" | "files" | "cleanup" | "history" | "settings";

interface UiState {
  page: PageId;
  /** Optional path the target page should open at (e.g. Explorer drill-down). */
  pagePath: string | null;
  setPage: (page: PageId, path?: string | null) => void;

  sidebarCollapsed: boolean;
  toggleSidebar: () => void;
  aboutOpen: boolean;
  shortcutsOpen: boolean;
  setAboutOpen: (v: boolean) => void;
  setShortcutsOpen: (v: boolean) => void;
  /** The theme actually applied to the document ("system" resolved). */
  resolvedTheme: "light" | "dark";
  setResolvedTheme: (t: "light" | "dark") => void;
}

function initialCollapsed(): boolean {
  try { return localStorage.getItem("reclaim.sidebar") === "1"; } catch { return false; }
}

export const useUiStore = create<UiState>((set, get) => ({
  page: "dashboard",
  pagePath: null,
  setPage: (page, path = null) => set({ page, pagePath: path }),

  sidebarCollapsed: initialCollapsed(),
  toggleSidebar: () => {
    const next = !get().sidebarCollapsed;
    try { localStorage.setItem("reclaim.sidebar", next ? "1" : "0"); } catch { /* ignore */ }
    set({ sidebarCollapsed: next });
  },
  aboutOpen: false,
  shortcutsOpen: false,
  setAboutOpen: (aboutOpen) => set({ aboutOpen }),
  setShortcutsOpen: (shortcutsOpen) => set({ shortcutsOpen }),
  resolvedTheme: "light",
  setResolvedTheme: (resolvedTheme) => set({ resolvedTheme }),
}));
