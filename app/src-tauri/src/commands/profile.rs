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

fn validate_profile_id(profile_id: &str) -> Result<(), String> {
    let is_valid = profile_id.starts_with("profile_")
        && profile_id.len() <= 96
        && profile_id.chars().all(|character| {
            character.is_ascii_alphanumeric() || character == '_' || character == '-'
        });
    if is_valid {
        Ok(())
    } else {
        Err("配置方案 ID 无效".to_string())
    }
}

fn get_profile_path(profile_id: &str) -> Result<PathBuf, String> {
    validate_profile_id(profile_id)?;
    Ok(get_profiles_dir().join(format!("{profile_id}.json")))
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
            if path.extension().is_some_and(|ext| ext == "json") {
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
    let file_path = get_profile_path(&profile_id)?;

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
    let file_path = get_profile_path(&profile_id)?;

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

pub fn replace_mod_id_in_all_profiles(old_mod_id: &str, new_mod_id: &str) -> Result<usize, String> {
    if old_mod_id == new_mod_id {
        return Ok(0);
    }

    let mut updated = 0;
    for mut profile in get_profiles() {
        if replace_mod_id_in_profile(&mut profile, old_mod_id, new_mod_id) {
            save_profile(&profile)?;
            updated += 1;
        }
    }
    Ok(updated)
}

fn replace_mod_id_in_profile(profile: &mut Profile, old_mod_id: &str, new_mod_id: &str) -> bool {
    let Some(old_index) = profile
        .mods
        .iter()
        .position(|item| item.mod_id == old_mod_id)
    else {
        return false;
    };

    if let Some(new_index) = profile
        .mods
        .iter()
        .position(|item| item.mod_id == new_mod_id)
    {
        let old = profile.mods[old_index].clone();
        let new = &mut profile.mods[new_index];
        new.enabled |= old.enabled;
        new.load_order = new.load_order.min(old.load_order);
        profile.mods.remove(old_index);
    } else {
        profile.mods[old_index].mod_id = new_mod_id.to_string();
    }
    true
}

pub fn load_profile(profile_id: &str) -> Result<Profile, String> {
    let file_path = get_profile_path(profile_id)?;

    if !file_path.exists() {
        return Err("配置方案不存在".to_string());
    }

    let content = fs::read_to_string(&file_path).map_err(|e| e.to_string())?;
    serde_json::from_str::<Profile>(&content).map_err(|e| e.to_string())
}

pub fn save_profile(profile: &Profile) -> Result<(), String> {
    let file_path = get_profile_path(&profile.id)?;

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

#[cfg(test)]
mod tests {
    use super::{replace_mod_id_in_profile, validate_profile_id, Profile, ProfileMod};

    #[test]
    fn accepts_generated_profile_ids() {
        assert!(validate_profile_id("profile_1779536514777").is_ok());
    }

    #[test]
    fn rejects_profile_path_traversal() {
        assert!(validate_profile_id(r"..\..\outside").is_err());
        assert!(validate_profile_id("profile_/outside").is_err());
    }

    #[test]
    fn relinking_mod_id_preserves_state_and_merges_duplicates() {
        let mut profile = Profile {
            id: "profile_1".to_string(),
            name: "测试".to_string(),
            description: String::new(),
            icon: String::new(),
            mods: vec![
                ProfileMod {
                    mod_id: "old".to_string(),
                    enabled: true,
                    load_order: 2,
                },
                ProfileMod {
                    mod_id: "new".to_string(),
                    enabled: false,
                    load_order: 5,
                },
            ],
            is_active: true,
            created_at: "1".to_string(),
        };

        assert!(replace_mod_id_in_profile(&mut profile, "old", "new"));
        assert_eq!(profile.mods.len(), 1);
        assert_eq!(profile.mods[0].mod_id, "new");
        assert!(profile.mods[0].enabled);
        assert_eq!(profile.mods[0].load_order, 2);
    }
}
