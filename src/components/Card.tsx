// Reclaim: Card surface with optional title row, plus Badge, Chip and Skeleton primitives.
import type { ButtonHTMLAttributes, ReactNode } from "react";
import { cn } from "./cn";

export function Card({ title, subtitle, actions, padded = true, className, children }: {
  title?: ReactNode; subtitle?: ReactNode; actions?: ReactNode; padded?: boolean; className?: string; children?: ReactNode;
}) {
  return (
    <section className={cn("rounded-xl border border-border bg-surface shadow-sm", className)}>
      {(title || actions) && (
        <header className="flex items-center justify-between gap-3 px-5 pt-4">
          <div className="min-w-0">
            {title && <h3 className="truncate text-sm font-semibold text-text">{title}</h3>}
            {subtitle && <p className="truncate text-xs text-muted">{subtitle}</p>}
          </div>
          {actions && <div className="flex shrink-0 items-center gap-1">{actions}</div>}
        </header>
      )}
      <div className={cn(padded && "p-5", (title || actions) && padded && "pt-3")}>{children}</div>
    </section>
  );
}

const badgeTones = {
  neutral: "bg-surface-2 text-muted",
  accent: "bg-accent-soft text-accent-strong dark:text-accent",
  success: "bg-success-soft text-success",
  warning: "bg-warning-soft text-warning",
  danger: "bg-danger-soft text-danger",
  info: "bg-info-soft text-info",
};

export function Badge({ tone = "neutral", children }: { tone?: keyof typeof badgeTones; children: ReactNode }) {
  return (
    <span className={cn("inline-flex items-center gap-1 rounded-full px-2 py-0.5 text-[11px] font-medium leading-4", badgeTones[tone])}>
      {children}
    </span>
  );
}

export function Chip({ active, color, count, onClick, children, ...rest }: {
  active?: boolean; color?: string; count?: ReactNode; onClick?: () => void; children: ReactNode;
} & Omit<ButtonHTMLAttributes<HTMLButtonElement>, "onClick" | "children" | "color">) {
  return (
    <button
      type="button"
      onClick={onClick}
      aria-pressed={active}
      className={cn(
        "inline-flex h-7 cursor-pointer items-center gap-1.5 rounded-full border px-3 text-xs font-medium transition-all duration-150 active:scale-95",
        active ? "border-transparent bg-text text-bg" : "border-border bg-surface text-muted hover:border-border-strong hover:text-text",
      )}
      {...rest}
    >
      {color && <span className="h-2 w-2 rounded-full" style={{ background: color }} />}
      {children}
      {count != null && <span className={cn("tnum", active ? "opacity-70" : "text-faint")}>{count}</span>}
    </button>
  );
}

export function Skeleton({ className }: { className?: string }) {
  return <div aria-hidden className={cn("rc-skeleton rounded-md", className)} />;
}
