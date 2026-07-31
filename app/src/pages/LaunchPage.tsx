import { useState } from "react";
import type {
  LaunchPreflight,
  LaunchPreflightCheck,
  ModInfo,
  Profile,
  RuntimeEnvironmentStatus,
  SpecialModStatus,
} from "../types/mod";
import type { ReactNode } from "react";

interface LaunchPageProps {
  mods: ModInfo[];
  activeProfile: Profile | null;
  gamePath: string;
  me3Path: string;
  launchExePath: string;
  specialModStatus: SpecialModStatus | null;
  runtimeEnvironmentStatus: RuntimeEnvironmentStatus | null;
  busy: boolean;
  onLaunch: () => void;
  onPreflight: () => Promise<LaunchPreflight | undefined>;
  onRefresh: () => void;
  onOpenDiagnostics: () => void;
  onPrepareOnline: () => void;
  onRestoreOnline: () => void;
}

export function LaunchPage({
  mods,
  activeProfile,
  gamePath,
  me3Path,
  launchExePath,
  specialModStatus,
  runtimeEnvironmentStatus,
  busy,
  onLaunch,
  onPreflight,
  onRefresh,
  onOpenDiagnostics,
  onPrepareOnline,
  onRestoreOnline,
}: LaunchPageProps) {
  const [preflight, setPreflight] = useState<LaunchPreflight | null>(null);
  const enabledMods = mods.filter((mod) => mod.enabled);
  const launchTarget = launchExePath.trim()
    ? launchExePath.split(/[\\/]/).pop() || "自定义启动程序"
    : "nightreign.exe";
  const serverRedirectorMod = enabledMods.find(
    (mod) => mod.networkBackend === "server_redirector"
  );
  const usingServerRedirector = Boolean(serverRedirectorMod);
  const redirectorEnvironmentConflict = Boolean(
    usingServerRedirector && specialModStatus?.serverRedirectorConflicts.length
  );
  const runtimeEnvironment = runtimeEnvironmentStatus?.effective ?? "unknown_mixed";
  const canInstallOnlinePatch =
    runtimeEnvironment === "spacewar_seamless" && !usingServerRedirector;
  const onlineReady = Boolean(
    usingServerRedirector
      ? runtimeEnvironment === "steam_official" && !redirectorEnvironmentConflict
      : runtimeEnvironment === "steam_official"
        ? !specialModStatus?.serverRedirectorConflicts.length &&
          !specialModStatus?.seamlessInstalled
        : runtimeEnvironment === "steam_seamless"
          ? specialModStatus?.seamlessInstalled &&
            !specialModStatus.serverRedirectorConflicts.length
          : runtimeEnvironment === "spacewar_seamless"
            ? specialModStatus?.seamlessInstalled &&
              specialModStatus.onlinefixInstalled
            : false
  );

  const runPreflight = async () => {
    const result = await onPreflight();
    if (result) {
      setPreflight(result);
    }
  };

  return (
    <PageFrame
      eyebrow="Launch Center"
      title="启动台"
      description="从这里确认环境状态、生成当前方案并通过 ME3 启动 Nightreign。"
    >
      <div className="min-h-0 flex-1 overflow-y-auto pr-1">
        <div className="grid gap-4 lg:grid-cols-[minmax(0,1fr)_19rem]">
          <section className="panel-card relative overflow-hidden rounded-xl p-5">
            <div className="absolute inset-x-0 top-0 h-px bg-gradient-to-r from-accent via-accent/35 to-transparent" />
            <div className="section-label">Ready to deploy</div>
            <div className="mt-3 flex flex-wrap items-end justify-between gap-4">
              <div className="min-w-0">
                <h2 className="truncate text-2xl font-bold tracking-tight text-text-primary">
                  {activeProfile?.name ?? "全局启用状态"}
                </h2>
                <p className="mt-2 text-sm leading-6 text-text-secondary">
                  将生成 ME3 Profile，并以 <span className="font-semibold text-text-primary">{launchTarget}</span> 启动。
                  {usingServerRedirector &&
                    " 当前方案将保留作者配置，使用 Server Redirector 和正版 Steam 环境。"}
                </p>
              </div>
              <div className="flex items-baseline gap-2 text-text-muted">
                <span className="display-number text-4xl font-semibold text-accent">{enabledMods.length}</span>
                <span className="text-xs">个 Mod 待加载</span>
              </div>
            </div>

            <div className="mt-5 grid gap-2 sm:grid-cols-3">
              <Metric label="启用 / 全部" value={`${enabledMods.length} / ${mods.length}`} />
              <Metric label="资源包" value={String(mods.filter((mod) => mod.type === "package").length)} />
              <Metric label="原生 DLL" value={String(mods.filter((mod) => mod.type === "native").length)} />
            </div>

            <div className="mt-5 grid gap-2 sm:grid-cols-2">
              <StatusRow label="游戏目录" value={gamePath || "未配置"} ok={Boolean(gamePath)} />
              <StatusRow label="ME3 目录" value={me3Path || "未配置"} ok={Boolean(me3Path)} />
            </div>
          </section>

          <section className="panel-card rounded-xl p-4">
            <div className="flex items-center justify-between">
              <div>
                <div className="section-label text-text-muted">Command</div>
                <h3 className="mt-1 text-base font-semibold text-text-primary">启动控制</h3>
              </div>
              <button
                type="button"
                disabled={busy}
                onClick={onRefresh}
                className="rounded-md px-2 py-1 text-xs font-medium text-text-muted transition-colors hover:bg-surface hover:text-text-primary disabled:opacity-50"
              >
                刷新状态
              </button>
            </div>
            <button
              type="button"
              disabled={busy}
              onClick={onLaunch}
              className="mt-4 flex w-full items-center justify-between rounded-lg bg-accent px-4 py-3.5 text-left text-sm font-bold text-black transition-all hover:bg-accent-hover disabled:cursor-not-allowed disabled:opacity-50"
            >
              <span>启动 Nightreign</span>
              <PlayArrow />
            </button>
            <div className="mt-2 grid gap-2">
              <button
                type="button"
                disabled={busy}
                onClick={() => void runPreflight()}
                className="rounded-lg border border-accent/35 bg-accent-soft px-3.5 py-2.5 text-left text-sm font-semibold text-accent transition-colors hover:border-accent/65 disabled:opacity-50"
              >
                启动前检查 <span className="font-normal text-text-muted">· 不启动游戏</span>
              </button>
              <div className="grid grid-cols-2 gap-2">
                <button
                  type="button"
                  onClick={onOpenDiagnostics}
                  className="rounded-lg border border-border bg-surface px-3 py-2.5 text-left text-xs text-text-secondary transition-colors hover:border-border-strong hover:text-text-primary"
                >
                  打开诊断
                </button>
                <button
                  type="button"
                  disabled={busy || !canInstallOnlinePatch}
                  onClick={onPrepareOnline}
                  title={
                    usingServerRedirector
                      ? "MMV Server Redirector 不能与 OnlineFix 共用"
                      : !canInstallOnlinePatch
                        ? "只有 Spacewar + Seamless 环境允许安装 OnlineFix 补丁"
                        : undefined
                  }
                  className="rounded-lg border border-border bg-surface px-3 py-2.5 text-left text-xs text-text-secondary transition-colors hover:border-border-strong hover:text-text-primary disabled:opacity-50"
                >
                  {usingServerRedirector
                    ? "MMV 不用此补丁"
                    : canInstallOnlinePatch
                      ? "应用联机补丁"
                      : "当前环境禁用补丁"}
                </button>
              </div>
              {specialModStatus?.latestPatchBackup && (
                <button
                  type="button"
                  disabled={busy}
                  onClick={onRestoreOnline}
                  className="rounded-lg border border-warning/35 bg-warning/10 px-3.5 py-2.5 text-left text-xs font-semibold text-warning transition-colors hover:border-warning/65 disabled:opacity-50"
                  title={specialModStatus.latestPatchBackup}
                >
                  恢复最近一次联机补丁备份
                </button>
              )}
            </div>
          </section>
        </div>

        {preflight && (
          <section className="panel-card mt-4 rounded-xl p-4">
            <div className="flex items-center justify-between gap-3">
              <div>
                <div className="section-label text-text-muted">Preflight report</div>
                <h3 className="mt-1 text-base font-semibold text-text-primary">启动前检查</h3>
              </div>
              <span
                className={`rounded-full border px-3 py-1 text-xs font-semibold ${
                  preflight.ready
                    ? "border-success/25 bg-success/10 text-success"
                    : "border-danger/25 bg-danger/10 text-danger"
                }`}
              >
                {preflight.ready ? "可以启动" : "需要处理"}
              </span>
            </div>
            <div className="mt-3 grid gap-2 md:grid-cols-2 xl:grid-cols-3">
              {preflight.checks.map((check) => (
                <PreflightRow key={check.id} check={check} />
              ))}
            </div>
          </section>
        )}

        <div className="mt-4 grid gap-4 lg:grid-cols-[minmax(0,1fr)_19rem]">
          <section className="panel-card rounded-xl p-4">
            <div className="flex items-center justify-between gap-3">
              <div>
                <div className="section-label text-text-muted">Environment</div>
                <h3 className="mt-1 text-base font-semibold text-text-primary">运行环境</h3>
              </div>
              <span className={`text-xs font-semibold ${onlineReady ? "text-success" : "text-warning"}`}>
                {onlineReady ? "联机组件就绪" : "存在可选组件提醒"}
              </span>
            </div>
            <div className="mt-3 grid gap-2 sm:grid-cols-2">
              <SpecialRow
                label="环境模式"
                ok={Boolean(runtimeEnvironmentStatus?.verified)}
                value={`${runtimeEnvironmentLabel(runtimeEnvironment)} · ${
                  runtimeEnvironmentStatus?.verified ? "已实测" : "未实测/需检查"
                }`}
              />
              <SpecialRow
                label="Server Redirector"
                ok={usingServerRedirector}
                value={
                  usingServerRedirector
                    ? `由 ${serverRedirectorMod?.name ?? "当前 Mod"} 提供`
                    : "当前方案未使用"
                }
              />
              <SpecialRow
                label="SeamlessCoop"
                ok={usingServerRedirector || Boolean(specialModStatus?.seamlessInstalled)}
                value={
                  usingServerRedirector
                    ? specialModStatus?.seamlessInstalled
                      ? "已安装 · 本方案不注入"
                      : "本方案不需要"
                    : specialModStatus?.seamlessInstalled
                      ? "已安装"
                      : "未安装"
                }
              />
              <SpecialRow
                label="OnlineFix / Spacewar"
                ok={
                  usingServerRedirector
                    ? !specialModStatus?.serverRedirectorConflicts.length
                    : Boolean(specialModStatus?.onlinefixInstalled)
                }
                value={
                  usingServerRedirector
                    ? specialModStatus?.serverRedirectorConflicts.length
                      ? `冲突 · ${specialModStatus.serverRedirectorConflicts.join(", ")}`
                      : "未安装 · 符合 MMV 要求"
                    : specialModStatus?.onlinefixInstalled
                      ? "已安装"
                      : "未安装"
                }
              />
              <SpecialRow
                label="Nighter"
                ok={usingServerRedirector || Boolean(specialModStatus?.nighterAvailable)}
                value={
                  usingServerRedirector
                    ? specialModStatus?.nighterAvailable
                      ? "已检测到 · 本方案不注入"
                      : "本方案不加载"
                    : specialModStatus?.nighterAvailable
                      ? specialModStatus.nighterPath
                      : "未检测到"
                }
              />
            </div>
            {runtimeEnvironmentStatus && (
              <p
                className={`mt-3 rounded-lg border px-3 py-2 text-xs leading-5 ${
                  runtimeEnvironmentStatus.verified
                    ? "border-success/25 bg-success/10 text-success"
                    : "border-warning/25 bg-warning/10 text-warning"
                }`}
              >
                配置：{runtimeEnvironmentLabel(runtimeEnvironmentStatus.configured)}；检测：
                {runtimeEnvironmentLabel(runtimeEnvironmentStatus.detected)}。仅 Spacewar +
                Seamless 已完成本机真实启动验证。
              </p>
            )}
            {usingServerRedirector && (
              <p
                className={`mt-3 rounded-lg border px-3 py-2 text-xs leading-5 ${
                  redirectorEnvironmentConflict
                    ? "border-danger/25 bg-danger/10 text-danger"
                    : "border-success/25 bg-success/10 text-success"
                }`}
              >
                {redirectorEnvironmentConflict
                  ? "MMV 已识别，但当前 Game 目录包含 OnlineFix / Spacewar；启动将被阻止。请在设置中选择干净的 Steam 正版 Game 目录。"
                  : "MMV 兼容模式已启用：保留作者存档与在线启动字段，使用 Steam 初始化，并屏蔽游戏目录中的 SeamlessCoop/nighter 注入。"}
              </p>
            )}
            {specialModStatus && specialModStatus.missingGameFiles.length > 0 && (
              <p className="mt-3 rounded-lg border border-warning/25 bg-warning/10 px-3 py-2 text-xs leading-5 text-warning">
                联机目录缺少：{specialModStatus.missingGameFiles.join(", ")}
              </p>
            )}
          </section>

          <section className="panel-card rounded-xl p-4">
            <div className="flex items-center justify-between">
              <h3 className="text-sm font-semibold text-text-primary">本次加载清单</h3>
              <span className="display-number text-xs text-text-muted">{enabledMods.length}</span>
            </div>
            <div className="mt-3 max-h-48 space-y-1.5 overflow-y-auto pr-1">
              {enabledMods.length === 0 ? (
                <p className="rounded-lg border border-dashed border-border p-3 text-xs leading-5 text-text-muted">
                  当前没有启用的 Mod，仍会尝试加载游戏根目录中的联机 DLL。
                </p>
              ) : (
                enabledMods.map((mod) => (
                  <div key={mod.id} className="flex items-center justify-between gap-2 rounded-md bg-surface px-2.5 py-2">
                    <span className="min-w-0 truncate text-xs text-text-secondary">{mod.name}</span>
                    <span className="shrink-0 text-[10px] text-text-muted">{mod.type === "native" ? "DLL" : "资源"}</span>
                  </div>
                ))
              )}
            </div>
          </section>
        </div>
      </div>
    </PageFrame>
  );
}

