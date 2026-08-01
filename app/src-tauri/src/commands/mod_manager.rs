use super::profile;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::ffi::OsStr;
use std::fs::{self, File};
use std::io::{self, Read, Seek, SeekFrom, Write};
use std::path::{Component, Path, PathBuf};
use std::process::Stdio;
use std::time::{Duration, Instant};
use tauri::command;
use zip::ZipArchive;

const MAX_LAUNCH_LOG_BYTES: u64 = 2 * 1024 * 1024;
const MAX_LAUNCH_LOG_READ_BYTES: u64 = 512 * 1024;
const MAX_CONFLICT_FILES_SCANNED: usize = 500_000;
const MAX_CONFLICT_RESULTS: usize = 10_000;
const MAX_SCAN_DEPTH: usize = 64;
const MAX_CLOTHING_SCAN_FILES: usize = 100_000;
const MAX_MULTIPLAYER_MANIFEST_BYTES: u64 = 2 * 1024 * 1024;
const RECOMMENDED_ME3_VERSION: (u32, u32, u32) = (0, 12, 1);
const OFFICIAL_MOD_SAVEFILE: &str = "NR0000.nmm";

#[cfg(windows)]
use std::os::windows::process::CommandExt;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub version: String,
    pub author: String,
    pub enabled: bool,
    pub path: String,
    #[serde(rename = "type")]
    pub mod_type: String,
    pub files: Vec<String>,
    pub source: String,
    #[serde(rename = "configFiles")]
    pub config_files: Vec<String>,
    #[serde(rename = "authorProfile")]
    pub author_profile: bool,
    #[serde(rename = "networkBackend")]
    pub network_backend: String,
    pub savefile: String,
    #[serde(rename = "startOnline")]
    pub start_online: Option<bool>,
    #[serde(rename = "profileMode")]
    pub profile_mode: ExternalProfileMode,
    #[serde(rename = "pathAvailable")]
    pub path_available: bool,
    #[serde(default)]
    pub clothing: ClothingModInfo,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExternalModRelinkResult {
    pub old_mod_id: String,
    pub new_mod_id: String,
    pub path: String,
    pub enabled: bool,
    pub clothing: ClothingModInfo,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ClothingModInfo {
    pub detected: bool,
    pub kind: String,
    pub part_file_count: usize,
    pub local_part_file_count: usize,
    pub online_part_file_count: usize,
    pub paired_part_file_count: usize,
    pub missing_online_part_count: usize,
    pub orphan_online_part_count: usize,
    pub has_regulation: bool,
    pub has_manual_online_setup: bool,
    pub online_support: String,
    pub requires_appearance_reset: bool,
    pub appearance_ids: Vec<String>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModInstallResult {
    pub path: String,
    pub zhocn_layout_normalized: bool,
    pub enabled: bool,
    pub clothing: ClothingModInfo,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AppConfig {
    pub game_path: String,
    pub me3_path: String,
    #[serde(default)]
    pub launch_exe_path: String,
    #[serde(default)]
    pub runtime_environment: RuntimeEnvironment,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LaunchArtifacts {
    pub profile_path: String,
    pub profile_content: String,
    pub script_path: String,
    pub script_content: String,
    pub log_path: String,
    pub log_content: String,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ConflictOwner {
    pub mod_id: String,
    pub mod_name: String,
    pub source_path: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FileConflict {
    pub relative_path: String,
    pub owners: Vec<ConflictOwner>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SpecialModStatus {
    pub game_path: String,
    pub seamless_installed: bool,
    pub onlinefix_installed: bool,
    pub server_redirector_conflicts: Vec<String>,
    pub nighter_available: bool,
    pub nighter_path: String,
    pub nighter_config_path: String,
    pub missing_game_files: Vec<String>,
    pub latest_patch_backup: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeEnvironmentStatus {
    pub configured: String,
    pub detected: String,
    pub effective: String,
    pub verified: bool,
    pub confidence: String,
    pub evidence: Vec<String>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LaunchPreflight {
    pub ready: bool,
    pub checks: Vec<LaunchPreflightCheck>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LaunchPreflightCheck {
    pub id: String,
    pub label: String,
    pub status: String,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct MultiplayerManifest {
    pub schema_version: u32,
    pub generated_at: String,
    pub manager_version: String,
    pub runtime_environment: String,
    pub network_backend: String,
    pub packages: Vec<MultiplayerPackageFingerprint>,
    pub natives: Vec<MultiplayerNativeFingerprint>,
    pub runtime_files: Vec<MultiplayerFileFingerprint>,
    pub seamless_settings_sha256: Option<String>,
    pub overall_sha256: String,
    pub warnings: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct MultiplayerPackageFingerprint {
    pub order: usize,
    pub name: String,
    pub file_count: usize,
    pub total_bytes: u64,
    pub tree_sha256: String,
    pub regulation_sha256: Option<String>,
    pub zhocn_item_sha256: Option<String>,
    pub zhocn_menu_sha256: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct MultiplayerNativeFingerprint {
    pub order: usize,
    pub name: String,
    pub size: u64,
    pub sha256: String,
    pub load_early: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct MultiplayerFileFingerprint {
    pub name: String,
    pub size: u64,
    pub sha256: String,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MultiplayerManifestExport {
    pub path: String,
    pub manifest: MultiplayerManifest,
}

#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct MultiplayerManifestDifference {
    pub severity: String,
    pub category: String,
    pub item: String,
    pub local: String,
    pub peer: String,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MultiplayerManifestComparison {
    pub compatible: bool,
    pub local: MultiplayerManifest,
    pub peer: MultiplayerManifest,
    pub differences: Vec<MultiplayerManifestDifference>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct TasklistProcess {
    name: String,
    pid: String,
}

#[derive(Debug, Clone, PartialEq)]
struct PackageEntry {
    path: PathBuf,
    fields: toml::Table,
}

#[derive(Debug, Clone, PartialEq)]
struct NativeEntry {
    path: PathBuf,
    load_early: bool,
    fields: toml::Table,
}

#[derive(Debug, Clone, Default, PartialEq)]
struct AuthorProfileMetadata {
    root_fields: toml::Table,
    source_paths: Vec<PathBuf>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
enum NetworkBackend {
    #[default]
    None,
    Seamless,
    ServerRedirector,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, Default, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ExternalProfileMode {
    #[default]
    Author,
    MmvSeamlessCommunity,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, Default, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeEnvironment {
    #[default]
    Auto,
    SteamOfficial,
    SteamSeamless,
    SpacewarSeamless,
    UnknownMixed,
}

impl RuntimeEnvironment {
    fn as_str(self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::SteamOfficial => "steam_official",
            Self::SteamSeamless => "steam_seamless",
            Self::SpacewarSeamless => "spacewar_seamless",
            Self::UnknownMixed => "unknown_mixed",
        }
    }
}

impl NetworkBackend {
    fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Seamless => "seamless",
            Self::ServerRedirector => "server_redirector",
        }
    }
}

#[derive(Debug)]
struct GeneratedProfilePlan {
    content: String,
    network_backend: NetworkBackend,
    author_profile_sources: Vec<PathBuf>,
    savefile: Option<String>,
    start_online: Option<bool>,
    selected_mod_count: usize,
    package_count: usize,
    native_count: usize,
    mmv_seamless_community_count: usize,
    regulation_files: Vec<PathBuf>,
    zhocn_packages: Vec<PathBuf>,
    packages: Vec<PackageEntry>,
    natives: Vec<NativeEntry>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PatchBackupManifest {
    game_path: String,
    created_at: String,
    files: Vec<PatchBackupFile>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PatchBackupFile {
    relative_path: String,
    existed: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
struct GameplayLaunchRecord {
    savefile: String,
    regulation_sha256: Option<String>,
    runtime_environment: String,
    selected_mod_count: usize,
    recorded_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
struct ExternalModsConfig {
    #[serde(default)]
    packages: Vec<ExternalModEntry>,
    #[serde(default)]
    natives: Vec<ExternalNativeEntry>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct ExternalModEntry {
    path: String,
    #[serde(default = "default_true")]
    enabled: bool,
    #[serde(default)]
    profile_mode: ExternalProfileMode,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct ExternalNativeEntry {
    path: String,
    #[serde(default = "default_true")]
    enabled: bool,
    #[serde(default)]
    load_early: bool,
}

fn default_true() -> bool {
    true
}

fn get_config_dir() -> PathBuf {
    let config_dir = dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("nightreign-mod-manager");
    fs::create_dir_all(&config_dir).ok();
    config_dir
}

fn get_config_path() -> PathBuf {
    get_config_dir().join("config.json")
}

fn get_external_config_path() -> PathBuf {
    get_config_dir().join("external_mods.json")
}

fn get_generated_profile_path() -> PathBuf {
    get_config_dir().join("active-nightreign.me3")
}

fn get_launch_script_path() -> PathBuf {
    let launch_dir = get_config_dir().join("launch");
    fs::create_dir_all(&launch_dir).ok();
    launch_dir.join("launch-nightreign.bat")
}

fn get_launch_log_path() -> PathBuf {
    let launch_dir = get_config_dir().join("launch");
    fs::create_dir_all(&launch_dir).ok();
    launch_dir.join("last-launch.log")
}

fn get_gameplay_launch_record_path() -> PathBuf {
    let launch_dir = get_config_dir().join("launch");
    fs::create_dir_all(&launch_dir).ok();
    launch_dir.join("last-gameplay-profile.json")
}

fn load_config() -> AppConfig {
    let config_path = get_config_path();
    if config_path.exists() {
        let content = fs::read_to_string(&config_path).unwrap_or_default();
        serde_json::from_str(&content).unwrap_or(AppConfig {
            game_path: String::new(),
            me3_path: String::new(),
            launch_exe_path: String::new(),
            runtime_environment: RuntimeEnvironment::Auto,
        })
    } else {
        AppConfig {
            game_path: String::new(),
            me3_path: String::new(),
            launch_exe_path: String::new(),
            runtime_environment: RuntimeEnvironment::Auto,
        }
    }
}

fn save_config(config: &AppConfig) -> Result<(), String> {
    let config_path = get_config_path();
    let content = serde_json::to_string_pretty(config).map_err(|e| e.to_string())?;
    fs::write(&config_path, content).map_err(|e| e.to_string())
}

fn load_external_mods_config() -> ExternalModsConfig {
    let config_path = get_external_config_path();
    let mut config = if config_path.exists() {
        let content = fs::read_to_string(&config_path).unwrap_or_default();
        serde_json::from_str(&content).unwrap_or_default()
    } else {
        ExternalModsConfig::default()
    };
    for entry in &mut config.packages {
        entry.path = normalize_windows_path_string(&entry.path);
    }
    for entry in &mut config.natives {
        entry.path = normalize_windows_path_string(&entry.path);
    }
    config
}

fn save_external_mods_config(config: &ExternalModsConfig) -> Result<(), String> {
    let config_path = get_external_config_path();
    let content = serde_json::to_string_pretty(config).map_err(|e| e.to_string())?;
    fs::write(&config_path, content).map_err(|e| e.to_string())
}

fn mods_dir_from_config(config: &AppConfig) -> Result<PathBuf, String> {
    if config.game_path.trim().is_empty() {
        return Err("请先设置游戏目录".to_string());
    }
    Ok(Path::new(&config.game_path).join("mods"))
}

#[command]
pub fn get_game_path() -> String {
    load_config().game_path
}

#[command]
pub fn set_game_path(path: String) -> Result<(), String> {
    if !Path::new(&path).join("nightreign.exe").exists() {
        return Err("游戏目录无效，请选择包含 nightreign.exe 的文件夹".to_string());
    }
    let mut config = load_config();
    if !config.launch_exe_path.trim().is_empty()
        && !is_path_inside_dir(Path::new(&config.launch_exe_path), Path::new(&path))
    {
        config.launch_exe_path.clear();
    }
    config.game_path = path;
    save_config(&config)
}

#[command]
pub fn get_runtime_environment_status() -> RuntimeEnvironmentStatus {
    let config = load_config();
    build_runtime_environment_status(&config)
}

#[command]
pub fn set_runtime_environment(environment: RuntimeEnvironment) -> Result<(), String> {
    if environment == RuntimeEnvironment::UnknownMixed {
        return Err("无法把“未知/混合环境”保存为运行模式，请选择自动检测或明确环境。".to_string());
    }
    let mut config = load_config();
    config.runtime_environment = environment;
    save_config(&config)
}

#[command]
pub fn get_me3_path() -> String {
    load_config().me3_path
}

#[command]
pub fn set_me3_path(path: String) -> Result<(), String> {
    find_me3_exe(Path::new(&path))?;
    let mut config = load_config();
    config.me3_path = path;
    save_config(&config)
}

#[command]
pub fn get_launch_exe_path() -> String {
    load_config().launch_exe_path
}

#[command]
pub fn set_launch_exe_path(path: String) -> Result<(), String> {
    let mut config = load_config();
    let trimmed = path.trim();

    if trimmed.is_empty() {
        config.launch_exe_path.clear();
        return save_config(&config);
    }

    if config.game_path.trim().is_empty() {
        return Err("请先设置游戏目录".to_string());
    }

    let launch_exe = Path::new(trimmed);
    validate_launch_exe(launch_exe, Path::new(&config.game_path))?;
    config.launch_exe_path = trimmed.to_string();
    save_config(&config)
}

#[command]
pub fn get_mods_dir() -> Result<String, String> {
    let config = load_config();
    Ok(mods_dir_from_config(&config)?.to_string_lossy().to_string())
}

#[command]
pub async fn scan_mods() -> Result<Vec<ModInfo>, String> {
    tauri::async_runtime::spawn_blocking(collect_mods)
        .await
        .map_err(|error| format!("扫描任务异常结束：{error}"))?
}

fn collect_mods() -> Result<Vec<ModInfo>, String> {
    let config = load_config();
    let mods_dir = mods_dir_from_config(&config)?;

    let mut mods = Vec::new();

    if mods_dir.exists() {
        for entry in fs::read_dir(&mods_dir)
            .map_err(|e| e.to_string())?
            .flatten()
        {
            let path = entry.path();
            if path.is_dir() {
                let enabled = !is_disabled_path(&path);
                if let Some(mod_info) = parse_mod_folder(&path, enabled) {
                    mods.push(mod_info);
                }
            } else if is_dll_or_disabled_dll(&path) {
                if let Some(mod_info) = parse_native_file(&path, "game_native") {
                    mods.push(mod_info);
                }
            }
        }
    }

    mods.extend(collect_external_mods());
    mods.sort_by_key(|item| item.name.to_lowercase());
    Ok(mods)
}

fn parse_mod_folder(path: &Path, enabled: bool) -> Option<ModInfo> {
    let folder_name = path.file_name()?.to_string_lossy().to_string();
    let id = strip_disabled_suffix(&folder_name).to_string();

    let me3_files = find_top_level_me3_files(path);
    let (mut description, version, mod_type) = if let Some(me3_file) = me3_files.first() {
        let content = fs::read_to_string(me3_file).unwrap_or_default();
        parse_me3_content(&content)
    } else if has_dll_file(path) && !has_package_like_content(path) {
        (String::new(), String::new(), "native".to_string())
    } else {
        (String::new(), String::new(), "package".to_string())
    };
    if description.is_empty() && is_complete_zhocn_package(path) {
        description = "完整简体中文文本覆盖层；启动前会检查是否重复并显示关键文件指纹".to_string();
    }

    let files = fs::read_dir(path)
        .ok()?
        .flatten()
        .map(|e| e.file_name().to_string_lossy().to_string())
        .collect();
    let profile_summary = inspect_author_profile(path);
    let clothing = analyze_clothing_mod(path);

    Some(ModInfo {
        id: id.clone(),
        name: extract_display_name(&id),
        description,
        version,
        author: String::new(),
        enabled,
        path: normalize_windows_path_string(&path.to_string_lossy()),
        mod_type,
        files,
        source: "local".to_string(),
        config_files: find_config_files(path),
        author_profile: profile_summary.author_profile,
        network_backend: profile_summary.network_backend.as_str().to_string(),
        savefile: profile_summary.savefile.unwrap_or_default(),
        start_online: profile_summary.start_online,
        profile_mode: ExternalProfileMode::Author,
        path_available: true,
        clothing,
    })
}

fn parse_native_file(path: &Path, source: &str) -> Option<ModInfo> {
    let file_name = path.file_name()?.to_string_lossy().to_string();
    let id = strip_disabled_suffix(&file_name).to_string();
    let enabled = !is_disabled_path(path);
    let active_path = if enabled {
        path.to_path_buf()
    } else {
        active_path_for(path)
    };
    let mut files = vec![file_name];
    for config_file in find_sidecar_config_files(&active_path) {
        if let Some(name) = config_file.file_name().and_then(|name| name.to_str()) {
            files.push(name.to_string());
        }
    }

    Some(ModInfo {
        id: id.clone(),
        name: extract_display_name(&id),
        description: String::new(),
        version: String::new(),
        author: String::new(),
        enabled,
        path: normalize_windows_path_string(&path.to_string_lossy()),
        mod_type: "native".to_string(),
        files,
        source: source.to_string(),
        config_files: find_sidecar_config_files(&active_path)
            .into_iter()
            .map(|path| path.to_string_lossy().to_string())
            .collect(),
        author_profile: false,
        network_backend: network_backend_for_native_path(&active_path)
            .as_str()
            .to_string(),
        savefile: String::new(),
        start_online: None,
        profile_mode: ExternalProfileMode::Author,
        path_available: path.exists(),
        clothing: ClothingModInfo::default(),
    })
}

fn collect_external_mods() -> Vec<ModInfo> {
    let config = load_external_mods_config();
    let mut mods = Vec::new();

    for entry in config.packages {
        let path = PathBuf::from(&entry.path);
        let mut mod_info = parse_mod_folder(&path, entry.enabled)
            .unwrap_or_else(|| external_package_fallback(&path, entry.enabled));
        mod_info.id = external_id("package", &path);
        mod_info.source = "external_package".to_string();
        mod_info.enabled = entry.enabled;
        mod_info.profile_mode = entry.profile_mode;
        if entry.profile_mode == ExternalProfileMode::MmvSeamlessCommunity {
            mod_info.network_backend = NetworkBackend::Seamless.as_str().to_string();
            mod_info.description =
                "社区 Seamless 兼容模式：只在生成副本中用 nrsc.dll 替代作者 Server Redirector"
                    .to_string();
        }
        mods.push(mod_info);
    }

    for entry in config.natives {
        let path = PathBuf::from(&entry.path);
        let mut mod_info = parse_native_file(&path, "external_native")
            .unwrap_or_else(|| external_native_fallback(&path, entry.enabled));
        mod_info.id = external_id("native", &path);
        mod_info.source = "external_native".to_string();
        mod_info.enabled = entry.enabled;
        mods.push(mod_info);
    }

    mods
}

fn external_package_fallback(path: &Path, enabled: bool) -> ModInfo {
    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("External Mod")
        .to_string();
    ModInfo {
        id: external_id("package", path),
        name: extract_display_name(&name),
        description: "外部目录未找到或无法解析".to_string(),
        version: String::new(),
        author: String::new(),
        enabled,
        path: normalize_windows_path_string(&path.to_string_lossy()),
        mod_type: "package".to_string(),
        files: Vec::new(),
        source: "external_package".to_string(),
        config_files: find_config_files(path),
        author_profile: false,
        network_backend: "none".to_string(),
        savefile: String::new(),
        start_online: None,
        profile_mode: ExternalProfileMode::Author,
        path_available: false,
        clothing: analyze_clothing_mod(path),
    }
}

fn external_native_fallback(path: &Path, enabled: bool) -> ModInfo {
    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("外部功能插件")
        .to_string();
    ModInfo {
        id: external_id("native", path),
        name,
        description: "外部功能插件未找到或无法解析".to_string(),
        version: String::new(),
        author: String::new(),
        enabled,
        path: normalize_windows_path_string(&path.to_string_lossy()),
        mod_type: "native".to_string(),
        files: Vec::new(),
        source: "external_native".to_string(),
        config_files: find_sidecar_config_files(path)
            .into_iter()
            .map(|path| path.to_string_lossy().to_string())
            .collect(),
        author_profile: false,
        network_backend: network_backend_for_native_path(path).as_str().to_string(),
        savefile: String::new(),
        start_online: None,
        profile_mode: ExternalProfileMode::Author,
        path_available: false,
        clothing: ClothingModInfo::default(),
    }
}

fn external_id(kind: &str, path: &Path) -> String {
    let resolved = fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
    format!("external:{kind}:{}", path_key(&resolved))
}

fn find_top_level_me3_files(path: &Path) -> Vec<PathBuf> {
    fs::read_dir(path)
        .ok()
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|e| {
            let path = e.path();
            let is_me3 = path
                .extension()
                .and_then(|ext| ext.to_str())
                .is_some_and(|ext| ext.eq_ignore_ascii_case("me3"));
            is_me3.then_some(path)
        })
        .collect()
}

fn parse_me3_content(content: &str) -> (String, String, String) {
    let mut description = String::new();
    let version = String::new();
    let mut has_native = false;
    let mut has_package = false;

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("# 描述:") || trimmed.starts_with("# Description:") {
            description = trimmed
                .split(':')
                .skip(1)
                .collect::<Vec<_>>()
                .join(":")
                .trim()
                .to_string();
        }
        if trimmed.starts_with("[[natives]]") {
            has_native = true;
        }
        if trimmed.starts_with("[[packages]]") || trimmed.starts_with("[[package]]") {
            has_package = true;
        }
    }

    let mod_type = if has_native && !has_package {
        "native".to_string()
    } else {
        "package".to_string()
    };

    (description, version, mod_type)
}

#[derive(Debug, Default)]
struct AuthorProfileSummary {
    author_profile: bool,
    network_backend: NetworkBackend,
    savefile: Option<String>,
    start_online: Option<bool>,
}

fn inspect_author_profile(mod_dir: &Path) -> AuthorProfileSummary {
    let mut summary = AuthorProfileSummary::default();

    for me3_file in find_top_level_me3_files(mod_dir) {
        let Ok(content) = fs::read_to_string(me3_file) else {
            continue;
        };
        let Ok(value) = content.parse::<toml::Value>() else {
            continue;
        };

        summary.author_profile = true;
        if summary.savefile.is_none() {
            summary.savefile = value
                .get("savefile")
                .and_then(toml::Value::as_str)
                .map(ToOwned::to_owned);
        }
        if summary.start_online.is_none() {
            summary.start_online = value.get("start_online").and_then(toml::Value::as_bool);
        }

        if let Some(natives) = value.get("natives").and_then(toml::Value::as_array) {
            for native in natives {
                let Some(path) = native.get("path").and_then(toml::Value::as_str) else {
                    continue;
                };
                let backend = network_backend_for_native_path(Path::new(path));
                match (summary.network_backend, backend) {
                    (_, NetworkBackend::ServerRedirector) => {
                        summary.network_backend = NetworkBackend::ServerRedirector;
                    }
                    (NetworkBackend::None, NetworkBackend::Seamless) => {
                        summary.network_backend = NetworkBackend::Seamless;
                    }
                    _ => {}
                }
            }
        }
    }

    summary
}

fn extract_display_name(folder_name: &str) -> String {
    if let Some(end) = folder_name.find('-') {
        let prefix = &folder_name[..end];
        if prefix.chars().all(|c| c.is_ascii_digit()) || prefix.is_empty() {
            return folder_name[end + 1..].to_string();
        }
    }
    folder_name.to_string()
}

#[command]
pub fn get_mod_info(mod_path: String) -> Result<ModInfo, String> {
    let path = Path::new(&mod_path);
    parse_mod_folder(path, !is_disabled_path(path)).ok_or_else(|| "无法解析mod信息".to_string())
}

#[command]
pub async fn install_mod_from_zip(zip_path: String) -> Result<ModInstallResult, String> {
    tauri::async_runtime::spawn_blocking(move || install_mod_from_zip_blocking(&zip_path))
        .await
        .map_err(|error| format!("ZIP 安装任务异常结束：{error}"))?
}

fn install_mod_from_zip_blocking(zip_path: &str) -> Result<ModInstallResult, String> {
    let zip_path = Path::new(&zip_path);
    let config = load_config();
    let mods_dir = mods_dir_from_config(&config)?;

    if !zip_path.exists() {
        return Err("ZIP文件不存在".to_string());
    }

    fs::create_dir_all(&mods_dir).map_err(|e| e.to_string())?;

    let file_name = zip_path
        .file_stem()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    let staging_dir =
        unique_destination(&mods_dir.join(format!(".installing-{}", current_timestamp())));
    fs::create_dir_all(&staging_dir).map_err(|e| e.to_string())?;

    let zhocn_layout_normalized = extract_zip(zip_path, &staging_dir)
        .and_then(|_| normalize_zhocn_layout(&staging_dir))
        .inspect_err(|_| {
            let _ = fs::remove_dir_all(&staging_dir);
        })?;

    let clothing = analyze_clothing_mod(&staging_dir);
    let enabled = safe_initial_mod_enabled(&clothing);
    let sanitized_name = sanitize_folder_name(strip_disabled_suffix(&file_name));
    let destination = unique_mod_destination(&mods_dir, &sanitized_name, enabled);
    fs::rename(&staging_dir, &destination)
        .inspect_err(|_| {
            let _ = fs::remove_dir_all(&staging_dir);
        })
        .map_err(|error| format!("完成 Mod 安装失败：{error}"))?;

    Ok(ModInstallResult {
        path: destination.to_string_lossy().to_string(),
        zhocn_layout_normalized,
        enabled,
        clothing,
    })
}

#[command]
pub fn add_external_mod(path: String) -> Result<ModInstallResult, String> {
    let mod_path =
        fs::canonicalize(Path::new(path.trim())).map_err(|e| format!("外部 Mod 目录无效：{e}"))?;
    if !mod_path.is_dir() {
        return Err("外部 Mod 必须选择文件夹".to_string());
    }

    let clothing = analyze_clothing_mod(&mod_path);
    let mut config = load_external_mods_config();
    let normalized = normalize_windows_path_string(&mod_path.to_string_lossy());
    let enabled = if let Some(existing) = config
        .packages
        .iter()
        .find(|entry| same_path_string(&entry.path, &normalized))
    {
        existing.enabled
    } else {
        let enabled = safe_initial_mod_enabled(&clothing);
        config.packages.push(ExternalModEntry {
            path: normalized.clone(),
            enabled,
            profile_mode: ExternalProfileMode::Author,
        });
        enabled
    };
    save_external_mods_config(&config)?;

    Ok(ModInstallResult {
        path: normalized,
        zhocn_layout_normalized: false,
        enabled,
        clothing,
    })
}

#[command]
pub fn relink_external_mod(
    mod_id: String,
    path: String,
) -> Result<ExternalModRelinkResult, String> {
    let mod_path = fs::canonicalize(Path::new(path.trim()))
        .map_err(|error| format!("新的外部 Mod 目录无效：{error}"))?;
    if !mod_path.is_dir() {
        return Err("请重新选择这个 Mod 的文件夹，而不是单个文件".to_string());
    }

    let mut config = load_external_mods_config();
    let original_config = config.clone();
    let old_index = config
        .packages
        .iter()
        .position(|entry| external_id("package", Path::new(&entry.path)) == mod_id)
        .ok_or_else(|| "没有找到需要重新定位的外部 Mod 记录".to_string())?;
    let old_mod_id = external_id("package", Path::new(&config.packages[old_index].path));
    let normalized = normalize_windows_path_string(&mod_path.to_string_lossy());
    let new_mod_id = external_id("package", &mod_path);
    let old_entry = config.packages[old_index].clone();

    if let Some(existing_index) = config
        .packages
        .iter()
        .enumerate()
        .find_map(|(index, entry)| {
            (index != old_index && same_path_string(&entry.path, &normalized)).then_some(index)
        })
    {
        config.packages[existing_index].enabled |= old_entry.enabled;
        config.packages.remove(old_index);
    } else {
        config.packages[old_index].path = normalized.clone();
    }
    save_external_mods_config(&config)?;

    if let Err(error) = profile::replace_mod_id_in_all_profiles(&old_mod_id, &new_mod_id) {
        let _ = save_external_mods_config(&original_config);
        return Err(format!("已撤销路径修改，因为同步配置方案失败：{error}"));
    }

    let entry = config
        .packages
        .iter()
        .find(|entry| same_path_string(&entry.path, &normalized))
        .ok_or_else(|| "重新定位后无法读取外部 Mod 记录".to_string())?;
    Ok(ExternalModRelinkResult {
        old_mod_id,
        new_mod_id,
        path: normalized,
        enabled: entry.enabled,
        clothing: analyze_clothing_mod(&mod_path),
    })
}

#[command]
pub fn add_external_dll(path: String) -> Result<(), String> {
    let dll_path = fs::canonicalize(Path::new(path.trim()))
        .map_err(|e| format!("外部功能插件（DLL）无效：{e}"))?;
    if !dll_path.is_file()
        || !dll_path
            .extension()
            .and_then(|ext| ext.to_str())
            .is_some_and(|ext| ext.eq_ignore_ascii_case("dll"))
    {
        return Err("请选择功能插件 .dll 文件".to_string());
    }

    let mut config = load_external_mods_config();
    let normalized = normalize_windows_path_string(&dll_path.to_string_lossy());
    if !config
        .natives
        .iter()
        .any(|entry| same_path_string(&entry.path, &normalized))
    {
        config.natives.push(ExternalNativeEntry {
            path: normalized,
            enabled: true,
            load_early: false,
        });
    }
    save_external_mods_config(&config)
}

#[command]
pub fn remove_external_mod(mod_id: String) -> Result<(), String> {
    let mut config = load_external_mods_config();
    let before_packages = config.packages.len();
    let before_natives = config.natives.len();

    config
        .packages
        .retain(|entry| external_id("package", Path::new(&entry.path)) != mod_id);
    config
        .natives
        .retain(|entry| external_id("native", Path::new(&entry.path)) != mod_id);

    if before_packages == config.packages.len() && before_natives == config.natives.len() {
        return Err("未找到外部 Mod 注册项".to_string());
    }

    save_external_mods_config(&config)
}

#[command]
pub fn toggle_external_mod(mod_id: String, enabled: bool) -> Result<(), String> {
    let mut config = load_external_mods_config();
    let mut found = false;

    for entry in &mut config.packages {
        if external_id("package", Path::new(&entry.path)) == mod_id {
            if enabled && !Path::new(&entry.path).is_dir() {
                return Err(
                    "这个外部 Mod 的原文件夹已经不存在。请先使用“重新定位”选择它现在所在的文件夹。"
                        .to_string(),
                );
            }
            entry.enabled = enabled;
            found = true;
        }
    }
    for entry in &mut config.natives {
        if external_id("native", Path::new(&entry.path)) == mod_id {
            if enabled && !Path::new(&entry.path).is_file() {
                return Err("这个外部功能插件已经不存在。请移除旧记录后重新添加 DLL。".to_string());
            }
            entry.enabled = enabled;
            found = true;
        }
    }

    if !found {
        return Err("未找到外部 Mod 注册项".to_string());
    }

    save_external_mods_config(&config)
}

#[command]
pub fn set_external_mod_profile_mode(
    mod_id: String,
    profile_mode: ExternalProfileMode,
) -> Result<(), String> {
    let mut config = load_external_mods_config();
    let entry = config
        .packages
        .iter_mut()
        .find(|entry| external_id("package", Path::new(&entry.path)) == mod_id)
        .ok_or_else(|| "未找到外部 Mod 注册项".to_string())?;

    if profile_mode == ExternalProfileMode::MmvSeamlessCommunity {
        validate_mmv_seamless_candidate(Path::new(&entry.path))?;
    }

    entry.profile_mode = profile_mode;
    save_external_mods_config(&config)
}

#[command]
pub fn read_mod_config_file(path: String) -> Result<String, String> {
    let config_path = validate_editable_config_path(&path)?;
    fs::read_to_string(&config_path)
        .map_err(|e| format!("读取配置文件失败：{}：{e}", config_path.to_string_lossy()))
}

#[command]
pub fn write_mod_config_file(path: String, content: String) -> Result<(), String> {
    let config_path = validate_editable_config_path(&path)?;
    if config_path
        .extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| ext.eq_ignore_ascii_case("json"))
    {
        serde_json::from_str::<serde_json::Value>(&content)
            .map_err(|e| format!("JSON 格式无效：{e}"))?;
    }
    fs::write(&config_path, content)
        .map_err(|e| format!("写入配置文件失败：{}：{e}", config_path.to_string_lossy()))
}

fn extract_zip(zip_path: &Path, extract_dir: &Path) -> Result<(), String> {
    let file = File::open(zip_path).map_err(|e| e.to_string())?;
    let mut archive = ZipArchive::new(file).map_err(|e| e.to_string())?;
    let single_root = detect_single_zip_root(&mut archive)?;

    for index in 0..archive.len() {
        let mut zip_file = archive.by_index(index).map_err(|e| e.to_string())?;
        let Some(enclosed_name) = zip_file.enclosed_name() else {
            return Err("ZIP中包含不安全路径".to_string());
        };

        let relative_path = strip_zip_root(enclosed_name, single_root.as_deref());
        if relative_path.as_os_str().is_empty() {
            continue;
        }

        let output_path = extract_dir.join(relative_path);
        if zip_file.name().ends_with('/') {
            fs::create_dir_all(&output_path).map_err(|e| e.to_string())?;
        } else {
            if let Some(parent) = output_path.parent() {
                fs::create_dir_all(parent).map_err(|e| e.to_string())?;
            }
            let mut output = File::create(&output_path).map_err(|e| e.to_string())?;
            io::copy(&mut zip_file, &mut output).map_err(|e| e.to_string())?;
        }
    }

    Ok(())
}

fn detect_single_zip_root(archive: &mut ZipArchive<File>) -> Result<Option<PathBuf>, String> {
    let mut root: Option<PathBuf> = None;
    let mut found_file = false;

    for index in 0..archive.len() {
        let zip_file = archive.by_index(index).map_err(|e| e.to_string())?;
        if zip_file.is_dir() {
            continue;
        }
        let Some(path) = zip_file.enclosed_name() else {
            return Err("ZIP中包含不安全路径".to_string());
        };
        let components = path.components().collect::<Vec<_>>();
        let Some(Component::Normal(first)) = components.first() else {
            return Ok(None);
        };
        if components.len() < 2 || is_semantic_zip_root(first) {
            return Ok(None);
        }

        let candidate = PathBuf::from(first);
        if root.as_ref().is_some_and(|current| current != &candidate) {
            return Ok(None);
        }
        root = Some(candidate);
        found_file = true;
    }

    Ok(found_file.then_some(root).flatten())
}

fn is_semantic_zip_root(root: &OsStr) -> bool {
    let root = root.to_string_lossy().to_ascii_lowercase();
    matches!(
        root.as_str(),
        "action"
            | "asset"
            | "chr"
            | "event"
            | "hks"
            | "map"
            | "menu"
            | "msg"
            | "parts"
            | "script"
            | "sd"
            | "sfx"
            | "sound"
            | "zhocn"
    )
}

fn normalize_zhocn_layout(package_root: &Path) -> Result<bool, String> {
    const ITEM_FILE: &str = "item_dlc01.msgbnd.dcx";
    const MENU_FILE: &str = "menu_dlc01.msgbnd.dcx";

    let canonical = package_root.join("msg").join("zhocn");
    let canonical_item = canonical.join(ITEM_FILE);
    let canonical_menu = canonical.join(MENU_FILE);
    if canonical_item.is_file() && canonical_menu.is_file() {
        return Ok(false);
    }

    let candidates = [package_root.join("zhocn"), package_root.to_path_buf()]
        .into_iter()
        .filter(|root| root.join(ITEM_FILE).is_file() && root.join(MENU_FILE).is_file())
        .collect::<Vec<_>>();

    if candidates.is_empty() {
        return Ok(false);
    }
    if candidates.len() > 1 || canonical_item.exists() || canonical_menu.exists() {
        return Err(
            "汉化 ZIP 同时包含多套或不完整的 zhocn 布局，已取消安装以避免覆盖顺序不确定"
                .to_string(),
        );
    }

    let source = &candidates[0];
    fs::create_dir_all(&canonical).map_err(|error| {
        format!(
            "创建标准汉化目录失败 {}：{error}",
            canonical.to_string_lossy()
        )
    })?;
    for file_name in [ITEM_FILE, MENU_FILE] {
        let from = source.join(file_name);
        let to = canonical.join(file_name);
        fs::rename(&from, &to).map_err(|error| {
            format!(
                "规范汉化目录失败 {} -> {}：{error}",
                from.to_string_lossy(),
                to.to_string_lossy()
            )
        })?;
    }

    if source != package_root {
        let _ = fs::remove_dir(source);
    }
    Ok(true)
}

fn strip_zip_root(path: &Path, root: Option<&Path>) -> PathBuf {
    if let Some(root) = root {
        path.strip_prefix(root).unwrap_or(path).to_path_buf()
    } else {
        path.to_path_buf()
    }
}

fn unique_destination(base: &Path) -> PathBuf {
    if !base.exists() {
        return base.to_path_buf();
    }

    let parent = base.parent().unwrap_or_else(|| Path::new("."));
    let name = base
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    for index in 1..1000 {
        let candidate = parent.join(format!("{name}_{index}"));
        if !candidate.exists() {
            return candidate;
        }
    }

    parent.join(format!("{name}_{}", current_timestamp()))
}

fn unique_mod_destination(mods_dir: &Path, name: &str, enabled: bool) -> PathBuf {
    let suffix = if enabled { "" } else { ".disabled" };
    if mod_destination_slot_available(mods_dir, name) {
        return mods_dir.join(format!("{name}{suffix}"));
    }

    for index in 1..1000 {
        let candidate_name = format!("{name}_{index}");
        if mod_destination_slot_available(mods_dir, &candidate_name) {
            return mods_dir.join(format!("{candidate_name}{suffix}"));
        }
    }

    mods_dir.join(format!("{name}_{}{suffix}", current_timestamp()))
}

fn mod_destination_slot_available(mods_dir: &Path, name: &str) -> bool {
    !mods_dir.join(name).exists() && !mods_dir.join(format!("{name}.disabled")).exists()
}

fn sanitize_folder_name(name: &str) -> String {
    let sanitized: String = name
        .chars()
        .map(|c| match c {
            '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*' => '_',
            _ => c,
        })
        .collect();

    let trimmed = sanitized.trim().trim_matches('.').to_string();
    if trimmed.is_empty() {
        format!("mod_{}", current_timestamp())
    } else {
        trimmed
    }
}

#[command]
pub fn uninstall_mod(mod_path: String) -> Result<(), String> {
    let path = validate_managed_mod_path(&mod_path)?;
    if path.exists() {
        trash::delete(&path).map_err(|e| format!("移动 Mod 到回收站失败：{e}"))?;
    }
    Ok(())
}

#[command]
pub fn toggle_mod(mod_path: String, enabled: bool) -> Result<(), String> {
    let path = validate_managed_mod_path(&mod_path)?;

    if enabled {
        let source = if path.exists() {
            path.clone()
        } else {
            disabled_path_for(&path)
        };
        let target = active_path_for(&source);
        if !source.exists() {
            return Err("Mod目录不存在".to_string());
        }
        if source != target {
            if target.exists() {
                return Err("启用失败：目标目录已存在".to_string());
            }
            fs::rename(&source, &target).map_err(|e| e.to_string())?;
        }
    } else if path.exists() {
        let target = disabled_path_for(&path);
        if target.exists() {
            return Err("禁用失败：目标目录已存在".to_string());
        }
        fs::rename(&path, &target).map_err(|e| e.to_string())?;
    }

    Ok(())
}

fn validate_managed_mod_path(mod_path: &str) -> Result<PathBuf, String> {
    let config = load_config();
    let mods_dir = mods_dir_from_config(&config)?;
    validate_direct_child(&mods_dir, Path::new(mod_path.trim()))
}

fn validate_direct_child(managed_root: &Path, candidate: &Path) -> Result<PathBuf, String> {
    let managed_root = fs::canonicalize(managed_root)
        .map_err(|e| format!("Mod 目录不可访问：{}：{e}", managed_root.to_string_lossy()))?;
    let candidate = fs::canonicalize(candidate)
        .map_err(|e| format!("Mod 不存在或不可访问：{}：{e}", candidate.to_string_lossy()))?;
    let parent = candidate
        .parent()
        .ok_or_else(|| "Mod 路径没有父目录".to_string())?;

    if parent != managed_root {
        return Err("拒绝操作：只能启停或删除 Game\\mods 的直属 Mod 项".to_string());
    }

    if !candidate.is_dir() && !is_dll_or_disabled_dll(&candidate) {
        return Err("拒绝操作：目标必须是 Mod 文件夹或功能插件 DLL".to_string());
    }

    Ok(candidate)
}

#[command]
pub fn generate_me3_profile() -> Result<String, String> {
    let plan = build_generated_profile_plan()?;
    let profile_path = write_generated_profile(&plan)?;
    Ok(profile_path.to_string_lossy().to_string())
}

fn write_generated_profile(plan: &GeneratedProfilePlan) -> Result<PathBuf, String> {
    let profile_path = get_generated_profile_path();
    fs::write(&profile_path, &plan.content).map_err(|e| e.to_string())?;
    Ok(profile_path)
}

fn build_generated_profile_plan() -> Result<GeneratedProfilePlan, String> {
    let mods = collect_mods()?;
    let selected_mods = mods_selected_for_generation(&mods);
    let selected_mod_count = selected_mods.len();
    let config = load_config();
    let mut packages = Vec::new();
    let mut natives = Vec::new();
    let mut metadata = AuthorProfileMetadata::default();
    let mut seen_packages = BTreeSet::new();
    let mut seen_natives = BTreeSet::new();
    let mut mmv_seamless_community_count = 0;

    for mod_info in selected_mods {
        let mod_dir = Path::new(&mod_info.path);
        if !mod_dir.exists() {
            return Err(format!(
                "已启用的“{}”找不到原文件夹：{}。请到 Mod 仓库使用“重新定位”，或先停用这条记录。",
                mod_info.name,
                mod_dir.to_string_lossy()
            ));
        }
        let (mod_packages, mut mod_natives, mut mod_metadata) =
            collect_profile_data_for_mod(mod_dir)?;
        if mod_info.profile_mode == ExternalProfileMode::MmvSeamlessCommunity {
            apply_mmv_seamless_community_override(
                mod_dir,
                &mod_packages,
                &mut mod_natives,
                &mut mod_metadata,
                &config,
            )?;
            mmv_seamless_community_count += 1;
        }
        extend_unique_packages(&mut packages, &mut seen_packages, mod_packages);
        extend_unique_natives(&mut natives, &mut seen_natives, mod_natives);
        merge_author_profile_metadata(&mut metadata, mod_metadata)?;
    }

    let selected_backend = detect_network_backend(&natives)?;
    if selected_backend == NetworkBackend::ServerRedirector {
        let game_dir = Path::new(&config.game_path);
        if metadata.source_paths.iter().any(|path| {
            path.parent()
                .is_some_and(|parent| is_path_inside_dir(parent, game_dir))
        }) {
            return Err(
                "Server Redirector 整合包必须放在实际 Game 目录之外；请移动目录后重新注册。"
                    .to_string(),
            );
        }
    }
    if selected_backend != NetworkBackend::ServerRedirector {
        let game_root_natives = infer_game_root_natives(Path::new(&config.game_path));
        extend_unique_natives(&mut natives, &mut seen_natives, game_root_natives);
    }
    let network_backend = detect_network_backend(&natives)?;
    apply_safe_default_savefile(
        &mut metadata,
        effective_runtime_environment(&config),
        network_backend,
        selected_mod_count,
    );
    let savefile = metadata
        .root_fields
        .get("savefile")
        .and_then(toml::Value::as_str)
        .map(ToOwned::to_owned);
    let start_online = metadata
        .root_fields
        .get("start_online")
        .and_then(toml::Value::as_bool);
    let author_profile_sources = metadata.source_paths.clone();
    let regulation_files = collect_regulation_files(&packages);
    let zhocn_packages = collect_zhocn_packages(&packages);
    let content = build_me3_profile(&metadata, &packages, &natives)?;
    let package_count = packages.len();
    let native_count = natives.len();

    Ok(GeneratedProfilePlan {
        content,
        network_backend,
        author_profile_sources,
        savefile,
        start_online,
        selected_mod_count,
        package_count,
        native_count,
        mmv_seamless_community_count,
        regulation_files,
        zhocn_packages,
        packages,
        natives,
    })
}

fn apply_safe_default_savefile(
    metadata: &mut AuthorProfileMetadata,
    environment: RuntimeEnvironment,
    network_backend: NetworkBackend,
    selected_mod_count: usize,
) {
    if environment == RuntimeEnvironment::SteamOfficial
        && network_backend == NetworkBackend::None
        && selected_mod_count > 0
        && !metadata.root_fields.contains_key("savefile")
    {
        metadata.root_fields.insert(
            "savefile".to_string(),
            toml::Value::String(OFFICIAL_MOD_SAVEFILE.to_string()),
        );
    }
}

fn runtime_environment_label(environment: RuntimeEnvironment) -> &'static str {
    match environment {
        RuntimeEnvironment::Auto => "自动检测",
        RuntimeEnvironment::SteamOfficial => "纯正版 Steam",
        RuntimeEnvironment::SteamSeamless => "正版 Steam + Seamless",
        RuntimeEnvironment::SpacewarSeamless => "Spacewar + Seamless",
        RuntimeEnvironment::UnknownMixed => "未知或混合环境",
    }
}

fn effective_save_filename(
    plan: &GeneratedProfilePlan,
    environment: RuntimeEnvironment,
    game_dir: &Path,
) -> Result<String, String> {
    let filename = if plan.network_backend == NetworkBackend::Seamless
        || matches!(
            environment,
            RuntimeEnvironment::SteamSeamless | RuntimeEnvironment::SpacewarSeamless
        ) {
        seamless_save_filename(game_dir).unwrap_or_else(|| "NR0000.co2".to_string())
    } else if let Some(savefile) = plan.savefile.as_deref() {
        savefile.trim().to_string()
    } else {
        "NR0000.sl2".to_string()
    };
    validate_save_filename(&filename)?;
    Ok(filename)
}

fn validate_save_filename(filename: &str) -> Result<(), String> {
    let path = Path::new(filename);
    let mut components = path.components();
    if filename.is_empty()
        || !matches!(components.next(), Some(Component::Normal(_)))
        || components.next().is_some()
        || path.file_name().and_then(|name| name.to_str()) != Some(filename)
    {
        return Err(format!(
            "启动配置（Profile）的存档文件名不安全，已阻止启动：{filename}"
        ));
    }
    Ok(())
}

fn seamless_save_filename(game_dir: &Path) -> Option<String> {
    let settings_path = game_dir.join("SeamlessCoop").join("nrsc_settings.ini");
    let content = fs::read_to_string(settings_path).ok()?;
    let extension = content.lines().find_map(|line| {
        let trimmed = line.trim();
        if trimmed.starts_with('#') || trimmed.starts_with(';') {
            return None;
        }
        let (key, value) = trimmed.split_once('=')?;
        key.trim()
            .eq_ignore_ascii_case("save_file_extension")
            .then(|| {
                value
                    .trim()
                    .trim_matches('"')
                    .trim_start_matches('.')
                    .to_string()
            })
    })?;
    if extension.is_empty()
        || !extension
            .chars()
            .all(|character| character.is_ascii_alphanumeric())
    {
        return None;
    }
    Some(format!("NR0000.{extension}"))
}

fn backup_effective_save(
    plan: &GeneratedProfilePlan,
    environment: RuntimeEnvironment,
    game_dir: &Path,
) -> Result<Option<PathBuf>, String> {
    let savefile = effective_save_filename(plan, environment, game_dir)?;
    let Some(app_data_dir) = dirs::config_dir() else {
        return Err("无法定位 Windows AppData，不能安全备份存档".to_string());
    };
    let saves_root = app_data_dir.join("Nightreign");
    if !saves_root.exists() {
        return Ok(None);
    }
    let account_dirs = fs::read_dir(&saves_root).map_err(|error| {
        format!(
            "读取 Nightreign 存档目录失败：{}，{error}",
            saves_root.to_string_lossy()
        )
    })?;
    let backup_dir = get_config_dir()
        .join("backups")
        .join("saves")
        .join(current_timestamp().to_string());
    let mut copied = false;
    for entry in account_dirs.filter_map(Result::ok) {
        let account_dir = entry.path();
        if !account_dir.is_dir() {
            continue;
        }
        let account_name = entry.file_name();
        for candidate in [&savefile, &format!("{savefile}.bak")] {
            let source = account_dir.join(candidate);
            if !source.is_file() {
                continue;
            }
            let target = backup_dir.join(&account_name).join(candidate);
            if let Some(parent) = target.parent() {
                fs::create_dir_all(parent)
                    .map_err(|error| format!("创建存档备份目录失败：{error}"))?;
            }
            fs::copy(&source, &target).map_err(|error| {
                format!(
                    "备份存档失败，已阻止启动：{} -> {}，{error}",
                    source.to_string_lossy(),
                    target.to_string_lossy()
                )
            })?;
            let source_hash = sha256_file(&source)?;
            let target_hash = sha256_file(&target)?;
            if source_hash != target_hash {
                let _ = fs::remove_file(&target);
                return Err(format!(
                    "存档备份回读校验失败，已阻止启动：{} -> {}",
                    source.to_string_lossy(),
                    target.to_string_lossy()
                ));
            }
            copied = true;
        }
    }
    Ok(copied.then_some(backup_dir))
}

fn load_gameplay_launch_record() -> Option<GameplayLaunchRecord> {
    let content = fs::read_to_string(get_gameplay_launch_record_path()).ok()?;
    serde_json::from_str(&content).ok()
}

fn regulation_sha256(plan: &GeneratedProfilePlan) -> Result<Option<String>, String> {
    match plan.regulation_files.as_slice() {
        [path] => sha256_file(path).map(Some),
        _ => Ok(None),
    }
}

fn save_gameplay_launch_record(
    plan: &GeneratedProfilePlan,
    environment: RuntimeEnvironment,
    game_dir: &Path,
) -> Result<(), String> {
    let record = GameplayLaunchRecord {
        savefile: effective_save_filename(plan, environment, game_dir)?,
        regulation_sha256: regulation_sha256(plan)?,
        runtime_environment: environment.as_str().to_string(),
        selected_mod_count: plan.selected_mod_count,
        recorded_at: current_timestamp().to_string(),
    };
    let content = serde_json::to_string_pretty(&record)
        .map_err(|error| format!("序列化玩法启动记录失败：{error}"))?;
    fs::write(get_gameplay_launch_record_path(), content)
        .map_err(|error| format!("写入玩法启动记录失败：{error}"))
}

fn count_existing_saves(savefile: &str) -> usize {
    let Some(app_data_dir) = dirs::config_dir() else {
        return 0;
    };
    let saves_root = app_data_dir.join("Nightreign");
    let Ok(account_dirs) = fs::read_dir(saves_root) else {
        return 0;
    };
    account_dirs
        .filter_map(Result::ok)
        .filter(|entry| entry.path().join(savefile).is_file())
        .count()
}

fn assess_gameplay_save_compatibility(
    savefile: &str,
    existing_save_count: usize,
    current_regulation_sha256: Option<&str>,
    previous: Option<&GameplayLaunchRecord>,
) -> (&'static str, String) {
    if existing_save_count == 0 {
        return (
            "pass",
            format!("没有找到现有 {savefile}；首次创建角色时会建立与当前玩法参数一致的新存档。"),
        );
    }

    match (current_regulation_sha256, previous) {
        (Some(current), Some(previous))
            if previous.savefile.eq_ignore_ascii_case(savefile)
                && previous.regulation_sha256.as_deref() == Some(current) =>
        {
            (
                "pass",
                format!(
                    "找到 {existing_save_count} 份 {savefile}；当前玩法数据文件（regulation.bin）与上次管理器启动记录一致（SHA-256={current}）。启动前仍会执行哈希回读备份。"
                ),
            )
        }
        (Some(current), Some(previous)) if previous.savefile.eq_ignore_ascii_case(savefile) => (
            "warning",
            format!(
                "找到 {existing_save_count} 份 {savefile}，但当前玩法数据文件（regulation.bin，SHA-256={current}）与上次管理器启动记录（{}）不同。旧存档可能保留已变化的武器、装备或服装 ID，表现为人物或武器不显示。请改回匹配方案，或在保留备份后用新角色验证；启动配置中的 savefile 不能隔离 Seamless 存档。",
                previous.regulation_sha256.as_deref().unwrap_or("无玩法数据文件")
            ),
        ),
        (Some(current), _) => (
            "warning",
            format!(
                "找到 {existing_save_count} 份来源未知的 {savefile}；当前玩法数据文件（regulation.bin）SHA-256={current}。旧存档若来自另一套地图、武器或服装参数，可能出现人物或装备不显示。启动前会哈希回读备份；建议先用新角色验证。"
            ),
        ),
        (None, Some(previous))
            if previous.savefile.eq_ignore_ascii_case(savefile)
                && previous.regulation_sha256.is_some() =>
        {
            (
                "warning",
                format!(
                    "找到 {existing_save_count} 份 {savefile}，但当前方案没有玩法数据文件（regulation.bin），上次管理器启动记录包含自定义玩法参数。移除地图、武器或扩展服装后继续旧存档可能导致人物或装备不显示。"
                ),
            )
        }
        (None, _) => (
            "pass",
            format!(
                "找到 {existing_save_count} 份 {savefile}；当前方案没有自定义玩法数据文件（regulation.bin）。启动前会执行哈希回读备份。"
            ),
        ),
    }
}

#[command]
pub fn get_launch_artifacts() -> Result<LaunchArtifacts, String> {
    let profile_path = get_generated_profile_path();
    let script_path = get_launch_script_path();
    let log_path = get_launch_log_path();

    Ok(LaunchArtifacts {
        profile_content: read_optional_text(&profile_path)?,
        profile_path: profile_path.to_string_lossy().to_string(),
        script_content: read_optional_text(&script_path)?,
        script_path: script_path.to_string_lossy().to_string(),
        log_content: read_text_tail(&log_path, MAX_LAUNCH_LOG_READ_BYTES)?,
        log_path: log_path.to_string_lossy().to_string(),
    })
}

#[command]
pub fn get_special_mod_status() -> Result<SpecialModStatus, String> {
    let config = load_config();
    if config.game_path.trim().is_empty() {
        return Err("请先设置游戏目录".to_string());
    }

    let game_dir = Path::new(&config.game_path);
    Ok(build_special_mod_status(game_dir))
}

#[command]
pub async fn get_launch_preflight() -> Result<LaunchPreflight, String> {
    tauri::async_runtime::spawn_blocking(build_launch_preflight)
        .await
        .map_err(|error| format!("启动前检查异常结束：{error}"))?
}

fn build_launch_preflight() -> Result<LaunchPreflight, String> {
    let config = load_config();
    let mut checks = Vec::new();
    let game_dir = Path::new(&config.game_path);
    let game_valid = !config.game_path.trim().is_empty() && validate_game_dir(game_dir).is_ok();
    let profile_plan = build_generated_profile_plan();
    let runtime_status = build_runtime_environment_status(&config);
    let runtime_environment = runtime_environment_from_str(&runtime_status.effective)
        .unwrap_or(RuntimeEnvironment::UnknownMixed);
    let using_server_redirector = profile_plan
        .as_ref()
        .is_ok_and(|plan| plan.network_backend == NetworkBackend::ServerRedirector);

    if game_valid {
        add_preflight_check(
            &mut checks,
            "game_path",
            "游戏目录",
            "pass",
            format!(
                "已找到 {}",
                game_dir.join("nightreign.exe").to_string_lossy()
            ),
        );
    } else {
        add_preflight_check(
            &mut checks,
            "game_path",
            "游戏目录",
            "error",
            "目录未配置，或没有找到 nightreign.exe".to_string(),
        );
    }

    match find_me3_exe(Path::new(&config.me3_path)) {
        Ok(me3_exe) => {
            let version = read_me3_version(&me3_exe);
            let outdated = version.is_some_and(|value| value < RECOMMENDED_ME3_VERSION);
            add_preflight_check(
                &mut checks,
                "me3",
                "ME3 引擎",
                if outdated || version.is_none() {
                    "warning"
                } else {
                    "pass"
                },
                match version {
                    Some(value) if outdated => format!(
                        "检测到 ME3 {}.{}.{}；建议升级到 0.12.1 或更高。当前 Spacewar 实测链路仍允许继续。",
                        value.0, value.1, value.2
                    ),
                    Some(value) => format!(
                        "已找到 {}（ME3 {}.{}.{}）",
                        me3_exe.to_string_lossy(),
                        value.0,
                        value.1,
                        value.2
                    ),
                    None => format!(
                        "已找到 {}，但无法读取版本；允许继续，建议确认不低于 0.12.1。",
                        me3_exe.to_string_lossy()
                    ),
                },
            );
        }
        Err(error) => add_preflight_check(&mut checks, "me3", "ME3 引擎", "error", error),
    }

    if game_valid {
        match resolve_launch_exe(&config, game_dir) {
            Ok(launch_exe) => add_preflight_check(
                &mut checks,
                "launch_target",
                "启动目标",
                "pass",
                launch_exe.to_string_lossy().to_string(),
            ),
            Err(error) => {
                add_preflight_check(&mut checks, "launch_target", "启动目标", "error", error)
            }
        }
    }

    let tasklist_processes = read_tasklist_processes();
    let steam_running = tasklist_processes
        .iter()
        .any(|process| process.name.eq_ignore_ascii_case("steam.exe"));
    if steam_running {
        add_preflight_check(
            &mut checks,
            "steam",
            "Steam 状态",
            "pass",
            if using_server_redirector {
                "已检测到 Steam；MMV Server Redirector 将使用正版 Steam 身份与邀请。".to_string()
            } else {
                "已检测到 Steam。联机补丁仍应与 Steam 保持相同权限级别。".to_string()
            },
        );
    } else {
        add_preflight_check(
            &mut checks,
            "steam",
            "Steam 状态",
            "error",
            "当前支持的三种运行环境都依赖 Steam/Spacewar 身份；请先启动 Steam，并保持相同权限级别。".to_string(),
        );
    }

    let runtime_validation = if game_valid {
        profile_plan
            .as_ref()
            .map_err(Clone::clone)
            .and_then(|plan| validate_runtime_launch_environment(plan, &config, game_dir))
    } else {
        Err("请先配置有效游戏目录".to_string())
    };
    add_preflight_check(
        &mut checks,
        "runtime_environment",
        "运行环境",
        if runtime_validation.is_ok() {
            if runtime_status.verified {
                "pass"
            } else {
                "warning"
            }
        } else {
            "error"
        },
        match runtime_validation {
            Ok(()) => format!(
                "{}；检测={}，配置={}{}",
                runtime_environment_label(runtime_environment),
                runtime_status.detected,
                runtime_status.configured,
                if runtime_status.verified {
                    "；当前用户已实测"
                } else {
                    "；仅完成保守逻辑与模拟测试"
                }
            ),
            Err(error) => format!(
                "{}；检测={}，配置={}。{}",
                runtime_environment_label(runtime_environment),
                runtime_status.detected,
                runtime_status.configured,
                error
            ),
        },
    );

    let running = format_guarded_processes(&tasklist_processes);
    if running.is_empty() {
        add_preflight_check(
            &mut checks,
            "running_processes",
            "重复启动保护",
            "pass",
            "没有残留的游戏或 ME3 注入进程".to_string(),
        );
    } else {
        add_preflight_check(
            &mut checks,
            "running_processes",
            "重复启动保护",
            "error",
            format!("请先关闭：{}", running.join(", ")),
        );
    }

    match &profile_plan {
        Ok(plan) => {
            add_preflight_check(
                &mut checks,
                "network_backend",
                "方案联机后端",
                "pass",
                match plan.network_backend {
                    NetworkBackend::ServerRedirector => {
                        "Server Redirector；已禁止自动注入游戏目录中的 nrsc.dll 与 nighter.dll"
                            .to_string()
                    }
                    NetworkBackend::Seamless => {
                        "SeamlessCoop；nrsc.dll 将 early load，nighter 按检测结果加载".to_string()
                    }
                    NetworkBackend::None => "未加载联机功能插件；适用于离线 Mod 方案".to_string(),
                },
            );

            match effective_save_filename(plan, runtime_environment, game_dir) {
                Ok(savefile) => {
                    let seamless_runtime = plan.network_backend == NetworkBackend::Seamless
                        || matches!(
                            runtime_environment,
                            RuntimeEnvironment::SteamSeamless
                                | RuntimeEnvironment::SpacewarSeamless
                        );
                    let ignored_profile_savefile = seamless_runtime
                        && plan
                            .savefile
                            .as_deref()
                            .is_some_and(|declared| !declared.eq_ignore_ascii_case(&savefile));
                    add_preflight_check(
                        &mut checks,
                        "savefile",
                        "存档隔离",
                        if (plan.network_backend == NetworkBackend::ServerRedirector
                            && savefile.eq_ignore_ascii_case("NR0000.co2"))
                            || ignored_profile_savefile
                        {
                            "warning"
                        } else {
                            "pass"
                        },
                        if ignored_profile_savefile {
                            format!(
                                "作者启动配置声明了 {}，但 Seamless 实际存档由 nrsc_settings.ini 决定；本次按真实文件 {savefile} 备份。不要把 Profile 的 savefile 当作 Spacewar/Seamless 存档隔离。",
                                plan.savefile.as_deref().unwrap_or_default()
                            )
                        } else if plan.network_backend == NetworkBackend::ServerRedirector
                            && savefile.eq_ignore_ascii_case("NR0000.co2")
                        {
                            "当前作者启动配置使用 NR0000.co2，与 Seamless 默认存档同名；它不是独立存档。每次启动前会自动备份，但切换方案前仍应确认存档用途。".to_string()
                        } else if seamless_runtime {
                            format!(
                                "Seamless 实际使用 {savefile}（由 nrsc_settings.ini 决定）；启动配置（Profile）的 savefile 不提供额外隔离。启动前会执行哈希回读备份。"
                            )
                        } else if runtime_environment == RuntimeEnvironment::SteamOfficial
                            && savefile.eq_ignore_ascii_case(OFFICIAL_MOD_SAVEFILE)
                        {
                            format!(
                                "普通正版 Mod 方案强制使用 {savefile}，避免写入官方 NR0000.sl2；启动前会备份已有文件。"
                            )
                        } else {
                            format!("本次使用 {savefile}；如已存在，启动前会自动备份。")
                        },
                    );

                    if seamless_runtime {
                        let current_regulation = regulation_sha256(plan).ok().flatten();
                        let previous = load_gameplay_launch_record();
                        let (status, message) = assess_gameplay_save_compatibility(
                            &savefile,
                            count_existing_saves(&savefile),
                            current_regulation.as_deref(),
                            previous.as_ref(),
                        );
                        add_preflight_check(
                            &mut checks,
                            "save_gameplay_compatibility",
                            "存档与玩法参数",
                            status,
                            message,
                        );
                    }
                }
                Err(error) => {
                    add_preflight_check(&mut checks, "savefile", "存档隔离", "error", error)
                }
            }

            if plan.author_profile_sources.is_empty() {
                add_preflight_check(
                    &mut checks,
                    "author_profile",
                    "作者启动配置",
                    "pass",
                    "当前方案没有需要继承根字段的作者 .me3".to_string(),
                );
            } else {
                let savefile = plan.savefile.as_deref().unwrap_or("未声明");
                let start_online = plan
                    .start_online
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| "未声明".to_string());
                add_preflight_check(
                    &mut checks,
                    "author_profile",
                    "作者启动配置",
                    "pass",
                    format!(
                        "已保留 {} 份作者配置；savefile={savefile}，start_online={start_online}",
                        plan.author_profile_sources.len()
                    ),
                );
            }

            let (regulation_status, regulation_message) = match plan.regulation_files.as_slice() {
                [path] => match sha256_file(path) {
                    Ok(hash) => (
                        "pass",
                        format!(
                            "当前只有一份玩法数据文件，来自“{}”；高级校验：regulation.bin SHA-256={hash}",
                            regulation_owner_label(path)
                        ),
                    ),
                    Err(error) => ("error", error),
                },
                [] if plan.mmv_seamless_community_count > 0 => (
                    "error",
                    "当前整合方案缺少玩法数据文件；地图、敌人、武器或扩展服装参数不会完整生效。"
                        .to_string(),
                ),
                [] => (
                    "pass",
                    "当前方案没有自定义玩法数据文件；纯外观替换通常不需要它。".to_string(),
                ),
                paths => (
                    "error",
                    format!(
                        "检测到 {} 份玩法数据文件，分别来自：{}。加载顺序只能决定最后覆盖哪一份，不能自动合并；请停用其中一个，或使用已经合并两者的兼容版本。",
                        paths.len(),
                        regulation_owner_labels(paths).join("、")
                    ),
                ),
            };
            add_preflight_check(
                &mut checks,
                "regulation_owner",
                "玩法参数是否冲突",
                regulation_status,
                regulation_message,
            );

            if plan.mmv_seamless_community_count > 0 {
                add_preflight_check(
                    &mut checks,
                    "mmv_seamless_community",
                    "MMV Seamless 兼容",
                    "warning",
                    format!(
                        "已对 {} 个外部 MMV 作者启动配置启用社区兼容模式：原文件保持只读，仅在生成副本中移除 cl_server_redirector.dll，并改由游戏目录中的 nrsc.dll early load。此路线有社区与 Spacewar 实例支持，但不受 MMV 作者团队官方支持。",
                        plan.mmv_seamless_community_count
                    ),
                );

                let (zhocn_status, zhocn_message) = match plan.zhocn_packages.as_slice() {
                    [package] => {
                        let root = package.join("msg").join("zhocn");
                        let item = root.join("item_dlc01.msgbnd.dcx");
                        let menu = root.join("menu_dlc01.msgbnd.dcx");
                        match (sha256_file(&item), sha256_file(&menu)) {
                            (Ok(item_hash), Ok(menu_hash)) => (
                                "pass",
                                format!(
                                    "检测到单一简中覆盖层：{}；item SHA-256={item_hash}；menu SHA-256={menu_hash}",
                                    package.to_string_lossy()
                                ),
                            ),
                            (Err(error), _) | (_, Err(error)) => ("error", error),
                        }
                    }
                    [] => (
                        "warning",
                        "尚未检测到完整的 msg\\zhocn\\item_dlc01.msgbnd.dcx 与 menu_dlc01.msgbnd.dcx。玩法可以测试，但新增敌人、武器和说明可能显示英文或占位符；请安装 602 于 2026-07-22 上传的 314 KB 主文件，其 Changelog 标注同步 2.1.7.1。"
                            .to_string(),
                    ),
                    packages => (
                        "error",
                        format!(
                            "检测到 {} 个完整简中覆盖层。602 与 559 会覆盖同一批文本，请只启用一份当前版本翻译。",
                            packages.len()
                        ),
                    ),
                };
                add_preflight_check(
                    &mut checks,
                    "zhocn_layer",
                    "简体中文覆盖",
                    zhocn_status,
                    zhocn_message,
                );
            }

            if plan.network_backend == NetworkBackend::ServerRedirector {
                let profiles_outside_game = plan.author_profile_sources.iter().all(|path| {
                    path.parent()
                        .is_some_and(|parent| !is_path_inside_dir(parent, game_dir))
                });
                add_preflight_check(
                    &mut checks,
                    "external_profile_location",
                    "MMV 外部位置",
                    if profiles_outside_game {
                        "pass"
                    } else {
                        "error"
                    },
                    if profiles_outside_game {
                        "作者启动配置位于实际 Game 目录之外，管理器只读使用原目录".to_string()
                    } else {
                        "Server Redirector 整合包位于实际 Game 目录内；请移动到 Game 目录之外后重新注册"
                            .to_string()
                    },
                );
            }
        }
        Err(error) => add_preflight_check(
            &mut checks,
            "profile_generation",
            "启动配置兼容性",
            "error",
            error.clone(),
        ),
    }

    if game_valid {
        let status = build_special_mod_status(game_dir);
        let redirector_environment_conflict =
            using_server_redirector && !status.server_redirector_conflicts.is_empty();
        let seamless_required = matches!(
            runtime_environment,
            RuntimeEnvironment::SteamSeamless | RuntimeEnvironment::SpacewarSeamless
        );
        add_preflight_check(
            &mut checks,
            "seamless",
            "SeamlessCoop",
            if using_server_redirector
                || (seamless_required && status.seamless_installed)
                || (!seamless_required && !status.seamless_installed)
            {
                "pass"
            } else {
                "error"
            },
            if using_server_redirector {
                if status.seamless_installed {
                    "已安装，但当前 MMV 方案明确不把 nrsc.dll 加入启动配置。".to_string()
                } else {
                    "当前 MMV 方案使用 Server Redirector，不需要 SeamlessCoop。".to_string()
                }
            } else if seamless_required && status.seamless_installed {
                "nrsc.dll 与设置文件齐全；生成启动配置时会将 nrsc.dll 设为 early load。".to_string()
            } else if seamless_required {
                "当前环境要求 SeamlessCoop，但 nrsc.dll 或设置文件不完整。".to_string()
            } else if status.seamless_installed {
                "当前选择纯正版 Steam，但目录中检测到 SeamlessCoop；请切换环境或使用干净目录。"
                    .to_string()
            } else {
                "当前环境不使用 SeamlessCoop。".to_string()
            },
        );
        add_preflight_check(
            &mut checks,
            "onlinefix",
            "OnlineFix / Spacewar",
            if redirector_environment_conflict
                || (matches!(
                    runtime_environment,
                    RuntimeEnvironment::SteamOfficial | RuntimeEnvironment::SteamSeamless
                ) && !status.server_redirector_conflicts.is_empty())
            {
                "error"
            } else if using_server_redirector
                || (runtime_environment == RuntimeEnvironment::SpacewarSeamless
                    && status.onlinefix_installed)
                || (runtime_environment != RuntimeEnvironment::SpacewarSeamless
                    && status.server_redirector_conflicts.is_empty())
            {
                "pass"
            } else {
                "error"
            },
            if redirector_environment_conflict {
                format!(
                    "MMV Server Redirector 只支持正版 Steam 联机环境，但当前 Game 目录检测到冲突文件：{}。为避免登录失败、黑屏或异常存档，已阻止启动；请在设置中选择干净的 Steam 正版 Game 目录。管理器不会删除现有补丁文件。",
                    status.server_redirector_conflicts.join(", ")
                )
            } else if using_server_redirector {
                "未检测到 OnlineFix；当前 MMV 方案由 Server Redirector 提供隔离联机。".to_string()
            } else if runtime_environment == RuntimeEnvironment::SpacewarSeamless
                && status.onlinefix_installed
            {
                "OnlineFix 核心文件齐全。".to_string()
            } else if matches!(
                runtime_environment,
                RuntimeEnvironment::SteamOfficial | RuntimeEnvironment::SteamSeamless
            ) && status.server_redirector_conflicts.is_empty()
            {
                "未检测到 OnlineFix/Spacewar 代理文件，符合当前正版环境。".to_string()
            } else {
                format!(
                    "联机补丁文件不完整：{}",
                    if status.missing_game_files.is_empty() {
                        "请重新选择补丁 Game 文件夹".to_string()
                    } else {
                        status.missing_game_files.join(", ")
                    }
                )
            },
        );
        add_preflight_check(
            &mut checks,
            "nighter",
            "深夜解锁",
            if using_server_redirector || status.nighter_available {
                "pass"
            } else {
                "warning"
            },
            if using_server_redirector {
                if status.nighter_available {
                    "已检测到 nighter.dll，但当前 MMV 方案不会注入它。".to_string()
                } else {
                    "当前 MMV 方案不加载 nighter.dll。".to_string()
                }
            } else if status.nighter_available {
                format!("已检测到 {}", status.nighter_path)
            } else {
                "未检测到 nighter.dll；不使用深夜解锁时可忽略。".to_string()
            },
        );

        match collect_mods() {
            Ok(mods) => {
                let enabled = mods.iter().filter(|item| item.enabled).count();
                let packages = mods
                    .iter()
                    .filter(|item| item.enabled && item.mod_type == "package")
                    .count();
                let natives = mods
                    .iter()
                    .filter(|item| item.enabled && item.mod_type == "native")
                    .count();
                add_preflight_check(
                    &mut checks,
                    "enabled_mods",
                    "当前 Mod",
                    if enabled > 0 { "pass" } else { "warning" },
                    if enabled > 0 {
                        if let Ok(plan) = &profile_plan {
                            format!(
                                "将加载 {} 个 Mod：{} 个资源型 Mod，{} 个功能插件",
                                plan.selected_mod_count, plan.package_count, plan.native_count
                            )
                        } else {
                            format!(
                                "将加载 {enabled} 个 Mod：{packages} 个资源型 Mod，{natives} 个功能插件"
                            )
                        }
                    } else {
                        "没有启用的 Mod；启动后只会尝试加载游戏根目录中的联机功能插件。".to_string()
                    },
                );

                let enabled_clothing = mods
                    .iter()
                    .filter(|item| item.enabled && item.clothing.detected)
                    .collect::<Vec<_>>();
                if !enabled_clothing.is_empty() {
                    let expanded_count = enabled_clothing
                        .iter()
                        .filter(|item| item.clothing.requires_appearance_reset)
                        .count();
                    let online_view_risks = enabled_clothing
                        .iter()
                        .filter(|item| {
                            matches!(item.clothing.online_support.as_str(), "missing" | "partial")
                        })
                        .map(|item| item.name.as_str())
                        .collect::<Vec<_>>();
                    let mut message = format!(
                        "检测到 {} 个服装 Mod；其中 {expanded_count} 个包含扩展服装数据。",
                        enabled_clothing.len()
                    );
                    if expanded_count > 0 {
                        message.push_str(
                            " 停用或切换方案前，请先在游戏内换回本体服装，避免存档留下无效外观 ID。",
                        );
                    }
                    if !online_view_risks.is_empty() {
                        message.push_str(&format!(
                            " 以下 Mod 的 _l 队友视角资源缺失或不完整：{}。",
                            online_view_risks.join("、")
                        ));
                    } else {
                        message.push_str(" 已找到成对的 _l 队友视角资源。");
                    }
                    add_preflight_check(
                        &mut checks,
                        "clothing_resources",
                        "服装与队友视角",
                        if online_view_risks.is_empty() {
                            "pass"
                        } else {
                            "warning"
                        },
                        message,
                    );
                }
            }
            Err(error) => add_preflight_check(
                &mut checks,
                "enabled_mods",
                "当前 Mod",
                "warning",
                format!("扫描失败：{error}"),
            ),
        }
    }

    Ok(LaunchPreflight {
        ready: !checks.iter().any(|check| check.status == "error"),
        checks,
    })
}

fn add_preflight_check(
    checks: &mut Vec<LaunchPreflightCheck>,
    id: &str,
    label: &str,
    status: &str,
    message: String,
) {
    checks.push(LaunchPreflightCheck {
        id: id.to_string(),
        label: label.to_string(),
        status: status.to_string(),
        message,
    });
}

#[command]
pub fn install_seamless_onlinefix(patch_game_path: String) -> Result<SpecialModStatus, String> {
    let config = load_config();
    if config.game_path.trim().is_empty() {
        return Err("请先设置游戏目录".to_string());
    }

    let game_dir = Path::new(&config.game_path);
    validate_game_dir(game_dir)?;
    if steam_manifest_for_game_dir(game_dir).is_some() {
        return Err(
            "当前目录属于 Steam 正版安装。为防止覆盖 Steam 运行文件（steam_api64.dll），管理器禁止向该目录安装 OnlineFix/Spacewar 补丁。"
                .to_string(),
        );
    }
    if effective_runtime_environment(&config) != RuntimeEnvironment::SpacewarSeamless {
        return Err(
            "只有明确选择或自动识别为“Spacewar + Seamless”时才允许安装该联机补丁。正版环境不适用。"
                .to_string(),
        );
    }

    let patch_source = Path::new(patch_game_path.trim());
    validate_patch_source(patch_source)?;

    let backup_dir = create_patch_backup(game_dir)?;
    if let Err(copy_error) = copy_patch_tree(patch_source, game_dir) {
        return match restore_patch_backup(&backup_dir, game_dir) {
            Ok(()) => Err(format!(
                "安装联机补丁失败，已恢复安装前文件。原始错误：{copy_error}"
            )),
            Err(restore_error) => Err(format!(
                "安装联机补丁失败，自动恢复也失败。请勿启动游戏；备份位于 {}。安装错误：{copy_error}；恢复错误：{restore_error}",
                backup_dir.to_string_lossy()
            )),
        };
    }
    Ok(build_special_mod_status(game_dir))
}

#[command]
pub fn restore_latest_online_patch_backup() -> Result<SpecialModStatus, String> {
    let config = load_config();
    if config.game_path.trim().is_empty() {
        return Err("请先设置游戏目录".to_string());
    }
    let game_dir = Path::new(&config.game_path);
    validate_game_dir(game_dir)?;
    let backup_dir =
        latest_patch_backup_dir().ok_or_else(|| "没有可恢复的联机补丁备份".to_string())?;
    restore_patch_backup(&backup_dir, game_dir)?;
    Ok(build_special_mod_status(game_dir))
}

#[command]
pub async fn detect_file_conflicts() -> Result<Vec<FileConflict>, String> {
    tauri::async_runtime::spawn_blocking(detect_file_conflicts_blocking)
        .await
        .map_err(|error| format!("冲突分析任务异常结束：{error}"))?
}

fn detect_file_conflicts_blocking() -> Result<Vec<FileConflict>, String> {
    let mods = collect_mods()?;
    let enabled_mods = mods
        .iter()
        .filter(|mod_info| mod_info.enabled)
        .collect::<Vec<_>>();
    let mut first_owner_by_path: HashMap<String, usize> = HashMap::new();
    let mut conflict_owners_by_path: BTreeMap<String, BTreeSet<usize>> = BTreeMap::new();
    let mut scanned_files = 0usize;

    for (mod_index, mod_info) in enabled_mods.iter().enumerate() {
        let mod_dir = Path::new(&mod_info.path);
        let (packages, natives) = collect_entries_for_mod(mod_dir)?;

        for package in packages {
            collect_conflict_package_files(
                &package.path,
                mod_index,
                &mut first_owner_by_path,
                &mut conflict_owners_by_path,
                &mut scanned_files,
            )?;
        }

        for native in natives {
            if let Some(file_name) = native.path.file_name().and_then(|name| name.to_str()) {
                record_conflict_candidate(
                    format!("native/{file_name}").to_lowercase(),
                    mod_index,
                    &mut first_owner_by_path,
                    &mut conflict_owners_by_path,
                    &mut scanned_files,
                )?;
            }
        }
    }

    Ok(conflict_owners_by_path
        .into_iter()
        .map(|(relative_path, owner_indexes)| FileConflict {
            relative_path,
            owners: owner_indexes
                .into_iter()
                .map(|owner_index| {
                    let mod_info = enabled_mods[owner_index];
                    ConflictOwner {
                        mod_id: mod_info.id.clone(),
                        mod_name: mod_info.name.clone(),
                        source_path: mod_info.path.clone(),
                    }
                })
                .collect(),
        })
        .collect())
}

fn mods_selected_for_generation(mods: &[ModInfo]) -> Vec<&ModInfo> {
    if let Some(active_profile) = profile::get_active_profile() {
        let mut profile_mods: Vec<_> = active_profile
            .mods
            .iter()
            .filter(|item| item.enabled)
            .collect();

        if !profile_mods.is_empty() {
            profile_mods.sort_by_key(|item| item.load_order);
            let mut selected = profile_mods
                .into_iter()
                .filter_map(|profile_mod| mods.iter().find(|item| item.id == profile_mod.mod_id))
                .collect::<Vec<_>>();
            for mod_info in mods.iter().filter(|item| {
                item.enabled
                    && !active_profile
                        .mods
                        .iter()
                        .any(|profile_mod| profile_mod.mod_id == item.id)
            }) {
                selected.push(mod_info);
            }
            return selected;
        }
    }

    mods.iter().filter(|item| item.enabled).collect()
}

fn extend_unique_packages(
    target: &mut Vec<PackageEntry>,
    seen: &mut BTreeSet<String>,
    entries: Vec<PackageEntry>,
) {
    for entry in entries {
        if seen.insert(path_key(&entry.path)) {
            target.push(entry);
        }
    }
}

fn extend_unique_natives(
    target: &mut Vec<NativeEntry>,
    seen: &mut BTreeSet<String>,
    entries: Vec<NativeEntry>,
) {
    for entry in entries {
        if seen.insert(path_key(&entry.path)) {
            target.push(entry);
        }
    }
}

#[command]
pub fn launch_game(game_path: String, me3_path: String) -> Result<String, String> {
    let config = load_config();
    let game_path = if game_path.trim().is_empty() {
        config.game_path.clone()
    } else {
        game_path
    };
    let me3_path = if me3_path.trim().is_empty() {
        config.me3_path.clone()
    } else {
        me3_path
    };

    if game_path.trim().is_empty() {
        return Err("请先设置游戏目录".to_string());
    }
    if me3_path.trim().is_empty() {
        return Err("请先设置ME3目录".to_string());
    }

    ensure_no_running_game_processes()?;

    let plan = build_generated_profile_plan()?;
    validate_single_regulation_owner(&plan)?;
    let runtime_environment = effective_runtime_environment(&config);
    validate_runtime_launch_environment(&plan, &config, Path::new(&game_path))?;
    let profile_path = write_generated_profile(&plan)?;
    let me3_exe = find_me3_exe(Path::new(&me3_path))?;
    let launch_exe = resolve_launch_exe(&config, Path::new(&game_path))?;
    let args = build_launch_args(
        &profile_path,
        &launch_exe,
        plan.network_backend,
        runtime_environment,
    );
    let working_dir = me3_exe.parent().unwrap_or_else(|| Path::new(&me3_path));
    let launch_script = write_launch_script(&me3_exe, &args, working_dir)?;
    let save_backup = backup_effective_save(&plan, runtime_environment, Path::new(&game_path))?;
    append_launch_log(&format!(
        "\n=== Launch {} ===\nscript: {}\nsave backup: {}\n{}\n",
        current_timestamp(),
        launch_script.to_string_lossy(),
        save_backup.as_ref().map_or_else(
            || "没有找到现有存档，无需备份".to_string(),
            |path| path.to_string_lossy().to_string()
        ),
        format_command_line(&me3_exe, &args, working_dir)
    ));

    launch_via_script(&launch_script)?;
    if let Err(error) =
        save_gameplay_launch_record(&plan, runtime_environment, Path::new(&game_path))
    {
        append_launch_log(&format!(
            "玩法启动记录写入失败（不影响本次启动）：{error}\n"
        ));
    }

    Ok(format!(
        "已在后台开始启动游戏。\n如未进入游戏，请在诊断页查看启动日志。\n日志：{}{}",
        get_launch_log_path().to_string_lossy(),
        save_backup.map_or_else(String::new, |path| format!(
            "\n存档备份：{}",
            path.to_string_lossy()
        ))
    ))
}

#[command]
pub fn diagnose_launch_game(game_path: String, me3_path: String) -> Result<String, String> {
    let config = load_config();
    let game_path = if game_path.trim().is_empty() {
        config.game_path.clone()
    } else {
        game_path
    };
    let me3_path = if me3_path.trim().is_empty() {
        config.me3_path.clone()
    } else {
        me3_path
    };

    if game_path.trim().is_empty() {
        return Err("请先设置游戏目录".to_string());
    }
    if me3_path.trim().is_empty() {
        return Err("请先设置ME3目录".to_string());
    }

    ensure_no_running_game_processes()?;

    let plan = build_generated_profile_plan()?;
    validate_single_regulation_owner(&plan)?;
    let runtime_environment = effective_runtime_environment(&config);
    validate_runtime_launch_environment(&plan, &config, Path::new(&game_path))?;
    let profile_path = write_generated_profile(&plan)?;
    let me3_exe = find_me3_exe(Path::new(&me3_path))?;
    let launch_exe = resolve_launch_exe(&config, Path::new(&game_path))?;
    let args = build_launch_args(
        &profile_path,
        &launch_exe,
        plan.network_backend,
        runtime_environment,
    );
    let working_dir = me3_exe.parent().unwrap_or_else(|| Path::new(&me3_path));
    let save_backup = backup_effective_save(&plan, runtime_environment, Path::new(&game_path))?;
    let command_line = format_command_line(&me3_exe, &args, working_dir);
    append_launch_log(&format!(
        "\n=== Diagnose {} ===\nsave backup: {}\n{}\n",
        current_timestamp(),
        save_backup.as_ref().map_or_else(
            || "没有找到现有存档，无需备份".to_string(),
            |path| path.to_string_lossy().to_string()
        ),
        command_line
    ));
    let log_path = get_launch_log_path();
    let log_file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
        .map_err(|e| format!("无法打开诊断日志：{e}"))?;
    let stdout_log = log_file
        .try_clone()
        .map_err(|e| format!("无法复制诊断日志句柄：{e}"))?;

    let mut child = std::process::Command::new(&me3_exe)
        .args(&args)
        .current_dir(working_dir)
        .stdout(Stdio::from(stdout_log))
        .stderr(Stdio::from(log_file))
        .spawn()
        .map_err(|e| format!("无法启动 ME3：{e}\n\n命令：{command_line}"))?;
    if let Err(error) =
        save_gameplay_launch_record(&plan, runtime_environment, Path::new(&game_path))
    {
        append_launch_log(&format!(
            "玩法启动记录写入失败（不影响本次诊断）：{error}\n"
        ));
    }

    let start = Instant::now();
    while start.elapsed() < Duration::from_secs(4) {
        if let Some(status) = child
            .try_wait()
            .map_err(|e| format!("检查 ME3 状态失败：{e}\n\n命令：{command_line}"))?
        {
            if status.success() {
                return Ok(format!("ME3 已完成启动流程。\n命令：{command_line}"));
            }

            let log_excerpt = read_text_tail(&log_path, 64 * 1024)
                .unwrap_or_else(|error| format!("无法读取诊断日志：{error}"));

            return Err(format!(
                "ME3 启动失败，退出码：{}\n\n命令：{}\n\n日志末尾：\n{}",
                status
                    .code()
                    .map_or_else(|| "unknown".to_string(), |code| code.to_string()),
                command_line,
                if log_excerpt.trim().is_empty() {
                    "(empty)"
                } else {
                    log_excerpt.trim()
                },
            ));
        }

        std::thread::sleep(Duration::from_millis(100));
    }

    Ok(format!("ME3 进程已启动。\n命令：{command_line}"))
}

fn write_launch_script(
    me3_exe: &Path,
    args: &[String],
    working_dir: &Path,
) -> Result<PathBuf, String> {
    let script_path = get_launch_script_path();
    let log_path = get_launch_log_path();
    let command = std::iter::once(quote_arg(&me3_exe.to_string_lossy()))
        .chain(args.iter().map(|arg| quote_arg(arg)))
        .collect::<Vec<_>>()
        .join(" ");
    let content = format!(
        "@echo off\r\nchcp 65001 >nul\r\ncd /d {}\r\necho === %date% %time% === >> {}\r\necho {} >> {}\r\n{} >> {} 2>>&1\r\necho exit_code=%%ERRORLEVEL%% >> {}\r\n",
        quote_arg(&working_dir.to_string_lossy()),
        quote_arg(&log_path.to_string_lossy()),
        command.replace('%', "%%"),
        quote_arg(&log_path.to_string_lossy()),
        command,
        quote_arg(&log_path.to_string_lossy()),
        quote_arg(&log_path.to_string_lossy())
    );

    fs::write(&script_path, content).map_err(|e| format!("创建启动脚本失败：{e}"))?;
    Ok(script_path)
}

fn launch_via_script(script_path: &Path) -> Result<(), String> {
    let mut command = std::process::Command::new("cmd");
    command
        // /C 在 ME3 启动流程结束后自动退出；输出已写入 last-launch.log，
        // 因此不再让玩家面对一个不会自动关闭的命令行窗口。
        .arg("/C")
        .arg(script_path)
        .current_dir(script_path.parent().unwrap_or_else(|| Path::new(".")));

    #[cfg(windows)]
    // CREATE_NO_WINDOW：后台运行 cmd/bat，避免启动时弹出黑色终端窗口。
    command.creation_flags(0x08000000);

    command
        .spawn()
        .map_err(|e| format!("执行启动脚本失败：{e}"))?;

    Ok(())
}

fn append_launch_log(message: &str) {
    let log_path = get_launch_log_path();
    rotate_launch_log_if_needed(&log_path);
    if let Ok(mut file) = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_path)
    {
        let _ = file.write_all(message.as_bytes());
    }
}

fn rotate_launch_log_if_needed(log_path: &Path) {
    let Ok(metadata) = fs::metadata(log_path) else {
        return;
    };
    if metadata.len() < MAX_LAUNCH_LOG_BYTES {
        return;
    }

    let rotated_path = log_path.with_extension("log.1");
    if rotated_path.exists() {
        let _ = fs::remove_file(&rotated_path);
    }
    let _ = fs::rename(log_path, rotated_path);
}

fn read_optional_text(path: &Path) -> Result<String, String> {
    if !path.exists() {
        return Ok(String::new());
    }

    fs::read_to_string(path).map_err(|e| format!("读取文件失败：{}，{}", path.to_string_lossy(), e))
}

fn read_text_tail(path: &Path, max_bytes: u64) -> Result<String, String> {
    if !path.exists() {
        return Ok(String::new());
    }

    let mut file =
        File::open(path).map_err(|e| format!("读取文件失败：{}，{e}", path.to_string_lossy()))?;
    let length = file
        .metadata()
        .map_err(|e| format!("读取文件元数据失败：{}，{e}", path.to_string_lossy()))?
        .len();
    let start = length.saturating_sub(max_bytes);
    file.seek(SeekFrom::Start(start))
        .map_err(|e| format!("定位文件失败：{}，{e}", path.to_string_lossy()))?;

    let mut bytes = Vec::with_capacity((length - start) as usize);
    file.read_to_end(&mut bytes)
        .map_err(|e| format!("读取文件失败：{}，{e}", path.to_string_lossy()))?;

    if start > 0 {
        if let Some(newline) = bytes.iter().position(|byte| *byte == b'\n') {
            bytes.drain(..=newline);
        }
    }

    let content = String::from_utf8_lossy(&bytes);
    if start > 0 {
        Ok(format!(
            "[日志已截断，仅显示最后 {} KB]\n{}",
            max_bytes / 1024,
            content
        ))
    } else {
        Ok(content.into_owned())
    }
}

fn validate_runtime_launch_environment(
    plan: &GeneratedProfilePlan,
    config: &AppConfig,
    game_dir: &Path,
) -> Result<(), String> {
    let environment = effective_runtime_environment(config);
    let status = build_special_mod_status(game_dir);
    let steam_running = read_tasklist_processes()
        .iter()
        .any(|process| process.name.eq_ignore_ascii_case("steam.exe"));
    if !steam_running {
        return Err(
            "当前运行环境依赖 Steam/Spacewar 身份，但未检测到 Steam。请先启动 Steam，再重新检查并启动。"
                .to_string(),
        );
    }

    if plan.network_backend == NetworkBackend::ServerRedirector {
        if environment != RuntimeEnvironment::SteamOfficial {
            return Err(
                "Server Redirector 只能在“纯正版 Steam”环境中启动；请切换到干净正版目录并在设置中选择对应环境。"
                .to_string(),
            );
        }
        if steam_manifest_for_game_dir(game_dir).is_none() {
            return Err(
            "MMV Server Redirector 需要可确认的 Steam 正版安装；当前 Game 目录没有匹配的 Steam 安装记录（AppManifest 2622380）。"
                    .to_string(),
            );
        }
        if !status.server_redirector_conflicts.is_empty() {
            return Err(format!(
                "当前方案使用 Server Redirector，但游戏目录中检测到会接管 Steam 或 DLL 加载的冲突文件：{}。请改选干净的 Steam 正版 Game 目录。",
                status.server_redirector_conflicts.join(", ")
            ));
        }
        return Ok(());
    }

    match environment {
        RuntimeEnvironment::SteamOfficial => {
            if steam_manifest_for_game_dir(game_dir).is_none() {
                return Err(
                    "未找到与当前 Game 目录匹配的 Steam 安装记录（AppManifest 2622380），不能确认这是正版安装目录。"
                        .to_string(),
                );
            }
            if !status.server_redirector_conflicts.is_empty() {
                return Err(format!(
                    "已选择纯正版 Steam，但目录中存在模拟层/代理文件：{}。",
                    status.server_redirector_conflicts.join(", ")
                ));
            }
            if plan.network_backend == NetworkBackend::Seamless {
                return Err(
                    "检测到 SeamlessCoop，但当前环境选择的是纯正版 Steam；请改选“正版 Steam + Seamless”。"
                        .to_string(),
                );
            }
            if plan.start_online == Some(true) {
                return Err(
                    "作者启动配置请求连接在线服务器，但当前是普通正版 Mod 方案。为保留 ME3 的官方匹配保护，已阻止启动。"
                        .to_string(),
                );
            }
            Ok(())
        }
        RuntimeEnvironment::SteamSeamless => {
            if steam_manifest_for_game_dir(game_dir).is_none() {
                return Err(
                    "未找到与当前 Game 目录匹配的 Steam 安装记录（AppManifest 2622380），不能确认这是正版 Seamless 安装目录。"
                        .to_string(),
                );
            }
            if !status.server_redirector_conflicts.is_empty() {
                return Err(format!(
                    "正版 Seamless 环境不能包含 OnlineFix/Spacewar 文件：{}。",
                    status.server_redirector_conflicts.join(", ")
                ));
            }
            if !status.seamless_installed || plan.network_backend != NetworkBackend::Seamless {
                return Err(
                    "已选择“正版 Steam + Seamless”，但没有检测到将被加载的 nrsc.dll。".to_string(),
                );
            }
            if plan.start_online == Some(true) {
                return Err(
                    "正版 Seamless 方案不应解除 ME3 的官方匹配保护；请移除请求 start_online=true 的冲突启动配置（Profile）。"
                        .to_string(),
                );
            }
            Ok(())
        }
        RuntimeEnvironment::SpacewarSeamless => {
            if !status.seamless_installed || !status.onlinefix_installed {
                return Err(
                    "Spacewar Seamless 环境缺少完整 SeamlessCoop 或 OnlineFix 文件。".to_string(),
                );
            }
            if plan.network_backend != NetworkBackend::Seamless {
                return Err(
                    "Spacewar Seamless 环境必须加载 nrsc.dll；当前生成方案没有 Seamless 后端。"
                        .to_string(),
                );
            }
            Ok(())
        }
        RuntimeEnvironment::Auto | RuntimeEnvironment::UnknownMixed => Err(
            "无法安全确认运行环境。请在设置中选择纯正版 Steam、正版 Steam + Seamless 或 Spacewar Seamless。"
                .to_string(),
        ),
    }
}

fn build_launch_args(
    profile_path: &Path,
    launch_exe: &Path,
    network_backend: NetworkBackend,
    runtime_environment: RuntimeEnvironment,
) -> Vec<String> {
    let mut args = vec![
        "launch".to_string(),
        "--exe".to_string(),
        launch_exe.to_string_lossy().to_string(),
    ];
    if runtime_environment == RuntimeEnvironment::SpacewarSeamless {
        args.push("--skip-steam-init".to_string());
    }
    if runtime_environment == RuntimeEnvironment::SpacewarSeamless
        || network_backend == NetworkBackend::ServerRedirector
    {
        args.push("--online".to_string());
    }
    args.extend([
        "--game".to_string(),
        "nightreign".to_string(),
        "-p".to_string(),
        profile_path.to_string_lossy().to_string(),
    ]);
    args
}

fn format_command_line(exe: &Path, args: &[String], working_dir: &Path) -> String {
    let mut parts = vec![format!(
        "cd /d {}",
        quote_arg(&working_dir.to_string_lossy())
    )];
    let command = std::iter::once(quote_arg(&exe.to_string_lossy()))
        .chain(args.iter().map(|arg| quote_arg(arg)))
        .collect::<Vec<_>>()
        .join(" ");
    parts.push(command);
    parts.join(" && ")
}

fn quote_arg(value: &str) -> String {
    if value.contains(' ') || value.contains('\\') || value.contains(':') {
        format!("\"{}\"", value.replace('"', "\\\""))
    } else {
        value.to_string()
    }
}

fn ensure_no_running_game_processes() -> Result<(), String> {
    let running = running_game_processes();
    if running.is_empty() {
        return Ok(());
    }

    Err(format!(
        "检测到仍在运行的游戏/注入进程：{}。\n如果刚执行过“启动游戏并诊断”，游戏已经启动，无需再次点击普通启动。否则请先关闭游戏窗口和 ME3 控制台后重试。",
        running.join(", ")
    ))
}

#[cfg(windows)]
fn running_game_processes() -> Vec<String> {
    format_guarded_processes(&read_tasklist_processes())
}

#[cfg(not(windows))]
fn running_game_processes() -> Vec<String> {
    Vec::new()
}

#[cfg(test)]
fn parse_tasklist_game_processes(stdout: &str) -> Vec<String> {
    format_guarded_processes(&parse_tasklist_processes(stdout))
}

#[cfg(windows)]
fn read_tasklist_processes() -> Vec<TasklistProcess> {
    let Ok(output) = std::process::Command::new("tasklist")
        .args(["/FO", "CSV", "/NH"])
        .output()
    else {
        return Vec::new();
    };

    parse_tasklist_processes(&String::from_utf8_lossy(&output.stdout))
}

#[cfg(not(windows))]
fn read_tasklist_processes() -> Vec<TasklistProcess> {
    Vec::new()
}

fn parse_tasklist_processes(stdout: &str) -> Vec<TasklistProcess> {
    stdout
        .lines()
        .filter_map(|line| {
            let mut fields = line.split(',');
            let name = fields.next()?.trim().trim_matches('"');
            let pid = fields.next()?.trim().trim_matches('"');
            (!name.is_empty()).then(|| TasklistProcess {
                name: name.to_string(),
                pid: pid.to_string(),
            })
        })
        .collect()
}

fn format_guarded_processes(processes: &[TasklistProcess]) -> Vec<String> {
    processes
        .iter()
        .filter(|process| {
            process.name.eq_ignore_ascii_case("nightreign.exe")
                || process.name.eq_ignore_ascii_case("me3-launcher.exe")
        })
        .map(|process| {
            if process
                .pid
                .chars()
                .all(|character| character.is_ascii_digit())
            {
                format!("{} (PID {})", process.name, process.pid)
            } else {
                process.name.clone()
            }
        })
        .collect()
}

fn resolve_launch_exe(config: &AppConfig, game_dir: &Path) -> Result<PathBuf, String> {
    let launch_exe = if config.launch_exe_path.trim().is_empty() {
        game_dir.join("nightreign.exe")
    } else {
        PathBuf::from(&config.launch_exe_path)
    };

    validate_launch_exe(&launch_exe, game_dir)?;

    if launch_exe
        .file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.eq_ignore_ascii_case("nrsc_launcher.exe"))
    {
        let game_exe = game_dir.join("nightreign.exe");
        validate_launch_exe(&game_exe, game_dir)?;
        append_launch_log(
            "检测到 nrsc_launcher.exe；ME3 启动链路将改用 nightreign.exe，并通过启动配置（Profile）加载 SeamlessCoop/nrsc.dll。\n",
        );
        return Ok(game_exe);
    }

    Ok(launch_exe)
}

fn validate_game_dir(game_dir: &Path) -> Result<(), String> {
    if !game_dir.join("nightreign.exe").exists() {
        return Err("游戏目录无效：未找到 nightreign.exe".to_string());
    }
    Ok(())
}

fn build_runtime_environment_status(config: &AppConfig) -> RuntimeEnvironmentStatus {
    let game_dir = Path::new(&config.game_path);
    let seamless = seamless_files_installed(game_dir);
    let onlinefix_markers = onlinefix_environment_markers(game_dir);
    let steam_manifest = steam_manifest_for_game_dir(game_dir);
    let strong_spacewar_marker = onlinefix_markers.iter().any(|file_name| {
        matches!(
            file_name.as_str(),
            "OnlineFix.ini" | "OnlineFix64.dll" | "steam_emu.ini"
        )
    });
    let (detected, confidence) = if strong_spacewar_marker {
        if seamless {
            (RuntimeEnvironment::SpacewarSeamless, "high")
        } else {
            (RuntimeEnvironment::UnknownMixed, "high")
        }
    } else if !onlinefix_markers.is_empty() {
        (RuntimeEnvironment::UnknownMixed, "medium")
    } else if steam_manifest.is_some() {
        if seamless {
            (RuntimeEnvironment::SteamSeamless, "high")
        } else {
            (RuntimeEnvironment::SteamOfficial, "high")
        }
    } else if seamless {
        (RuntimeEnvironment::UnknownMixed, "medium")
    } else {
        (RuntimeEnvironment::UnknownMixed, "low")
    };
    let effective = if config.runtime_environment == RuntimeEnvironment::Auto {
        detected
    } else {
        config.runtime_environment
    };

    let mut evidence = Vec::new();
    if seamless {
        evidence.push("检测到 SeamlessCoop/nrsc.dll 与设置文件".to_string());
    } else {
        evidence.push("未检测到完整 SeamlessCoop".to_string());
    }
    if !onlinefix_markers.is_empty() {
        evidence.push(format!(
            "检测到 Spacewar/模拟层标记：{}",
            onlinefix_markers.join(", ")
        ));
    }
    if let Some(manifest) = steam_manifest {
        evidence.push(format!(
            "游戏目录与 Steam 安装记录（AppManifest）匹配：{}",
            manifest.to_string_lossy()
        ));
    } else {
        evidence.push("未确认该目录属于 Steam 安装记录（AppManifest 2622380）".to_string());
    }

    let mut warnings = Vec::new();
    if detected == RuntimeEnvironment::UnknownMixed {
        warnings
            .push("自动检测无法安全确认环境；启动前必须在设置中明确选择并处理冲突。".to_string());
    }
    if config.runtime_environment != RuntimeEnvironment::Auto
        && detected != RuntimeEnvironment::UnknownMixed
        && config.runtime_environment != detected
    {
        warnings.push(format!(
            "手动选择的环境 {} 与文件检测结果 {} 不一致。",
            config.runtime_environment.as_str(),
            detected.as_str()
        ));
    }
    if effective != RuntimeEnvironment::SpacewarSeamless {
        warnings.push("该环境尚未由当前用户完成真实启动回归，将使用保守参数。".to_string());
    }

    RuntimeEnvironmentStatus {
        configured: config.runtime_environment.as_str().to_string(),
        detected: detected.as_str().to_string(),
        effective: effective.as_str().to_string(),
        verified: effective == RuntimeEnvironment::SpacewarSeamless,
        confidence: confidence.to_string(),
        evidence,
        warnings,
    }
}

fn effective_runtime_environment(config: &AppConfig) -> RuntimeEnvironment {
    let status = build_runtime_environment_status(config);
    runtime_environment_from_str(&status.effective).unwrap_or(RuntimeEnvironment::UnknownMixed)
}

fn runtime_environment_from_str(value: &str) -> Option<RuntimeEnvironment> {
    match value {
        "auto" => Some(RuntimeEnvironment::Auto),
        "steam_official" => Some(RuntimeEnvironment::SteamOfficial),
        "steam_seamless" => Some(RuntimeEnvironment::SteamSeamless),
        "spacewar_seamless" => Some(RuntimeEnvironment::SpacewarSeamless),
        "unknown_mixed" => Some(RuntimeEnvironment::UnknownMixed),
        _ => None,
    }
}

fn seamless_files_installed(game_dir: &Path) -> bool {
    game_dir.join("SeamlessCoop").join("nrsc.dll").exists()
        && game_dir
            .join("SeamlessCoop")
            .join("nrsc_settings.ini")
            .exists()
}

fn onlinefix_environment_markers(game_dir: &Path) -> Vec<String> {
    [
        "OnlineFix.ini",
        "OnlineFix64.dll",
        "steam_emu.ini",
        "dlllist.txt",
        "winmm.dll",
    ]
    .into_iter()
    .filter(|file_name| game_dir.join(file_name).exists())
    .map(ToOwned::to_owned)
    .collect()
}

fn steam_manifest_for_game_dir(game_dir: &Path) -> Option<PathBuf> {
    let install_dir = game_dir.parent()?;
    let common_dir = install_dir.parent()?;
    if !common_dir
        .file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.eq_ignore_ascii_case("common"))
    {
        return None;
    }
    let steamapps_dir = common_dir.parent()?;
    if !steamapps_dir
        .file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.eq_ignore_ascii_case("steamapps"))
    {
        return None;
    }

    let manifest = steamapps_dir.join("appmanifest_2622380.acf");
    let content = fs::read_to_string(&manifest).ok()?;
    let expected_install_dir = install_dir.file_name()?.to_string_lossy();
    let manifest_install_dir = content.lines().find_map(|line| {
        let trimmed = line.trim();
        if !trimmed.to_ascii_lowercase().starts_with("\"installdir\"") {
            return None;
        }
        trimmed.split('"').nth(3).map(str::to_string)
    })?;

    manifest_install_dir
        .eq_ignore_ascii_case(&expected_install_dir)
        .then_some(manifest)
}

#[allow(dead_code)]
fn infer_install_root(game_dir: &Path) -> PathBuf {
    game_dir
        .parent()
        .map_or_else(|| game_dir.to_path_buf(), Path::to_path_buf)
}

#[allow(dead_code)]
fn infer_patch_source_dir(game_dir: &Path) -> PathBuf {
    infer_install_root(game_dir).join("联机补丁").join("Game")
}

#[allow(dead_code)]
fn infer_sibling_mods_dir(game_dir: &Path) -> PathBuf {
    infer_install_root(game_dir).join("mods")
}

fn build_special_mod_status(game_dir: &Path) -> SpecialModStatus {
    let nighter_path = game_dir.join("mods").join("nighter.dll");
    let nighter_config_path = game_dir.join("mods").join("nighter.json");

    let missing_game_files = patch_required_files()
        .iter()
        .filter(|relative_path| !game_dir.join(relative_path).exists())
        .map(|path| normalize_relative_path(path))
        .collect::<Vec<_>>();

    SpecialModStatus {
        game_path: game_dir.to_string_lossy().to_string(),
        seamless_installed: seamless_files_installed(game_dir),
        onlinefix_installed: onlinefix_required_files()
            .iter()
            .all(|relative_path| game_dir.join(relative_path).exists()),
        server_redirector_conflicts: server_redirector_conflict_files(game_dir),
        nighter_available: nighter_path.exists(),
        nighter_path: nighter_path.to_string_lossy().to_string(),
        nighter_config_path: nighter_config_path.to_string_lossy().to_string(),
        missing_game_files,
        latest_patch_backup: latest_patch_backup_dir()
            .map_or_else(String::new, |path| path.to_string_lossy().to_string()),
    }
}

fn patch_required_files() -> Vec<PathBuf> {
    vec![
        PathBuf::from("SeamlessCoop").join("nrsc.dll"),
        PathBuf::from("SeamlessCoop").join("nrsc_settings.ini"),
        PathBuf::from("nrsc_launcher.exe"),
        PathBuf::from("OnlineFix.ini"),
        PathBuf::from("OnlineFix64.dll"),
        PathBuf::from("dlllist.txt"),
        PathBuf::from("winmm.dll"),
        PathBuf::from("steam_api64.dll"),
    ]
}

fn onlinefix_required_files() -> Vec<PathBuf> {
    vec![
        PathBuf::from("OnlineFix.ini"),
        PathBuf::from("OnlineFix64.dll"),
        PathBuf::from("dlllist.txt"),
        PathBuf::from("winmm.dll"),
    ]
}

fn server_redirector_conflict_files(game_dir: &Path) -> Vec<String> {
    onlinefix_environment_markers(game_dir)
}

fn patch_backup_root() -> PathBuf {
    get_config_dir().join("backups").join("online-patch")
}

fn latest_patch_backup_dir() -> Option<PathBuf> {
    let root = patch_backup_root();
    let mut candidates = fs::read_dir(root)
        .ok()?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.is_dir() && path.join("manifest.json").is_file())
        .collect::<Vec<_>>();
    candidates.sort();
    candidates.pop()
}

fn create_patch_backup(game_dir: &Path) -> Result<PathBuf, String> {
    create_patch_backup_in(game_dir, &patch_backup_root())
}

fn create_patch_backup_in(game_dir: &Path, backup_root: &Path) -> Result<PathBuf, String> {
    let backup_dir = backup_root.join(current_timestamp().to_string());
    let files_dir = backup_dir.join("files");
    fs::create_dir_all(&files_dir).map_err(|error| {
        format!(
            "创建联机补丁备份目录失败：{}，{error}",
            files_dir.to_string_lossy()
        )
    })?;

    let mut files = Vec::new();
    for relative_path in patch_required_files() {
        let source = game_dir.join(&relative_path);
        let existed = source.is_file();
        if existed {
            let backup_target = files_dir.join(&relative_path);
            if let Some(parent) = backup_target.parent() {
                fs::create_dir_all(parent).map_err(|error| {
                    format!(
                        "创建补丁备份子目录失败：{}，{error}",
                        parent.to_string_lossy()
                    )
                })?;
            }
            fs::copy(&source, &backup_target).map_err(|error| {
                format!(
                    "备份游戏原文件失败：{} -> {}，{error}",
                    source.to_string_lossy(),
                    backup_target.to_string_lossy()
                )
            })?;
        }
        files.push(PatchBackupFile {
            relative_path: normalize_relative_path(&relative_path),
            existed,
        });
    }

    let manifest = PatchBackupManifest {
        game_path: game_dir.to_string_lossy().to_string(),
        created_at: current_timestamp().to_string(),
        files,
    };
    let content = serde_json::to_string_pretty(&manifest)
        .map_err(|error| format!("生成补丁备份清单失败：{error}"))?;
    fs::write(backup_dir.join("manifest.json"), content)
        .map_err(|error| format!("写入补丁备份清单失败：{error}"))?;
    Ok(backup_dir)
}

fn restore_patch_backup(backup_dir: &Path, game_dir: &Path) -> Result<(), String> {
    let manifest_path = backup_dir.join("manifest.json");
    let content = fs::read_to_string(&manifest_path).map_err(|error| {
        format!(
            "读取补丁备份清单失败：{}，{error}",
            manifest_path.to_string_lossy()
        )
    })?;
    let manifest: PatchBackupManifest =
        serde_json::from_str(&content).map_err(|error| format!("解析补丁备份清单失败：{error}"))?;
    if path_key(Path::new(&manifest.game_path)) != path_key(game_dir) {
        return Err(format!(
            "备份属于另一个游戏目录：{}。当前目录：{}",
            manifest.game_path,
            game_dir.to_string_lossy()
        ));
    }

    let allowed = patch_required_files()
        .into_iter()
        .map(|path| normalize_relative_path(&path))
        .collect::<BTreeSet<_>>();
    for file in manifest.files {
        if !allowed.contains(&file.relative_path) {
            return Err(format!(
                "备份清单包含不受管理的路径，已停止恢复：{}",
                file.relative_path
            ));
        }
        let relative_path = PathBuf::from(&file.relative_path);
        if relative_path
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
        {
            return Err(format!("备份清单路径不安全：{}", file.relative_path));
        }
        let target = game_dir.join(&relative_path);
        if file.existed {
            let source = backup_dir.join("files").join(&relative_path);
            if !source.is_file() {
                return Err(format!("备份文件缺失：{}", source.to_string_lossy()));
            }
            if let Some(parent) = target.parent() {
                fs::create_dir_all(parent).map_err(|error| format!("创建恢复目录失败：{error}"))?;
            }
            fs::copy(&source, &target).map_err(|error| {
                format!(
                    "恢复原文件失败：{} -> {}，{error}",
                    source.to_string_lossy(),
                    target.to_string_lossy()
                )
            })?;
        } else if target.exists() {
            fs::remove_file(&target).map_err(|error| {
                format!(
                    "移除补丁新增文件失败：{}，{error}",
                    target.to_string_lossy()
                )
            })?;
        }
    }
    Ok(())
}

fn validate_patch_source(patch_source: &Path) -> Result<(), String> {
    if !patch_source.exists() {
        return Err(format!(
            "未找到联机补丁源目录：{}",
            patch_source.to_string_lossy()
        ));
    }

    let missing = patch_required_files()
        .into_iter()
        .filter(|relative_path| !patch_source.join(relative_path).exists())
        .map(|path| normalize_relative_path(&path))
        .collect::<Vec<_>>();

    if !missing.is_empty() {
        return Err(format!("联机补丁源目录缺少文件：{}", missing.join(", ")));
    }

    Ok(())
}

fn copy_patch_tree(patch_source: &Path, game_dir: &Path) -> Result<(), String> {
    for relative_path in patch_required_files() {
        let source = patch_source.join(&relative_path);
        let target = game_dir.join(&relative_path);

        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("创建目录失败：{}，{}", parent.to_string_lossy(), e))?;
        }

        fs::copy(&source, &target).map_err(|e| {
            format!(
                "复制文件失败：{} -> {}，{}",
                source.to_string_lossy(),
                target.to_string_lossy(),
                e
            )
        })?;
    }

    Ok(())
}

fn validate_launch_exe(launch_exe: &Path, game_dir: &Path) -> Result<(), String> {
    if !launch_exe.exists() {
        return Err(format!("启动程序不存在：{}", launch_exe.to_string_lossy()));
    }

    if !launch_exe.is_file() {
        return Err("启动程序必须是 .exe 文件".to_string());
    }

    if !launch_exe
        .extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| ext.eq_ignore_ascii_case("exe"))
    {
        return Err("启动程序必须是 .exe 文件".to_string());
    }

    if !is_path_inside_dir(launch_exe, game_dir) {
        return Err("启动程序必须位于已配置的游戏目录内".to_string());
    }

    Ok(())
}

fn is_path_inside_dir(path: &Path, dir: &Path) -> bool {
    match (fs::canonicalize(path), fs::canonicalize(dir)) {
        (Ok(path), Ok(dir)) => path.starts_with(dir),
        _ => false,
    }
}

fn find_me3_exe(me3_path: &Path) -> Result<PathBuf, String> {
    let direct = me3_path.join("me3.exe");
    if direct.exists() {
        return Ok(direct);
    }

    let nested = me3_path.join("bin").join("me3.exe");
    if nested.exists() {
        return Ok(nested);
    }

    Err("ME3命令行工具未找到，请选择 ME3 根目录或 bin 目录".to_string())
}

fn read_me3_version(me3_exe: &Path) -> Option<(u32, u32, u32)> {
    let output = std::process::Command::new(me3_exe)
        .arg("--version")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .ok()?;
    let combined = format!(
        "{} {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    combined.split_whitespace().find_map(parse_semantic_version)
}

fn parse_semantic_version(value: &str) -> Option<(u32, u32, u32)> {
    value
        .split(|character: char| !character.is_ascii_digit() && character != '.')
        .find_map(|candidate| {
            let mut parts = candidate.split('.');
            let major = parts.next()?.parse().ok()?;
            let minor = parts.next()?.parse().ok()?;
            let patch = parts.next()?.parse().ok()?;
            parts.next().is_none().then_some((major, minor, patch))
        })
}

fn collect_entries_for_mod(
    mod_dir: &Path,
) -> Result<(Vec<PackageEntry>, Vec<NativeEntry>), String> {
    let (packages, natives, _) = collect_profile_data_for_mod(mod_dir)?;
    Ok((packages, natives))
}

fn validate_mmv_seamless_candidate(mod_dir: &Path) -> Result<(), String> {
    let (packages, natives, metadata) = collect_profile_data_for_mod(mod_dir)?;
    if metadata.source_paths.is_empty() {
        return Err("该外部 Mod 没有可读取的作者启动配置（.me3 Profile）".to_string());
    }
    if !natives.iter().any(|entry| {
        network_backend_for_native_path(&entry.path) == NetworkBackend::ServerRedirector
    }) {
        return Err(
            "只允许对包含 cl_server_redirector.dll 的 MMV 作者启动配置启用此模式".to_string(),
        );
    }

    let regulation_files = collect_regulation_files(&packages);
    if regulation_files.len() != 1 {
        return Err(format!(
            "MMV Seamless 兼容模式要求恰好一个玩法数据文件（regulation.bin），当前检测到 {} 个",
            regulation_files.len()
        ));
    }

    Ok(())
}

fn apply_mmv_seamless_community_override(
    mod_dir: &Path,
    packages: &[PackageEntry],
    natives: &mut Vec<NativeEntry>,
    metadata: &mut AuthorProfileMetadata,
    config: &AppConfig,
) -> Result<(), String> {
    validate_mmv_seamless_candidate(mod_dir)?;

    let environment = effective_runtime_environment(config);
    if !matches!(
        environment,
        RuntimeEnvironment::SteamSeamless | RuntimeEnvironment::SpacewarSeamless
    ) {
        return Err(
            "MMV Seamless 社区兼容模式只允许用于 Steam + Seamless 或 Spacewar + Seamless 环境"
                .to_string(),
        );
    }

    let before = natives.len();
    natives.retain(|entry| {
        network_backend_for_native_path(&entry.path) != NetworkBackend::ServerRedirector
    });
    if natives.len() == before {
        return Err("未能从生成副本中移除 Server Redirector".to_string());
    }

    let nrsc = Path::new(&config.game_path)
        .join("SeamlessCoop")
        .join("nrsc.dll");
    if !nrsc.is_file() {
        return Err(format!(
            "MMV Seamless 社区兼容模式需要游戏目录中的 {}",
            nrsc.to_string_lossy()
        ));
    }

    if collect_regulation_files(packages).len() != 1 {
        return Err(
            "MMV Seamless 社区兼容模式要求玩法数据文件（regulation.bin）只属于一个资源型 Mod"
                .to_string(),
        );
    }

    metadata
        .root_fields
        .insert("start_online".to_string(), toml::Value::Boolean(true));
    Ok(())
}

fn collect_regulation_files(packages: &[PackageEntry]) -> Vec<PathBuf> {
    packages
        .iter()
        .filter_map(|entry| {
            let regulation = entry.path.join("regulation.bin");
            regulation.is_file().then_some(regulation)
        })
        .collect()
}

fn regulation_owner_label(path: &Path) -> String {
    let parent = path.parent().unwrap_or(path);
    let owner = if parent
        .file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.eq_ignore_ascii_case("mod"))
    {
        parent.parent().unwrap_or(parent)
    } else {
        parent
    };
    owner
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("未命名 Mod")
        .to_string()
}

fn regulation_owner_labels(paths: &[PathBuf]) -> Vec<String> {
    paths
        .iter()
        .map(|path| regulation_owner_label(path))
        .collect()
}

fn validate_single_regulation_owner(plan: &GeneratedProfilePlan) -> Result<(), String> {
    if plan.regulation_files.len() <= 1 {
        return Ok(());
    }
    Err(format!(
        "当前方案同时启用了 {} 份玩法数据文件，来自：{}。regulation.bin 不能靠加载顺序自动合并；请先停用其中一个，或改用已合并的兼容版本。",
        plan.regulation_files.len(),
        regulation_owner_labels(&plan.regulation_files).join("、")
    ))
}

fn collect_zhocn_packages(packages: &[PackageEntry]) -> Vec<PathBuf> {
    packages
        .iter()
        .filter(|entry| is_complete_zhocn_package(&entry.path))
        .map(|entry| entry.path.clone())
        .collect()
}

fn is_complete_zhocn_package(package_root: &Path) -> bool {
    let root = package_root.join("msg").join("zhocn");
    root.join("item_dlc01.msgbnd.dcx").is_file() && root.join("menu_dlc01.msgbnd.dcx").is_file()
}

fn sha256_file(path: &Path) -> Result<String, String> {
    let mut file = File::open(path)
        .map_err(|error| format!("读取哈希文件失败 {}：{error}", path.display()))?;
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let read = file
            .read(&mut buffer)
            .map_err(|error| format!("计算哈希失败 {}：{error}", path.display()))?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    Ok(format!("{:X}", hasher.finalize()))
}

fn sha256_bytes(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:X}", hasher.finalize())
}

fn manifest_display_name(path: &Path) -> String {
    let name = path
        .file_name()
        .and_then(OsStr::to_str)
        .unwrap_or("未命名")
        .to_string();
    if name.eq_ignore_ascii_case("mod") {
        path.parent()
            .and_then(Path::file_name)
            .and_then(OsStr::to_str)
            .unwrap_or(&name)
            .to_string()
    } else {
        name
    }
}

fn collect_manifest_tree_files(
    root: &Path,
    current: &Path,
    files: &mut Vec<(String, PathBuf)>,
    depth: usize,
) -> Result<(), String> {
    if depth > MAX_SCAN_DEPTH {
        return Err(format!(
            "联机清单扫描已停止：目录嵌套超过安全上限 {MAX_SCAN_DEPTH}"
        ));
    }
    let entries = fs::read_dir(current).map_err(|error| {
        format!(
            "读取资源型 Mod（package）失败 {}：{error}",
            current.display()
        )
    })?;
    for entry in entries {
        let entry = entry.map_err(|error| format!("读取资源型 Mod 项失败：{error}"))?;
        let file_type = entry
            .file_type()
            .map_err(|error| format!("读取资源型 Mod 文件类型失败：{error}"))?;
        if file_type.is_symlink() {
            return Err(format!(
                "联机清单拒绝跟随符号链接：{}",
                entry.path().display()
            ));
        }
        let path = entry.path();
        if file_type.is_dir() {
            collect_manifest_tree_files(root, &path, files, depth + 1)?;
        } else if file_type.is_file() {
            if files.len() >= MAX_CONFLICT_FILES_SCANNED {
                return Err(format!(
                    "联机清单扫描已停止：文件数量超过安全上限 {MAX_CONFLICT_FILES_SCANNED}"
                ));
            }
            let relative = path
                .strip_prefix(root)
                .map_err(|_| "资源型 Mod 文件超出根目录".to_string())?;
            let relative = normalize_relative_path(relative)
                .replace('\\', "/")
                .to_lowercase();
            files.push((relative, path));
        }
    }
    Ok(())
}

fn fingerprint_package_tree(root: &Path) -> Result<(usize, u64, String), String> {
    let mut files = Vec::new();
    collect_manifest_tree_files(root, root, &mut files, 0)?;
    files.sort_by(|left, right| left.0.cmp(&right.0));
    if let Some(duplicate) = files.windows(2).find(|items| items[0].0 == items[1].0) {
        return Err(format!(
            "资源型 Mod（package）包含大小写折叠后重复的相对路径：{}",
            duplicate[0].0
        ));
    }

    let mut total_bytes = 0_u64;
    let mut hasher = Sha256::new();
    for (relative, path) in &files {
        let size = fs::metadata(path)
            .map_err(|error| format!("读取文件大小失败 {}：{error}", path.display()))?
            .len();
        total_bytes = total_bytes
            .checked_add(size)
            .ok_or_else(|| "资源型 Mod 总大小溢出".to_string())?;
        let file_hash = sha256_file(path)?;
        hasher.update(relative.as_bytes());
        hasher.update([0]);
        hasher.update(size.to_le_bytes());
        hasher.update([0]);
        hasher.update(file_hash.as_bytes());
        hasher.update([b'\n']);
    }

    Ok((files.len(), total_bytes, format!("{:X}", hasher.finalize())))
}

fn fingerprint_file(name: &str, path: &Path) -> Result<MultiplayerFileFingerprint, String> {
    Ok(MultiplayerFileFingerprint {
        name: name.to_string(),
        size: fs::metadata(path)
            .map_err(|error| format!("读取文件大小失败 {}：{error}", path.display()))?
            .len(),
        sha256: sha256_file(path)?,
    })
}

fn build_multiplayer_manifest_from_plan(
    plan: &GeneratedProfilePlan,
    config: &AppConfig,
) -> Result<MultiplayerManifest, String> {
    let game_dir = Path::new(&config.game_path);
    validate_game_dir(game_dir)?;

    let mut packages = Vec::with_capacity(plan.packages.len());
    for (index, package) in plan.packages.iter().enumerate() {
        let (file_count, total_bytes, tree_sha256) = fingerprint_package_tree(&package.path)?;
        let zhocn_root = package.path.join("msg").join("zhocn");
        packages.push(MultiplayerPackageFingerprint {
            order: index + 1,
            name: manifest_display_name(&package.path),
            file_count,
            total_bytes,
            tree_sha256,
            regulation_sha256: package
                .path
                .join("regulation.bin")
                .is_file()
                .then(|| sha256_file(&package.path.join("regulation.bin")))
                .transpose()?,
            zhocn_item_sha256: zhocn_root
                .join("item_dlc01.msgbnd.dcx")
                .is_file()
                .then(|| sha256_file(&zhocn_root.join("item_dlc01.msgbnd.dcx")))
                .transpose()?,
            zhocn_menu_sha256: zhocn_root
                .join("menu_dlc01.msgbnd.dcx")
                .is_file()
                .then(|| sha256_file(&zhocn_root.join("menu_dlc01.msgbnd.dcx")))
                .transpose()?,
        });
    }

    let mut natives = Vec::with_capacity(plan.natives.len());
    for (index, native) in plan.natives.iter().enumerate() {
        let file = fingerprint_file(&manifest_display_name(&native.path), &native.path)?;
        natives.push(MultiplayerNativeFingerprint {
            order: index + 1,
            name: file.name,
            size: file.size,
            sha256: file.sha256,
            load_early: native.load_early,
        });
    }

    let mut runtime_files = Vec::new();
    for (name, relative) in [
        ("nightreign.exe", "nightreign.exe"),
        ("OnlineFix64.dll", "OnlineFix64.dll"),
        ("winmm.dll", "winmm.dll"),
        ("steam_api64.dll", "steam_api64.dll"),
    ] {
        let path = game_dir.join(relative);
        if path.is_file() {
            runtime_files.push(fingerprint_file(name, &path)?);
        }
    }
    runtime_files.sort_by(|left, right| left.name.cmp(&right.name));

    let settings_path = game_dir.join("SeamlessCoop").join("nrsc_settings.ini");
    let seamless_settings_sha256 = settings_path
        .is_file()
        .then(|| sha256_file(&settings_path))
        .transpose()?;
    let mut warnings = vec![
        "清单已脱敏：不包含绝对路径、Windows 用户名、账号目录或存档内容。".to_string(),
        "OnlineFix.ini 与 steam_emu.ini 可能包含本机身份信息，未写入清单；双方仍需自行确认联机身份与邀请设置。".to_string(),
    ];
    if plan.regulation_files.len() != 1 {
        warnings.push(format!(
            "当前检测到 {} 个玩法数据文件（regulation.bin）；地图、敌人、武器或扩展服装参数可能不完整或存在覆盖。",
            plan.regulation_files.len()
        ));
    }
    if plan.zhocn_packages.len() != 1 {
        warnings.push(format!(
            "当前检测到 {} 个完整简中覆盖层；建议双方只启用同一份。",
            plan.zhocn_packages.len()
        ));
    }

    let mut manifest = MultiplayerManifest {
        schema_version: 1,
        generated_at: current_timestamp().to_string(),
        manager_version: env!("CARGO_PKG_VERSION").to_string(),
        runtime_environment: effective_runtime_environment(config).as_str().to_string(),
        network_backend: plan.network_backend.as_str().to_string(),
        packages,
        natives,
        runtime_files,
        seamless_settings_sha256,
        overall_sha256: String::new(),
        warnings,
    };
    manifest.overall_sha256 = calculate_multiplayer_manifest_sha256(&manifest)?;
    Ok(manifest)
}

fn calculate_multiplayer_manifest_sha256(manifest: &MultiplayerManifest) -> Result<String, String> {
    let mut comparable = manifest.clone();
    comparable.generated_at.clear();
    comparable.manager_version.clear();
    comparable.overall_sha256.clear();
    comparable.warnings.clear();
    for package in &mut comparable.packages {
        package.name.clear();
    }
    let bytes =
        serde_json::to_vec(&comparable).map_err(|error| format!("序列化联机指纹失败：{error}"))?;
    Ok(sha256_bytes(&bytes))
}

fn build_multiplayer_manifest() -> Result<MultiplayerManifest, String> {
    let config = load_config();
    let plan = build_generated_profile_plan()?;
    build_multiplayer_manifest_from_plan(&plan, &config)
}

fn validate_multiplayer_manifest_path(path: &str, must_exist: bool) -> Result<PathBuf, String> {
    let path = PathBuf::from(path.trim());
    if path.as_os_str().is_empty()
        || !path
            .extension()
            .and_then(OsStr::to_str)
            .is_some_and(|extension| extension.eq_ignore_ascii_case("json"))
    {
        return Err("联机清单必须使用 .json 文件".to_string());
    }
    if must_exist {
        if !path.is_file() {
            return Err("所选联机清单不存在".to_string());
        }
    } else if !path.parent().is_some_and(Path::is_dir) {
        return Err("联机清单目标目录不存在".to_string());
    }
    Ok(path)
}

#[command]
pub async fn export_multiplayer_manifest(
    path: String,
) -> Result<MultiplayerManifestExport, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let path = validate_multiplayer_manifest_path(&path, false)?;
        let manifest = build_multiplayer_manifest()?;
        let content = serde_json::to_string_pretty(&manifest)
            .map_err(|error| format!("序列化联机清单失败：{error}"))?;
        fs::write(&path, content)
            .map_err(|error| format!("写入联机清单失败 {}：{error}", path.display()))?;
        Ok(MultiplayerManifestExport {
            path: path.to_string_lossy().to_string(),
            manifest,
        })
    })
    .await
    .map_err(|error| format!("导出联机清单任务异常结束：{error}"))?
}

fn load_multiplayer_manifest(path: &Path) -> Result<MultiplayerManifest, String> {
    let metadata =
        fs::metadata(path).map_err(|error| format!("读取联机清单元数据失败：{error}"))?;
    if metadata.len() > MAX_MULTIPLAYER_MANIFEST_BYTES {
        return Err(format!(
            "联机清单超过 {} MB 安全上限",
            MAX_MULTIPLAYER_MANIFEST_BYTES / 1024 / 1024
        ));
    }
    let content = fs::read_to_string(path).map_err(|error| format!("读取联机清单失败：{error}"))?;
    let manifest: MultiplayerManifest =
        serde_json::from_str(&content).map_err(|error| format!("联机清单 JSON 无效：{error}"))?;
    if manifest.schema_version != 1 {
        return Err(format!("不支持的联机清单版本：{}", manifest.schema_version));
    }
    validate_multiplayer_manifest_structure(&manifest)?;
    let calculated = calculate_multiplayer_manifest_sha256(&manifest)?;
    if !manifest.overall_sha256.eq_ignore_ascii_case(&calculated) {
        return Err("联机清单总体指纹与内容不一致，文件可能损坏或被手动修改".to_string());
    }
    Ok(manifest)
}

fn validate_multiplayer_manifest_structure(manifest: &MultiplayerManifest) -> Result<(), String> {
    if manifest.packages.len() > 10_000
        || manifest.natives.len() > 10_000
        || manifest.runtime_files.len() > 10_000
    {
        return Err("联机清单条目数量超过安全上限".to_string());
    }
    if manifest
        .packages
        .iter()
        .enumerate()
        .any(|(index, package)| package.order != index + 1)
    {
        return Err("联机一致性清单中的资源型 Mod 加载顺序不连续".to_string());
    }
    if manifest
        .natives
        .iter()
        .enumerate()
        .any(|(index, native)| native.order != index + 1)
    {
        return Err("联机一致性清单中的功能插件加载顺序不连续".to_string());
    }
    let mut runtime_names = BTreeSet::new();
    if manifest
        .runtime_files
        .iter()
        .any(|file| !runtime_names.insert(file.name.to_lowercase()))
    {
        return Err("联机清单包含重复的运行时文件条目".to_string());
    }
    Ok(())
}

fn push_manifest_difference(
    differences: &mut Vec<MultiplayerManifestDifference>,
    severity: &str,
    category: &str,
    item: impl Into<String>,
    local: impl Into<String>,
    peer: impl Into<String>,
) {
    differences.push(MultiplayerManifestDifference {
        severity: severity.to_string(),
        category: category.to_string(),
        item: item.into(),
        local: local.into(),
        peer: peer.into(),
    });
}

fn compare_multiplayer_manifests(
    local: MultiplayerManifest,
    peer: MultiplayerManifest,
) -> MultiplayerManifestComparison {
    let mut differences = Vec::new();
    if local.manager_version != peer.manager_version {
        push_manifest_difference(
            &mut differences,
            "warning",
            "manager",
            "管理器版本",
            &local.manager_version,
            &peer.manager_version,
        );
    }
    for (item, local_value, peer_value) in [
        (
            "运行环境",
            local.runtime_environment.as_str(),
            peer.runtime_environment.as_str(),
        ),
        (
            "联机后端",
            local.network_backend.as_str(),
            peer.network_backend.as_str(),
        ),
    ] {
        if local_value != peer_value {
            push_manifest_difference(
                &mut differences,
                "error",
                "runtime",
                item,
                local_value,
                peer_value,
            );
        }
    }

    if local.packages.len() != peer.packages.len() {
        push_manifest_difference(
            &mut differences,
            "error",
            "packages",
            "资源型 Mod 数量",
            local.packages.len().to_string(),
            peer.packages.len().to_string(),
        );
    }
    for index in 0..local.packages.len().max(peer.packages.len()) {
        match (local.packages.get(index), peer.packages.get(index)) {
            (Some(left), Some(right)) => {
                if left.name != right.name {
                    push_manifest_difference(
                        &mut differences,
                        "warning",
                        "packages",
                        format!("加载顺序 {} / 显示名称", index + 1),
                        &left.name,
                        &right.name,
                    );
                }
                if left.tree_sha256 != right.tree_sha256 {
                    push_manifest_difference(
                        &mut differences,
                        "error",
                        "packages",
                        format!("加载顺序 {}", index + 1),
                        format!("{} · {}", left.name, left.tree_sha256),
                        format!("{} · {}", right.name, right.tree_sha256),
                    );
                }
                for (category, item, local_value, peer_value) in [
                    (
                        "gameplay",
                        "regulation.bin",
                        left.regulation_sha256.as_deref(),
                        right.regulation_sha256.as_deref(),
                    ),
                    (
                        "translation",
                        "item_dlc01.msgbnd.dcx",
                        left.zhocn_item_sha256.as_deref(),
                        right.zhocn_item_sha256.as_deref(),
                    ),
                    (
                        "translation",
                        "menu_dlc01.msgbnd.dcx",
                        left.zhocn_menu_sha256.as_deref(),
                        right.zhocn_menu_sha256.as_deref(),
                    ),
                ] {
                    if local_value != peer_value {
                        push_manifest_difference(
                            &mut differences,
                            "error",
                            category,
                            format!("加载顺序 {} / {item}", index + 1),
                            local_value.unwrap_or("缺少"),
                            peer_value.unwrap_or("缺少"),
                        );
                    }
                }
            }
            (Some(left), None) => push_manifest_difference(
                &mut differences,
                "error",
                "packages",
                format!("加载顺序 {}", index + 1),
                &left.name,
                "缺少",
            ),
            (None, Some(right)) => push_manifest_difference(
                &mut differences,
                "error",
                "packages",
                format!("加载顺序 {}", index + 1),
                "缺少",
                &right.name,
            ),
            (None, None) => {}
        }
    }

    if local.natives.len() != peer.natives.len() {
        push_manifest_difference(
            &mut differences,
            "error",
            "natives",
            "功能插件数量",
            local.natives.len().to_string(),
            peer.natives.len().to_string(),
        );
    }
    for index in 0..local.natives.len().max(peer.natives.len()) {
        match (local.natives.get(index), peer.natives.get(index)) {
            (Some(left), Some(right)) => {
                if left.name != right.name
                    || left.sha256 != right.sha256
                    || left.load_early != right.load_early
                {
                    push_manifest_difference(
                        &mut differences,
                        "error",
                        "natives",
                        format!("加载顺序 {}", index + 1),
                        format!(
                            "{} · {} · early={}",
                            left.name, left.sha256, left.load_early
                        ),
                        format!(
                            "{} · {} · early={}",
                            right.name, right.sha256, right.load_early
                        ),
                    );
                }
            }
            (Some(left), None) => push_manifest_difference(
                &mut differences,
                "error",
                "natives",
                format!("加载顺序 {}", index + 1),
                &left.name,
                "缺少",
            ),
            (None, Some(right)) => push_manifest_difference(
                &mut differences,
                "error",
                "natives",
                format!("加载顺序 {}", index + 1),
                "缺少",
                &right.name,
            ),
            (None, None) => {}
        }
    }

    let local_runtime = local
        .runtime_files
        .iter()
        .map(|item| (item.name.as_str(), item))
        .collect::<BTreeMap<_, _>>();
    let peer_runtime = peer
        .runtime_files
        .iter()
        .map(|item| (item.name.as_str(), item))
        .collect::<BTreeMap<_, _>>();
    for name in local_runtime
        .keys()
        .chain(peer_runtime.keys())
        .copied()
        .collect::<BTreeSet<_>>()
    {
        match (local_runtime.get(name), peer_runtime.get(name)) {
            (Some(left), Some(right)) if left.sha256 != right.sha256 => {
                push_manifest_difference(
                    &mut differences,
                    "error",
                    "runtime_files",
                    name,
                    &left.sha256,
                    &right.sha256,
                );
            }
            (Some(_), None) => push_manifest_difference(
                &mut differences,
                "error",
                "runtime_files",
                name,
                "存在",
                "缺少",
            ),
            (None, Some(_)) => push_manifest_difference(
                &mut differences,
                "error",
                "runtime_files",
                name,
                "缺少",
                "存在",
            ),
            _ => {}
        }
    }
    if local.seamless_settings_sha256 != peer.seamless_settings_sha256 {
        push_manifest_difference(
            &mut differences,
            "error",
            "seamless",
            "nrsc_settings.ini",
            local.seamless_settings_sha256.as_deref().unwrap_or("缺少"),
            peer.seamless_settings_sha256.as_deref().unwrap_or("缺少"),
        );
    }

    MultiplayerManifestComparison {
        compatible: !differences
            .iter()
            .any(|difference| difference.severity == "error"),
        local,
        peer,
        differences,
    }
}

#[command]
pub async fn compare_multiplayer_manifest(
    path: String,
) -> Result<MultiplayerManifestComparison, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let path = validate_multiplayer_manifest_path(&path, true)?;
        let peer = load_multiplayer_manifest(&path)?;
        let local = build_multiplayer_manifest()?;
        Ok(compare_multiplayer_manifests(local, peer))
    })
    .await
    .map_err(|error| format!("比较联机清单任务异常结束：{error}"))?
}

fn collect_profile_data_for_mod(
    mod_dir: &Path,
) -> Result<(Vec<PackageEntry>, Vec<NativeEntry>, AuthorProfileMetadata), String> {
    if mod_dir.is_file() {
        if mod_dir
            .extension()
            .and_then(|ext| ext.to_str())
            .is_some_and(|ext| ext.eq_ignore_ascii_case("dll"))
        {
            let resolved = fs::canonicalize(mod_dir).unwrap_or_else(|_| mod_dir.to_path_buf());
            return Ok((
                Vec::new(),
                vec![NativeEntry {
                    path: normalize_windows_path_buf(resolved),
                    load_early: false,
                    fields: default_native_fields(false),
                }],
                AuthorProfileMetadata::default(),
            ));
        }

        return Ok((Vec::new(), Vec::new(), AuthorProfileMetadata::default()));
    }

    let mut packages = Vec::new();
    let mut natives = Vec::new();
    let mut metadata = AuthorProfileMetadata::default();

    for me3_file in find_top_level_me3_files(mod_dir) {
        let content = fs::read_to_string(&me3_file).map_err(|e| e.to_string())?;
        let (mut mod_packages, mut mod_natives, mut mod_metadata) =
            parse_me3_document(mod_dir, &content)?;
        mod_metadata.source_paths.push(me3_file);
        packages.append(&mut mod_packages);
        natives.append(&mut mod_natives);
        merge_author_profile_metadata(&mut metadata, mod_metadata)?;
    }

    if packages.is_empty() && natives.is_empty() {
        let (mut inferred_packages, mut inferred_natives) = infer_entries_for_mod(mod_dir);
        packages.append(&mut inferred_packages);
        natives.append(&mut inferred_natives);
    }

    Ok((packages, natives, metadata))
}

#[cfg(test)]
fn parse_me3_entries(
    mod_dir: &Path,
    content: &str,
) -> Result<(Vec<PackageEntry>, Vec<NativeEntry>), String> {
    let (packages, natives, _) = parse_me3_document(mod_dir, content)?;
    Ok((packages, natives))
}

fn parse_me3_document(
    mod_dir: &Path,
    content: &str,
) -> Result<(Vec<PackageEntry>, Vec<NativeEntry>, AuthorProfileMetadata), String> {
    let value: toml::Value = content
        .parse()
        .map_err(|e: toml::de::Error| e.to_string())?;
    let table = value
        .as_table()
        .ok_or_else(|| "ME3 启动配置（Profile）根节点必须是 TOML 表".to_string())?;
    let mut packages = Vec::new();
    let mut natives = Vec::new();
    let mut root_fields = table.clone();
    for key in ["packages", "package", "natives"] {
        root_fields.remove(key);
    }
    if let Some(supports) = root_fields
        .get_mut("supports")
        .and_then(toml::Value::as_array_mut)
    {
        for support in supports {
            if let Some(fields) = support.as_table_mut() {
                fields.insert(
                    "game".to_string(),
                    toml::Value::String("nightreign".to_string()),
                );
            }
        }
    }

    for key in ["packages", "package"] {
        if let Some(entries) = value.get(key).and_then(|v| v.as_array()) {
            for entry in entries {
                if let Some(path) = entry.get("path").and_then(|v| v.as_str()) {
                    let resolved = resolve_mod_path(mod_dir, path);
                    if resolved.exists() {
                        let mut fields = entry.as_table().cloned().unwrap_or_default();
                        fields.insert(
                            "path".to_string(),
                            toml::Value::String(normalize_windows_path_string(
                                &resolved.to_string_lossy(),
                            )),
                        );
                        packages.push(PackageEntry {
                            path: resolved,
                            fields,
                        });
                    }
                }
            }
        }
    }

    if let Some(entries) = value.get("natives").and_then(|v| v.as_array()) {
        for entry in entries {
            if let Some(path) = entry.get("path").and_then(|v| v.as_str()) {
                let resolved = resolve_mod_path(mod_dir, path);
                if resolved.exists() {
                    let mut fields = entry.as_table().cloned().unwrap_or_default();
                    fields.insert(
                        "path".to_string(),
                        toml::Value::String(normalize_windows_path_string(
                            &resolved.to_string_lossy(),
                        )),
                    );
                    let load_early = entry
                        .get("load_early")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false);
                    natives.push(NativeEntry {
                        path: resolved,
                        load_early,
                        fields,
                    });
                }
            }
        }
    }

    Ok((
        packages,
        natives,
        AuthorProfileMetadata {
            root_fields,
            source_paths: Vec::new(),
        },
    ))
}

fn infer_entries_for_mod(mod_dir: &Path) -> (Vec<PackageEntry>, Vec<NativeEntry>) {
    let mut packages = Vec::new();
    let natives = find_dll_files(mod_dir)
        .into_iter()
        .map(|path| NativeEntry {
            path,
            load_early: false,
            fields: default_native_fields(false),
        })
        .collect();

    let mod_subdir = mod_dir.join("mod");
    if mod_subdir.is_dir() {
        packages.push(PackageEntry {
            path: mod_subdir,
            fields: default_package_fields(),
        });
    } else if has_package_like_content(mod_dir) {
        packages.push(PackageEntry {
            path: mod_dir.to_path_buf(),
            fields: default_package_fields(),
        });
    }

    (packages, natives)
}

fn infer_game_root_natives(game_dir: &Path) -> Vec<NativeEntry> {
    if game_dir.as_os_str().is_empty() {
        return Vec::new();
    }

    let mut natives = Vec::new();
    let seamless_dir = game_dir.join("SeamlessCoop");
    let nrsc = seamless_dir.join("nrsc.dll");
    if nrsc.exists() {
        natives.push(NativeEntry {
            path: normalize_windows_path_buf(nrsc),
            load_early: true,
            fields: default_native_fields(true),
        });
    }

    for nighter in [
        seamless_dir.join("nighter.dll"),
        game_dir.join("mods").join("nighter.dll"),
    ] {
        if !nighter.exists() {
            continue;
        }
        natives.push(NativeEntry {
            path: normalize_windows_path_buf(nighter),
            load_early: false,
            fields: default_native_fields(false),
        });
    }

    natives
}

fn resolve_mod_path(mod_dir: &Path, entry_path: &str) -> PathBuf {
    let path = Path::new(entry_path);
    let resolved = if path.is_absolute() {
        path.to_path_buf()
    } else {
        mod_dir.join(path)
    };

    normalize_windows_path_buf(fs::canonicalize(&resolved).unwrap_or(resolved))
}

fn build_me3_profile(
    metadata: &AuthorProfileMetadata,
    packages: &[PackageEntry],
    natives: &[NativeEntry],
) -> Result<String, String> {
    let mut root = metadata.root_fields.clone();
    root.entry("profileVersion".to_string())
        .or_insert_with(|| toml::Value::String("v1".to_string()));

    let mut supports = root
        .remove("supports")
        .and_then(|value| value.as_array().cloned())
        .unwrap_or_default();
    if supports.is_empty() {
        supports.push(toml::Value::Table(toml::Table::new()));
    }
    for support in &mut supports {
        if let Some(fields) = support.as_table_mut() {
            fields.insert(
                "game".to_string(),
                toml::Value::String("nightreign".to_string()),
            );
        }
    }
    root.insert("supports".to_string(), toml::Value::Array(supports));
    root.insert(
        "natives".to_string(),
        toml::Value::Array(
            natives
                .iter()
                .map(|native| {
                    let mut fields = native.fields.clone();
                    fields.insert(
                        "path".to_string(),
                        toml::Value::String(normalize_windows_path_string(
                            &native.path.to_string_lossy(),
                        )),
                    );
                    fields.insert(
                        "load_early".to_string(),
                        toml::Value::Boolean(native.load_early),
                    );
                    toml::Value::Table(fields)
                })
                .collect(),
        ),
    );
    root.insert(
        "packages".to_string(),
        toml::Value::Array(
            packages
                .iter()
                .map(|package| {
                    let mut fields = package.fields.clone();
                    fields.insert(
                        "path".to_string(),
                        toml::Value::String(normalize_windows_path_string(
                            &package.path.to_string_lossy(),
                        )),
                    );
                    toml::Value::Table(fields)
                })
                .collect(),
        ),
    );

    toml::to_string_pretty(&root)
        .map_err(|error| format!("生成 ME3 启动配置（Profile）失败：{error}"))
}

fn default_package_fields() -> toml::Table {
    let mut fields = toml::Table::new();
    fields.insert("enabled".to_string(), toml::Value::Boolean(true));
    fields.insert("load_after".to_string(), toml::Value::Array(Vec::new()));
    fields.insert("load_before".to_string(), toml::Value::Array(Vec::new()));
    fields
}

fn default_native_fields(load_early: bool) -> toml::Table {
    let mut fields = default_package_fields();
    fields.insert("optional".to_string(), toml::Value::Boolean(false));
    fields.insert("load_early".to_string(), toml::Value::Boolean(load_early));
    fields
}

fn merge_author_profile_metadata(
    target: &mut AuthorProfileMetadata,
    incoming: AuthorProfileMetadata,
) -> Result<(), String> {
    for (key, value) in incoming.root_fields {
        if let Some(existing) = target.root_fields.get(&key) {
            if existing != &value {
                return Err(format!(
                    "多个作者启动配置的根字段冲突：{key}。请只启用一个需要独立启动语义的整合包。"
                ));
            }
            continue;
        }
        target.root_fields.insert(key, value);
    }
    target.source_paths.extend(incoming.source_paths);
    Ok(())
}

fn network_backend_for_native_path(path: &Path) -> NetworkBackend {
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default();
    if file_name.eq_ignore_ascii_case("cl_server_redirector.dll") {
        NetworkBackend::ServerRedirector
    } else if file_name.eq_ignore_ascii_case("nrsc.dll") {
        NetworkBackend::Seamless
    } else {
        NetworkBackend::None
    }
}

fn detect_network_backend(natives: &[NativeEntry]) -> Result<NetworkBackend, String> {
    let has_server_redirector = natives.iter().any(|entry| {
        network_backend_for_native_path(&entry.path) == NetworkBackend::ServerRedirector
    });
    let has_seamless = natives
        .iter()
        .any(|entry| network_backend_for_native_path(&entry.path) == NetworkBackend::Seamless);
    let has_nighter = natives.iter().any(|entry| {
        entry
            .path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.eq_ignore_ascii_case("nighter.dll"))
    });

    if has_server_redirector && (has_seamless || has_nighter) {
        Err(
            "检测到 Server Redirector 与 SeamlessCoop/nighter 同时启用。MMV 方案只能使用 Server Redirector，请停用单独注册的 nrsc.dll 或 nighter.dll。"
                .to_string(),
        )
    } else if has_server_redirector {
        Ok(NetworkBackend::ServerRedirector)
    } else if has_seamless {
        Ok(NetworkBackend::Seamless)
    } else {
        Ok(NetworkBackend::None)
    }
}

