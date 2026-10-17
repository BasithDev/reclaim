// Reclaim: Explorer page. Treemap/sunburst of the current folder, breadcrumbs, child list with row actions.
// Owned by A4; reads the finished scan from the scan store and drills down via api.getTree.
import { useEffect, useMemo, useState } from "react";
import { AnimatePresence, motion } from "motion/react";
import ReactECharts from "echarts-for-react";
import { ArrowUp, ChevronRight, File, Folder, FolderOpen, FolderSearch, MoreHorizontal, RefreshCw, Trash2, Eye, Grid2x2, Sun } from "lucide-react";
import { openPath, revealItemInDir } from "@tauri-apps/plugin-opener";
import { api } from "../../lib/api";
import { formatBytes, formatCount, formatPercent, formatRelative } from "../../lib/format";
import type { TreeNode } from "../../lib/types";
import {
  Button, Card, ConfirmDialog, DropdownMenu, EmptyState, IconButton, PageHeader, SegmentedControl, Skeleton,
  Tooltip, useChartTheme, useToast,
} from "../../components";
import { useScanStore } from "../../store/scan";
import { useDeleteMode } from "../../store/settings";
import { useUiStore } from "../../store/ui";
import { useAsync } from "../dashboard/useAsync";

type View = "treemap" | "sunburst";

/** Breadcrumb segments between the scan root and `path` (both may use / or \). */
function crumbs(root: string, path: string | null) {
  const out = [{ label: root, path: null as string | null }];
  if (!path || path === root) return out;
  const sep = path.includes("\\") && !path.includes("/") ? "\\" : "/";
  const rest = path.slice(root.length).split(/[\\/]/).filter(Boolean);
  let cur = root.replace(/[\\/]+$/, "");
  for (const seg of rest) {
    cur = `${cur}${sep}${seg}`;
    out.push({ label: seg, path: cur });
  }
  return out;
}

