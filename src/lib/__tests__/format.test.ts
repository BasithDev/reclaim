// Tests for lib/format.ts: bytes, deltas, counts, percentages, relative dates and durations.
import { describe, expect, it } from "vitest";
import { formatBytes, formatCount, formatDate, formatDelta, formatDuration, formatPercent, formatRelative } from "../format";

describe("formatBytes", () => {
  it("handles zero, negatives and NaN", () => {
    expect(formatBytes(0)).toBe("0 B");
    expect(formatBytes(-5)).toBe("0 B");
    expect(formatBytes(NaN)).toBe("0 B");
  });
  it("uses base 1024 units", () => {
    expect(formatBytes(512)).toBe("512 B");
    expect(formatBytes(1536)).toBe("1.5 KB");
    expect(formatBytes(1024 ** 3)).toBe("1.0 GB");
    expect(formatBytes(134 * 1024 ** 3)).toBe("134.0 GB");
  });
  it("respects decimals and caps at PB", () => {
    expect(formatBytes(1536, 0)).toBe("2 KB");
    expect(formatBytes(1024 ** 6)).toBe("1024.0 PB");
  });
});

describe("formatDelta", () => {
  it("signs values", () => {
    expect(formatDelta(1024 ** 3)).toBe("+1.0 GB");
    expect(formatDelta(-(1024 ** 2))).toBe("-1.0 MB");
    expect(formatDelta(0)).toBe("0 B");
  });
});

describe("formatCount / formatPercent", () => {
  it("formats counts with grouping", () => expect(formatCount(1234567)).toBe(new Intl.NumberFormat().format(1234567)));
  it("computes percentages and guards zero", () => {
    expect(formatPercent(1, 4)).toBe("25%");
    expect(formatPercent(1, 3, 1)).toBe("33.3%");
    expect(formatPercent(5, 0)).toBe("0%");
  });
});

describe("formatRelative / formatDate", () => {
  it("handles null", () => {
    expect(formatRelative(null)).toBe("unknown");
    expect(formatDate(null)).toBe("unknown");
  });
  it("describes recent and old times", () => {
    expect(formatRelative(Date.now() - 3 * 86_400_000)).toMatch(/3 days ago/);
    expect(formatRelative(Date.now() - 2 * 3_600_000)).toMatch(/2 hours ago/);
    expect(formatRelative(Date.now() - 400 * 86_400_000)).toMatch(/year|months/);
  });
  it("formats a date string", () => expect(formatDate(Date.UTC(2024, 0, 15, 12))).toMatch(/2024/));
});

describe("formatDuration", () => {
  it("formats seconds, minutes and hours", () => {
    expect(formatDuration(45_000)).toBe("45s");
    expect(formatDuration(83_500)).toBe("1m 24s");
    expect(formatDuration(3_900_000)).toBe("1h 5m");
  });
});
