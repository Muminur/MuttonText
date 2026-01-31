//! Tauri IPC commands for import, export, backup, and update operations.

use std::sync::Mutex;

use crate::commands::error::CommandError;
use crate::managers::backup_manager::{BackupInfo, BackupManager};
use crate::managers::export_manager::{ExportFormat, ExportManager};
use crate::managers::import_manager::{ConflictResolution, ImportFormat, ImportManager, ImportPreview, ImportResult};
use crate::managers::update_manager::{UpdateManager, VersionInfo};
use crate::models::combo::Combo;
use crate::models::group::Group;

/// State for backup manager, managed by Tauri.
pub struct BackupState {
    pub backup_manager: Mutex<BackupManager>,
}

/// State for update manager, managed by Tauri.
pub struct UpdateState {
    pub update_manager: Mutex<UpdateManager>,
}

impl From<crate::managers::import_manager::ImportError> for CommandError {
    fn from(err: crate::managers::import_manager::ImportError) -> Self {
        CommandError {
            code: "IMPORT_ERROR".to_string(),
            message: err.to_string(),
        }
    }
}

impl From<crate::managers::export_manager::ExportError> for CommandError {
    fn from(err: crate::managers::export_manager::ExportError) -> Self {
        CommandError {
            code: "EXPORT_ERROR".to_string(),
            message: err.to_string(),
        }
    }
}

impl From<crate::managers::backup_manager::BackupError> for CommandError {
    fn from(err: crate::managers::backup_manager::BackupError) -> Self {
        CommandError {
            code: "BACKUP_ERROR".to_string(),
            message: err.to_string(),
        }
    }
}

fn parse_import_format(format: &str) -> Result<ImportFormat, CommandError> {
    match format {
        "beeftextJson" => Ok(ImportFormat::BeeftextJson),
        "beeftextCsv" => Ok(ImportFormat::BeeftextCsv),
        "textExpanderCsv" => Ok(ImportFormat::TextExpanderCsv),
        "muttonTextJson" => Ok(ImportFormat::MuttonTextJson),
        "auto" => Ok(ImportFormat::MuttonTextJson), // fallback
        _ => Err(CommandError {
            code: "INVALID_FORMAT".to_string(),
            message: format!("Unknown import format: {}", format),
        }),
    }
}

fn parse_conflict_resolution(resolution: &str) -> Result<ConflictResolution, CommandError> {
    match resolution {
        "skip" => Ok(ConflictResolution::Skip),
        "overwrite" => Ok(ConflictResolution::Overwrite),
        "rename" => Ok(ConflictResolution::Rename),
        _ => Err(CommandError {
            code: "INVALID_CONFLICT_RESOLUTION".to_string(),
            message: format!("Unknown conflict resolution: {}", resolution),
        }),
    }
}

fn parse_export_format(format: &str) -> Result<ExportFormat, CommandError> {
    match format {
        "muttonTextJson" => Ok(ExportFormat::MuttonTextJson),
        "textExpanderCsv" => Ok(ExportFormat::TextExpanderCsv),
        "cheatsheetCsv" => Ok(ExportFormat::CheatsheetCsv),
        _ => Err(CommandError {
            code: "INVALID_FORMAT".to_string(),
            message: format!("Unknown export format: {}", format),
        }),
    }
}

/// Import combos from the given content string.
#[tauri::command]
pub fn import_combos(
    content: String,
    format: String,
    conflict_resolution: String,
) -> Result<ImportResult, CommandError> {
    const MAX_IMPORT_SIZE: usize = 10 * 1024 * 1024; // 10 MB
    if content.len() > MAX_IMPORT_SIZE {
        return Err(CommandError {
            code: "VALIDATION_ERROR".to_string(),
            message: "Import content exceeds 10 MB limit".to_string(),
        });
    }

    let conflict = parse_conflict_resolution(&conflict_resolution)?;
    let fmt = if format == "auto" {
        ImportManager::detect_format(&content)?
    } else {
        parse_import_format(&format)?
    };

    match fmt {
        ImportFormat::BeeftextJson => {
            Ok(ImportManager::import_beeftext_json(&content, conflict)?)
        }
        ImportFormat::BeeftextCsv => {
            Ok(ImportManager::import_beeftext_csv(&content, conflict)?)
        }
        ImportFormat::TextExpanderCsv => {
            Ok(ImportManager::import_textexpander_csv(&content, conflict)?)
        }
        ImportFormat::MuttonTextJson => Ok(ImportManager::import_muttontext_json(&content)?),
    }
}

/// Preview what an import would produce.
#[tauri::command]
pub fn preview_import(content: String) -> Result<ImportPreview, CommandError> {
    const MAX_IMPORT_SIZE: usize = 10 * 1024 * 1024; // 10 MB
    if content.len() > MAX_IMPORT_SIZE {
        return Err(CommandError {
            code: "VALIDATION_ERROR".to_string(),
            message: "Import content exceeds 10 MB limit".to_string(),
        });
    }

    Ok(ImportManager::preview_import(&content)?)
}

