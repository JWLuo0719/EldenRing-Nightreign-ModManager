import type { ModInfo, Profile, SpecialModStatus } from "../types/mod";
import type { ReactNode } from "react";

interface LaunchPageProps {
  mods: ModInfo[];
  activeProfile: Profile | null;
  gamePath: string;
  me3Path: string;
  launchExePath: string;
  specialModStatus: SpecialModStatus | null;
  busy: boolean;
  onLaunch: () => void;
  onRefresh: () => void;
  onOpenDiagnostics: () => void;
  onPrepareOnline: () => void;
}

export function LaunchPage({
  mods,
  activeProfile,
  gamePath,
  me3Path,
  launchExePath,
  specialModStatus,
  busy,
  onLaunch,
  onRefresh,
  onOpenDiagnostics,
  onPrepareOnline,
}: LaunchPageProps) {
  const enabledMods = mods.filter((mod) => mod.enabled);
  const launchTarget = launchExePath.trim()
    ? launchExePath.split(/[\\/]/).pop() || "自定义启动程序"
    : "nightreign.exe";
  const onlineReady = Boolean(
    specialModStatus?.seamlessInstalled && specialModStatus.onlinefixInstalled
  );

  return (
    <PageFrame
      eyebrow="Launch Center"
      title="启动台"
      description="从这里确认环境状态、生成当前方案并通过 ME3 启动 Nightreign。"
    >
      <div className="grid min-h-0 flex-1 gap-4 overflow-y-auto pr-1 xl:grid-cols-[1.2fr_0.8fr]">
        <section className="rounded-xl border border-border bg-panel p-6">
          <div className="flex flex-wrap items-start justify-between gap-4">
            <div>
              <div className="text-sm text-text-muted">当前配置方案</div>
              <h2 className="mt-2 text-3xl font-bold text-text-primary">
                {activeProfile?.name ?? "全局启用状态"}
              </h2>
              <p className="mt-3 max-w-2xl text-sm leading-6 text-text-secondary">
                启动时会先生成 active-nightreign.me3，再执行 launch-nightreign.bat。
                当前目标程序：{launchTarget}。
              </p>
            </div>
            <button
              type="button"
              disabled={busy}
              onClick={onLaunch}
              className="rounded-lg bg-accent px-6 py-3 text-sm font-bold text-black transition-colors hover:bg-accent-hover disabled:cursor-not-allowed disabled:opacity-50"
            >
              启动游戏
            </button>
          </div>

          <div className="mt-8 grid gap-3 md:grid-cols-3">
            <Metric label="启用 Mod" value={`${enabledMods.length}/${mods.length}`} />
            <Metric label="资源包" value={String(mods.filter((mod) => mod.type === "package").length)} />
            <Metric label="DLL" value={String(mods.filter((mod) => mod.type === "native").length)} />
          </div>

          <div className="mt-8 grid gap-3">
            <StatusRow label="游戏目录" value={gamePath || "未配置"} ok={Boolean(gamePath)} />
            <StatusRow label="ME3 目录" value={me3Path || "未配置"} ok={Boolean(me3Path)} />
            <StatusRow
              label="联机环境"
              value={
                onlineReady
                  ? "SeamlessCoop 与 OnlineFix 已在当前游戏目录就绪"
                  : "当前游戏目录缺少 SeamlessCoop 或 OnlineFix 文件"
              }
              ok={onlineReady}
            />
            <StatusRow
              label="Nighter 深夜解锁"
              value={
                specialModStatus?.nighterAvailable
                  ? specialModStatus.nighterPath
                  : "未检测到 Game\\mods\\nighter.dll"
              }
              ok={Boolean(specialModStatus?.nighterAvailable)}
            />
          </div>
        </section>

        <aside className="flex min-h-0 flex-col gap-4">
          <section className="rounded-xl border border-border bg-panel p-5">
            <div className="flex items-center justify-between">
              <h3 className="text-base font-semibold text-text-primary">快速操作</h3>
              <button
                type="button"
                disabled={busy}
                onClick={onRefresh}
                className="text-sm font-medium text-accent hover:text-accent-hover disabled:opacity-50"
              >
                刷新
              </button>
            </div>
            <div className="mt-4 grid gap-2">
              <button
                type="button"
                onClick={onOpenDiagnostics}
                className="rounded-lg border border-border bg-surface px-4 py-3 text-left text-sm text-text-secondary transition-colors hover:border-accent/45 hover:text-text-primary"
              >
                打开诊断页，查看 ME3 profile、启动命令和诊断输出
              </button>
              <button
                type="button"
                disabled={busy}
                onClick={onPrepareOnline}
                className="rounded-lg border border-border bg-surface px-4 py-3 text-left text-sm text-text-secondary transition-colors hover:border-accent/45 hover:text-text-primary disabled:cursor-not-allowed disabled:opacity-50"
              >
                选择补丁 Game 文件夹并应用
              </button>
            </div>
          </section>

          <section className="rounded-xl border border-border bg-panel p-5">
            <h3 className="text-base font-semibold text-text-primary">特殊 Mod 检测</h3>
            <div className="mt-4 space-y-3">
              <SpecialRow
                label="SeamlessCoop"
                ok={Boolean(specialModStatus?.seamlessInstalled)}
                value={specialModStatus?.seamlessInstalled ? "已安装" : "未安装"}
              />
              <SpecialRow
                label="OnlineFix"
                ok={Boolean(specialModStatus?.onlinefixInstalled)}
                value={specialModStatus?.onlinefixInstalled ? "已安装" : "未安装"}
              />
              <SpecialRow
                label="Nighter"
                ok={Boolean(specialModStatus?.nighterAvailable)}
                value={specialModStatus?.nighterAvailable ? specialModStatus.nighterPath : "未检测到 Game\\mods\\nighter.dll"}
              />
            </div>
            {specialModStatus && specialModStatus.missingGameFiles.length > 0 && (
              <p className="mt-4 rounded-lg border border-warning/25 bg-warning/10 p-3 text-xs leading-5 text-warning">
                当前游戏目录缺少：{specialModStatus.missingGameFiles.join(", ")}
              </p>
            )}
          </section>

          <section className="flex min-h-0 flex-1 flex-col rounded-xl border border-border bg-panel p-5">
            <h3 className="text-center text-base font-semibold text-text-primary">本次将加载</h3>
            <div className="mt-4 min-h-0 flex-1 space-y-2 overflow-y-auto pr-1">
              {enabledMods.length === 0 ? (
                <div className="grid h-full min-h-36 place-items-center rounded-lg border border-dashed border-border p-5 text-center">
                  <p className="max-w-sm text-sm leading-6 text-text-muted">
                    当前没有启用的 Mod。ME3 profile 仍会尝试加入游戏根目录下的 SeamlessCoop DLL。
                  </p>
                </div>
              ) : (
                enabledMods.map((mod) => (
                  <div key={mod.id} className="flex min-h-11 items-center justify-between rounded-lg bg-surface px-3 py-2">
                    <span className="min-w-0 truncate text-sm text-text-secondary">{mod.name}</span>
                    <span className="ml-3 shrink-0 text-xs text-text-muted">
                      {mod.type === "native" ? "DLL" : "资源包"}
                    </span>
                  </div>
                ))
              )}
            </div>
          </section>
        </aside>
      </div>
    </PageFrame>
  );
}

