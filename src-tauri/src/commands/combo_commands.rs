//! Tauri IPC commands for combo CRUD operations.

use tauri::State;
use uuid::Uuid;

use crate::models::combo::Combo;
use crate::models::matching::MatchingMode;

use super::error::CommandError;
use super::AppState;

/// Parses a UUID string, returning a `CommandError` on failure.
fn parse_uuid(field: &str, value: &str) -> Result<Uuid, CommandError> {
    Uuid::parse_str(value).map_err(|_| CommandError::invalid_uuid(field, value))
}

/// Parses a matching mode string ("strict" or "loose").
fn parse_matching_mode(value: &str) -> Result<MatchingMode, CommandError> {
    match value.to_lowercase().as_str() {
        "strict" => Ok(MatchingMode::Strict),
        "loose" => Ok(MatchingMode::Loose),
        _ => Err(CommandError::invalid_matching_mode(value)),
    }
}

/// Returns all combos.
#[tauri::command]
pub fn get_all_combos(state: State<AppState>) -> Result<Vec<Combo>, CommandError> {
    let manager = state
        .combo_manager
        .lock()
        .map_err(|_| CommandError {
            code: "LOCK_ERROR".to_string(),
            message: "Failed to acquire combo manager lock".to_string(),
        })?;
    Ok(manager.get_all_combos())
}

/// Returns a single combo by ID, or null if not found.
#[tauri::command]
pub fn get_combo(state: State<AppState>, id: String) -> Result<Option<Combo>, CommandError> {
    let uuid = parse_uuid("id", &id)?;
    let manager = state
        .combo_manager
        .lock()
        .map_err(|_| CommandError {
            code: "LOCK_ERROR".to_string(),
            message: "Failed to acquire combo manager lock".to_string(),
        })?;
    Ok(manager.get_combo(uuid))
}

/// Creates a new combo.
#[tauri::command]
pub fn create_combo(
    state: State<AppState>,
    name: String,
    keyword: String,
    snippet: String,
    group_id: String,
    matching_mode: String,
    case_sensitive: bool,
) -> Result<Combo, CommandError> {
    let gid = parse_uuid("group_id", &group_id)?;
    let mode = parse_matching_mode(&matching_mode)?;
    let mut manager = state
        .combo_manager
        .lock()
        .map_err(|_| CommandError {
            code: "LOCK_ERROR".to_string(),
            message: "Failed to acquire combo manager lock".to_string(),
        })?;
    manager
        .create_combo(name, keyword, snippet, gid, mode, case_sensitive)
        .map_err(CommandError::from)
}

/// Updates an existing combo. Only provided fields are changed.
#[tauri::command]
pub fn update_combo(
    state: State<AppState>,
    id: String,
    name: Option<String>,
    keyword: Option<String>,
    snippet: Option<String>,
    group_id: Option<String>,
    matching_mode: Option<String>,
    case_sensitive: Option<bool>,
    enabled: Option<bool>,
) -> Result<Combo, CommandError> {
    let uuid = parse_uuid("id", &id)?;
    let gid = group_id.map(|g| parse_uuid("group_id", &g)).transpose()?;
    let mode = matching_mode
        .map(|m| parse_matching_mode(&m))
        .transpose()?;
    let mut manager = state
        .combo_manager
        .lock()
        .map_err(|_| CommandError {
            code: "LOCK_ERROR".to_string(),
            message: "Failed to acquire combo manager lock".to_string(),
        })?;
    manager
        .update_combo(uuid, name, keyword, snippet, gid, mode, case_sensitive, enabled)
        .map_err(CommandError::from)
}

/// Deletes a combo by ID.
#[tauri::command]
pub fn delete_combo(state: State<AppState>, id: String) -> Result<(), CommandError> {
    let uuid = parse_uuid("id", &id)?;
    let mut manager = state
        .combo_manager
        .lock()
        .map_err(|_| CommandError {
            code: "LOCK_ERROR".to_string(),
            message: "Failed to acquire combo manager lock".to_string(),
        })?;
    manager.delete_combo(uuid).map_err(CommandError::from)
}

/// Duplicates a combo, returning the new copy.
#[tauri::command]
pub fn duplicate_combo(state: State<AppState>, id: String) -> Result<Combo, CommandError> {
    let uuid = parse_uuid("id", &id)?;
    let mut manager = state
        .combo_manager
        .lock()
        .map_err(|_| CommandError {
            code: "LOCK_ERROR".to_string(),
            message: "Failed to acquire combo manager lock".to_string(),
        })?;
    manager.duplicate_combo(uuid).map_err(CommandError::from)
}

/// Moves a combo to a different group.
#[tauri::command]
pub fn move_combo_to_group(
    state: State<AppState>,
    combo_id: String,
    group_id: String,
) -> Result<(), CommandError> {
    let cid = parse_uuid("combo_id", &combo_id)?;
    let gid = parse_uuid("group_id", &group_id)?;
    let mut manager = state
        .combo_manager
        .lock()
        .map_err(|_| CommandError {
            code: "LOCK_ERROR".to_string(),
            message: "Failed to acquire combo manager lock".to_string(),
        })?;
    manager
        .move_combo_to_group(cid, gid)
        .map_err(CommandError::from)
}

/// Toggles a combo's enabled state and returns the new state.
#[tauri::command]
pub fn toggle_combo(state: State<AppState>, id: String) -> Result<bool, CommandError> {
    let uuid = parse_uuid("id", &id)?;
    let mut manager = state
        .combo_manager
        .lock()
        .map_err(|_| CommandError {
            code: "LOCK_ERROR".to_string(),
            message: "Failed to acquire combo manager lock".to_string(),
        })?;
    manager.toggle_combo(uuid).map_err(CommandError::from)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_uuid_valid() {
        let id = Uuid::new_v4();
        let result = parse_uuid("id", &id.to_string());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), id);
    }

    #[test]
    fn test_parse_uuid_invalid() {
        let result = parse_uuid("id", "not-a-uuid");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_matching_mode_strict() {
        assert_eq!(parse_matching_mode("strict").unwrap(), MatchingMode::Strict);
        assert_eq!(parse_matching_mode("Strict").unwrap(), MatchingMode::Strict);
        assert_eq!(parse_matching_mode("STRICT").unwrap(), MatchingMode::Strict);
    }

    #[test]
    fn test_parse_matching_mode_loose() {
        assert_eq!(parse_matching_mode("loose").unwrap(), MatchingMode::Loose);
    }

    #[test]
    fn test_parse_matching_mode_invalid() {
        let result = parse_matching_mode("invalid");
        assert!(result.is_err());
    }
}
