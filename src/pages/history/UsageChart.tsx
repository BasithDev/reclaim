// Reclaim: ECharts area chart of disk used over time (from snapshot diskUsedBytes), themed via useChartTheme.
import { useMemo } from "react";
import ReactECharts from "echarts-for-react";
import { useChartTheme } from "../../components";
import { formatBytes, formatDate } from "../../lib/format";
import type { SnapshotMeta } from "../../lib/types";

export function UsageChart({ snapshots }: { snapshots: SnapshotMeta[] }) {
  const t = useChartTheme();
  const option = useMemo(() => {
    const pts = snapshots.filter((s) => s.diskUsedBytes != null).sort((a, b) => a.takenAtMs - b.takenAtMs);
    return {
      animationDuration: 600,
      grid: { left: 8, right: 16, top: 16, bottom: 8, containLabel: true },
      tooltip: {
        trigger: "axis", backgroundColor: t.tooltipBg, borderColor: t.tooltipBorder, textStyle: { color: t.textColor, fontSize: 12 },
        formatter: (p: { dataIndex: number }[]) => {
          const s = pts[p[0].dataIndex];
          return `<div>${formatDate(s.takenAtMs)}</div><b>${formatBytes(s.diskUsedBytes ?? 0)}</b> used${s.diskTotalBytes ? ` of ${formatBytes(s.diskTotalBytes)}` : ""}<br/><span style="opacity:.7">${s.root}</span>`;
        },
      },
      xAxis: { type: "category", boundaryGap: false, data: pts.map((s) => formatDate(s.takenAtMs)), axisLabel: { color: t.mutedColor, hideOverlap: true, formatter: (v: string) => v.split(",").slice(0, 2).join(",") }, axisLine: { lineStyle: { color: t.gridColor } }, axisTick: { show: false } },
      yAxis: { type: "value", scale: true, axisLabel: { color: t.mutedColor, formatter: (v: number) => formatBytes(v, 0) }, splitLine: { lineStyle: { color: t.gridColor } } },
      series: [{
        type: "line", smooth: true, symbol: "circle", symbolSize: 7, showSymbol: pts.length < 40, data: pts.map((s) => s.diskUsedBytes),
        lineStyle: { color: t.accent, width: 2.5 }, itemStyle: { color: t.accent },
        areaStyle: { color: { type: "linear", x: 0, y: 0, x2: 0, y2: 1, colorStops: [{ offset: 0, color: `${t.accent}55` }, { offset: 1, color: `${t.accent}00` }] } },
      }],
    };
  }, [snapshots, t]);
  return <ReactECharts option={option} style={{ height: 240 }} notMerge opts={{ renderer: "canvas" }} />;
}
