// Smoke tests for every page: loading, empty, error and data states, driven by the mock backend or api spies.
import { afterAll, afterEach, beforeAll, beforeEach, describe, expect, it, vi } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import type { ReactElement } from "react";
import { ToastProvider } from "../../components";
import { api } from "../../lib/api";
import { installMockBackend, uninstallMockBackend } from "../../lib/mock";
import type { ScanSummary } from "../../lib/types";
import { useScanStore } from "../../store/scan";
import { DEFAULT_SETTINGS, useSettingsStore } from "../../store/settings";
import { useUiStore } from "../../store/ui";
import DashboardPage from "../dashboard";
import ExplorerPage from "../explorer";
import FilesPage from "../files";
import CleanupPage from "../cleanup";
import HistoryPage from "../history";
import SettingsPage from "../settings";

const wrap = (el: ReactElement) => render(<ToastProvider>{el}</ToastProvider>);
const SUMMARY: ScanSummary = {
  scanId: "mock-scan-x", root: "/home/basith", totalBytes: 52 * 1024 ** 3, apparentBytes: 54 * 1024 ** 3, fileCount: 600000, dirCount: 40000, errorCount: 3,
  startedAtMs: Date.now() - 5000, finishedAtMs: Date.now(), durationMs: 5000, typeBreakdown: [{ category: "video", bytes: 1e10, count: 20 }], snapshotId: 4,
};

beforeAll(() => installMockBackend({ latency: false }));
afterAll(() => uninstallMockBackend());
afterEach(() => vi.restoreAllMocks());
beforeEach(() => {
  useScanStore.setState({ status: "idle", progress: null, summary: null, scanId: null, error: null });
  useUiStore.setState({ page: "dashboard", pagePath: null });
  useSettingsStore.setState({ settings: DEFAULT_SETTINGS, loaded: false, error: null });
});

/** Make the mock backend know a finished scan so tree/file queries work. */
async function finishedScan() {
  const id = await api.startScan({ root: "/home/basith" });
  return id;
}
void finishedScan;

