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
}: LaunchPageProps) {
  const [preflight, setPreflight] = useState<LaunchPreflight | null>(null);
  const [showLaunchHelp, setShowLaunchHelp] = useState(false);
  const enabledMods = mods.filter((mod) => mod.enabled);
  const hasCustomLaunchTarget = Boolean(launchExePath.trim());
  const serverRedirectorMod = enabledMods.find(
    (mod) => mod.networkBackend === "server_redirector"
  );
  const usingServerRedirector = Boolean(serverRedirectorMod);
  const redirectorEnvironmentConflict = Boolean(
    usingServerRedirector && specialModStatus?.serverRedirectorConflicts.length
  );
  const runtimeEnvironment = runtimeEnvironmentStatus?.effective ?? "unknown_mixed";
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
      description="先检查当前方案，再一键启动游戏；遇到问题会直接告诉你下一步怎么做。"
    >
      <div className="min-h-0 flex-1 overflow-y-auto pr-1">
        <div className="grid gap-4 lg:grid-cols-[minmax(0,1fr)_19rem]">
          <section className="panel-card relative overflow-hidden rounded-xl p-5">
            <div className="absolute inset-x-0 top-0 h-px bg-gradient-to-r from-accent via-accent/35 to-transparent" />
            <div className="section-label">当前准备状态</div>
            <div className="mt-3 flex flex-wrap items-end justify-between gap-4">
              <div className="min-w-0">
                <h2 className="truncate text-2xl font-bold tracking-tight text-text-primary">
                  {activeProfile?.name ?? "全局启用状态"}
                </h2>
                <p className="mt-2 text-sm leading-6 text-text-secondary">
                  将按当前方案加载内容并启动游戏。
                  {hasCustomLaunchTarget && " 当前使用你在设置中选择的额外启动程序。"}
                  {usingServerRedirector &&
                    " 当前方案沿用作者的正版联机方式，需要 Steam 正版环境。"}
                </p>
              </div>
              <div className="flex items-baseline gap-2 text-text-muted">
                <span className="display-number text-4xl font-semibold text-accent">{enabledMods.length}</span>
                <span className="text-xs">项内容待加载</span>
              </div>
            </div>

            <div className="mt-5 grid gap-2 sm:grid-cols-3">
              <Metric label="启用 / 全部" value={`${enabledMods.length} / ${mods.length}`} />
              <Metric label="资源型 Mod" value={String(mods.filter((mod) => mod.type === "package").length)} />
              <Metric label="功能插件" value={String(mods.filter((mod) => mod.type === "native").length)} />
            </div>

            <div className="mt-5 grid gap-2 sm:grid-cols-2">
              <StatusRow label="游戏目录" value={gamePath || "未配置"} ok={Boolean(gamePath)} />
              <StatusRow label="ME3 目录" value={me3Path || "未配置"} ok={Boolean(me3Path)} />
            </div>
          </section>

          <section className="panel-card rounded-xl p-4">
            <div className="flex items-center justify-between">
              <div>
                <div className="section-label text-text-muted">开始游戏</div>
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
              <span>启动游戏</span>
              <PlayArrow />
            </button>
            <p className="mt-2 text-xs leading-5 text-text-muted">
              启动过程会在后台完成，不会弹出终端窗口；需要排错时再查看启动日志。
            </p>
            <div className="mt-2 grid gap-2">
              <button
                type="button"
                disabled={busy}
                onClick={() => void runPreflight()}
                className="rounded-lg border border-accent/35 bg-accent-soft px-3.5 py-2.5 text-left text-sm font-semibold text-accent transition-colors hover:border-accent/65 disabled:opacity-50"
              >
                启动前检查 <span className="font-normal text-text-muted">· 不启动游戏</span>
              </button>
              <div className="grid gap-2 sm:grid-cols-2">
                <button
                  type="button"
                  onClick={onOpenDiagnostics}
                  className="rounded-lg border border-border bg-surface px-3 py-2.5 text-left text-xs text-text-secondary transition-colors hover:border-border-strong hover:text-text-primary"
                >
                  查看启动日志和诊断
                </button>
                <button
                  type="button"
                  onClick={() => setShowLaunchHelp(true)}
                  className="rounded-lg border border-border bg-surface px-3 py-2.5 text-left text-xs text-text-secondary transition-colors hover:border-border-strong hover:text-text-primary"
                >
                  启动说明
                </button>
              </div>
            </div>
          </section>
        </div>

        {preflight && (
          <section className="panel-card mt-4 rounded-xl p-4">
            <div className="flex items-center justify-between gap-3">
              <div>
                <div className="section-label text-text-muted">启动前检查结果</div>
                <h3 className="mt-1 text-base font-semibold text-text-primary">启动前检查</h3>
                <p className="mt-1 text-xs leading-5 text-text-muted">
                  红色必须处理；黄色不一定阻止启动，但请先理解风险。
                </p>
              </div>
              <span
                className={`rounded-full border px-3 py-1 text-xs font-semibold ${
                  preflight.ready
                    ? "border-success/25 bg-success/10 text-success"
                    : "border-danger/25 bg-danger/10 text-danger"
                }`}
              >
                {preflight.ready ? "可以启动" : `先处理 ${preflight.checks.filter((check) => check.status === "error").length} 项问题`}
              </span>
            </div>
            <div className="mt-3 grid gap-2 md:grid-cols-2 xl:grid-cols-3">
              {preflight.checks.map((check) => (
                <PreflightRow key={check.id} check={check} onOpenHelp={() => setShowLaunchHelp(true)} />
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
                tone={runtimeEnvironmentStatus?.verified ? "success" : "warning"}
                value={`${runtimeEnvironmentLabel(runtimeEnvironment)} · ${
                  runtimeEnvironmentStatus?.verified ? "已实测" : "未实测/需检查"
                }`}
              />
              <SpecialRow
                label="作者指定的联机方式"
                tone={usingServerRedirector ? "success" : "warning"}
                value={
                  usingServerRedirector
                    ? `正版联机桥接（Server Redirector），由 ${serverRedirectorMod?.name ?? "当前 Mod"} 提供`
                    : "当前方案未使用作者指定的联机方式"
                }
              />
              <SpecialRow
                label="社区联机插件"
                tone={
                  usingServerRedirector || specialModStatus?.seamlessInstalled
                    ? "success"
                    : "error"
                }
                value={
                  usingServerRedirector
                    ? specialModStatus?.seamlessInstalled
                      ? "已安装 · 当前方案不加载"
                      : "当前方案不需要"
                    : specialModStatus?.seamlessInstalled
                      ? "已安装"
                      : "未安装"
                }
              />
              <SpecialRow
                label="社区联机运行组件"
                tone={
                  (usingServerRedirector && specialModStatus?.serverRedirectorConflicts.length) ||
                  (!usingServerRedirector && !specialModStatus?.onlinefixInstalled)
                    ? "error"
                    : "success"
                }
                value={
                  usingServerRedirector
                    ? specialModStatus?.serverRedirectorConflicts.length
                      ? `冲突 · ${specialModStatus.serverRedirectorConflicts.join(", ")}`
                      : "未安装 · 符合作者的正版联机要求"
                    : specialModStatus?.onlinefixInstalled
                      ? "已安装"
                      : "未安装"
                }
              />
              <SpecialRow
                label="深夜解锁（可选）"
                tone={specialModStatus?.nighterAvailable ? "success" : "warning"}
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
                你选择的方式：{runtimeEnvironmentLabel(runtimeEnvironmentStatus.configured)}；自动检测：
                {runtimeEnvironmentLabel(runtimeEnvironmentStatus.detected)}。当前只有社区联机（Spacewar +
                Seamless）完成本机真实启动验证。
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
                  ? "当前方案要求正版 Steam，但这个游戏目录含有社区联机运行文件；为避免登录失败和异常存档，已阻止启动。请在设置中选择干净的 Steam 正版 Game 目录。"
                  : "当前已采用 MMV 社区兼容方式：保留作者文件，只在本次启动中使用社区联机插件。技术细节可在诊断页查看。"}
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
                  当前没有启用任何 Mod；游戏仍会按当前联机方式启动。
                </p>
              ) : (
                enabledMods.map((mod) => (
                  <div key={mod.id} className="flex items-center justify-between gap-2 rounded-md bg-surface px-2.5 py-2">
                    <span className="min-w-0 truncate text-xs text-text-secondary">{mod.name}</span>
                    <span className="shrink-0 text-[10px] text-text-muted">{mod.type === "native" ? "功能插件" : "资源型 Mod"}</span>
                  </div>
                ))
              )}
            </div>
          </section>
        </div>
      </div>
      {showLaunchHelp && <LaunchHelpDialog onClose={() => setShowLaunchHelp(false)} />}
    </PageFrame>
  );
}

function runtimeEnvironmentLabel(value: string) {
  const labels: Record<string, string> = {
    auto: "自动检测",
    steam_official: "官方 Steam 游玩",
    steam_seamless: "Steam + 社区联机插件",
    spacewar_seamless: "社区联机（Spacewar + Seamless）",
    unknown_mixed: "需要确认的混合环境",
  };
  return labels[value] ?? value;
}

function PreflightRow({
  check,
  onOpenHelp,
}: {
  check: LaunchPreflightCheck;
  onOpenHelp: () => void;
}) {
  const [showDetails, setShowDetails] = useState(false);
  const guidance = getPreflightGuidance(check);
  const dotClass =
    check.status === "pass"
      ? "bg-success"
      : check.status === "warning"
        ? "bg-warning"
        : "bg-danger";

  return (
    <div
      className={`rounded-lg border px-3 py-3 ${
        check.status === "error"
          ? "border-danger/30 bg-danger/5"
          : check.status === "warning"
            ? "border-warning/30 bg-warning/5"
            : "border-border bg-surface/80"
      }`}
    >
      <div className="flex items-start gap-2.5">
        <span className={`mt-1.5 h-2 w-2 shrink-0 rounded-full ${dotClass}`} />
        <div className="min-w-0">
          <div className="text-sm font-semibold text-text-primary">{preflightLabel(check)}</div>
          <div className="mt-1 text-xs leading-5 text-text-secondary">
            {check.status === "pass" ? guidance.ok : guidance.problem}
          </div>
          {check.status !== "pass" && (
            <div className="mt-2 rounded-md border border-current/15 bg-black/10 px-2.5 py-2 text-xs leading-5 text-text-primary">
              <span className="font-semibold">下一步：</span>
              {guidance.action}
            </div>
          )}
          <div className="mt-2 flex flex-wrap items-center gap-x-3 gap-y-1">
            <button
              type="button"
              onClick={() => setShowDetails((visible) => !visible)}
              className="text-xs text-text-muted underline-offset-2 transition-colors hover:text-text-primary hover:underline"
            >
              {showDetails ? "收起系统详情" : "查看系统详情"}
            </button>
            {check.status !== "pass" && (
              <button
                type="button"
                onClick={onOpenHelp}
                className="text-xs text-accent underline-offset-2 transition-colors hover:underline"
              >
                看不懂？打开说明
              </button>
            )}
          </div>
          {showDetails && (
            <div className="mt-2 break-all rounded-md border border-border bg-black/15 px-2.5 py-2 text-xs leading-5 text-text-muted">
              {check.message}
            </div>
          )}
        </div>
      </div>
    </div>
  );
}

function getPreflightGuidance(check: LaunchPreflightCheck) {
  const guidance: Record<string, { ok: string; problem: string; action: string }> = {
    game_path: {
      ok: "游戏本体已找到。",
      problem: "管理器没有找到游戏本体。",
      action: "打开“设置”，重新选择包含 nightreign.exe 的 Game 文件夹。",
    },
    me3: {
      ok: "Mod 加载工具已找到。",
      problem: "Mod 加载工具缺失、路径错误或版本过旧。",
      action: "打开“设置”，选择 ME3 根目录或 bin 目录；建议使用 ME3 0.12.1 及以上版本。",
    },
    launch_target: {
      ok: "本次要启动的游戏程序已找到。",
      problem: "当前启动目标无效。",
      action: "在“设置”清除错误的自定义启动程序，或重新选择游戏目录中的启动文件。",
    },
    steam: {
      ok: "Steam 已启动，可以提供当前联机方式所需的身份。",
      problem: "Steam 未启动，或它与游戏的权限级别不同。",
      action: "先启动 Steam，再启动管理器；不要只把其中一个程序设为管理员运行。",
    },
    runtime_environment: {
      ok: "当前游戏目录与选择的联机方式匹配。",
      problem: "选择的联机方式和当前游戏目录不匹配。",
      action: "到“设置”确认运行环境。正版 Steam 与 Spacewar/Seamless 必须使用不同的 Game 目录。",
    },
    running_processes: {
      ok: "没有检测到上一轮残留的游戏或加载进程。",
      problem: "上一轮游戏或加载工具仍未完全退出。",
      action: "关闭游戏和 ME3，等待几秒后点击“刷新状态”；仍存在时可在任务管理器结束对应进程。",
    },
    network_backend: {
      ok: "本次需要的联机组件已经确定。",
      problem: "当前方案的联机组件无法确定。",
      action: "检查当前启用的 Mod 和启动配置；不确定时先到“Mod 仓库”停用最近添加的联机类 Mod。",
    },
    savefile: {
      ok: "本次使用的存档已确定，启动前会自动备份。",
      problem: "管理器无法判断本次该备份哪个存档。",
      action: "先不要启动。检查设置和当前方案，确认是否切换过 Seamless、正版 Steam 或作者提供的启动配置。",
    },
    save_gameplay_compatibility: {
      ok: "没有发现已记录的玩法参数变化风险。",
      problem: "这套玩法内容可能和最近使用的存档不一致。",
      action: "先保留自动备份；如果刚停用地图、武器或扩展服装，建议换回原方案或用新角色验证。",
    },
    author_profile: {
      ok: "作者提供的启动要求已保留在本次启动中。",
      problem: "作者提供的启动要求无法完整读取。",
      action: "检查 Mod 的 .me3 文件是否仍在原位置；不要直接编辑或移动作者文件。",
    },
    mmv_seamless_community: {
      ok: "当前没有使用 MMV 社区兼容方式。",
      problem: "当前使用的是社区兼容方式，而不是作者官方联机路线。",
      action: "确认你选择的是 Spacewar/Seamless 社区方案；不要把它与 Server Redirector 或干净正版 Steam 目录混用。",
    },
    regulation_owner: {
      ok: "玩法数据文件数量正确。",
      problem: "玩法数据文件缺失或有多份互相覆盖。",
      action: "到“Mod 仓库”只保留一个已合并的地图/武器/扩展服装方案；不要同时启用多份 regulation.bin。",
    },
    zhocn_layer: {
      ok: "中文文本层数量正确。",
      problem: "中文文本缺失或有多份互相覆盖。",
      action: "需要中文时只启用一份完整汉化；若启用多份，请先停用旧翻译再检查。",
    },
    external_profile_location: {
      ok: "作者启动配置的位置符合当前环境要求。",
      problem: "MMV 作者文件放在了游戏 Game 目录内。",
      action: "把作者整合包放到 Game 目录之外，再从“Mod 仓库”重新添加外部 Mod。",
    },
    seamless: {
      ok: "当前方案需要的 Seamless 组件状态正常。",
      problem: "当前联机方式需要 SeamlessCoop，但关键文件不完整，或它与正版方案冲突。",
      action: "确认选择的是正确的 Game 目录；不要把正版 Steam、Server Redirector 与 Spacewar/Seamless 文件混在同一目录。",
    },
    onlinefix: {
      ok: "当前联机方式所需的运行文件状态正常。",
      problem: "联机运行文件缺失，或与当前方案发生冲突。",
      action: "不要在管理器中覆盖补丁文件。请改选对应环境的完整 Game 目录；混合目录需要由你手动恢复或另建干净目录。",
    },
    nighter: {
      ok: "深夜解锁组件状态正常。",
      problem: "未检测到深夜解锁组件。",
      action: "如果不需要深夜解锁可以忽略；需要时请按作者说明手动放入文件，不要把它与 Server Redirector 同时加载。",
    },
    enabled_mods: {
      ok: "本次会按当前方案加载已启用的内容。",
      problem: "当前没有启用内容，或管理器无法扫描 Mod。",
      action: "到“Mod 仓库”检查是否误停用；如果只想玩原版，仍可直接启动。",
    },
    clothing_resources: {
      ok: "服装 Mod 的队友视角资源已配对。",
      problem: "部分服装缺少队友视角资源。",
      action: "你自己可能仍正常显示；联机前按可信作者说明手动准备 _l 资源，管理器不会运行包内脚本。",
    },
    profile_generation: {
      ok: "启动配置可以生成。",
      problem: "管理器无法生成本次启动配置。",
      action: "到“Mod 仓库”检查最近启用的 Mod；重点确认作者 .me3、功能插件和资源型 Mod 的原始路径没有被移动。",
    },
  };

  const result = (
    guidance[check.id] ?? {
      ok: "检查通过。",
      problem: "此项需要你确认。",
      action: "打开“启动说明”查看名词解释；仍无法判断时进入“诊断”页查看启动日志。",
    }
  );

  if (check.id === "me3" && check.status === "error") {
    return {
      ...result,
      problem: "管理器没有找到 Mod 加载工具。",
    };
  }
  return result;
}

function preflightLabel(check: LaunchPreflightCheck) {
  const labels: Record<string, string> = {
    game_path: "游戏所在文件夹",
    me3: "Mod 加载工具（ME3）",
    launch_target: "游戏启动程序",
    steam: "Steam 登录状态",
    runtime_environment: "当前联机方式",
    running_processes: "游戏是否已完全退出",
    network_backend: "本次联机方式",
    savefile: "本次备份的存档",
    save_gameplay_compatibility: "存档安全提醒",
    author_profile: "作者提供的启动要求",
    mmv_seamless_community: "MMV 社区兼容方式",
    regulation_owner: "玩法数据文件",
    zhocn_layer: "中文文本",
    external_profile_location: "作者文件位置",
    seamless: "社区联机插件（Seamless）",
    onlinefix: "社区联机运行组件",
    nighter: "深夜解锁（可选）",
    enabled_mods: "本次加载内容",
    clothing_resources: "服装联机显示",
    profile_generation: "本次启动准备",
  };
  return labels[check.id] ?? check.label;
}

function LaunchHelpDialog({ onClose }: { onClose: () => void }) {
  return (
    <div className="fixed inset-0 z-40 flex items-center justify-center bg-black/65 px-4 py-6 backdrop-blur-sm">
      <section className="panel-card flex max-h-full w-full max-w-2xl flex-col rounded-xl shadow-2xl" role="dialog" aria-modal="true" aria-label="启动说明">
        <header className="flex shrink-0 items-start justify-between gap-4 border-b border-border px-5 py-4">
          <div>
            <div className="section-label">新手说明</div>
            <h2 className="mt-1 text-lg font-semibold text-text-primary">启动前检查怎么看</h2>
          </div>
          <button
            type="button"
            onClick={onClose}
            className="rounded-md px-2 py-1 text-sm text-text-muted transition-colors hover:bg-surface hover:text-text-primary"
          >
            关闭
          </button>
        </header>
        <div className="min-h-0 overflow-y-auto px-5 py-4 text-sm leading-6 text-text-secondary">
          <p>每次启动只要按这个顺序：先确认“设置”里的游戏目录和联机方式，再运行启动前检查；没有红色项目时即可启动游戏。</p>
          <div className="mt-4 grid gap-2 sm:grid-cols-3">
            <HelpStep number="1" title="确认目录" text="游戏目录必须直接包含 nightreign.exe。" />
            <HelpStep number="2" title="运行检查" text="红色先处理，黄色先阅读风险说明。" />
            <HelpStep number="3" title="后台启动" text="启动后不出现终端；异常时看诊断日志。" />
          </div>
          <div className="mt-5 border-t border-border pt-4">
            <h3 className="text-sm font-semibold text-text-primary">常见名称</h3>
            <dl className="mt-2 grid gap-x-5 gap-y-3 sm:grid-cols-2">
              <HelpTerm term="启动配置" description="管理器本次要加载哪些 Mod、功能插件以及加载顺序。高级名称是 Profile / .me3。" />
              <HelpTerm term="资源型 Mod" description="地图、模型、贴图、文本等内容文件。高级名称是 package。" />
              <HelpTerm term="功能插件" description="提供联机或特殊功能的 DLL。高级名称是 native DLL。" />
              <HelpTerm term="玩法数据文件" description="记录武器、服装 ID 和玩法参数的 regulation.bin；多份会互相覆盖。" />
              <HelpTerm term="队友视角资源" description="名称带 _l 的服装文件，决定联机队友看到的模型。" />
              <HelpTerm term="启动日志" description="后台启动时的过程记录。只有启动失败或要排错时才需要查看。" />
            </dl>
          </div>
          <p className="mt-5 rounded-lg border border-warning/25 bg-warning/10 px-3 py-2 text-xs leading-5 text-warning">
            重要：正版 Steam、Server Redirector 与 Spacewar/Seamless 不能共用同一个 Game 目录。不要为了通过检查而删除补丁文件；应选择对应的完整目录。
          </p>
        </div>
      </section>
    </div>
  );
}

function HelpStep({ number, title, text }: { number: string; title: string; text: string }) {
  return (
    <div className="rounded-lg border border-border bg-surface/75 p-3">
      <div className="text-xs font-semibold text-accent">步骤 {number}</div>
      <div className="mt-1 font-semibold text-text-primary">{title}</div>
      <p className="mt-1 text-xs leading-5 text-text-muted">{text}</p>
    </div>
  );
}

function HelpTerm({ term, description }: { term: string; description: string }) {
  return (
    <div>
      <dt className="font-semibold text-text-primary">{term}</dt>
      <dd className="mt-0.5 text-xs leading-5 text-text-muted">{description}</dd>
    </div>
  );
}

function SpecialRow({
  label,
  value,
  tone,
}: {
  label: string;
  value: string;
  tone: "success" | "warning" | "error";
}) {
  const dotClass = tone === "success" ? "bg-success" : tone === "warning" ? "bg-warning" : "bg-danger";
  return (
    <div className="rounded-lg border border-border bg-surface/80 px-3 py-2.5">
      <div className="flex items-center justify-between gap-3">
        <span className="text-sm font-semibold text-text-primary">{label}</span>
        <span className={`h-2 w-2 rounded-full ${dotClass}`} />
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
