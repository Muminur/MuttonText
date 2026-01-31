//! Tauri IPC command handlers.

use std::sync::Mutex;
use crate::managers::combo_manager::ComboManager;

pub mod combo_commands;
pub mod error;
pub mod group_commands;
pub mod picker_commands;
pub mod shortcut_commands;

/// Application state shared across all Tauri commands.
pub struct AppState {
    pub combo_manager: Mutex<ComboManager>,
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_commands_module_loads() {
        assert!(true);
    }
}
