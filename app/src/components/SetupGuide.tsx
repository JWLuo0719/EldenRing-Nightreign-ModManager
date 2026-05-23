import { useState } from "react";
import { open } from "@tauri-apps/plugin-dialog";

interface SetupGuideProps {
  initialGamePath?: string;
  initialMe3Path?: string;
  initialLaunchExePath?: string;
  busy?: boolean;
  onSetupComplete: (
    gamePath: string,
    me3Path: string,
    launchExePath: string
  ) => Promise<void> | void;
}

export function SetupGuide({
  initialGamePath = "",
  initialMe3Path = "",
  initialLaunchExePath = "",
  busy = false,
  onSetupComplete,
}: SetupGuideProps) {
  const [gamePath, setGamePath] = useState(initialGamePath);
  const [me3Path, setMe3Path] = useState(initialMe3Path);
  const [launchExePath, setLaunchExePath] = useState(initialLaunchExePath);

  const selectGamePath = async () => {
    const selected = await open({ directory: true, title: "选择游戏安装目录" });
    if (typeof selected === "string") {
      setGamePath(selected);
    }
  };

  const selectMe3Path = async () => {
    const selected = await open({ directory: true, title: "选择 ME3 目录" });
    if (typeof selected === "string") {
      setMe3Path(selected);
    }
  };

  const selectLaunchExePath = async () => {
    const selected = await open({
      multiple: false,
      title: "选择自定义启动程序",
      filters: [{ name: "Windows 可执行文件", extensions: ["exe"] }],
    });
    if (typeof selected === "string") {
      setLaunchExePath(selected);
    }
  };

  const canSubmit = Boolean(gamePath && me3Path && !busy);

  return (
    <div className="flex min-h-0 flex-1 items-center justify-center overflow-auto bg-app px-6 py-8">
      <div className="w-full max-w-4xl">
        <div className="mb-8">
          <div className="text-[11px] font-semibold uppercase tracking-[0.24em] text-accent">
            First Run
          </div>
          <h1 className="mt-3 text-3xl font-bold text-text-primary">完成启动环境配置</h1>
          <p className="mt-3 max-w-2xl text-sm leading-6 text-text-secondary">
            管理器需要游戏目录和 ME3 目录才能生成 active-nightreign.me3 并通过 ME3 启动。
            这些路径只保存在本机配置目录，不会写入源码。
          </p>
        </div>

        <div className="grid gap-4 lg:grid-cols-3">
          <SetupCard
            index="1"
            title="游戏目录"
            subtitle="包含 nightreign.exe 的 Game 文件夹"
            value={gamePath}
            placeholder="尚未选择游戏目录"
            actionLabel="浏览"
            done={Boolean(gamePath)}
            onBrowse={selectGamePath}
          />
          <SetupCard
            index="2"
            title="ME3 目录"
            subtitle="包含 bin/me3.exe，或直接选择 bin"
            value={me3Path}
            placeholder="尚未选择 ME3 目录"
            actionLabel="浏览"
            done={Boolean(me3Path)}
            onBrowse={selectMe3Path}
          />
          <SetupCard
            index="3"
            title="启动程序"
            subtitle="可选。默认使用 nightreign.exe"
            value={launchExePath}
            placeholder="默认：nightreign.exe"
            actionLabel="选择"
            done={Boolean(launchExePath)}
            optional
            onBrowse={selectLaunchExePath}
            onClear={() => setLaunchExePath("")}
          />
        </div>

        <div className="mt-6 rounded-xl border border-border bg-panel p-4 text-sm leading-6 text-text-secondary">
          SeamlessCoop/Spacewar 环境下，即使这里选择过 nrsc_launcher.exe，ME3 启动链路也会自动改用
          nightreign.exe，并通过 profile 注入 SeamlessCoop/nrsc.dll。
        </div>

        <div className="mt-8 flex justify-end">
          <button
            type="button"
            disabled={!canSubmit}
            onClick={() => void onSetupComplete(gamePath, me3Path, launchExePath)}
            className="rounded-lg bg-accent px-6 py-3 text-sm font-bold text-black transition-colors hover:bg-accent-hover disabled:cursor-not-allowed disabled:opacity-45"
          >
            进入工作台
          </button>
        </div>
      </div>
    </div>
  );
}

function SetupCard({
  index,
  title,
  subtitle,
  value,
  placeholder,
  actionLabel,
  done,
  optional,
  onBrowse,
  onClear,
}: {
  index: string;
  title: string;
  subtitle: string;
  value: string;
  placeholder: string;
  actionLabel: string;
  done: boolean;
  optional?: boolean;
  onBrowse: () => Promise<void>;
  onClear?: () => void;
}) {
  return (
    <section className="flex min-h-64 flex-col rounded-xl border border-border bg-panel p-5">
      <div className="flex items-center justify-between">
        <div className={`grid h-8 w-8 place-items-center rounded-md text-sm font-bold ${done ? "bg-success text-black" : "bg-surface text-text-muted"}`}>
          {done ? "✓" : index}
        </div>
        {optional && <span className="text-xs text-text-muted">可选</span>}
      </div>
      <h2 className="mt-5 text-lg font-semibold text-text-primary">{title}</h2>
      <p className="mt-2 text-sm leading-5 text-text-muted">{subtitle}</p>
      <div className="mt-5 min-h-16 rounded-lg border border-border bg-surface p-3 text-xs leading-5 text-text-secondary">
        {value || placeholder}
      </div>
      <div className="mt-auto flex gap-2 pt-5">
        <button
          type="button"
          onClick={() => void onBrowse()}
          className="rounded-lg bg-accent px-4 py-2 text-sm font-semibold text-black transition-colors hover:bg-accent-hover"
        >
          {actionLabel}
        </button>
        {onClear && (
          <button
            type="button"
            onClick={onClear}
            className="rounded-lg border border-border px-4 py-2 text-sm font-medium text-text-secondary transition-colors hover:bg-surface hover:text-text-primary"
          >
            默认
          </button>
        )}
      </div>
    </section>
  );
}
