import { useState } from "react";
import type { FileConflict, LaunchArtifacts } from "../types/mod";
import { PageFrame } from "./LaunchPage";

type ViewerKey = "profile" | "script" | "log" | "diagnose";

interface DiagnosticsPageProps {
  busy: boolean;
  onDiagnose: () => Promise<string | undefined>;
  onGenerateProfile: () => Promise<{ profilePath: string; content: string } | undefined>;
  onReadArtifacts: () => Promise<LaunchArtifacts | undefined>;
  onDetectConflicts: () => Promise<FileConflict[] | undefined>;
  onToast: (message: string) => void;
}

export function DiagnosticsPage({
  busy,
  onDiagnose,
  onGenerateProfile,
  onReadArtifacts,
  onDetectConflicts,
  onToast,
}: DiagnosticsPageProps) {
  const [activeViewer, setActiveViewer] = useState<ViewerKey>("profile");
  const [diagnosticOutput, setDiagnosticOutput] = useState("");
  const [profilePath, setProfilePath] = useState("");
  const [profileContent, setProfileContent] = useState("");
  const [scriptPath, setScriptPath] = useState("");
  const [scriptContent, setScriptContent] = useState("");
  const [logPath, setLogPath] = useState("");
  const [logContent, setLogContent] = useState("");
  const [conflicts, setConflicts] = useState<FileConflict[]>([]);
  const [hasAnalyzedConflicts, setHasAnalyzedConflicts] = useState(false);

  const runDiagnose = async () => {
    const result = await onDiagnose();
    if (result) {
      setDiagnosticOutput(result);
      setActiveViewer("diagnose");
    }
  };

  const refreshProfile = async () => {
    const result = await onGenerateProfile();
    if (result) {
      setProfilePath(result.profilePath);
      setProfileContent(result.content);
      setActiveViewer("profile");
    }
  };

  const refreshArtifacts = async () => {
    const result = await onReadArtifacts();
    if (result) {
      setProfilePath(result.profilePath);
      setProfileContent(result.profileContent);
      setScriptPath(result.scriptPath);
      setScriptContent(result.scriptContent);
      setLogPath(result.logPath);
      setLogContent(result.logContent);
      setActiveViewer("log");
    }
  };

  const analyzeConflicts = async () => {
    const result = await onDetectConflicts();
    if (result) {
      setConflicts(result);
      setHasAnalyzedConflicts(true);
    }
  };

  const viewer = getViewerState(activeViewer, {
    diagnosticOutput,
    profilePath,
    profileContent,
    scriptPath,
    scriptContent,
    logPath,
    logContent,
  });

  const copyText = async (text: string) => {
    if (!text.trim()) {
      return;
    }
    await navigator.clipboard.writeText(text);
    onToast("已复制到剪贴板");
  };

  return (
    <PageFrame
      eyebrow="Diagnostics"
      title="冲突与诊断"
      description="集中查看启动文件、诊断输出和文件级冲突。"
    >
      <div className="grid min-h-0 flex-1 gap-4 xl:grid-cols-[22rem_1fr]">
        <aside className="flex min-h-0 flex-col gap-3 overflow-y-auto pr-1">
          <section className="rounded-xl border border-border bg-panel p-4">
            <h2 className="text-base font-semibold text-text-primary">启动诊断</h2>
            <p className="mt-2 text-xs leading-5 text-text-muted">
              这些操作只读取或生成诊断信息，不会修改 Mod 文件。
            </p>
            <div className="mt-4 grid gap-2">
              <ActionButton disabled={busy} label="生成 ME3 Profile" onClick={refreshProfile} />
              <ActionButton disabled={busy} label="读取脚本和日志" onClick={refreshArtifacts} />
              <ActionButton disabled={busy} label="执行启动诊断" onClick={runDiagnose} />
            </div>
          </section>

          <section className="flex min-h-0 flex-1 flex-col rounded-xl border border-border bg-panel">
            <div className="flex shrink-0 items-center justify-between border-b border-border p-4">
              <div>
                <h2 className="text-base font-semibold text-text-primary">文件级冲突</h2>
                <p className="mt-1 text-xs text-text-muted">仅分析当前启用的 Mod</p>
              </div>
              <button
                type="button"
                disabled={busy}
                onClick={() => void analyzeConflicts()}
                className="rounded-lg bg-accent px-3 py-2 text-sm font-semibold text-black transition-colors hover:bg-accent-hover disabled:opacity-50"
              >
                分析
              </button>
            </div>
            <div className="min-h-0 flex-1 overflow-y-auto p-3">
              {!hasAnalyzedConflicts ? (
                <p className="rounded-lg border border-dashed border-border p-4 text-sm leading-6 text-text-muted">
                  尚未分析。点击“分析”后会递归检查启用 Mod 的 package/native 文件路径。
                </p>
              ) : conflicts.length === 0 ? (
                <p className="rounded-lg border border-success/25 bg-success/10 p-4 text-sm leading-6 text-success">
                  未发现文件级冲突。
                </p>
              ) : (
                <div className="space-y-2">
                  {conflicts.map((conflict) => (
                    <div key={conflict.relativePath} className="rounded-lg border border-warning/30 bg-warning/10 p-3">
                      <div className="break-all text-xs font-semibold text-warning">{conflict.relativePath}</div>
                      <div className="mt-2 space-y-1">
                        {conflict.owners.map((owner) => (
                          <div
                            key={`${conflict.relativePath}-${owner.modId}`}
                            className="truncate text-xs leading-5 text-text-secondary"
                            title={owner.sourcePath}
                          >
                            {owner.modName}
                          </div>
                        ))}
                      </div>
                    </div>
                  ))}
                </div>
              )}
            </div>
          </section>
        </aside>

        <section className="flex min-h-0 flex-col rounded-xl border border-border bg-panel">
          <div className="shrink-0 border-b border-border p-3">
            <div className="flex flex-wrap items-center justify-between gap-3">
              <div className="flex rounded-lg border border-border bg-surface p-1">
                {viewerTabs.map((tab) => (
                  <button
                    key={tab.key}
                    type="button"
                    onClick={() => setActiveViewer(tab.key)}
                    className={`rounded-md px-3 py-1.5 text-xs font-semibold transition-colors ${
                      activeViewer === tab.key
                        ? "bg-accent text-black"
                        : "text-text-secondary hover:text-text-primary"
                    }`}
                  >
                    {tab.label}
                  </button>
                ))}
              </div>
              <button
                type="button"
                onClick={() => void copyText(viewer.content)}
                className="rounded-lg border border-border px-3 py-2 text-sm font-medium text-text-secondary transition-colors hover:bg-surface hover:text-text-primary"
              >
                复制当前内容
              </button>
            </div>
            <div className="mt-3 min-h-9 rounded-lg bg-surface px-3 py-2">
              <div className="text-sm font-semibold text-text-primary">{viewer.title}</div>
              {viewer.path && <div className="mt-1 truncate text-xs text-text-muted">{viewer.path}</div>}
            </div>
          </div>

          <pre className="min-h-0 flex-1 overflow-auto whitespace-pre-wrap p-4 font-mono text-xs leading-5 text-text-secondary">
            {viewer.content || viewer.empty}
          </pre>
        </section>
      </div>
    </PageFrame>
  );
}

