// Toast notification system
import { useState, useCallback, createContext, useContext, useRef, type ReactNode } from 'react';

interface Toast {
  id: string;
  title: string;
  description?: string;
  variant?: 'default' | 'destructive';
}

interface ToastCtx {
  toasts: Toast[];
  toast: (t: Omit<Toast, 'id'>) => void;
  dismiss: (id: string) => void;
}

const ToastContext = createContext<ToastCtx | null>(null);

let globalToast: ToastCtx['toast'] = (t) => console.log('Toast:', t.title, t.description);
let globalDismiss: ToastCtx['dismiss'] = () => {};

export function useToast() {
  const ctx = useContext(ToastContext);
  if (ctx) return ctx;
  // Fallback for usage outside provider — uses the global ref set by Toaster
  return { toast: globalToast, dismiss: globalDismiss, toasts: [] as Toast[] };
}

let idCounter = 0;

export function ToastProvider({ children }: { children: ReactNode }) {
  const [toasts, setToasts] = useState<Toast[]>([]);
  const timersRef = useRef<Map<string, ReturnType<typeof setTimeout>>>(new Map());

  const dismiss = useCallback((id: string) => {
    setToasts((prev) => prev.filter((t) => t.id !== id));
    const timer = timersRef.current.get(id);
    if (timer) {
      clearTimeout(timer);
      timersRef.current.delete(id);
    }
  }, []);

  const toast = useCallback(
    (t: Omit<Toast, 'id'>) => {
      const id = `toast_${++idCounter}`;
      setToasts((prev) => [...prev, { ...t, id }]);
      // Auto-dismiss after 5 seconds
      const timer = setTimeout(() => {
        setToasts((prev) => prev.filter((x) => x.id !== id));
        timersRef.current.delete(id);
      }, 5000);
      timersRef.current.set(id, timer);
    },
    [],
  );

  // Keep global refs updated so useToast works outside the provider tree too
  globalToast = toast;
  globalDismiss = dismiss;

  return (
    <ToastContext.Provider value={{ toasts, toast, dismiss }}>
      {children}
    </ToastContext.Provider>
  );
}

export function Toaster() {
  const { toasts, dismiss } = useToast();

  if (toasts.length === 0) return null;

  return (
    <div className="fixed bottom-4 right-4 z-[100] flex flex-col gap-2 pointer-events-none">
      {toasts.map((t) => (
        <div
          key={t.id}
          className={`pointer-events-auto relative rounded-lg border p-4 pr-8 shadow-lg max-w-sm animate-in slide-in-from-bottom-5 ${
            t.variant === 'destructive'
              ? 'border-destructive bg-destructive text-destructive-foreground'
              : 'bg-background border-border'
          }`}
        >
          <div className="font-semibold text-sm">{t.title}</div>
          {t.description && (
            <div className="text-xs text-muted-foreground mt-1">{t.description}</div>
          )}
          <button
            onClick={() => dismiss(t.id)}
            className="absolute top-2 right-2 text-xs opacity-50 hover:opacity-100"
          >
            ✕
          </button>
        </div>
      ))}
    </div>
  );
}
