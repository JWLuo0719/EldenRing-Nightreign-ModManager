import type { ConfirmState, Toast } from "../types/mod";

export function ToastHost({ toasts }: { toasts: Toast[] }) {
  return (
    <div className="pointer-events-none fixed bottom-5 right-5 z-50 flex w-96 max-w-[calc(100vw-2rem)] flex-col gap-2">
      {toasts.map((toast) => (
        <div
          key={toast.id}
          className={`pointer-events-auto rounded-lg border px-4 py-3 text-sm shadow-2xl ${
            toast.type === "success"
              ? "border-success/35 bg-success/15 text-success"
              : toast.type === "error"
                ? "border-danger/35 bg-danger/15 text-danger"
                : "border-accent/35 bg-accent-soft text-accent"
          }`}
        >
          {toast.message}
        </div>
      ))}
    </div>
  );
}

export function ConfirmDialog({
  state,
  onCancel,
  onConfirm,
}: {
  state: ConfirmState;
  onCancel: () => void;
  onConfirm: () => void;
}) {
  return (
    <div className="fixed inset-0 z-40 flex items-center justify-center bg-black/60 px-4 backdrop-blur-sm">
      <div className="w-full max-w-md rounded-xl border border-border bg-panel p-6 shadow-2xl">
        <div className="text-lg font-semibold text-text-primary">{state.title}</div>
        <p className="mt-3 max-h-[50vh] overflow-auto whitespace-pre-wrap text-sm leading-6 text-text-secondary">
          {state.message}
        </p>
        <div className="mt-6 flex justify-end gap-3">
          <button
            type="button"
            onClick={onCancel}
            className="rounded-lg border border-border px-4 py-2 text-sm font-medium text-text-secondary transition-colors hover:bg-surface hover:text-text-primary"
          >
            取消
          </button>
          <button
            type="button"
            onClick={onConfirm}
            className={`rounded-lg px-4 py-2 text-sm font-semibold text-white transition-colors ${
              state.danger ? "bg-danger hover:bg-danger/85" : "bg-accent text-black hover:bg-accent-hover"
            }`}
          >
            {state.confirmText}
          </button>
        </div>
      </div>
    </div>
  );
}
