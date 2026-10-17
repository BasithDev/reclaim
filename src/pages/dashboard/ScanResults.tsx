// Reclaim: Dashboard results after a scan: stat tiles, type donut, top folders bar chart,
// reclaimable summary, deleted-but-open warning and "grew since last scan".
import { motion } from "motion/react";
import ReactECharts from "echarts-for-react";
import { AlertTriangle, ArrowUpRight, Clock, Files, FolderTree, HardDrive, RefreshCw, Sparkles, TrendingUp } from "lucide-react";
import { api } from "../../lib/api";
import { formatBytes, formatCount, formatDelta, formatDuration } from "../../lib/format";
import type { ScanSummary } from "../../lib/types";
import { AnimatedNumber, Badge, Button, Card, EmptyState, Skeleton, StatTile, categoryMeta, useChartTheme } from "../../components";
import { useUiStore } from "../../store/ui";
import { useAsync } from "./useAsync";

const item = { hidden: { opacity: 0, y: 12 }, show: { opacity: 1, y: 0, transition: { duration: 0.4, ease: [0.16, 1, 0.3, 1] as const } } };

function ErrorRow({ message, retry }: { message: string; retry: () => void }) {
  return (
    <div className="flex items-center justify-between gap-3 text-sm text-danger">
      <span className="min-w-0 truncate">{message}</span>
      <Button size="sm" icon={RefreshCw} onClick={retry}>Retry</Button>
    </div>
  );
}

export default function ScanResults({ summary }: { summary: ScanSummary }) {
  return (
    <motion.div initial="hidden" animate="show" transition={{ staggerChildren: 0.06 }} className="grid gap-4">
      <motion.div variants={item} className="grid grid-cols-2 gap-4 lg:grid-cols-4">
        <StatTile label="Scanned size" icon={HardDrive} tone="accent" value={<AnimatedNumber value={summary.totalBytes} format={formatBytes} />} sub={summary.root} />
        <StatTile label="Files" icon={Files} value={<AnimatedNumber value={summary.fileCount} />} />
        <StatTile label="Folders" icon={FolderTree} value={<AnimatedNumber value={summary.dirCount} />} sub={summary.errorCount ? `${formatCount(summary.errorCount)} unreadable` : undefined} />
        <StatTile label="Duration" icon={Clock} value={formatDuration(summary.durationMs)} />
      </motion.div>
      <DeletedOpen />
      <motion.div variants={item} className="grid gap-4 lg:grid-cols-2">
        <TypeDonut summary={summary} />
        <TopFolders summary={summary} />
      </motion.div>
      <motion.div variants={item} className="grid gap-4 lg:grid-cols-2">
        <Reclaimable />
        <Growth summary={summary} />
      </motion.div>
    </motion.div>
  );
}

function TypeDonut({ summary }: { summary: ScanSummary }) {
  const t = useChartTheme();
  const data = summary.typeBreakdown.filter((s) => s.bytes > 0).sort((a, b) => b.bytes - a.bytes);
  return (
    <Card title="What's using the space" subtitle="By file type">
      {data.length === 0 ? (
        <EmptyState icon={HardDrive} title="Nothing to show" description="No files were counted in this scan." />
      ) : (
        <div className="flex flex-col items-center gap-4 sm:flex-row">
          <ReactECharts
            style={{ height: 240, width: 240, flexShrink: 0 }}
            notMerge
            option={{
              animationDuration: 900, animationEasing: "cubicOut",
              tooltip: {
                trigger: "item", backgroundColor: t.tooltipBg, borderColor: t.tooltipBorder, textStyle: { color: t.textColor },
                formatter: (p: { name: string; value: number; percent: number }) => `${p.name}<br/><b>${formatBytes(p.value)}</b> (${p.percent}%)`,
              },
              series: [{
                type: "pie", radius: ["58%", "88%"], padAngle: 2, itemStyle: { borderRadius: 6 },
                label: { show: false }, emphasis: { scale: true, scaleSize: 6 },
                data: data.map((s) => ({ name: categoryMeta[s.category].label, value: s.bytes, itemStyle: { color: categoryMeta[s.category].color } })),
              }],
              graphic: [{ type: "text", left: "center", top: "center", style: { text: formatBytes(summary.totalBytes), fill: t.textColor, fontSize: 18, fontWeight: 600 } }],
            }}
          />
          <ul className="grid w-full flex-1 gap-1.5 text-sm">
            {data.slice(0, 7).map((s) => (
              <li key={s.category} className="flex items-center gap-2">
                <span className="h-2.5 w-2.5 rounded-full" style={{ background: categoryMeta[s.category].color }} />
                <span className="flex-1 truncate text-text">{categoryMeta[s.category].label}</span>
                <span className="tnum text-muted">{formatBytes(s.bytes)}</span>
              </li>
            ))}
          </ul>
        </div>
      )}
    </Card>
  );
}

