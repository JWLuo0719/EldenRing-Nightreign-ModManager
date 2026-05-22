import { useState } from "react";
import { open } from "@tauri-apps/plugin-dialog";

interface SetupGuideProps {
  onSetupComplete: (gamePath: string, me3Path: string) => void;
}

export function SetupGuide({ onSetupComplete }: SetupGuideProps) {
  const [gamePath, setGamePath] = useState("");
  const [me3Path, setMe3Path] = useState("");
  const [step, setStep] = useState(1);

  const selectGamePath = async () => {
    const selected = await open({
      directory: true,
      title: "选择游戏安装目录",
    });
    if (selected) {
      setGamePath(selected as string);
    }
  };

  const selectMe3Path = async () => {
    const selected = await open({
      directory: true,
      title: "选择 ME3 安装目录",
    });
    if (selected) {
      setMe3Path(selected as string);
    }
  };

  const handleComplete = () => {
    if (gamePath && me3Path) {
      onSetupComplete(gamePath, me3Path);
    }
  };

  return (
    <div className="flex-1 flex items-center justify-center bg-bg-primary">
      <div className="w-full max-w-md p-8">
        <div className="text-center mb-8">
          <div className="w-16 h-16 rounded-2xl bg-accent/20 flex items-center justify-center mx-auto mb-4">
            <span className="text-3xl">🌙</span>
          </div>
          <h1 className="text-2xl font-bold text-text-primary mb-2">
            欢迎使用夜临 Mod 管理器
          </h1>
          <p className="text-sm text-text-muted">
            首次使用，请完成以下设置
          </p>
        </div>

        <div className="space-y-6">
          <div
            className={`p-4 rounded-lg border transition-all cursor-pointer ${
              step >= 1
                ? "bg-bg-secondary border-border"
                : "bg-bg-primary border-transparent opacity-50"
            }`}
            onClick={() => setStep(1)}
          >
            <div className="flex items-center gap-3 mb-3">
              <div
                className={`w-6 h-6 rounded-full flex items-center justify-center text-xs font-bold ${
                  gamePath
                    ? "bg-success text-white"
                    : step === 1
                    ? "bg-accent text-white"
                    : "bg-bg-tertiary text-text-muted"
                }`}
              >
                {gamePath ? "✓" : "1"}
              </div>
              <span className="text-sm font-medium text-text-primary">
                选择游戏目录
              </span>
            </div>

            {step === 1 && (
              <div className="space-y-3">
                <p className="text-xs text-text-muted">
                  选择艾尔登法环：黑夜君临的安装目录
                </p>
                <div className="flex gap-2">
                  <input
                    type="text"
                    value={gamePath}
                    placeholder="点击右侧按钮选择目录..."
                    readOnly
                    className="flex-1 px-3 py-2 bg-bg-tertiary border border-border rounded text-sm text-text-primary placeholder-text-muted focus:outline-none"
                  />
                  <button
                    onClick={(e) => {
                      e.stopPropagation();
                      selectGamePath();
                    }}
                    className="px-4 py-2 bg-accent hover:bg-accent-hover text-white text-sm rounded transition-colors"
                  >
                    浏览
                  </button>
                </div>
              </div>
            )}
          </div>

          <div
            className={`p-4 rounded-lg border transition-all cursor-pointer ${
              step >= 2
                ? "bg-bg-secondary border-border"
                : "bg-bg-primary border-transparent opacity-50"
            }`}
            onClick={() => gamePath && setStep(2)}
          >
            <div className="flex items-center gap-3 mb-3">
              <div
                className={`w-6 h-6 rounded-full flex items-center justify-center text-xs font-bold ${
                  me3Path
                    ? "bg-success text-white"
                    : step === 2
                    ? "bg-accent text-white"
                    : "bg-bg-tertiary text-text-muted"
                }`}
              >
                {me3Path ? "✓" : "2"}
              </div>
              <span className="text-sm font-medium text-text-primary">
                选择 ME3 目录
              </span>
            </div>

            {step === 2 && (
              <div className="space-y-3">
                <p className="text-xs text-text-muted">
                  选择 Mod Engine 3 的安装目录
                </p>
                <div className="flex gap-2">
                  <input
                    type="text"
                    value={me3Path}
                    placeholder="点击右侧按钮选择目录..."
                    readOnly
                    className="flex-1 px-3 py-2 bg-bg-tertiary border border-border rounded text-sm text-text-primary placeholder-text-muted focus:outline-none"
                  />
                  <button
                    onClick={(e) => {
                      e.stopPropagation();
                      selectMe3Path();
                    }}
                    className="px-4 py-2 bg-accent hover:bg-accent-hover text-white text-sm rounded transition-colors"
                  >
                    浏览
                  </button>
                </div>
              </div>
            )}
          </div>
        </div>

        {gamePath && me3Path && (
          <button
            onClick={handleComplete}
            className="w-full mt-6 px-4 py-3 bg-accent hover:bg-accent-hover text-white font-medium rounded-lg transition-colors"
          >
            开始使用
          </button>
        )}
      </div>
    </div>
  );
}
