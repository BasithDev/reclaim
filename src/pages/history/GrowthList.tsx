// Reclaim: growth list for two compared snapshots: diverging delta bars, +/- sizes, "new" badge, click to open in Explorer.
import { useCallback, useEffect, useState } from "react";
import { motion } from "motion/react";
import { AlertCircle, TrendingUp } from "lucide-react";
import { Badge, Button, EmptyState, Skeleton } from "../../components";
import { api } from "../../lib/api";
import { formatBytes, formatDelta } from "../../lib/format";
import type { GrowthEntry } from "../../lib/types";

export function GrowthList({ olderId, newerId, canOpen, onOpen }: {
  olderId: number; newerId: number; canOpen: (path: string) => boolean; onOpen: (path: string) => void;
}) {
  const [rows, setRows] = useState<GrowthEntry[] | null>(null);
  const [error, setError] = useState<string | null>(null);

  const load = useCallback(() => {
    setRows(null); setError(null);
    let live = true;
    api.diffSnapshots(olderId, newerId, 40).then((r) => live && setRows(r)).catch((e) => live && setError(String(e)));
    return () => { live = false; };
  }, [olderId, newerId]);
  useEffect(() => load(), [load]);

  if (error) return <EmptyState icon={AlertCircle} title="Couldn't compare snapshots" description={error} action={<Button onClick={load}>Retry</Button>} />;
  if (!rows) return <div className="space-y-2">{Array.from({ length: 5 }, (_, i) => <Skeleton key={i} className="h-9 w-full" />)}</div>;
  if (rows.length === 0) return <EmptyState icon={TrendingUp} title="No changes between these snapshots" description="Nothing grew or shrank enough to report." />;

  const max = Math.max(...rows.map((r) => Math.abs(r.deltaBytes)), 1);
  return (
    <ul className="divide-y divide-border">
      {rows.map((r, i) => {
        const grew = r.deltaBytes >= 0;
        const clickable = canOpen(r.path);
        return (
          <li key={r.path}>
            <button
              disabled={!clickable} onClick={() => onOpen(r.path)} title={clickable ? "Show in Explorer" : r.path}
              className="grid w-full grid-cols-[minmax(0,1fr)_minmax(0,220px)_88px] items-center gap-4 py-2.5 text-left enabled:cursor-pointer enabled:hover:bg-surface-2 disabled:cursor-default"
            >
              <div className="min-w-0">
                <div className="flex items-center gap-2"><span className="truncate font-mono text-xs text-text">{r.path}</span>{r.isNew && <Badge tone="info">new</Badge>}</div>
                <div className="text-[11px] text-faint tnum">{formatBytes(r.beforeBytes)} → {formatBytes(r.afterBytes)}</div>
              </div>
              <div className="relative h-2 rounded-full bg-surface-2">
                <div className="absolute inset-y-[-2px] left-1/2 w-px bg-border-strong" />
                <motion.div
                  className="absolute h-full rounded-full"
                  style={{ background: grew ? "var(--warning)" : "var(--accent)", [grew ? "left" : "right"]: "50%" }}
                  initial={{ width: 0 }} animate={{ width: `${(Math.abs(r.deltaBytes) / max) * 50}%` }} transition={{ delay: i * 0.02, duration: 0.5, ease: "easeOut" }}
                />
              </div>
              <div className="text-right text-sm font-medium tnum" style={{ color: grew ? "var(--warning)" : "var(--accent-strong)" }}>{formatDelta(r.deltaBytes)}</div>
            </button>
          </li>
        );
      })}
    </ul>
  );
}
