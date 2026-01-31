//! Tauri IPC commands for preferences management.

use std::sync::Mutex;

use tauri::State;

use crate::managers::preferences_manager::{PreferencesError, PreferencesManager};
use crate::models::preferences::Preferences;

use super::error::CommandError;

/// Tauri-managed state wrapper for PreferencesManager.
pub struct PreferencesState {
    pub preferences_manager: Mutex<PreferencesManager>,
}

impl From<PreferencesError> for CommandError {
    fn from(err: PreferencesError) -> Self {
        match &err {
            PreferencesError::Validation(_) => CommandError {
                code: "VALIDATION_ERROR".to_string(),
                message: err.to_string(),
            },
            PreferencesError::AppAlreadyExcluded(_) => CommandError {
                code: "APP_ALREADY_EXCLUDED".to_string(),
                message: err.to_string(),
            },
            PreferencesError::Io(_) => CommandError {
                code: "IO_ERROR".to_string(),
                message: err.to_string(),
            },
            PreferencesError::Serialization(_) => CommandError {
                code: "SERIALIZATION_ERROR".to_string(),
                message: err.to_string(),
            },
        }
    }
}

fn lock_prefs<'a>(
    state: &'a State<'_, PreferencesState>,
) -> Result<std::sync::MutexGuard<'a, PreferencesManager>, CommandError> {
    state
        .preferences_manager
        .lock()
        .map_err(|e| CommandError {
            code: "LOCK_ERROR".to_string(),
            message: format!("Failed to acquire preferences lock: {e}"),
        })
}

#[tauri::command]
pub fn get_preferences(state: State<'_, PreferencesState>) -> Result<Preferences, CommandError> {
    let mgr = lock_prefs(&state)?;
    Ok(mgr.get().clone())
}

#[tauri::command]
pub fn update_preferences(
    preferences: Preferences,
    state: State<'_, PreferencesState>,
) -> Result<(), CommandError> {
    let mut mgr = lock_prefs(&state)?;
    mgr.update(preferences).map_err(CommandError::from)
}

#[tauri::command]
pub fn reset_preferences(state: State<'_, PreferencesState>) -> Result<(), CommandError> {
    let mut mgr = lock_prefs(&state)?;
    mgr.reset_to_defaults().map_err(CommandError::from)
}

#[tauri::command]
pub fn get_excluded_apps(
    state: State<'_, PreferencesState>,
) -> Result<Vec<String>, CommandError> {
    let mgr = lock_prefs(&state)?;
    Ok(mgr.get_excluded_apps().to_vec())
}

#[tauri::command]
pub fn add_excluded_app(
    app: String,
    state: State<'_, PreferencesState>,
) -> Result<(), CommandError> {
    let mut mgr = lock_prefs(&state)?;
    mgr.add_excluded_app(app).map_err(CommandError::from)
}

#[tauri::command]
pub fn remove_excluded_app(
    app: String,
    state: State<'_, PreferencesState>,
) -> Result<bool, CommandError> {
    let mut mgr = lock_prefs(&state)?;
    mgr.remove_excluded_app(&app).map_err(CommandError::from)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::preferences::{PasteMethod, Theme};

    #[test]
    fn test_preferences_serialization_roundtrip() {
        let prefs = Preferences::default();
        let json = serde_json::to_string(&prefs).unwrap();
        let deser: Preferences = serde_json::from_str(&json).unwrap();
        assert_eq!(prefs, deser);
    }

    #[test]
    fn test_preferences_error_to_command_error_validation() {
        let err = PreferencesError::Validation("bad".to_string());
        let cmd_err: CommandError = err.into();
        assert_eq!(cmd_err.code, "VALIDATION_ERROR");
    }

    #[test]
    fn test_preferences_error_to_command_error_excluded() {
        let err = PreferencesError::AppAlreadyExcluded("app".to_string());
        let cmd_err: CommandError = err.into();
        assert_eq!(cmd_err.code, "APP_ALREADY_EXCLUDED");
    }

    #[test]
    fn test_preferences_json_fields() {
        let prefs = Preferences::default();
        let json = serde_json::to_string(&prefs).unwrap();
        // Verify camelCase
        assert!(json.contains("playSound"));
        assert!(json.contains("showSystemTray"));
        assert!(json.contains("pasteMethod"));
    }

    #[test]
    fn test_paste_method_serialization() {
        let json = serde_json::to_string(&PasteMethod::Clipboard).unwrap();
        assert_eq!(json, "\"clipboard\"");
        let json = serde_json::to_string(&PasteMethod::SimulateKeystrokes).unwrap();
        assert_eq!(json, "\"simulateKeystrokes\"");
    }

    #[test]
    fn test_theme_serialization() {
        let json = serde_json::to_string(&Theme::System).unwrap();
        assert_eq!(json, "\"system\"");
        let json = serde_json::to_string(&Theme::Dark).unwrap();
        assert_eq!(json, "\"dark\"");
    }

    #[test]
    fn test_preferences_state_struct() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("prefs.json");
        let mgr = PreferencesManager::new(path).unwrap();
        let state = PreferencesState {
            preferences_manager: Mutex::new(mgr),
        };
        let guard = state.preferences_manager.lock().unwrap();
        assert!(guard.get().enabled);
    }
}
