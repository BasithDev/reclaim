// Reclaim: results panel shown after a cleanup run: animated "Freed X" counter and per-cleaner outcomes.
import { motion } from "motion/react";
import { AlertCircle, CheckCircle2, X } from "lucide-react";
import { AnimatedNumber, Card, IconButton } from "../../components";
import { formatBytes } from "../../lib/format";
import type { CleanerInfo, CleanupResult } from "../../lib/types";

export function ResultsPanel({ results, cleaners, onDismiss }: { results: CleanupResult[]; cleaners: CleanerInfo[]; onDismiss: () => void }) {
  const freed = results.reduce((s, r) => s + r.freedBytes, 0);
  const name = (id: string) => cleaners.find((c) => c.id === id)?.name ?? id;
  return (
    <motion.div initial={{ opacity: 0, y: 8 }} animate={{ opacity: 1, y: 0 }} className="mb-6">
      <Card actions={<IconButton icon={X} label="Dismiss" size="sm" onClick={onDismiss} />}>
        <div className="flex items-center gap-4">
          <div className="flex h-12 w-12 items-center justify-center rounded-xl bg-accent-soft text-accent"><CheckCircle2 size={24} /></div>
          <div>
            <div className="text-xs text-muted">Space reclaimed</div>
            <div className="text-3xl font-semibold tracking-tight text-accent"><AnimatedNumber value={freed} format={formatBytes} duration={1.2} /></div>
          </div>
        </div>
        <ul className="mt-4 divide-y divide-border text-sm">
          {results.map((r) => (
            <li key={r.cleanerId} className="py-2">
              <div className="flex items-center gap-2">
                {r.success ? <CheckCircle2 size={14} className="text-success" /> : <AlertCircle size={14} className="text-danger" />}
                <span className="flex-1 text-text">{name(r.cleanerId)}</span>
                <span className="text-xs text-muted tnum">{r.itemsRemoved.toLocaleString()} items</span>
                <span className="w-20 text-right font-medium text-text tnum">{formatBytes(r.freedBytes)}</span>
              </div>
              {r.errors.length > 0 && (
                <ul className="selectable mt-1 space-y-0.5 pl-6 text-xs text-danger">
                  {r.errors.slice(0, 5).map((e, i) => <li key={i} className="break-words">{e}</li>)}
                  {r.errors.length > 5 && <li>and {r.errors.length - 5} more errors</li>}
                </ul>
              )}
            </li>
          ))}
        </ul>
      </Card>
    </motion.div>
  );
}
