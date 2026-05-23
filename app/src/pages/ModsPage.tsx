import { useDeferredValue, useMemo, useState } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import { ModCard } from "../components/ModCard";
import type { ModInfo, Profile } from "../types/mod";
import { PageFrame } from "./LaunchPage";

interface ModsPageProps {
  mods: ModInfo[];
  activeProfile: Profile | null;
  busy: boolean;
  onToggle: (mod: ModInfo) => void;
  onDelete: (mod: ModInfo) => void;
  onRefresh: () => void;
  onInstallZip: (zipPath: string) => Promise<void>;
  onAddExternalMod: () => Promise<void>;
  onAddExternalDll: () => Promise<void>;
  onReadConfig: (path: string) => Promise<string | undefined>;
  onWriteConfig: (path: string, content: string) => Promise<void>;
}

type FilterType = "all" | "package" | "native" | "enabled" | "disabled";

export function ModsPage({
  mods,
  activeProfile,
  busy,
  onToggle,
  onDelete,
  onRefresh,
  onInstallZip,
  onAddExternalMod,
  onAddExternalDll,
  onReadConfig,
  onWriteConfig,
}: ModsPageProps) {
  const [searchQuery, setSearchQuery] = useState("");
  const [filterType, setFilterType] = useState<FilterType>("all");
  const [editor, setEditor] = useState<ConfigEditorState | null>(null);
  const deferredSearch = useDeferredValue(searchQuery);

  const filteredMods = useMemo(() => {
    const query = deferredSearch.trim().toLowerCase();
    return mods.filter((mod) => {
      const matchesSearch =
        !query ||
        mod.name.toLowerCase().includes(query) ||
        mod.description.toLowerCase().includes(query) ||
        mod.id.toLowerCase().includes(query);
      const matchesType =
        filterType === "all" ||
        (filterType === "enabled" && mod.enabled) ||
        (filterType === "disabled" && !mod.enabled) ||
        mod.type === filterType;
      return matchesSearch && matchesType;
    });
  }, [deferredSearch, filterType, mods]);

  const conflicts = useMemo(() => getPotentialConflicts(mods), [mods]);

  const installZip = async () => {
    const selected = await open({
      multiple: false,
      title: "选择 Mod ZIP 压缩包",
      filters: [{ name: "ZIP 压缩包", extensions: ["zip"] }],
    });

    if (typeof selected === "string") {
      await onInstallZip(selected);
    }
  };

  const openConfigEditor = async (mod: ModInfo) => {
    const path = mod.configFiles[0];
    if (!path) {
      return;
    }
    const content = await onReadConfig(path);
    if (content === undefined) {
      return;
    }
    setEditor({ modName: mod.name, path, content, error: "" });
  };

  const saveConfigEditor = async () => {
    if (!editor) {
      return;
    }

    if (editor.path.toLowerCase().endsWith(".json")) {
      try {
        JSON.parse(editor.content);
      } catch (error) {
        setEditor({
          ...editor,
          error: error instanceof Error ? error.message : String(error),
        });
        return;
      }
    }

    await onWriteConfig(editor.path, editor.content);
    setEditor(null);
  };

  return (
    <PageFrame
      eyebrow="Mod Library"
      title="Mod 仓库"
      description="扫描 Game\\mods，注册外部 Mod/DLL，并编辑可识别的 JSON/INI 配置。"
    >
      <div className="flex min-h-0 flex-1 flex-col gap-3">
        <section className="shrink-0 rounded-xl border border-border bg-panel p-4">
          <div className="grid gap-3 xl:grid-cols-[1fr_auto]">
            <div className="flex min-w-0 flex-wrap items-center gap-2">
              <div className="relative min-w-64 flex-1">
                <SearchIcon />
                <input
                  value={searchQuery}
                  onChange={(event) => setSearchQuery(event.target.value)}
                  placeholder="搜索名称、说明或 ID"
                  className="w-full rounded-lg border border-border bg-surface py-2 pl-10 pr-3 text-sm text-text-primary outline-none transition-colors placeholder:text-text-muted focus:border-accent/65"
                />
              </div>
              <div className="flex flex-wrap gap-1.5">
                {filterItems.map((item) => (
                  <button
                    key={item.key}
                    type="button"
                    onClick={() => setFilterType(item.key)}
                    className={`rounded-md px-2.5 py-1.5 text-xs font-semibold transition-colors ${
                      filterType === item.key
                        ? "bg-accent text-black"
                        : "bg-surface text-text-secondary hover:text-text-primary"
                    }`}
                  >
                    {item.label}
                  </button>
                ))}
              </div>
            </div>
            <div className="flex shrink-0 flex-wrap justify-end gap-2">
              <button
                type="button"
                disabled={busy}
                onClick={() => void installZip()}
                className="rounded-lg bg-accent px-4 py-2 text-sm font-semibold text-black transition-colors hover:bg-accent-hover disabled:opacity-50"
              >
                安装 ZIP
              </button>
              <button
                type="button"
                disabled={busy}
                onClick={() => void onAddExternalMod()}
                className="rounded-lg border border-border bg-surface px-4 py-2 text-sm font-medium text-text-secondary transition-colors hover:border-accent/45 hover:text-text-primary disabled:opacity-50"
              >
                添加外部 Mod
              </button>
              <button
                type="button"
                disabled={busy}
                onClick={() => void onAddExternalDll()}
                className="rounded-lg border border-border bg-surface px-4 py-2 text-sm font-medium text-text-secondary transition-colors hover:border-accent/45 hover:text-text-primary disabled:opacity-50"
              >
                添加外部 DLL
              </button>
              <button
                type="button"
                disabled={busy}
                onClick={onRefresh}
                className="rounded-lg border border-border px-4 py-2 text-sm font-medium text-text-secondary transition-colors hover:bg-surface hover:text-text-primary disabled:opacity-50"
              >
                刷新
              </button>
            </div>
          </div>

          <div className="mt-3 grid gap-2 md:grid-cols-4">
            <CompactStat label="全部" value={String(mods.length)} />
            <CompactStat label="启用" value={String(mods.filter((mod) => mod.enabled).length)} />
            <CompactStat label="资源包" value={String(mods.filter((mod) => mod.type === "package").length)} />
            <CompactStat label="DLL" value={String(mods.filter((mod) => mod.type === "native").length)} />
          </div>

          {conflicts.length > 0 && (
            <div className="mt-2 rounded-lg border border-warning/25 bg-warning/10 px-3 py-2 text-xs leading-5 text-warning">
              顶层同名项提示：
              {conflicts.map((conflict) => conflict.key).join(" / ")}
            </div>
          )}
        </section>

        <section className="flex min-h-0 flex-1 flex-col rounded-xl border border-border bg-panel">
          <div className="flex shrink-0 items-center justify-between border-b border-border px-4 py-2.5">
            <div className="text-sm font-semibold text-text-primary">Mod 配置卡片</div>
            <div className="text-xs text-text-muted">显示 {filteredMods.length} / {mods.length}</div>
          </div>
          <div className="min-h-0 flex-1 overflow-y-auto p-3">
            {filteredMods.length === 0 ? (
              <div className="grid h-full place-items-center rounded-xl border border-dashed border-border p-8 text-center">
                <div>
                  <div className="text-lg font-semibold text-text-primary">没有匹配的 Mod</div>
                  <p className="mt-2 text-sm text-text-muted">调整搜索条件，或安装 ZIP 后刷新。</p>
                </div>
              </div>
            ) : (
              <div className="grid gap-3 2xl:grid-cols-2">
                {filteredMods.map((mod) => (
                  <ModCard
                    key={mod.id}
                    mod={mod}
                    tracked={activeProfile?.mods.some((item) => item.modId === mod.id) ?? false}
                    onToggle={onToggle}
                    onDelete={onDelete}
                    onConfigure={openConfigEditor}
                  />
                ))}
              </div>
            )}
          </div>
        </section>
      </div>

      {editor && (
        <ConfigEditor
          state={editor}
          busy={busy}
          onChange={(content) => setEditor({ ...editor, content, error: "" })}
          onClose={() => setEditor(null)}
          onSave={() => void saveConfigEditor()}
        />
      )}
    </PageFrame>
  );
}

