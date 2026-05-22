import { useState } from "react";
import type { Profile } from "../types/mod";

interface SidebarProps {
  profiles: Profile[];
  activeProfile: Profile | null;
  onProfileSelect: (profile: Profile) => void;
  onProfileCreate: () => void;
}

export function Sidebar({
  profiles,
  activeProfile,
  onProfileSelect,
  onProfileCreate,
}: SidebarProps) {
  const [isExpanded, setIsExpanded] = useState(true);

  return (
    <div
      className={`${
        isExpanded ? "w-52" : "w-12"
      } bg-bg-secondary border-r border-border flex flex-col transition-all duration-200 shrink-0`}
    >
      <div className="p-2 border-b border-border flex items-center justify-between">
        {isExpanded && (
          <span className="text-xs text-text-muted font-medium px-2">
            配置方案
          </span>
        )}
        <button
          onClick={() => setIsExpanded(!isExpanded)}
          className="w-8 h-8 flex items-center justify-center rounded hover:bg-bg-hover transition-colors text-text-muted"
        >
          <svg
            className={`w-4 h-4 transition-transform ${
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

      <div className="flex-1 overflow-y-auto py-1">
        {profiles.map((profile) => (
          <button
            key={profile.id}
            onClick={() => onProfileSelect(profile)}
            className={`w-full flex items-center gap-2 px-3 py-2 transition-colors ${
              activeProfile?.id === profile.id
                ? "bg-accent-dim text-accent"
                : "text-text-secondary hover:bg-bg-hover"
            }`}
          >
            <span className="text-lg shrink-0">{profile.icon || "📦"}</span>
            {isExpanded && (
              <div className="text-left min-w-0">
                <div className="text-sm truncate">{profile.name}</div>
                {profile.description && (
                  <div className="text-xs text-text-muted truncate">
                    {profile.description}
                  </div>
                )}
              </div>
            )}
          </button>
        ))}
      </div>

      {isExpanded && (
        <div className="p-2 border-t border-border">
          <button
            onClick={onProfileCreate}
            className="w-full flex items-center justify-center gap-1.5 px-3 py-2 rounded bg-bg-tertiary hover:bg-bg-hover text-text-secondary text-sm transition-colors"
          >
            <svg
              className="w-4 h-4"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              strokeWidth="2"
            >
              <path d="M12 5v14M5 12h14" />
            </svg>
            新建方案
          </button>
        </div>
      )}
    </div>
  );
}
