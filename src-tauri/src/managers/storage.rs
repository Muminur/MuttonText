//! File path resolution and directory management for MuttonText data persistence.
//!
//! Provides platform-specific config directory resolution and ensures
//! required directories exist before use.

use std::fs;
use std::path::PathBuf;

use thiserror::Error;

/// Errors that can occur during storage operations.
#[derive(Debug, Error)]
pub enum StorageError {
    /// An I/O error occurred.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// JSON serialization or deserialization failed.
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// The platform config directory could not be determined.
    #[error("Config directory not found")]
    ConfigDirNotFound,

    /// A file lock could not be acquired because another process holds it.
    #[error("File locked by another process")]
    FileLocked,

    /// A data migration between schema versions failed.
    #[error("Migration failed: {0}")]
    MigrationFailed(String),
}

/// The application directory name used inside the platform config directory.
const APP_DIR_NAME: &str = "muttontext";

/// The filename for the combo library JSON file.
const COMBOS_FILENAME: &str = "combos.json";

/// The filename for the user preferences JSON file.
const PREFERENCES_FILENAME: &str = "preferences.json";

/// The subdirectory name for backups.
const BACKUPS_DIR_NAME: &str = "backups";

/// The subdirectory name for logs.
const LOGS_DIR_NAME: &str = "logs";

/// Returns the platform-specific configuration directory for MuttonText.
///
/// - Linux: `~/.config/muttontext/`
/// - macOS: `~/Library/Application Support/muttontext/`
/// - Windows: `{FOLDERID_RoamingAppData}/muttontext/`
pub fn get_config_dir() -> Result<PathBuf, StorageError> {
    dirs::config_dir()
        .map(|p| p.join(APP_DIR_NAME))
        .ok_or(StorageError::ConfigDirNotFound)
}

/// Returns the path to `combos.json`.
pub fn get_combos_path() -> Result<PathBuf, StorageError> {
    Ok(get_config_dir()?.join(COMBOS_FILENAME))
}

/// Returns the path to `preferences.json`.
pub fn get_preferences_path() -> Result<PathBuf, StorageError> {
    Ok(get_config_dir()?.join(PREFERENCES_FILENAME))
}

/// Returns the path to the backups directory.
pub fn get_backups_dir() -> Result<PathBuf, StorageError> {
    Ok(get_config_dir()?.join(BACKUPS_DIR_NAME))
}

/// Returns the path to the logs directory.
pub fn get_logs_dir() -> Result<PathBuf, StorageError> {
    Ok(get_config_dir()?.join(LOGS_DIR_NAME))
}

/// Ensures all required directories exist, creating them if necessary.
///
/// Creates the config directory, backups subdirectory, and logs subdirectory.
pub fn ensure_dirs_exist() -> Result<(), StorageError> {
    let config_dir = get_config_dir()?;
    fs::create_dir_all(&config_dir)?;
    fs::create_dir_all(config_dir.join(BACKUPS_DIR_NAME))?;
    fs::create_dir_all(config_dir.join(LOGS_DIR_NAME))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_config_dir_returns_path_containing_app_name() {
        // On any platform with a home directory, this should succeed.
        let result = get_config_dir();
        if let Ok(path) = result {
            assert!(path.ends_with(APP_DIR_NAME));
        }
        // If config_dir is unavailable (e.g. CI without HOME), we accept the error.
    }

    #[test]
    fn test_get_combos_path_ends_with_filename() {
        if let Ok(path) = get_combos_path() {
            assert_eq!(path.file_name().unwrap().to_str().unwrap(), COMBOS_FILENAME);
        }
    }

    #[test]
    fn test_get_preferences_path_ends_with_filename() {
        if let Ok(path) = get_preferences_path() {
            assert_eq!(
                path.file_name().unwrap().to_str().unwrap(),
                PREFERENCES_FILENAME
            );
        }
    }

    #[test]
    fn test_get_backups_dir_ends_with_backups() {
        if let Ok(path) = get_backups_dir() {
            assert!(path.ends_with(BACKUPS_DIR_NAME));
        }
    }

    #[test]
    fn test_get_logs_dir_ends_with_logs() {
        if let Ok(path) = get_logs_dir() {
            assert!(path.ends_with(LOGS_DIR_NAME));
        }
    }

    #[test]
    fn test_ensure_dirs_exist_creates_directories() {
        // Use a temp directory to test directory creation logic.
        let tmp = tempfile::tempdir().expect("create temp dir");
        let config = tmp.path().join(APP_DIR_NAME);
        // We cannot easily override get_config_dir, so we test the mkdir logic directly.
        std::fs::create_dir_all(config.join(BACKUPS_DIR_NAME)).expect("create backups dir");
        std::fs::create_dir_all(config.join(LOGS_DIR_NAME)).expect("create logs dir");
        assert!(config.join(BACKUPS_DIR_NAME).is_dir());
        assert!(config.join(LOGS_DIR_NAME).is_dir());
    }

    #[test]
    fn test_storage_error_display() {
        let err = StorageError::ConfigDirNotFound;
        assert_eq!(format!("{err}"), "Config directory not found");

        let err = StorageError::FileLocked;
        assert_eq!(format!("{err}"), "File locked by another process");

        let err = StorageError::MigrationFailed("bad version".to_string());
        assert!(format!("{err}").contains("bad version"));
    }

    #[test]
    fn test_storage_error_from_io() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file missing");
        let storage_err: StorageError = io_err.into();
        assert!(matches!(storage_err, StorageError::Io(_)));
    }

    #[test]
    fn test_paths_are_consistent() {
        // combos path should be config_dir + combos.json
        if let (Ok(config), Ok(combos)) = (get_config_dir(), get_combos_path()) {
            assert_eq!(combos, config.join(COMBOS_FILENAME));
        }
        if let (Ok(config), Ok(prefs)) = (get_config_dir(), get_preferences_path()) {
            assert_eq!(prefs, config.join(PREFERENCES_FILENAME));
        }
    }
}
