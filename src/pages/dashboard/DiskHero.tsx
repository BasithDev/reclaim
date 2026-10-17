// Reclaim: Dashboard hero. Big ring gauge for the system disk, scan buttons, and cards for other disks.
import { motion } from "motion/react";
import { FolderSearch, HardDrive, Play, RefreshCw, Usb } from "lucide-react";
import { open } from "@tauri-apps/plugin-dialog";
import { api } from "../../lib/api";
import { formatBytes, formatPercent } from "../../lib/format";
import type { DiskInfo } from "../../lib/types";
import { AnimatedNumber, Badge, Button, Card, EmptyState, ProgressBar, RingGauge, Skeleton } from "../../components";
import { useScanStore } from "../../store/scan";
import { useAsync } from "./useAsync";

function tone(frac: number) {
  return frac > 0.9 ? "danger" : frac > 0.75 ? "warning" : "default";
}

export default function DiskHero() {
  const { data: disks, loading, error, reload } = useAsync(() => api.listDisks(), []);
  const status = useScanStore((s) => s.status);
  const startScan = useScanStore((s) => s.startScan);
  const scanning = status === "scanning";

  const pick = async () => {
    const dir = await open({ directory: true, multiple: false, title: "Choose a folder to scan" });
    if (typeof dir === "string") void startScan(dir);
  };

  if (loading && !disks) {
    return (
      <div className="grid gap-4">
        <Skeleton className="h-52 w-full rounded-2xl" />
        <div className="grid grid-cols-3 gap-4"><Skeleton className="h-24" /><Skeleton className="h-24" /><Skeleton className="h-24" /></div>
      </div>
    );
  }
  if (error && !disks) {
    return (
      <Card>
        <EmptyState icon={HardDrive} title="Could not read disks" description={error}
          action={<Button icon={RefreshCw} onClick={reload}>Retry</Button>} />
      </Card>
    );
  }
  if (!disks || disks.length === 0) {
    return (
      <Card>
        <EmptyState icon={HardDrive} title="No disks found" description="You can still scan a folder directly."
          action={<Button icon={FolderSearch} onClick={pick}>Scan folder…</Button>} />
      </Card>
    );
  }

  const main: DiskInfo = disks.find((d) => d.isSystem) ?? disks[0];
  const others = disks.filter((d) => d !== main);
  const frac = main.totalBytes > 0 ? main.usedBytes / main.totalBytes : 0;

  return (
    <div className="grid gap-4">
      <motion.div
        initial={{ opacity: 0, y: 12 }} animate={{ opacity: 1, y: 0 }} transition={{ duration: 0.4, ease: [0.16, 1, 0.3, 1] }}
        className="relative overflow-hidden rounded-2xl border border-border bg-surface p-6 shadow-sm md:p-8"
      >
        <div aria-hidden className="pointer-events-none absolute -right-24 -top-24 h-72 w-72 rounded-full opacity-20 blur-3xl" style={{ background: "var(--accent)" }} />
        <div className="relative flex flex-col items-center gap-8 md:flex-row">
          <RingGauge
            value={frac} size={190} thickness={16} tone={tone(frac)}
            label={<AnimatedNumber value={frac * 100} format={(n) => `${Math.round(n)}%`} />}
            sublabel="used"
          />
          <div className="min-w-0 flex-1 text-center md:text-left">
            <div className="flex items-center justify-center gap-2 md:justify-start">
              <h2 className="truncate text-lg font-semibold text-text">{main.name || main.mountPoint}</h2>
              <Badge tone="neutral">{main.fileSystem}</Badge>
              {main.kind !== "unknown" && <Badge tone="info">{main.kind.toUpperCase()}</Badge>}
            </div>
            <p className="mt-1 text-sm text-muted">{main.mountPoint}</p>
            <div className="mt-4 text-5xl font-semibold tracking-tight text-text">
              <AnimatedNumber value={main.availableBytes} format={(n) => formatBytes(n)} />
              <span className="ml-2 text-base font-normal text-muted">free of {formatBytes(main.totalBytes)}</span>
            </div>
            <p className="mt-1 text-sm text-muted">{formatBytes(main.usedBytes)} used · {formatPercent(main.usedBytes, main.totalBytes)} full</p>
            <div className="mt-6 flex flex-wrap justify-center gap-2 md:justify-start">
              <Button size="lg" variant="primary" icon={Play} disabled={scanning} onClick={() => void startScan()}>
                {scanning ? "Scanning…" : "Scan now"}
              </Button>
              <Button size="lg" icon={FolderSearch} disabled={scanning} onClick={pick}>Scan folder…</Button>
            </div>
          </div>
        </div>
      </motion.div>

      {others.length > 0 && (
        <div className="grid gap-4 sm:grid-cols-2 xl:grid-cols-3">
          {others.map((d, i) => {
            const f = d.totalBytes > 0 ? d.usedBytes / d.totalBytes : 0;
            return (
              <motion.div key={d.id} initial={{ opacity: 0, y: 10 }} animate={{ opacity: 1, y: 0 }} transition={{ delay: 0.05 * i, duration: 0.35 }}>
                <Card className="transition-shadow hover:shadow-md">
                  <div className="flex items-center gap-3">
                    <span className="flex h-9 w-9 items-center justify-center rounded-lg bg-surface-2 text-muted">
                      {d.isRemovable ? <Usb size={18} /> : <HardDrive size={18} />}
                    </span>
                    <div className="min-w-0 flex-1">
                      <div className="truncate text-sm font-medium text-text">{d.name || d.mountPoint}</div>
                      <div className="truncate text-xs text-muted">{d.mountPoint} · {d.fileSystem}</div>
                    </div>
                    <Button size="sm" variant="ghost" disabled={scanning} onClick={() => void startScan(d.mountPoint)}>Scan</Button>
                  </div>
                  <ProgressBar className="mt-4" value={f} tone={tone(f)} />
                  <div className="mt-2 flex justify-between text-xs text-muted">
                    <span>{formatBytes(d.usedBytes)} used</span>
                    <span>{formatBytes(d.availableBytes)} free</span>
                  </div>
                </Card>
              </motion.div>
            );
          })}
        </div>
      )}
    </div>
  );
}
