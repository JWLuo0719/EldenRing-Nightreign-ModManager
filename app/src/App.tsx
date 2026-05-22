import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Titlebar } from "./components/Titlebar";
import { Sidebar } from "./components/Sidebar";
import { ModList } from "./components/ModList";
import { SetupGuide } from "./components/SetupGuide";
import type { ModInfo, Profile } from "./types/mod";

function App() {
  const [isSetup, setIsSetup] = useState(false);
  const [loading, setLoading] = useState(true);
  const [mods, setMods] = useState<ModInfo[]>([]);
  const [profiles, setProfiles] = useState<Profile[]>([]);
  const [activeProfile, setActiveProfile] = useState<Profile | null>(null);

  const checkSetup = useCallback(async () => {
    try {
      const gamePath = await invoke<string>("get_game_path");
      const me3Path = await invoke<string>("get_me3_path");
      setIsSetup(!!gamePath && !!me3Path);
      if (gamePath && me3Path) {
        await loadData();
      }
    } catch (e) {
      console.error("检查配置失败:", e);
    } finally {
      setLoading(false);
    }
  }, []);

  const loadData = useCallback(async () => {
    try {
      const [modsData, profilesData, activeData] = await Promise.all([
        invoke<ModInfo[]>("scan_mods"),
        invoke<Profile[]>("get_profiles"),
        invoke<Profile | null>("get_active_profile"),
      ]);
      setMods(modsData);
      setProfiles(profilesData);
      setActiveProfile(activeData);
    } catch (e) {
      console.error("加载数据失败:", e);
    }
  }, []);

  useEffect(() => {
    checkSetup();
  }, [checkSetup]);

  const handleSetupComplete = async (gamePath: string, me3Path: string) => {
    try {
      await invoke("set_game_path", { path: gamePath });
      await invoke("set_me3_path", { path: me3Path });
      setIsSetup(true);
      await loadData();
    } catch (e) {
      console.error("保存配置失败:", e);
    }
  };

  const handleToggleMod = async (mod: ModInfo) => {
    try {
      await invoke("toggle_mod", {
        modPath: mod.path,
        enabled: !mod.enabled,
      });
      await loadData();
    } catch (e) {
      console.error("切换mod状态失败:", e);
    }
  };

  const handleDeleteMod = async (mod: ModInfo) => {
    try {
      await invoke("uninstall_mod", { modPath: mod.path });
      await loadData();
    } catch (e) {
      console.error("删除mod失败:", e);
    }
  };

  const handleProfileSelect = (profile: Profile) => {
    setActiveProfile(profile);
  };

  const handleProfileCreate = async () => {
    try {
      const newProfile = await invoke<Profile>("create_profile", {
        name: `方案 ${profiles.length + 1}`,
        description: "",
        icon: "📦",
      });
      setProfiles([...profiles, newProfile]);
    } catch (e) {
      console.error("创建方案失败:", e);
    }
  };

  if (loading) {
    return (
      <div className="h-screen flex flex-col bg-bg-primary">
        <Titlebar />
        <div className="flex-1 flex items-center justify-center">
          <div className="text-text-muted text-sm">加载中...</div>
        </div>
      </div>
    );
  }

  if (!isSetup) {
    return (
      <div className="h-screen flex flex-col bg-bg-primary">
        <Titlebar />
        <SetupGuide onSetupComplete={handleSetupComplete} />
      </div>
    );
  }

  return (
    <div className="h-screen flex flex-col bg-bg-primary">
      <Titlebar />
      <div className="flex-1 flex overflow-hidden">
        <Sidebar
          profiles={profiles}
          activeProfile={activeProfile}
          onProfileSelect={handleProfileSelect}
          onProfileCreate={handleProfileCreate}
        />
        <ModList
          mods={mods}
          onToggle={handleToggleMod}
          onDelete={handleDeleteMod}
          onRefresh={loadData}
        />
      </div>
    </div>
  );
}

export default App;
