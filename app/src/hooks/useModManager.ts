import { useCallback, useEffect, useMemo, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { open, save } from "@tauri-apps/plugin-dialog";
import type {
  ConfirmState,
  FileConflict,
  LaunchArtifacts,
  LaunchPreflight,
  MultiplayerManifestComparison,
  MultiplayerManifestExport,
  ModInfo,
  ModInstallResult,
  Profile,
  ProfileMod,
  RuntimeEnvironment,
  RuntimeEnvironmentStatus,
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
  const [runtimeEnvironmentStatus, setRuntimeEnvironmentStatus] =
    useState<RuntimeEnvironmentStatus | null>(null);
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
    const [modsData, profilesData, activeData, specialStatus, runtimeStatus] = await Promise.all([
      invoke<ModInfo[]>("scan_mods"),
      invoke<Profile[]>("get_profiles"),
      invoke<Profile | null>("get_active_profile"),
      invoke<SpecialModStatus>("get_special_mod_status"),
      invoke<RuntimeEnvironmentStatus>("get_runtime_environment_status"),
    ]);
    setMods(modsData);
    setProfiles(profilesData);
    setActiveProfile(activeData);
    setSpecialModStatus(specialStatus);
    setRuntimeEnvironmentStatus(runtimeStatus);
    return { modsData, profilesData, activeData };
  }, []);

  const loadPaths = useCallback(async () => {
    const [nextGamePath, nextMe3Path, nextLaunchExePath, runtimeStatus] = await Promise.all([
      invoke<string>("get_game_path"),
      invoke<string>("get_me3_path"),
      invoke<string>("get_launch_exe_path"),
      invoke<RuntimeEnvironmentStatus>("get_runtime_environment_status"),
    ]);

    setGamePathState(nextGamePath);
    setMe3PathState(nextMe3Path);
    setLaunchExePathState(nextLaunchExePath);
    setRuntimeEnvironmentStatus(runtimeStatus);

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
    async (
      nextGamePath: string,
      nextMe3Path: string,
      nextLaunchExePath: string,
      runtimeEnvironment: RuntimeEnvironment
    ) => {
      await runTask(async () => {
        await invoke("set_game_path", { path: nextGamePath });
        await invoke("set_me3_path", { path: nextMe3Path });
        await invoke("set_launch_exe_path", { path: nextLaunchExePath });
        await invoke("set_runtime_environment", { environment: runtimeEnvironment });
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

  const setExternalProfileMode = useCallback(
    (mod: ModInfo) => {
      const useCommunityMode = mod.profileMode !== "mmv_seamless_community";
      setConfirmState({
        title: useCommunityMode ? "启用社区 Seamless 兼容模式" : "恢复作者联机方式",
        message: useCommunityMode
          ? `管理器只会在生成的 active-nightreign.me3 副本中移除 Server Redirector，并改用游戏目录现有的 SeamlessCoop\\nrsc.dll。\n\n原始 Mod、作者 .me3、regulation.bin 和 DLL 都不会被修改。此方式来自社区实践，不受 MMV 作者支持；启用前会强制检查运行环境、单一 regulation.bin 和中文层。`
          : "管理器将恢复使用作者 .me3 中的 Server Redirector。原始 Mod 文件仍不会被修改；该模式只适用于干净的 Steam 正版目录。",
        confirmText: useCommunityMode ? "启用兼容模式" : "恢复作者模式",
        danger: useCommunityMode,
        onConfirm: async () => {
          await runTask(async () => {
            await invoke("set_external_mod_profile_mode", {
              modId: mod.id,
              profileMode: useCommunityMode
                ? "mmv_seamless_community"
                : "author",
            });
            await loadWorkspace();
            pushToast(
              "success",
              useCommunityMode
                ? "已启用社区 Seamless 兼容模式"
                : "已恢复作者 Server Redirector 模式"
            );
          }, "切换外部 Mod Profile 模式失败");
        },
      });
    },
    [loadWorkspace, pushToast, runTask]
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
        `将把所选补丁 Game 文件夹中的 SeamlessCoop 和 OnlineFix 文件复制到当前游戏 Game 目录。\n\n源目录：${selected}\n\n管理器会先备份 8 个受管目标；失败时自动回滚，并可从启动台恢复最近备份。该操作只允许 Spacewar + Seamless 环境，Steam 正版目录会被后端阻止。`,
      confirmText: "备份后安装",
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

  const restoreOnlinePatchBackup = useCallback(() => {
    setConfirmState({
      title: "恢复联机补丁安装前状态",
      message:
        "将按最近一次备份清单恢复被覆盖的原文件，并移除那次安装新增的受管文件。只处理清单中的 8 个联机补丁目标。",
      confirmText: "恢复最近备份",
      danger: true,
      onConfirm: async () => {
        await runTask(async () => {
          const status = await invoke<SpecialModStatus>(
            "restore_latest_online_patch_backup"
          );
          setSpecialModStatus(status);
          await loadWorkspace();
          pushToast("success", "已恢复联机补丁安装前状态");
        }, "恢复联机补丁备份失败");
      },
    });
  }, [loadWorkspace, pushToast, runTask]);

  const installZip = useCallback(
    async (zipPath: string) => {
      await runTask(async () => {
        const result = await invoke<ModInstallResult>("install_mod_from_zip", { zipPath });
        await loadWorkspace();
        pushToast(
          "success",
          result.zhocnLayoutNormalized
            ? "Mod 已安装，汉化目录已规范为 msg\\zhocn"
            : "Mod 已安装并通过目录结构检查"
        );
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
            if (isExternalMod(mod)) {
              await invoke("toggle_external_mod", { modId: mod.id, enabled: desiredEnabled });
            } else {
              await invoke("toggle_mod", { modPath: mod.path, enabled: desiredEnabled });
            }
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

  const runLaunchPreflight = useCallback(async () => {
    return runTask(async () => {
      const result = await invoke<LaunchPreflight>("get_launch_preflight");
      pushToast(
        result.ready ? "success" : "error",
        result.ready ? "启动前检查通过" : "启动前检查发现阻止启动的问题"
      );
      return result;
    }, "启动前检查失败");
  }, [pushToast, runTask]);

  const diagnoseLaunch = useCallback(async () => {
    return runTask(async () => {
      const result = await invoke<string>("diagnose_launch_game", {
        gamePath: "",
        me3Path: "",
      });
      pushToast("success", "ME3/游戏已由诊断启动，请勿再次点击普通启动");
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

  const exportMultiplayerManifest = useCallback(async () => {
    const selected = await save({
      title: "导出双方联机一致性清单",
      defaultPath: "nightreign-multiplayer-manifest.json",
      filters: [{ name: "Nightreign 联机清单", extensions: ["json"] }],
    });
    if (typeof selected !== "string") {
      return;
    }

    return runTask(async () => {
      const result = await invoke<MultiplayerManifestExport>(
        "export_multiplayer_manifest",
        { path: selected }
      );
      pushToast("success", "联机一致性清单已导出");
      return result;
    }, "导出联机清单失败");
  }, [pushToast, runTask]);

  const compareMultiplayerManifest = useCallback(async () => {
    const selected = await open({
      multiple: false,
      title: "选择好友导出的联机清单",
      filters: [{ name: "Nightreign 联机清单", extensions: ["json"] }],
    });
    if (typeof selected !== "string") {
      return;
    }

    return runTask(async () => {
      const result = await invoke<MultiplayerManifestComparison>(
        "compare_multiplayer_manifest",
        { path: selected }
      );
      pushToast(
        result.compatible ? "success" : "error",
        result.compatible
          ? "双方联机关键文件、加载顺序与设置一致"
          : `发现 ${result.differences.filter((item) => item.severity === "error").length} 项阻断差异`
      );
      return result;
    }, "比较联机清单失败");
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
    runtimeEnvironmentStatus,
    toasts,
    confirmState,
    stats,
    setConfirmState,
    pushToast,
    savePaths,
    refresh,
    toggleMod,
    deleteMod,
    setExternalProfileMode,
    addExternalMod,
    addExternalDll,
    readModConfig,
    writeModConfig,
    installZip,
    installSeamlessOnlinefix,
    restoreOnlinePatchBackup,
    activateProfile,
    createProfile,
    deleteProfile,
    updateProfile,
    launchGame,
    runLaunchPreflight,
    diagnoseLaunch,
    generateProfilePreview,
    getLaunchArtifacts,
    detectFileConflicts,
    exportMultiplayerManifest,
    compareMultiplayerManifest,
  };
}
