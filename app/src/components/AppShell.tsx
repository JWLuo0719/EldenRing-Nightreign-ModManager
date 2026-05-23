import type { PageKey } from "../types/mod";
import type { ReactNode } from "react";

interface AppShellProps {
  currentPage: PageKey;
  onPageChange: (page: PageKey) => void;
  activeProfileName: string;
  enabledCount: number;
  totalMods: number;
  children: ReactNode;
}

const navItems: Array<{
  key: PageKey;
  label: string;
  description: string;
  icon: ReactNode;
}> = [
  {
    key: "launch",
    label: "启动台",
    description: "状态与一键启动",
    icon: <PlayIcon />,
  },
  {
    key: "mods",
    label: "Mod 仓库",
    description: "安装、筛选、启停",
    icon: <BoxIcon />,
  },
  {
    key: "profiles",
    label: "配置方案",
    description: "快照与加载顺序",
    icon: <LayersIcon />,
  },
  {
    key: "diagnostics",
    label: "诊断",
    description: "Profile、命令、日志",
    icon: <TerminalIcon />,
  },
  {
    key: "settings",
    label: "设置",
    description: "路径与启动程序",
    icon: <SettingsIcon />,
  },
];

export function AppShell({
  currentPage,
  onPageChange,
  activeProfileName,
  enabledCount,
  totalMods,
  children,
}: AppShellProps) {
  return (
    <div className="flex min-h-0 flex-1 bg-app text-text-primary">
      <aside className="flex w-64 shrink-0 flex-col border-r border-border bg-panel">
        <div className="px-5 py-5">
          <div className="text-[11px] font-semibold uppercase tracking-[0.22em] text-text-muted">
            Workspace
          </div>
          <div className="mt-3 rounded-lg border border-border bg-surface px-4 py-3">
            <div className="truncate text-sm font-semibold text-text-primary">
              {activeProfileName}
            </div>
            <div className="mt-2 flex items-center gap-2 text-xs text-text-muted">
              <span className="h-2 w-2 rounded-full bg-success" />
              <span>
                {enabledCount}/{totalMods} 个 Mod 启用
              </span>
            </div>
          </div>
        </div>

        <nav className="min-h-0 flex-1 space-y-1 overflow-y-auto px-3">
          {navItems.map((item) => {
            const active = currentPage === item.key;
            return (
              <button
                key={item.key}
                type="button"
                onClick={() => onPageChange(item.key)}
                className={`flex w-full items-center gap-3 rounded-lg px-3 py-3 text-left transition-colors ${
                  active
                    ? "bg-accent text-black"
                    : "text-text-secondary hover:bg-surface hover:text-text-primary"
                }`}
              >
                <span className="grid h-8 w-8 shrink-0 place-items-center rounded-md bg-black/10">
                  {item.icon}
                </span>
                <span className="min-w-0">
                  <span className="block text-sm font-semibold">{item.label}</span>
                  <span
                    className={`mt-0.5 block truncate text-xs ${
                      active ? "text-black/65" : "text-text-muted"
                    }`}
                  >
                    {item.description}
                  </span>
                </span>
              </button>
            );
          })}
        </nav>

        <div className="border-t border-border px-5 py-4 text-xs leading-5 text-text-muted">
          ME3 链路已按 SeamlessCoop/Spacewar 环境配置。启动失败时优先使用诊断页。
        </div>
      </aside>

      <main className="min-w-0 flex-1 overflow-hidden">{children}</main>
    </div>
  );
}

function PlayIcon() {
  return (
    <svg className="h-4 w-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
      <path d="M5 3l14 9-14 9V3z" />
    </svg>
  );
}

function BoxIcon() {
  return (
    <svg className="h-4 w-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
      <path d="M21 16V8a2 2 0 0 0-1-1.73L13 2.27a2 2 0 0 0-2 0L4 6.27A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16z" />
      <path d="M3.3 7 12 12l8.7-5M12 22V12" />
    </svg>
  );
}

function LayersIcon() {
  return (
    <svg className="h-4 w-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
      <path d="m12 2 9 5-9 5-9-5 9-5z" />
      <path d="m3 12 9 5 9-5M3 17l9 5 9-5" />
    </svg>
  );
}

function TerminalIcon() {
  return (
    <svg className="h-4 w-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
      <path d="m4 17 6-6-6-6M12 19h8" />
    </svg>
  );
}

function SettingsIcon() {
  return (
    <svg className="h-4 w-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
      <circle cx="12" cy="12" r="3" />
      <path d="M19.4 15a1.7 1.7 0 0 0 .34 1.87l.06.06a2 2 0 1 1-2.83 2.83l-.06-.06A1.7 1.7 0 0 0 15 19.4a1.7 1.7 0 0 0-1 1.55V21a2 2 0 1 1-4 0v-.09a1.7 1.7 0 0 0-1-1.51 1.7 1.7 0 0 0-1.87.34l-.06.06a2 2 0 1 1-2.83-2.83l.06-.06A1.7 1.7 0 0 0 4.6 15a1.7 1.7 0 0 0-1.55-1H3a2 2 0 1 1 0-4h.09A1.7 1.7 0 0 0 4.6 8.6a1.7 1.7 0 0 0-.34-1.87l-.06-.06a2 2 0 1 1 2.83-2.83l.06.06A1.7 1.7 0 0 0 9 4.6a1.7 1.7 0 0 0 1-1.55V3a2 2 0 1 1 4 0v.09a1.7 1.7 0 0 0 1 1.51 1.7 1.7 0 0 0 1.87-.34l.06-.06a2 2 0 1 1 2.83 2.83l-.06.06A1.7 1.7 0 0 0 19.4 9c.22.53.74.95 1.55.95H21a2 2 0 1 1 0 4h-.09A1.7 1.7 0 0 0 19.4 15z" />
    </svg>
  );
}
