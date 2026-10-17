// Reclaim: DropdownMenu (portal popover with items, separators, shortcuts, keyboard nav).
import { useEffect, useRef, useState, type ReactNode } from "react";
import { createPortal } from "react-dom";
import { AnimatePresence, motion } from "motion/react";
import type { LucideIcon } from "lucide-react";
import { cn } from "./cn";

export type MenuItem =
  | { label: string; icon?: LucideIcon; shortcut?: string; onSelect: () => void; disabled?: boolean; danger?: boolean }
  | "separator";

export function DropdownMenu({ trigger, items }: { trigger: ReactNode; items: MenuItem[] }) {
  const [open, setOpen] = useState(false);
  const [pos, setPos] = useState({ x: 0, y: 0 });
  const wrap = useRef<HTMLDivElement>(null);
  const menu = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!open) return;
    const onDown = (e: MouseEvent) => {
      const t = e.target as Node;
      if (!menu.current?.contains(t) && !wrap.current?.contains(t)) setOpen(false);
    };
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") { setOpen(false); return; }
      if (e.key === "ArrowDown" || e.key === "ArrowUp") {
        e.preventDefault();
        const btns = Array.from(menu.current?.querySelectorAll<HTMLButtonElement>("button:not(:disabled)") ?? []);
        const i = btns.indexOf(document.activeElement as HTMLButtonElement);
        const n = e.key === "ArrowDown" ? (i + 1) % btns.length : (i - 1 + btns.length) % btns.length;
        btns[n]?.focus();
      }
    };
    const onBlur = () => setOpen(false);
    window.addEventListener("mousedown", onDown);
    window.addEventListener("keydown", onKey);
    window.addEventListener("blur", onBlur);
    return () => { window.removeEventListener("mousedown", onDown); window.removeEventListener("keydown", onKey); window.removeEventListener("blur", onBlur); };
  }, [open]);

  const toggle = () => {
    const r = wrap.current?.getBoundingClientRect();
    if (r) setPos({ x: Math.min(r.left, window.innerWidth - 240), y: r.bottom + 4 });
    setOpen((o) => !o);
  };

  return (
    <>
      <div ref={wrap} className="inline-block" onClick={toggle}>{trigger}</div>
      {createPortal(
        <AnimatePresence>
          {open && (
            <motion.div
              ref={menu} role="menu"
              initial={{ opacity: 0, y: -4, scale: 0.98 }} animate={{ opacity: 1, y: 0, scale: 1 }} exit={{ opacity: 0, scale: 0.98 }} transition={{ duration: 0.12 }}
              style={{ position: "fixed", left: pos.x, top: pos.y, zIndex: 70, transformOrigin: "top left" }}
              className="min-w-[210px] rounded-lg border border-border bg-surface p-1 shadow-lg"
            >
              {items.map((it, i) =>
                it === "separator" ? <div key={i} className="my-1 h-px bg-border" /> : (
                  <button
                    key={i} role="menuitem" disabled={it.disabled}
                    onClick={() => { setOpen(false); it.onSelect(); }}
                    className={cn(
                      "flex w-full cursor-pointer items-center gap-2.5 rounded-md px-2.5 py-1.5 text-left text-[13px] outline-none transition-colors disabled:pointer-events-none disabled:opacity-40",
                      it.danger ? "text-danger hover:bg-danger-soft focus:bg-danger-soft" : "text-text hover:bg-surface-2 focus:bg-surface-2",
                    )}
                  >
                    {it.icon ? <it.icon size={15} className="shrink-0 text-muted" /> : <span className="w-[15px]" />}
                    <span className="flex-1">{it.label}</span>
                    {it.shortcut && <kbd className="font-sans text-[11px] text-faint">{it.shortcut}</kbd>}
                  </button>
                ),
              )}
            </motion.div>
          )}
        </AnimatePresence>,
        document.body,
      )}
    </>
  );
}
