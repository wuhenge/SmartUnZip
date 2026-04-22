mod commands;
mod registry;
mod update;

pub use commands::*;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            commands::load_config,
            commands::save_config,
            commands::get_config_path,
            commands::validate_7zip_path,
            commands::check_for_updates,
            commands::open_url,
            commands::check_context_menu,
            commands::add_context_menu,
            commands::remove_context_menu,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
