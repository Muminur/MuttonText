// Core business logic managers

pub mod combo_manager;
pub mod combo_storage;
pub mod file_watcher;
pub mod preferences_storage;
pub mod storage;

// Re-export commonly used types for convenience
pub use combo_manager::{ComboManager, ComboManagerError};
pub use combo_storage::ComboStorage;
pub use file_watcher::FileWatcher;
pub use preferences_storage::PreferencesStorage;
pub use storage::{StorageError, ensure_dirs_exist, get_config_dir, get_combos_path, get_preferences_path, get_backups_dir, get_logs_dir};

pub mod input_manager;
pub mod matching;
pub mod clipboard_manager;
pub mod substitution;
pub mod expansion_pipeline;

// Re-export Milestone 6 types
pub use matching::{MatcherEngine, MatchResult};
pub use clipboard_manager::ClipboardManager;
pub use substitution::SubstitutionEngine;
pub use expansion_pipeline::ExpansionPipeline;

pub mod variable_evaluator;

// Re-export Milestone 7 types
pub use variable_evaluator::{VariableEvaluator, VariableError, EvalContext, EvalResult, KeyAction};

pub mod shortcut_manager;

// Re-export Milestone 8 types
pub use shortcut_manager::{ShortcutManager, ShortcutError};

pub mod tray_manager;
pub mod preferences_manager;
pub mod lifecycle_manager;
pub mod emoji_manager;

// Re-export Milestone 9 types
pub use tray_manager::{TrayManager, TrayState as TrayIconState, TrayMenuItem};
pub use preferences_manager::{PreferencesManager, PreferencesError};
pub use lifecycle_manager::{LifecycleManager, LifecycleError, AutostartConfig};
pub use emoji_manager::{EmojiManager, EmojiEntry, EmojiError};

// Milestone 10: Import/Export/Backup/Update
pub mod import_manager;
pub mod export_manager;
pub mod backup_manager;
pub mod update_manager;

pub use import_manager::ImportManager;
pub use export_manager::ExportManager;
pub use backup_manager::BackupManager;
pub use update_manager::UpdateManager;

#[cfg(test)]
mod tests {
    #[test]
    fn test_managers_module_loads() {
        assert!(true);
    }
}
