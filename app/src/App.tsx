import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { Titlebar } from "./components/Titlebar";
import { Sidebar } from "./components/Sidebar";
import { ModList } from "./components/ModList";
import { SetupGuide } from "./components/SetupGuide";
import type { ModInfo, Profile, ProfileMod } from "./types/mod";

type ToastType = "success" | "error" | "info";

interface Toast {
  id: number;
  type: ToastType;
  message: string;
}

interface ConfirmState {
  title: string;
  message: string;
  confirmText: string;
  danger?: boolean;
  onConfirm: () => Promise<void>;
}

function snapshotMods(mods: ModInfo[]): ProfileMod[] {
  return mods.map((mod, index) => ({
    modId: mod.id,
    enabled: mod.enabled,
    loadOrder: index + 1,
  }));
}

function App() {
  const [showSetup, setShowSetup] = useState(false);
  const [loading, setLoading] = useState(true);
  const [busy, setBusy] = useState(false);
  const [mods, setMods] = useState<ModInfo[]>([]);
  const [profiles, setProfiles] = useState<Profile[]>([]);
  const [activeProfile, setActiveProfile] = useState<Profile | null>(null);
  const [gamePath, setGamePath] = useState("");
  const [me3Path, setMe3Path] = useState("");
  const [launchExePath, setLaunchExePath] = useState("");
  const [toasts, setToasts] = useState<Toast[]>([]);
  const [confirmState, setConfirmState] = useState<ConfirmState | null>(null);

  const pushToast = useCallback((type: ToastType, message: string) => {
    const id = Date.now();
    setToasts((items) => [...items, { id, type, message }]);
    window.setTimeout(() => {
      setToasts((items) => items.filter((item) => item.id !== id));
    }, 3600);
  }, []);

  const loadData = useCallback(async () => {
    const [modsData, profilesData, activeData] = await Promise.all([
      invoke<ModInfo[]>("scan_mods"),
      invoke<Profile[]>("get_profiles"),
      invoke<Profile | null>("get_active_profile"),
    ]);
    setMods(modsData);
    setProfiles(profilesData);
    setActiveProfile(activeData);
    return { modsData, profilesData, activeData };
  }, []);

  const runTask = useCallback(
    async (task: () => Promise<void>, fallbackMessage: string) => {
      try {
        setBusy(true);
        await task();
      } catch (e) {
        console.error(fallbackMessage, e);
        const message = `${fallbackMessage}：${String(e)}`;
        pushToast("error", message);
        setConfirmState({
          title: fallbackMessage,
          message,
          confirmText: "知道了",
          danger: true,
          onConfirm: async () => {},
        });
      } finally {
        setBusy(false);
      }
    },
    [pushToast]
  );

  const checkSetup = useCallback(async () => {
    try {
      const gamePath = await invoke<string>("get_game_path");
      const me3Path = await invoke<string>("get_me3_path");
      const launchExePath = await invoke<string>("get_launch_exe_path");
      setGamePath(gamePath);
      setMe3Path(me3Path);
      setLaunchExePath(launchExePath);
      const configured = !!gamePath && !!me3Path;
      setShowSetup(!configured);
      if (configured) {
        await loadData();
      }
    } catch (e) {
      console.error("检查配置失败:", e);
      pushToast("error", `检查配置失败：${String(e)}`);
    } finally {
      setLoading(false);
    }
  }, [loadData, pushToast]);

  useEffect(() => {
    // Initial Tauri config hydration drives the first screen.
    // eslint-disable-next-line react-hooks/set-state-in-effect
    void checkSetup();
  }, [checkSetup]);

  const handleSetupComplete = async (
    nextGamePath: string,
    nextMe3Path: string,
    nextLaunchExePath: string
  ) => {
    await runTask(async () => {
      await invoke("set_game_path", { path: nextGamePath });
      await invoke("set_me3_path", { path: nextMe3Path });
      await invoke("set_launch_exe_path", { path: nextLaunchExePath });
      setGamePath(nextGamePath);
      setMe3Path(nextMe3Path);
      setLaunchExePath(nextLaunchExePath);
      setShowSetup(false);
      await loadData();
      pushToast("success", "路径配置已保存");
    }, "保存配置失败");
  };

  const handleShowSettings = () => {
    setShowSetup(true);
  };

  const handleToggleMod = async (mod: ModInfo) => {
    await runTask(async () => {
      const nextEnabled = !mod.enabled;
      await invoke("toggle_mod", {
        modPath: mod.path,
        enabled: nextEnabled,
      });
      if (activeProfile) {
        await invoke<Profile | null>("update_active_profile_mod", {
          modId: mod.id,
          enabled: nextEnabled,
        });
      }
      await loadData();
      pushToast("success", `${mod.name} 已${nextEnabled ? "启用" : "禁用"}`);
    }, "切换 Mod 状态失败");
  };

  const handleDeleteMod = (mod: ModInfo) => {
    setConfirmState({
      title: "删除 Mod",
      message: `确定删除「${mod.name}」吗？此操作会移除整个 Mod 文件夹，无法从管理器内撤销。`,
      confirmText: "删除",
      danger: true,
      onConfirm: async () => {
        await runTask(async () => {
          await invoke("uninstall_mod", { modPath: mod.path });
          if (activeProfile) {
            await invoke("update_profile", {
              profile: {
                ...activeProfile,
                mods: activeProfile.mods.filter((item) => item.modId !== mod.id),
              },
            });
          }
          await loadData();
          pushToast("success", `${mod.name} 已删除`);
        }, "删除 Mod 失败");
      },
    });
  };

  const handleProfileSelect = async (profile: Profile) => {
    await runTask(async () => {
      await invoke("activate_profile", { profileId: profile.id });

      const desiredState = new Map(
        profile.mods.map((item) => [item.modId, item.enabled])
      );
      for (const mod of mods) {
        const desiredEnabled = desiredState.get(mod.id);
        if (desiredEnabled !== undefined && desiredEnabled !== mod.enabled) {
          await invoke("toggle_mod", {
            modPath: mod.path,
            enabled: desiredEnabled,
          });
        }
      }

      await loadData();
      pushToast("info", `已切换到「${profile.name}」`);
    }, "切换方案失败");
  };

  const handleProfileCreate = async () => {
    await runTask(async () => {
      const newProfile = await invoke<Profile>("create_profile", {
        name: `方案 ${profiles.length + 1}`,
        description: "从当前启用状态创建",
        icon: "◆",
      });
      const seededProfile: Profile = {
        ...newProfile,
        mods: snapshotMods(mods),
        isActive: true,
      };
      await invoke("update_profile", { profile: seededProfile });
      await invoke("activate_profile", { profileId: seededProfile.id });
      await loadData();
      pushToast("success", `已创建「${seededProfile.name}」`);
    }, "创建方案失败");
  };

  const handleInstallMod = async () => {
    const selected = await open({
      multiple: false,
      title: "选择 Mod ZIP 压缩包",
      filters: [{ name: "ZIP 压缩包", extensions: ["zip"] }],
    });

    if (typeof selected !== "string") {
      return;
    }

    await runTask(async () => {
      await invoke("install_mod_from_zip", { zipPath: selected });
      await loadData();
      pushToast("success", "Mod 已安装，请检查结构后再启动游戏");
    }, "安装 Mod 失败");
  };

  const handleLaunchGame = async () => {
    await runTask(async () => {
      const result = await invoke<string>("launch_game", {
        gamePath: "",
        me3Path: "",
      });
      pushToast("success", result.split("\n")[0] || "已通过 ME3 启动游戏");
    }, "启动游戏失败");
  };

  const handleConfirm = async () => {
    const current = confirmState;
    setConfirmState(null);
    await current?.onConfirm();
  };

  const shellClass =
    "h-screen flex flex-col bg-bg-primary text-text-primary overflow-hidden";

  if (loading) {
    return (
      <div className={shellClass}>
        <Titlebar />
        <div className="flex-1 flex items-center justify-center bg-[radial-gradient(circle_at_center,var(--color-accent-dim),transparent_45%)]">
          <div className="flex flex-col items-center gap-4 rounded-3xl border border-border/70 bg-bg-secondary/80 px-10 py-8 shadow-2xl">
            <div className="w-9 h-9 border-2 border-accent border-t-transparent rounded-full animate-spin" />
            <div className="text-text-muted text-sm tracking-[0.24em] uppercase">
              Loading Mod Workspace
            </div>
          </div>
        </div>
      </div>
    );
  }

  if (showSetup) {
    return (
      <div className={shellClass}>
        <Titlebar />
        <SetupGuide
          initialGamePath={gamePath}
          initialMe3Path={me3Path}
          initialLaunchExePath={launchExePath}
          onSetupComplete={handleSetupComplete}
        />
        <ToastStack toasts={toasts} />
      </div>
    );
  }

  return (
    <div className={shellClass}>
      <Titlebar onSettingsClick={handleShowSettings} />
      <div className="flex-1 flex overflow-hidden bg-[linear-gradient(135deg,rgba(231,111,81,0.08),transparent_34%),radial-gradient(circle_at_82%_12%,rgba(233,196,106,0.12),transparent_30%)]">
        <Sidebar
          profiles={profiles}
          activeProfile={activeProfile}
          onProfileSelect={handleProfileSelect}
          onProfileCreate={handleProfileCreate}
          modsCount={mods.length}
        />
        <ModList
          mods={mods}
          activeProfile={activeProfile}
          launchExePath={launchExePath}
          onToggle={handleToggleMod}
          onDelete={handleDeleteMod}
          onRefresh={() => {
            void runTask(async () => {
              await loadData();
              pushToast("info", "Mod 列表已刷新");
            }, "刷新失败");
          }}
          onInstall={handleInstallMod}
          onLaunch={handleLaunchGame}
          busy={busy}
        />
      </div>
      <ToastStack toasts={toasts} />
      {confirmState && (
        <ConfirmDialog
          state={confirmState}
          onCancel={() => setConfirmState(null)}
          onConfirm={() => {
            void handleConfirm();
          }}
        />
      )}
    </div>
  );
}

