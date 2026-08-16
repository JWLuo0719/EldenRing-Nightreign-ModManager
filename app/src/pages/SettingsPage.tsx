import { useState } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import { PageFrame } from "./LaunchPage";
import type { RuntimeEnvironment, RuntimeEnvironmentStatus } from "../types/mod";

interface SettingsPageProps {
  gamePath: string;
  me3Path: string;
  launchExePath: string;
  runtimeEnvironmentStatus: RuntimeEnvironmentStatus | null;
  communityCompatibilityMode: boolean;
  busy: boolean;
  onSave: (
    gamePath: string,
    me3Path: string,
    launchExePath: string,
    runtimeEnvironment: RuntimeEnvironment
  ) => Promise<void>;
  onCommunityCompatibilityModeChange: (enabled: boolean) => void;
}

export function SettingsPage({
  gamePath,
  me3Path,
  launchExePath,
  runtimeEnvironmentStatus,
  communityCompatibilityMode,
  busy,
  onSave,
  onCommunityCompatibilityModeChange,
}: SettingsPageProps) {
  const [draftGamePath, setDraftGamePath] = useState(gamePath);
  const [draftMe3Path, setDraftMe3Path] = useState(me3Path);
  const [draftLaunchExePath, setDraftLaunchExePath] = useState(launchExePath);
  const [draftRuntimeEnvironment, setDraftRuntimeEnvironment] =
    useState<RuntimeEnvironment>(runtimeEnvironmentStatus?.configured ?? "auto");

  const selectGamePath = async () => {
    const selected = await open({ directory: true, title: "选择游戏安装目录" });
    if (typeof selected === "string") {
      setDraftGamePath(selected);
    }
  };

  const selectMe3Path = async () => {
    const selected = await open({ directory: true, title: "选择 ME3 目录" });
    if (typeof selected === "string") {
      setDraftMe3Path(selected);
    }
  };

  const selectLaunchExePath = async () => {
    const selected = await open({
      multiple: false,
      title: "选择自定义启动程序",
      filters: [{ name: "Windows 可执行文件", extensions: ["exe"] }],
    });
    if (typeof selected === "string") {
      setDraftLaunchExePath(selected);
    }
  };

  return (
    <PageFrame
      eyebrow="Settings"
      title="设置"
      description="告诉管理器游戏在哪、用哪个加载工具，以及你平时采用哪种联机方式。"
    >
      <div className="grid min-h-0 flex-1 gap-4 lg:grid-cols-[minmax(0,1fr)_16rem]">
        <section className="panel-card rounded-xl p-5">
          <div className="section-label text-text-muted">游戏启动设置</div>
          <h2 className="mt-1 text-lg font-semibold text-text-primary">本机路径</h2>
          <div className="mt-5 divide-y divide-border">
            <PathField
              label="游戏所在文件夹"
              hint="请选择 Game 文件夹"
              value={draftGamePath}
              placeholder="选择安装目录中的 Game 文件夹"
              onChange={setDraftGamePath}
              onBrowse={selectGamePath}
            />
            <PathField
              label="Mod 加载工具（ME3）"
              hint="选择 ME3 文件夹"
              value={draftMe3Path}
              placeholder="选择 ME3 根目录或 bin 目录"
              onChange={setDraftMe3Path}
              onBrowse={selectMe3Path}
            />
            <PathField
              label="额外启动程序（可选）"
              hint="通常保持默认即可"
              value={draftLaunchExePath}
              placeholder="留空：使用游戏主程序"
              onChange={setDraftLaunchExePath}
              onBrowse={selectLaunchExePath}
              onClear={() => setDraftLaunchExePath("")}
            />
          </div>

          <div className="mt-5 border-t border-border pt-5">
            <div className="mb-2 flex items-end justify-between gap-3">
              <label className="text-sm font-semibold text-text-primary">你平时如何联机</label>
              <span className="text-xs text-text-muted">
                自动检测：{runtimeEnvironmentLabel(runtimeEnvironmentStatus?.detected)}
              </span>
            </div>
            <select
              value={draftRuntimeEnvironment}
              onChange={(event) =>
                setDraftRuntimeEnvironment(event.target.value as RuntimeEnvironment)
              }
              className="w-full rounded-lg border border-border bg-surface px-3 py-2.5 text-sm text-text-primary outline-none focus:border-accent/65"
            >
              <option value="auto">自动检测</option>
              <option value="steam_official">官方 Steam 游玩（未实测）</option>
              <option value="steam_seamless">Steam + 社区联机插件（未实测）</option>
              <option value="spacewar_seamless">社区联机：Spacewar + Seamless（已实测）</option>
            </select>
            <p className="mt-2 text-xs leading-5 text-warning">
              当前只有“社区联机：Spacewar + Seamless”完成本机真实验证。它不能和官方 Steam 或作者指定的正版联机文件共用同一个 Game 目录。
            </p>
          </div>

          <div className="mt-5 border-t border-border pt-5">
            <div className="flex flex-wrap items-start justify-between gap-4">
              <div className="min-w-0 flex-1">
                <div className="flex flex-wrap items-center gap-2">
                  <h3 className="text-sm font-semibold text-text-primary">MMV 社区兼容方式</h3>
                  <span
                    className={`rounded-full border px-2 py-0.5 text-[10px] font-semibold ${
                      communityCompatibilityMode
                        ? "border-warning/30 bg-warning/10 text-warning"
                        : "border-border bg-surface text-text-muted"
                    }`}
                  >
                    {communityCompatibilityMode ? "已启用" : "使用作者方式"}
                  </span>
                </div>
                <p className="mt-2 max-w-2xl text-xs leading-5 text-text-muted">
                  这是独立的全局设置，不需要先安装 MMV。启用后，今后注册或启用符合条件的作者启动配置时，管理器会在生成副本中改用社区联机插件；原始 Mod 和作者文件保持不变。
                </p>
              </div>
              <button
                type="button"
                disabled={busy}
                onClick={() =>
                  onCommunityCompatibilityModeChange(!communityCompatibilityMode)
                }
                className={`shrink-0 rounded-lg border px-4 py-2.5 text-sm font-semibold transition-colors disabled:cursor-not-allowed disabled:opacity-50 ${
                  communityCompatibilityMode
                    ? "border-border bg-surface text-text-secondary hover:text-text-primary"
                    : "border-warning/35 bg-warning/10 text-warning hover:border-warning/60 hover:bg-warning/15"
                }`}
              >
                {communityCompatibilityMode ? "恢复作者联机方式" : "启用社区兼容方式"}
              </button>
            </div>
          </div>

          <div className="mt-5 flex justify-end border-t border-border pt-4">
            <button
              type="button"
              disabled={busy || !draftGamePath || !draftMe3Path}
              onClick={() =>
                void onSave(
                  draftGamePath,
                  draftMe3Path,
                  draftLaunchExePath,
                  draftRuntimeEnvironment
                )
              }
              className="rounded-lg bg-accent px-5 py-2.5 text-sm font-bold text-black transition-colors hover:bg-accent-hover disabled:cursor-not-allowed disabled:opacity-50"
            >
              保存设置
            </button>
          </div>
        </section>

        <aside className="panel-card h-fit rounded-xl p-4">
          <div className="section-label text-text-muted">选择提示</div>
          <h3 className="mt-1 text-sm font-semibold text-text-primary">选择提示</h3>
          <ol className="mt-4 space-y-4 text-xs leading-5 text-text-muted">
            <li className="flex gap-3"><RuleNumber value="01" /><span>选择游戏安装目录中的 <b className="text-text-secondary">Game</b> 文件夹；管理器会自动检查游戏主程序（nightreign.exe）。</span></li>
            <li className="flex gap-3"><RuleNumber value="02" /><span>ME3 可以选择根目录，也可以直接选择 <b className="text-text-secondary">bin</b> 文件夹。</span></li>
            <li className="flex gap-3"><RuleNumber value="03" /><span>不确定联机方式时先保持“自动检测”；不要把两种联机文件混放到同一个 Game 目录。</span></li>
            <li className="flex gap-3"><RuleNumber value="04" /><span>社区兼容方式可以提前设置；只有检测到符合条件的 MMV 作者启动配置时才会实际改写生成副本。</span></li>
          </ol>
        </aside>
      </div>
    </PageFrame>
  );
}

