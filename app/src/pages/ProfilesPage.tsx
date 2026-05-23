import { useState } from "react";
import type { ModInfo, Profile, ProfileMod } from "../types/mod";
import { PageFrame } from "./LaunchPage";

interface ProfilesPageProps {
  profiles: Profile[];
  activeProfile: Profile | null;
  mods: ModInfo[];
  busy: boolean;
  onCreate: () => void;
  onActivate: (profile: Profile) => void;
  onDelete: (profile: Profile) => void;
  onUpdate: (profile: Profile) => void;
}

export function ProfilesPage({
  profiles,
  activeProfile,
  mods,
  busy,
  onCreate,
  onActivate,
  onDelete,
  onUpdate,
}: ProfilesPageProps) {
  const [draggedModId, setDraggedModId] = useState<string | null>(null);
  const activeProfileMods = activeProfile?.mods
    .slice()
    .sort((a, b) => a.loadOrder - b.loadOrder)
    .map((item) => ({
      item,
      mod: mods.find((mod) => mod.id === item.modId),
    }));

  const handleDrop = (targetModId: string) => {
    if (!activeProfile || !draggedModId || draggedModId === targetModId) {
      setDraggedModId(null);
      return;
    }

    onUpdate({
      ...activeProfile,
      mods: reorderProfileMods(activeProfile.mods, draggedModId, targetModId),
    });
    setDraggedModId(null);
  };

  return (
    <PageFrame
      eyebrow="Profiles"
      title="配置方案"
      description="方案保存 Mod 启用快照和加载顺序。拖拽右侧条目可以调整当前方案的加载顺序。"
    >
      <div className="grid min-h-0 flex-1 gap-4 xl:grid-cols-[22rem_1fr]">
        <section className="flex min-h-0 flex-col rounded-xl border border-border bg-panel">
          <div className="flex shrink-0 items-center justify-between border-b border-border p-4">
            <h2 className="text-base font-semibold text-text-primary">方案列表</h2>
            <button
              type="button"
              disabled={busy}
              onClick={onCreate}
              className="rounded-lg bg-accent px-3 py-2 text-sm font-semibold text-black transition-colors hover:bg-accent-hover disabled:opacity-50"
            >
              新建
            </button>
          </div>
          <div className="min-h-0 flex-1 overflow-y-auto p-3">
            {profiles.length === 0 ? (
              <div className="rounded-lg border border-dashed border-border p-4 text-sm leading-6 text-text-muted">
                还没有配置方案。
              </div>
            ) : (
              <div className="space-y-2">
                {profiles.map((profile) => {
                  const active = activeProfile?.id === profile.id;
                  return (
                    <button
                      key={profile.id}
                      type="button"
                      onClick={() => onActivate(profile)}
                      className={`w-full rounded-lg border p-4 text-left transition-colors ${
                        active
                          ? "border-accent/45 bg-accent-soft"
                          : "border-border bg-surface hover:border-accent/35"
                      }`}
                    >
                      <div className="flex items-start justify-between gap-3">
                        <div className="min-w-0">
                          <div className="truncate text-sm font-semibold text-text-primary">
                            {profile.icon} {profile.name}
                          </div>
                          <div className="mt-1 text-xs text-text-muted">
                            {profile.mods.filter((item) => item.enabled).length}/{profile.mods.length} 个 Mod
                          </div>
                        </div>
                        {active && (
                          <span className="rounded-md bg-accent px-2 py-1 text-xs font-semibold text-black">
                            当前
                          </span>
                        )}
                      </div>
                      {profile.description && (
                        <p className="mt-3 line-clamp-2 text-xs leading-5 text-text-secondary">
                          {profile.description}
                        </p>
                      )}
                    </button>
                  );
                })}
              </div>
            )}
          </div>
        </section>

        <section className="flex min-h-0 flex-col rounded-xl border border-border bg-panel">
          <div className="flex shrink-0 items-start justify-between gap-4 border-b border-border p-5">
            <div>
              <h2 className="text-lg font-semibold text-text-primary">
                {activeProfile ? `${activeProfile.icon} ${activeProfile.name}` : "未选择方案"}
              </h2>
              <p className="mt-2 text-sm text-text-muted">
                {activeProfile ? activeProfile.description || "无描述" : "从左侧选择或新建一个方案。"}
              </p>
            </div>
            {activeProfile && (
              <button
                type="button"
                disabled={busy}
                onClick={() => onDelete(activeProfile)}
                className="rounded-lg border border-danger/35 px-3 py-2 text-sm font-medium text-danger transition-colors hover:bg-danger/15 disabled:opacity-50"
              >
                删除方案
              </button>
            )}
          </div>

          <div className="min-h-0 flex-1 overflow-y-auto p-5">
            {!activeProfile || !activeProfileMods ? (
              <div className="grid h-full place-items-center rounded-xl border border-dashed border-border p-8 text-center text-sm text-text-muted">
                没有方案内容。
              </div>
            ) : activeProfileMods.length === 0 ? (
              <div className="rounded-xl border border-dashed border-border p-8 text-center text-sm text-text-muted">
                当前方案没有记录 Mod。
              </div>
            ) : (
              <div className="space-y-2">
                {activeProfileMods.map(({ item, mod }) => (
                  <div
                    key={item.modId}
                    draggable={!busy}
                    onDragStart={() => setDraggedModId(item.modId)}
                    onDragOver={(event) => event.preventDefault()}
                    onDrop={() => handleDrop(item.modId)}
                    onDragEnd={() => setDraggedModId(null)}
                    className={`grid cursor-grab grid-cols-[3rem_1fr_auto] items-center gap-3 rounded-lg border px-4 py-3 transition-colors active:cursor-grabbing ${
                      draggedModId === item.modId
                        ? "border-accent bg-accent-soft"
                        : "border-border bg-surface hover:border-accent/35"
                    }`}
                  >
                    <div className="text-sm font-semibold text-text-muted">#{item.loadOrder}</div>
                    <div className="min-w-0">
                      <div className="truncate text-sm font-semibold text-text-primary">
                        {mod?.name ?? item.modId}
                      </div>
                      <div className="mt-1 text-xs text-text-muted">
                        {mod?.type === "native" ? "DLL" : "资源包"}
                      </div>
                    </div>
                    <span
                      className={`rounded-md px-2 py-1 text-xs font-semibold ${
                        item.enabled ? "bg-success/15 text-success" : "bg-panel text-text-muted"
                      }`}
                    >
                      {item.enabled ? "启用" : "停用"}
                    </span>
                  </div>
                ))}
              </div>
            )}
          </div>
        </section>
      </div>
    </PageFrame>
  );
}

function reorderProfileMods(mods: ProfileMod[], draggedModId: string, targetModId: string): ProfileMod[] {
  const sorted = mods.slice().sort((a, b) => a.loadOrder - b.loadOrder);
  const fromIndex = sorted.findIndex((item) => item.modId === draggedModId);
  const toIndex = sorted.findIndex((item) => item.modId === targetModId);

  if (fromIndex < 0 || toIndex < 0) {
    return mods;
  }

  const [moved] = sorted.splice(fromIndex, 1);
  sorted.splice(toIndex, 0, moved);

  return sorted.map((item, index) => ({
    ...item,
    loadOrder: index + 1,
  }));
}