function TopFolders({ summary }: { summary: ScanSummary }) {
  const t = useChartTheme();
  const setPage = useUiStore((s) => s.setPage);
  const { data, loading, error, reload } = useAsync(() => api.getTree(summary.scanId, null, 1, 10), [summary.scanId]);
  const dirs = (data?.children ?? []).filter((c) => c.isDir).slice(0, 10);
  return (
    <Card title="Biggest folders" subtitle="Click a bar to explore it">
      {loading && !data ? <Skeleton className="h-60 w-full" />
        : error ? <ErrorRow message={error} retry={reload} />
        : dirs.length === 0 ? <EmptyState icon={FolderTree} title="No folders" description="This location has no sub-folders." />
        : (
          <ReactECharts
            style={{ height: 260 }}
            notMerge
            onEvents={{ click: (p: { dataIndex: number }) => { const d = [...dirs].reverse()[p.dataIndex]; if (d) setPage("explorer", d.path); } }}
            option={{
              animationDuration: 800, animationEasing: "cubicOut",
              grid: { left: 8, right: 70, top: 4, bottom: 4, containLabel: true },
              tooltip: {
                trigger: "item", backgroundColor: t.tooltipBg, borderColor: t.tooltipBorder, textStyle: { color: t.textColor },
                formatter: (p: { dataIndex: number }) => { const d = [...dirs].reverse()[p.dataIndex]; return `${d.path}<br/><b>${formatBytes(d.sizeBytes)}</b>`; },
              },
              xAxis: { type: "value", show: false },
              yAxis: { type: "category", data: [...dirs].reverse().map((d) => d.name), axisLine: { show: false }, axisTick: { show: false }, axisLabel: { color: t.textColor, width: 110, overflow: "truncate" } },
              series: [{
                type: "bar", barWidth: 14, cursor: "pointer",
                itemStyle: { borderRadius: 7, color: t.accent },
                label: { show: true, position: "right", color: t.mutedColor, formatter: (p: { dataIndex: number }) => formatBytes([...dirs].reverse()[p.dataIndex].sizeBytes) },
                data: [...dirs].reverse().map((d) => d.sizeBytes),
              }],
            }}
          />
        )}
    </Card>
  );
}

