import { useEffect, useMemo } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";

interface TitlebarProps {
  onSettingsClick?: () => void;
}

export function Titlebar({ onSettingsClick }: TitlebarProps) {
  const appWindow = useMemo(() => getCurrentWindow(), []);

  useEffect(() => {
    const minimize = () => appWindow.minimize();
    const maximize = () => appWindow.toggleMaximize();
    const close = () => appWindow.close();

    document
      .getElementById("titlebar-minimize")
      ?.addEventListener("click", minimize);
    document
      .getElementById("titlebar-maximize")
      ?.addEventListener("click", maximize);
    document
      .getElementById("titlebar-close")
      ?.addEventListener("click", close);

    return () => {
      document
        .getElementById("titlebar-minimize")
        ?.removeEventListener("click", minimize);
      document
        .getElementById("titlebar-maximize")
        ?.removeEventListener("click", maximize);
      document
        .getElementById("titlebar-close")
        ?.removeEventListener("click", close);
    };
  }, [appWindow]);

  return (
    <div className="h-10 bg-bg-secondary flex items-center select-none shrink-0 border-b border-border">
      <div
        className="flex-1 flex items-center px-4 h-full"
        data-tauri-drag-region
      >
        <div className="flex items-center gap-3">
          <div className="w-6 h-6 rounded-lg bg-accent flex items-center justify-center text-xs font-bold text-bg-primary shadow-sm">
            N
          </div>
          <span className="text-sm text-text-primary font-semibold tracking-wide">
            夜临 Mod 管理器
          </span>
        </div>
      </div>
      <div className="flex h-full">
        {onSettingsClick && (
          <button
            onClick={onSettingsClick}
            className="w-11 h-full flex items-center justify-center hover:bg-bg-hover transition-colors text-text-secondary hover:text-text-primary"
            title="设置"
          >
            <svg
              className="w-4 h-4"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              strokeWidth="2"
            >
              <circle cx="12" cy="12" r="3" />
              <path d="M19.4 15a1.65 1.65 0 00.33 1.82l.06.06a2 2 0 010 2.83 2 2 0 01-2.83 0l-.06-.06a1.65 1.65 0 00-1.82-.33 1.65 1.65 0 00-1 1.51V21a2 2 0 01-2 2 2 2 0 01-2-2v-.09A1.65 1.65 0 009 19.4a1.65 1.65 0 00-1.82.33l-.06.06a2 2 0 01-2.83 0 2 2 0 010-2.83l.06-.06A1.65 1.65 0 004.68 15a1.65 1.65 0 00-1.51-1H3a2 2 0 01-2-2 2 2 0 012-2h.09A1.65 1.65 0 004.6 9a1.65 1.65 0 00-.33-1.82l-.06-.06a2 2 0 010-2.83 2 2 0 012.83 0l.06.06A1.65 1.65 0 009 4.68a1.65 1.65 0 001-1.51V3a2 2 0 012-2 2 2 0 012 2v.09a1.65 1.65 0 001 1.51 1.65 1.65 0 001.82-.33l.06-.06a2 2 0 012.83 0 2 2 0 010 2.83l-.06.06A1.65 1.65 0 0019.32 9a1.65 1.65 0 001.51 1H21a2 2 0 012 2 2 2 0 01-2 2h-.09a1.65 1.65 0 00-1.51 1z" />
            </svg>
          </button>
        )}
        <button
          id="titlebar-minimize"
          className="w-11 h-full flex items-center justify-center hover:bg-bg-hover transition-colors"
        >
          <svg
            className="w-3.5 h-3.5 text-text-secondary"
            viewBox="0 0 12 12"
          >
            <rect
              x="1"
              y="5.5"
              width="10"
              height="1"
              fill="currentColor"
            />
          </svg>
        </button>
        <button
          id="titlebar-maximize"
          className="w-11 h-full flex items-center justify-center hover:bg-bg-hover transition-colors"
        >
          <svg
            className="w-3.5 h-3.5 text-text-secondary"
            viewBox="0 0 12 12"
          >
            <rect
              x="1.5"
              y="1.5"
              width="9"
              height="9"
              fill="none"
              stroke="currentColor"
              strokeWidth="1"
            />
          </svg>
        </button>
        <button
          id="titlebar-close"
          className="w-11 h-full flex items-center justify-center hover:bg-danger transition-colors group"
        >
          <svg
            className="w-3.5 h-3.5 text-text-secondary group-hover:text-white"
            viewBox="0 0 12 12"
          >
            <path
              d="M2 2L10 10M10 2L2 10"
              stroke="currentColor"
              strokeWidth="1.2"
            />
          </svg>
        </button>
      </div>
    </div>
  );
}
