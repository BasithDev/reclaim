// Reclaim: collapsible left sidebar with navigation and a system-disk usage meter.
import { useEffect, useState } from "react";
import { motion } from "motion/react";
import { HardDrive, PanelLeftClose, PanelLeftOpen } from "lucide-react";
import { IconButton, ProgressBar, Tooltip, cn } from "../components";
import { api } from "../lib/api";
import { formatBytes } from "../lib/format";
import type { DiskInfo } from "../lib/types";
import { useScanStore } from "../store/scan";
import { useUiStore } from "../store/ui";
import { NAV } from "./nav";

function DiskMeter({ collapsed }: { collapsed: boolean }) {
  const [disk, setDisk] = useState<DiskInfo | null>(null);
  const status = useScanStore((s) => s.status);
  useEffect(() => {
    api.listDisks().then((d) => setDisk(d.find((x) => x.isSystem) ?? d[0] ?? null)).catch(() => setDisk(null));
  }, [status]);
  if (!disk) return null;
  const ratio = disk.totalBytes ? disk.usedBytes / disk.totalBytes : 0;
  const tone = ratio > 0.9 ? "danger" : ratio > 0.75 ? "warning" : "accent";
  const tip = `${disk.name || disk.mountPoint}: ${formatBytes(disk.usedBytes)} of ${formatBytes(disk.totalBytes)} used`;
  if (collapsed) {
    return (
      <Tooltip content={tip}>
        <div className="mx-auto flex w-10 flex-col items-center gap-1.5"><HardDrive size={16} className="text-muted" /><ProgressBar value={ratio} tone={tone} /></div>
      </Tooltip>
    );
  }
  return (
    <div className="rounded-lg border border-border bg-surface p-3">
      <div className="mb-2 flex items-center gap-2 text-xs text-muted">
        <HardDrive size={14} /><span className="truncate font-medium text-text">{disk.name || disk.mountPoint}</span>
        <span className="ml-auto tnum">{Math.round(ratio * 100)}%</span>
      </div>
      <ProgressBar value={ratio} tone={tone} />
      <div className="mt-2 text-[11px] text-muted tnum">{formatBytes(disk.availableBytes)} free of {formatBytes(disk.totalBytes)}</div>
    </div>
  );
}

export function Sidebar() {
  const { page, setPage, sidebarCollapsed: collapsed, toggleSidebar } = useUiStore();
  const scanning = useScanStore((s) => s.status === "scanning");
  return (
    <motion.aside
      initial={false} animate={{ width: collapsed ? 64 : 232 }} transition={{ type: "spring", stiffness: 380, damping: 36 }}
      className="flex shrink-0 flex-col border-r border-border bg-surface-2/60 px-3 pb-3 pt-3"
    >
      <div className={cn("mb-4 flex h-9 items-center", collapsed ? "justify-center" : "justify-between pl-1")}>
        <div className="flex items-center gap-2.5 overflow-hidden">
          <div className="flex h-8 w-8 shrink-0 items-center justify-center rounded-lg bg-accent text-accent-contrast"><HardDrive size={17} /></div>
          {!collapsed && <span className="text-[15px] font-semibold tracking-tight text-text">Reclaim</span>}
        </div>
      </div>
      <nav className="flex flex-1 flex-col gap-0.5">
        {NAV.map(({ id, label, icon: Icon, shortcut }) => {
          const active = page === id;
          const btn = (
            <button
              key={id} onClick={() => setPage(id)} aria-current={active ? "page" : undefined}
              className={cn(
                "relative flex h-9 w-full cursor-pointer items-center gap-3 rounded-lg text-[13px] font-medium transition-colors",
                collapsed ? "justify-center" : "px-3",
                active ? "text-text" : "text-muted hover:bg-surface-3/60 hover:text-text",
              )}
            >
              {active && <motion.span layoutId="nav-active" className="absolute inset-0 rounded-lg border border-border bg-surface shadow-sm" transition={{ type: "spring", stiffness: 500, damping: 38 }} />}
              <span className="relative flex items-center gap-3">
                <Icon size={17} className={active ? "text-accent" : undefined} />
                {!collapsed && label}
              </span>
              {id === "dashboard" && scanning && <span className="absolute right-3 h-1.5 w-1.5 animate-pulse rounded-full bg-accent" />}
            </button>
          );
          return collapsed ? <Tooltip key={id} content={`${label}  ${shortcut}`}>{btn}</Tooltip> : btn;
        })}
      </nav>
      <div className="space-y-2">
        <DiskMeter collapsed={collapsed} />
        <div className={cn("flex", collapsed ? "justify-center" : "justify-end")}>
          <IconButton icon={collapsed ? PanelLeftOpen : PanelLeftClose} label={collapsed ? "Expand sidebar (Ctrl+B)" : "Collapse sidebar (Ctrl+B)"} size="sm" onClick={toggleSidebar} />
        </div>
      </div>
    </motion.aside>
  );
}
