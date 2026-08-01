import { useState } from "react";
import type {
  FileConflict,
  LaunchArtifacts,
  MultiplayerManifestComparison,
  MultiplayerManifestExport,
} from "../types/mod";
import { TechnicalGlossary } from "../components/TechnicalGlossary";
import { PageFrame } from "./LaunchPage";

type ViewerKey = "profile" | "script" | "log" | "diagnose" | "multiplayer";

interface DiagnosticsPageProps {
  busy: boolean;
  onDiagnose: () => Promise<string | undefined>;
  onGenerateProfile: () => Promise<{ profilePath: string; content: string } | undefined>;
  onReadArtifacts: () => Promise<LaunchArtifacts | undefined>;
  onDetectConflicts: () => Promise<FileConflict[] | undefined>;
  onExportMultiplayerManifest: () => Promise<MultiplayerManifestExport | undefined>;
  onCompareMultiplayerManifest: () => Promise<MultiplayerManifestComparison | undefined>;
  onToast: (message: string) => void;
}

export function DiagnosticsPage({
  busy,
  onDiagnose,
  onGenerateProfile,
  onReadArtifacts,
  onDetectConflicts,
  onExportMultiplayerManifest,
  onCompareMultiplayerManifest,
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
  const [multiplayerPath, setMultiplayerPath] = useState("");
  const [multiplayerContent, setMultiplayerContent] = useState("");
  const [multiplayerComparison, setMultiplayerComparison] =
    useState<MultiplayerManifestComparison | null>(null);
  const visibleConflicts = conflicts.slice(0, 500);

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

  const exportManifest = async () => {
    const result = await onExportMultiplayerManifest();
    if (result) {
      setMultiplayerPath(result.path);
      setMultiplayerContent(JSON.stringify(result.manifest, null, 2));
      setMultiplayerComparison(null);
      setActiveViewer("multiplayer");
    }
  };

  const compareManifest = async () => {
    const result = await onCompareMultiplayerManifest();
    if (result) {
      setMultiplayerPath(result.compatible ? "双方清单一致" : "双方清单存在差异");
      setMultiplayerContent(formatMultiplayerComparison(result));
      setMultiplayerComparison(result);
      setActiveViewer("multiplayer");
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
    multiplayerPath,
    multiplayerContent,
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
      <div className="grid min-h-0 flex-1 gap-4 lg:grid-cols-[18rem_minmax(0,1fr)]">
        <aside className="flex min-h-0 flex-col gap-3 overflow-y-auto pr-1">
          <section className="panel-card rounded-xl p-4">
            <h2 className="text-base font-semibold text-text-primary">启动诊断</h2>
            <p className="mt-2 text-xs leading-5 text-text-muted">
              生成启动配置和读取日志不会修改 Mod 文件；“启动游戏并诊断”会真实打开 ME3 和游戏。
            </p>
            <div className="mt-4 grid gap-2">
              <ActionButton disabled={busy} label="生成启动配置" onClick={refreshProfile} />
              <ActionButton disabled={busy} label="读取脚本和日志" onClick={refreshArtifacts} />
              <ActionButton disabled={busy} label="启动游戏并诊断" onClick={runDiagnose} />
            </div>
            <div className="mt-3">
              <TechnicalGlossary />
            </div>
          </section>

          <section className="panel-card rounded-xl p-4">
            <h2 className="text-base font-semibold text-text-primary">双方联机一致性</h2>
            <p className="mt-2 text-xs leading-5 text-text-muted">
              导出脱敏指纹，比较游戏、联机功能插件、设置、完整资源内容与加载顺序。
              大型整合首次读取可能需要 1–2 分钟。
            </p>
            <div className="mt-4 grid gap-2 sm:grid-cols-2 lg:grid-cols-1">
              <ActionButton disabled={busy} label="导出本机清单" onClick={exportManifest} />
              <ActionButton disabled={busy} label="比较好友清单" onClick={compareManifest} />
            </div>
            {multiplayerComparison && (
              <div
                className={`mt-3 rounded-lg border p-3 text-xs leading-5 ${
                  multiplayerComparison.compatible
                    ? "border-success/25 bg-success/10 text-success"
                    : "border-warning/30 bg-warning/10 text-warning"
                }`}
              >
                {multiplayerComparison.compatible
                  ? "关键文件、设置和加载顺序一致，可以进入双端游戏验收。"
                  : `发现 ${multiplayerComparison.differences.filter((item) => item.severity === "error").length} 项阻断差异；先修正后再联机。`}
              </div>
            )}
          </section>

          <section className="panel-card flex min-h-0 flex-1 flex-col rounded-xl">
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
                  尚未分析。点击“分析”后会递归检查启用的资源型 Mod 与功能插件路径。
                </p>
              ) : conflicts.length === 0 ? (
                <p className="rounded-lg border border-success/25 bg-success/10 p-4 text-sm leading-6 text-success">
                  未发现文件级冲突。
                </p>
              ) : (
                <div className="space-y-2">
                  {conflicts.length > visibleConflicts.length && (
                    <p className="rounded-lg border border-warning/30 bg-warning/10 p-3 text-xs leading-5 text-warning">
                      冲突较多，为控制界面内存仅显示前 {visibleConflicts.length} 条，共 {conflicts.length} 条。
                    </p>
                  )}
                  {visibleConflicts.map((conflict) => (
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

        <section className="panel-card flex min-h-0 flex-col rounded-xl">
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
  { key: "profile", label: "启动配置" },
  { key: "script", label: "脚本" },
  { key: "log", label: "日志" },
  { key: "diagnose", label: "诊断输出" },
  { key: "multiplayer", label: "一致性清单" },
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
    multiplayerPath: string;
    multiplayerContent: string;
  }
) {
  switch (activeViewer) {
    case "multiplayer":
      return {
        title: "双方联机一致性",
        path: data.multiplayerPath,
        content: data.multiplayerContent,
        empty: "尚未导出或比较联机一致性清单。",
      };
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
        empty: "尚未生成启动配置预览。高级内容为 ME3 Profile（.me3）。",
      };
  }
}

function formatMultiplayerComparison(result: MultiplayerManifestComparison) {
  const errors = result.differences.filter((item) => item.severity === "error");
  const warnings = result.differences.filter((item) => item.severity === "warning");
  const lines = [
    result.compatible ? "结论：双方关键内容一致" : "结论：双方存在阻断差异",
    `本机总体指纹：${result.local.overallSha256}`,
    `好友总体指纹：${result.peer.overallSha256}`,
    `阻断差异：${errors.length}；提醒：${warnings.length}`,
  ];
  for (const difference of result.differences) {
    lines.push(
      "",
      `[${difference.severity === "error" ? "阻断" : "提醒"}] ${difference.category} / ${difference.item}`,
      `本机：${difference.local}`,
      `好友：${difference.peer}`
    );
  }
  return lines.join("\n");
}
