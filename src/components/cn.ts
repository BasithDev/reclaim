// Reclaim: tiny className combiner plus shared tone -> CSS variable maps for components.
import clsx, { type ClassValue } from "clsx";

export const cn = (...v: ClassValue[]) => clsx(v);

export type Tone = "default" | "accent" | "warning" | "danger" | "info";

/** CSS colour for a tone (used by bars, gauges and tile icons). */
export const toneColor: Record<Tone, string> = {
  default: "var(--accent)",
  accent: "var(--accent)",
  warning: "var(--warning)",
  danger: "var(--danger)",
  info: "var(--info)",
};

export const toneSoft: Record<Tone, string> = {
  default: "var(--surface-2)",
  accent: "var(--accent-soft)",
  warning: "var(--warning-soft)",
  danger: "var(--danger-soft)",
  info: "var(--info-soft)",
};
