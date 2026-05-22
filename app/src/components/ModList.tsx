import { useState } from "react";
import type { ModInfo } from "../types/mod";
import { ModCard } from "./ModCard";

interface ModListProps {
  mods: ModInfo[];
  onToggle: (mod: ModInfo) => void;
  onDelete: (mod: ModInfo) => void;
  onRefresh: () => void;
}

export function ModList({ mods, onToggle, onDelete, onRefresh }: ModListProps) {
  const [searchQuery, setSearchQuery] = useState("");
  const [filterType, setFilterType] = useState<"all" | "package" | "native">(
    "all"
  );

  const filteredMods = mods.filter((mod) => {
    const matchesSearch =
      mod.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
      mod.description.toLowerCase().includes(searchQuery.toLowerCase());
    const matchesType = filterType === "all" || mod.type === filterType;
    return matchesSearch && matchesType;
  });

  const enabledCount = mods.filter((m) => m.enabled).length;

  return (
    <div className="flex-1 flex flex-col min-w-0 overflow-hidden">
      <div className="p-4 border-b border-border shrink-0">
        <div className="flex items-center justify-between mb-3">
          <div className="flex items-center gap-3">
            <h2 className="text-lg font-semibold text-text-primary">
              Mod 列表
            </h2>
            <span className="text-xs text-text-muted bg-bg-tertiary px-2 py-0.5 rounded">
              {enabledCount}/{mods.length} 已启用
            </span>
          </div>
          <button
            onClick={onRefresh}
            className="flex items-center gap-1.5 px-3 py-1.5 rounded bg-bg-tertiary hover:bg-bg-hover text-text-secondary text-sm transition-colors"
          >
            <svg
              className="w-4 h-4"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              strokeWidth="2"
            >
              <path d="M1 4v6h6M23 20v-6h-6" />
              <path d="M20.49 9A9 9 0 005.64 5.64L1 10m22 4l-4.64 4.36A9 9 0 013.51 15" />
            </svg>
            刷新
          </button>
        </div>

        <div className="flex gap-2">
          <div className="flex-1 relative">
            <svg
              className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-text-muted"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              strokeWidth="2"
            >
              <circle cx="11" cy="11" r="8" />
              <path d="M21 21l-4.35-4.35" />
            </svg>
            <input
              type="text"
              placeholder="搜索 Mod..."
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              className="w-full pl-10 pr-4 py-2 bg-bg-tertiary border border-border rounded text-sm text-text-primary placeholder-text-muted focus:outline-none focus:border-accent/50 transition-colors"
            />
          </div>

          <div className="flex bg-bg-tertiary rounded border border-border overflow-hidden">
            {(["all", "package", "native"] as const).map((type) => (
              <button
                key={type}
                onClick={() => setFilterType(type)}
                className={`px-3 py-2 text-xs transition-colors ${
                  filterType === type
                    ? "bg-accent text-white"
                    : "text-text-secondary hover:bg-bg-hover"
                }`}
              >
                {type === "all" ? "全部" : type === "package" ? "资源包" : "DLL"}
              </button>
            ))}
          </div>
        </div>
      </div>

      <div className="flex-1 overflow-y-auto p-4 space-y-2">
        {filteredMods.length === 0 ? (
          <div className="flex flex-col items-center justify-center h-full text-text-muted">
            <svg
              className="w-16 h-16 mb-4 opacity-30"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              strokeWidth="1"
            >
              <path d="M21 16V8a2 2 0 00-1-1.73l-7-4a2 2 0 00-2 0l-7 4A2 2 0 003 8v8a2 2 0 001 1.73l7 4a2 2 0 002 0l7-4A2 2 0 0021 16z" />
              <path d="M3.27 6.96L12 12.01l8.73-5.05M12 22.08V12" />
            </svg>
            <p className="text-sm">暂无 Mod</p>
            <p className="text-xs mt-1">将 Mod 文件夹放入游戏目录的 mods 文件夹中</p>
          </div>
        ) : (
          filteredMods.map((mod) => (
            <ModCard
              key={mod.id}
              mod={mod}
              onToggle={onToggle}
              onDelete={onDelete}
            />
          ))
        )}
      </div>
    </div>
  );
}
