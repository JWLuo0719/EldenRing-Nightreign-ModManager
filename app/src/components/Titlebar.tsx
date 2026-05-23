import { useMemo } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";

interface TitlebarProps {
  onSettingsClick?: () => void;
}

export function Titlebar({ onSettingsClick }: TitlebarProps) {
  const appWindow = useMemo(() => getCurrentWindow(), []);

  return (
    <div className="flex h-10 shrink-0 select-none items-center border-b border-border bg-panel">
      <div className="flex h-full flex-1 items-center gap-3 px-4" data-tauri-drag-region>
        <div className="grid h-6 w-6 place-items-center rounded-md bg-accent text-xs font-black text-black">
          N
        </div>
        <span className="text-sm font-semibold text-text-primary">Nightreign Mod Manager</span>
      </div>

      <div className="flex h-full">
        {onSettingsClick && (
          <button
            type="button"
            onClick={onSettingsClick}
            className="grid h-full w-11 place-items-center text-text-muted transition-colors hover:bg-surface hover:text-text-primary"
            title="设置"
          >
            <SettingsIcon />
          </button>
        )}
        <button
          type="button"
          onClick={() => void appWindow.minimize()}
          className="grid h-full w-11 place-items-center text-text-muted transition-colors hover:bg-surface hover:text-text-primary"
          title="最小化"
        >
          <span className="h-px w-3.5 bg-current" />
        </button>
        <button
          type="button"
          onClick={() => void appWindow.toggleMaximize()}
          className="grid h-full w-11 place-items-center text-text-muted transition-colors hover:bg-surface hover:text-text-primary"
          title="最大化"
        >
          <span className="h-3.5 w-3.5 border border-current" />
        </button>
        <button
          type="button"
          onClick={() => void appWindow.close()}
          className="grid h-full w-11 place-items-center text-text-muted transition-colors hover:bg-danger hover:text-white"
          title="关闭"
        >
          <CloseIcon />
        </button>
      </div>
    </div>
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

function CloseIcon() {
  return (
    <svg className="h-3.5 w-3.5" viewBox="0 0 12 12" fill="none" stroke="currentColor" strokeWidth="1.4">
      <path d="M2 2l8 8M10 2l-8 8" />
    </svg>
  );
}