/// Export combos to the given format.
#[tauri::command]
pub fn export_combos(
    combos: Vec<Combo>,
    groups: Vec<Group>,
    format: String,
) -> Result<String, CommandError> {
    let fmt = parse_export_format(&format)?;
    Ok(ExportManager::export_to_format(&combos, &groups, fmt)?)
}

/// Create a new backup.
#[tauri::command]
pub fn create_backup(
    state: tauri::State<'_, super::AppState>,
    backup_state: tauri::State<'_, BackupState>,
) -> Result<BackupInfo, CommandError> {
    let combo_mgr = state.combo_manager.lock().map_err(|_| CommandError {
        code: "INTERNAL_ERROR".to_string(),
        message: "Lock poisoned".to_string(),
    })?;
    let combos = combo_mgr.get_all_combos();
    let groups = combo_mgr.get_all_groups();
    let prefs = serde_json::json!({});

    let backup_mgr = backup_state.backup_manager.lock().map_err(|_| CommandError {
        code: "INTERNAL_ERROR".to_string(),
        message: "Lock poisoned".to_string(),
    })?;
    Ok(backup_mgr.create_backup(&combos, &groups, &prefs)?)
}

/// Restore a backup by ID.
#[tauri::command]
pub fn restore_backup(
    backup_state: tauri::State<'_, BackupState>,
    backup_id: String,
) -> Result<serde_json::Value, CommandError> {
    let backup_mgr = backup_state.backup_manager.lock().map_err(|_| CommandError {
        code: "INTERNAL_ERROR".to_string(),
        message: "Lock poisoned".to_string(),
    })?;
    let data = backup_mgr.restore_backup(&backup_id)?;
    serde_json::to_value(&data).map_err(|e| CommandError {
        code: "SERIALIZATION_ERROR".to_string(),
        message: e.to_string(),
    })
}

/// List all available backups.
#[tauri::command]
pub fn list_backups(
    backup_state: tauri::State<'_, BackupState>,
) -> Result<Vec<BackupInfo>, CommandError> {
    let backup_mgr = backup_state.backup_manager.lock().map_err(|_| CommandError {
        code: "INTERNAL_ERROR".to_string(),
        message: "Lock poisoned".to_string(),
    })?;
    Ok(backup_mgr.list_backups()?)
}

/// Delete a backup by ID.
#[tauri::command]
pub fn delete_backup(
    backup_state: tauri::State<'_, BackupState>,
    backup_id: String,
) -> Result<(), CommandError> {
    let backup_mgr = backup_state.backup_manager.lock().map_err(|_| CommandError {
        code: "INTERNAL_ERROR".to_string(),
        message: "Lock poisoned".to_string(),
    })?;
    Ok(backup_mgr.delete_backup(&backup_id)?)
}

/// Check for available updates given latest version info.
#[tauri::command]
pub fn check_for_updates(
    update_state: tauri::State<'_, UpdateState>,
    latest: VersionInfo,
) -> Result<bool, CommandError> {
    let mgr = update_state.update_manager.lock().map_err(|_| CommandError {
        code: "INTERNAL_ERROR".to_string(),
        message: "Lock poisoned".to_string(),
    })?;
    Ok(mgr.check_update_available(&latest))
}

