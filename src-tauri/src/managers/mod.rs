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

// Submodules to be added as features are implemented:
// pub mod preferences_manager;
// pub mod backup_manager;

#[cfg(test)]
mod tests {
    #[test]
    fn test_managers_module_loads() {
        assert!(true);
    }
}
