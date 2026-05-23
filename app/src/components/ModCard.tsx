import type { ModInfo } from "../types/mod";
import type { ReactNode } from "react";

interface ModCardProps {
  mod: ModInfo;
  tracked: boolean;
  onToggle: (mod: ModInfo) => void;
  onDelete: (mod: ModInfo) => void;
  onConfigure: (mod: ModInfo) => void;
}

export function ModCard({ mod, tracked, onToggle, onDelete, onConfigure }: ModCardProps) {
  const isExternal = mod.source === "external_package" || mod.source === "external_native";
  const hasConfig = mod.configFiles.length > 0;

  return (
    <article className={`group rounded-lg border bg-panel p-4 transition-colors hover:border-accent/45 ${mod.enabled ? "border-accent/35" : "border-border"}`}>
      <div className="flex items-start justify-between gap-4">
        <div className="min-w-0 flex-1">
          <div className="mb-3 flex flex-wrap items-center gap-2">
            <Badge tone={mod.type === "native" ? "warning" : "accent"}>
              {mod.type === "native" ? "DLL" : "资源包"}
            </Badge>
            <Badge tone={mod.enabled ? "success" : "muted"}>{mod.enabled ? "启用" : "停用"}</Badge>
            {tracked && <Badge tone="info">当前方案</Badge>}
            {isExternal && <Badge tone="info">外部</Badge>}
            {mod.source === "game_native" && <Badge tone="info">Game\\mods</Badge>}
          </div>

          <h3 className="truncate text-base font-semibold text-text-primary">{mod.name}</h3>
          <p className="mt-2 line-clamp-2 min-h-10 text-sm leading-5 text-text-secondary">
            {mod.description || "未提供说明。启动前建议确认 Mod 目录结构和依赖项。"}
          </p>

          <div className="mt-4 flex flex-wrap items-center gap-2 text-xs text-text-muted">
            {mod.version && <span className="rounded-md bg-surface px-2 py-1">v{mod.version}</span>}
            <span className="rounded-md bg-surface px-2 py-1">{mod.files.length} 个顶层项</span>
            <span className="max-w-full truncate rounded-md bg-surface px-2 py-1">{mod.id}</span>
          </div>
        </div>

        <div className="flex shrink-0 items-center gap-2">
          {hasConfig && (
            <button
              type="button"
              onClick={() => onConfigure(mod)}
              className="grid h-9 w-9 place-items-center rounded-lg text-text-muted opacity-80 transition-colors hover:bg-accent/15 hover:text-accent group-hover:opacity-100"
              title="编辑配置"
            >
              <ConfigIcon />
            </button>
          )}
          <button
            type="button"
            onClick={() => onDelete(mod)}
            className="grid h-9 w-9 place-items-center rounded-lg text-text-muted opacity-80 transition-colors hover:bg-danger/15 hover:text-danger group-hover:opacity-100"
            title={isExternal ? "移除外部注册" : "删除"}
          >
            <TrashIcon />
          </button>
          <button
            type="button"
            onClick={() => onToggle(mod)}
            className={`relative h-7 w-12 rounded-full border transition-colors ${mod.enabled ? "border-accent bg-accent" : "border-border bg-surface"}`}
            aria-label={mod.enabled ? "停用 Mod" : "启用 Mod"}
          >
            <span
              className={`absolute top-0.5 h-6 w-6 rounded-full bg-white shadow transition-transform ${mod.enabled ? "translate-x-[20px]" : "translate-x-0.5"}`}
            />
          </button>
        </div>
      </div>
    </article>
  );
}

function Badge({
  tone,
  children,
}: {
  tone: "accent" | "success" | "warning" | "muted" | "info";
  children: ReactNode;
}) {
  const className = {
    accent: "bg-accent-soft text-accent",
    success: "bg-success/15 text-success",
    warning: "bg-warning/15 text-warning",
    muted: "bg-surface text-text-muted",
    info: "border border-border bg-surface text-text-secondary",
  }[tone];

  return <span className={`rounded-md px-2 py-1 text-xs font-semibold ${className}`}>{children}</span>;
}

function ConfigIcon() {
  return (
    <svg className="h-4 w-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
      <path d="M12 20h9" />
      <path d="M16.5 3.5a2.1 2.1 0 0 1 3 3L7 19l-4 1 1-4L16.5 3.5z" />
    </svg>
  );
}

function TrashIcon() {
  return (
    <svg className="h-4 w-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
      <path d="M3 6h18M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2" />
    </svg>
  );
}
