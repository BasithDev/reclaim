// Reclaim: Cleanup page. Lists cleaners by group with lazily previewed sizes, a sticky action footer,
// a confirmation dialog and a results panel.
import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { AnimatePresence, motion } from "motion/react";
import { AlertCircle, RefreshCw, ShieldAlert, Sparkles } from "lucide-react";
import { AnimatedNumber, Button, Card, ConfirmDialog, EmptyState, PageHeader, Skeleton, useToast } from "../../components";
import { api } from "../../lib/api";
import { formatBytes } from "../../lib/format";
import type { CleanerGroup, CleanerInfo, CleanupResult } from "../../lib/types";
import { useSettingsStore } from "../../store/settings";
import { CleanerRow, type PreviewState } from "./CleanerRow";
import { ResultsPanel } from "./ResultsPanel";

const GROUPS: { id: CleanerGroup; label: string; blurb: string }[] = [
  { id: "system", label: "System", blurb: "Logs, caches and leftovers from the operating system" },
  { id: "developer", label: "Developer", blurb: "Build output and package caches" },
  { id: "applications", label: "Applications", blurb: "Browsers, editors and other apps" },
  { id: "user", label: "User", blurb: "Your own files and folders" },
];
const CONCURRENCY = 4;

export default function CleanupPage() {
  const { toast } = useToast();
  const confirmBefore = useSettingsStore((s) => s.settings.confirmBeforeClean);
  const [cleaners, setCleaners] = useState<CleanerInfo[] | null>(null);
  const [loadError, setLoadError] = useState<string | null>(null);
  const [previews, setPreviews] = useState<Record<string, PreviewState>>({});
  const [selected, setSelected] = useState<Set<string>>(new Set());
  const [expanded, setExpanded] = useState<Set<string>>(new Set());
  const [confirmOpen, setConfirmOpen] = useState(false);
  const [running, setRunning] = useState(false);
  const [cleaningIds, setCleaningIds] = useState<Set<string>>(new Set());
  const [results, setResults] = useState<CleanupResult[] | null>(null);
  const alive = useRef(true);
  useEffect(() => { alive.current = true; return () => { alive.current = false; }; }, []);

  const loadPreview = useCallback(async (id: string) => {
    setPreviews((p) => ({ ...p, [id]: { status: "loading" } }));
    try {
      const preview = await api.previewCleaner(id);
      if (alive.current) setPreviews((p) => ({ ...p, [id]: { status: "ok", preview } }));
    } catch (e) {
      if (alive.current) setPreviews((p) => ({ ...p, [id]: { status: "error", message: String(e) } }));
    }
  }, []);

  const loadAll = useCallback(async () => {
    setLoadError(null); setCleaners(null); setPreviews({});
    try {
      const list = await api.listCleaners();
      if (!alive.current) return;
      setCleaners(list);
      const queue = [...list.map((c) => c.id)];
      setPreviews(Object.fromEntries(queue.map((id) => [id, { status: "loading" } as PreviewState])));
      const worker = async () => {
        for (let id = queue.shift(); id && alive.current; id = queue.shift()) {
          try {
            const preview = await api.previewCleaner(id);
            if (alive.current) setPreviews((p) => ({ ...p, [id!]: { status: "ok", preview } }));
          } catch (e) {
            if (alive.current) setPreviews((p) => ({ ...p, [id!]: { status: "error", message: String(e) } }));
          }
        }
      };
      void Promise.all(Array.from({ length: CONCURRENCY }, worker));
    } catch (e) {
      if (alive.current) setLoadError(String(e));
    }
  }, []);
  useEffect(() => { void loadAll(); }, [loadAll]);

  const selectedList = useMemo(() => (cleaners ?? []).filter((c) => selected.has(c.id)), [cleaners, selected]);
  const selectedBytes = selectedList.reduce((s, c) => {
    const p = previews[c.id];
    return s + (p?.status === "ok" ? p.preview.totalBytes : 0);
  }, 0);
  const anyAdmin = selectedList.some((c) => c.requiresAdmin);
  const anyCommand = selectedList.some((c) => c.method === "command");

  const flip = (set: Set<string>, id: string, on?: boolean) => {
    const n = new Set(set);
    if (on ?? !n.has(id)) n.add(id); else n.delete(id);
    return n;
  };

  const run = async () => {
    const ids = selectedList.map((c) => c.id);
    // Close the dialog straight away so the rows themselves show progress while removal runs.
    setConfirmOpen(false);
    setResults(null);
    setRunning(true);
    setCleaningIds(new Set(ids));
    try {
      // Junk is always deleted permanently: sending caches to the Trash frees nothing.
      const res = await api.runCleaners(ids, true);
      setResults(res);
      setSelected(new Set());
      const freed = res.reduce((s, r) => s + r.freedBytes, 0);
      const withIssues = res.filter((r) => !r.success || r.errors.length > 0).length;
      toast({ title: `Freed ${formatBytes(freed)}`, description: withIssues ? `${withIssues} of ${res.length} cleaners skipped some items` : `All ${res.length} cleaners finished`, tone: withIssues ? "info" : "success" });
      ids.forEach((id) => void loadPreview(id));
      window.scrollTo?.({ top: 0 });
    } catch (e) {
      toast({ title: "Cleanup failed", description: String(e), tone: "error" });
    } finally {
      setRunning(false);
      setCleaningIds(new Set());
    }
  };

  const headerActions = <Button icon={RefreshCw} onClick={() => void loadAll()} disabled={running}>Rescan</Button>;

  return (
    <div className="pb-20">
      <PageHeader title="Cleanup" description="Reclaim space from caches, logs and leftovers. Nothing is removed until you confirm." actions={headerActions} />

      <AnimatePresence>{results && <ResultsPanel results={results} cleaners={cleaners ?? []} onDismiss={() => setResults(null)} />}</AnimatePresence>

      {loadError ? (
        <Card><EmptyState icon={AlertCircle} title="Couldn't load cleaners" description={loadError} action={<Button onClick={() => void loadAll()}>Retry</Button>} /></Card>
      ) : !cleaners ? (
        <div className="space-y-4">{[0, 1].map((i) => <Skeleton key={i} className="h-48 w-full" />)}</div>
      ) : cleaners.length === 0 ? (
        <Card><EmptyState icon={Sparkles} title="Nothing to clean on this system" description="No cleaners are available for your platform." /></Card>
      ) : (
        <div className="space-y-5">
          {GROUPS.map((g) => {
            const list = cleaners.filter((c) => c.group === g.id);
            if (list.length === 0) return null;
            const allOn = list.every((c) => selected.has(c.id));
            const total = list.reduce((s, c) => { const p = previews[c.id]; return s + (p?.status === "ok" ? p.preview.totalBytes : 0); }, 0);
            return (
              <Card key={g.id} padded={false} title={g.label} subtitle={g.blurb}
                actions={<>
                  <span className="mr-2 text-xs text-muted tnum">{formatBytes(total)}</span>
                  <Button size="sm" variant="ghost" onClick={() => setSelected((s) => {
                    const n = new Set(s);
                    list.forEach((c) => { const p = previews[c.id]; const has = p?.status === "ok" && (p.preview.totalBytes > 0 || p.preview.itemCount > 0); if (!allOn && has) n.add(c.id); else if (allOn) n.delete(c.id); });
                    return n;
                  })}>{allOn ? "Clear" : "Select all"}</Button>
                </>}>
                <div className="mt-3 border-t border-border">
                  {list.map((c) => (
                    <CleanerRow
                      key={c.id} info={c} state={previews[c.id]} selected={selected.has(c.id)} expanded={expanded.has(c.id)} cleaning={cleaningIds.has(c.id)}
                      onToggle={(v) => setSelected((s) => flip(s, c.id, v))} onExpand={() => setExpanded((s) => flip(s, c.id))} onRetry={() => void loadPreview(c.id)}
                    />
                  ))}
                </div>
              </Card>
            );
          })}
        </div>
      )}

      <AnimatePresence>
        {(selectedList.length > 0 || running) && (
          <motion.div
            initial={{ y: 40, opacity: 0 }} animate={{ y: 0, opacity: 1 }} exit={{ y: 40, opacity: 0 }} transition={{ type: "spring", stiffness: 400, damping: 34 }}
            className="sticky bottom-0 z-10 -mb-4 mt-6"
          >
            <div className="flex items-center gap-4 rounded-xl border border-border bg-surface px-5 py-3 shadow-lg">
              <div className="min-w-0 flex-1">
                <div className="text-sm font-semibold text-text"><AnimatedNumber value={selectedBytes} format={formatBytes} /> selected</div>
                <div className="text-xs text-muted">{running ? "Cleaning… keep Reclaim open" : `${selectedList.length} cleaner${selectedList.length === 1 ? "" : "s"} · deleted permanently`}</div>
              </div>
              <Button variant="ghost" disabled={running} onClick={() => setSelected(new Set())}>Clear</Button>
              <Button variant="primary" icon={Sparkles} loading={running} onClick={() => (confirmBefore ? setConfirmOpen(true) : void run())}>Clean selected</Button>
            </div>
          </motion.div>
        )}
      </AnimatePresence>

      <ConfirmDialog
        open={confirmOpen} onClose={() => !running && setConfirmOpen(false)} onConfirm={() => void run()} loading={running}
        title="Clean selected items?" confirmLabel={`Clean ${formatBytes(selectedBytes)}`} tone="danger"
        message={
          <div className="space-y-3">
            <ul className="space-y-1">
              {selectedList.map((c) => {
                const p = previews[c.id];
                return <li key={c.id} className="flex justify-between gap-4"><span className="text-text">{c.name}</span><span className="tnum">{p?.status === "ok" ? formatBytes(p.preview.totalBytes) : "-"}</span></li>;
              })}
            </ul>
            <p>These are caches, logs and leftovers, so they are deleted permanently (not moved to the Trash). Apps recreate caches when they need them.</p>
            {anyCommand && <p>Some cleaners run system commands that remove data directly.</p>}
            {anyAdmin && <p className="flex items-start gap-2"><ShieldAlert size={15} className="mt-0.5 shrink-0 text-warning" />Administrator permission will be requested for some of these.</p>}
          </div>
        }
      />
    </div>
  );
}