fn has_dll_file(path: &Path) -> bool {
    contains_file_with_extension(path, "dll", 0)
}

fn contains_file_with_extension(path: &Path, extension: &str, depth: usize) -> bool {
    if depth > MAX_SCAN_DEPTH {
        return false;
    }
    let Ok(entries) = fs::read_dir(path) else {
        return false;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if contains_file_with_extension(&path, extension, depth + 1) {
                return true;
            }
        } else if path
            .extension()
            .and_then(|ext| ext.to_str())
            .is_some_and(|ext| ext.eq_ignore_ascii_case(extension))
        {
            return true;
        }
    }

    false
}

fn is_dll_or_disabled_dll(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| {
            let lower = name.to_lowercase();
            lower.ends_with(".dll") || lower.ends_with(".dll.disabled")
        })
}

fn find_dll_files(path: &Path) -> Vec<PathBuf> {
    let mut dlls = Vec::new();
    collect_files_with_extension(path, "dll", &mut dlls);
    dlls
}

fn find_config_files(path: &Path) -> Vec<String> {
    if path.is_file() {
        return find_sidecar_config_files(path)
            .into_iter()
            .map(|path| path.to_string_lossy().to_string())
            .collect();
    }

    let Ok(entries) = fs::read_dir(path) else {
        return Vec::new();
    };

    entries
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.is_file() && is_editable_config_extension(path))
        .map(|path| path.to_string_lossy().to_string())
        .collect()
}