function runtimeEnvironmentLabel(value: string) {
  const labels: Record<string, string> = {
    auto: "自动检测",
    steam_official: "纯正版 Steam",
    steam_seamless: "正版 Steam + Seamless",
    spacewar_seamless: "Spacewar + Seamless",
    unknown_mixed: "未知/混合环境",
  };
  return labels[value] ?? value;
}

function PreflightRow({ check }: { check: LaunchPreflightCheck }) {
  const dotClass =
    check.status === "pass"
      ? "bg-success"
      : check.status === "warning"
        ? "bg-warning"
        : "bg-danger";

  return (
    <div className="rounded-lg border border-border bg-surface/80 px-3 py-2.5">
      <div className="flex items-start gap-2.5">
        <span className={`mt-1.5 h-2 w-2 shrink-0 rounded-full ${dotClass}`} />
        <div className="min-w-0">
          <div className="text-sm font-semibold text-text-primary">{check.label}</div>
          <div className="mt-1 break-all text-xs leading-5 text-text-muted">{check.message}</div>
        </div>
      </div>
    </div>
  );
}

function SpecialRow({ label, value, ok }: { label: string; value: string; ok: boolean }) {
  return (
    <div className="rounded-lg border border-border bg-surface/80 px-3 py-2.5">
      <div className="flex items-center justify-between gap-3">
        <span className="text-sm font-semibold text-text-primary">{label}</span>
        <span className={`h-2 w-2 rounded-full ${ok ? "bg-success" : "bg-danger"}`} />
      </div>
      <div className="mt-1 truncate text-xs text-text-muted" title={value}>
        {value}
      </div>
    </div>
  );
}