const viewerTabs: Array<{ key: ViewerKey; label: string }> = [
  { key: "profile", label: "Profile" },
  { key: "script", label: "脚本" },
  { key: "log", label: "日志" },
  { key: "diagnose", label: "诊断输出" },
];

function ActionButton({
  label,
  disabled,
  onClick,
}: {
  label: string;
  disabled: boolean;
  onClick: () => void | Promise<void>;
}) {
  return (
    <button
      type="button"
      disabled={disabled}
      onClick={() => void onClick()}
      className="rounded-lg border border-border bg-surface px-4 py-3 text-left text-sm font-medium text-text-secondary transition-colors hover:border-accent/45 hover:text-text-primary disabled:cursor-not-allowed disabled:opacity-50"
    >
      {label}
    </button>
  );
}

function getViewerState(
  activeViewer: ViewerKey,
  data: {
    diagnosticOutput: string;
    profilePath: string;
    profileContent: string;
    scriptPath: string;
    scriptContent: string;
    logPath: string;
    logContent: string;
  }
) {
  switch (activeViewer) {
    case "script":
      return {
        title: "launch-nightreign.bat",
        path: data.scriptPath,
        content: data.scriptContent,
        empty: "尚未读取启动脚本。",
      };
    case "log":
      return {
        title: "last-launch.log",
        path: data.logPath,
        content: data.logContent,
        empty: "尚未读取启动日志。",
      };
    case "diagnose":
      return {
        title: "诊断输出",
        path: "",
        content: data.diagnosticOutput,
        empty: "尚未执行启动诊断。",
      };
    case "profile":
    default:
      return {
        title: "active-nightreign.me3",
        path: data.profilePath,
        content: data.profileContent,
        empty: "尚未生成 ME3 profile 预览。",
      };
  }
}
