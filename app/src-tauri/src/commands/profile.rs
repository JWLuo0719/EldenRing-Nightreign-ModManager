use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri::command;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Profile {
    pub id: String,
    pub name: String,
    pub description: String,
    pub icon: String,
    pub mods: Vec<ProfileMod>,
    #[serde(rename = "isActive", alias = "is_active")]
    pub is_active: bool,
    #[serde(rename = "createdAt", alias = "created_at")]
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProfileMod {
    #[serde(rename = "modId", alias = "mod_id")]
    pub mod_id: String,
    pub enabled: bool,
    #[serde(rename = "loadOrder", alias = "load_order")]
    pub load_order: u32,
}

fn get_profiles_dir() -> PathBuf {
    let config_dir = dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("nightreign-mod-manager");
    fs::create_dir_all(&config_dir).ok();
    config_dir.join("profiles")
}

fn get_active_profile_path() -> PathBuf {
    let config_dir = dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("nightreign-mod-manager");
    config_dir.join("active_profile.txt")
}

#[command]
pub fn get_profiles() -> Vec<Profile> {
    let profiles_dir = get_profiles_dir();
    if !profiles_dir.exists() {
        return Vec::new();
    }

    let active_id = get_active_profile_id();
    let mut profiles = Vec::new();

    if let Ok(entries) = fs::read_dir(&profiles_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map_or(false, |ext| ext == "json") {
                if let Ok(content) = fs::read_to_string(&path) {
                    if let Ok(mut profile) = serde_json::from_str::<Profile>(&content) {
                        profile.is_active = Some(&profile.id) == active_id.as_ref();
                        profiles.push(profile);
                    }
                }
            }
        }
    }

    profiles.sort_by(|a, b| a.created_at.cmp(&b.created_at));
    profiles
}

pub fn get_active_profile_id() -> Option<String> {
    let path = get_active_profile_path();
    if path.exists() {
        fs::read_to_string(path).ok().map(|s| s.trim().to_string())
    } else {
        None
    }
}

#[command]
pub fn create_profile(name: String, description: String, icon: String) -> Result<Profile, String> {
    let profiles_dir = get_profiles_dir();
    fs::create_dir_all(&profiles_dir).map_err(|e| e.to_string())?;

    let id = format!(
        "profile_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis()
    );

    let profile = Profile {
        id: id.clone(),
        name,
        description,
        icon,
        mods: Vec::new(),
        is_active: false,
        created_at: chrono_now(),
    };

    let content = serde_json::to_string_pretty(&profile).map_err(|e| e.to_string())?;
    let file_path = profiles_dir.join(format!("{}.json", id));
    fs::write(&file_path, content).map_err(|e| e.to_string())?;

    Ok(profile)
}

#[command]
pub fn delete_profile(profile_id: String) -> Result<(), String> {
    let profiles_dir = get_profiles_dir();
    let file_path = profiles_dir.join(format!("{}.json", profile_id));

    if file_path.exists() {
        fs::remove_file(&file_path).map_err(|e| e.to_string())?;
    }

    let active_id = get_active_profile_id();
    if active_id.as_deref() == Some(&profile_id) {
        let active_path = get_active_profile_path();
        if active_path.exists() {
            fs::remove_file(&active_path).ok();
        }
    }

    Ok(())
}

#[command]
pub fn activate_profile(profile_id: String) -> Result<(), String> {
    let profiles_dir = get_profiles_dir();
    let file_path = profiles_dir.join(format!("{}.json", profile_id));

    if !file_path.exists() {
        return Err("配置方案不存在".to_string());
    }

    let active_path = get_active_profile_path();
    fs::write(&active_path, &profile_id).map_err(|e| e.to_string())?;

    Ok(())
}

#[command]
pub fn get_active_profile() -> Option<Profile> {
    let active_id = get_active_profile_id()?;
    let mut profile = load_profile(&active_id).ok()?;
    profile.is_active = true;
    Some(profile)
}

#[command]
pub fn update_profile(profile: Profile) -> Result<(), String> {
    save_profile(&profile)
}

#[command]
pub fn update_active_profile_mod(mod_id: String, enabled: bool) -> Result<Option<Profile>, String> {
    let Some(mut profile) = get_active_profile() else {
        return Ok(None);
    };

    if let Some(profile_mod) = profile.mods.iter_mut().find(|item| item.mod_id == mod_id) {
        profile_mod.enabled = enabled;
    } else {
        let load_order = profile
            .mods
            .iter()
            .map(|item| item.load_order)
            .max()
            .unwrap_or(0)
            + 1;
        profile.mods.push(ProfileMod {
            mod_id,
            enabled,
            load_order,
        });
    }

    save_profile(&profile)?;
    Ok(Some(profile))
}

pub fn load_profile(profile_id: &str) -> Result<Profile, String> {
    let profiles_dir = get_profiles_dir();
    let file_path = profiles_dir.join(format!("{}.json", profile_id));

    if !file_path.exists() {
        return Err("配置方案不存在".to_string());
    }

    let content = fs::read_to_string(&file_path).map_err(|e| e.to_string())?;
    serde_json::from_str::<Profile>(&content).map_err(|e| e.to_string())
}

pub fn save_profile(profile: &Profile) -> Result<(), String> {
    let profiles_dir = get_profiles_dir();
    let file_path = profiles_dir.join(format!("{}.json", profile.id));

    if !file_path.exists() {
        return Err("配置方案不存在".to_string());
    }

    let content = serde_json::to_string_pretty(&profile).map_err(|e| e.to_string())?;
    fs::write(&file_path, content).map_err(|e| e.to_string())?;

    Ok(())
}

fn chrono_now() -> String {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
        .to_string()
}