describe("Settings page", () => {
  it("shows skeletons then the loaded settings", async () => {
    wrap(<SettingsPage />);
    expect(screen.queryByText("Appearance")).toBeNull();
    expect(await screen.findByText("Appearance")).toBeInTheDocument();
    expect(screen.getByText("Excluded paths")).toBeInTheDocument();
    expect(screen.getByText("/home/basith/.steam")).toBeInTheDocument();
  });
  it("saves a changed switch through the API", async () => {
    wrap(<SettingsPage />);
    await screen.findByText("Appearance");
    await userEvent.click(screen.getAllByRole("switch")[0]);
    await waitFor(async () => expect((await api.getSettings()).crossFilesystems).toBe(true));
  });
  it("shows an error banner with retry when loading fails", async () => {
    vi.spyOn(api, "getSettings").mockRejectedValue("boom");
    wrap(<SettingsPage />);
    expect(await screen.findByText("boom")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Retry" })).toBeInTheDocument();
  });
});

describe("Cleanup page", () => {
  it("lists grouped cleaners with previewed sizes", async () => {
    wrap(<CleanupPage />);
    expect(await screen.findByText("Systemd journal")).toBeInTheDocument();
    expect(screen.getByText("Developer")).toBeInTheDocument();
    expect((await screen.findAllByText("1.4 GB", {}, { timeout: 3000 })).length).toBeGreaterThan(0);
  });
  it("selecting a cleaner shows the sticky footer and a confirm dialog", async () => {
    wrap(<CleanupPage />);
    await screen.findByText("Systemd journal");
    await screen.findAllByText(/GB|MB/, {}, { timeout: 3000 });
    await waitFor(() => expect(screen.getAllByRole("checkbox")[0]).toBeInTheDocument());
    await userEvent.click(screen.getAllByRole("checkbox")[0]);
    await userEvent.click(await screen.findByRole("button", { name: /Clean selected/ }));
    expect(await screen.findByRole("dialog", { name: "Clean selected items?" })).toBeInTheDocument();
  });
  it("runs the cleanup and shows the results panel", async () => {
    wrap(<CleanupPage />);
    await screen.findByText("Systemd journal");
    await userEvent.click(screen.getAllByRole("checkbox")[0]);
    await userEvent.click(await screen.findByRole("button", { name: /Clean selected/ }));
    await userEvent.click(await screen.findByRole("button", { name: /^Clean \d/ }));
    expect(await screen.findByText("Space reclaimed", {}, { timeout: 3000 })).toBeInTheDocument();
  });
  it("shows an error with retry when cleaners fail to load", async () => {
    vi.spyOn(api, "listCleaners").mockRejectedValue("no cleaners");
    wrap(<CleanupPage />);
    expect(await screen.findByText("Couldn't load cleaners")).toBeInTheDocument();
    expect(screen.getByText("no cleaners")).toBeInTheDocument();
  });
  it("shows an empty state when there are no cleaners", async () => {
    vi.spyOn(api, "listCleaners").mockResolvedValue([]);
    wrap(<CleanupPage />);
    expect(await screen.findByText("Nothing to clean on this system")).toBeInTheDocument();
  });
  it("shows a per-row retry when one preview fails", async () => {
    vi.spyOn(api, "previewCleaner").mockRejectedValue("preview broke");
    wrap(<CleanupPage />);
    expect((await screen.findAllByText("Retry"))[0]).toBeInTheDocument();
  });
});

describe("History page", () => {
  it("renders snapshots and the growth list for the latest pair", async () => {
    wrap(<HistoryPage />);
    expect(await screen.findByText("Snapshots")).toBeInTheDocument();
    expect(await screen.findByText("latest")).toBeInTheDocument();
    expect(await screen.findByText("/home/basith/Downloads", {}, { timeout: 3000 })).toBeInTheDocument();
  });
  it("shows the empty state with no snapshots", async () => {
    vi.spyOn(api, "listSnapshots").mockResolvedValue([]);
    wrap(<HistoryPage />);
    expect(await screen.findByText("No snapshots yet")).toBeInTheDocument();
  });
  it("shows an error with retry", async () => {
    vi.spyOn(api, "listSnapshots").mockRejectedValue("db locked");
    wrap(<HistoryPage />);
    expect(await screen.findByText("Couldn't load history")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Retry" })).toBeInTheDocument();
  });
  it("deleting a snapshot asks for confirmation", async () => {
    wrap(<HistoryPage />);
    await screen.findByText("latest");
    await userEvent.click(screen.getAllByLabelText("Delete snapshot")[0]);
    expect(await screen.findByRole("dialog", { name: "Delete snapshot?" })).toBeInTheDocument();
  });
});

describe("Dashboard page (A4)", () => {
  it("shows disks and the no-scan empty state", async () => {
    wrap(<DashboardPage />);
    expect(await screen.findByText("No scan yet")).toBeInTheDocument();
    expect(await screen.findByText(/Ubuntu/)).toBeInTheDocument();
  });
  it("shows the error state when the scan failed", async () => {
    useScanStore.setState({ status: "error", error: "scan exploded" });
    wrap(<DashboardPage />);
    expect(await screen.findByText("Scan failed")).toBeInTheDocument();
    expect(screen.getByText("scan exploded")).toBeInTheDocument();
  });
  it("shows results after a scan", async () => {
    const scanId = await api.startScan({ root: "/home/basith" });
    void scanId;
    useScanStore.setState({ status: "done", summary: { ...SUMMARY, scanId } });
    wrap(<DashboardPage />);
    expect(await screen.findByText("Biggest folders", {}, { timeout: 3000 })).toBeInTheDocument();
  });
});

describe("Explorer page (A4)", () => {
  it("shows the empty state before a scan", () => {
    wrap(<ExplorerPage />);
    expect(screen.getByText("Nothing scanned yet")).toBeInTheDocument();
  });
  it("shows the folder contents after a scan", async () => {
    const scanId = await api.startScan({ root: "/home/basith" });
    useScanStore.setState({ status: "done", summary: { ...SUMMARY, scanId } });
    wrap(<ExplorerPage />);
    expect(await screen.findByText("Downloads", {}, { timeout: 3000 })).toBeInTheDocument();
  });
  it("shows an error state with retry", async () => {
    vi.spyOn(api, "getTree").mockRejectedValue("tree failed");
    useScanStore.setState({ status: "done", summary: SUMMARY });
    wrap(<ExplorerPage />);
    expect(await screen.findByText("Could not load this folder")).toBeInTheDocument();
  });
});

describe("Large Files page (A4)", () => {
  it("shows the empty state before a scan", () => {
    wrap(<FilesPage />);
    expect(screen.getByText("Nothing scanned yet")).toBeInTheDocument();
  });
  it("lists large files after a scan", async () => {
    const scanId = await api.startScan({ root: "/home/basith" });
    useScanStore.setState({ status: "done", summary: { ...SUMMARY, scanId } });
    wrap(<FilesPage />);
    expect(await screen.findByText("ubuntu-24.04.2-desktop-amd64.iso", {}, { timeout: 3000 })).toBeInTheDocument();
  });
  it("shows an error state", async () => {
    vi.spyOn(api, "getLargestFiles").mockRejectedValue("files failed");
    useScanStore.setState({ status: "done", summary: SUMMARY });
    wrap(<FilesPage />);
    expect(await screen.findByText("Could not load files")).toBeInTheDocument();
  });
});
