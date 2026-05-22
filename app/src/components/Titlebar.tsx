import { useEffect } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";

export function Titlebar() {
  const appWindow = getCurrentWindow();

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
  }, []);

  return (
    <div className="h-9 bg-bg-secondary flex items-center select-none shrink-0 border-b border-border">
      <div
        className="flex-1 flex items-center px-3 h-full"
        data-tauri-drag-region
      >
        <div className="flex items-center gap-2">
          <div className="w-4 h-4 rounded-sm bg-accent flex items-center justify-center text-[10px] font-bold text-bg-primary">
            N
          </div>
          <span className="text-sm text-text-primary font-medium">
            夜临 Mod 管理器
          </span>
        </div>
      </div>
      <div className="flex h-full">
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
