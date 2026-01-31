pub mod commands;
pub mod managers;
pub mod models;
pub mod platform;
pub mod utils;

use std::sync::Mutex;

use tracing_subscriber::EnvFilter;

use commands::AppState;
use commands::shortcut_commands::ShortcutState;
use managers::combo_manager::ComboManager;
use managers::combo_storage::ComboStorage;
use managers::shortcut_manager::ShortcutManager;
use managers::storage::get_combos_path;

/// Initialize the tracing subscriber for structured logging.
fn init_tracing() {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    init_tracing();

    let combos_path = get_combos_path().expect("Failed to resolve combos.json path");
    let storage = ComboStorage::new(combos_path);
    let manager = ComboManager::new(storage).expect("Failed to initialize ComboManager");
    let shortcut_manager = ShortcutManager::new();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(AppState {
            combo_manager: Mutex::new(manager),
        })
        .manage(ShortcutState {
            shortcut_manager: Mutex::new(shortcut_manager),
        })
        .invoke_handler(tauri::generate_handler![
            // Combo commands
            commands::combo_commands::get_all_combos,
            commands::combo_commands::get_combo,
            commands::combo_commands::create_combo,
            commands::combo_commands::update_combo,
            commands::combo_commands::delete_combo,
            commands::combo_commands::duplicate_combo,
            commands::combo_commands::move_combo_to_group,
            commands::combo_commands::toggle_combo,
            // Group commands
            commands::group_commands::get_all_groups,
            commands::group_commands::get_group,
            commands::group_commands::create_group,
            commands::group_commands::update_group,
            commands::group_commands::delete_group,
            commands::group_commands::toggle_group,
            // Picker commands
            commands::picker_commands::open_picker_window,
            commands::picker_commands::close_picker_window,
            commands::picker_commands::search_combos,
            // Shortcut commands
            commands::shortcut_commands::register_picker_shortcut,
            commands::shortcut_commands::unregister_picker_shortcut,
            commands::shortcut_commands::get_picker_shortcut,
            commands::shortcut_commands::get_default_picker_shortcut,
            commands::shortcut_commands::set_shortcut_enabled,
            commands::shortcut_commands::is_shortcut_enabled,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_app_crate_compiles() {
        assert!(true);
    }
}