function SpecialRow({ label, value, ok }: { label: string; value: string; ok: boolean }) {
  return (
    <div className="rounded-lg border border-border bg-surface px-3 py-2">
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
    <div className="flex h-full min-h-0 flex-col overflow-hidden p-6">
      <header className="mb-4 shrink-0">
        <div className="text-[11px] font-semibold uppercase tracking-[0.24em] text-accent">{eyebrow}</div>
        <h1 className="mt-1.5 text-xl font-bold text-text-primary">{title}</h1>
        <p className="mt-1.5 max-w-3xl text-sm leading-5 text-text-secondary">{description}</p>
      </header>
      {children}
    </div>
  );
}

function Metric({ label, value }: { label: string; value: string }) {
  return (
    <div className="rounded-lg border border-border bg-surface px-4 py-3">
      <div className="text-xs text-text-muted">{label}</div>
      <div className="mt-1 text-2xl font-bold text-text-primary">{value}</div>
    </div>
  );
}

function StatusRow({ label, value, ok }: { label: string; value: string; ok: boolean }) {
  return (
    <div className="flex items-start gap-3 rounded-lg border border-border bg-surface px-4 py-3">
      <span className={`mt-1 h-2.5 w-2.5 shrink-0 rounded-full ${ok ? "bg-success" : "bg-danger"}`} />
      <div className="min-w-0">
        <div className="text-sm font-semibold text-text-primary">{label}</div>
        <div className="mt-1 break-all text-xs leading-5 text-text-muted">{value}</div>
      </div>
    </div>
  );
}