export default function ExplorerPage() {
  const del = useDeleteMode();
  const status = useScanStore((s) => s.status);
  const summary = useScanStore((s) => s.summary);
  const startScan = useScanStore((s) => s.startScan);
  const pagePath = useUiStore((s) => s.pagePath);
  const [path, setPath] = useState<string | null>(() => useUiStore.getState().pagePath);
  const [view, setView] = useState<View>("treemap");
  const [removed, setRemoved] = useState<Set<string>>(new Set());
  const [confirm, setConfirm] = useState<TreeNode | null>(null);
  const [busy, setBusy] = useState(false);
  const { toast } = useToast();
  const theme = useChartTheme();

  const scanId = summary?.scanId ?? null;
  const root = summary?.root ?? "";

  // Open at the path other pages asked for (Dashboard bar click), and reset for a new scan.
  useEffect(() => { setPath(pagePath && pagePath !== root ? pagePath : null); setRemoved(new Set()); }, [pagePath, scanId, root]);

  const { data, loading, error, reload } = useAsync(
    () => (scanId ? api.getTree(scanId, path, 2, 40) : Promise.reject("no scan")),
    [scanId, path],
  );

  const children = useMemo(() => (data?.children ?? []).filter((c) => !removed.has(c.path)), [data, removed]);
  const total = data?.sizeBytes ?? 0;

  const parentOf = (p: string | null) => {
    const c = crumbs(root, p);
    return c.length > 1 ? c[c.length - 2].path : null;
  };

  if (!scanId) {
    return (
      <div className="grid gap-6">
        <PageHeader title="Explorer" description="Drill into folders to see what is big." />
        <Card>
          <EmptyState icon={FolderSearch} title={status === "scanning" ? "Scan in progress" : "Nothing scanned yet"}
            description={status === "scanning" ? "The map appears when the scan finishes." : "Run a scan to explore your folders as a map."}
            action={status === "scanning" ? undefined : <Button variant="primary" onClick={() => void startScan()}>Scan now</Button>} />
        </Card>
      </div>
    );
  }

  const open = (n: TreeNode) => { if (n.isDir) setPath(n.path); };
  const trash = async () => {
    if (!confirm) return;
    setBusy(true);
    try {
      const freed = await api.removePath(confirm.path, del.permanent);
      setRemoved((s) => new Set(s).add(confirm.path));
      toast({ title: del.past, description: `${confirm.name} (${formatBytes(freed)})`, tone: "success" });
      setConfirm(null);
    } catch (e) {
      toast({ title: "Could not remove", description: String(e), tone: "error" });
    } finally { setBusy(false); }
  };

  const bc = crumbs(root, path);
  const palette = theme.palette;
  const mkNode = (n: TreeNode, i: number, depth: number): Record<string, unknown> => ({
    name: n.name, value: n.sizeBytes, path: n.path, isDir: n.isDir,
    itemStyle: depth === 0 ? { color: palette[i % palette.length] } : undefined,
    children: n.children.length ? n.children.map((c) => mkNode(c, i, depth + 1)) : undefined,
  });
  const chartData = children.map((c, i) => mkNode(c, i, 0));
  const tip = {
    backgroundColor: theme.tooltipBg, borderColor: theme.tooltipBorder, textStyle: { color: theme.textColor },
    formatter: (p: { name: string; value: number; data: { path?: string } }) =>
      `${p.data.path ?? p.name}<br/><b>${formatBytes(p.value)}</b> (${formatPercent(p.value, total, 1)})`,
  };
  const onEvents = { click: (p: { data?: { path?: string; isDir?: boolean } }) => { if (p.data?.isDir && p.data.path) setPath(p.data.path); } };

  const option = view === "treemap"
    ? {
        animationDurationUpdate: 500,
        tooltip: tip,
        series: [{
          type: "treemap", roam: false, nodeClick: false, breadcrumb: { show: false }, left: 0, top: 0, right: 0, bottom: 0,
          squareRatio: 1.2, visibleMin: 300,
          label: { show: true, formatter: "{b}", color: "#fff", overflow: "truncate" },
          upperLabel: { show: true, height: 22, color: "#fff", formatter: "{b}" },
          itemStyle: { borderColor: theme.tooltipBg, borderWidth: 2, gapWidth: 2, borderRadius: 4 },
          levels: [
            { itemStyle: { borderWidth: 0, gapWidth: 4 } },
            { itemStyle: { gapWidth: 2, borderColorSaturation: 0.7 }, colorSaturation: [0.35, 0.6] },
          ],
          data: chartData,
        }],
      }
    : {
        animationDurationUpdate: 500,
        tooltip: tip,
        series: [{
          type: "sunburst", radius: ["12%", "95%"], sort: "desc", nodeClick: false,
          itemStyle: { borderColor: theme.tooltipBg, borderWidth: 1.5, borderRadius: 4 },
          label: { rotate: "radial", color: "#fff", minAngle: 14, overflow: "truncate" },
          data: chartData,
        }],
      };

  return (
    <div className="grid gap-5">
      <PageHeader title="Explorer" description="Click a folder to drill down."
        actions={<SegmentedControl value={view} onChange={(v) => setView(v as View)}
          options={[{ value: "treemap", label: "Treemap", icon: Grid2x2 }, { value: "sunburst", label: "Sunburst", icon: Sun }]} />} />

      <div className="flex items-center gap-2">
        <Tooltip content="Up one level">
          <IconButton icon={ArrowUp} label="Up one level" disabled={path === null} onClick={() => setPath(parentOf(path))} />
        </Tooltip>
        <nav className="flex min-w-0 flex-1 flex-wrap items-center gap-0.5 text-sm" aria-label="Breadcrumb">
          {bc.map((c, i) => (
            <span key={c.path ?? "root"} className="flex items-center gap-0.5">
              {i > 0 && <ChevronRight size={14} className="text-faint" />}
              <button disabled={i === bc.length - 1} onClick={() => setPath(c.path)}
                className={`max-w-56 cursor-pointer truncate rounded-md px-1.5 py-0.5 transition-colors ${i === bc.length - 1 ? "font-semibold text-text" : "text-muted hover:bg-surface-2 hover:text-text"}`}>
                {c.label}
              </button>
            </span>
          ))}
        </nav>
        {data && <span className="tnum text-sm text-muted">{formatBytes(total)}</span>}
      </div>

      {error && !data ? (
        <Card>
          <EmptyState icon={FolderSearch} title="Could not load this folder" description={error}
            action={<Button icon={RefreshCw} onClick={reload}>Retry</Button>} />
        </Card>
      ) : (
        <div className="grid gap-5 xl:grid-cols-[minmax(0,1.6fr)_minmax(0,1fr)]">
          <Card padded={false} className="overflow-hidden">
            <div className="h-[460px] p-2">
              {loading && !data ? <Skeleton className="h-full w-full" />
                : children.length === 0 ? <EmptyState icon={FolderOpen} title="This folder is empty" description="There is nothing to draw here." />
                : <ReactECharts key={view} style={{ height: "100%", width: "100%" }} notMerge option={option} onEvents={onEvents} />}
            </div>
          </Card>

          <Card title="Contents" subtitle={data ? `${formatCount(data.fileCount)} files · ${formatCount(data.dirCount)} folders` : undefined} padded={false}>
            <div className="max-h-[460px] overflow-y-auto px-2 pb-2 pt-2">
              {loading && !data ? (
                <div className="grid gap-2 p-3">{Array.from({ length: 8 }).map((_, i) => <Skeleton key={i} className="h-9" />)}</div>
              ) : children.length === 0 ? (
                <EmptyState icon={FolderOpen} title="Empty" description="No items in this folder." />
              ) : (
                <AnimatePresence initial={false}>
                  {children.map((c, i) => {
                    const frac = total > 0 ? c.sizeBytes / total : 0;
                    return (
                      <motion.div key={c.path} layout initial={{ opacity: 0, x: 8 }} animate={{ opacity: 1, x: 0 }} exit={{ opacity: 0, height: 0 }}
                        transition={{ delay: Math.min(i, 12) * 0.02 }}
                        className="group relative flex items-center gap-3 rounded-lg px-3 py-2 hover:bg-surface-2">
                        <button onClick={() => open(c)} className={`flex min-w-0 flex-1 items-center gap-3 text-left ${c.isDir ? "cursor-pointer" : "cursor-default"}`}>
                          <span className="shrink-0" style={{ color: palette[i % palette.length] }}>{c.isDir ? <Folder size={18} /> : <File size={18} />}</span>
                          <span className="min-w-0 flex-1">
                            <span className="block truncate text-sm text-text" title={c.path}>{c.name}</span>
                            <span className="mt-1 block h-1 overflow-hidden rounded-full bg-surface-3">
                              <motion.span className="block h-full rounded-full" style={{ background: palette[i % palette.length] }}
                                initial={{ width: 0 }} animate={{ width: `${Math.max(frac * 100, 1)}%` }} transition={{ duration: 0.6, ease: [0.16, 1, 0.3, 1] }} />
                            </span>
                          </span>
                          <span className="w-28 shrink-0 text-right text-xs">
                            <span className="tnum block whitespace-nowrap font-medium text-text">{formatBytes(c.sizeBytes)} <span className="font-normal text-muted">· {formatPercent(c.sizeBytes, total, 1)}</span></span>
                            {c.modifiedMs != null && <span className="block truncate whitespace-nowrap text-faint">{formatRelative(c.modifiedMs)}</span>}
                          </span>
                        </button>
                        <DropdownMenu
                          trigger={<IconButton icon={MoreHorizontal} label="Actions" size="sm" />}
                          items={[
                            { label: c.isDir ? "Open in file manager" : "Open", icon: FolderOpen, onSelect: () => void openPath(c.path).catch((e) => toast({ title: "Could not open", description: String(e), tone: "error" })) },
                            { label: "Reveal in file manager", icon: Eye, onSelect: () => void revealItemInDir(c.path).catch((e) => toast({ title: "Could not reveal", description: String(e), tone: "error" })) },
                            "separator",
                            { label: del.verb, icon: Trash2, danger: true, onSelect: () => setConfirm(c) },
                          ]}
                        />
                      </motion.div>
                    );
                  })}
                </AnimatePresence>
              )}
              {data && data.hiddenChildren > 0 && (
                <p className="px-3 py-2 text-xs text-muted">+ {formatCount(data.hiddenChildren)} smaller items ({formatBytes(data.otherBytes)})</p>
              )}
            </div>
          </Card>
        </div>
      )}

      <ConfirmDialog open={!!confirm} onClose={() => setConfirm(null)} onConfirm={() => void trash()} loading={busy} tone="danger"
        title={del.question} confirmLabel={del.verb}
        message={confirm && <><b>{confirm.name}</b> ({formatBytes(confirm.sizeBytes)}). {del.note}</>} />
    </div>
  );
}
