import { useState } from "react";
import type { Profile } from "../types/mod";

interface SidebarProps {
  profiles: Profile[];
  activeProfile: Profile | null;
  onProfileSelect: (profile: Profile) => void;
  onProfileCreate: () => void;
  modsCount: number;
}

export function Sidebar({
  profiles,
  activeProfile,
  onProfileSelect,
  onProfileCreate,
  modsCount,
}: SidebarProps) {
  const [isExpanded, setIsExpanded] = useState(true);

  return (
    <aside
      className={`${
        isExpanded ? "w-72" : "w-16"
      } flex shrink-0 flex-col border-r border-border/80 bg-bg-secondary/85 transition-all duration-300 backdrop-blur-xl`}
    >
      <div className="border-b border-border/80 p-3">
        <div className="flex items-center justify-between">
          {isExpanded && (
            <div className="px-2">
              <div className="text-xs font-black uppercase tracking-[0.28em] text-accent">
                Profiles
              </div>
              <div className="mt-1 text-sm font-semibold text-text-primary">
                配置方案
              </div>
            </div>
          )}
          <button
            onClick={() => setIsExpanded(!isExpanded)}
            className="grid h-10 w-10 place-items-center rounded-xl text-text-muted transition-colors hover:bg-bg-hover hover:text-text-primary"
            aria-label={isExpanded ? "折叠侧栏" : "展开侧栏"}
          >
            <svg
              className={`h-4 w-4 transition-transform duration-300 ${
                isExpanded ? "" : "rotate-180"
              }`}
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              strokeWidth="2"
            >
              <path d="M15 18l-6-6 6-6" />
            </svg>
          </button>
        </div>

        {isExpanded && (
          <div className="mt-4 rounded-2xl border border-border bg-bg-primary/70 p-4">
            <div className="text-xs text-text-muted">当前方案</div>
            <div className="mt-1 truncate text-lg font-black text-text-primary">
              {activeProfile?.name ?? "未选择"}
            </div>
            <div className="mt-3 grid grid-cols-2 gap-2 text-xs">
              <div className="rounded-xl bg-bg-tertiary px-3 py-2">
                <div className="text-text-muted">方案数</div>
                <div className="mt-1 font-black text-text-primary">
                  {profiles.length}
                </div>
              </div>
              <div className="rounded-xl bg-bg-tertiary px-3 py-2">
                <div className="text-text-muted">Mod</div>
                <div className="mt-1 font-black text-text-primary">
                  {modsCount}
                </div>
              </div>
            </div>
          </div>
        )}
      </div>

      <div className="flex-1 overflow-y-auto py-3">
        {profiles.length === 0 && isExpanded ? (
          <div className="mx-3 rounded-2xl border border-dashed border-border p-4 text-sm leading-6 text-text-muted">
            尚未创建方案。新建方案会记录当前 Mod 启用快照。
          </div>
        ) : (
          profiles.map((profile) => {
            const active = activeProfile?.id === profile.id;
            return (
              <button
                key={profile.id}
                onClick={() => onProfileSelect(profile)}
                className={`mx-2 mb-2 flex w-[calc(100%-1rem)] items-center gap-3 rounded-2xl border px-3 py-3 text-left transition-all ${
                  active
                    ? "border-accent/45 bg-accent-dim text-accent shadow-lg shadow-accent/5"
                    : "border-transparent text-text-secondary hover:border-border hover:bg-bg-hover"
                }`}
              >
                <span className="grid h-9 w-9 shrink-0 place-items-center rounded-xl bg-bg-tertiary text-lg font-black">
                  {profile.icon || "◆"}
                </span>
                {isExpanded && (
                  <span className="min-w-0 flex-1">
                    <span className="block truncate text-sm font-bold">
                      {profile.name}
                    </span>
                    <span className="mt-0.5 block truncate text-xs text-text-muted">
                      {profile.mods.filter((item) => item.enabled).length}/
                      {profile.mods.length} 个 Mod
                    </span>
                  </span>
                )}
              </button>
            );
          })
        )}
      </div>

      <div className="border-t border-border/80 p-3">
        <button
          onClick={onProfileCreate}
          className={`flex w-full items-center justify-center gap-2 rounded-2xl bg-accent px-4 py-3 text-sm font-black text-bg-primary transition-all hover:-translate-y-0.5 hover:bg-accent-hover ${
            isExpanded ? "" : "px-0"
          }`}
          title="新建方案"
        >
          <svg
            className="h-4 w-4"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            strokeWidth="2"
          >
            <path d="M12 5v14M5 12h14" />
          </svg>
          {isExpanded && "新建方案"}
        </button>
      </div>
    </aside>
  );
}
