// Reclaim: one expandable cleaner row (checkbox, name, risk, admin shield, lazily loaded size, preview items).
import { AnimatePresence, motion } from "motion/react";
import { AlertCircle, ChevronRight, ShieldAlert, Terminal } from "lucide-react";
import { Badge, Checkbox, IconButton, Skeleton, Tooltip, cn } from "../../components";
import { formatBytes, formatRelative } from "../../lib/format";
import type { CleanerInfo, CleanupPreview, Risk } from "../../lib/types";

export type PreviewState = { status: "loading" } | { status: "error"; message: string } | { status: "ok"; preview: CleanupPreview };

const riskTone: Record<Risk, "success" | "warning" | "danger"> = { safe: "success", moderate: "warning", careful: "danger" };
const MAX_ITEMS = 50;

export function CleanerRow({ info, state, selected, expanded, onToggle, onExpand, onRetry }: {
  info: CleanerInfo; state: PreviewState | undefined; selected: boolean; expanded: boolean;
  onToggle: (v: boolean) => void; onExpand: () => void; onRetry: () => void;
}) {
  const ok = state?.status === "ok" ? state.preview : null;
  const empty = ok != null && ok.totalBytes === 0 && ok.itemCount === 0 && !ok.command;
  return (
    <div className={cn("border-b border-border last:border-0", empty && "opacity-60")}>
      <div className="flex items-center gap-3 px-5 py-3">
        <Checkbox checked={selected} onChange={onToggle} />
        <button onClick={onExpand} className="flex min-w-0 flex-1 cursor-pointer items-center gap-2 text-left" aria-expanded={expanded}>
          <ChevronRight size={14} className={cn("shrink-0 text-faint transition-transform duration-200", expanded && "rotate-90")} />
          <div className="min-w-0 flex-1">
            <div className="flex items-center gap-2">
              <span className="truncate text-sm font-medium text-text">{info.name}</span>
              <Badge tone={riskTone[info.risk]}>{info.risk}</Badge>
              {info.requiresAdmin && <Tooltip content="Needs administrator permission"><span className="text-faint"><ShieldAlert size={14} /></span></Tooltip>}
              {info.method === "command" && <Tooltip content="Runs a command"><span className="text-faint"><Terminal size={14} /></span></Tooltip>}
            </div>
            <div className="truncate text-xs text-muted">{info.description}</div>
          </div>
        </button>
        <div className="w-24 shrink-0 text-right text-sm font-medium text-text tnum">
          {!state || state.status === "loading" ? <Skeleton className="ml-auto h-4 w-16" />
            : state.status === "error" ? (
              <Tooltip content={state.message}><button onClick={onRetry} className="inline-flex cursor-pointer items-center gap-1 text-xs text-danger"><AlertCircle size={13} />Retry</button></Tooltip>
            ) : empty ? <span className="text-xs font-normal text-faint">Nothing to clean</span> : formatBytes(state.preview.totalBytes)}
        </div>
      </div>
      <AnimatePresence initial={false}>
        {expanded && (
          <motion.div initial={{ height: 0, opacity: 0 }} animate={{ height: "auto", opacity: 1 }} exit={{ height: 0, opacity: 0 }} transition={{ duration: 0.2 }} className="overflow-hidden">
            <div className="mx-5 mb-3 ml-14 rounded-lg bg-surface-2 p-3 text-xs">
              {!state || state.status === "loading" ? <Skeleton className="h-4 w-1/2" />
                : state.status === "error" ? <div className="flex items-center gap-2 text-danger">{state.message}<IconButton icon={AlertCircle} label="Retry" size="sm" onClick={onRetry} /></div>
                : (
                  <div className="space-y-2">
                    {state.preview.command && (
                      <div><div className="mb-1 text-faint">Command</div><code className="selectable block break-all rounded-md bg-surface px-2 py-1.5 font-mono text-text">{state.preview.command}</code></div>
                    )}
                    {state.preview.items.length > 0 && (
                      <div>
                        <div className="mb-1 text-faint">{state.preview.itemCount.toLocaleString()} item{state.preview.itemCount === 1 ? "" : "s"}</div>
                        <ul className="max-h-56 space-y-0.5 overflow-y-auto">
                          {state.preview.items.slice(0, MAX_ITEMS).map((it) => (
                            <li key={it.path} className="flex items-center gap-3 py-0.5">
                              <span className="selectable min-w-0 flex-1 truncate font-mono text-text" title={it.path}>{it.label ?? it.path}</span>
                              {it.modifiedMs != null && <span className="shrink-0 text-faint">{formatRelative(it.modifiedMs)}</span>}
                              <span className="w-16 shrink-0 text-right text-muted tnum">{formatBytes(it.sizeBytes)}</span>
                            </li>
                          ))}
                        </ul>
                        {state.preview.itemCount > MAX_ITEMS && <div className="mt-1 text-faint">and {(state.preview.itemCount - MAX_ITEMS).toLocaleString()} more…</div>}
                      </div>
                    )}
                    {!state.preview.command && state.preview.items.length === 0 && <div className="text-muted">Nothing found to clean.</div>}
                  </div>
                )}
            </div>
          </motion.div>
        )}
      </AnimatePresence>
    </div>
  );
}