fn find_sidecar_config_files(path: &Path) -> Vec<PathBuf> {
    let active_path = if is_disabled_path(path) {
        active_path_for(path)
    } else {
        path.to_path_buf()
    };
    let Some(parent) = active_path.parent() else {
        return Vec::new();
    };
    let Some(stem) = active_path.file_stem().and_then(|stem| stem.to_str()) else {
        return Vec::new();
    };

    ["json", "ini"]
        .iter()
        .map(|extension| parent.join(format!("{stem}.{extension}")))
        .filter(|candidate| candidate.exists())
        .collect()
}

fn is_editable_config_extension(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| ext.eq_ignore_ascii_case("json") || ext.eq_ignore_ascii_case("ini"))
}

fn same_path_string(left: &str, right: &str) -> bool {
    let left_path = Path::new(left);
    let right_path = Path::new(right);
    match (fs::canonicalize(left_path), fs::canonicalize(right_path)) {
        (Ok(left), Ok(right)) => path_key(&left) == path_key(&right),
        _ => normalize_windows_path_string(left)
            .eq_ignore_ascii_case(&normalize_windows_path_string(right)),
    }
}

fn path_key(path: &Path) -> String {
    normalize_windows_path_string(&path.to_string_lossy()).to_lowercase()
}

fn normalize_windows_path_buf(path: PathBuf) -> PathBuf {
    PathBuf::from(normalize_windows_path_string(&path.to_string_lossy()))
}

