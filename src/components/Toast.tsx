// Reclaim: ToastProvider (mounted by the shell) and useToast() for bottom-right notifications.
import { createContext, useCallback, useContext, useMemo, useState, type ReactNode } from "react";
import { AnimatePresence, motion } from "motion/react";
import { AlertCircle, CheckCircle2, Info, X } from "lucide-react";

type Tone = "success" | "error" | "info";
interface ToastOpts { title: string; description?: string; tone?: Tone }
interface ToastItem extends ToastOpts { id: number }

const Ctx = createContext<{ toast: (o: ToastOpts) => void }>({ toast: () => {} });
export const useToast = () => useContext(Ctx);

const icons = { success: CheckCircle2, error: AlertCircle, info: Info };
const colors = { success: "var(--success)", error: "var(--danger)", info: "var(--info)" };
let nextId = 1;

export function ToastProvider({ children }: { children: ReactNode }) {
  const [items, setItems] = useState<ToastItem[]>([]);
  const dismiss = useCallback((id: number) => setItems((l) => l.filter((t) => t.id !== id)), []);
  const toast = useCallback((o: ToastOpts) => {
    const id = nextId++;
    setItems((l) => [...l.slice(-3), { ...o, id }]);
    window.setTimeout(() => dismiss(id), o.tone === "error" ? 7000 : 4000);
  }, [dismiss]);
  const value = useMemo(() => ({ toast }), [toast]);

  return (
    <Ctx.Provider value={value}>
      {children}
      <div className="pointer-events-none fixed bottom-4 right-4 z-[60] flex w-80 flex-col gap-2">
        <AnimatePresence initial={false}>
          {items.map((t) => {
            const Icon = icons[t.tone ?? "info"];
            return (
              <motion.div
                key={t.id} layout
                initial={{ opacity: 0, x: 24, scale: 0.97 }} animate={{ opacity: 1, x: 0, scale: 1 }} exit={{ opacity: 0, x: 24 }}
                transition={{ type: "spring", stiffness: 420, damping: 34 }}
                className="pointer-events-auto flex gap-3 rounded-xl border border-border bg-surface p-3 shadow-lg"
              >
                <Icon size={18} className="mt-0.5 shrink-0" style={{ color: colors[t.tone ?? "info"] }} />
                <div className="min-w-0 flex-1">
                  <div className="text-sm font-medium text-text">{t.title}</div>
                  {t.description && <div className="mt-0.5 break-words text-xs text-muted">{t.description}</div>}
                </div>
                <button aria-label="Dismiss" onClick={() => dismiss(t.id)} className="h-5 w-5 shrink-0 cursor-pointer text-faint hover:text-text"><X size={14} /></button>
              </motion.div>
            );
          })}
        </AnimatePresence>
      </div>
    </Ctx.Provider>
  );
}
