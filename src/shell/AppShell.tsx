// Reclaim: the app frame: sidebar, menu bar, animated page outlet, dialogs and shortcuts.
import { Suspense, type ComponentType } from "react";
import { AnimatePresence, motion } from "motion/react";
import { ToastProvider } from "../components";
import { useUiStore, type PageId } from "../store/ui";
import DashboardPage from "../pages/dashboard";
import ExplorerPage from "../pages/explorer";
import FilesPage from "../pages/files";
import CleanupPage from "../pages/cleanup";
import HistoryPage from "../pages/history";
import SettingsPage from "../pages/settings";
import { MenuBar } from "./MenuBar";
import { Sidebar } from "./Sidebar";
import { AboutDialog, ShortcutsDialog } from "./Dialogs";
import { useShortcuts } from "./useShortcuts";

const PAGES: Record<PageId, ComponentType> = {
  dashboard: DashboardPage, explorer: ExplorerPage, files: FilesPage,
  cleanup: CleanupPage, history: HistoryPage, settings: SettingsPage,
};

export function AppShell() {
  const page = useUiStore((s) => s.page);
  useShortcuts();
  const Page = PAGES[page];
  return (
    <ToastProvider>
      <div className="flex h-full bg-bg text-text">
        <Sidebar />
        <div className="flex min-w-0 flex-1 flex-col">
          <MenuBar />
          <main className="relative min-h-0 flex-1 overflow-y-auto">
            <AnimatePresence mode="wait" initial={false}>
              <motion.div
                key={page}
                initial={{ opacity: 0, y: 8 }} animate={{ opacity: 1, y: 0 }} exit={{ opacity: 0, y: -8 }}
                transition={{ duration: 0.18, ease: "easeOut" }}
                className="mx-auto w-full max-w-6xl px-8 py-8"
              >
                <Suspense fallback={null}><Page /></Suspense>
              </motion.div>
            </AnimatePresence>
          </main>
        </div>
      </div>
      <AboutDialog />
      <ShortcutsDialog />
    </ToastProvider>
  );
}
