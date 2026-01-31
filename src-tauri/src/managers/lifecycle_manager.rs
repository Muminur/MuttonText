//! Application lifecycle management: single instance, first run, autostart.

use std::fs::{self, File};
use std::path::{Path, PathBuf};

use fs2::FileExt;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Errors from lifecycle operations.
#[derive(Debug, Error)]
pub enum LifecycleError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Another instance is already running")]
    AlreadyRunning,

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

/// Configuration for autostart behavior.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutostartConfig {
    /// Whether autostart is enabled.
    pub enabled: bool,
    /// Whether to start minimized to tray.
    pub minimized: bool,
}

impl Default for AutostartConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            minimized: false,
        }
    }
}

const LOCK_FILENAME: &str = "muttontext.lock";
const FIRST_RUN_MARKER: &str = ".first_run_complete";
const AUTOSTART_CONFIG_FILENAME: &str = "autostart.json";

/// Manages application lifecycle concerns.
#[derive(Debug)]
pub struct LifecycleManager {
    /// The lock file handle, kept alive to hold the lock.
    _lock_file: File,
    /// Autostart configuration.
    autostart_config: AutostartConfig,
    /// Directory for storing lifecycle files.
    app_dir: PathBuf,
}

impl LifecycleManager {
    /// Tries to acquire a single-instance file lock.
    ///
    /// Returns `Err(LifecycleError::AlreadyRunning)` if another instance holds the lock.
    pub fn try_acquire_lock(app_dir: &Path) -> Result<Self, LifecycleError> {
        fs::create_dir_all(app_dir)?;
        let lock_path = app_dir.join(LOCK_FILENAME);
        let lock_file = File::create(&lock_path)?;

        lock_file
            .try_lock_exclusive()
            .map_err(|_| LifecycleError::AlreadyRunning)?;

        // Load autostart config if it exists
        let autostart_path = app_dir.join(AUTOSTART_CONFIG_FILENAME);
        let autostart_config = if autostart_path.exists() {
            let content = fs::read_to_string(&autostart_path)?;
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            AutostartConfig::default()
        };

        Ok(Self {
            _lock_file: lock_file,
            autostart_config,
            app_dir: app_dir.to_path_buf(),
        })
    }

    /// Returns true if this is the first time the app has been run.
    pub fn is_first_run(app_dir: &Path) -> bool {
        !app_dir.join(FIRST_RUN_MARKER).exists()
    }

    /// Marks the first run as complete by creating a marker file.
    pub fn mark_first_run_complete(app_dir: &Path) -> Result<(), LifecycleError> {
        fs::create_dir_all(app_dir)?;
        fs::write(app_dir.join(FIRST_RUN_MARKER), "")?;
        Ok(())
    }

    /// Returns the current autostart configuration.
    pub fn get_autostart_config(&self) -> &AutostartConfig {
        &self.autostart_config
    }

    /// Sets the autostart configuration and persists it.
    pub fn set_autostart(&mut self, config: AutostartConfig) -> Result<(), LifecycleError> {
        self.autostart_config = config;
        let path = self.app_dir.join(AUTOSTART_CONFIG_FILENAME);
        let json = serde_json::to_string_pretty(&self.autostart_config)?;
        fs::write(path, json)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_acquire_lock_succeeds() {
        let tmp = tempfile::tempdir().unwrap();
        let mgr = LifecycleManager::try_acquire_lock(tmp.path());
        assert!(mgr.is_ok());
    }

    #[test]
    fn test_second_lock_fails() {
        let tmp = tempfile::tempdir().unwrap();
        let _mgr1 = LifecycleManager::try_acquire_lock(tmp.path()).unwrap();
        let result = LifecycleManager::try_acquire_lock(tmp.path());
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            LifecycleError::AlreadyRunning
        ));
    }

    #[test]
    fn test_lock_released_on_drop() {
        let tmp = tempfile::tempdir().unwrap();
        {
            let _mgr = LifecycleManager::try_acquire_lock(tmp.path()).unwrap();
        }
        // After drop, should be able to acquire again
        let result = LifecycleManager::try_acquire_lock(tmp.path());
        assert!(result.is_ok());
    }

    #[test]
    fn test_first_run_detection() {
        let tmp = tempfile::tempdir().unwrap();
        assert!(LifecycleManager::is_first_run(tmp.path()));

        LifecycleManager::mark_first_run_complete(tmp.path()).unwrap();
        assert!(!LifecycleManager::is_first_run(tmp.path()));
    }

    #[test]
    fn test_autostart_config_default() {
        let tmp = tempfile::tempdir().unwrap();
        let mgr = LifecycleManager::try_acquire_lock(tmp.path()).unwrap();
        let config = mgr.get_autostart_config();
        assert!(!config.enabled);
        assert!(!config.minimized);
    }

    #[test]
    fn test_set_autostart_persists() {
        let tmp = tempfile::tempdir().unwrap();
        {
            let mut mgr = LifecycleManager::try_acquire_lock(tmp.path()).unwrap();
            mgr.set_autostart(AutostartConfig {
                enabled: true,
                minimized: true,
            })
            .unwrap();
        }
        // Reload
        let mgr = LifecycleManager::try_acquire_lock(tmp.path()).unwrap();
        assert!(mgr.get_autostart_config().enabled);
        assert!(mgr.get_autostart_config().minimized);
    }

    #[test]
    fn test_autostart_config_serialization() {
        let config = AutostartConfig {
            enabled: true,
            minimized: false,
        };
        let json = serde_json::to_string(&config).unwrap();
        let deser: AutostartConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(config, deser);
    }

    #[test]
    fn test_lifecycle_error_display() {
        let err = LifecycleError::AlreadyRunning;
        assert!(format!("{err}").contains("already running"));
    }
}
