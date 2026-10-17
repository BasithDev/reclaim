// Reclaim: useChartTheme() returns concrete colours for ECharts from the current theme tokens,
// and re-renders when the theme (data-theme on <html>) changes.
import { useEffect, useState } from "react";
import type { FileCategory } from "../lib/types";

export interface ChartTheme {
  textColor: string; mutedColor: string; gridColor: string; tooltipBg: string; tooltipBorder: string;
  accent: string; palette: string[];
}

const PALETTE_CATS: FileCategory[] = ["video", "document", "code", "image", "audio", "archive", "database", "diskImage", "log", "executable"];

function read(): ChartTheme {
  const s = getComputedStyle(document.documentElement);
  const g = (n: string) => s.getPropertyValue(n).trim();
  return {
    textColor: g("--text"), mutedColor: g("--text-muted"), gridColor: g("--border"),
    tooltipBg: g("--surface"), tooltipBorder: g("--border-strong"), accent: g("--accent"),
    palette: [g("--accent"), ...PALETTE_CATS.filter((c) => c !== "code").map((c) => g(`--cat-${c}`))],
  };
}

export function useChartTheme(): ChartTheme {
  const [theme, setTheme] = useState(read);
  useEffect(() => {
    const update = () => requestAnimationFrame(() => setTheme(read()));
    const mo = new MutationObserver(update);
    mo.observe(document.documentElement, { attributes: true, attributeFilter: ["data-theme"] });
    update();
    return () => mo.disconnect();
  }, []);
  return theme;
}
