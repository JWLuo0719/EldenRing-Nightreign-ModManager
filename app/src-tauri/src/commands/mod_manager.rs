use super::profile;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::fs::{self, File};
use std::io::{self, Read};
use std::path::{Component, Path, PathBuf};
use std::process::Stdio;
use std::time::{Duration, Instant};
use tauri::command;
use zip::ZipArchive;

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
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AppConfig {
    pub game_path: String,
    pub me3_path: String,
    #[serde(default)]
    pub launch_exe_path: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct PackageEntry {
    path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct NativeEntry {
    path: PathBuf,
    load_early: bool,
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

fn load_config() -> AppConfig {
    let config_path = get_config_path();
    if config_path.exists() {
        let content = fs::read_to_string(&config_path).unwrap_or_default();
        serde_json::from_str(&content).unwrap_or(AppConfig {
            game_path: String::new(),
            me3_path: String::new(),
            launch_exe_path: String::new(),
        })
    } else {
        AppConfig {
            game_path: String::new(),
            me3_path: String::new(),
            launch_exe_path: String::new(),
        }
    }
}

fn save_config(config: &AppConfig) -> Result<(), String> {
    let config_path = get_config_path();
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
pub fn scan_mods() -> Vec<ModInfo> {
    collect_mods().unwrap_or_default()
}

fn collect_mods() -> Result<Vec<ModInfo>, String> {
    let config = load_config();
    let mods_dir = mods_dir_from_config(&config)?;

    if !mods_dir.exists() {
        return Ok(Vec::new());
    }

    let mut mods = Vec::new();

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
        }
    }

    mods.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    Ok(mods)
}

fn parse_mod_folder(path: &Path, enabled: bool) -> Option<ModInfo> {
    let folder_name = path.file_name()?.to_string_lossy().to_string();
    let id = strip_disabled_suffix(&folder_name).to_string();

    let me3_files = find_top_level_me3_files(path);
    let (description, version, mod_type) = if let Some(me3_file) = me3_files.first() {
        let content = fs::read_to_string(me3_file).unwrap_or_default();
        parse_me3_content(&content)
    } else if has_dll_file(path) && !has_package_like_content(path) {
        (String::new(), String::new(), "native".to_string())
    } else {
        (String::new(), String::new(), "package".to_string())
    };

    let files = fs::read_dir(path)
        .ok()?
        .flatten()
        .map(|e| e.file_name().to_string_lossy().to_string())
        .collect();

    Some(ModInfo {
        id: id.clone(),
        name: extract_display_name(&id),
        description,
        version,
        author: String::new(),
        enabled,
        path: path.to_string_lossy().to_string(),
        mod_type,
        files,
    })
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
                .map_or(false, |ext| ext.eq_ignore_ascii_case("me3"));
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
pub fn install_mod_from_zip(zip_path: String) -> Result<String, String> {
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

    let extract_dir = unique_destination(&mods_dir.join(sanitize_folder_name(&file_name)));
    fs::create_dir_all(&extract_dir).map_err(|e| e.to_string())?;

    extract_zip(zip_path, &extract_dir).inspect_err(|_| {
        let _ = fs::remove_dir_all(&extract_dir);
    })?;

    Ok(extract_dir.to_string_lossy().to_string())
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

        let relative_path = strip_zip_root(&enclosed_name, single_root.as_deref());
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
    let mut roots = BTreeSet::new();

    for index in 0..archive.len() {
        let zip_file = archive.by_index(index).map_err(|e| e.to_string())?;
        if let Some(path) = zip_file.enclosed_name() {
            if let Some(Component::Normal(root)) = path.components().next() {
                roots.insert(PathBuf::from(root));
            }
        }
    }

    Ok((roots.len() == 1).then(|| roots.into_iter().next().unwrap()))
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
    let path = Path::new(&mod_path);
    if path.exists() {
        fs::remove_dir_all(path).map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[command]
pub fn toggle_mod(mod_path: String, enabled: bool) -> Result<(), String> {
    let path = Path::new(&mod_path);

    if enabled {
        let source = if path.exists() {
            path.to_path_buf()
        } else {
            disabled_path_for(path)
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
        let target = disabled_path_for(path);
        if target.exists() {
            return Err("禁用失败：目标目录已存在".to_string());
        }
        fs::rename(path, &target).map_err(|e| e.to_string())?;
    }

    Ok(())
}

#[command]
pub fn generate_me3_profile() -> Result<String, String> {
    let mods = collect_mods()?;
    let selected_mods = mods_selected_for_generation(&mods);
    let mut packages = Vec::new();
    let mut natives = Vec::new();
    let mut seen_packages = BTreeSet::new();
    let mut seen_natives = BTreeSet::new();

    for mod_info in selected_mods {
        let mod_dir = Path::new(&mod_info.path);
        let (mod_packages, mod_natives) = collect_entries_for_mod(mod_dir)?;
        extend_unique(&mut packages, &mut seen_packages, mod_packages);
        extend_unique(&mut natives, &mut seen_natives, mod_natives);
    }

    let config = load_config();
    let external_natives = infer_game_root_natives(Path::new(&config.game_path));
    extend_unique(&mut natives, &mut seen_natives, external_natives);

    let profile_path = get_generated_profile_path();
    let content = build_me3_profile(&packages, &natives);
    fs::write(&profile_path, content).map_err(|e| e.to_string())?;
    Ok(profile_path.to_string_lossy().to_string())
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
            return profile_mods
                .into_iter()
                .filter_map(|profile_mod| mods.iter().find(|item| item.id == profile_mod.mod_id))
                .collect();
        }
    }

    mods.iter().filter(|item| item.enabled).collect()
}

fn extend_unique<T>(target: &mut Vec<T>, seen: &mut BTreeSet<T>, entries: Vec<T>)
where
    T: Ord + Clone,
{
    for entry in entries {
        if seen.insert(entry.clone()) {
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

    let profile_path = generate_me3_profile()?;
    let profile_path = PathBuf::from(profile_path);
    let me3_exe = find_me3_exe(Path::new(&me3_path))?;
    let launch_exe = resolve_launch_exe(&config, Path::new(&game_path))?;
    let args = build_launch_args(&profile_path, &launch_exe);
    let working_dir = me3_exe.parent().unwrap_or_else(|| Path::new(&me3_path));
    let launch_script = write_launch_script(&me3_exe, &args, working_dir)?;
    append_launch_log(&format!(
        "\n=== Launch {} ===\nscript: {}\n{}\n",
        current_timestamp(),
        launch_script.to_string_lossy(),
        format_command_line(&me3_exe, &args, working_dir)
    ));

    launch_via_script(&launch_script)?;

    Ok(format!(
        "启动脚本已执行。\n脚本：{}\n日志：{}",
        launch_script.to_string_lossy(),
        get_launch_log_path().to_string_lossy()
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

    let profile_path = generate_me3_profile()?;
    let profile_path = PathBuf::from(profile_path);
    let me3_exe = find_me3_exe(Path::new(&me3_path))?;
    let launch_exe = resolve_launch_exe(&config, Path::new(&game_path))?;
    let args = build_launch_args(&profile_path, &launch_exe);
    let working_dir = me3_exe.parent().unwrap_or_else(|| Path::new(&me3_path));
    let command_line = format_command_line(&me3_exe, &args, working_dir);

    let mut child = std::process::Command::new(&me3_exe)
        .args(&args)
        .current_dir(working_dir)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("无法启动 ME3：{e}\n\n命令：{command_line}"))?;

    let start = Instant::now();
    while start.elapsed() < Duration::from_secs(4) {
        if let Some(status) = child
            .try_wait()
            .map_err(|e| format!("检查 ME3 状态失败：{e}\n\n命令：{command_line}"))?
        {
            let stdout = read_child_pipe(child.stdout.take());
            let stderr = read_child_pipe(child.stderr.take());

            if status.success() {
                return Ok(format!("ME3 已完成启动流程。\n命令：{command_line}"));
            }

            return Err(format!(
                "ME3 启动失败，退出码：{}\n\n命令：{}\n\nstdout:\n{}\n\nstderr:\n{}",
                status
                    .code()
                    .map_or_else(|| "unknown".to_string(), |code| code.to_string()),
                command_line,
                if stdout.trim().is_empty() {
                    "(empty)"
                } else {
                    stdout.trim()
                },
                if stderr.trim().is_empty() {
                    "(empty)"
                } else {
                    stderr.trim()
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
        .arg("/K")
        .arg(script_path)
        .current_dir(script_path.parent().unwrap_or_else(|| Path::new(".")));

    #[cfg(windows)]
    command.creation_flags(0x00000010);

    command
        .spawn()
        .map_err(|e| format!("执行启动脚本失败：{e}"))?;

    Ok(())
}

fn append_launch_log(message: &str) {
    use std::io::Write;

    let log_path = get_launch_log_path();
    if let Ok(mut file) = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_path)
    {
        let _ = file.write_all(message.as_bytes());
    }
}

fn build_launch_args(profile_path: &Path, launch_exe: &Path) -> Vec<String> {
    vec![
        "launch".to_string(),
        "--exe".to_string(),
        launch_exe.to_string_lossy().to_string(),
        "--skip-steam-init".to_string(),
        "--online".to_string(),
        "--game".to_string(),
        "nightreign".to_string(),
        "-p".to_string(),
        profile_path.to_string_lossy().to_string(),
    ]
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

fn read_child_pipe<T>(pipe: Option<T>) -> String
where
    T: Read,
{
    let Some(mut pipe) = pipe else {
        return String::new();
    };

    let mut output = String::new();
    let _ = pipe.read_to_string(&mut output);
    output
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
        .map_or(false, |name| name.eq_ignore_ascii_case("nrsc_launcher.exe"))
    {
        let game_exe = game_dir.join("nightreign.exe");
        validate_launch_exe(&game_exe, game_dir)?;
        append_launch_log(
            "检测到 nrsc_launcher.exe；ME3 启动链路将改用 nightreign.exe，并通过 profile 加载 SeamlessCoop/nrsc.dll。\n",
        );
        return Ok(game_exe);
    }

    Ok(launch_exe)
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
        .map_or(false, |ext| ext.eq_ignore_ascii_case("exe"))
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

fn collect_entries_for_mod(
    mod_dir: &Path,
) -> Result<(Vec<PackageEntry>, Vec<NativeEntry>), String> {
    let mut packages = Vec::new();
    let mut natives = Vec::new();

    for me3_file in find_top_level_me3_files(mod_dir) {
        let content = fs::read_to_string(&me3_file).map_err(|e| e.to_string())?;
        let (mut mod_packages, mut mod_natives) = parse_me3_entries(mod_dir, &content)?;
        packages.append(&mut mod_packages);
        natives.append(&mut mod_natives);
    }

    if packages.is_empty() && natives.is_empty() {
        let (mut inferred_packages, mut inferred_natives) = infer_entries_for_mod(mod_dir);
        packages.append(&mut inferred_packages);
        natives.append(&mut inferred_natives);
    }

    Ok((packages, natives))
}

fn parse_me3_entries(
    mod_dir: &Path,
    content: &str,
) -> Result<(Vec<PackageEntry>, Vec<NativeEntry>), String> {
    let value: toml::Value = content
        .parse()
        .map_err(|e: toml::de::Error| e.to_string())?;
    let mut packages = Vec::new();
    let mut natives = Vec::new();

    for key in ["packages", "package"] {
        if let Some(entries) = value.get(key).and_then(|v| v.as_array()) {
            for entry in entries {
                if let Some(path) = entry.get("path").and_then(|v| v.as_str()) {
                    packages.push(PackageEntry {
                        path: resolve_mod_path(mod_dir, path),
                    });
                }
            }
        }
    }

    if let Some(entries) = value.get("natives").and_then(|v| v.as_array()) {
        for entry in entries {
            if let Some(path) = entry.get("path").and_then(|v| v.as_str()) {
                natives.push(NativeEntry {
                    path: resolve_mod_path(mod_dir, path),
                    load_early: entry
                        .get("load_early")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false),
                });
            }
        }
    }

    Ok((packages, natives))
}

fn infer_entries_for_mod(mod_dir: &Path) -> (Vec<PackageEntry>, Vec<NativeEntry>) {
    let mut packages = Vec::new();
    let natives = find_dll_files(mod_dir)
        .into_iter()
        .map(|path| NativeEntry {
            path,
            load_early: false,
        })
        .collect();

    let mod_subdir = mod_dir.join("mod");
    if mod_subdir.is_dir() {
        packages.push(PackageEntry { path: mod_subdir });
    } else if has_package_like_content(mod_dir) {
        packages.push(PackageEntry {
            path: mod_dir.to_path_buf(),
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
            path: nrsc,
            load_early: true,
        });
    }

    let nighter = seamless_dir.join("nighter.dll");
    if nighter.exists() {
        natives.push(NativeEntry {
            path: nighter,
            load_early: false,
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

    fs::canonicalize(&resolved).unwrap_or(resolved)
}

fn build_me3_profile(packages: &[PackageEntry], natives: &[NativeEntry]) -> String {
    let mut content =
        String::from("profileVersion = \"v1\"\n\n[[supports]]\ngame = \"nightreign\"\n");

    for native in natives {
        content.push_str("\n[[natives]]\n");
        content.push_str(&format!("path = {}\n", toml_string(&native.path)));
        content.push_str("optional = false\n");
        content.push_str("enabled = true\n");
        content.push_str("load_before = []\n");
        content.push_str("load_after = []\n");
        content.push_str(&format!("load_early = {}\n", native.load_early));
    }

    for package in packages {
        content.push_str("\n[[packages]]\n");
        content.push_str("enabled = true\n");
        content.push_str(&format!("path = {}\n", toml_string(&package.path)));
        content.push_str("load_after = []\n");
        content.push_str("load_before = []\n");
    }

    content
}

fn toml_string(path: &Path) -> String {
    let value = path.to_string_lossy().to_string();
    serde_json::to_string(&value).unwrap_or_else(|_| "\"\"".to_string())
}

fn has_dll_file(path: &Path) -> bool {
    !find_dll_files(path).is_empty()
}

fn find_dll_files(path: &Path) -> Vec<PathBuf> {
    let mut dlls = Vec::new();
    collect_files_with_extension(path, "dll", &mut dlls);
    dlls
}

fn collect_files_with_extension(path: &Path, extension: &str, files: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(path) else {
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_files_with_extension(&path, extension, files);
        } else if path
            .extension()
            .and_then(|ext| ext.to_str())
            .map_or(false, |ext| ext.eq_ignore_ascii_case(extension))
        {
            files.push(fs::canonicalize(&path).unwrap_or(path));
        }
    }
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
                    .map_or(false, |ext| ext.eq_ignore_ascii_case("dcx"))
            })
}

fn is_disabled_path(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .map_or(false, |name| name.ends_with(".disabled"))
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

    #[test]
    fn infers_package_for_parts_only_mod() {
        let mod_dir = PathBuf::from(r"D:\Game\mods\duchessunmask");
        let packages = vec![PackageEntry {
            path: mod_dir.clone(),
        }];
        let natives = Vec::new();

        let profile = build_me3_profile(&packages, &natives);

        assert!(profile.contains("[[packages]]"));
        assert!(profile.contains("game = \"nightreign\""));
        assert!(profile.contains(r"D:\\Game\\mods\\duchessunmask"));
    }

    #[test]
    fn build_profile_preserves_package_order() {
        let packages = vec![
            PackageEntry {
                path: PathBuf::from(r"D:\Game\mods\z_last"),
            },
            PackageEntry {
                path: PathBuf::from(r"D:\Game\mods\a_first"),
            },
        ];
        let natives = Vec::new();

        let profile = build_me3_profile(&packages, &natives);
        let z_index = profile.find(r"D:\\Game\\mods\\z_last").unwrap();
        let a_index = profile.find(r"D:\\Game\\mods\\a_first").unwrap();

        assert!(z_index < a_index);
    }
}
