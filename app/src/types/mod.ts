export interface ModInfo {
  id: string;
  name: string;
  description: string;
  version: string;
  author: string;
  enabled: boolean;
  path: string;
  type: "package" | "native";
  files: string[];
  source: "local" | "game_native" | "external_package" | "external_native";
  configFiles: string[];
  icon?: string;
}

export interface Profile {
  id: string;
  name: string;
  description: string;
  icon: string;
  mods: ProfileMod[];
  isActive: boolean;
  createdAt: string;
}

export interface ProfileMod {
  modId: string;
  enabled: boolean;
  loadOrder: number;
}

export interface AppConfig {
  gamePath: string;
  me3Path: string;
  launchExePath: string;
  language: string;
  theme: string;
}

export type PageKey = "launch" | "mods" | "profiles" | "diagnostics" | "settings";

export type ToastType = "success" | "error" | "info";

export interface Toast {
  id: number;
  type: ToastType;
  message: string;
}

export interface ConfirmState {
  title: string;
  message: string;
  confirmText: string;
  danger?: boolean;
  onConfirm: () => Promise<void>;
}

export interface LaunchArtifacts {
  profilePath: string;
  profileContent: string;
  scriptPath: string;
  scriptContent: string;
  logPath: string;
  logContent: string;
}

export interface ConflictOwner {
  modId: string;
  modName: string;
  sourcePath: string;
}

export interface FileConflict {
  relativePath: string;
  owners: ConflictOwner[];
}

export interface SpecialModStatus {
  gamePath: string;
  seamlessInstalled: boolean;
  onlinefixInstalled: boolean;
  nighterAvailable: boolean;
  nighterPath: string;
  nighterConfigPath: string;
  missingGameFiles: string[];
}

export interface Me3Profile {
  profileVersion: string;
  supports: Me3Support[];
  packages: Me3Package[];
  natives: Me3Native[];
}

export interface Me3Support {
  game: string;
}

export interface Me3Package {
  path: string;
}

export interface Me3Native {
  path: string;
  load_early?: boolean;
  load_before?: { id: string; optional: boolean }[];
}