fn normalize_windows_path_string(path: &str) -> String {
    if let Some(stripped) = path.strip_prefix(r"\\?\UNC\") {
        format!(r"\\{}", stripped)
    } else if let Some(stripped) = path.strip_prefix(r"\\?\") {
        stripped.to_string()
    } else {
        path.to_string()
    }
}

fn validate_editable_config_path(path: &str) -> Result<PathBuf, String> {
    let config_path =
        fs::canonicalize(Path::new(path.trim())).map_err(|e| format!("配置文件不存在：{e}"))?;
    if !config_path.is_file() || !is_editable_config_extension(&config_path) {
        return Err("只能编辑 JSON 或 INI 配置文件".to_string());
    }

    let mods = collect_mods()?;
    let allowed = mods.iter().any(|mod_info| {
        mod_info.config_files.iter().any(|candidate| {
            fs::canonicalize(candidate)
                .map(|candidate| candidate == config_path)
                .unwrap_or(false)
        })
    });

    if !allowed {
        return Err("该配置文件不属于当前已扫描的 Mod".to_string());
    }

    Ok(config_path)
}

fn collect_files_with_extension(path: &Path, extension: &str, files: &mut Vec<PathBuf>) {
    collect_files_with_extension_inner(path, extension, files, 0);
}

fn collect_files_with_extension_inner(
    path: &Path,
    extension: &str,
    files: &mut Vec<PathBuf>,
    depth: usize,
) {
    if depth > MAX_SCAN_DEPTH {
        return;
    }
    let Ok(entries) = fs::read_dir(path) else {
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_files_with_extension_inner(&path, extension, files, depth + 1);
        } else if path
            .extension()
            .and_then(|ext| ext.to_str())
            .is_some_and(|ext| ext.eq_ignore_ascii_case(extension))
        {
            files.push(normalize_windows_path_buf(
                fs::canonicalize(&path).unwrap_or(path),
            ));
        }
    }
}

