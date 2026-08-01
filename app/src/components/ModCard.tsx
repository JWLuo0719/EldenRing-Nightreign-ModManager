import type { ModInfo } from "../types/mod";
import type { ReactNode } from "react";
import { playerTerms } from "../lib/terminology";

interface ModCardProps {
  mod: ModInfo;
  tracked: boolean;
  onToggle: (mod: ModInfo) => void;
  onDelete: (mod: ModInfo) => void;
  onConfigure: (mod: ModInfo) => void;
  onProfileMode: (mod: ModInfo) => void;
  onRelink: (mod: ModInfo) => void;
}

export function ModCard({
  mod,
  tracked,
  onToggle,
  onDelete,
  onConfigure,
  onProfileMode,
  onRelink,
}: ModCardProps) {
  const isExternal = mod.source === "external_package" || mod.source === "external_native";
  const hasConfig = mod.configFiles.length > 0;
  const canChangeProfileMode =
    mod.source === "external_package" &&
    mod.authorProfile &&
    (mod.networkBackend === "server_redirector" ||
      mod.profileMode === "mmv_seamless_community");
  const clothing = mod.clothing;

  return (
    <article className={`group relative overflow-hidden rounded-lg border bg-surface/55 p-4 transition-all hover:-translate-y-px hover:border-accent/45 hover:bg-surface/80 ${mod.enabled ? "border-accent/30" : "border-border"}`}>
      {mod.enabled && <span className="absolute inset-y-3 left-0 w-0.5 rounded-full bg-accent" />}
      <div className="flex items-start justify-between gap-4">
        <div className="min-w-0 flex-1">
          <div className="mb-3 flex flex-wrap items-center gap-2">
            <Badge tone={mod.type === "native" ? "warning" : "accent"}>
              {mod.type === "native" ? playerTerms.native : playerTerms.package}
            </Badge>
            <Badge tone={mod.enabled ? "success" : "muted"}>{mod.enabled ? "启用" : "停用"}</Badge>
            {tracked && <Badge tone="info">当前方案</Badge>}
            {isExternal && <Badge tone="info">外部</Badge>}
            {!mod.pathAvailable && <Badge tone="warning">原文件夹已失效</Badge>}
            {mod.authorProfile && <Badge tone="accent">作者启动配置</Badge>}
            {clothing.detected && (
              <Badge tone={clothing.requiresAppearanceReset ? "warning" : "info"}>
                {clothing.kind === "expanded" ? "扩展服装" : "服装替换"}
              </Badge>
            )}
            {clothing.onlineSupport === "complete" && <Badge tone="success">队友视角完整</Badge>}
            {(clothing.onlineSupport === "missing" || clothing.onlineSupport === "partial") && (
              <Badge tone="warning">队友视角待检查</Badge>
            )}
            {mod.profileMode === "mmv_seamless_community" && (
              <Badge tone="warning">社区 Seamless</Badge>
            )}
            {mod.networkBackend === "server_redirector" && (
              <Badge tone="success">Server Redirector</Badge>
            )}
            {mod.networkBackend === "seamless" && <Badge tone="info">Seamless</Badge>}
            {mod.source === "game_native" && <Badge tone="info">Game\\mods</Badge>}
          </div>

          <h3 className="truncate text-[15px] font-semibold text-text-primary">{mod.name}</h3>
          <p className="mt-2 line-clamp-2 min-h-10 text-sm leading-5 text-text-secondary">
            {mod.description || "未提供说明。启用前请先查看结构检查和依赖提示。"}
          </p>

          {!mod.pathAvailable && (
            <div className="mt-3 rounded-lg border border-danger/30 bg-danger/10 px-3 py-2 text-xs leading-5 text-danger">
              <div className="font-semibold">管理器找不到这个 Mod 的原文件夹</div>
              <div className="mt-0.5 opacity-90">
                它不会进入本次启动配置。文件夹可能被改名或移动；请重新选择现在的位置。
              </div>
              {mod.source === "external_package" && (
                <button
                  type="button"
                  onClick={() => onRelink(mod)}
                  className="mt-2 rounded-lg border border-danger/40 bg-black/15 px-3 py-1.5 font-semibold transition-colors hover:bg-black/25"
                >
                  重新定位文件夹
                </button>
              )}
            </div>
          )}

          {clothing.detected && (
            <div
              className={`mt-3 rounded-lg border px-3 py-2 text-xs leading-5 ${
                clothing.requiresAppearanceReset
                  ? "border-danger/30 bg-danger/10 text-danger"
                  : clothing.onlineSupport === "complete"
                    ? "border-success/25 bg-success/10 text-success"
                    : "border-warning/25 bg-warning/10 text-warning"
              }`}
            >
              <div className="font-semibold">
                {clothing.requiresAppearanceReset
                  ? "停用前先换回本体服装"
                  : clothing.onlineSupport === "complete"
                    ? "本机与队友视角资源已配对"
                    : "联机时队友看到的外观可能不完整"}
              </div>
              <div className="mt-0.5 opacity-90">
                {clothing.requiresAppearanceReset
                  ? "此 Mod 含新增服装数据。若存档仍选中扩展服装，停用后人物可能不显示或无法切回。"
                  : clothing.onlineSupport === "complete"
                    ? `已识别 ${clothing.pairedPartFileCount} 组 _l 队友视角资源。双方仍需使用相同文件。`
                    : `缺少 ${clothing.missingOnlinePartCount} 个 _l 配对；管理器不会自动运行包内脚本。`}
              </div>
            </div>
          )}

          <div className="mt-4 flex flex-wrap items-center gap-2 text-xs text-text-muted">
            {mod.version && <span className="rounded-md bg-elevated px-2 py-1">v{mod.version}</span>}
            {mod.savefile && (
              <span className="rounded-md bg-elevated px-2 py-1">存档 {mod.savefile}</span>
            )}
            {mod.startOnline !== null && (
              <span className="rounded-md bg-elevated px-2 py-1">
                {mod.startOnline ? "在线启动" : "离线启动"}
              </span>
            )}
            <span className="rounded-md bg-elevated px-2 py-1">{mod.files.length} 个顶层项</span>
            <span className="max-w-full truncate rounded-md bg-elevated px-2 py-1">{mod.id}</span>
          </div>

          <details className="mt-3 rounded-lg border border-border/80 bg-elevated/45 px-3 py-2 text-xs text-text-muted">
            <summary className="cursor-pointer select-none font-semibold text-text-secondary">
              高级详情
            </summary>
            <div className="mt-2 grid gap-1 leading-5">
              <div>加载类型：{mod.type === "native" ? "native DLL" : "package"}</div>
              {mod.authorProfile && <div>作者文件：ME3 Profile（.me3）</div>}
              {clothing.detected && (
                <>
                  <div>
                    parts：本机 {clothing.localPartFileCount} / _l 队友视角 {clothing.onlinePartFileCount} / 配对 {clothing.pairedPartFileCount}
                  </div>
                  <div>regulation.bin：{clothing.hasRegulation ? "包含" : "未发现"}</div>
                  {clothing.appearanceIds.length > 0 && (
                    <div>
                      外观 ID：{clothing.appearanceIds.slice(0, 12).join(", ")}
                      {clothing.appearanceIds.length > 12 ? ` 等 ${clothing.appearanceIds.length} 项` : ""}
                    </div>
                  )}
                  {clothing.hasManualOnlineSetup && (
                    <div>检测到联机准备脚本：仅提示，管理器不会执行</div>
                  )}
                </>
              )}
              <div className="truncate" title={mod.path}>路径：{mod.path}</div>
            </div>
          </details>

          {canChangeProfileMode && (
            <button
              type="button"
              onClick={() => onProfileMode(mod)}
              className="mt-3 rounded-lg border border-warning/35 bg-warning/10 px-3 py-1.5 text-xs font-semibold text-warning transition-colors hover:border-warning/60 hover:bg-warning/15"
            >
              {mod.profileMode === "mmv_seamless_community"
                ? "恢复作者 Server Redirector"
                : "改用社区 Seamless 兼容"}
            </button>
          )}
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
            disabled={!mod.pathAvailable}
            className={`relative h-7 w-12 rounded-full border transition-colors ${mod.enabled ? "border-accent bg-accent" : "border-border bg-surface"}`}
            aria-label={mod.enabled ? "停用 Mod" : "启用 Mod"}
            title={mod.pathAvailable ? undefined : "请先重新定位原文件夹"}
          >
            <span
              className={`absolute left-0 top-0.5 h-6 w-6 rounded-full shadow transition-all ${
                mod.enabled
                  ? "translate-x-[20px] bg-white"
                  : "translate-x-0.5 bg-text-muted"
              }`}
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
