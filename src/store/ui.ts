// Reclaim: global UI state (current page, resolved theme). Owned by A3; other agents
// may read it and call setPage() for navigation. See docs/CONTRACT.md.

import { create } from "zustand";

export type PageId = "dashboard" | "explorer" | "files" | "cleanup" | "history" | "settings";

interface UiState {
  page: PageId;
  /** Optional path the target page should open at (e.g. Explorer drill-down). */
  pagePath: string | null;
  setPage: (page: PageId, path?: string | null) => void;
}

export const useUiStore = create<UiState>((set) => ({
  page: "dashboard",
  pagePath: null,
  setPage: (page, path = null) => set({ page, pagePath: path }),
}));
