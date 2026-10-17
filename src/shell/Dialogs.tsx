// Reclaim: the About and Keyboard Shortcuts dialogs, driven by useUiStore.
import { useEffect, useState } from "react";
import { openUrl } from "@tauri-apps/plugin-opener";
import { ExternalLink, HardDrive } from "lucide-react";
import { Button, Dialog } from "../components";
import { api } from "../lib/api";
import type { AppInfo } from "../lib/types";
import { useUiStore } from "../store/ui";
import { SHORTCUTS } from "./nav";

const REPO = "https://github.com/BasithDev/reclaim";

export function AboutDialog() {
  const open = useUiStore((s) => s.aboutOpen);
  const close = () => useUiStore.getState().setAboutOpen(false);
  const [info, setInfo] = useState<AppInfo | null>(null);
  useEffect(() => { if (open && !info) api.getAppInfo().then(setInfo).catch(() => undefined); }, [open, info]);

  return (
    <Dialog open={open} onClose={close} title="About Reclaim" size="sm"
      footer={<Button icon={ExternalLink} onClick={() => openUrl(REPO).catch(() => window.open(REPO, "_blank"))}>View on GitHub</Button>}>
      <div className="flex items-center gap-3 pb-3">
        <div className="flex h-12 w-12 items-center justify-center rounded-xl bg-accent text-accent-contrast"><HardDrive size={24} /></div>
        <div>
          <div className="text-base font-semibold text-text">Reclaim</div>
          <div className="text-xs text-muted">Find what eats your disk, then get the space back.</div>
        </div>
      </div>
      <dl className="grid grid-cols-[auto_1fr] gap-x-6 gap-y-1.5 text-sm">
        <dt className="text-faint">Version</dt><dd className="text-text">{info?.version ?? "…"}</dd>
        <dt className="text-faint">System</dt><dd className="text-text">{info ? `${info.os} · ${info.arch}` : "…"}</dd>
        <dt className="text-faint">Home</dt><dd className="truncate text-text">{info?.homeDir ?? "…"}</dd>
      </dl>
    </Dialog>
  );
}

export function ShortcutsDialog() {
  const open = useUiStore((s) => s.shortcutsOpen);
  return (
    <Dialog open={open} onClose={() => useUiStore.getState().setShortcutsOpen(false)} title="Keyboard shortcuts" size="sm">
      <ul className="divide-y divide-border">
        {SHORTCUTS.map((s) => (
          <li key={s.keys} className="flex items-center justify-between py-2">
            <span className="text-text">{s.label}</span>
            <kbd className="rounded-md border border-border bg-surface-2 px-1.5 py-0.5 font-sans text-xs text-muted">{s.keys}</kbd>
          </li>
        ))}
      </ul>
    </Dialog>
  );
}