fn collect_conflict_package_files(
    package_root: &Path,
    mod_index: usize,
    first_owner_by_path: &mut HashMap<String, usize>,
    conflict_owners_by_path: &mut BTreeMap<String, BTreeSet<usize>>,
    scanned_files: &mut usize,
) -> Result<(), String> {
    collect_conflict_package_files_inner(
        package_root,
        package_root,
        mod_index,
        first_owner_by_path,
        conflict_owners_by_path,
        scanned_files,
        0,
    )
}

fn collect_conflict_package_files_inner(
    package_root: &Path,
    current_dir: &Path,
    mod_index: usize,
    first_owner_by_path: &mut HashMap<String, usize>,
    conflict_owners_by_path: &mut BTreeMap<String, BTreeSet<usize>>,
    scanned_files: &mut usize,
    depth: usize,
) -> Result<(), String> {
    if depth > MAX_SCAN_DEPTH {
        return Err(format!(
            "冲突分析已停止：目录嵌套超过安全上限 {}",
            MAX_SCAN_DEPTH
        ));
    }
    let Ok(entries) = fs::read_dir(current_dir) else {
        return Ok(());
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_conflict_package_files_inner(
                package_root,
                &path,
                mod_index,
                first_owner_by_path,
                conflict_owners_by_path,
                scanned_files,
                depth + 1,
            )?;
            continue;
        }

        let Some(relative_path) = path
            .strip_prefix(package_root)
            .ok()
            .map(normalize_relative_path)
        else {
            continue;
        };

        if relative_path.is_empty() {
            continue;
        }

        record_conflict_candidate(
            relative_path.to_lowercase(),
            mod_index,
            first_owner_by_path,
            conflict_owners_by_path,
            scanned_files,
        )?;
    }

    Ok(())
}

