// Reclaim: Dashboard drill-down for one file type (e.g. "Videos"): lists that type's largest files from the
// current scan, with search, reveal in the file manager, and select-to-delete (follows Settings › delete mode).
import { useMemo, useState } from "react";
import { FileSearch, FolderOpen, Search, Trash2 } from "lucide-react";
import { revealItemInDir } from "@tauri-apps/plugin-opener";
import { api } from "../../lib/api";
import { formatBytes, formatCount, formatRelative } from "../../lib/format";
import type { FileCategory, FileEntry } from "../../lib/types";
import {
  Button, Checkbox, ConfirmDialog, Dialog, EmptyState, IconButton, Skeleton, Tooltip, categoryMeta, useToast,
} from "../../components";
import { useDeleteMode } from "../../store/settings";
import { useAsync } from "./useAsync";

/** The engine keeps the largest 500 files per type, which covers what matters for cleanup. */
const LIMIT = 500;

function parentDir(path: string, name: string): string {
  return path.endsWith(name) ? path.slice(0, path.length - name.length).replace(/[\\/]$/, "") : path;
}

export function CategoryFilesDialog({ scanId, category, onClose }: { scanId: string; category: FileCategory | null; onClose: () => void }) {
  const { toast } = useToast();
  const del = useDeleteMode();
  const { data, loading, error, reload } = useAsync(
    () => (category ? api.getLargestFiles(scanId, LIMIT, category) : Promise.resolve([] as FileEntry[])),
    [scanId, category],
  );
  const [query, setQuery] = useState("");
  const [selected, setSelected] = useState<Set<string>>(new Set());
  const [removed, setRemoved] = useState<Set<string>>(new Set());
  const [confirm, setConfirm] = useState<FileEntry[] | null>(null);
  const [busy, setBusy] = useState(false);

  const rows = useMemo(() => {
    const q = query.trim().toLowerCase();
    return (data ?? []).filter((f) => !removed.has(f.path) && (!q || f.path.toLowerCase().includes(q)));
  }, [data, removed, query]);
  const chosen = rows.filter((f) => selected.has(f.path));
  const chosenBytes = chosen.reduce((n, f) => n + f.sizeBytes, 0);
  const shownBytes = rows.reduce((n, f) => n + f.sizeBytes, 0);
  const allOn = rows.length > 0 && rows.every((f) => selected.has(f.path));

  const close = () => { if (!busy) { setQuery(""); setSelected(new Set()); setRemoved(new Set()); onClose(); } };
  const toggle = (p: string, on: boolean) => setSelected((s) => { const n = new Set(s); if (on) n.add(p); else n.delete(p); return n; });

  const remove = async () => {
    if (!confirm) return;
    setBusy(true);
    let freed = 0; const done = new Set<string>(); const failed: string[] = [];
    for (const f of confirm) {
      try { freed += await api.removePath(f.path, del.permanent); done.add(f.path); }
      catch (e) { failed.push(`${f.name}: ${e}`); }
    }
    setRemoved((s) => new Set([...s, ...done]));
    setSelected((s) => new Set([...s].filter((p) => !done.has(p))));
    setBusy(false);
    setConfirm(null);
    if (done.size) toast({ title: `${del.past} ${done.size} ${done.size === 1 ? "file" : "files"}`, description: `${formatBytes(freed)} freed`, tone: "success" });
    if (failed.length) toast({ title: `${failed.length} could not be removed`, description: failed[0], tone: "error" });
  };

  const meta = category ? categoryMeta[category] : null;
  return (
    <>
      <Dialog
        open={category != null} onClose={close} size="xl"
        title={meta ? `${meta.label} in this scan` : ""}
        footer={
          <div className="flex w-full items-center gap-3">
            <div className="min-w-0 flex-1 text-xs text-muted tnum">
              {chosen.length > 0 ? <><b className="text-text">{chosen.length}</b> selected · <b className="text-text">{formatBytes(chosenBytes)}</b></> : "Select files to remove them"}
            </div>
            <Button variant="ghost" onClick={close}>Close</Button>
            <Button variant="danger" icon={Trash2} disabled={chosen.length === 0} onClick={() => setConfirm(chosen)}>{del.verb}</Button>
          </div>
        }
      >
        {loading && !data ? (
          <div className="space-y-2 py-2">{Array.from({ length: 8 }, (_, i) => <Skeleton key={i} className="h-9 w-full" />)}</div>
        ) : error ? (
          <EmptyState icon={FileSearch} title="Couldn't load files" description={error} action={<Button onClick={reload}>Retry</Button>} />
        ) : (
          <div className="py-1">
            <div className="sticky top-0 z-10 -mx-5 flex items-center gap-3 border-b border-border bg-surface px-5 pb-3 pt-1">
              <Checkbox checked={allOn} indeterminate={!allOn && chosen.length > 0}
                onChange={(on) => setSelected(on ? new Set(rows.map((f) => f.path)) : new Set())} />
              <div className="relative flex-1">
                <Search size={14} className="pointer-events-none absolute left-2.5 top-1/2 -translate-y-1/2 text-faint" />
                <input
                  value={query} onChange={(e) => setQuery(e.target.value)} placeholder="Filter by name or folder…"
                  className="h-8 w-full rounded-lg border border-border bg-surface-2 pl-8 pr-3 text-sm text-text outline-none placeholder:text-faint focus:border-accent"
                />
              </div>
              <span className="shrink-0 text-xs text-muted tnum">{formatCount(rows.length)} files · {formatBytes(shownBytes)}</span>
            </div>
            {rows.length === 0 ? (
              <EmptyState icon={FileSearch} title={query ? "No matches" : "No files of this type"} description={query ? "Try a different filter." : "Nothing left to show."} />
            ) : (
              <ul className="divide-y divide-border">
                {rows.map((f) => (
                  <li key={f.path} className="flex items-center gap-3 py-2">
                    <Checkbox checked={selected.has(f.path)} onChange={(on) => toggle(f.path, on)} />
                    <div className="min-w-0 flex-1">
                      <div className="truncate text-sm font-medium text-text" title={f.name}>{f.name}</div>
                      <div className="truncate text-xs text-faint" title={f.path}>{parentDir(f.path, f.name)}</div>
                    </div>
                    <span className="hidden w-24 shrink-0 text-right text-xs text-faint sm:block">{formatRelative(f.modifiedMs)}</span>
                    <span className="w-20 shrink-0 text-right text-sm font-medium text-text tnum">{formatBytes(f.sizeBytes)}</span>
                    <Tooltip content="Show in folder">
                      <IconButton icon={FolderOpen} label="Show in folder" size="sm" onClick={() => void revealItemInDir(f.path).catch(() => undefined)} />
                    </Tooltip>
                    <Tooltip content={del.verb}>
                      <IconButton icon={Trash2} label={del.verb} size="sm" onClick={() => setConfirm([f])} />
                    </Tooltip>
                  </li>
                ))}
              </ul>
            )}
            {data && data.length >= LIMIT && (
              <p className="py-3 text-center text-xs text-faint">Showing the {LIMIT} largest files of this type.</p>
            )}
          </div>
        )}
      </Dialog>
      <ConfirmDialog
        open={confirm != null} onClose={() => !busy && setConfirm(null)} onConfirm={() => void remove()} loading={busy}
        title={del.question} confirmLabel={del.verb} tone="danger"
        message={confirm && (confirm.length === 1
          ? <><b>{confirm[0].name}</b> ({formatBytes(confirm[0].sizeBytes)}). {del.note}</>
          : <><b>{confirm.length} files</b> ({formatBytes(confirm.reduce((n, f) => n + f.sizeBytes, 0))}). {del.note}</>)}
      />
    </>
  );
}
