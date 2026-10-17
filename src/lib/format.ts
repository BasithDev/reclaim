// Reclaim: shared formatting helpers (bytes, counts, dates, durations) used across all pages.

const UNITS = ["B", "KB", "MB", "GB", "TB", "PB"];

/** 1536 -> "1.5 KB" (base 1024, matching what file managers show). */
export function formatBytes(bytes: number, decimals = 1): string {
  if (!Number.isFinite(bytes) || bytes <= 0) return "0 B";
  const i = Math.min(Math.floor(Math.log(bytes) / Math.log(1024)), UNITS.length - 1);
  const value = bytes / 1024 ** i;
  return `${value.toFixed(i === 0 ? 0 : decimals)} ${UNITS[i]}`;
}

/** Signed variant for growth deltas: "+1.2 GB" / "-300 MB". */
export function formatDelta(bytes: number): string {
  const sign = bytes > 0 ? "+" : bytes < 0 ? "-" : "";
  return `${sign}${formatBytes(Math.abs(bytes))}`;
}

export function formatCount(n: number): string {
  return new Intl.NumberFormat().format(n);
}

export function formatPercent(part: number, whole: number, decimals = 0): string {
  if (whole <= 0) return "0%";
  return `${((part / whole) * 100).toFixed(decimals)}%`;
}

/** "3 days ago", "just now". */
export function formatRelative(ms: number | null): string {
  if (ms == null) return "unknown";
  const diff = Date.now() - ms;
  const rtf = new Intl.RelativeTimeFormat(undefined, { numeric: "auto" });
  const steps: [number, Intl.RelativeTimeFormatUnit][] = [
    [60_000, "second"], [3_600_000, "minute"], [86_400_000, "hour"],
    [2_592_000_000, "day"], [31_536_000_000, "month"], [Infinity, "year"],
  ];
  const divisors = [1000, 60_000, 3_600_000, 86_400_000, 2_592_000_000, 31_536_000_000];
  for (let i = 0; i < steps.length; i++) {
    if (Math.abs(diff) < steps[i][0]) return rtf.format(-Math.round(diff / divisors[i]), steps[i][1]);
  }
  return "unknown";
}

export function formatDate(ms: number | null): string {
  if (ms == null) return "unknown";
  return new Date(ms).toLocaleString(undefined, { dateStyle: "medium", timeStyle: "short" });
}

/** 83500 -> "1m 23s". */
export function formatDuration(ms: number): string {
  const s = Math.round(ms / 1000);
  if (s < 60) return `${s}s`;
  const m = Math.floor(s / 60);
  return m < 60 ? `${m}m ${s % 60}s` : `${Math.floor(m / 60)}h ${m % 60}m`;
}
