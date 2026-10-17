// Reclaim: data-display components: AnimatedNumber, ProgressBar, RingGauge, StatTile, CategoryDot.
import { useEffect, useRef, type ReactNode } from "react";
import type { LucideIcon } from "lucide-react";
import { animate, motion, useReducedMotion } from "motion/react";
import type { FileCategory } from "../lib/types";
import { cn, toneColor, toneSoft, type Tone } from "./cn";
import { categoryMeta } from "./categoryMeta";

export function AnimatedNumber({ value, format = (n) => Math.round(n).toLocaleString(), duration = 0.8 }: {
  value: number; format?: (n: number) => string; duration?: number;
}) {
  const ref = useRef<HTMLSpanElement>(null);
  const last = useRef(value);
  const reduce = useReducedMotion();
  const fmt = useRef(format);
  fmt.current = format;
  useEffect(() => {
    const el = ref.current;
    if (!el) return;
    if (reduce) { el.textContent = fmt.current(value); last.current = value; return; }
    const controls = animate(last.current, value, {
      duration, ease: [0.16, 1, 0.3, 1],
      onUpdate: (v) => { el.textContent = fmt.current(v); last.current = v; },
    });
    return () => controls.stop();
  }, [value, duration, reduce]);
  return <span ref={ref} className="tnum">{format(last.current)}</span>;
}

export function ProgressBar({ value = 0, tone = "default", indeterminate, className }: {
  value?: number; tone?: Tone; indeterminate?: boolean; className?: string;
}) {
  const v = Math.max(0, Math.min(1, value || 0));
  return (
    <div
      role="progressbar" aria-valuenow={indeterminate ? undefined : Math.round(v * 100)}
      className={cn("relative h-1.5 w-full overflow-hidden rounded-full bg-surface-3", className)}
    >
      {indeterminate ? (
        <div className="absolute h-full w-2/5 rounded-full" style={{ background: toneColor[tone], animation: "rc-indeterminate 1.3s ease-in-out infinite" }} />
      ) : (
        <motion.div
          className="h-full rounded-full" style={{ background: toneColor[tone] }}
          initial={false} animate={{ width: `${v * 100}%` }} transition={{ type: "spring", stiffness: 140, damping: 24 }}
        />
      )}
    </div>
  );
}

export function RingGauge({ value, size = 160, thickness = 12, label, sublabel, tone = "default" }: {
  value: number; size?: number; thickness?: number; label?: ReactNode; sublabel?: ReactNode; tone?: Tone;
}) {
  const v = Math.max(0, Math.min(1, value || 0));
  const r = (size - thickness) / 2;
  const c = 2 * Math.PI * r;
  return (
    <div className="relative shrink-0" style={{ width: size, height: size }}>
      <svg width={size} height={size} className="-rotate-90">
        <circle cx={size / 2} cy={size / 2} r={r} fill="none" stroke="var(--surface-3)" strokeWidth={thickness} />
        <motion.circle
          cx={size / 2} cy={size / 2} r={r} fill="none" stroke={toneColor[tone]} strokeWidth={thickness}
          strokeLinecap="round" strokeDasharray={c}
          initial={{ strokeDashoffset: c }} animate={{ strokeDashoffset: c * (1 - v) }}
          transition={{ duration: 1, ease: [0.16, 1, 0.3, 1] }}
        />
      </svg>
      <div className="absolute inset-0 flex flex-col items-center justify-center text-center">
        {label != null && <div className="text-2xl font-semibold leading-none tracking-tight text-text">{label}</div>}
        {sublabel != null && <div className="mt-1 text-xs text-muted">{sublabel}</div>}
      </div>
    </div>
  );
}

export function StatTile({ label, value, sub, icon: Icon, tone = "default" }: {
  label: string; value: ReactNode; sub?: ReactNode; icon?: LucideIcon; tone?: "default" | "accent" | "warning" | "danger";
}) {
  return (
    <div className="rounded-xl border border-border bg-surface p-4 shadow-sm">
      <div className="flex items-center justify-between">
        <span className="text-xs font-medium text-muted">{label}</span>
        {Icon && (
          <span className="flex h-7 w-7 items-center justify-center rounded-md" style={{ background: toneSoft[tone], color: tone === "default" ? "var(--text-muted)" : toneColor[tone] }}>
            <Icon size={15} />
          </span>
        )}
      </div>
      <div className="mt-2 text-2xl font-semibold tracking-tight text-text">{value}</div>
      {sub && <div className="mt-0.5 text-xs text-muted">{sub}</div>}
    </div>
  );
}

export function CategoryDot({ category }: { category: FileCategory }) {
  return <span aria-hidden className="inline-block h-2.5 w-2.5 shrink-0 rounded-full" style={{ background: `var(--cat-${category})` }} title={categoryMeta[category].label} />;
}
