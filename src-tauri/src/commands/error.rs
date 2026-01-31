//! Error types for Tauri IPC command responses.
//!
//! Tauri requires command errors to implement `serde::Serialize`.
//! `CommandError` provides a structured error type with a code and message
//! that the frontend can parse reliably.

use serde::{Deserialize, Serialize};

use crate::managers::combo_manager::ComboManagerError;
use crate::managers::backup_manager::BackupError;

/// A serializable error type returned by Tauri commands to the frontend.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandError {
    /// A machine-readable error code (e.g. "COMBO_NOT_FOUND").
    pub code: String,
    /// A human-readable error message.
    pub message: String,
}

/// A structured error response for the frontend with optional details.
///
/// This provides a richer error payload than `CommandError` for cases
/// where additional context is helpful.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ErrorResponse {
    /// Machine-readable error code (e.g., "COMBO_NOT_FOUND").
    pub code: String,
    /// Human-readable error message.
    pub message: String,
    /// Optional additional details (context-dependent).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
}

impl ErrorResponse {
    /// Creates a new ErrorResponse.
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            details: None,
        }
    }

    /// Creates a new ErrorResponse with details.
    pub fn with_details(
        code: impl Into<String>,
        message: impl Into<String>,
        details: impl Into<String>,
    ) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            details: Some(details.into()),
        }
    }
}

impl std::fmt::Display for ErrorResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] {}", self.code, self.message)?;
        if let Some(ref details) = self.details {
            write!(f, " ({})", details)?;
        }
        Ok(())
    }
}

impl From<CommandError> for ErrorResponse {
    fn from(err: CommandError) -> Self {
        Self {
            code: err.code,
            message: err.message,
            details: None,
        }
    }
}

impl std::fmt::Display for CommandError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] {}", self.code, self.message)
    }
}

impl From<ComboManagerError> for CommandError {
    fn from(err: ComboManagerError) -> Self {
        match &err {
            ComboManagerError::ComboNotFound(_) => CommandError {
                code: "COMBO_NOT_FOUND".to_string(),
                message: err.to_string(),
            },
            ComboManagerError::GroupNotFound(_) => CommandError {
                code: "GROUP_NOT_FOUND".to_string(),
                message: err.to_string(),
            },
            ComboManagerError::Validation(_) => CommandError {
                code: "VALIDATION_ERROR".to_string(),
                message: err.to_string(),
            },
            ComboManagerError::Storage(_) => CommandError {
                code: "STORAGE_ERROR".to_string(),
                message: err.to_string(),
            },
            ComboManagerError::ValidationMessage(_) => CommandError {
                code: "VALIDATION_ERROR".to_string(),
                message: err.to_string(),
            },
        }
    }
}

impl From<BackupError> for CommandError {
    fn from(err: BackupError) -> Self {
        match &err {
            BackupError::Io(_) => CommandError {
                code: "BACKUP_IO_ERROR".to_string(),
                message: err.to_string(),
            },
            BackupError::Serialization(_) => CommandError {
                code: "BACKUP_SERIALIZATION_ERROR".to_string(),
                message: err.to_string(),
            },
            BackupError::NotFound(_) => CommandError {
                code: "BACKUP_NOT_FOUND".to_string(),
                message: err.to_string(),
            },
            BackupError::InvalidBackup(_) => CommandError {
                code: "BACKUP_INVALID".to_string(),
                message: err.to_string(),
            },
            BackupError::InvalidBackupId(_) => CommandError {
                code: "BACKUP_INVALID_ID".to_string(),
                message: err.to_string(),
            },
        }
    }
}

impl CommandError {
    /// Creates a CommandError for invalid UUID parsing.
    pub fn invalid_uuid(field: &str, value: &str) -> Self {
        CommandError {
            code: "INVALID_UUID".to_string(),
            message: format!("Invalid UUID for {field}: {value}"),
        }
    }