export function PageFrame({
  eyebrow,
  title,
  description,
  children,
}: {
  eyebrow: string;
  title: string;
  description: string;
  children: ReactNode;
}) {
  return (
    <div className="page-enter flex h-full min-h-0 flex-col overflow-hidden px-5 pb-5 pt-4">
      <header className="mb-4 flex shrink-0 items-end justify-between gap-6 border-b border-border pb-3">
        <div className="min-w-0">
          <div className="section-label">{eyebrow}</div>
          <div className="mt-1.5 flex flex-wrap items-baseline gap-x-4 gap-y-1">
            <h1 className="text-xl font-bold tracking-tight text-text-primary">{title}</h1>
            <p className="max-w-3xl truncate text-xs leading-5 text-text-muted">{description}</p>
          </div>
        </div>
      </header>
      {children}
    </div>
  );
}

function Metric({ label, value }: { label: string; value: string }) {
  return (
    <div className="rounded-lg border border-border bg-surface/75 px-3 py-2.5">
      <div className="text-[11px] text-text-muted">{label}</div>
      <div className="display-number mt-1 text-xl font-semibold text-text-primary">{value}</div>
    </div>
  );
}

function StatusRow({ label, value, ok }: { label: string; value: string; ok: boolean }) {
  return (
    <div className="flex items-start gap-2.5 rounded-lg border border-border bg-surface/75 px-3 py-2.5">
      <span className={`mt-1.5 h-1.5 w-1.5 shrink-0 rounded-full ${ok ? "bg-success" : "bg-danger"}`} />
      <div className="min-w-0">
        <div className="text-sm font-semibold text-text-primary">{label}</div>
        <div className="mt-1 truncate text-xs leading-5 text-text-muted" title={value}>{value}</div>
      </div>
    </div>
  );
}

function PlayArrow() {
  return (
    <svg className="h-4 w-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.2">
      <path d="M5 12h14M13 6l6 6-6 6" />
    </svg>
  );
}
