import { useState } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import { PageFrame } from "./LaunchPage";

interface SettingsPageProps {
  gamePath: string;
  me3Path: string;
  launchExePath: string;
  busy: boolean;
  onSave: (gamePath: string, me3Path: string, launchExePath: string) => Promise<void>;
}

export function SettingsPage({
  gamePath,
  me3Path,
  launchExePath,
  busy,
  onSave,
}: SettingsPageProps) {
  const [draftGamePath, setDraftGamePath] = useState(gamePath);
  const [draftMe3Path, setDraftMe3Path] = useState(me3Path);
  const [draftLaunchExePath, setDraftLaunchExePath] = useState(launchExePath);

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
      <section className="max-w-4xl rounded-xl border border-border bg-panel p-5">
        <div className="space-y-4">
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
            hint="可选。留空时使用 nightreign.exe"
            value={draftLaunchExePath}
            placeholder="留空：nightreign.exe"
            onChange={setDraftLaunchExePath}
            onBrowse={selectLaunchExePath}
            onClear={() => setDraftLaunchExePath("")}
          />
        </div>

        <div className="mt-6 flex justify-end">
          <button
            type="button"
            disabled={busy || !draftGamePath || !draftMe3Path}
            onClick={() => void onSave(draftGamePath, draftMe3Path, draftLaunchExePath)}
            className="rounded-lg bg-accent px-5 py-2.5 text-sm font-bold text-black transition-colors hover:bg-accent-hover disabled:cursor-not-allowed disabled:opacity-50"
          >
            保存设置
          </button>
        </div>
      </section>
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
    <div>
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