function PathField({
  label,
  hint,
  value,
  placeholder,
  onChange,
  onBrowse,
  onClear,
}: {
  label: string;
  hint: string;
  value: string;
  placeholder: string;
  onChange: (value: string) => void;
  onBrowse: () => Promise<void>;
  onClear?: () => void;
}) {
  return (
    <div className="py-4 first:pt-0 last:pb-0">
      <div className="mb-2 flex items-end justify-between gap-3">
        <label className="text-sm font-semibold text-text-primary">{label}</label>
        <span className="text-xs text-text-muted">{hint}</span>
      </div>
      <div className="flex gap-2">
        <input
          value={value}
          onChange={(event) => onChange(event.target.value)}
          placeholder={placeholder}
          className="min-w-0 flex-1 rounded-lg border border-border bg-surface px-3 py-2.5 text-sm text-text-primary outline-none transition-colors placeholder:text-text-muted focus:border-accent/65"
        />
        <button
          type="button"
          onClick={() => void onBrowse()}
          className="rounded-lg border border-border px-4 py-2.5 text-sm font-medium text-text-secondary transition-colors hover:bg-surface hover:text-text-primary"
        >
          浏览
        </button>
        {onClear && (
          <button
            type="button"
            onClick={onClear}
            className="rounded-lg border border-border px-4 py-2.5 text-sm font-medium text-text-secondary transition-colors hover:bg-surface hover:text-text-primary"
          >
            默认
          </button>
        )}
      </div>
    </div>
  );
}

function RuleNumber({ value }: { value: string }) {
  return <span className="display-number shrink-0 text-[10px] font-semibold text-accent">{value}</span>;
}

function runtimeEnvironmentLabel(value: string | undefined) {
  const labels: Record<string, string> = {
    auto: "自动检测",
    steam_official: "官方 Steam 游玩",
    steam_seamless: "Steam + 社区联机插件",
    spacewar_seamless: "社区联机（Spacewar + Seamless）",
    unknown_mixed: "需要确认的混合环境",
  };
  return value ? labels[value] ?? value : "尚未检测";
}
