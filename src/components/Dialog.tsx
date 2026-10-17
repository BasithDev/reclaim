// Reclaim: modal Dialog and ConfirmDialog (portal, backdrop, Esc to close, animated).
import { useEffect, type ReactNode } from "react";
import { createPortal } from "react-dom";
import { AnimatePresence, motion } from "motion/react";
import { X } from "lucide-react";
import { Button, IconButton } from "./Button";
import { cn } from "./cn";

const widths = { sm: "max-w-sm", md: "max-w-lg", lg: "max-w-2xl", xl: "max-w-4xl" };

export function Dialog({ open, onClose, title, children, footer, size = "md" }: {
  open: boolean; onClose: () => void; title: string; children?: ReactNode; footer?: ReactNode; size?: "sm" | "md" | "lg" | "xl";
}) {
  useEffect(() => {
    if (!open) return;
    const onKey = (e: KeyboardEvent) => { if (e.key === "Escape") { e.stopImmediatePropagation(); onClose(); } };
    window.addEventListener("keydown", onKey, true);
    return () => window.removeEventListener("keydown", onKey, true);
  }, [open, onClose]);

  return createPortal(
    <AnimatePresence>
      {open && (
        <motion.div
          className="fixed inset-0 z-50 flex items-center justify-center p-6"
          initial={{ opacity: 0 }} animate={{ opacity: 1 }} exit={{ opacity: 0 }} transition={{ duration: 0.15 }}
        >
          <div className="absolute inset-0 bg-overlay backdrop-blur-[2px]" onClick={onClose} />
          <motion.div
            role="dialog" aria-modal="true" aria-label={title}
            initial={{ opacity: 0, scale: 0.96, y: 8 }} animate={{ opacity: 1, scale: 1, y: 0 }} exit={{ opacity: 0, scale: 0.97, y: 4 }}
            transition={{ type: "spring", stiffness: 420, damping: 34 }}
            className={cn("relative flex max-h-[85vh] w-full flex-col rounded-xl border border-border bg-surface shadow-lg", widths[size])}
          >
            <div className="flex items-center justify-between gap-4 px-5 pt-4 pb-2">
              <h2 className="text-base font-semibold text-text">{title}</h2>
              <IconButton icon={X} label="Close" size="sm" onClick={onClose} />
            </div>
            <div className="selectable min-h-0 flex-1 overflow-y-auto px-5 py-2 text-sm text-muted">{children}</div>
            {footer && <div className="flex justify-end gap-2 px-5 pb-4 pt-3">{footer}</div>}
          </motion.div>
        </motion.div>
      )}
    </AnimatePresence>,
    document.body,
  );
}

export function ConfirmDialog({ open, onClose, onConfirm, title, message, confirmLabel = "Confirm", tone = "default", loading }: {
  open: boolean; onClose: () => void; onConfirm: () => void; title: string; message: ReactNode;
  confirmLabel?: string; tone?: "default" | "danger"; loading?: boolean;
}) {
  return (
    <Dialog
      open={open} onClose={onClose} title={title} size="sm"
      footer={<>
        <Button variant="ghost" onClick={onClose} disabled={loading}>Cancel</Button>
        <Button variant={tone === "danger" ? "danger" : "primary"} onClick={onConfirm} loading={loading}>{confirmLabel}</Button>
      </>}
    >
      {message}
    </Dialog>
  );
}