fn record_conflict_candidate(
    relative_path: String,
    mod_index: usize,
    first_owner_by_path: &mut HashMap<String, usize>,
    conflict_owners_by_path: &mut BTreeMap<String, BTreeSet<usize>>,
    scanned_files: &mut usize,
) -> Result<(), String> {
    *scanned_files += 1;
    if *scanned_files > MAX_CONFLICT_FILES_SCANNED {
        return Err(format!(
            "冲突分析已停止：文件数量超过安全上限 {}，请减少启用的 Mod 后重试",
            MAX_CONFLICT_FILES_SCANNED
        ));
    }

    match first_owner_by_path.entry(relative_path) {
        std::collections::hash_map::Entry::Vacant(entry) => {
            entry.insert(mod_index);
        }
        std::collections::hash_map::Entry::Occupied(entry) => {
            let first_owner = *entry.get();
            if first_owner != mod_index {
                let owners = conflict_owners_by_path
                    .entry(entry.key().clone())
                    .or_default();
                owners.insert(first_owner);
                owners.insert(mod_index);

                if conflict_owners_by_path.len() > MAX_CONFLICT_RESULTS {
                    return Err(format!(
                        "冲突分析已停止：冲突数量超过安全上限 {}",
                        MAX_CONFLICT_RESULTS
                    ));
                }
            }
        }
    }

    Ok(())
}

