// Tests for the zustand stores: ui (navigation/sidebar), settings (load/update/theme) and the scan event flow
// (driven by the mock backend's simulated scan events).
import { afterAll, beforeAll, beforeEach, describe, expect, it, vi } from "vitest";
import { installMockBackend, uninstallMockBackend } from "../../lib/mock";
import { api } from "../../lib/api";
import { useUiStore } from "../ui";
import { applyTheme, useSettingsStore, DEFAULT_SETTINGS } from "../settings";

describe("ui store", () => {
  beforeEach(() => useUiStore.setState({ page: "dashboard", pagePath: null, sidebarCollapsed: false }));
  it("setPage sets page and optional path", () => {
    useUiStore.getState().setPage("explorer", "/home/x");
    expect(useUiStore.getState()).toMatchObject({ page: "explorer", pagePath: "/home/x" });
    useUiStore.getState().setPage("cleanup");
    expect(useUiStore.getState().pagePath).toBeNull();
  });
  it("toggles the sidebar", () => {
    useUiStore.getState().toggleSidebar();
    expect(useUiStore.getState().sidebarCollapsed).toBe(true);
    useUiStore.getState().toggleSidebar();
    expect(useUiStore.getState().sidebarCollapsed).toBe(false);
  });
});

describe("settings store", () => {
  beforeAll(() => installMockBackend({ latency: false }));
  afterAll(() => uninstallMockBackend());
  beforeEach(() => useSettingsStore.setState({ settings: DEFAULT_SETTINGS, loaded: false, error: null }));

  it("loads settings from the backend", async () => {
    await useSettingsStore.getState().load();
    const s = useSettingsStore.getState();
    expect(s.loaded).toBe(true);
    expect(s.settings.excludedPaths.length).toBeGreaterThan(0);
  });
  it("update persists and applies the theme to <html>", async () => {
    await useSettingsStore.getState().update({ theme: "dark" });
    expect(document.documentElement.getAttribute("data-theme")).toBe("dark");
    expect(useUiStore.getState().resolvedTheme).toBe("dark");
    expect((await api.getSettings()).theme).toBe("dark");
    await useSettingsStore.getState().update({ theme: "light" });
    expect(document.documentElement.getAttribute("data-theme")).toBe("light");
  });
  it("rolls back and rethrows when saving fails", async () => {
    const spy = vi.spyOn(api, "saveSettings").mockRejectedValueOnce("disk full");
    await expect(useSettingsStore.getState().update({ crossFilesystems: true })).rejects.toBe("disk full");
    expect(useSettingsStore.getState().settings.crossFilesystems).toBe(false);
    expect(useSettingsStore.getState().error).toBe("disk full");
    spy.mockRestore();
  });
  it("system theme follows matchMedia", () => {
    applyTheme("system", false);
    expect(document.documentElement.getAttribute("data-theme")).toBe("light");
  });
});

describe("scan store event flow", () => {
  beforeAll(() => installMockBackend({ latency: false, scanMs: 400 }));
  afterAll(() => uninstallMockBackend());

  it("goes idle -> scanning -> done with progress and a summary", async () => {
    const { useScanStore } = await import("../scan");
    expect(useScanStore.getState().status).toBe("idle");
    const p = useScanStore.getState().startScan("/home/basith");
    expect(useScanStore.getState().status).toBe("scanning");
    await p;
    await vi.waitFor(() => expect(useScanStore.getState().progress).not.toBeNull(), { timeout: 2000 });
    await vi.waitFor(() => expect(useScanStore.getState().status).toBe("done"), { timeout: 4000 });
    const s = useScanStore.getState();
    expect(s.summary?.root).toBe("/home/basith");
    expect(s.summary?.totalBytes).toBeGreaterThan(40 * 1024 ** 3);
    expect(s.scanId).toBe(s.summary?.scanId);
  });

  it("cancelling returns to done (previous summary kept)", async () => {
    const { useScanStore } = await import("../scan");
    await useScanStore.getState().startScan("/home/basith");
    await vi.waitFor(() => expect(useScanStore.getState().progress).not.toBeNull(), { timeout: 2000 });
    await useScanStore.getState().cancelScan();
    await vi.waitFor(() => expect(useScanStore.getState().status).not.toBe("scanning"), { timeout: 2000 });
    expect(useScanStore.getState().error).toBeNull();
  });

  it("reports a start failure as error state", async () => {
    const { useScanStore } = await import("../scan");
    useScanStore.setState({ status: "idle" });
    const spy = vi.spyOn(api, "startScan").mockRejectedValueOnce("root does not exist");
    await useScanStore.getState().startScan("/nope");
    expect(useScanStore.getState()).toMatchObject({ status: "error", error: "root does not exist" });
    spy.mockRestore();
  });
});
