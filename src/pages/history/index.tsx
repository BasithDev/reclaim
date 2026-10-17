// Reclaim: History page: disk-usage-over-time chart, the snapshot list, and a growth comparison of two snapshots.
import { useCallback, useEffect, useMemo, useState } from "react";
import { AnimatePresence, motion } from "motion/react";
import { AlertCircle, History, Trash2 } from "lucide-react";
import { Badge, Button, Card, Checkbox, ConfirmDialog, EmptyState, IconButton, PageHeader, Skeleton, useToast } from "../../components";
import { api } from "../../lib/api";
import { formatBytes, formatCount, formatDate } from "../../lib/format";
import type { SnapshotMeta } from "../../lib/types";
import { useScanStore } from "../../store/scan";
import { useUiStore } from "../../store/ui";
import { GrowthList } from "./GrowthList";
import { UsageChart } from "./UsageChart";

/** Default comparison: the latest snapshot and the one before it for the same root. */
function defaultPair(list: SnapshotMeta[]): number[] {
  const sorted = [...list].sort((a, b) => b.takenAtMs - a.takenAtMs);
  const latest = sorted[0];
  const prev = latest && sorted.slice(1).find((s) => s.root === latest.root);
  return latest && prev ? [prev.id, latest.id] : [];
}

export default function HistoryPage() {
  const { toast } = useToast();
  const [snaps, setSnaps] = useState<SnapshotMeta[] | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [picked, setPicked] = useState<number[]>([]);
  const [toDelete, setToDelete] = useState<SnapshotMeta | null>(null);
  const [deleting, setDeleting] = useState(false);
  const scanRoot = useScanStore((s) => s.summary?.root ?? null);
  const setPage = useUiStore((s) => s.setPage);

  const load = useCallback(async () => {
    setError(null);
    try {
      const list = await api.listSnapshots();
      setSnaps(list);
      setPicked((p) => (p.length === 2 && p.every((id) => list.some((s) => s.id === id)) ? p : defaultPair(list)));
    } catch (e) { setError(String(e)); }
  }, []);
  useEffect(() => { void load(); }, [load]);

  const sorted = useMemo(() => [...(snaps ?? [])].sort((a, b) => b.takenAtMs - a.takenAtMs), [snaps]);
  const pair = useMemo(() => {
    if (picked.length !== 2 || !snaps) return null;
    const [a, b] = picked.map((id) => snaps.find((s) => s.id === id)!);
    return a && b ? (a.takenAtMs <= b.takenAtMs ? [a, b] : [b, a]) : null;
  }, [picked, snaps]);

  const toggle = (id: number, on: boolean) =>
    setPicked((p) => (on ? [...p.filter((x) => x !== id), id].slice(-2) : p.filter((x) => x !== id)));

  const remove = async () => {
    if (!toDelete) return;
    setDeleting(true);
    try {
      await api.deleteSnapshot(toDelete.id);
      toast({ title: "Snapshot deleted", tone: "success" });
      setPicked((p) => p.filter((x) => x !== toDelete.id));
      setToDelete(null);
      await load();
    } catch (e) { toast({ title: "Could not delete snapshot", description: String(e), tone: "error" }); }
    finally { setDeleting(false); }
  };

  if (error) return (<div><PageHeader title="History" /><Card><EmptyState icon={AlertCircle} title="Couldn't load history" description={error} action={<Button onClick={() => void load()}>Retry</Button>} /></Card></div>);
  if (!snaps) return (<div><PageHeader title="History" description="How your disk usage changed over time." /><div className="space-y-4"><Skeleton className="h-64 w-full" /><Skeleton className="h-56 w-full" /></div></div>);
  if (snaps.length === 0) return (
    <div><PageHeader title="History" description="How your disk usage changed over time." />
      <Card><EmptyState icon={History} title="No snapshots yet" description="Every scan saves a snapshot (if enabled in Settings), so you can see what grew over time." action={<Button variant="primary" onClick={() => setPage("dashboard")}>Go to Dashboard</Button>} /></Card></div>
  );

  return (
    <div>
      <PageHeader title="History" description="How your disk usage changed over time." />
      <div className="space-y-4">
        <Card title="Disk used over time" subtitle={`${snaps.length} snapshot${snaps.length === 1 ? "" : "s"}`}>
          {snaps.filter((s) => s.diskUsedBytes != null).length >= 2 ? <UsageChart snapshots={snaps} />
            : <EmptyState icon={History} title="Not enough data yet" description="Scan again later to see the trend." />}
        </Card>

        <Card padded={false} title="Snapshots" subtitle="Tick two to compare what grew between them" >
          <ul className="mt-3 divide-y divide-border border-t border-border">
            {sorted.map((s, i) => (
              <motion.li key={s.id} layout initial={{ opacity: 0 }} animate={{ opacity: 1 }} transition={{ delay: Math.min(i, 8) * 0.02 }}
                className="flex items-center gap-3 px-5 py-2.5">
                <Checkbox checked={picked.includes(s.id)} onChange={(v) => toggle(s.id, v)} />
                <div className="min-w-0 flex-1">
                  <div className="flex items-center gap-2 text-sm text-text">{formatDate(s.takenAtMs)}{i === 0 && <Badge tone="accent">latest</Badge>}</div>
                  <div className="truncate font-mono text-xs text-muted">{s.root}</div>
                </div>
                <div className="text-right text-xs text-muted tnum">
                  <div className="text-sm font-medium text-text">{formatBytes(s.totalBytes)}</div>
                  {formatCount(s.fileCount)} files
                </div>
                <IconButton icon={Trash2} label="Delete snapshot" size="sm" onClick={() => setToDelete(s)} />
              </motion.li>
            ))}
          </ul>
        </Card>

        <Card title="Growth" subtitle={pair ? `${formatDate(pair[0].takenAtMs)} → ${formatDate(pair[1].takenAtMs)}` : "Pick two snapshots above"}>
          <AnimatePresence mode="wait">
            <motion.div key={pair ? `${pair[0].id}-${pair[1].id}` : "none"} initial={{ opacity: 0 }} animate={{ opacity: 1 }} exit={{ opacity: 0 }} transition={{ duration: 0.15 }}>
              {pair ? (
                <GrowthList
                  olderId={pair[0].id} newerId={pair[1].id}
                  canOpen={(p) => !!scanRoot && p.startsWith(scanRoot)}
                  onOpen={(p) => setPage("explorer", p)}
                />
              ) : (
                <EmptyState icon={History} title="Nothing to compare" description="Tick two snapshots to see which folders grew or shrank." />
              )}
            </motion.div>
          </AnimatePresence>
        </Card>
      </div>

      <ConfirmDialog
        open={!!toDelete} onClose={() => !deleting && setToDelete(null)} onConfirm={() => void remove()} loading={deleting}
        title="Delete snapshot?" confirmLabel="Delete" tone="danger"
        message={<>The snapshot from <b className="text-text">{toDelete ? formatDate(toDelete.takenAtMs) : ""}</b> will be removed from History. Your files are not affected.</>}
      />
    </div>
  );
}
