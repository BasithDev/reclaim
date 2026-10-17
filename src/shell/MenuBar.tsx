// Reclaim: top bar with in-app File/View/Help menus, the current page title and a theme toggle.
import { Check, FolderSearch, Info, Keyboard, Monitor, Moon, Power, ScanLine, Sun, PanelLeft, XCircle } from "lucide-react";
import { DropdownMenu, IconButton, type MenuItem } from "../components";
import { useScanStore } from "../store/scan";
import { useSettingsStore } from "../store/settings";
import { useUiStore } from "../store/ui";
import { NAV } from "./nav";
import { newScan, quit, scanFolder, setTheme } from "./actions";

function MenuButton({ children }: { children: string }) {
  return <button className="h-7 cursor-pointer rounded-md px-2.5 text-[13px] text-muted transition-colors hover:bg-surface-2 hover:text-text">{children}</button>;
}

export function MenuBar() {
  const { page, setPage, toggleSidebar, setAboutOpen, setShortcutsOpen, resolvedTheme } = useUiStore();
  const theme = useSettingsStore((s) => s.settings.theme);
  const scanning = useScanStore((s) => s.status === "scanning");
  const check = (on: boolean) => (on ? Check : undefined);

  const file: MenuItem[] = [
    { label: "New scan", icon: ScanLine, shortcut: "Ctrl+N", onSelect: newScan, disabled: scanning },
    { label: "Scan folder…", icon: FolderSearch, shortcut: "Ctrl+O", onSelect: () => void scanFolder(), disabled: scanning },
    { label: "Cancel scan", icon: XCircle, shortcut: "Esc", onSelect: () => void useScanStore.getState().cancelScan(), disabled: !scanning },
    "separator",
    { label: "Quit", icon: Power, onSelect: quit },
  ];
  const view: MenuItem[] = [
    ...NAV.map((n) => ({ label: n.label, icon: n.icon, shortcut: n.shortcut, onSelect: () => setPage(n.id) })),
    "separator",
    { label: "Toggle sidebar", icon: PanelLeft, shortcut: "Ctrl+B", onSelect: toggleSidebar },
    "separator",
    { label: "Theme: System", icon: check(theme === "system"), onSelect: () => setTheme("system") },
    { label: "Theme: Light", icon: check(theme === "light"), onSelect: () => setTheme("light") },
    { label: "Theme: Dark", icon: check(theme === "dark"), onSelect: () => setTheme("dark") },
  ];
  const help: MenuItem[] = [
    { label: "Keyboard shortcuts", icon: Keyboard, shortcut: "?", onSelect: () => setShortcutsOpen(true) },
    { label: "About Reclaim", icon: Info, onSelect: () => setAboutOpen(true) },
  ];
  const title = NAV.find((n) => n.id === page)?.label;

  return (
    <header className="flex h-11 shrink-0 items-center justify-between border-b border-border bg-bg px-3">
      <div className="flex items-center gap-0.5">
        <DropdownMenu trigger={<MenuButton>File</MenuButton>} items={file} />
        <DropdownMenu trigger={<MenuButton>View</MenuButton>} items={view} />
        <DropdownMenu trigger={<MenuButton>Help</MenuButton>} items={help} />
      </div>
      <div className="text-[13px] font-medium text-text">{title}</div>
      <div className="flex w-[160px] justify-end">
        <DropdownMenu
          trigger={<IconButton icon={theme === "system" ? Monitor : resolvedTheme === "dark" ? Moon : Sun} label="Theme" size="sm" />}
          items={[
            { label: "System", icon: Monitor, onSelect: () => setTheme("system") },
            { label: "Light", icon: Sun, onSelect: () => setTheme("light") },
            { label: "Dark", icon: Moon, onSelect: () => setTheme("dark") },
          ]}
        />
      </div>
    </header>
  );
}
