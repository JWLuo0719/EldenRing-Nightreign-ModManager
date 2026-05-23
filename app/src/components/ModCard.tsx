import type { ModInfo } from "../types/mod";

interface ModCardProps {
  mod: ModInfo;
  tracked: boolean;
  onToggle: (mod: ModInfo) => void;
  onDelete: (mod: ModInfo) => void;
}

export function ModCard({ mod, tracked, onToggle, onDelete }: ModCardProps) {
  return (
    <article
      className={`group relative overflow-hidden rounded-3xl border bg-bg-secondary/90 p-5 transition-all duration-200 hover:-translate-y-0.5 hover:border-accent/45 hover:shadow-xl hover:shadow-black/20 ${
        mod.enabled ? "border-accent/35" : "border-border/80"
      }`}
    >
      <div
        className={`absolute left-0 top-0 h-full w-1 ${
          mod.enabled ? "bg-accent" : "bg-border"
        }`}
      />

      <div className="flex items-start justify-between gap-4">
        <div className="min-w-0 flex-1">
          <div className="mb-3 flex flex-wrap items-center gap-2">
            <span
              className={`rounded-lg px-2.5 py-1 text-xs font-black ${
                mod.type === "native"
                  ? "bg-warning/15 text-warning"
                  : "bg-accent-dim text-accent"
              }`}
            >
              {mod.type === "native" ? "DLL" : "资源包"}
            </span>
            {tracked && (
              <span className="rounded-lg border border-success/25 bg-success/10 px-2.5 py-1 text-xs font-semibold text-success">
                当前方案
              </span>
            )}
            <span
              className={`rounded-lg px-2.5 py-1 text-xs font-semibold ${
                mod.enabled
                  ? "bg-success/10 text-success"
                  : "bg-bg-tertiary text-text-muted"
              }`}
            >
              {mod.enabled ? "启用" : "停用"}
            </span>
          </div>

          <h3 className="truncate text-lg font-black tracking-tight text-text-primary">
            {mod.name}
          </h3>

          <p className="mt-2 line-clamp-2 min-h-10 text-sm leading-5 text-text-secondary">
            {mod.description || "未提供说明。建议启动前确认 Mod 目录结构和依赖项。"}
          </p>

          <div className="mt-4 flex flex-wrap items-center gap-3 text-xs text-text-muted">
            {mod.version && (
              <span className="rounded-md bg-bg-tertiary px-2 py-1">
                v{mod.version}
              </span>
            )}
            <span className="rounded-md bg-bg-tertiary px-2 py-1">
              {mod.files.length} 个顶层项
            </span>
            <span className="max-w-full truncate rounded-md bg-bg-tertiary px-2 py-1">
              {mod.id}
            </span>
          </div>
        </div>

        <div className="flex shrink-0 items-center gap-2">
          <button
            onClick={() => onDelete(mod)}
            className="grid h-9 w-9 place-items-center rounded-xl border border-transparent text-text-muted opacity-70 transition-all hover:border-danger/30 hover:bg-danger/15 hover:text-danger group-hover:opacity-100"
            title="删除"
          >
            <svg
              className="h-4 w-4"
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
            className={`relative h-7 w-12 rounded-full border transition-colors duration-200 ${
              mod.enabled
                ? "border-accent bg-accent"
                : "border-border bg-bg-tertiary"
            }`}
            aria-label={mod.enabled ? "禁用 Mod" : "启用 Mod"}
          >
            <span
              className={`absolute top-0.5 h-6 w-6 rounded-full bg-white shadow transition-transform duration-200 ${
                mod.enabled ? "translate-x-[20px]" : "translate-x-0.5"
              }`}
            />
          </button>
        </div>
      </div>
    </article>
  );
}