fn normalize_relative_path(path: &Path) -> String {
    path.components()
        .filter_map(|component| match component {
            Component::Normal(value) => Some(value.to_string_lossy().to_string()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("/")
}

fn has_package_like_content(path: &Path) -> bool {
    const PACKAGE_DIRS: &[&str] = &[
        "action", "chr", "event", "map", "material", "menu", "msg", "parts", "script", "sfx",
    ];

    if path.join("mod").is_dir() || path.join("regulation.bin").exists() {
        return true;
    }

    PACKAGE_DIRS.iter().any(|dir| path.join(dir).is_dir())
        || fs::read_dir(path)
            .ok()
            .into_iter()
            .flatten()
            .flatten()
            .any(|e| {
                e.path()
                    .extension()
                    .and_then(|ext| ext.to_str())
                    .is_some_and(|ext| ext.eq_ignore_ascii_case("dcx"))
            })
}

#[derive(Default)]
struct ClothingScanState {
    local_parts: BTreeSet<String>,
    online_parts: BTreeSet<String>,
    appearance_ids: BTreeSet<String>,
    has_regulation: bool,
    has_manual_online_setup: bool,
    scanned_files: usize,
    truncated: bool,
}

fn analyze_clothing_mod(path: &Path) -> ClothingModInfo {
    if !path.is_dir() {
        return ClothingModInfo {
            kind: "none".to_string(),
            online_support: "not_applicable".to_string(),
            ..ClothingModInfo::default()
        };
    }

    let mut state = ClothingScanState::default();
    collect_clothing_files(path, path, &mut state, 0);

    let detected = !state.local_parts.is_empty() || !state.online_parts.is_empty();
    if !detected {
        return ClothingModInfo {
            kind: "none".to_string(),
            online_support: "not_applicable".to_string(),
            has_regulation: state.has_regulation,
            has_manual_online_setup: state.has_manual_online_setup,
            ..ClothingModInfo::default()
        };
    }

    let paired_part_file_count = state
        .local_parts
        .iter()
        .filter(|path| state.online_parts.contains(&online_part_path(path)))
        .count();
    let missing_online_part_count = state
        .local_parts
        .len()
        .saturating_sub(paired_part_file_count);
    let orphan_online_part_count = state
        .online_parts
        .iter()
        .filter(|path| !state.local_parts.contains(&local_part_path(path)))
        .count();
    let online_support = if state.online_parts.is_empty() {
        "missing"
    } else if missing_online_part_count == 0 && orphan_online_part_count == 0 {
        "complete"
    } else {
        "partial"
    };
    let requires_appearance_reset = state.has_regulation;
    let mut warnings = Vec::new();

    if requires_appearance_reset {
        warnings.push(
            "包含玩法数据文件，可能提供本体不存在的服装 ID；停用或删除前请先在游戏内换回本体服装"
                .to_string(),
        );
    }
    match online_support {
        "missing" => warnings.push(
            "未发现与本机服装资源配对的 _l 队友视角文件；联机队友看到的外观可能不同或异常"
                .to_string(),
        ),
        "partial" => warnings.push(format!(
            "队友视角资源不完整：缺少 {missing_online_part_count} 个 _l 配对，另有 {orphan_online_part_count} 个孤立 _l 文件"
        )),
        _ => {}
    }
    if state.has_manual_online_setup {
        warnings.push(
            "包含联机准备脚本；管理器只识别并提示，不会自动运行 BAT、CMD、PowerShell 或 EXE"
                .to_string(),
        );
    }
    if state.truncated {
        warnings.push(format!(
            "服装结构扫描达到 {MAX_CLOTHING_SCAN_FILES} 个文件上限，结果可能不完整"
        ));
    }

    ClothingModInfo {
        detected,
        kind: if requires_appearance_reset {
            "expanded".to_string()
        } else {
            "replacement".to_string()
        },
        part_file_count: state.local_parts.len() + state.online_parts.len(),
        local_part_file_count: state.local_parts.len(),
        online_part_file_count: state.online_parts.len(),
        paired_part_file_count,
        missing_online_part_count,
        orphan_online_part_count,
        has_regulation: state.has_regulation,
        has_manual_online_setup: state.has_manual_online_setup,
        online_support: online_support.to_string(),
        requires_appearance_reset,
        appearance_ids: state.appearance_ids.into_iter().collect(),
        warnings,
    }
}

fn safe_initial_mod_enabled(clothing: &ClothingModInfo) -> bool {
    !clothing.requires_appearance_reset
}

fn collect_clothing_files(
    root: &Path,
    current: &Path,
    state: &mut ClothingScanState,
    depth: usize,
) {
    if depth > MAX_SCAN_DEPTH || state.truncated {
        return;
    }
    let Ok(entries) = fs::read_dir(current) else {
        return;
    };

    for entry in entries.flatten() {
        if state.scanned_files >= MAX_CLOTHING_SCAN_FILES {
            state.truncated = true;
            return;
        }
        let Ok(file_type) = entry.file_type() else {
            continue;
        };
        if file_type.is_symlink() {
            continue;
        }
        let path = entry.path();
        if file_type.is_dir() {
            collect_clothing_files(root, &path, state, depth + 1);
            continue;
        }
        if !file_type.is_file() {
            continue;
        }

        state.scanned_files += 1;
        let file_name = entry.file_name().to_string_lossy().to_ascii_lowercase();
        if file_name == "regulation.bin" {
            state.has_regulation = true;
        }
        if is_manual_online_setup_file(&file_name) {
            state.has_manual_online_setup = true;
        }
        if !is_clothing_part_file(&file_name) {
            continue;
        }

        let relative = path
            .strip_prefix(root)
            .ok()
            .map(normalize_relative_path)
            .unwrap_or_else(|| file_name.clone())
            .to_ascii_lowercase();
        if file_name.ends_with("_l.partsbnd.dcx") {
            state.online_parts.insert(relative);
        } else {
            state.local_parts.insert(relative);
        }
        if let Some(appearance_id) = clothing_appearance_id(&file_name) {
            state.appearance_ids.insert(appearance_id);
        }
    }
}

fn is_clothing_part_file(file_name: &str) -> bool {
    file_name.ends_with(".partsbnd.dcx")
        && ["am_", "bd_", "hd_", "lg_", "fc_", "hr_"]
            .iter()
            .any(|prefix| file_name.starts_with(prefix))
}

fn is_manual_online_setup_file(file_name: &str) -> bool {
    file_name.contains("online")
        && [".bat", ".cmd", ".ps1", ".exe"]
            .iter()
            .any(|extension| file_name.ends_with(extension))
}

fn clothing_appearance_id(file_name: &str) -> Option<String> {
    let stem = file_name.strip_suffix(".partsbnd.dcx")?;
    let stem = stem.strip_suffix("_l").unwrap_or(stem);
    stem.split('_')
        .find(|segment| {
            segment.len() >= 3 && segment.chars().all(|character| character.is_ascii_digit())
        })
        .map(ToOwned::to_owned)
}

fn online_part_path(local_path: &str) -> String {
    local_path
        .strip_suffix(".partsbnd.dcx")
        .map(|stem| format!("{stem}_l.partsbnd.dcx"))
        .unwrap_or_else(|| format!("{local_path}_l"))
}

fn local_part_path(online_path: &str) -> String {
    online_path
        .strip_suffix("_l.partsbnd.dcx")
        .map(|stem| format!("{stem}.partsbnd.dcx"))
        .unwrap_or_else(|| online_path.to_string())
}

fn is_disabled_path(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.ends_with(".disabled"))
}

fn strip_disabled_suffix(name: &str) -> &str {
    name.strip_suffix(".disabled").unwrap_or(name)
}

fn disabled_path_for(path: &Path) -> PathBuf {
    if is_disabled_path(path) {
        return path.to_path_buf();
    }

    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    let name = path.file_name().unwrap_or_default().to_string_lossy();
    parent.join(format!("{name}.disabled"))
}

fn active_path_for(path: &Path) -> PathBuf {
    if !is_disabled_path(path) {
        return path.to_path_buf();
    }

    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    let name = path.file_name().unwrap_or_default().to_string_lossy();
    parent.join(strip_disabled_suffix(&name))
}

fn current_timestamp() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
}

#[cfg(test)]
mod tests {
    use super::*;
    use zip::write::FileOptions;

    fn create_test_zip(path: &Path, entries: &[(&str, &[u8])]) {
        let file = File::create(path).unwrap();
        let mut writer = zip::ZipWriter::new(file);
        let options = FileOptions::default().compression_method(zip::CompressionMethod::Stored);
        for (name, content) in entries {
            writer.start_file(*name, options).unwrap();
            writer.write_all(content).unwrap();
        }
        writer.finish().unwrap();
    }

    #[test]
    fn infers_package_for_parts_only_mod() {
        let mod_dir = PathBuf::from(r"D:\Game\mods\duchessunmask");
        let packages = vec![PackageEntry {
            path: mod_dir.clone(),
            fields: default_package_fields(),
        }];
        let natives = Vec::new();

        let profile =
            build_me3_profile(&AuthorProfileMetadata::default(), &packages, &natives).unwrap();
        let value = profile.parse::<toml::Value>().unwrap();

        assert_eq!(
            value
                .get("supports")
                .and_then(toml::Value::as_array)
                .and_then(|items| items.first())
                .and_then(|entry| entry.get("game"))
                .and_then(toml::Value::as_str),
            Some("nightreign")
        );
        assert_eq!(
            value
                .get("packages")
                .and_then(toml::Value::as_array)
                .and_then(|items| items.first())
                .and_then(|entry| entry.get("path"))
                .and_then(toml::Value::as_str),
            Some(r"D:\Game\mods\duchessunmask")
        );
    }

    #[test]
    fn classifies_parts_only_clothing_and_missing_online_view() {
        let root = std::env::temp_dir().join(format!(
            "nightreign_clothing_replacement_test_{}",
            current_timestamp()
        ));
        fs::create_dir_all(root.join("parts")).unwrap();
        fs::write(
            root.join("parts").join("bd_m_5030.partsbnd.dcx"),
            b"duchess",
        )
        .unwrap();

        let clothing = analyze_clothing_mod(&root);

        assert!(clothing.detected);
        assert_eq!(clothing.kind, "replacement");
        assert_eq!(clothing.local_part_file_count, 1);
        assert_eq!(clothing.online_part_file_count, 0);
        assert_eq!(clothing.online_support, "missing");
        assert!(!clothing.requires_appearance_reset);
        assert!(safe_initial_mod_enabled(&clothing));
        assert_eq!(clothing.appearance_ids, vec!["5030"]);
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn classifies_expanded_clothing_and_complete_online_pairs() {
        let root = std::env::temp_dir().join(format!(
            "nightreign_clothing_expansion_test_{}",
            current_timestamp()
        ));
        let parts = root.join("parts");
        fs::create_dir_all(&parts).unwrap();
        fs::write(root.join("regulation.bin"), b"extended outfits").unwrap();
        fs::write(parts.join("bd_m_9996.partsbnd.dcx"), b"local").unwrap();
        fs::write(parts.join("bd_m_9996_l.partsbnd.dcx"), b"online").unwrap();
        fs::write(parts.join("01_Online.bat"), b"copy local online").unwrap();

        let clothing = analyze_clothing_mod(&root);

        assert_eq!(clothing.kind, "expanded");
        assert!(clothing.has_regulation);
        assert!(clothing.has_manual_online_setup);
        assert!(clothing.requires_appearance_reset);
        assert!(!safe_initial_mod_enabled(&clothing));
        assert_eq!(clothing.online_support, "complete");
        assert_eq!(clothing.paired_part_file_count, 1);
        assert_eq!(clothing.missing_online_part_count, 0);
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn risky_clothing_destination_stays_disabled_when_names_collide() {
        let root = std::env::temp_dir().join(format!(
            "nightreign_clothing_destination_test_{}",
            current_timestamp()
        ));
        fs::create_dir_all(root.join("Outfits.disabled")).unwrap();

        let destination = unique_mod_destination(&root, "Outfits", false);

        assert_eq!(destination, root.join("Outfits_1.disabled"));
        assert!(is_disabled_path(&destination));
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    #[ignore = "requires local pure-replacement and expanded-clothing sample directories"]
    fn verifies_local_clothing_samples_read_only() {
        let replacement_dir = std::env::var("NIGHTREIGN_CLOTHING_REPLACEMENT_TEST_DIR")
            .map(PathBuf::from)
            .expect("set NIGHTREIGN_CLOTHING_REPLACEMENT_TEST_DIR");
        let expanded_dir = std::env::var("NIGHTREIGN_CLOTHING_EXPANDED_TEST_DIR")
            .map(PathBuf::from)
            .expect("set NIGHTREIGN_CLOTHING_EXPANDED_TEST_DIR");

        let replacement = analyze_clothing_mod(&replacement_dir);
        let expanded = analyze_clothing_mod(&expanded_dir);

        assert_eq!(replacement.kind, "replacement");
        assert_eq!(replacement.local_part_file_count, 5);
        assert_eq!(replacement.online_support, "missing");
        assert!(!replacement.has_regulation);
        assert_eq!(expanded.kind, "expanded");
        assert_eq!(expanded.local_part_file_count, 228);
        assert_eq!(expanded.online_part_file_count, 228);
        assert_eq!(expanded.paired_part_file_count, 228);
        assert_eq!(expanded.online_support, "complete");
        assert!(expanded.has_regulation);
        assert!(expanded.has_manual_online_setup);
    }

    #[test]
    fn build_profile_preserves_package_order() {
        let packages = vec![
            PackageEntry {
                path: PathBuf::from(r"D:\Game\mods\z_last"),
                fields: default_package_fields(),
            },
            PackageEntry {
                path: PathBuf::from(r"D:\Game\mods\a_first"),
                fields: default_package_fields(),
            },
        ];
        let natives = Vec::new();

        let profile =
            build_me3_profile(&AuthorProfileMetadata::default(), &packages, &natives).unwrap();
        let value = profile.parse::<toml::Value>().unwrap();
        let paths = value
            .get("packages")
            .and_then(toml::Value::as_array)
            .unwrap()
            .iter()
            .filter_map(|entry| entry.get("path").and_then(toml::Value::as_str))
            .collect::<Vec<_>>();

        assert_eq!(paths, vec![r"D:\Game\mods\z_last", r"D:\Game\mods\a_first"]);
    }

    #[test]
    fn build_profile_strips_windows_verbatim_prefix() {
        let packages = vec![PackageEntry {
            path: PathBuf::from(r"\\?\D:\Game\mods\duchessunmask"),
            fields: default_package_fields(),
        }];
        let natives = vec![NativeEntry {
            path: PathBuf::from(r"\\?\D:\Game\mods\nighter.dll"),
            load_early: false,
            fields: default_native_fields(false),
        }];

        let profile =
            build_me3_profile(&AuthorProfileMetadata::default(), &packages, &natives).unwrap();
        let value = profile.parse::<toml::Value>().unwrap();
        let package_path = value
            .get("packages")
            .and_then(toml::Value::as_array)
            .and_then(|items| items.first())
            .and_then(|entry| entry.get("path"))
            .and_then(toml::Value::as_str)
            .unwrap();
        let native_path = value
            .get("natives")
            .and_then(toml::Value::as_array)
            .and_then(|items| items.first())
            .and_then(|entry| entry.get("path"))
            .and_then(toml::Value::as_str)
            .unwrap();

        assert_eq!(package_path, r"D:\Game\mods\duchessunmask");
        assert_eq!(native_path, r"D:\Game\mods\nighter.dll");
    }

    #[test]
    fn unique_native_paths_ignore_verbatim_prefix() {
        let mut target = Vec::new();
        let mut seen = BTreeSet::new();

        extend_unique_natives(
            &mut target,
            &mut seen,
            vec![
                NativeEntry {
                    path: PathBuf::from(r"\\?\D:\Game\mods\nighter.dll"),
                    load_early: false,
                    fields: default_native_fields(false),
                },
                NativeEntry {
                    path: PathBuf::from(r"D:\Game\mods\nighter.dll"),
                    load_early: false,
                    fields: default_native_fields(false),
                },
            ],
        );

        assert_eq!(target.len(), 1);
    }

    #[test]
    fn parse_me3_entries_skips_missing_paths() {
        let root = std::env::temp_dir().join(format!(
            "nightreign_mod_manager_test_{}",
            current_timestamp()
        ));
        let mod_dir = root.join("More Map");
        let package_dir = mod_dir.join("mod");
        fs::create_dir_all(&package_dir).unwrap();

        let content = r#"
profileVersion = "v1"

[[packages]]
path = "mod"

[[natives]]
path = "mod/SeamlessCoop/nrsc.dll"
load_early = true
"#;

        let (packages, natives) = parse_me3_entries(&mod_dir, content).unwrap();
        let _ = fs::remove_dir_all(&root);

        assert_eq!(packages.len(), 1);
        assert!(natives.is_empty());
    }

    #[test]
    fn zip_installer_preserves_game_semantic_roots() {
        let root = std::env::temp_dir().join(format!(
            "nightreign_mod_manager_semantic_zip_test_{}",
            current_timestamp()
        ));
        fs::create_dir_all(&root).unwrap();

        let message_zip = root.join("message.zip");
        create_test_zip(
            &message_zip,
            &[
                ("msg/zhocn/item_dlc01.msgbnd.dcx", b"item"),
                ("msg/zhocn/menu_dlc01.msgbnd.dcx", b"menu"),
            ],
        );
        let mut message_archive = ZipArchive::new(File::open(&message_zip).unwrap()).unwrap();
        assert_eq!(detect_single_zip_root(&mut message_archive).unwrap(), None);
        drop(message_archive);
        let message_output = root.join("message-output");
        fs::create_dir_all(&message_output).unwrap();
        extract_zip(&message_zip, &message_output).unwrap();
        assert!(message_output
            .join("msg")
            .join("zhocn")
            .join("item_dlc01.msgbnd.dcx")
            .is_file());

        let parts_zip = root.join("parts.zip");
        create_test_zip(&parts_zip, &[("parts/example.partsbnd.dcx", b"parts")]);
        let mut parts_archive = ZipArchive::new(File::open(&parts_zip).unwrap()).unwrap();
        assert_eq!(detect_single_zip_root(&mut parts_archive).unwrap(), None);

        let regulation_zip = root.join("regulation.zip");
        create_test_zip(&regulation_zip, &[("regulation.bin", b"regulation")]);
        let mut regulation_archive = ZipArchive::new(File::open(&regulation_zip).unwrap()).unwrap();
        assert_eq!(
            detect_single_zip_root(&mut regulation_archive).unwrap(),
            None
        );

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn zip_installer_only_strips_a_real_wrapper_directory() {
        let root = std::env::temp_dir().join(format!(
            "nightreign_mod_manager_wrapper_zip_test_{}",
            current_timestamp()
        ));
        fs::create_dir_all(&root).unwrap();
        let archive_path = root.join("wrapped.zip");
        create_test_zip(
            &archive_path,
            &[
                ("zhocn-2.1.7.1/msg/zhocn/item_dlc01.msgbnd.dcx", b"item"),
                ("zhocn-2.1.7.1/msg/zhocn/menu_dlc01.msgbnd.dcx", b"menu"),
            ],
        );
        let mut archive = ZipArchive::new(File::open(&archive_path).unwrap()).unwrap();

        let wrapper = detect_single_zip_root(&mut archive).unwrap();

        assert_eq!(wrapper, Some(PathBuf::from("zhocn-2.1.7.1")));
        drop(archive);
        let output = root.join("output");
        fs::create_dir_all(&output).unwrap();
        extract_zip(&archive_path, &output).unwrap();
        assert!(output
            .join("msg")
            .join("zhocn")
            .join("menu_dlc01.msgbnd.dcx")
            .is_file());
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn normalizes_flat_zhocn_archive_layout() {
        let root = std::env::temp_dir().join(format!(
            "nightreign_mod_manager_flat_zhocn_test_{}",
            current_timestamp()
        ));
        let source = root.join("zhocn");
        fs::create_dir_all(&source).unwrap();
        fs::write(source.join("item_dlc01.msgbnd.dcx"), b"item").unwrap();
        fs::write(source.join("menu_dlc01.msgbnd.dcx"), b"menu").unwrap();

        assert!(normalize_zhocn_layout(&root).unwrap());
        assert!(root
            .join("msg")
            .join("zhocn")
            .join("item_dlc01.msgbnd.dcx")
            .is_file());
        assert!(!source.join("item_dlc01.msgbnd.dcx").exists());
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn normalizes_root_level_zhocn_files() {
        let root = std::env::temp_dir().join(format!(
            "nightreign_mod_manager_root_zhocn_test_{}",
            current_timestamp()
        ));
        fs::create_dir_all(&root).unwrap();
        fs::write(root.join("item_dlc01.msgbnd.dcx"), b"item").unwrap();
        fs::write(root.join("menu_dlc01.msgbnd.dcx"), b"menu").unwrap();

        assert!(normalize_zhocn_layout(&root).unwrap());
        assert!(root
            .join("msg")
            .join("zhocn")
            .join("menu_dlc01.msgbnd.dcx")
            .is_file());
        assert!(!root.join("menu_dlc01.msgbnd.dcx").exists());
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn rejects_ambiguous_zhocn_archive_layouts() {
        let root = std::env::temp_dir().join(format!(
            "nightreign_mod_manager_ambiguous_zhocn_test_{}",
            current_timestamp()
        ));
        let canonical = root.join("msg").join("zhocn");
        let flat = root.join("zhocn");
        fs::create_dir_all(&canonical).unwrap();
        fs::create_dir_all(&flat).unwrap();
        fs::write(canonical.join("item_dlc01.msgbnd.dcx"), b"item").unwrap();
        fs::write(flat.join("item_dlc01.msgbnd.dcx"), b"item").unwrap();
        fs::write(flat.join("menu_dlc01.msgbnd.dcx"), b"menu").unwrap();

        let error = normalize_zhocn_layout(&root).unwrap_err();

        assert!(error.contains("多套或不完整"));
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn mmv_profile_preserves_launch_semantics_and_normalizes_game_id() {
        let root = std::env::temp_dir().join(format!(
            "nightreign_mod_manager_mmv_test_{}",
            current_timestamp()
        ));
        let mod_dir = root.join("MMV");
        let package_dir = mod_dir.join("mod");
        let redirector = package_dir
            .join("Server Redirector")
            .join("cl_server_redirector.dll");
        fs::create_dir_all(redirector.parent().unwrap()).unwrap();
        fs::write(&redirector, b"test").unwrap();

        let content = r#"
profileVersion = "v1"
savefile = "NR0000.co2"
start_online = true
future_option = "preserve-me"

[[supports]]
game = "nightrein"
future_support_option = true

[[packages]]
path = "mod"
source = "mmv-package"

[[natives]]
path = "mod/Server Redirector/cl_server_redirector.dll"
load_early = true
source = "mmv-native"
"#;

        let (packages, natives, metadata) = parse_me3_document(&mod_dir, content).unwrap();
        let generated = build_me3_profile(&metadata, &packages, &natives).unwrap();
        let value = generated.parse::<toml::Value>().unwrap();
        let _ = fs::remove_dir_all(&root);

        assert_eq!(
            value.get("savefile").and_then(toml::Value::as_str),
            Some("NR0000.co2")
        );
        assert_eq!(
            value.get("start_online").and_then(toml::Value::as_bool),
            Some(true)
        );
        assert_eq!(
            value.get("future_option").and_then(toml::Value::as_str),
            Some("preserve-me")
        );
        let support = value
            .get("supports")
            .and_then(toml::Value::as_array)
            .and_then(|items| items.first())
            .unwrap();
        assert_eq!(
            support.get("game").and_then(toml::Value::as_str),
            Some("nightreign")
        );
        assert_eq!(
            support
                .get("future_support_option")
                .and_then(toml::Value::as_bool),
            Some(true)
        );
        assert_eq!(
            value
                .get("packages")
                .and_then(toml::Value::as_array)
                .and_then(|items| items.first())
                .and_then(|entry| entry.get("source"))
                .and_then(toml::Value::as_str),
            Some("mmv-package")
        );
        assert_eq!(
            value
                .get("natives")
                .and_then(toml::Value::as_array)
                .and_then(|items| items.first())
                .and_then(|entry| entry.get("source"))
                .and_then(toml::Value::as_str),
            Some("mmv-native")
        );
        assert_eq!(
            detect_network_backend(&natives).unwrap(),
            NetworkBackend::ServerRedirector
        );
    }

    #[test]
    fn mmv_community_override_only_changes_generated_profile_data() {
        let root = std::env::temp_dir().join(format!(
            "nightreign_mod_manager_mmv_seamless_test_{}",
            current_timestamp()
        ));
        let mod_dir = root.join("MMV");
        let package_dir = mod_dir.join("mod");
        let redirector = package_dir
            .join("Server Redirector")
            .join("cl_server_redirector.dll");
        let game_dir = root.join("Game");
        let nrsc = game_dir.join("SeamlessCoop").join("nrsc.dll");
        fs::create_dir_all(redirector.parent().unwrap()).unwrap();
        fs::create_dir_all(nrsc.parent().unwrap()).unwrap();
        fs::write(&redirector, b"author redirector").unwrap();
        fs::write(package_dir.join("regulation.bin"), b"merged regulation").unwrap();
        fs::write(&nrsc, b"seamless").unwrap();

        let author_profile = r#"
profileVersion = "v1"
savefile = "NR0000.co2"
start_online = true

[[supports]]
game = "nightrein"

[[packages]]
path = "mod"

[[natives]]
path = "mod/Server Redirector/cl_server_redirector.dll"
load_early = true
"#;
        let profile_path = mod_dir.join("MMV.me3");
        fs::write(&profile_path, author_profile).unwrap();

        let (packages, mut natives, mut metadata) = collect_profile_data_for_mod(&mod_dir).unwrap();
        let config = AppConfig {
            game_path: game_dir.to_string_lossy().to_string(),
            me3_path: String::new(),
            launch_exe_path: String::new(),
            runtime_environment: RuntimeEnvironment::SpacewarSeamless,
        };

        apply_mmv_seamless_community_override(
            &mod_dir,
            &packages,
            &mut natives,
            &mut metadata,
            &config,
        )
        .unwrap();
        let mut seen = BTreeSet::new();
        extend_unique_natives(&mut natives, &mut seen, infer_game_root_natives(&game_dir));
        let generated = build_me3_profile(&metadata, &packages, &natives).unwrap();

        assert_eq!(
            detect_network_backend(&natives).unwrap(),
            NetworkBackend::Seamless
        );
        assert!(generated.contains("nrsc.dll"));
        assert!(!generated.contains("cl_server_redirector.dll"));
        assert_eq!(fs::read_to_string(&profile_path).unwrap(), author_profile);
        assert_eq!(fs::read(&redirector).unwrap(), b"author redirector");
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn mmv_community_candidate_requires_one_regulation_owner() {
        let root = std::env::temp_dir().join(format!(
            "nightreign_mod_manager_regulation_owner_test_{}",
            current_timestamp()
        ));
        let mod_dir = root.join("MMV");
        for package_name in ["map", "weapons"] {
            let package = mod_dir.join(package_name);
            fs::create_dir_all(&package).unwrap();
            fs::write(package.join("regulation.bin"), package_name.as_bytes()).unwrap();
        }
        let redirector = mod_dir
            .join("map")
            .join("Server Redirector")
            .join("cl_server_redirector.dll");
        fs::create_dir_all(redirector.parent().unwrap()).unwrap();
        fs::write(&redirector, b"redirector").unwrap();
        fs::write(
            mod_dir.join("MMV.me3"),
            r#"
profileVersion = "v1"
[[supports]]
game = "nightreign"
[[packages]]
path = "map"
[[packages]]
path = "weapons"
[[natives]]
path = "map/Server Redirector/cl_server_redirector.dll"
"#,
        )
        .unwrap();

        let error = validate_mmv_seamless_candidate(&mod_dir).unwrap_err();

        assert!(error.contains("恰好一个玩法数据文件"));
        assert!(error.contains("regulation.bin"));
        assert!(error.contains('2'));
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn detects_only_complete_zhocn_package_layouts() {
        let root = std::env::temp_dir().join(format!(
            "nightreign_mod_manager_zhocn_test_{}",
            current_timestamp()
        ));
        let complete = root.join("complete");
        let incomplete = root.join("incomplete");
        let legacy = root.join("legacy");
        let zhocn = complete.join("msg").join("zhocn");
        fs::create_dir_all(&zhocn).unwrap();
        fs::create_dir_all(incomplete.join("msg").join("zhocn")).unwrap();
        fs::create_dir_all(&legacy).unwrap();
        fs::write(zhocn.join("item_dlc01.msgbnd.dcx"), b"item").unwrap();
        fs::write(zhocn.join("menu_dlc01.msgbnd.dcx"), b"menu").unwrap();
        fs::write(
            incomplete
                .join("msg")
                .join("zhocn")
                .join("item_dlc01.msgbnd.dcx"),
            b"item",
        )
        .unwrap();
        fs::write(legacy.join("item_dlc01.msgbnd.dcx"), b"legacy item").unwrap();
        fs::write(legacy.join("menu_dlc01.msgbnd.dcx"), b"legacy menu").unwrap();
        let packages = [&complete, &incomplete, &legacy]
            .into_iter()
            .map(|path| PackageEntry {
                path: path.to_path_buf(),
                fields: default_package_fields(),
            })
            .collect::<Vec<_>>();

        let detected = collect_zhocn_packages(&packages);

        assert_eq!(detected, vec![complete]);
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn sha256_file_returns_stable_uppercase_fingerprint() {
        let path = std::env::temp_dir().join(format!(
            "nightreign_mod_manager_sha256_test_{}",
            current_timestamp()
        ));
        fs::write(&path, b"abc").unwrap();

        let fingerprint = sha256_file(&path).unwrap();

        assert_eq!(
            fingerprint,
            "BA7816BF8F01CFEA414140DE5DAE2223B00361A396177A9CB410FF61F20015AD"
        );
        let _ = fs::remove_file(path);
    }

    #[test]
    fn package_tree_fingerprint_is_stable_and_detects_content_changes() {
        let root = std::env::temp_dir().join(format!(
            "nightreign_manifest_tree_test_{}",
            current_timestamp()
        ));
        fs::create_dir_all(root.join("map")).unwrap();
        fs::write(root.join("regulation.bin"), b"regulation").unwrap();
        fs::write(root.join("map").join("asset.dcx"), b"asset-v1").unwrap();

        let first = fingerprint_package_tree(&root).unwrap();
        let second = fingerprint_package_tree(&root).unwrap();
        fs::write(root.join("map").join("asset.dcx"), b"asset-v2").unwrap();
        let changed = fingerprint_package_tree(&root).unwrap();

        assert_eq!(first, second);
        assert_eq!(first.0, 2);
        assert_ne!(first.2, changed.2);
        let _ = fs::remove_dir_all(root);
    }

    fn sample_multiplayer_manifest() -> MultiplayerManifest {
        let mut manifest = MultiplayerManifest {
            schema_version: 1,
            generated_at: "1".to_string(),
            manager_version: "0.1.0".to_string(),
            runtime_environment: "spacewar_seamless".to_string(),
            network_backend: "seamless".to_string(),
            packages: vec![MultiplayerPackageFingerprint {
                order: 1,
                name: "MMV and Weapons".to_string(),
                file_count: 2,
                total_bytes: 12,
                tree_sha256: "PACKAGE".to_string(),
                regulation_sha256: Some("REGULATION".to_string()),
                zhocn_item_sha256: None,
                zhocn_menu_sha256: None,
            }],
            natives: vec![MultiplayerNativeFingerprint {
                order: 1,
                name: "nrsc.dll".to_string(),
                size: 10,
                sha256: "NRSC".to_string(),
                load_early: true,
            }],
            runtime_files: vec![MultiplayerFileFingerprint {
                name: "nightreign.exe".to_string(),
                size: 20,
                sha256: "GAME".to_string(),
            }],
            seamless_settings_sha256: Some("SETTINGS".to_string()),
            overall_sha256: String::new(),
            warnings: Vec::new(),
        };
        manifest.overall_sha256 = calculate_multiplayer_manifest_sha256(&manifest).unwrap();
        manifest
    }

    #[test]
    fn multiplayer_manifest_comparison_detects_package_mismatch() {
        let local = sample_multiplayer_manifest();
        let identical = compare_multiplayer_manifests(local.clone(), local.clone());
        assert!(identical.compatible);

        let mut renamed = local.clone();
        renamed.packages[0].name = "好友自定义目录名".to_string();
        renamed.overall_sha256 = calculate_multiplayer_manifest_sha256(&renamed).unwrap();
        assert_eq!(local.overall_sha256, renamed.overall_sha256);
        let renamed_comparison = compare_multiplayer_manifests(local.clone(), renamed);
        assert!(renamed_comparison.compatible);
        assert!(renamed_comparison
            .differences
            .iter()
            .any(|difference| difference.severity == "warning"));

        let mut peer = local.clone();
        peer.packages[0].tree_sha256 = "DIFFERENT".to_string();
        peer.overall_sha256 = calculate_multiplayer_manifest_sha256(&peer).unwrap();
        let comparison = compare_multiplayer_manifests(local, peer);

        assert!(!comparison.compatible);
        assert!(comparison
            .differences
            .iter()
            .any(|difference| difference.category == "packages"));
    }

    #[test]
    fn multiplayer_manifest_loader_rejects_tampered_content() {
        let path = std::env::temp_dir().join(format!(
            "nightreign_manifest_tamper_test_{}.json",
            current_timestamp()
        ));
        let manifest = sample_multiplayer_manifest();
        let mut value = serde_json::to_value(&manifest).unwrap();
        value["runtimeEnvironment"] = serde_json::Value::String("steam_official".to_string());
        fs::write(&path, serde_json::to_string_pretty(&value).unwrap()).unwrap();

        let error = load_multiplayer_manifest(&path).unwrap_err();

        assert!(error.contains("总体指纹与内容不一致"));
        let _ = fs::remove_file(path);
    }

    #[test]
    fn multiplayer_manifest_does_not_serialize_absolute_paths() {
        let root = std::env::temp_dir().join(format!(
            "nightreign_manifest_redaction_test_{}",
            current_timestamp()
        ));
        let game_dir = root.join("PrivateUser").join("Game");
        let package = root.join("PrivateUser").join("Mods").join("TestPack");
        let native = game_dir.join("SeamlessCoop").join("nrsc.dll");
        fs::create_dir_all(package.join("msg").join("zhocn")).unwrap();
        fs::create_dir_all(native.parent().unwrap()).unwrap();
        fs::write(game_dir.join("nightreign.exe"), b"game").unwrap();
        fs::write(package.join("regulation.bin"), b"regulation").unwrap();
        fs::write(
            package
                .join("msg")
                .join("zhocn")
                .join("item_dlc01.msgbnd.dcx"),
            b"item",
        )
        .unwrap();
        fs::write(
            package
                .join("msg")
                .join("zhocn")
                .join("menu_dlc01.msgbnd.dcx"),
            b"menu",
        )
        .unwrap();
        fs::write(&native, b"nrsc").unwrap();
        fs::write(
            game_dir.join("SeamlessCoop").join("nrsc_settings.ini"),
            b"save_file_extension = co2",
        )
        .unwrap();
        let plan = GeneratedProfilePlan {
            content: String::new(),
            network_backend: NetworkBackend::Seamless,
            author_profile_sources: Vec::new(),
            savefile: None,
            start_online: Some(true),
            selected_mod_count: 1,
            package_count: 1,
            native_count: 1,
            mmv_seamless_community_count: 1,
            regulation_files: vec![package.join("regulation.bin")],
            zhocn_packages: vec![package.clone()],
            packages: vec![PackageEntry {
                path: package,
                fields: default_package_fields(),
            }],
            natives: vec![NativeEntry {
                path: native,
                load_early: true,
                fields: default_native_fields(true),
            }],
        };
        let config = AppConfig {
            game_path: game_dir.to_string_lossy().to_string(),
            me3_path: String::new(),
            launch_exe_path: String::new(),
            runtime_environment: RuntimeEnvironment::SpacewarSeamless,
        };

        let manifest = build_multiplayer_manifest_from_plan(&plan, &config).unwrap();
        let json = serde_json::to_string(&manifest).unwrap();

        assert!(!json.contains("PrivateUser"));
        assert!(!json.contains(&root.to_string_lossy().to_string()));
        assert_eq!(manifest.packages[0].name, "TestPack");
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn rejects_server_redirector_and_seamless_in_same_profile() {
        let natives = vec![
            NativeEntry {
                path: PathBuf::from(r"D:\Mods\MMV\cl_server_redirector.dll"),
                load_early: true,
                fields: default_native_fields(true),
            },
            NativeEntry {
                path: PathBuf::from(r"D:\Game\SeamlessCoop\nrsc.dll"),
                load_early: true,
                fields: default_native_fields(true),
            },
        ];

        let error = detect_network_backend(&natives).unwrap_err();

        assert!(error.contains("Server Redirector"));
        assert!(error.contains("SeamlessCoop"));
    }

    #[test]
    fn server_redirector_launch_keeps_steam_initialization() {
        let args = build_launch_args(
            Path::new(r"C:\Profiles\active-nightreign.me3"),
            Path::new(r"D:\Steam\Nightreign\Game\nightreign.exe"),
            NetworkBackend::ServerRedirector,
            RuntimeEnvironment::SteamOfficial,
        );

        assert!(!args.iter().any(|arg| arg == "--skip-steam-init"));
        assert!(args.iter().any(|arg| arg == "--online"));
    }

    #[test]
    fn seamless_launch_preserves_spacewar_steam_bypass() {
        let args = build_launch_args(
            Path::new(r"C:\Profiles\active-nightreign.me3"),
            Path::new(r"D:\Game\Nightreign\Game\nightreign.exe"),
            NetworkBackend::Seamless,
            RuntimeEnvironment::SpacewarSeamless,
        );

        assert!(args.iter().any(|arg| arg == "--skip-steam-init"));
        assert!(args.iter().any(|arg| arg == "--online"));
    }

    #[test]
    fn official_launch_modes_keep_me3_matchmaking_protection() {
        for (backend, environment) in [
            (NetworkBackend::None, RuntimeEnvironment::SteamOfficial),
            (NetworkBackend::Seamless, RuntimeEnvironment::SteamSeamless),
        ] {
            let args = build_launch_args(
                Path::new(r"C:\Profiles\active-nightreign.me3"),
                Path::new(r"D:\Steam\Nightreign\Game\nightreign.exe"),
                backend,
                environment,
            );
            assert!(!args.iter().any(|arg| arg == "--skip-steam-init"));
            assert!(!args.iter().any(|arg| arg == "--online"));
        }
    }

    #[test]
    fn nighter_alone_is_not_a_seamless_backend() {
        let natives = vec![NativeEntry {
            path: PathBuf::from(r"D:\Game\mods\nighter.dll"),
            load_early: false,
            fields: default_native_fields(false),
        }];

        assert_eq!(
            detect_network_backend(&natives).unwrap(),
            NetworkBackend::None
        );
    }

    #[test]
    fn official_mods_get_an_isolated_savefile_by_default() {
        let mut metadata = AuthorProfileMetadata::default();
        apply_safe_default_savefile(
            &mut metadata,
            RuntimeEnvironment::SteamOfficial,
            NetworkBackend::None,
            1,
        );

        assert_eq!(
            metadata
                .root_fields
                .get("savefile")
                .and_then(toml::Value::as_str),
            Some(OFFICIAL_MOD_SAVEFILE)
        );
    }

    #[test]
    fn seamless_backup_uses_nrsc_save_even_when_profile_declares_another_file() {
        let root = std::env::temp_dir().join(format!(
            "nightreign_mod_manager_seamless_save_test_{}",
            current_timestamp()
        ));
        let settings_dir = root.join("SeamlessCoop");
        fs::create_dir_all(&settings_dir).unwrap();
        fs::write(
            settings_dir.join("nrsc_settings.ini"),
            b"save_file_extension = co2",
        )
        .unwrap();
        let plan = GeneratedProfilePlan {
            content: String::new(),
            network_backend: NetworkBackend::Seamless,
            author_profile_sources: Vec::new(),
            savefile: Some("NR0000.codex-test".to_string()),
            start_online: Some(true),
            selected_mod_count: 1,
            package_count: 0,
            native_count: 1,
            mmv_seamless_community_count: 0,
            regulation_files: Vec::new(),
            zhocn_packages: Vec::new(),
            packages: Vec::new(),
            natives: Vec::new(),
        };

        let actual =
            effective_save_filename(&plan, RuntimeEnvironment::SpacewarSeamless, &root).unwrap();

        assert_eq!(actual, "NR0000.co2");
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn gameplay_save_affinity_warns_when_regulation_changes() {
        let previous = GameplayLaunchRecord {
            savefile: "NR0000.co2".to_string(),
            regulation_sha256: Some("OLD".to_string()),
            runtime_environment: "spacewar_seamless".to_string(),
            selected_mod_count: 3,
            recorded_at: "1".to_string(),
        };

        let (status, message) =
            assess_gameplay_save_compatibility("NR0000.co2", 1, Some("NEW"), Some(&previous));

        assert_eq!(status, "warning");
        assert!(message.contains("人物或武器不显示"));
        assert!(message.contains("启动配置中的 savefile 不能隔离 Seamless 存档"));
    }

    #[test]
    fn patch_backup_restores_overwritten_and_removes_new_files() {
        let root = std::env::temp_dir().join(format!(
            "nightreign_mod_manager_patch_backup_test_{}",
            current_timestamp()
        ));
        let game_dir = root.join("Game");
        let backup_root = root.join("backups");
        fs::create_dir_all(game_dir.join("SeamlessCoop")).unwrap();
        fs::write(game_dir.join("steam_api64.dll"), b"original").unwrap();

        let backup_dir = create_patch_backup_in(&game_dir, &backup_root).unwrap();
        fs::write(game_dir.join("steam_api64.dll"), b"patched").unwrap();
        fs::write(game_dir.join("OnlineFix64.dll"), b"new").unwrap();

        restore_patch_backup(&backup_dir, &game_dir).unwrap();

        assert_eq!(
            fs::read(game_dir.join("steam_api64.dll")).unwrap(),
            b"original"
        );
        assert!(!game_dir.join("OnlineFix64.dll").exists());
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn parses_me3_semantic_version() {
        assert_eq!(parse_semantic_version("me3-v0.12.1"), Some((0, 12, 1)));
        assert_eq!(parse_semantic_version("not-a-version"), None);
    }

    #[test]
    fn detects_spacewar_only_with_strong_marker_and_seamless() {
        let root = std::env::temp_dir().join(format!(
            "nightreign_mod_manager_spacewar_detection_test_{}",
            current_timestamp()
        ));
        let game_dir = root.join("Game");
        fs::create_dir_all(game_dir.join("SeamlessCoop")).unwrap();
        fs::write(game_dir.join("SeamlessCoop").join("nrsc.dll"), b"dll").unwrap();
        fs::write(
            game_dir.join("SeamlessCoop").join("nrsc_settings.ini"),
            b"save_file_extension = co2",
        )
        .unwrap();
        fs::write(game_dir.join("OnlineFix64.dll"), b"dll").unwrap();
        let config = AppConfig {
            game_path: game_dir.to_string_lossy().to_string(),
            me3_path: String::new(),
            launch_exe_path: String::new(),
            runtime_environment: RuntimeEnvironment::Auto,
        };

        let status = build_runtime_environment_status(&config);

        assert_eq!(status.detected, "spacewar_seamless");
        assert_eq!(status.effective, "spacewar_seamless");
        assert!(status.verified);
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn seamless_without_steam_manifest_is_not_assumed_official() {
        let root = std::env::temp_dir().join(format!(
            "nightreign_mod_manager_unverified_seamless_test_{}",
            current_timestamp()
        ));
        let game_dir = root.join("Game");
        fs::create_dir_all(game_dir.join("SeamlessCoop")).unwrap();
        fs::write(game_dir.join("SeamlessCoop").join("nrsc.dll"), b"dll").unwrap();
        fs::write(
            game_dir.join("SeamlessCoop").join("nrsc_settings.ini"),
            b"save_file_extension = co2",
        )
        .unwrap();
        let config = AppConfig {
            game_path: game_dir.to_string_lossy().to_string(),
            me3_path: String::new(),
            launch_exe_path: String::new(),
            runtime_environment: RuntimeEnvironment::Auto,
        };

        let status = build_runtime_environment_status(&config);

        assert_eq!(status.detected, "unknown_mixed");
        assert_eq!(status.effective, "unknown_mixed");
        assert!(!status.verified);
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn server_redirector_detects_partial_onlinefix_and_steam_emulator_files() {
        let root = std::env::temp_dir().join(format!(
            "nightreign_mod_manager_redirector_conflict_test_{}",
            current_timestamp()
        ));
        fs::create_dir_all(&root).unwrap();
        fs::write(root.join("OnlineFix64.dll"), b"test").unwrap();
        fs::write(root.join("steam_emu.ini"), b"test").unwrap();

        let conflicts = server_redirector_conflict_files(&root);
        let _ = fs::remove_dir_all(&root);

        assert_eq!(
            conflicts,
            vec!["OnlineFix64.dll".to_string(), "steam_emu.ini".to_string()]
        );
    }

    #[test]
    #[ignore = "requires NIGHTREIGN_MMV_TEST_DIR to point to a downloaded MMV pack"]
    fn verifies_downloaded_mmv_author_profile() {
        let mod_dir = std::env::var("NIGHTREIGN_MMV_TEST_DIR")
            .map(PathBuf::from)
            .expect("set NIGHTREIGN_MMV_TEST_DIR");
        let (packages, natives, metadata) = collect_profile_data_for_mod(&mod_dir).unwrap();
        let generated = build_me3_profile(&metadata, &packages, &natives).unwrap();
        let value = generated.parse::<toml::Value>().unwrap();

        assert!(!packages.is_empty());
        assert_eq!(
            detect_network_backend(&natives).unwrap(),
            NetworkBackend::ServerRedirector
        );
        assert_eq!(
            value.get("savefile").and_then(toml::Value::as_str),
            Some("NR0000.co2")
        );
        assert_eq!(
            value.get("start_online").and_then(toml::Value::as_bool),
            Some(true)
        );
        assert_eq!(
            value
                .get("supports")
                .and_then(toml::Value::as_array)
                .and_then(|items| items.first())
                .and_then(|entry| entry.get("game"))
                .and_then(toml::Value::as_str),
            Some("nightreign")
        );
        assert!(!natives.iter().any(|native| {
            network_backend_for_native_path(&native.path) == NetworkBackend::Seamless
        }));
    }

    #[test]
    #[ignore = "requires NIGHTREIGN_MMV_TEST_DIR and NIGHTREIGN_GAME_TEST_DIR"]
    fn verifies_downloaded_mmv_community_override() {
        let mod_dir = std::env::var("NIGHTREIGN_MMV_TEST_DIR")
            .map(PathBuf::from)
            .expect("set NIGHTREIGN_MMV_TEST_DIR");
        let game_dir = std::env::var("NIGHTREIGN_GAME_TEST_DIR")
            .map(PathBuf::from)
            .expect("set NIGHTREIGN_GAME_TEST_DIR");
        let (packages, mut natives, mut metadata) = collect_profile_data_for_mod(&mod_dir).unwrap();
        let config = AppConfig {
            game_path: game_dir.to_string_lossy().to_string(),
            me3_path: String::new(),
            launch_exe_path: String::new(),
            runtime_environment: RuntimeEnvironment::SpacewarSeamless,
        };

        apply_mmv_seamless_community_override(
            &mod_dir,
            &packages,
            &mut natives,
            &mut metadata,
            &config,
        )
        .unwrap();
        let mut seen = BTreeSet::new();
        extend_unique_natives(&mut natives, &mut seen, infer_game_root_natives(&game_dir));
        let generated = build_me3_profile(&metadata, &packages, &natives).unwrap();

        assert_eq!(collect_regulation_files(&packages).len(), 1);
        assert_eq!(
            detect_network_backend(&natives).unwrap(),
            NetworkBackend::Seamless
        );
        assert!(!generated.contains("cl_server_redirector.dll"));
        assert!(generated.contains("nrsc.dll"));
    }

    #[test]
    #[ignore = "requires a local workspace with MMV enabled"]
    fn verifies_current_workspace_uses_mmv_without_seamless_injection() {
        assert_eq!(
            std::env::var("NIGHTREIGN_VERIFY_CURRENT_WORKSPACE").as_deref(),
            Ok("1")
        );
        let mods = collect_mods().unwrap();
        let mmv = mods
            .iter()
            .find(|mod_info| mod_info.network_backend == "server_redirector")
            .expect("enabled MMV should be detected");
        assert!(mmv.author_profile);
        assert_eq!(mmv.savefile, "NR0000.co2");
        assert_eq!(mmv.start_online, Some(true));

        let plan = build_generated_profile_plan().unwrap();
        let value = plan.content.parse::<toml::Value>().unwrap();
        let native_paths = value
            .get("natives")
            .and_then(toml::Value::as_array)
            .unwrap()
            .iter()
            .filter_map(|entry| entry.get("path").and_then(toml::Value::as_str))
            .collect::<Vec<_>>();

        assert_eq!(plan.network_backend, NetworkBackend::ServerRedirector);
        assert_eq!(plan.savefile.as_deref(), Some("NR0000.co2"));
        assert_eq!(plan.start_online, Some(true));
        assert!(native_paths
            .iter()
            .any(|path| { path.ends_with(r"Server Redirector\cl_server_redirector.dll") }));
        assert!(!native_paths.iter().any(|path| {
            path.ends_with(r"SeamlessCoop\nrsc.dll") || path.ends_with(r"SeamlessCoop\nighter.dll")
        }));

        let profile_path = PathBuf::from(generate_me3_profile().unwrap());
        let written = fs::read_to_string(profile_path).unwrap();
        assert_eq!(written, plan.content);

        let preflight = build_launch_preflight().unwrap();
        assert!(preflight.checks.iter().any(|check| {
            check.id == "network_backend"
                && check.status == "pass"
                && check.message.contains("Server Redirector")
        }));
        assert!(preflight.checks.iter().any(|check| {
            check.id == "author_profile"
                && check.status == "pass"
                && check.message.contains("NR0000.co2")
        }));
        assert!(!preflight.ready);
        assert!(preflight.checks.iter().any(|check| {
            check.id == "runtime_environment"
                && check.status == "error"
                && check.message.contains("Spacewar")
                && check.message.contains("Server Redirector")
        }));
        assert!(preflight.checks.iter().any(|check| {
            check.id == "onlinefix"
                && check.status == "error"
                && check.message.contains("OnlineFix64.dll")
                && check.message.contains("steam_emu.ini")
        }));
    }

    #[test]
    fn managed_mod_path_must_be_a_direct_child() {
        let root = std::env::temp_dir().join(format!(
            "nightreign_mod_manager_path_test_{}",
            current_timestamp()
        ));
        let direct_mod = root.join("direct-mod");
        let nested_mod = direct_mod.join("nested-mod");
        fs::create_dir_all(&nested_mod).unwrap();

        assert_eq!(
            validate_direct_child(&root, &direct_mod).unwrap(),
            fs::canonicalize(&direct_mod).unwrap()
        );
        assert!(validate_direct_child(&root, &nested_mod).is_err());

        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn conflict_index_keeps_only_compact_owner_indexes() {
        let mut first_owners = HashMap::new();
        let mut conflicts = BTreeMap::new();
        let mut scanned = 0;

        record_conflict_candidate(
            "parts/example.dcx".to_string(),
            0,
            &mut first_owners,
            &mut conflicts,
            &mut scanned,
        )
        .unwrap();
        record_conflict_candidate(
            "parts/example.dcx".to_string(),
            1,
            &mut first_owners,
            &mut conflicts,
            &mut scanned,
        )
        .unwrap();

        assert_eq!(first_owners.len(), 1);
        assert_eq!(conflicts["parts/example.dcx"], BTreeSet::from([0, 1]));
    }

    #[test]
    fn launch_validation_rejects_multiple_regulation_owners() {
        let plan = GeneratedProfilePlan {
            content: String::new(),
            network_backend: NetworkBackend::None,
            author_profile_sources: Vec::new(),
            savefile: None,
            start_online: None,
            selected_mod_count: 2,
            package_count: 2,
            native_count: 0,
            mmv_seamless_community_count: 0,
            regulation_files: vec![
                PathBuf::from(r"C:\Mods\SkinOverhaul\regulation.bin"),
                PathBuf::from(r"C:\Mods\MMV\mod\regulation.bin"),
            ],
            zhocn_packages: Vec::new(),
            packages: Vec::new(),
            natives: Vec::new(),
        };

        let error = validate_single_regulation_owner(&plan).unwrap_err();
        assert!(error.contains("SkinOverhaul"));
        assert!(error.contains("MMV"));
        assert!(error.contains("不能靠加载顺序自动合并"));
    }

    #[test]
    fn launch_log_reader_only_keeps_the_requested_tail() {
        let log_path = std::env::temp_dir().join(format!(
            "nightreign_mod_manager_log_test_{}.log",
            current_timestamp()
        ));
        let content = (0..100)
            .map(|index| format!("launch-line-{index:03}\n"))
            .collect::<String>();
        fs::write(&log_path, content).unwrap();

        let tail = read_text_tail(&log_path, 96).unwrap();
        let _ = fs::remove_file(&log_path);

        assert!(tail.starts_with("[日志已截断"));
        assert!(tail.contains("launch-line-099"));
        assert!(!tail.contains("launch-line-000"));
        assert!(tail.len() < 256);
    }

    #[test]
    fn tasklist_parser_matches_only_exact_guarded_process_names() {
        let output = concat!(
            "\"nightreign-mod-manager.exe\",\"100\",\"Console\",\"1\",\"10,000 K\"\n",
            "\"nightreign.exe\",\"200\",\"Console\",\"1\",\"500,000 K\"\n",
            "\"me3-launcher.exe\",\"300\",\"Console\",\"1\",\"20,000 K\"\n",
            "\"steam.exe\",\"350\",\"Console\",\"1\",\"100,000 K\"\n",
            "\"unrelated.exe\",\"400\",\"Console\",\"1\",\"5,000 K\"\n",
        );

        assert_eq!(
            parse_tasklist_game_processes(output),
            vec![
                "nightreign.exe (PID 200)".to_string(),
                "me3-launcher.exe (PID 300)".to_string(),
            ]
        );
        assert!(parse_tasklist_processes(output)
            .iter()
            .any(|process| process.name.eq_ignore_ascii_case("steam.exe")));
    }

    #[test]
    #[ignore = "requires NIGHTREIGN_ZHOCN_ZIP to point to a downloaded translation archive"]
    fn installs_downloaded_zhocn_sample_into_configured_game() {
        let archive = std::env::var("NIGHTREIGN_ZHOCN_ZIP")
            .map(PathBuf::from)
            .expect("set NIGHTREIGN_ZHOCN_ZIP");
        assert_eq!(
            sha256_file(&archive).unwrap(),
            "3166254E167551F8F8B85B2897070371D6D58C57407687F4607CC91493946B58"
        );

        let installed = install_mod_from_zip_blocking(&archive.to_string_lossy()).unwrap();
        let installed_path = PathBuf::from(&installed.path);

        assert!(installed.zhocn_layout_normalized);
        assert!(installed_path
            .join("msg")
            .join("zhocn")
            .join("item_dlc01.msgbnd.dcx")
            .is_file());
        assert!(installed_path
            .join("msg")
            .join("zhocn")
            .join("menu_dlc01.msgbnd.dcx")
            .is_file());
        eprintln!("installed zhocn sample at {}", installed.path);
    }

    #[test]
    #[ignore = "requires NIGHTREIGN_MMV_TEST_DIR, NIGHTREIGN_ZHOCN_TEST_DIR and NIGHTREIGN_GAME_TEST_DIR"]
    fn exports_real_mmv_zhocn_multiplayer_manifest() {
        let mmv_dir = std::env::var("NIGHTREIGN_MMV_TEST_DIR")
            .map(PathBuf::from)
            .expect("set NIGHTREIGN_MMV_TEST_DIR");
        let zhocn_dir = std::env::var("NIGHTREIGN_ZHOCN_TEST_DIR")
            .map(PathBuf::from)
            .expect("set NIGHTREIGN_ZHOCN_TEST_DIR");
        let game_dir = std::env::var("NIGHTREIGN_GAME_TEST_DIR")
            .map(PathBuf::from)
            .expect("set NIGHTREIGN_GAME_TEST_DIR");
        let config = AppConfig {
            game_path: game_dir.to_string_lossy().to_string(),
            me3_path: String::new(),
            launch_exe_path: String::new(),
            runtime_environment: RuntimeEnvironment::SpacewarSeamless,
        };
        let (mut packages, mut natives, mut metadata) =
            collect_profile_data_for_mod(&mmv_dir).unwrap();
        apply_mmv_seamless_community_override(
            &mmv_dir,
            &packages,
            &mut natives,
            &mut metadata,
            &config,
        )
        .unwrap();
        let mut seen_natives = natives
            .iter()
            .map(|native| path_key(&native.path))
            .collect::<BTreeSet<_>>();
        extend_unique_natives(
            &mut natives,
            &mut seen_natives,
            infer_game_root_natives(&game_dir),
        );
        packages.push(PackageEntry {
            path: zhocn_dir,
            fields: default_package_fields(),
        });
        let regulation_files = collect_regulation_files(&packages);
        let zhocn_packages = collect_zhocn_packages(&packages);
        let content = build_me3_profile(&metadata, &packages, &natives).unwrap();
        let plan = GeneratedProfilePlan {
            content,
            network_backend: detect_network_backend(&natives).unwrap(),
            author_profile_sources: metadata.source_paths.clone(),
            savefile: metadata
                .root_fields
                .get("savefile")
                .and_then(toml::Value::as_str)
                .map(ToOwned::to_owned),
            start_online: metadata
                .root_fields
                .get("start_online")
                .and_then(toml::Value::as_bool),
            selected_mod_count: 2,
            package_count: packages.len(),
            native_count: natives.len(),
            mmv_seamless_community_count: 1,
            regulation_files,
            zhocn_packages,
            packages,
            natives,
        };

        let manifest = build_multiplayer_manifest_from_plan(&plan, &config).unwrap();

        assert_eq!(manifest.runtime_environment, "spacewar_seamless");
        assert_eq!(manifest.network_backend, "seamless");
        assert_eq!(manifest.packages.len(), 2);
        assert_eq!(
            manifest.packages[0].regulation_sha256.as_deref(),
            Some("D36B9960E19C748112F2A8D0D4C00D33A2BEC8AE9BB1707975516C3DBB64F579")
        );
        assert_eq!(
            manifest.packages[1].zhocn_item_sha256.as_deref(),
            Some("A1FF17385256E7AAD60F88F74D9292D4C11B75541EF6019BB4AB5085F52B8BC6")
        );
        assert_eq!(
            manifest.packages[1].zhocn_menu_sha256.as_deref(),
            Some("F6EEA4A210CBC6E3314998FA9001C487CEAD87081530152011B7B3BC060201D7")
        );
        assert!(manifest.natives.iter().any(|native| {
            native.name.eq_ignore_ascii_case("nrsc.dll")
                && native.load_early
                && native.sha256
                    == "243EEC929A97B71E1E2E3B4215778B89C37D629436B8DD5403E830593D3CE24E"
        }));
        let json = serde_json::to_string_pretty(&manifest).unwrap();
        assert!(!json.contains(&game_dir.to_string_lossy().to_string()));
        assert!(!json.contains(&mmv_dir.to_string_lossy().to_string()));
        if let Ok(output) = std::env::var("NIGHTREIGN_MANIFEST_OUTPUT") {
            fs::write(output, json).unwrap();
        }
    }

    #[test]
    #[ignore = "requires the configured local Nightreign workspace"]
    fn exports_current_workspace_multiplayer_manifest() {
        assert_eq!(
            std::env::var("NIGHTREIGN_VERIFY_CURRENT_WORKSPACE").as_deref(),
            Ok("1")
        );
        let manifest = build_multiplayer_manifest().unwrap();
        assert!(!manifest.packages.is_empty());
        assert_eq!(manifest.network_backend, "seamless");
        assert_eq!(manifest.runtime_environment, "spacewar_seamless");
    }
}