    /// Creates a CommandError for invalid matching mode.
    pub fn invalid_matching_mode(value: &str) -> Self {
        CommandError {
            code: "INVALID_MATCHING_MODE".to_string(),
            message: format!(
                "Invalid matching mode: '{value}'. Expected 'strict' or 'loose'."
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn test_command_error_serializes() {
        let err = CommandError {
            code: "TEST".to_string(),
            message: "test message".to_string(),
        };
        let json = serde_json::to_string(&err).expect("serialize");
        assert!(json.contains("\"code\":\"TEST\""));
        assert!(json.contains("\"message\":\"test message\""));
    }

    #[test]
    fn test_command_error_display() {
        let err = CommandError {
            code: "TEST".to_string(),
            message: "msg".to_string(),
        };
        assert_eq!(format!("{err}"), "[TEST] msg");
    }

    #[test]
    fn test_from_combo_not_found() {
        let id = Uuid::new_v4();
        let err: CommandError = ComboManagerError::ComboNotFound(id).into();
        assert_eq!(err.code, "COMBO_NOT_FOUND");
    }

    #[test]
    fn test_from_group_not_found() {
        let id = Uuid::new_v4();
        let err: CommandError = ComboManagerError::GroupNotFound(id).into();
        assert_eq!(err.code, "GROUP_NOT_FOUND");
    }

    #[test]
    fn test_invalid_uuid_error() {
        let err = CommandError::invalid_uuid("id", "not-a-uuid");
        assert_eq!(err.code, "INVALID_UUID");
        assert!(err.message.contains("not-a-uuid"));
    }

    #[test]
    fn test_invalid_matching_mode_error() {
        let err = CommandError::invalid_matching_mode("bad");
        assert_eq!(err.code, "INVALID_MATCHING_MODE");
        assert!(err.message.contains("bad"));
    }

    // ── MT-1106: ErrorResponse tests ─────────────────────────────

    #[test]
    fn test_error_response_new() {
        let resp = ErrorResponse::new("TEST_CODE", "test message");
        assert_eq!(resp.code, "TEST_CODE");
        assert_eq!(resp.message, "test message");
        assert!(resp.details.is_none());
    }

    #[test]
    fn test_error_response_with_details() {
        let resp = ErrorResponse::with_details("ERR", "msg", "extra info");
        assert_eq!(resp.details, Some("extra info".to_string()));
    }

    #[test]
    fn test_error_response_display() {
        let resp = ErrorResponse::new("CODE", "message");
        assert_eq!(format!("{}", resp), "[CODE] message");

        let resp = ErrorResponse::with_details("CODE", "message", "details");
        assert_eq!(format!("{}", resp), "[CODE] message (details)");
    }

    #[test]
    fn test_error_response_serialization() {
        let resp = ErrorResponse::new("TEST", "msg");
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("\"code\":\"TEST\""));
        assert!(json.contains("\"message\":\"msg\""));
        // details should be omitted when None
        assert!(!json.contains("details"));

        let resp = ErrorResponse::with_details("TEST", "msg", "det");
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("\"details\":\"det\""));
    }

    #[test]
    fn test_error_response_deserialization() {
        let json = r#"{"code":"X","message":"y"}"#;
        let resp: ErrorResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.code, "X");
        assert_eq!(resp.message, "y");
        assert!(resp.details.is_none());
    }

    #[test]
    fn test_error_response_from_command_error() {
        let cmd_err = CommandError {
            code: "CODE".to_string(),
            message: "msg".to_string(),
        };
        let resp: ErrorResponse = cmd_err.into();
        assert_eq!(resp.code, "CODE");
        assert_eq!(resp.message, "msg");
    }

    #[test]
    fn test_from_backup_not_found() {
        let err: CommandError = BackupError::NotFound("abc".to_string()).into();
        assert_eq!(err.code, "BACKUP_NOT_FOUND");
    }

    #[test]
    fn test_from_backup_invalid_id() {
        let err: CommandError = BackupError::InvalidBackupId("../bad".to_string()).into();
        assert_eq!(err.code, "BACKUP_INVALID_ID");
    }

    #[test]
    fn test_from_backup_serialization() {
        let err: CommandError = BackupError::Serialization("bad json".to_string()).into();
        assert_eq!(err.code, "BACKUP_SERIALIZATION_ERROR");
    }
}