function ToastStack({ toasts }: { toasts: Toast[] }) {
  return (
    <div className="pointer-events-none fixed bottom-5 right-5 z-50 flex w-96 max-w-[calc(100vw-2rem)] flex-col gap-2">
      {toasts.map((toast) => (
        <div
          key={toast.id}
          className={`pointer-events-auto rounded-2xl border px-4 py-3 text-sm shadow-2xl backdrop-blur-xl ${
            toast.type === "success"
              ? "border-success/30 bg-success/15 text-success"
              : toast.type === "error"
              ? "border-danger/30 bg-danger/15 text-danger"
              : "border-accent/30 bg-accent-dim text-accent"
          }`}
        >
          {toast.message}
        </div>
      ))}
    </div>
  );
}

function ConfirmDialog({
  state,
  onCancel,
  onConfirm,
}: {
  state: ConfirmState;
  onCancel: () => void;
  onConfirm: () => void;
}) {
  return (
    <div className="fixed inset-0 z-40 flex items-center justify-center bg-black/55 px-4 backdrop-blur-sm">
      <div className="w-full max-w-md rounded-3xl border border-border bg-bg-secondary p-6 shadow-2xl">
        <div className="mb-3 text-lg font-semibold text-text-primary">
          {state.title}
        </div>
        <p className="mb-6 max-h-[50vh] overflow-auto whitespace-pre-wrap text-sm leading-6 text-text-secondary">
          {state.message}
        </p>
        <div className="flex justify-end gap-3">
          <button
            onClick={onCancel}
            className="rounded-xl border border-border px-4 py-2 text-sm font-medium text-text-secondary transition-colors hover:bg-bg-hover hover:text-text-primary"
          >
            取消
          </button>
          <button
            onClick={onConfirm}
            className={`rounded-xl px-4 py-2 text-sm font-semibold text-white transition-colors ${
              state.danger
                ? "bg-danger hover:bg-danger/85"
                : "bg-accent hover:bg-accent-hover"
            }`}
          >
            {state.confirmText}
          </button>
        </div>
      </div>
    </div>
  );
}

export default App;
