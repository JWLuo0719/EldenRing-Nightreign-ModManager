import { useDeferredValue, useState } from "react";
import type { ModInfo, Profile } from "../types/mod";
import { ModCard } from "./ModCard";

interface ModListProps {
  mods: ModInfo[];
  activeProfile: Profile | null;
  launchExePath: string;
  onToggle: (mod: ModInfo) => void;
  onDelete: (mod: ModInfo) => void;
  onRefresh: () => void;
  onInstall: () => void;
  onLaunch: () => void;
  busy: boolean;
}

interface PotentialConflict {
  key: string;
  mods: string[];
}

function getPotentialConflicts(mods: ModInfo[]): PotentialConflict[] {
  const owners = new Map<string, string[]>();

  for (const mod of mods.filter((item) => item.enabled)) {
    for (const file of mod.files) {
      const key = file.toLowerCase();
      const current = owners.get(key) ?? [];
      current.push(mod.name);
      owners.set(key, current);
    }
  }

  return Array.from(owners.entries())
    .filter(([, names]) => names.length > 1)
    .map(([key, names]) => ({ key, mods: names }))
    .slice(0, 4);
}

export function ModList({
  mods,
  activeProfile,
  launchExePath,
  onToggle,
  onDelete,
  onRefresh,
  onInstall,
  onLaunch,
  busy,
}: ModListProps) {
  const [searchQuery, setSearchQuery] = useState("");
  const [filterType, setFilterType] = useState<"all" | "package" | "native">(
    "all"
  );
  const deferredSearch = useDeferredValue(searchQuery);

  const filteredMods = mods.filter((mod) => {
    const query = deferredSearch.trim().toLowerCase();
    const matchesSearch =
      !query ||
      mod.name.toLowerCase().includes(query) ||
      mod.description.toLowerCase().includes(query);
    const matchesType = filterType === "all" || mod.type === filterType;
    return matchesSearch && matchesType;
  });

  const enabledCount = mods.filter((m) => m.enabled).length;
  const packageCount = mods.filter((m) => m.type === "package").length;
  const nativeCount = mods.filter((m) => m.type === "native").length;
  const trackedCount = activeProfile?.mods.length ?? 0;
  const conflicts = getPotentialConflicts(mods);
  const launchTarget = launchExePath.trim()
    ? launchExePath.split(/[\\/]/).pop() || "自定义启动器"
    : "nightreign.exe";

  return (
    <main className="flex-1 min-w-0 overflow-hidden p-5">
      <div className="flex h-full min-h-0 flex-col overflow-hidden rounded-[2rem] border border-border/80 bg-bg-primary/70 shadow-2xl shadow-black/25 backdrop-blur-xl">
        <section className="relative shrink-0 overflow-hidden border-b border-border/80 p-6">
          <div className="absolute -right-12 -top-24 h-56 w-56 rounded-full bg-accent/15 blur-3xl" />
          <div className="absolute right-20 top-8 h-16 w-40 rotate-12 rounded-full border border-accent/20" />

          <div className="relative flex flex-col gap-5">
            <div className="flex flex-wrap items-start justify-between gap-4">
              <div>
                <div className="mb-2 text-xs font-semibold uppercase tracking-[0.32em] text-accent">
                  Nightreign Loadout
                </div>
                <h1 className="text-3xl font-black tracking-tight text-text-primary">
                  Mod 工作台
                </h1>
                <p className="mt-2 max-w-2xl text-sm leading-6 text-text-secondary">
                  当前启动会生成 active-nightreign.me3，并按
                  {activeProfile ? `「${activeProfile.name}」` : "全局启用状态"}
                  写入 ME3 配置。启动目标：{launchTarget}。
                </p>
              </div>

              <div className="flex flex-wrap items-center gap-2">
                <button
                  onClick={onInstall}
                  disabled={busy}
                  className="inline-flex items-center gap-2 rounded-xl border border-border bg-bg-tertiary px-4 py-2.5 text-sm font-semibold text-text-secondary transition-all hover:-translate-y-0.5 hover:border-accent/40 hover:text-text-primary disabled:cursor-not-allowed disabled:opacity-50"
                >
                  <PlusIcon />
                  安装 ZIP
                </button>
                <button
                  onClick={onRefresh}
                  disabled={busy}
                  className="inline-flex items-center gap-2 rounded-xl border border-border bg-bg-tertiary px-4 py-2.5 text-sm font-semibold text-text-secondary transition-all hover:-translate-y-0.5 hover:border-accent/40 hover:text-text-primary disabled:cursor-not-allowed disabled:opacity-50"
                >
                  <RefreshIcon />
                  刷新
                </button>
                <button
                  onClick={onLaunch}
                  disabled={busy}
                  className="inline-flex items-center gap-2 rounded-xl bg-accent px-5 py-2.5 text-sm font-black text-bg-primary shadow-xl shadow-accent/20 transition-all hover:-translate-y-0.5 hover:bg-accent-hover disabled:cursor-not-allowed disabled:opacity-50"
                >
                  <PlayIcon />
                  启动游戏
                </button>
              </div>
            </div>

            <div className="grid gap-3 md:grid-cols-4">
              <StatCard label="已启用" value={`${enabledCount}/${mods.length}`} />
              <StatCard label="资源包" value={String(packageCount)} />
              <StatCard label="DLL" value={String(nativeCount)} />
              <StatCard label="方案追踪" value={String(trackedCount)} />
            </div>

            <div className="grid gap-3 lg:grid-cols-[1fr_18rem]">
              <div className="relative">
                <SearchIcon />
                <input
                  type="text"
                  placeholder="搜索 Mod 名称或说明..."
                  value={searchQuery}
                  onChange={(e) => setSearchQuery(e.target.value)}
                  className="w-full rounded-2xl border border-border bg-bg-tertiary/90 py-3 pl-11 pr-4 text-sm text-text-primary placeholder-text-muted outline-none transition-colors focus:border-accent/60"
                />
              </div>

              <div className="grid grid-cols-3 overflow-hidden rounded-2xl border border-border bg-bg-tertiary/80">
                {(["all", "package", "native"] as const).map((type) => (
                  <button
                    key={type}
                    onClick={() => setFilterType(type)}
                    className={`px-3 py-3 text-sm font-semibold transition-colors ${
                      filterType === type
                        ? "bg-accent text-bg-primary"
                        : "text-text-secondary hover:bg-bg-hover"
                    }`}
                  >
                    {type === "all" ? "全部" : type === "package" ? "资源" : "DLL"}
                  </button>
                ))}
              </div>
            </div>

            <div
              className={`rounded-2xl border px-4 py-3 text-xs leading-5 ${
                conflicts.length
                  ? "border-warning/35 bg-warning/10 text-warning"
                  : "border-accent/20 bg-accent-dim text-text-secondary"
              }`}
            >
              {conflicts.length ? (
                <span>
                  检测到潜在同名顶层文件/目录叠加：
                  {conflicts
                    .map((item) => `${item.key} (${item.mods.join(" / ")})`)
                    .join("；")}
                  。这不是精确冲突判定，但启动前值得检查。
                </span>
              ) : (
                <span>
                  直接从 Steam 或 nightreign.exe 启动不会使用当前列表；请通过这里的“启动游戏”进入。
                  如使用 Mod 启动器，请先在设置里指定启动程序。
                </span>
              )}
            </div>
          </div>
        </section>

        <section className="flex-1 overflow-y-auto p-5">
          {filteredMods.length === 0 ? (
            <div className="grid h-full place-items-center">
              <div className="max-w-md rounded-[2rem] border border-dashed border-border bg-bg-secondary/70 p-10 text-center">
                <div className="mx-auto mb-5 grid h-20 w-20 place-items-center rounded-3xl bg-bg-tertiary text-4xl text-text-muted">
                  ◇
                </div>
                <p className="mb-2 text-lg font-bold text-text-primary">
                  没有匹配的 Mod
                </p>
                <p className="text-sm leading-6 text-text-muted">
                  安装 ZIP，或把 Mod 文件夹放入游戏目录的 mods 文件夹后刷新。
                </p>
              </div>
            </div>
          ) : (
            <div className="grid gap-3 2xl:grid-cols-2">
              {filteredMods.map((mod) => (
                <ModCard
                  key={mod.id}
                  mod={mod}
                  tracked={
                    activeProfile?.mods.some((item) => item.modId === mod.id) ??
                    false
                  }
                  onToggle={onToggle}
                  onDelete={onDelete}
                />
              ))}
            </div>
          )}
        </section>
      </div>
    </main>
  );
}

