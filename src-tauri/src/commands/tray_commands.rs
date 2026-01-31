//! Tauri IPC commands for system tray operations.

use std::sync::Mutex;

use tauri::State;

use crate::managers::tray_manager::{TrayManager, TrayMenuItem, TrayState};

use super::error::CommandError;

/// Tauri-managed state wrapper for TrayManager.
pub struct TrayMgrState {
    pub tray_manager: Mutex<TrayManager>,
}

#[tauri::command]
pub fn get_tray_state(state: State<'_, TrayMgrState>) -> Result<TrayState, CommandError> {
    let mgr = state.tray_manager.lock().map_err(|e| CommandError {
        code: "LOCK_ERROR".to_string(),
        message: format!("Failed to acquire tray lock: {e}"),
    })?;
    Ok(mgr.state())
}

#[tauri::command]
pub fn set_tray_enabled(
    enabled: bool,
    state: State<'_, TrayMgrState>,
) -> Result<(), CommandError> {
    let mut mgr = state.tray_manager.lock().map_err(|e| CommandError {
        code: "LOCK_ERROR".to_string(),
        message: format!("Failed to acquire tray lock: {e}"),
    })?;
    if enabled {
        mgr.set_state(TrayState::Active);
    } else {
        mgr.set_state(TrayState::Paused);
    }
    Ok(())
}

#[tauri::command]
pub fn get_tray_menu_items(
    state: State<'_, TrayMgrState>,
) -> Result<Vec<TrayMenuItem>, CommandError> {
    let mgr = state.tray_manager.lock().map_err(|e| CommandError {
        code: "LOCK_ERROR".to_string(),
        message: format!("Failed to acquire tray lock: {e}"),
    })?;
    Ok(mgr.build_menu_items())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tray_state_serialization() {
        let state = TrayState::Active;
        let json = serde_json::to_string(&state).unwrap();
        assert_eq!(json, "\"active\"");

        let state = TrayState::Paused;
        let json = serde_json::to_string(&state).unwrap();
        assert_eq!(json, "\"paused\"");

        let state = TrayState::ExcludedApp;
        let json = serde_json::to_string(&state).unwrap();
        assert_eq!(json, "\"excludedApp\"");
    }

    #[test]
    fn test_tray_state_deserialization() {
        let state: TrayState = serde_json::from_str("\"active\"").unwrap();
        assert_eq!(state, TrayState::Active);

        let state: TrayState = serde_json::from_str("\"paused\"").unwrap();
        assert_eq!(state, TrayState::Paused);
    }

    #[test]
    fn test_tray_menu_item_serialization() {
        let item = TrayMenuItem {
            id: "show".to_string(),
            label: "Show MuttonText".to_string(),
            enabled: true,
            checked: None,
        };
        let json = serde_json::to_string(&item).unwrap();
        assert!(json.contains("\"id\":\"show\""));
        assert!(json.contains("\"label\":\"Show MuttonText\""));
        assert!(json.contains("\"enabled\":true"));
        assert!(json.contains("\"checked\":null"));
    }

    #[test]
    fn test_tray_menu_item_with_checked() {
        let item = TrayMenuItem {
            id: "enabled".to_string(),
            label: "Enabled".to_string(),
            enabled: true,
            checked: Some(true),
        };
        let json = serde_json::to_string(&item).unwrap();
        assert!(json.contains("\"checked\":true"));
    }

    #[test]
    fn test_tray_mgr_state_struct() {
        let state = TrayMgrState {
            tray_manager: Mutex::new(TrayManager::new()),
        };
        let mgr = state.tray_manager.lock().unwrap();
        assert_eq!(mgr.state(), TrayState::Active);
    }
}
