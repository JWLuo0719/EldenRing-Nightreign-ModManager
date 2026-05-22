mod commands;

use commands::mod_manager;
use commands::profile;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_log::Builder::default().build())
        .invoke_handler(tauri::generate_handler![
            mod_manager::get_game_path,
            mod_manager::set_game_path,
            mod_manager::scan_mods,
            mod_manager::get_mod_info,
            mod_manager::install_mod_from_zip,
            mod_manager::uninstall_mod,
            mod_manager::toggle_mod,
            mod_manager::get_me3_path,
            mod_manager::set_me3_path,
            mod_manager::launch_game,
            profile::get_profiles,
            profile::create_profile,
            profile::delete_profile,
            profile::activate_profile,
            profile::get_active_profile,
            profile::update_profile,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
