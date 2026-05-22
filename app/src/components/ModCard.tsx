import type { ModInfo } from "../types/mod";

interface ModCardProps {
  mod: ModInfo;
  onToggle: (mod: ModInfo) => void;
  onDelete: (mod: ModInfo) => void;
}

export function ModCard({ mod, onToggle, onDelete }: ModCardProps) {
  return (
    <div
      className={`group bg-bg-secondary border rounded-lg p-4 transition-all duration-200 hover:border-accent/30 ${
        mod.enabled ? "border-accent/20" : "border-border"
      }`}
    >
      <div className="flex items-start justify-between gap-3">
        <div className="min-w-0 flex-1">
          <div className="flex items-center gap-2 mb-1">
            <span
              className={`text-xs px-1.5 py-0.5 rounded ${
                mod.type === "native"
                  ? "bg-warning/15 text-warning"
                  : "bg-accent-dim text-accent"
              }`}
            >
              {mod.type === "native" ? "DLL" : "资源包"}
            </span>
            <h3 className="text-sm font-medium text-text-primary truncate">
              {mod.name}
            </h3>
          </div>

          {mod.description && (
            <p className="text-xs text-text-muted mb-2 line-clamp-2">
              {mod.description}
            </p>
          )}

          <div className="flex items-center gap-3 text-xs text-text-muted">
            {mod.version && <span>v{mod.version}</span>}
            <span>{mod.files.length} 个文件</span>
          </div>
        </div>

        <div className="flex items-center gap-1 shrink-0">
          <button
            onClick={() => onDelete(mod)}
            className="w-8 h-8 flex items-center justify-center rounded opacity-0 group-hover:opacity-100 hover:bg-danger/15 text-text-muted hover:text-danger transition-all"
            title="删除"
          >
            <svg
              className="w-4 h-4"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              strokeWidth="2"
            >
              <path d="M3 6h18M19 6v14a2 2 0 01-2 2H7a2 2 0 01-2-2V6m3 0V4a2 2 0 012-2h4a2 2 0 012 2v2" />
            </svg>
          </button>

          <button
            onClick={() => onToggle(mod)}
            className={`relative w-11 h-6 rounded-full transition-colors duration-200 ${
              mod.enabled ? "bg-accent" : "bg-bg-tertiary"
            }`}
          >
            <div
              className={`absolute top-0.5 w-5 h-5 rounded-full bg-white shadow transition-transform duration-200 ${
                mod.enabled ? "translate-x-5.5" : "translate-x-0.5"
              }`}
            />
          </button>
        </div>
      </div>
    </div>
  );
}