interface ConfigEditorState {
  modName: string;
  path: string;
  content: string;
  error: string;
}

const filterItems: Array<{ key: FilterType; label: string }> = [
  { key: "all", label: "全部" },
  { key: "enabled", label: "启用" },
  { key: "disabled", label: "停用" },
  { key: "package", label: "资源包" },
  { key: "native", label: "DLL" },
];

function getPotentialConflicts(mods: ModInfo[]) {
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
    .slice(0, 6);
}

function CompactStat({ label, value }: { label: string; value: string }) {
  return (
    <div className="flex items-center justify-between rounded-lg border border-border bg-surface px-3 py-2">
      <span className="text-xs text-text-muted">{label}</span>
      <span className="text-sm font-semibold text-text-primary">{value}</span>
    </div>
  );
}

function ConfigEditor({
  state,
  busy,
  onChange,
  onClose,
  onSave,
}: {
  state: ConfigEditorState;
  busy: boolean;
  onChange: (content: string) => void;
  onClose: () => void;
  onSave: () => void;
}) {
  const fileName = state.path.split(/[\\/]/).pop() ?? state.path;

  return (
    <div className="fixed inset-0 z-50 grid place-items-center bg-black/55 px-6">
      <section className="flex h-[78vh] w-full max-w-4xl flex-col rounded-xl border border-border bg-panel shadow-2xl">
        <div className="flex shrink-0 items-start justify-between gap-4 border-b border-border px-5 py-4">
          <div className="min-w-0">
            <div className="text-sm font-semibold text-text-primary">编辑 DLL 配置</div>
            <div className="mt-1 truncate text-xs text-text-muted" title={state.path}>
              {state.modName} / {fileName}
            </div>
          </div>
          <button
            type="button"
            onClick={onClose}
            className="rounded-lg border border-border px-3 py-1.5 text-sm text-text-secondary transition-colors hover:bg-surface hover:text-text-primary"
          >
            关闭
          </button>
        </div>

        <div className="min-h-0 flex-1 p-5">
          <textarea
            value={state.content}
            onChange={(event) => onChange(event.target.value)}
            spellCheck={false}
            className="h-full w-full resize-none rounded-lg border border-border bg-surface p-4 font-mono text-sm leading-6 text-text-primary outline-none transition-colors focus:border-accent/65"
          />
        </div>

        <div className="flex shrink-0 items-center justify-between gap-4 border-t border-border px-5 py-4">
          <div className="min-w-0 text-xs text-danger">
            {state.error ? `JSON 格式错误：${state.error}` : ""}
          </div>
          <button
            type="button"
            disabled={busy}
            onClick={onSave}
            className="rounded-lg bg-accent px-5 py-2 text-sm font-semibold text-black transition-colors hover:bg-accent-hover disabled:opacity-50"
          >
            保存配置
          </button>
        </div>
      </section>
    </div>
  );
}

function SearchIcon() {
  return (
    <svg
      className="pointer-events-none absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-text-muted"
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
