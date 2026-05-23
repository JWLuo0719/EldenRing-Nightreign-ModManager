import { useCallback, useEffect, useMemo, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import type {
  ConfirmState,
  FileConflict,
  LaunchArtifacts,
  ModInfo,
  Profile,
  ProfileMod,
  SpecialModStatus,
  Toast,
  ToastType,
} from "../types/mod";

function snapshotMods(mods: ModInfo[]): ProfileMod[] {
  return mods.map((mod, index) => ({
    modId: mod.id,
    enabled: mod.enabled,
    loadOrder: index + 1,
  }));
}

function getErrorMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}

function isExternalMod(mod: ModInfo) {
  return mod.source === "external_package" || mod.source === "external_native";
}

export function useModManager() {
  const [loading, setLoading] = useState(true);
  const [busy, setBusy] = useState(false);
  const [configured, setConfigured] = useState(false);
  const [mods, setMods] = useState<ModInfo[]>([]);
  const [profiles, setProfiles] = useState<Profile[]>([]);
  const [activeProfile, setActiveProfile] = useState<Profile | null>(null);
  const [gamePath, setGamePathState] = useState("");
  const [me3Path, setMe3PathState] = useState("");
  const [launchExePath, setLaunchExePathState] = useState("");
  const [specialModStatus, setSpecialModStatus] = useState<SpecialModStatus | null>(null);
  const [toasts, setToasts] = useState<Toast[]>([]);
  const [confirmState, setConfirmState] = useState<ConfirmState | null>(null);

  const pushToast = useCallback((type: ToastType, message: string) => {
    const id = Date.now() + Math.random();
    setToasts((items) => [...items, { id, type, message }]);
    window.setTimeout(() => {
      setToasts((items) => items.filter((item) => item.id !== id));
    }, 3600);
  }, []);

  const loadWorkspace = useCallback(async () => {
    const [modsData, profilesData, activeData, specialStatus] = await Promise.all([
      invoke<ModInfo[]>("scan_mods"),
      invoke<Profile[]>("get_profiles"),
      invoke<Profile | null>("get_active_profile"),
      invoke<SpecialModStatus>("get_special_mod_status"),
    ]);
    setMods(modsData);
    setProfiles(profilesData);
    setActiveProfile(activeData);
    setSpecialModStatus(specialStatus);
    return { modsData, profilesData, activeData };
  }, []);

  const loadPaths = useCallback(async () => {
    const [nextGamePath, nextMe3Path, nextLaunchExePath] = await Promise.all([
      invoke<string>("get_game_path"),
      invoke<string>("get_me3_path"),
      invoke<string>("get_launch_exe_path"),
    ]);

    setGamePathState(nextGamePath);
    setMe3PathState(nextMe3Path);
    setLaunchExePathState(nextLaunchExePath);

    const isConfigured = Boolean(nextGamePath && nextMe3Path);
    setConfigured(isConfigured);

    if (isConfigured) {
      await loadWorkspace();
    }
  }, [loadWorkspace]);

  const runTask = useCallback(
    async <T,>(task: () => Promise<T>, fallbackMessage: string) => {
      try {
        setBusy(true);
        return await task();
      } catch (error) {
        const message = `${fallbackMessage}：${getErrorMessage(error)}`;
        console.error(message, error);
        pushToast("error", message);
        return undefined;
      } finally {
        setBusy(false);
      }
    },
    [pushToast]
  );

  useEffect(() => {
    void (async () => {
      try {
        await loadPaths();
      } catch (error) {
        pushToast("error", `读取配置失败：${getErrorMessage(error)}`);
      } finally {
        setLoading(false);
      }
    })();
  }, [loadPaths, pushToast]);

  const savePaths = useCallback(
    async (nextGamePath: string, nextMe3Path: string, nextLaunchExePath: string) => {
      await runTask(async () => {
        await invoke("set_game_path", { path: nextGamePath });
        await invoke("set_me3_path", { path: nextMe3Path });
        await invoke("set_launch_exe_path", { path: nextLaunchExePath });
        setGamePathState(nextGamePath);
        setMe3PathState(nextMe3Path);
        setLaunchExePathState(nextLaunchExePath);
        setConfigured(Boolean(nextGamePath && nextMe3Path));
        await loadWorkspace();
        pushToast("success", "路径配置已保存");
      }, "保存配置失败");
    },
    [loadWorkspace, pushToast, runTask]
  );

  const refresh = useCallback(async () => {
    await runTask(async () => {
      await loadWorkspace();
      pushToast("info", "工作区已刷新");
    }, "刷新失败");
  }, [loadWorkspace, pushToast, runTask]);

  const toggleMod = useCallback(
    async (mod: ModInfo) => {
      await runTask(async () => {
        const nextEnabled = !mod.enabled;
        if (isExternalMod(mod)) {
          await invoke("toggle_external_mod", { modId: mod.id, enabled: nextEnabled });
        } else {
          await invoke("toggle_mod", { modPath: mod.path, enabled: nextEnabled });
        }
        if (activeProfile) {
          await invoke<Profile | null>("update_active_profile_mod", {
            modId: mod.id,
            enabled: nextEnabled,
          });
        }
        await loadWorkspace();
        pushToast("success", `${mod.name} 已${nextEnabled ? "启用" : "停用"}`);
      }, "切换 Mod 状态失败");
    },
    [activeProfile, loadWorkspace, pushToast, runTask]
  );

  const deleteMod = useCallback(
    (mod: ModInfo) => {
      if (isExternalMod(mod)) {
        setConfirmState({
          title: "移除外部 Mod",
          message: `确定从管理器移除“${mod.name}”吗？\n\n这只会删除外部注册记录，不会移动或删除原始文件。`,
          confirmText: "移除注册",
          danger: false,
          onConfirm: async () => {
            await runTask(async () => {
              await invoke("remove_external_mod", { modId: mod.id });
              if (activeProfile) {
                await invoke("update_profile", {
                  profile: {
                    ...activeProfile,
                    mods: activeProfile.mods.filter((item) => item.modId !== mod.id),
                  },
                });
              }
              await loadWorkspace();
              pushToast("success", `${mod.name} 已从外部注册中移除`);
            }, "移除外部 Mod 失败");
          },
        });
        return;
      }

      setConfirmState({
        title: "删除 Mod",
        message: `确定将“${mod.name}”移动到系统回收站吗？\n\n这会移走整个 Mod 文件夹；如需恢复，请从 Windows 回收站还原。`,
        confirmText: "移到回收站",
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
            await loadWorkspace();
            pushToast("success", `${mod.name} 已移动到回收站`);
          }, "移动 Mod 到回收站失败");
        },
      });
    },
    [activeProfile, loadWorkspace, pushToast, runTask]
  );

  const addExternalMod = useCallback(async () => {
    const selected = await open({
      directory: true,
      title: "选择外部 Mod 文件夹",
    });

    if (typeof selected !== "string") {
      return;
    }

    await runTask(async () => {
      await invoke("add_external_mod", { path: selected });
      await loadWorkspace();
      pushToast("success", "外部 Mod 已添加到管理器");
    }, "添加外部 Mod 失败");
  }, [loadWorkspace, pushToast, runTask]);

  const addExternalDll = useCallback(async () => {
    const selected = await open({
      multiple: false,
      title: "选择外部 DLL",
      filters: [{ name: "DLL", extensions: ["dll"] }],
    });

    if (typeof selected !== "string") {
      return;
    }

    await runTask(async () => {
      await invoke("add_external_dll", { path: selected });
      await loadWorkspace();
      pushToast("success", "外部 DLL 已添加到管理器");
    }, "添加外部 DLL 失败");
  }, [loadWorkspace, pushToast, runTask]);

  const readModConfig = useCallback(
    async (path: string) => {
      return runTask(async () => {
        return await invoke<string>("read_mod_config_file", { path });
      }, "读取 Mod 配置失败");
    },
    [runTask]
  );

  const writeModConfig = useCallback(
    async (path: string, content: string) => {
      await runTask(async () => {
        await invoke("write_mod_config_file", { path, content });
        await loadWorkspace();
        pushToast("success", "Mod 配置已保存");
      }, "保存 Mod 配置失败");
    },
    [loadWorkspace, pushToast, runTask]
  );

  const installSeamlessOnlinefix = useCallback(async () => {
    const selected = await open({
      directory: true,
      title: "选择联机补丁中的 Game 文件夹",
    });

    if (typeof selected !== "string") {
      return;
    }

    setConfirmState({
      title: "准备联机补丁",
      message:
        `将把所选补丁 Game 文件夹中的 SeamlessCoop 和 OnlineFix 文件复制到当前游戏 Game 目录。\n\n源目录：${selected}\n\n会覆盖同名文件，包括 steam_api64.dll。请确认你要进入 SeamlessCoop/Spacewar 环境。`,
      confirmText: "复制并覆盖",
      danger: true,
      onConfirm: async () => {
        await runTask(async () => {
          const status = await invoke<SpecialModStatus>("install_seamless_onlinefix", {
            patchGamePath: selected,
          });
          setSpecialModStatus(status);
          await loadWorkspace();
          pushToast("success", "联机补丁已准备完成");
        }, "准备联机补丁失败");
      },
    });
  }, [loadWorkspace, pushToast, runTask]);

  const installZip = useCallback(
    async (zipPath: string) => {
      await runTask(async () => {
        await invoke("install_mod_from_zip", { zipPath });
        await loadWorkspace();
        pushToast("success", "Mod 已安装，请检查结构后再启动游戏");
      }, "安装 Mod 失败");
    },
    [loadWorkspace, pushToast, runTask]
  );

  const activateProfile = useCallback(
    async (profile: Profile) => {
      await runTask(async () => {
        await invoke("activate_profile", { profileId: profile.id });
        const desiredState = new Map(profile.mods.map((item) => [item.modId, item.enabled]));

        for (const mod of mods) {
          const desiredEnabled = desiredState.get(mod.id);
          if (desiredEnabled !== undefined && desiredEnabled !== mod.enabled) {
            await invoke("toggle_mod", { modPath: mod.path, enabled: desiredEnabled });
          }
        }

        await loadWorkspace();
        pushToast("info", `已切换到“${profile.name}”`);
      }, "切换方案失败");
    },
    [loadWorkspace, mods, pushToast, runTask]
  );

  const createProfile = useCallback(async () => {
    await runTask(async () => {
      const newProfile = await invoke<Profile>("create_profile", {
        name: `方案 ${profiles.length + 1}`,
        description: "从当前 Mod 启用状态创建",
        icon: "◆",
      });
      const seededProfile: Profile = {
        ...newProfile,
        mods: snapshotMods(mods),
        isActive: true,
      };
      await invoke("update_profile", { profile: seededProfile });
      await invoke("activate_profile", { profileId: seededProfile.id });
      await loadWorkspace();
      pushToast("success", `已创建“${seededProfile.name}”`);
    }, "创建方案失败");
  }, [loadWorkspace, mods, profiles.length, pushToast, runTask]);

  const deleteProfile = useCallback(
    (profile: Profile) => {
      setConfirmState({
        title: "删除配置方案",
        message: `确定删除“${profile.name}”吗？这不会删除 Mod 文件。`,
        confirmText: "删除",
        danger: true,
        onConfirm: async () => {
          await runTask(async () => {
            await invoke("delete_profile", { profileId: profile.id });
            await loadWorkspace();
            pushToast("success", `已删除“${profile.name}”`);
          }, "删除方案失败");
        },
      });
    },
    [loadWorkspace, pushToast, runTask]
  );

  const updateProfile = useCallback(
    async (profile: Profile) => {
      await runTask(async () => {
        await invoke("update_profile", { profile });
        await loadWorkspace();
        pushToast("success", "方案已保存");
      }, "保存方案失败");
    },
    [loadWorkspace, pushToast, runTask]
  );

  const launchGame = useCallback(async () => {
    await runTask(async () => {
      const result = await invoke<string>("launch_game", { gamePath: "", me3Path: "" });
      pushToast("success", result.split("\n")[0] || "已通过 ME3 启动游戏");
      return result;
    }, "启动游戏失败");
  }, [pushToast, runTask]);

  const diagnoseLaunch = useCallback(async () => {
    return runTask(async () => {
      const result = await invoke<string>("diagnose_launch_game", {
        gamePath: "",
        me3Path: "",
      });
      pushToast("info", "诊断命令已执行");
      return result;
    }, "诊断启动失败");
  }, [pushToast, runTask]);

  const generateProfilePreview = useCallback(async () => {
    return runTask(async () => {
      await invoke<string>("generate_me3_profile");
      const artifacts = await invoke<LaunchArtifacts>("get_launch_artifacts");
      pushToast("info", "已生成 active-nightreign.me3 预览");
      return {
        profilePath: artifacts.profilePath,
        content: artifacts.profileContent,
      };
    }, "生成 ME3 profile 失败");
  }, [pushToast, runTask]);

  const getLaunchArtifacts = useCallback(async () => {
    return runTask(async () => {
      const artifacts = await invoke<LaunchArtifacts>("get_launch_artifacts");
      pushToast("info", "启动文件已读取");
      return artifacts;
    }, "读取启动文件失败");
  }, [pushToast, runTask]);

  const detectFileConflicts = useCallback(async () => {
    return runTask(async () => {
      const conflicts = await invoke<FileConflict[]>("detect_file_conflicts");
      pushToast("info", conflicts.length ? `发现 ${conflicts.length} 个文件级冲突` : "未发现文件级冲突");
      return conflicts;
    }, "分析文件冲突失败");
  }, [pushToast, runTask]);

  const stats = useMemo(() => {
    const enabled = mods.filter((mod) => mod.enabled).length;
    const packages = mods.filter((mod) => mod.type === "package").length;
    const natives = mods.filter((mod) => mod.type === "native").length;
    return { enabled, packages, natives, total: mods.length };
  }, [mods]);

  return {
    loading,
    busy,
    configured,
    mods,
    profiles,
    activeProfile,
    gamePath,
    me3Path,
    launchExePath,
    specialModStatus,
    toasts,
    confirmState,
    stats,
    setConfirmState,
    pushToast,
    savePaths,
    refresh,
    toggleMod,
    deleteMod,
    addExternalMod,
    addExternalDll,
    readModConfig,
    writeModConfig,
    installZip,
    installSeamlessOnlinefix,
    activateProfile,
    createProfile,
    deleteProfile,
    updateProfile,
    launchGame,
    diagnoseLaunch,
    generateProfilePreview,
    getLaunchArtifacts,
    detectFileConflicts,
  };
}
