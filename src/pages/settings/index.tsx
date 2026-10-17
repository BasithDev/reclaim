// Reclaim: Settings page (appearance, scanning, cleanup, history, about). Persists through useSettingsStore.
import { useEffect, useState, type ReactNode } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import { openPath, openUrl } from "@tauri-apps/plugin-opener";
import { appDataDir } from "@tauri-apps/api/path";
import { AlertCircle, ExternalLink, FolderOpen, Monitor, Moon, Plus, Sun, X } from "lucide-react";
import { motion, AnimatePresence } from "motion/react";
import { Button, Card, IconButton, PageHeader, SegmentedControl, Skeleton, Switch, useToast } from "../../components";
import { api } from "../../lib/api";
import type { AppInfo, Settings, ThemePreference } from "../../lib/types";
import { useSettingsStore } from "../../store/settings";

function Row({ title, description, children }: { title: string; description?: string; children: ReactNode }) {
  return (
    <div className="flex items-center justify-between gap-6 border-b border-border py-3.5 first:pt-0 last:border-0 last:pb-0">
      <div className="min-w-0">
        <div className="text-sm font-medium text-text">{title}</div>
        {description && <div className="mt-0.5 text-xs text-muted">{description}</div>}
      </div>
      <div className="shrink-0">{children}</div>
    </div>
  );
}

export default function SettingsPage() {
  const { settings, loaded, error, load, update } = useSettingsStore();
  const { toast } = useToast();
  const [info, setInfo] = useState<AppInfo | null>(null);

  useEffect(() => { if (!loaded) void load(); }, [loaded, load]);
  useEffect(() => { api.getAppInfo().then(setInfo).catch(() => undefined); }, []);

  const save = (p: Partial<Settings>) =>
    update(p).catch((e) => toast({ title: "Could not save settings", description: String(e), tone: "error" }));

  const pickFolder = async (): Promise<string | null> => {
    try {
      const d = await open({ directory: true, multiple: false });
      return typeof d === "string" ? d : null;
    } catch { return null; }
  };

  if (!loaded) {
    return (
      <div>
        <PageHeader title="Settings" description="Preferences for Reclaim." />
        <div className="space-y-4">{[0, 1, 2].map((i) => <Skeleton key={i} className="h-32 w-full" />)}</div>
      </div>
    );
  }

  return (
    <div>
      <PageHeader title="Settings" description="Preferences for Reclaim. Changes are saved automatically." />
      {error && (
        <div className="mb-4 flex items-center gap-3 rounded-lg border border-border bg-danger-soft px-4 py-3 text-sm text-danger">
          <AlertCircle size={16} /><span className="min-w-0 flex-1 truncate">{error}</span>
          <Button size="sm" onClick={() => void load()}>Retry</Button>
        </div>
      )}
      <div className="max-w-3xl space-y-4">
        <Card title="Appearance">
          <Row title="Theme" description="Follow your system or pick one.">
            <SegmentedControl
              value={settings.theme} onChange={(v) => void save({ theme: v as ThemePreference })}
              options={[{ value: "system", label: "System", icon: Monitor }, { value: "light", label: "Light", icon: Sun }, { value: "dark", label: "Dark", icon: Moon }]}
            />
          </Row>
        </Card>

        <Card title="Scanning">
          <Row title="Default scan folder" description={settings.defaultScanRoot ?? "Your home folder"}>
            <div className="flex gap-2">
              <Button size="sm" icon={FolderOpen} onClick={async () => { const d = await pickFolder(); if (d) void save({ defaultScanRoot: d }); }}>Choose…</Button>
              {settings.defaultScanRoot && <Button size="sm" variant="ghost" onClick={() => void save({ defaultScanRoot: null })}>Reset</Button>}
            </div>
          </Row>
          <Row title="Cross filesystems" description="Also scan other drives mounted inside the scanned folder.">
            <Switch checked={settings.crossFilesystems} onChange={(v) => void save({ crossFilesystems: v })} />
          </Row>
          <div className="pt-3.5">
            <div className="flex items-center justify-between">
              <div>
                <div className="text-sm font-medium text-text">Excluded paths</div>
                <div className="mt-0.5 text-xs text-muted">Folders that scans skip entirely.</div>
              </div>
              <Button size="sm" icon={Plus} onClick={async () => {
                const d = await pickFolder();
                if (d && !settings.excludedPaths.includes(d)) void save({ excludedPaths: [...settings.excludedPaths, d] });
              }}>Add folder…</Button>
            </div>
            <ul className="mt-3 space-y-1.5">
              <AnimatePresence initial={false}>
                {settings.excludedPaths.map((p) => (
                  <motion.li key={p} layout initial={{ opacity: 0, height: 0 }} animate={{ opacity: 1, height: "auto" }} exit={{ opacity: 0, height: 0 }}
                    className="flex items-center gap-2 overflow-hidden rounded-md bg-surface-2 py-1 pl-3 pr-1 text-xs">
                    <span className="selectable min-w-0 flex-1 truncate font-mono text-text">{p}</span>
                    <IconButton icon={X} label="Remove" size="sm" onClick={() => void save({ excludedPaths: settings.excludedPaths.filter((x) => x !== p) })} />
                  </motion.li>
                ))}
              </AnimatePresence>
            </ul>
            {settings.excludedPaths.length === 0 && <p className="mt-3 text-xs text-faint">No excluded paths.</p>}
          </div>
        </Card>

        <Card title="Cleanup">
          <Row title="Delete mode" description={settings.permanentDelete ? "Files are deleted permanently and cannot be restored." : "Files go to the Trash / Recycle Bin and can be restored."}>
            <SegmentedControl
              value={settings.permanentDelete ? "permanent" : "trash"} onChange={(v) => void save({ permanentDelete: v === "permanent" })}
              options={[{ value: "trash", label: "Trash" }, { value: "permanent", label: "Permanent" }]}
            />
          </Row>
          <Row title="Confirm before cleaning" description="Show a summary and ask before anything is removed.">
            <Switch checked={settings.confirmBeforeClean} onChange={(v) => void save({ confirmBeforeClean: v })} />
          </Row>
        </Card>

        <Card title="History">
          <Row title="Save snapshots" description="Keep a record of every scan so History can show what grew.">
            <Switch checked={settings.saveSnapshots} onChange={(v) => void save({ saveSnapshots: v })} />
          </Row>
          <Row title="Data folder" description="Where snapshots and settings are stored.">
            <Button size="sm" icon={FolderOpen} onClick={async () => {
              try { await openPath(await appDataDir()); } catch (e) { toast({ title: "Could not open folder", description: String(e), tone: "error" }); }
            }}>Open</Button>
          </Row>
        </Card>

        <Card title="About">
          <Row title="Reclaim" description={info ? `Version ${info.version} · ${info.os} ${info.arch}` : "Loading…"}>
            <Button size="sm" icon={ExternalLink} onClick={() => { const u = "https://github.com/BasithDev/reclaim"; openUrl(u).catch(() => window.open(u, "_blank")); }}>GitHub</Button>
          </Row>
        </Card>
      </div>
    </div>
  );
}
