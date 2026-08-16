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
  authorProfile: boolean;
  networkBackend: "none" | "seamless" | "server_redirector";
  savefile: string;
  startOnline: boolean | null;
  profileMode: "author" | "mmv_seamless_community";
  pathAvailable: boolean;
  clothing: ClothingModInfo;
  icon?: string;
}

export interface ClothingModInfo {
  detected: boolean;
  kind: "none" | "replacement" | "expanded";
  partFileCount: number;
  localPartFileCount: number;
  onlinePartFileCount: number;
  pairedPartFileCount: number;
  missingOnlinePartCount: number;
  orphanOnlinePartCount: number;
  hasRegulation: boolean;
  hasManualOnlineSetup: boolean;
  onlineSupport: "not_applicable" | "missing" | "partial" | "complete";
  requiresAppearanceReset: boolean;
  appearanceIds: string[];
  warnings: string[];
}

export interface ModInstallResult {
  path: string;
  zhocnLayoutNormalized: boolean;
  enabled: boolean;
  clothing: ClothingModInfo;
}

export interface ExternalModRelinkResult {
  oldModId: string;
  newModId: string;
  path: string;
  enabled: boolean;
  clothing: ClothingModInfo;
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
  communityCompatibilityMode: boolean;
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

export interface MultiplayerManifest {
  schemaVersion: number;
  generatedAt: string;
  managerVersion: string;
  runtimeEnvironment: string;
  networkBackend: string;
  packages: MultiplayerPackageFingerprint[];
  natives: MultiplayerNativeFingerprint[];
  runtimeFiles: MultiplayerFileFingerprint[];
  seamlessSettingsSha256: string | null;
  overallSha256: string;
  warnings: string[];
}

export interface MultiplayerPackageFingerprint {
  order: number;
  name: string;
  fileCount: number;
  totalBytes: number;
  treeSha256: string;
  regulationSha256: string | null;
  zhocnItemSha256: string | null;
  zhocnMenuSha256: string | null;
}

export interface MultiplayerNativeFingerprint {
  order: number;
  name: string;
  size: number;
  sha256: string;
  loadEarly: boolean;
}

export interface MultiplayerFileFingerprint {
  name: string;
  size: number;
  sha256: string;
}

export interface MultiplayerManifestExport {
  path: string;
  manifest: MultiplayerManifest;
}

export interface MultiplayerManifestDifference {
  severity: "error" | "warning";
  category: string;
  item: string;
  local: string;
  peer: string;
}

export interface MultiplayerManifestComparison {
  compatible: boolean;
  local: MultiplayerManifest;
  peer: MultiplayerManifest;
  differences: MultiplayerManifestDifference[];
}

export interface SpecialModStatus {
  gamePath: string;
  seamlessInstalled: boolean;
  onlinefixInstalled: boolean;
  serverRedirectorConflicts: string[];
  nighterAvailable: boolean;
  nighterLoaded: boolean;
  nighterPath: string;
  nighterConfigPath: string;
  missingGameFiles: string[];
  latestPatchBackup: string;
}

export type RuntimeEnvironment =
  | "auto"
  | "steam_official"
  | "steam_seamless"
  | "spacewar_seamless";

export interface RuntimeEnvironmentStatus {
  configured: RuntimeEnvironment;
  detected: RuntimeEnvironment | "unknown_mixed";
  effective: RuntimeEnvironment | "unknown_mixed";
  verified: boolean;
  confidence: "low" | "medium" | "high";
  evidence: string[];
  warnings: string[];
}

export type PreflightStatus = "pass" | "warning" | "error";

export interface LaunchPreflightCheck {
  id: string;
  label: string;
  status: PreflightStatus;
  message: string;
}

export interface LaunchPreflight {
  ready: boolean;
  checks: LaunchPreflightCheck[];
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
