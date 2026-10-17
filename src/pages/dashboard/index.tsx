// Reclaim: Dashboard page. Disk hero, live scan panel, and scan results with charts. Owned by A4.
import { AnimatePresence } from "motion/react";
import { RefreshCw, ScanSearch } from "lucide-react";
import { Button, Card, EmptyState, PageHeader } from "../../components";
import { useScanStore } from "../../store/scan";
import DiskHero from "./DiskHero";
import ScanLive from "./ScanLive";
import ScanResults from "./ScanResults";

export default function DashboardPage() {
  const status = useScanStore((s) => s.status);
  const summary = useScanStore((s) => s.summary);
  const error = useScanStore((s) => s.error);
  const startScan = useScanStore((s) => s.startScan);

  return (
    <div className="grid gap-6">
      <PageHeader title="Dashboard" description="See where your disk space went." />
      <DiskHero />
      <AnimatePresence mode="wait">
        {status === "scanning" ? (
          <ScanLive key="live" />
        ) : status === "error" ? (
          <Card key="err">
            <EmptyState icon={ScanSearch} title="Scan failed" description={error ?? "Unknown error"}
              action={<Button icon={RefreshCw} onClick={() => void startScan()}>Try again</Button>} />
          </Card>
        ) : summary ? (
          <ScanResults key={summary.scanId} summary={summary} />
        ) : (
          <Card key="empty">
            <EmptyState icon={ScanSearch} title="No scan yet" description="Run a scan to see what is using your space, the biggest folders, and what you can clean."
              action={<Button variant="primary" onClick={() => void startScan()}>Scan now</Button>} />
          </Card>
        )}
      </AnimatePresence>
    </div>
  );
}
