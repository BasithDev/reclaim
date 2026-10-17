// Reclaim: global keyboard shortcuts for the app (pages, scan, sidebar, theme, help).
import { useEffect } from "react";
import { useUiStore } from "../store/ui";
import { useScanStore } from "../store/scan";
import { NAV } from "./nav";
import { cycleTheme, newScan, scanFolder } from "./actions";

export function useShortcuts() {
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      const ui = useUiStore.getState();
      const el = e.target as HTMLElement | null;
      const typing = !!el && (el.tagName === "INPUT" || el.tagName === "TEXTAREA" || el.isContentEditable);
      const mod = e.ctrlKey || e.metaKey;

      if (mod && !e.shiftKey && e.key >= "1" && e.key <= "6") { e.preventDefault(); ui.setPage(NAV[Number(e.key) - 1].id); return; }
      if (mod && !e.shiftKey && e.key.toLowerCase() === "n") { e.preventDefault(); newScan(); return; }
      if (mod && !e.shiftKey && e.key.toLowerCase() === "o") { e.preventDefault(); void scanFolder(); return; }
      if (mod && !e.shiftKey && e.key.toLowerCase() === "b") { e.preventDefault(); ui.toggleSidebar(); return; }
      if (mod && e.shiftKey && e.key.toLowerCase() === "l") { e.preventDefault(); cycleTheme(); return; }
      if (mod && e.key === ",") { e.preventDefault(); ui.setPage("settings"); return; }
      if (e.key === "Escape" && !document.querySelector("[role=dialog]") && useScanStore.getState().status === "scanning") {
        void useScanStore.getState().cancelScan();
        return;
      }
      if (e.key === "?" && !typing && !mod) { e.preventDefault(); ui.setShortcutsOpen(true); }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, []);
}
