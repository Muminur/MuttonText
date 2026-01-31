//! Error types for Tauri IPC command responses.
//!
//! Tauri requires command errors to implement `serde::Serialize`.
//! `CommandError` provides a structured error type with a code and message
//! that the frontend can parse reliably.

use serde::Serialize;

use crate::managers::combo_manager::ComboManagerError;

/// A serializable error type returned by Tauri commands to the frontend.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandError {
    /// A machine-readable error code (e.g. "COMBO_NOT_FOUND").
    pub code: String,
    /// A human-readable error message.
    pub message: String,
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
}
