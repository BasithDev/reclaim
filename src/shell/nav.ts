// Reclaim: navigation item definitions and the keyboard shortcut table shared by the sidebar, menus and dialogs.
import { Folders, History, LayoutDashboard, Settings, Sparkles, FileSearch, type LucideIcon } from "lucide-react";
import type { PageId } from "../store/ui";

export const NAV: { id: PageId; label: string; icon: LucideIcon; shortcut: string }[] = [
  { id: "dashboard", label: "Dashboard", icon: LayoutDashboard, shortcut: "Ctrl+1" },
  { id: "explorer", label: "Explorer", icon: Folders, shortcut: "Ctrl+2" },
  { id: "files", label: "Large Files", icon: FileSearch, shortcut: "Ctrl+3" },
  { id: "cleanup", label: "Cleanup", icon: Sparkles, shortcut: "Ctrl+4" },
  { id: "history", label: "History", icon: History, shortcut: "Ctrl+5" },
  { id: "settings", label: "Settings", icon: Settings, shortcut: "Ctrl+6" },
];

export const SHORTCUTS: { keys: string; label: string }[] = [
  { keys: "Ctrl+1 … 6", label: "Go to page" },
  { keys: "Ctrl+N", label: "New scan" },
  { keys: "Ctrl+O", label: "Scan folder…" },
  { keys: "Esc", label: "Cancel scan" },
  { keys: "Ctrl+B", label: "Toggle sidebar" },
  { keys: "Ctrl+Shift+L", label: "Cycle theme" },
  { keys: "Ctrl+,", label: "Open settings" },
  { keys: "?", label: "Show keyboard shortcuts" },
];
