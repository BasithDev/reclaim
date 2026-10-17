// Reclaim: Dashboard live panel shown while a scan runs (counters, files/s, current path, cancel).
import { motion } from "motion/react";
import { Square } from "lucide-react";
import { formatBytes, formatCount, formatDuration } from "../../lib/format";
import { AnimatedNumber, Button, Card, ProgressBar } from "../../components";
import { useScanStore } from "../../store/scan";

export default function ScanLive() {
  const progress = useScanStore((s) => s.progress);
  const cancelScan = useScanStore((s) => s.cancelScan);
  const rate = progress && progress.elapsedMs > 0 ? (progress.filesScanned / progress.elapsedMs) * 1000 : 0;

  return (
    <motion.div initial={{ opacity: 0, y: 12 }} animate={{ opacity: 1, y: 0 }} exit={{ opacity: 0, y: -8 }}>
      <Card
        title="Scanning…"
        subtitle={progress ? `Elapsed ${formatDuration(progress.elapsedMs)}` : "Starting up"}
        actions={<Button size="sm" variant="danger" icon={Square} onClick={() => void cancelScan()}>Cancel</Button>}
      >
        <ProgressBar value={0} indeterminate />
        <div className="mt-5 grid grid-cols-2 gap-6 md:grid-cols-4">
          <Metric label="Size counted" value={<AnimatedNumber value={progress?.bytesCounted ?? 0} format={formatBytes} duration={0.3} />} big />
          <Metric label="Files" value={<AnimatedNumber value={progress?.filesScanned ?? 0} format={(n) => formatCount(Math.round(n))} duration={0.3} />} />
          <Metric label="Folders" value={<AnimatedNumber value={progress?.dirsScanned ?? 0} format={(n) => formatCount(Math.round(n))} duration={0.3} />} />
          <Metric label="Speed" value={<AnimatedNumber value={rate} format={(n) => `${formatCount(Math.round(n))}/s`} duration={0.3} />} />
        </div>
        <p className="mt-5 truncate font-mono text-xs text-faint" title={progress?.currentPath}>{progress?.currentPath || "…"}</p>
        {!!progress?.errors && <p className="mt-1 text-xs text-warning">{formatCount(progress.errors)} items could not be read</p>}
      </Card>
    </motion.div>
  );
}

function Metric({ label, value, big }: { label: string; value: React.ReactNode; big?: boolean }) {
  return (
    <div>
      <div className="text-xs text-muted">{label}</div>
      <div className={big ? "mt-1 text-3xl font-semibold tracking-tight text-text" : "mt-1 text-2xl font-semibold tracking-tight text-text"}>{value}</div>
    </div>
  );
}