function StatCard({ label, value }: { label: string; value: string }) {
  return (
    <div className="rounded-2xl border border-border/80 bg-bg-secondary/80 px-4 py-3">
      <div className="text-[11px] font-semibold uppercase tracking-[0.22em] text-text-muted">
        {label}
      </div>
      <div className="mt-1 text-2xl font-black text-text-primary">{value}</div>
    </div>
  );
}

function PlusIcon() {
  return (
    <svg className="h-4 w-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
      <path d="M12 5v14M5 12h14" />
    </svg>
  );
}

function RefreshIcon() {
  return (
    <svg className="h-4 w-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
      <path d="M1 4v6h6M23 20v-6h-6" />
      <path d="M20.49 9A9 9 0 005.64 5.64L1 10m22 4l-4.64 4.36A9 9 0 013.51 15" />
    </svg>
  );
}

function PlayIcon() {
  return (
    <svg className="h-4 w-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
      <path d="M5 3l14 9-14 9V3z" />
    </svg>
  );
}

function SearchIcon() {
  return (
    <svg
      className="pointer-events-none absolute left-4 top-1/2 h-4 w-4 -translate-y-1/2 text-text-muted"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="2"
    >
      <circle cx="11" cy="11" r="8" />
      <path d="M21 21l-4.35-4.35" />
    </svg>
  );
}
