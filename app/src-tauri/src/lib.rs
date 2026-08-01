mod commands;

use commands::mod_manager;
use commands::profile;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            mod_manager::get_game_path,
            mod_manager::set_game_path,
            mod_manager::get_runtime_environment_status,
            mod_manager::set_runtime_environment,
            mod_manager::scan_mods,
            mod_manager::get_mod_info,
            mod_manager::install_mod_from_zip,
            mod_manager::add_external_mod,
            mod_manager::relink_external_mod,
            mod_manager::add_external_dll,
            mod_manager::remove_external_mod,
            mod_manager::toggle_external_mod,
            mod_manager::set_external_mod_profile_mode,
            mod_manager::read_mod_config_file,
            mod_manager::write_mod_config_file,
            mod_manager::uninstall_mod,
            mod_manager::toggle_mod,
            mod_manager::get_me3_path,
            mod_manager::set_me3_path,
            mod_manager::get_launch_exe_path,
            mod_manager::set_launch_exe_path,
            mod_manager::get_mods_dir,
            mod_manager::launch_game,
            mod_manager::diagnose_launch_game,
            mod_manager::generate_me3_profile,
            mod_manager::get_launch_artifacts,
            mod_manager::get_launch_preflight,
            mod_manager::detect_file_conflicts,
            mod_manager::export_multiplayer_manifest,
            mod_manager::compare_multiplayer_manifest,
            mod_manager::get_special_mod_status,
            mod_manager::install_seamless_onlinefix,
            mod_manager::restore_latest_online_patch_backup,
            profile::get_profiles,
            profile::create_profile,
            profile::delete_profile,
            profile::activate_profile,
            profile::get_active_profile,
            profile::update_profile,
            profile::update_active_profile_mod,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
