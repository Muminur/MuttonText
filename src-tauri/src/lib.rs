pub mod commands;
pub mod managers;
pub mod models;
pub mod platform;
pub mod utils;

use std::sync::Mutex;

use tracing_subscriber::EnvFilter;

use commands::AppState;
use commands::shortcut_commands::ShortcutState;
use commands::tray_commands::TrayMgrState;
use commands::preferences_commands::PreferencesState;
use commands::data_commands::{BackupState, UpdateState};
use managers::combo_manager::ComboManager;
use managers::combo_storage::ComboStorage;
use managers::shortcut_manager::ShortcutManager;
use managers::tray_manager::TrayManager;
use managers::preferences_manager::PreferencesManager;
use managers::backup_manager::BackupManager;
use managers::update_manager::UpdateManager;
use managers::storage::{get_combos_path, get_preferences_path, get_backups_dir};

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
    let tray_manager = TrayManager::new();
    let preferences_path = get_preferences_path().expect("Failed to resolve preferences.json path");
    let preferences_manager = PreferencesManager::new(preferences_path).expect("Failed to initialize PreferencesManager");
    let backups_dir = get_backups_dir().expect("Failed to resolve backups directory");
    let backup_manager = BackupManager::new(backups_dir, 10);
    let update_manager = UpdateManager::new(env!("CARGO_PKG_VERSION").to_string());

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(AppState {
            combo_manager: Mutex::new(manager),
        })
        .manage(ShortcutState {
            shortcut_manager: Mutex::new(shortcut_manager),
        })
        .manage(TrayMgrState {
            tray_manager: Mutex::new(tray_manager),
        })
        .manage(PreferencesState {
            preferences_manager: Mutex::new(preferences_manager),
        })
        .manage(BackupState {
            backup_manager: Mutex::new(backup_manager),
        })
        .manage(UpdateState {
            update_manager: Mutex::new(update_manager),
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
            // Tray commands
            commands::tray_commands::get_tray_state,
            commands::tray_commands::set_tray_enabled,
            commands::tray_commands::get_tray_menu_items,
            // Preferences commands
            commands::preferences_commands::get_preferences,
            commands::preferences_commands::update_preferences,
            commands::preferences_commands::reset_preferences,
            commands::preferences_commands::get_excluded_apps,
            commands::preferences_commands::add_excluded_app,
            commands::preferences_commands::remove_excluded_app,
            // Data commands (import/export/backup/update)
            commands::data_commands::import_combos,
            commands::data_commands::preview_import,
            commands::data_commands::export_combos,
            commands::data_commands::create_backup,
            commands::data_commands::restore_backup,
            commands::data_commands::list_backups,
            commands::data_commands::delete_backup,
            commands::data_commands::check_for_updates,
            commands::data_commands::skip_update_version,
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
