// Reclaim: Tooltip shown on hover/focus, rendered in a portal so it is never clipped.
import { cloneElement, useRef, useState, type ReactElement, type ReactNode } from "react";
import { createPortal } from "react-dom";
import { AnimatePresence, motion } from "motion/react";

export function Tooltip({ content, children }: { content: ReactNode; children: ReactElement<Record<string, unknown>> }) {
  const ref = useRef<HTMLElement | null>(null);
  const timer = useRef<number | undefined>(undefined);
  const [pos, setPos] = useState<{ x: number; y: number } | null>(null);

  const show = () => {
    window.clearTimeout(timer.current);
    timer.current = window.setTimeout(() => {
      const r = ref.current?.getBoundingClientRect();
      if (r) setPos({ x: r.left + r.width / 2, y: r.bottom + 8 });
    }, 350);
  };
  const hide = () => { window.clearTimeout(timer.current); setPos(null); };

  if (!content) return children;
  const child = cloneElement(children, {
    ref: (el: HTMLElement | null) => { ref.current = el; },
    onMouseEnter: show, onMouseLeave: hide, onFocus: show, onBlur: hide, onMouseDown: hide,
  });
  return (
    <>
      {child}
      {createPortal(
        <AnimatePresence>
          {pos && (
            <motion.div
              initial={{ opacity: 0, y: -2 }} animate={{ opacity: 1, y: 0 }} exit={{ opacity: 0 }} transition={{ duration: 0.12 }}
              style={{ position: "fixed", left: pos.x, top: pos.y, translateX: "-50%", zIndex: 100 }}
              className="pointer-events-none max-w-xs rounded-md border border-border bg-surface px-2 py-1 text-xs text-text shadow-md"
            >
              {content}
            </motion.div>
          )}
        </AnimatePresence>,
        document.body,
      )}
    </>
  );
}
