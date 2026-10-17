// Reclaim: Large Files page. Filterable, sortable table of the biggest files with reveal/trash and bulk trash.
// Owned by A4; data from api.getLargestFiles for the latest finished scan.
import { useEffect, useMemo, useState } from "react";
import { AnimatePresence, motion } from "motion/react";
import { ArrowDown, ArrowUp, Eye, FileSearch, RefreshCw, Search, Trash2 } from "lucide-react";
import { revealItemInDir } from "@tauri-apps/plugin-opener";
import { api } from "../../lib/api";
import { formatBytes, formatRelative } from "../../lib/format";
import type { FileCategory, FileEntry } from "../../lib/types";
import {
  Button, Card, CategoryDot, Checkbox, Chip, ConfirmDialog, EmptyState, IconButton, PageHeader, Skeleton,
  Tooltip, categoryMeta, useToast,
} from "../../components";
import { useScanStore } from "../../store/scan";
import { useDeleteMode } from "../../store/settings";
import { useAsync } from "../dashboard/useAsync";

type SortKey = "size" | "modified";

export default function FilesPage() {
  const del = useDeleteMode();
  const status = useScanStore((s) => s.status);
  const summary = useScanStore((s) => s.summary);
  const startScan = useScanStore((s) => s.startScan);
  const scanId = summary?.scanId ?? null;
  const { toast } = useToast();

  const [category, setCategory] = useState<FileCategory | null>(null);
  const [query, setQuery] = useState("");
  const [sort, setSort] = useState<{ key: SortKey; desc: boolean }>({ key: "size", desc: true });
  const [selected, setSelected] = useState<Set<string>>(new Set());
  const [removed, setRemoved] = useState<Set<string>>(new Set());
  const [confirm, setConfirm] = useState<FileEntry[] | null>(null);
  const [busy, setBusy] = useState(false);

  // Load everything once (top 500) and filter client-side so chip totals stay stable.
  const { data, loading, error, reload } = useAsync(
    () => (scanId ? api.getLargestFiles(scanId, 500, null) : Promise.reject("no scan")),
    [scanId],
  );
  useEffect(() => { setSelected(new Set()); setRemoved(new Set()); }, [scanId]);

  const files = useMemo(() => (data ?? []).filter((f) => !removed.has(f.path)), [data, removed]);
  const totals = useMemo(() => {
    const m = new Map<FileCategory, { bytes: number; count: number }>();
    for (const f of files) {
      const t = m.get(f.category) ?? { bytes: 0, count: 0 };
      t.bytes += f.sizeBytes; t.count++;
      m.set(f.category, t);
    }
    return m;
  }, [files]);

  const rows = useMemo(() => {
    const q = query.trim().toLowerCase();
    const list = files.filter((f) => (!category || f.category === category) && (!q || f.path.toLowerCase().includes(q)));
    const dir = sort.desc ? -1 : 1;
    return list.sort((a, b) => dir * (sort.key === "size" ? a.sizeBytes - b.sizeBytes : (a.modifiedMs ?? 0) - (b.modifiedMs ?? 0)));
  }, [files, category, query, sort]);

  const selectedFiles = files.filter((f) => selected.has(f.path));
  const selectedBytes = selectedFiles.reduce((n, f) => n + f.sizeBytes, 0);
  const allSelected = rows.length > 0 && rows.every((r) => selected.has(r.path));

  const toggleSort = (key: SortKey) => setSort((s) => (s.key === key ? { key, desc: !s.desc } : { key, desc: true }));
  const toggle = (p: string, v: boolean) => setSelected((s) => { const n = new Set(s); if (v) n.add(p); else n.delete(p); return n; });

  const trash = async () => {
    if (!confirm) return;
    setBusy(true);
    let freed = 0; const done = new Set<string>(); const failed: string[] = [];
    for (const f of confirm) {
      try { freed += await api.removePath(f.path, del.permanent); done.add(f.path); }
      catch (e) { failed.push(`${f.name}: ${e}`); }
    }
    setRemoved((s) => new Set([...s, ...done]));
    setSelected((s) => new Set([...s].filter((p) => !done.has(p))));
    if (done.size) toast({ title: `${del.past} ${done.size} ${done.size === 1 ? "file" : "files"}`, description: `${formatBytes(freed)} freed`, tone: "success" });
    if (failed.length) toast({ title: `${failed.length} could not be removed`, description: failed[0], tone: "error" });
    setBusy(false); setConfirm(null);
  };

  const header = <PageHeader title="Large Files" description="The biggest files from your last scan." />;

  if (!scanId) {
    return (
      <div className="grid gap-6">
        {header}
        <Card>
          <EmptyState icon={FileSearch} title={status === "scanning" ? "Scan in progress" : "Nothing scanned yet"}
            description={status === "scanning" ? "Large files appear when the scan finishes." : "Run a scan to list your largest files."}
            action={status === "scanning" ? undefined : <Button variant="primary" onClick={() => void startScan()}>Scan now</Button>} />
        </Card>
      </div>
    );
  }

  const SortIcon = sort.desc ? ArrowDown : ArrowUp;
  const sortBtn = (key: SortKey, label: string) => (
    <button onClick={() => toggleSort(key)} className="inline-flex cursor-pointer items-center gap-1 hover:text-text">
      {label}{sort.key === key && <SortIcon size={12} />}
    </button>
  );

  return (
    <div className="grid gap-5 pb-20">
      {header}

      {error && !data ? (
        <Card>
          <EmptyState icon={FileSearch} title="Could not load files" description={error} action={<Button icon={RefreshCw} onClick={reload}>Retry</Button>} />
        </Card>
      ) : (
        <>
          <div className="flex flex-wrap items-center gap-2">
            {loading && !data ? Array.from({ length: 6 }).map((_, i) => <Skeleton key={i} className="h-7 w-24 rounded-full" />) : (
              <>
                <Chip active={category === null} count={files.length} onClick={() => setCategory(null)}>All</Chip>
                {[...totals.entries()].sort((a, b) => b[1].bytes - a[1].bytes).map(([c, t]) => (
                  <Chip key={c} active={category === c} color={categoryMeta[c].color} count={formatBytes(t.bytes)} onClick={() => setCategory(category === c ? null : c)}>
                    {categoryMeta[c].label}
                  </Chip>
                ))}
              </>
            )}
            <div className="relative ml-auto w-full sm:w-64">
              <Search size={14} className="pointer-events-none absolute left-3 top-1/2 -translate-y-1/2 text-faint" />
              <input value={query} onChange={(e) => setQuery(e.target.value)} placeholder="Search files…" aria-label="Search files"
                className="h-9 w-full rounded-lg border border-border bg-surface pl-8 pr-3 text-sm text-text outline-none transition-colors placeholder:text-faint focus:border-accent" />
            </div>
          </div>

          <Card padded={false} className="overflow-hidden">
            <div className="grid grid-cols-[2rem_minmax(0,1fr)_6rem_8rem_5rem] items-center gap-3 border-b border-border px-4 py-2.5 text-xs font-medium text-muted">
              <Checkbox checked={allSelected} indeterminate={!allSelected && rows.some((r) => selected.has(r.path))}
                onChange={(v) => setSelected(v ? new Set(rows.map((r) => r.path)) : new Set())} />
              <span>Name</span>
              <span className="text-right">{sortBtn("size", "Size")}</span>
              <span>{sortBtn("modified", "Modified")}</span>
              <span />
            </div>
            {loading && !data ? (
              <div className="grid gap-2 p-4">{Array.from({ length: 10 }).map((_, i) => <Skeleton key={i} className="h-10" />)}</div>
            ) : rows.length === 0 ? (
              <EmptyState icon={FileSearch} title="No matching files" description={query || category ? "Try a different filter or search." : "No files were found in this scan."}
                action={query || category ? <Button onClick={() => { setQuery(""); setCategory(null); }}>Clear filters</Button> : undefined} />
            ) : (
              <div className="max-h-[calc(100vh-22rem)] min-h-64 overflow-y-auto">
                {rows.slice(0, 300).map((f) => (
                  <div key={f.path} className="group grid grid-cols-[2rem_minmax(0,1fr)_6rem_8rem_5rem] items-center gap-3 border-b border-border/60 px-4 py-2 text-sm transition-colors last:border-0 hover:bg-surface-2">
                    <Checkbox checked={selected.has(f.path)} onChange={(v) => toggle(f.path, v)} />
                    <div className="flex min-w-0 items-center gap-2.5">
                      <CategoryDot category={f.category} />
                      <div className="min-w-0">
                        <div className="truncate text-text" title={f.path}>{f.name}</div>
                        <div className="truncate text-xs text-faint" title={f.path}>{f.path}</div>
                      </div>
                    </div>
                    <span className="tnum text-right font-medium text-text">{formatBytes(f.sizeBytes)}</span>
                    <span className="text-muted">{formatRelative(f.modifiedMs)}</span>
                    <span className="flex justify-end gap-0.5 opacity-0 transition-opacity focus-within:opacity-100 group-hover:opacity-100">
                      <Tooltip content="Reveal in file manager">
                        <IconButton icon={Eye} label="Reveal in file manager" size="sm"
                          onClick={() => void revealItemInDir(f.path).catch((e) => toast({ title: "Could not reveal", description: String(e), tone: "error" }))} />
                      </Tooltip>
                      <Tooltip content={del.verb}>
                        <IconButton icon={Trash2} label={del.verb} size="sm" onClick={() => setConfirm([f])} />
                      </Tooltip>
                    </span>
                  </div>
                ))}
                {rows.length > 300 && <p className="px-4 py-3 text-xs text-muted">Showing the first 300 of {rows.length}. Narrow the filter to see more.</p>}
              </div>
            )}
          </Card>
        </>
      )}

      <AnimatePresence>
        {selectedFiles.length > 0 && (
          <motion.div initial={{ y: 60, opacity: 0 }} animate={{ y: 0, opacity: 1 }} exit={{ y: 60, opacity: 0 }}
            transition={{ type: "spring", stiffness: 300, damping: 28 }}
            className="fixed bottom-5 left-1/2 z-30 flex -translate-x-1/2 items-center gap-4 rounded-xl border border-border-strong bg-surface px-5 py-3 shadow-lg">
            <span className="text-sm text-text"><b>{selectedFiles.length}</b> selected · <b className="tnum">{formatBytes(selectedBytes)}</b></span>
            <Button size="sm" variant="ghost" onClick={() => setSelected(new Set())}>Clear</Button>
            <Button size="sm" variant="danger" icon={Trash2} onClick={() => setConfirm(selectedFiles)}>{del.verb}</Button>
          </motion.div>
        )}
      </AnimatePresence>

      <ConfirmDialog open={!!confirm} onClose={() => setConfirm(null)} onConfirm={() => void trash()} loading={busy} tone="danger"
        title={del.question} confirmLabel={del.verb}
        message={confirm && (confirm.length === 1
          ? <><b>{confirm[0].name}</b> ({formatBytes(confirm[0].sizeBytes)}). {del.note}</>
          : <><b>{confirm.length} files</b> ({formatBytes(confirm.reduce((n, f) => n + f.sizeBytes, 0))}). {del.note}</>)} />
    </div>
  );
}