/// Skip a specific version for updates.
#[tauri::command]
pub fn skip_update_version(
    update_state: tauri::State<'_, UpdateState>,
    version: String,
) -> Result<(), CommandError> {
    let mut mgr = update_state.update_manager.lock().map_err(|_| CommandError {
        code: "INTERNAL_ERROR".to_string(),
        message: "Lock poisoned".to_string(),
    })?;
    mgr.skip_version(&version);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── Format Parsing ───────────────────────────────────────────

    #[test]
    fn test_parse_import_format_valid() {
        assert_eq!(parse_import_format("beeftextJson").unwrap(), ImportFormat::BeeftextJson);
        assert_eq!(parse_import_format("beeftextCsv").unwrap(), ImportFormat::BeeftextCsv);
        assert_eq!(parse_import_format("textExpanderCsv").unwrap(), ImportFormat::TextExpanderCsv);
        assert_eq!(parse_import_format("muttonTextJson").unwrap(), ImportFormat::MuttonTextJson);
    }

    #[test]
    fn test_parse_import_format_invalid() {
        assert!(parse_import_format("badFormat").is_err());
    }

    #[test]
    fn test_parse_conflict_resolution_valid() {
        assert_eq!(parse_conflict_resolution("skip").unwrap(), ConflictResolution::Skip);
        assert_eq!(parse_conflict_resolution("overwrite").unwrap(), ConflictResolution::Overwrite);
        assert_eq!(parse_conflict_resolution("rename").unwrap(), ConflictResolution::Rename);
    }

    #[test]
    fn test_parse_conflict_resolution_invalid() {
        assert!(parse_conflict_resolution("bad").is_err());
    }

    #[test]
    fn test_parse_export_format_valid() {
        assert_eq!(parse_export_format("muttonTextJson").unwrap(), ExportFormat::MuttonTextJson);
        assert_eq!(parse_export_format("textExpanderCsv").unwrap(), ExportFormat::TextExpanderCsv);
        assert_eq!(parse_export_format("cheatsheetCsv").unwrap(), ExportFormat::CheatsheetCsv);
    }

    #[test]
    fn test_parse_export_format_invalid() {
        assert!(parse_export_format("bad").is_err());
    }

    // ── ImportResult Serialization ───────────────────────────────

    #[test]
    fn test_import_result_serialization() {
        let result = ImportResult {
            imported_count: 5,
            skipped_count: 1,
            errors: vec!["test error".to_string()],
            combos: Vec::new(),
            groups: Vec::new(),
        };
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("importedCount"));
        assert!(json.contains("skippedCount"));
    }

    // ── ExportFormat Serialization ───────────────────────────────

    #[test]
    fn test_export_format_serialization() {
        let json = serde_json::to_string(&ExportFormat::MuttonTextJson).unwrap();
        assert_eq!(json, r#""muttonTextJson""#);
    }

    // ── ImportFormat Serialization ───────────────────────────────

    #[test]
    fn test_import_format_serialization() {
        let json = serde_json::to_string(&ImportFormat::BeeftextJson).unwrap();
        assert_eq!(json, r#""beeftextJson""#);
    }

    // ── Command Error From conversions ───────────────────────────

    #[test]
    fn test_import_error_to_command_error() {
        let err: CommandError = crate::managers::import_manager::ImportError::UnrecognizedFormat.into();
        assert_eq!(err.code, "IMPORT_ERROR");
    }

    #[test]
    fn test_export_error_to_command_error() {
        let err: CommandError =
            crate::managers::export_manager::ExportError::Serialization("test".to_string()).into();
        assert_eq!(err.code, "EXPORT_ERROR");
    }

    #[test]
    fn test_backup_error_to_command_error() {
        let err: CommandError =
            crate::managers::backup_manager::BackupError::NotFound("test".to_string()).into();
        assert_eq!(err.code, "BACKUP_ERROR");
    }

    // ── Import/Export commands (no Tauri state) ──────────────────

    #[test]
    fn test_import_combos_command() {
        let content = r#"{"combos":[{"keyword":"sig","snippet":"hello"}],"groups":[]}"#;
        let result = import_combos(
            content.to_string(),
            "beeftextJson".to_string(),
            "skip".to_string(),
        )
        .unwrap();
        assert_eq!(result.imported_count, 1);
    }

    #[test]
    fn test_import_combos_auto_detect() {
        let content = r#"{"combos":[{"keyword":"sig","snippet":"hello"}],"groups":[]}"#;
        let result = import_combos(
            content.to_string(),
            "auto".to_string(),
            "skip".to_string(),
        )
        .unwrap();
        assert!(result.imported_count >= 0);
    }

    #[test]
    fn test_preview_import_command() {
        let content = r#"{"combos":[{"keyword":"sig","snippet":"hello"}],"groups":[{"name":"G"}]}"#;
        let preview = preview_import(content.to_string()).unwrap();
        assert_eq!(preview.combo_count, 1);
    }

    #[test]
    fn test_export_combos_command() {
        use crate::models::combo::ComboBuilder;
        use crate::models::group::Group;

        let group = Group::new("Test");
        let combo = ComboBuilder::new()
            .keyword("sig")
            .snippet("hello")
            .group_id(group.id)
            .build()
            .unwrap();
        let result = export_combos(
            vec![combo],
            vec![group],
            "muttonTextJson".to_string(),
        )
        .unwrap();
        assert!(result.contains("sig"));
    }

    // ── Import Size Limit ────────────────────────────────────────

    #[test]
    fn test_import_combos_size_limit() {
        let huge_content = "x".repeat(11 * 1024 * 1024); // 11 MB - exceeds 10 MB limit
        let result = import_combos(
            huge_content,
            "muttonTextJson".to_string(),
            "skip".to_string(),
        );
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.code, "VALIDATION_ERROR");
        assert!(err.message.contains("10 MB limit"));
    }

    #[test]
    fn test_preview_import_size_limit() {
        let huge_content = "x".repeat(11 * 1024 * 1024); // 11 MB - exceeds 10 MB limit
        let result = preview_import(huge_content);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.code, "VALIDATION_ERROR");
        assert!(err.message.contains("10 MB limit"));
    }

    #[test]
    fn test_import_combos_within_size_limit() {
        let content = r#"{"combos":[],"groups":[]}"#;
        // This is well within the 10 MB limit
        let result = import_combos(
            content.to_string(),
            "muttonTextJson".to_string(),
            "skip".to_string(),
        );
        assert!(result.is_ok());
    }
}
