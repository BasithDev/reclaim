// Vitest setup: jest-dom matchers, DOM cleanup, and jsdom polyfills (matchMedia, ResizeObserver, canvas) used by the UI.
import "@testing-library/jest-dom/vitest";
import { afterEach, vi } from "vitest";
import { cleanup } from "@testing-library/react";

afterEach(() => cleanup());

Object.defineProperty(window, "matchMedia", {
  writable: true,
  value: (query: string) => ({
    matches: false, media: query, onchange: null,
    addEventListener: () => {}, removeEventListener: () => {}, addListener: () => {}, removeListener: () => {}, dispatchEvent: () => false,
  }),
});
class RO { observe() {} unobserve() {} disconnect() {} }
(globalThis as unknown as { ResizeObserver: typeof RO }).ResizeObserver ??= RO;
window.scrollTo = vi.fn() as unknown as typeof window.scrollTo;

// ECharts needs a real canvas; tests don't care about pixels, so stub the chart component.
vi.mock("echarts-for-react", () => ({ default: () => null }));
