//! Tauri IPC commands for group CRUD operations.

use tauri::State;
use uuid::Uuid;

use crate::models::group::Group;

use super::error::CommandError;
use super::AppState;

/// Parses a UUID string, returning a `CommandError` on failure.
fn parse_uuid(field: &str, value: &str) -> Result<Uuid, CommandError> {
    Uuid::parse_str(value).map_err(|_| CommandError::invalid_uuid(field, value))
}

/// Returns all groups.
#[tauri::command]
pub fn get_all_groups(state: State<AppState>) -> Result<Vec<Group>, CommandError> {
    let manager = state
        .combo_manager
        .lock()
        .map_err(|_| CommandError {
            code: "LOCK_ERROR".to_string(),
            message: "Failed to acquire combo manager lock".to_string(),
        })?;
    Ok(manager.get_all_groups())
}

/// Returns a single group by ID.
#[tauri::command]
pub fn get_group(state: State<AppState>, id: String) -> Result<Option<Group>, CommandError> {
    let uuid = parse_uuid("id", &id)?;
    let manager = state
        .combo_manager
        .lock()
        .map_err(|_| CommandError {
            code: "LOCK_ERROR".to_string(),
            message: "Failed to acquire combo manager lock".to_string(),
        })?;
    Ok(manager.get_group(uuid))
}

/// Creates a new group.
#[tauri::command]
pub fn create_group(
    state: State<AppState>,
    name: String,
    description: String,
) -> Result<Group, CommandError> {
    let mut manager = state
        .combo_manager
        .lock()
        .map_err(|_| CommandError {
            code: "LOCK_ERROR".to_string(),
            message: "Failed to acquire combo manager lock".to_string(),
        })?;
    manager
        .create_group(name, description)
        .map_err(CommandError::from)
}

/// Updates an existing group. Only provided fields are changed.
#[tauri::command]
pub fn update_group(
    state: State<AppState>,
    id: String,
    name: Option<String>,
    description: Option<String>,
) -> Result<Group, CommandError> {
    let uuid = parse_uuid("id", &id)?;
    let mut manager = state
        .combo_manager
        .lock()
        .map_err(|_| CommandError {
            code: "LOCK_ERROR".to_string(),
            message: "Failed to acquire combo manager lock".to_string(),
        })?;
    manager
        .update_group(uuid, name, description)
        .map_err(CommandError::from)
}

/// Deletes a group and all its combos.
#[tauri::command]
pub fn delete_group(state: State<AppState>, id: String) -> Result<(), CommandError> {
    let uuid = parse_uuid("id", &id)?;
    let mut manager = state
        .combo_manager
        .lock()
        .map_err(|_| CommandError {
            code: "LOCK_ERROR".to_string(),
            message: "Failed to acquire combo manager lock".to_string(),
        })?;
    manager.delete_group(uuid).map_err(CommandError::from)
}

/// Toggles a group's enabled state (and all its combos). Returns the new state.
#[tauri::command]
pub fn toggle_group(state: State<AppState>, id: String) -> Result<bool, CommandError> {
    let uuid = parse_uuid("id", &id)?;
    let mut manager = state
        .combo_manager
        .lock()
        .map_err(|_| CommandError {
            code: "LOCK_ERROR".to_string(),
            message: "Failed to acquire combo manager lock".to_string(),
        })?;
    manager.toggle_group(uuid).map_err(CommandError::from)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_uuid_valid() {
        let id = Uuid::new_v4();
        assert!(parse_uuid("id", &id.to_string()).is_ok());
    }

    #[test]
    fn test_parse_uuid_invalid() {
        assert!(parse_uuid("id", "bad").is_err());
    }
}
