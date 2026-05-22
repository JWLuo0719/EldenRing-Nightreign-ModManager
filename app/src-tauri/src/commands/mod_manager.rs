use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use tauri::command;

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
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AppConfig {
    pub game_path: String,
    pub me3_path: String,
}

fn get_config_path() -> PathBuf {
    let config_dir = dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("nightreign-mod-manager");
    fs::create_dir_all(&config_dir).ok();
    config_dir.join("config.json")
}

fn load_config() -> AppConfig {
    let config_path = get_config_path();
    if config_path.exists() {
        let content = fs::read_to_string(&config_path).unwrap_or_default();
        serde_json::from_str(&content).unwrap_or(AppConfig {
            game_path: String::new(),
            me3_path: String::new(),
        })
    } else {
        AppConfig {
            game_path: String::new(),
            me3_path: String::new(),
        }
    }
}

fn save_config(config: &AppConfig) -> Result<(), String> {
    let config_path = get_config_path();
    let content = serde_json::to_string_pretty(config).map_err(|e| e.to_string())?;
    fs::write(&config_path, content).map_err(|e| e.to_string())
}

#[command]
pub fn get_game_path() -> String {
    load_config().game_path
}

#[command]
pub fn set_game_path(path: String) -> Result<(), String> {
    let mut config = load_config();
    config.game_path = path;
    save_config(&config)
}

#[command]
pub fn get_me3_path() -> String {
    load_config().me3_path
}

#[command]
pub fn set_me3_path(path: String) -> Result<(), String> {
    let mut config = load_config();
    config.me3_path = path;
    save_config(&config)
}

#[command]
pub fn scan_mods() -> Vec<ModInfo> {
    let config = load_config();
    let mods_dir = Path::new(&config.game_path).join("mods");

    if !mods_dir.exists() {
        return Vec::new();
    }

    let mut mods = Vec::new();

    if let Ok(entries) = fs::read_dir(&mods_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if let Some(mod_info) = parse_mod_folder(&path) {
                    mods.push(mod_info);
                }
            }
        }
    }

    mods
}

fn parse_mod_folder(path: &Path) -> Option<ModInfo> {
    let name = path
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    let me3_files: Vec<PathBuf> = fs::read_dir(path)
        .ok()?
        .flatten()
        .filter(|e| {
            e.path()
                .extension()
                .map_or(false, |ext| ext == "me3")
        })
        .map(|e| e.path())
        .collect();

    let (description, version, mod_type) = if let Some(me3_file) = me3_files.first() {
        let content = fs::read_to_string(me3_file).unwrap_or_default();
        parse_me3_content(&content)
    } else {
        (String::new(), String::new(), "package".to_string())
    };

    let files = fs::read_dir(path)
        .ok()?
        .flatten()
        .map(|e| {
            e.path()
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string()
        })
        .collect();

    Some(ModInfo {
        id: name.clone(),
        name: extract_display_name(&name),
        description,
        version,
        author: String::new(),
        enabled: false,
        path: path.to_string_lossy().to_string(),
        mod_type,
        files,
    })
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
        if trimmed.starts_with("[[packages]]") {
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

fn extract_display_name(folder_name: &str) -> String {
    let name = folder_name;
    let name = if let Some(end) = name.find('-') {
        let prefix = &name[..end];
        if prefix.chars().all(|c| c.is_ascii_digit()) || prefix.is_empty() {
            &name[end + 1..]
        } else {
            name
        }
    } else {
        name
    };
    name.to_string()
}

#[command]
pub fn get_mod_info(mod_path: String) -> Result<ModInfo, String> {
    let path = Path::new(&mod_path);
    parse_mod_folder(path).ok_or_else(|| "无法解析mod信息".to_string())
}

#[command]
pub fn install_mod_from_zip(zip_path: String, mods_dir: String) -> Result<String, String> {
    let zip_path = Path::new(&zip_path);
    let mods_dir = Path::new(&mods_dir);

    if !zip_path.exists() {
        return Err("ZIP文件不存在".to_string());
    }

    fs::create_dir_all(mods_dir).map_err(|e| e.to_string())?;

    let file_name = zip_path
        .file_stem()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    let extract_dir = mods_dir.join(&file_name);

    if extract_dir.exists() {
        return Err("同名mod已存在".to_string());
    }

    fs::create_dir_all(&extract_dir).map_err(|e| e.to_string())?;

    Ok(extract_dir.to_string_lossy().to_string())
}

#[command]
pub fn uninstall_mod(mod_path: String) -> Result<(), String> {
    let path = Path::new(&mod_path);
    if path.exists() {
        fs::remove_dir_all(path).map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[command]
pub fn toggle_mod(mod_path: String, enabled: bool) -> Result<(), String> {
    let path = Path::new(&mod_path);
    let disabled_path = path.with_extension("disabled");

    if enabled && disabled_path.exists() {
        fs::rename(&disabled_path, path).map_err(|e| e.to_string())?;
    } else if !enabled && path.exists() {
        fs::rename(path, &disabled_path).map_err(|e| e.to_string())?;
    }

    Ok(())
}

#[command]
pub fn launch_game(_game_path: String, me3_path: String) -> Result<(), String> {
    let me3_exe = Path::new(&me3_path).join("bin").join("me3-launcher.exe");

    if !me3_exe.exists() {
        return Err("ME3启动器未找到".to_string());
    }

    std::process::Command::new(&me3_exe)
        .current_dir(Path::new(&me3_path).join("bin"))
        .spawn()
        .map_err(|e| e.to_string())?;

    Ok(())
}
