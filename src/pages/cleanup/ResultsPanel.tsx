// Reclaim: results panel shown after a cleanup run: animated "Freed X" counter and per-cleaner outcomes,
// with raw errors summarised in plain language (details expandable).
import { useState } from "react";
import { motion } from "motion/react";
import { AlertCircle, AlertTriangle, CheckCircle2, ChevronRight, X } from "lucide-react";
import { AnimatedNumber, Card, IconButton, cn } from "../../components";
import { formatBytes } from "../../lib/format";
import type { CleanerInfo, CleanupResult } from "../../lib/types";

const PERMISSION = /permission denied|os error 13|PermissionDenied|access is denied|os error 5\b/i;
const IN_USE = /being used by another process|os error 32|resource busy|os error 16/i;
const ELEVATION = /administrator permission was (denied|cancelled)/i;

/** Turns raw backend errors into one short line each, grouped by cause. */
function summarise(errors: string[]): string[] {
  const perm = errors.filter((e) => PERMISSION.test(e)).length;
  const busy = errors.filter((e) => !PERMISSION.test(e) && IN_USE.test(e)).length;
  const elev = errors.filter((e) => ELEVATION.test(e)).length;
  const other = errors.length - perm - busy - elev;
  const n = (k: number, what: string) => `${k} ${k === 1 ? "item" : "items"} ${what}`;
  const out: string[] = [];
  if (perm) out.push(n(perm, "skipped: owned by the system or another user"));
  if (busy) out.push(n(busy, "skipped: in use by a running program"));
  if (elev) out.push("System items skipped: the administrator prompt was cancelled");
  if (other) out.push(n(other, "could not be removed"));
  return out;
}

function ResultRow({ r, name }: { r: CleanupResult; name: string }) {
  const [open, setOpen] = useState(false);
  const issues = r.errors.length > 0;
  const Icon = !r.success ? AlertCircle : issues ? AlertTriangle : CheckCircle2;
  const tone = !r.success ? "text-danger" : issues ? "text-warning" : "text-success";
  return (
    <li className="py-2.5">
      <div className="flex items-center gap-2">
        <Icon size={15} className={cn("shrink-0", tone)} />
        <span className="flex-1 text-text">{name}</span>
        <span className="text-xs text-muted tnum">{r.itemsRemoved.toLocaleString()} removed</span>
        <span className="w-20 text-right font-medium text-text tnum">{formatBytes(r.freedBytes)}</span>
      </div>
      {issues && (
        <div className="mt-1 pl-6 text-xs">
          {summarise(r.errors).map((line) => <div key={line} className="text-muted">{line}</div>)}
          <button onClick={() => setOpen((o) => !o)} className="mt-1 inline-flex cursor-pointer items-center gap-1 text-faint hover:text-muted">
            <ChevronRight size={12} className={cn("transition-transform", open && "rotate-90")} />{open ? "Hide details" : "Show details"}
          </button>
          {open && (
            <ul className="selectable mt-1 max-h-40 space-y-0.5 overflow-y-auto rounded-md bg-surface-2 p-2 font-mono text-[11px] text-muted">
              {r.errors.map((e, i) => <li key={i} className="break-all">{e}</li>)}
            </ul>
          )}
        </div>
      )}
    </li>
  );
}

export function ResultsPanel({ results, cleaners, onDismiss }: { results: CleanupResult[]; cleaners: CleanerInfo[]; onDismiss: () => void }) {
  const freed = results.reduce((s, r) => s + r.freedBytes, 0);
  const name = (id: string) => cleaners.find((c) => c.id === id)?.name ?? id;
  const anyIssue = results.some((r) => !r.success || r.errors.length > 0);
  return (
    <motion.div initial={{ opacity: 0, y: 8 }} animate={{ opacity: 1, y: 0 }} className="mb-6">
      <Card actions={<IconButton icon={X} label="Dismiss" size="sm" onClick={onDismiss} />}>
        <div className="flex items-center gap-4">
          <div className="flex h-12 w-12 items-center justify-center rounded-xl bg-accent-soft text-accent"><CheckCircle2 size={24} /></div>
          <div>
            <div className="text-xs text-muted">Space reclaimed</div>
            <div className="text-3xl font-semibold tracking-tight text-accent"><AnimatedNumber value={freed} format={formatBytes} duration={1.2} /></div>
            {anyIssue && <div className="mt-0.5 text-xs text-muted">Some items were skipped; see below.</div>}
          </div>
        </div>
        <ul className="mt-4 divide-y divide-border text-sm">
          {results.map((r) => <ResultRow key={r.cleanerId} r={r} name={name(r.cleanerId)} />)}
        </ul>
      </Card>
    </motion.div>
  );
}
