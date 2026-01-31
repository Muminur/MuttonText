//! Tauri IPC commands for global shortcut management.

use tauri::State;
use std::sync::Mutex;

use crate::managers::shortcut_manager::{ShortcutManager, ShortcutError};

use super::error::CommandError;

/// Application state for shortcut manager.
pub struct ShortcutState {
    pub shortcut_manager: Mutex<ShortcutManager>,
}

impl From<ShortcutError> for CommandError {
    fn from(err: ShortcutError) -> Self {
        let code = match err {
            ShortcutError::InvalidFormat(_) => "INVALID_SHORTCUT",
            ShortcutError::AlreadyRegistered(_) => "SHORTCUT_ALREADY_REGISTERED",
            ShortcutError::RegistrationFailed(_) => "SHORTCUT_REGISTRATION_FAILED",
            ShortcutError::UnregistrationFailed(_) => "SHORTCUT_UNREGISTRATION_FAILED",
            ShortcutError::NoShortcutRegistered => "NO_SHORTCUT_REGISTERED",
        };

        CommandError {
            code: code.to_string(),
            message: err.to_string(),
        }
    }
}

/// Registers a global shortcut for the picker window.
#[tauri::command]
pub fn register_picker_shortcut(
    state: State<ShortcutState>,
    shortcut: String,
) -> Result<(), CommandError> {
    let mut manager = state
        .shortcut_manager
        .lock()
        .map_err(|_| CommandError {
            code: "LOCK_ERROR".to_string(),
            message: "Failed to acquire shortcut manager lock".to_string(),
        })?;

    manager
        .register_picker_shortcut(&shortcut)
        .map_err(CommandError::from)
}

/// Unregisters the current picker shortcut.
#[tauri::command]
pub fn unregister_picker_shortcut(state: State<ShortcutState>) -> Result<(), CommandError> {
    let mut manager = state
        .shortcut_manager
        .lock()
        .map_err(|_| CommandError {
            code: "LOCK_ERROR".to_string(),
            message: "Failed to acquire shortcut manager lock".to_string(),
        })?;

    manager
        .unregister_picker_shortcut()
        .map_err(CommandError::from)
}

/// Gets the currently registered shortcut.
#[tauri::command]
pub fn get_picker_shortcut(state: State<ShortcutState>) -> Result<Option<String>, CommandError> {
    let manager = state
        .shortcut_manager
        .lock()
        .map_err(|_| CommandError {
            code: "LOCK_ERROR".to_string(),
            message: "Failed to acquire shortcut manager lock".to_string(),
        })?;

    Ok(manager.get_registered_shortcut().map(String::from))
}

/// Gets the default picker shortcut.
#[tauri::command]
pub fn get_default_picker_shortcut() -> String {
    ShortcutManager::default_shortcut()
}

/// Enables or disables the shortcut manager.
#[tauri::command]
pub fn set_shortcut_enabled(
    state: State<ShortcutState>,
    enabled: bool,
) -> Result<(), CommandError> {
    let mut manager = state
        .shortcut_manager
        .lock()
        .map_err(|_| CommandError {
            code: "LOCK_ERROR".to_string(),
            message: "Failed to acquire shortcut manager lock".to_string(),
        })?;

    manager.set_enabled(enabled);
    Ok(())
}

/// Checks if the shortcut manager is enabled.
#[tauri::command]
pub fn is_shortcut_enabled(state: State<ShortcutState>) -> Result<bool, CommandError> {
    let manager = state
        .shortcut_manager
        .lock()
        .map_err(|_| CommandError {
            code: "LOCK_ERROR".to_string(),
            message: "Failed to acquire shortcut manager lock".to_string(),
        })?;

    Ok(manager.is_enabled())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_default_picker_shortcut() {
        let shortcut = get_default_picker_shortcut();
        assert_eq!(shortcut, "Ctrl+Shift+Space");
    }

    #[test]
    fn test_shortcut_error_conversion() {
        let err = ShortcutError::InvalidFormat("test".to_string());
        let cmd_err: CommandError = err.into();
        assert_eq!(cmd_err.code, "INVALID_SHORTCUT");

        let err = ShortcutError::AlreadyRegistered("test".to_string());
        let cmd_err: CommandError = err.into();
        assert_eq!(cmd_err.code, "SHORTCUT_ALREADY_REGISTERED");

        let err = ShortcutError::RegistrationFailed("test".to_string());
        let cmd_err: CommandError = err.into();
        assert_eq!(cmd_err.code, "SHORTCUT_REGISTRATION_FAILED");

        let err = ShortcutError::NoShortcutRegistered;
        let cmd_err: CommandError = err.into();
        assert_eq!(cmd_err.code, "NO_SHORTCUT_REGISTERED");
    }

    #[test]
    fn test_shortcut_commands_module_compiles() {
        // Basic compilation test
        assert!(true);
    }
}
