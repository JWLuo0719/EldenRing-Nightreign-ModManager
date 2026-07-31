import { useState } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import { PageFrame } from "./LaunchPage";
import type { RuntimeEnvironment, RuntimeEnvironmentStatus } from "../types/mod";

interface SettingsPageProps {
  gamePath: string;
  me3Path: string;
  launchExePath: string;
  runtimeEnvironmentStatus: RuntimeEnvironmentStatus | null;
  busy: boolean;
  onSave: (
    gamePath: string,
    me3Path: string,
    launchExePath: string,
    runtimeEnvironment: RuntimeEnvironment
  ) => Promise<void>;
}

export function SettingsPage({
  gamePath,
  me3Path,
  launchExePath,
  runtimeEnvironmentStatus,
  busy,
  onSave,
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
      description="配置游戏目录、ME3 目录和可选启动程序。路径校验由 Rust 后端执行。"
    >
      <div className="grid min-h-0 flex-1 gap-4 lg:grid-cols-[minmax(0,1fr)_16rem]">
        <section className="panel-card rounded-xl p-5">
          <div className="section-label text-text-muted">Local paths</div>
          <h2 className="mt-1 text-lg font-semibold text-text-primary">本机路径</h2>
          <div className="mt-5 divide-y divide-border">
            <PathField
              label="游戏安装目录"
              hint="必须包含 nightreign.exe"
              value={draftGamePath}
              placeholder="选择包含 nightreign.exe 的 Game 文件夹"
              onChange={setDraftGamePath}
              onBrowse={selectGamePath}
            />
            <PathField
              label="ME3 目录"
              hint="包含 me3.exe 或 bin/me3.exe"
              value={draftMe3Path}
              placeholder="选择 ME3 根目录或 bin 目录"
              onChange={setDraftMe3Path}
              onBrowse={selectMe3Path}
            />
            <PathField
              label="启动程序"
              hint="可选，留空时使用 nightreign.exe"
              value={draftLaunchExePath}
              placeholder="留空：nightreign.exe"
              onChange={setDraftLaunchExePath}
              onBrowse={selectLaunchExePath}
              onClear={() => setDraftLaunchExePath("")}
            />
          </div>

          <div className="mt-5 border-t border-border pt-5">
            <div className="mb-2 flex items-end justify-between gap-3">
              <label className="text-sm font-semibold text-text-primary">运行环境</label>
              <span className="text-xs text-text-muted">
                检测：{runtimeEnvironmentStatus?.detected ?? "unknown"}
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
              <option value="steam_official">纯正版 Steam（未实测）</option>
              <option value="steam_seamless">正版 Steam + Seamless（未实测）</option>
              <option value="spacewar_seamless">Spacewar + Seamless（已实测）</option>
            </select>
            <p className="mt-2 text-xs leading-5 text-warning">
              当前仅 Spacewar + Seamless 完成真实环境回归。正版两种模式会使用更严格的门禁和保守启动参数。
            </p>
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
          <div className="section-label text-text-muted">Path rules</div>
          <h3 className="mt-1 text-sm font-semibold text-text-primary">选择提示</h3>
          <ol className="mt-4 space-y-4 text-xs leading-5 text-text-muted">
            <li className="flex gap-3"><RuleNumber value="01" /><span>游戏目录选择包含 <b className="text-text-secondary">nightreign.exe</b> 的 Game 文件夹。</span></li>
            <li className="flex gap-3"><RuleNumber value="02" /><span>ME3 可选择根目录，也可以直接选择 <b className="text-text-secondary">bin</b>。</span></li>
            <li className="flex gap-3"><RuleNumber value="03" /><span>联机启动器会自动转换为 ME3 可用的游戏目标。</span></li>
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
