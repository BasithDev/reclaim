// Reclaim: form controls: Checkbox, Switch, Tabs and SegmentedControl.
import type { ReactNode } from "react";
import type { LucideIcon } from "lucide-react";
import { Check, Minus } from "lucide-react";
import { motion } from "motion/react";
import { useId } from "react";
import { cn } from "./cn";

export function Checkbox({ checked, indeterminate, onChange, label }: {
  checked: boolean; indeterminate?: boolean; onChange: (v: boolean) => void; label?: ReactNode;
}) {
  const on = checked || indeterminate;
  return (
    <label className="inline-flex cursor-pointer items-center gap-2 text-sm text-text" onClick={(e) => e.stopPropagation()}>
      <button
        type="button" role="checkbox" aria-checked={indeterminate ? "mixed" : checked}
        onClick={() => onChange(!checked)}
        className={cn(
          "flex h-4 w-4 shrink-0 cursor-pointer items-center justify-center rounded border transition-all duration-150 active:scale-90",
          on ? "border-accent bg-accent text-accent-contrast" : "border-border-strong bg-surface hover:border-accent",
        )}
      >
        {on && <motion.span initial={{ scale: 0.4, opacity: 0 }} animate={{ scale: 1, opacity: 1 }} transition={{ duration: 0.12 }}>
          {indeterminate ? <Minus size={12} strokeWidth={3} /> : <Check size={12} strokeWidth={3} />}
        </motion.span>}
      </button>
      {label}
    </label>
  );
}

export function Switch({ checked, onChange, label, description }: {
  checked: boolean; onChange: (v: boolean) => void; label?: ReactNode; description?: ReactNode;
}) {
  const id = useId();
  return (
    <div className="flex items-center justify-between gap-6">
      {(label || description) && (
        <label htmlFor={id} className="min-w-0 cursor-pointer">
          {label && <div className="text-sm font-medium text-text">{label}</div>}
          {description && <div className="text-xs text-muted">{description}</div>}
        </label>
      )}
      <button
        id={id} type="button" role="switch" aria-checked={checked} onClick={() => onChange(!checked)}
        className={cn("relative h-5 w-9 shrink-0 cursor-pointer rounded-full transition-colors duration-200", checked ? "bg-accent" : "bg-border-strong")}
      >
        <motion.span
          className="absolute left-0.5 top-0.5 h-4 w-4 rounded-full bg-white shadow"
          animate={{ x: checked ? 16 : 0 }} transition={{ type: "spring", stiffness: 500, damping: 32 }}
        />
      </button>
    </div>
  );
}

interface Item { value: string; label: string; icon?: LucideIcon }

export function Tabs({ value, onChange, items }: { value: string; onChange: (v: string) => void; items: Item[] }) {
  const id = useId();
  return (
    <div role="tablist" className="flex gap-1 border-b border-border">
      {items.map(({ value: v, label, icon: Icon }) => (
        <button
          key={v} role="tab" aria-selected={v === value} onClick={() => onChange(v)}
          className={cn("relative flex cursor-pointer items-center gap-1.5 px-3 py-2 text-sm font-medium transition-colors", v === value ? "text-text" : "text-muted hover:text-text")}
        >
          {Icon && <Icon size={14} />}
          {label}
          {v === value && <motion.span layoutId={`tab-${id}`} className="absolute inset-x-2 -bottom-px h-0.5 rounded bg-accent" transition={{ type: "spring", stiffness: 500, damping: 36 }} />}
        </button>
      ))}
    </div>
  );
}

export function SegmentedControl({ value, onChange, options }: { value: string; onChange: (v: string) => void; options: Item[] }) {
  const id = useId();
  return (
    <div role="radiogroup" className="inline-flex rounded-lg bg-surface-2 p-0.5">
      {options.map(({ value: v, label, icon: Icon }) => (
        <button
          key={v} role="radio" aria-checked={v === value} onClick={() => onChange(v)}
          className={cn("relative flex h-8 cursor-pointer items-center gap-1.5 rounded-md px-3 text-xs font-medium transition-colors", v === value ? "text-text" : "text-muted hover:text-text")}
        >
          {v === value && <motion.span layoutId={`seg-${id}`} className="absolute inset-0 rounded-md border border-border bg-surface shadow-sm" transition={{ type: "spring", stiffness: 500, damping: 36 }} />}
          <span className="relative flex items-center gap-1.5">{Icon && <Icon size={14} />}{label}</span>
        </button>
      ))}
    </div>
  );
}