function Reclaimable() {
  const setPage = useUiStore((s) => s.setPage);
  const { data, loading, error, reload } = useAsync(async () => {
    const cleaners = await api.listCleaners();
    const results = await Promise.allSettled(cleaners.map((c) => api.previewCleaner(c.id)));
    const ok = results.filter((r): r is PromiseFulfilledResult<Awaited<ReturnType<typeof api.previewCleaner>>> => r.status === "fulfilled");
    return { bytes: ok.reduce((n, r) => n + r.value.totalBytes, 0), sources: ok.filter((r) => r.value.totalBytes > 0).length };
  }, []);
  return (
    <Card title="Reclaimable" subtitle="Junk that can be cleaned safely"
      actions={<Button size="sm" variant="ghost" icon={ArrowUpRight} onClick={() => setPage("cleanup")}>Open Cleanup</Button>}>
      {loading && !data ? <Skeleton className="h-20 w-full" />
        : error ? <ErrorRow message={error} retry={reload} />
        : data && data.bytes > 0 ? (
          <button onClick={() => setPage("cleanup")} className="group flex w-full cursor-pointer items-center gap-4 text-left">
            <span className="flex h-12 w-12 items-center justify-center rounded-xl bg-accent-soft text-accent"><Sparkles size={22} /></span>
            <span>
              <span className="block text-4xl font-semibold tracking-tight text-accent"><AnimatedNumber value={data.bytes} format={formatBytes} /></span>
              <span className="text-sm text-muted">across {data.sources} {data.sources === 1 ? "source" : "sources"}</span>
            </span>
            <ArrowUpRight className="ml-auto text-faint transition-transform group-hover:-translate-y-0.5 group-hover:translate-x-0.5" size={18} />
          </button>
        ) : <EmptyState icon={Sparkles} title="Nothing to clean" description="No reclaimable junk was found." />}
    </Card>
  );
}

function Growth({ summary }: { summary: ScanSummary }) {
  const { data, loading, error, reload } = useAsync(async () => {
    if (summary.snapshotId == null) return null;
    const snaps = await api.listSnapshots();
    const prev = snaps.find((s) => s.root === summary.root && s.id < summary.snapshotId!);
    if (!prev) return null;
    return { prev, entries: await api.diffSnapshots(prev.id, summary.snapshotId, 5) };
  }, [summary.snapshotId, summary.root]);
  const max = Math.max(1, ...(data?.entries ?? []).map((e) => Math.abs(e.deltaBytes)));
  return (
    <Card title="Grew since last scan" subtitle="Top 5 changes in this location">
      {loading && !data ? <Skeleton className="h-28 w-full" />
        : error ? <ErrorRow message={error} retry={reload} />
        : !data ? <EmptyState icon={TrendingUp} title="No previous scan" description="Scan this location again later to see what grew." />
        : data.entries.length === 0 ? <EmptyState icon={TrendingUp} title="No changes" description="Nothing changed noticeably since the last scan." />
        : (
          <ul className="grid gap-3">
            {data.entries.map((e) => (
              <li key={e.path}>
                <div className="flex items-center gap-2 text-sm">
                  <span className="min-w-0 flex-1 truncate text-text" title={e.path}>{e.path}</span>
                  {e.isNew && <Badge tone="info">new</Badge>}
                  <span className={`tnum font-medium ${e.deltaBytes >= 0 ? "text-danger" : "text-success"}`}>{formatDelta(e.deltaBytes)}</span>
                </div>
                <div className="mt-1 h-1 overflow-hidden rounded-full bg-surface-3">
                  <motion.div className="h-full rounded-full" style={{ background: e.deltaBytes >= 0 ? "var(--danger)" : "var(--success)" }}
                    initial={{ width: 0 }} animate={{ width: `${(Math.abs(e.deltaBytes) / max) * 100}%` }} transition={{ duration: 0.7, ease: [0.16, 1, 0.3, 1] }} />
                </div>
              </li>
            ))}
          </ul>
        )}
    </Card>
  );
}

function DeletedOpen() {
  const setPage = useUiStore((s) => s.setPage);
  const { data } = useAsync(() => api.getDeletedOpenFiles(), []);
  if (!data || data.length === 0) return null;
  const bytes = data.reduce((n, f) => n + f.sizeBytes, 0);
  return (
    <motion.div variants={item} className="flex items-center gap-3 rounded-xl border border-warning bg-warning-soft p-4">
      <AlertTriangle className="shrink-0 text-warning" size={20} />
      <p className="flex-1 text-sm text-text">
        <b>{formatBytes(bytes)}</b> is held by {data.length} deleted {data.length === 1 ? "file" : "files"} still open in running programs
        ({[...new Set(data.map((f) => f.processName))].slice(0, 3).join(", ")}). Restart those programs to free it.
      </p>
      <Button size="sm" onClick={() => setPage("cleanup")}>Details</Button>
    </motion.div>
  );
}
