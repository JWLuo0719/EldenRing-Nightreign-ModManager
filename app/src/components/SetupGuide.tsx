import { useState } from "react";
import { open } from "@tauri-apps/plugin-dialog";

interface SetupGuideProps {
  initialGamePath?: string;
  initialMe3Path?: string;
  initialLaunchExePath?: string;
  onSetupComplete: (
    gamePath: string,
    me3Path: string,
    launchExePath: string
  ) => void;
}

export function SetupGuide({
  initialGamePath = "",
  initialMe3Path = "",
  initialLaunchExePath = "",
  onSetupComplete,
}: SetupGuideProps) {
  const [gamePath, setGamePath] = useState(initialGamePath);
  const [me3Path, setMe3Path] = useState(initialMe3Path);
  const [launchExePath, setLaunchExePath] = useState(initialLaunchExePath);
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

  const handleComplete = () => {
    if (gamePath && me3Path) {
      onSetupComplete(gamePath, me3Path, launchExePath);
    }
  };

  return (
    <div className="flex-1 flex items-center justify-center bg-bg-primary overflow-auto">
      <div className="w-full max-w-2xl p-10">
        <div className="text-center mb-10">
          <div className="w-20 h-20 rounded-2xl bg-accent/20 flex items-center justify-center mx-auto mb-5 shadow-lg shadow-accent/10">
            <span className="text-4xl">🌙</span>
          </div>
          <h1 className="text-3xl font-bold text-text-primary mb-3">
            欢迎使用夜临 Mod 管理器
          </h1>
          <p className="text-base text-text-secondary">
            首次使用，请完成以下配置
          </p>
          <p className="text-xs text-text-muted mt-2">
            管理器会通过 ME3 启动游戏；直接运行游戏本体不会加载这里启用的 Mod。
          </p>
        </div>

        <div className="space-y-5">
          {/* Step 1: 游戏目录 */}
          <div
            className={`p-5 rounded-xl border-2 transition-all cursor-pointer ${
              step >= 1
                ? "bg-bg-secondary border-accent/30 shadow-lg shadow-accent/5"
                : "bg-bg-primary border-transparent opacity-50"
            }`}
            onClick={() => setStep(1)}
          >
            <div className="flex items-center gap-4 mb-4">
              <div
                className={`w-8 h-8 rounded-lg flex items-center justify-center text-sm font-bold transition-all ${
                  gamePath
                    ? "bg-success text-white scale-110"
                    : step === 1
                    ? "bg-accent text-white animate-pulse"
                    : "bg-bg-tertiary text-text-muted"
                }`}
              >
                {gamePath ? "✓" : "1"}
              </div>
              <div>
                <h3 className="text-base font-semibold text-text-primary">
                  选择游戏安装目录
                </h3>
                <p className="text-xs text-text-muted mt-0.5">
                  Elden Ring: Nightreign
                </p>
              </div>
            </div>

            {step === 1 && (
              <div className="space-y-4 ml-12">
                <div className="bg-bg-tertiary/50 rounded-lg p-3 border border-border/50">
                  <p className="text-xs text-text-secondary leading-relaxed">
                    <span className="text-accent font-medium">提示：</span>
                    请定位到游戏的<span className="text-text-primary font-medium">根目录</span>，
                    即包含 <code className="px-1.5 py-0.5 bg-bg-primary rounded text-accent text-[11px]">nightreign.exe</code> 的文件夹。
                  </p>
                  <p className="text-xs text-text-muted mt-2">
                    不要选择 <span className="text-text-secondary">mods</span> 子目录，也不要选择压缩包所在目录。
                  </p>
                </div>

                <div className="flex gap-3">
                  <input
                    type="text"
                    value={gamePath}
                    placeholder="请通过右侧按钮选择目录..."
                    readOnly
                    className="flex-1 px-4 py-2.5 bg-bg-tertiary border border-border rounded-lg text-sm text-text-primary placeholder-text-muted focus:outline-none focus:border-accent/50 transition-colors"
                  />
                  <button
                    onClick={(e) => {
                      e.stopPropagation();
                      selectGamePath();
                    }}
                    className="px-5 py-2.5 bg-accent hover:bg-accent-hover text-white text-sm font-medium rounded-lg transition-all hover:shadow-lg hover:shadow-accent/20"
                  >
                    浏览...
                  </button>
                </div>
              </div>
            )}
          </div>

          {/* 连接线 */}
          <div className="flex justify-center">
            <div className={`w-0.5 h-8 transition-colors ${gamePath ? 'bg-accent/30' : 'bg-border'}`} />
          </div>

          {/* Step 2: ME3 目录 */}
          <div
            className={`p-5 rounded-xl border-2 transition-all cursor-pointer ${
              step >= 2
                ? "bg-bg-secondary border-accent/30 shadow-lg shadow-accent/5"
                : "bg-bg-primary border-transparent opacity-50"
            }`}
            onClick={() => gamePath && setStep(2)}
          >
            <div className="flex items-center gap-4 mb-4">
              <div
                className={`w-8 h-8 rounded-lg flex items-center justify-center text-sm font-bold transition-all ${
                  me3Path
                    ? "bg-success text-white scale-110"
                    : step === 2
                    ? "bg-accent text-white animate-pulse"
                    : "bg-bg-tertiary text-text-muted"
                }`}
              >
                {me3Path ? "✓" : "2"}
              </div>
              <div>
                <h3 className="text-base font-semibold text-text-primary">
                  选择 ME3 工具目录
                </h3>
                <p className="text-xs text-text-muted mt-0.5">
                  Mod Engine 3
                </p>
              </div>
            </div>

            {step === 2 && (
              <div className="space-y-4 ml-12">
                <div className="bg-bg-tertiary/50 rounded-lg p-3 border border-border/50">
                  <p className="text-xs text-text-secondary leading-relaxed">
                    <span className="text-accent font-medium">提示：</span>
                    请定位到 ME3 的<span className="text-text-primary font-medium">安装目录</span>，
                    即包含 <code className="px-1.5 py-0.5 bg-bg-primary rounded text-accent text-[11px]">bin</code> 文件夹的目录。
                    如果已经进入 <code className="px-1.5 py-0.5 bg-bg-primary rounded text-accent text-[11px]">bin</code> 目录也可以保存。
                  </p>
                  <p className="text-xs text-text-muted mt-2">
                    需要能找到：<span className="text-text-secondary">me3/bin/me3.exe</span>
                  </p>
                </div>

                <div className="flex gap-3">
                  <input
                    type="text"
                    value={me3Path}
                    placeholder="请通过右侧按钮选择目录..."
                    readOnly
                    className="flex-1 px-4 py-2.5 bg-bg-tertiary border border-border rounded-lg text-sm text-text-primary placeholder-text-muted focus:outline-none focus:border-accent/50 transition-colors"
                  />
                  <button
                    onClick={(e) => {
                      e.stopPropagation();
                      selectMe3Path();
                    }}
                    className="px-5 py-2.5 bg-accent hover:bg-accent-hover text-white text-sm font-medium rounded-lg transition-all hover:shadow-lg hover:shadow-accent/20"
                  >
                    浏览...
                  </button>
                </div>
              </div>
            )}
          </div>

          <div className="flex justify-center">
            <div className={`w-0.5 h-8 transition-colors ${me3Path ? 'bg-accent/30' : 'bg-border'}`} />
          </div>

          {/* Step 3: 启动程序 */}
          <div
            className={`p-5 rounded-xl border-2 transition-all cursor-pointer ${
              step >= 3
                ? "bg-bg-secondary border-accent/30 shadow-lg shadow-accent/5"
                : "bg-bg-primary border-transparent opacity-50"
            }`}
            onClick={() => gamePath && me3Path && setStep(3)}
          >
            <div className="flex items-center gap-4 mb-4">
              <div
                className={`w-8 h-8 rounded-lg flex items-center justify-center text-sm font-bold transition-all ${
                  step === 3
                    ? "bg-accent text-bg-primary animate-pulse"
                    : launchExePath
                    ? "bg-success text-white scale-110"
                    : "bg-bg-tertiary text-text-muted"
                }`}
              >
                {launchExePath ? "✓" : "3"}
              </div>
              <div>
                <h3 className="text-base font-semibold text-text-primary">
                  启动程序（可选）
                </h3>
                <p className="text-xs text-text-muted mt-0.5">
                  默认使用 nightreign.exe；如安装了 Mod 启动器，可在这里指定
                </p>
              </div>
            </div>

            {step === 3 && (
              <div className="space-y-4 ml-12">
                <div className="bg-bg-tertiary/50 rounded-lg p-3 border border-border/50">
                  <p className="text-xs text-text-secondary leading-relaxed">
                    <span className="text-accent font-medium">提示：</span>
                    如果你需要通过某个 Mod 自带启动器进入游戏，请选择游戏根目录内的
                    <code className="mx-1 px-1.5 py-0.5 bg-bg-primary rounded text-accent text-[11px]">.exe</code>
                    文件。留空则使用
                    <code className="mx-1 px-1.5 py-0.5 bg-bg-primary rounded text-accent text-[11px]">nightreign.exe</code>。
                  </p>
                  <p className="text-xs text-text-muted mt-2">
                    为避免误启动外部程序，管理器只接受位于游戏目录内的 exe。
                  </p>
                </div>

                <div className="flex gap-3">
                  <input
                    type="text"
                    value={launchExePath}
                    placeholder="留空：使用 nightreign.exe"
                    readOnly
                    className="flex-1 px-4 py-2.5 bg-bg-tertiary border border-border rounded-lg text-sm text-text-primary placeholder-text-muted focus:outline-none focus:border-accent/50 transition-colors"
                  />
                  <button
                    onClick={(e) => {
                      e.stopPropagation();
                      selectLaunchExePath();
                    }}
                    className="px-5 py-2.5 bg-accent hover:bg-accent-hover text-bg-primary text-sm font-semibold rounded-lg transition-all hover:shadow-lg hover:shadow-accent/20"
                  >
                    浏览...
                  </button>
                  <button
                    onClick={(e) => {
                      e.stopPropagation();
                      setLaunchExePath("");
                    }}
                    className="px-4 py-2.5 bg-bg-tertiary hover:bg-bg-hover text-text-secondary text-sm font-medium rounded-lg transition-all"
                  >
                    默认
                  </button>
                </div>
              </div>
            )}
          </div>
        </div>

        {gamePath && me3Path && (
          <button
            onClick={handleComplete}
            className="w-full mt-8 px-6 py-4 bg-accent hover:bg-accent-hover text-white font-semibold rounded-xl transition-all hover:shadow-xl hover:shadow-accent/30 text-base"
          >
            开始使用
          </button>
        )}

        {!gamePath || !me3Path ? (
          <p className="text-center text-xs text-text-muted mt-6">
            完成以上配置后，回到主界面点击“启动”来加载已启用 Mod。
          </p>
        ) : null}
      </div>
    </div>
  );
}
