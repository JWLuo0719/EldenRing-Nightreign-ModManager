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
