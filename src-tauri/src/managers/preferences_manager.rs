//! High-level preferences management with validation and convenience methods.

use std::path::{Path, PathBuf};

use thiserror::Error;
use tracing;

use crate::models::preferences::Preferences;

/// Errors from preferences management operations.
#[derive(Debug, Error)]
pub enum PreferencesError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("App already excluded: {0}")]
    AppAlreadyExcluded(String),
}

/// Manages user preferences with load/save/validation.
pub struct PreferencesManager {
    preferences: Preferences,
    storage_path: PathBuf,
}

impl PreferencesManager {
    /// Creates a new manager, loading from `storage_path` or using defaults.
    pub fn new(storage_path: PathBuf) -> Result<Self, PreferencesError> {
        let preferences = Self::load(&storage_path)?;
        Ok(Self {
            preferences,
            storage_path,
        })
    }

    /// Returns a reference to the current preferences.
    pub fn get(&self) -> &Preferences {
        &self.preferences
    }

    /// Updates preferences after validation and saves to disk.
    pub fn update(&mut self, prefs: Preferences) -> Result<(), PreferencesError> {
        Self::validate(&prefs)?;
        self.preferences = prefs;
        self.save()
    }

    /// Saves current preferences to disk.
    pub fn save(&self) -> Result<(), PreferencesError> {
        if let Some(parent) = self.storage_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(&self.preferences)?;
        std::fs::write(&self.storage_path, json)?;
        tracing::debug!("Preferences saved to {:?}", self.storage_path);
        Ok(())
    }

    /// Loads preferences from the given path, returning defaults if not found.
    pub fn load(path: &Path) -> Result<Preferences, PreferencesError> {
        if !path.exists() {
            tracing::info!("Preferences file not found at {:?}, using defaults", path);
            return Ok(Preferences::default());
        }
        let content = std::fs::read_to_string(path)?;
        let prefs: Preferences = serde_json::from_str(&content)?;
        Ok(prefs)
    }

    /// Resets preferences to defaults and saves.
    pub fn reset_to_defaults(&mut self) -> Result<(), PreferencesError> {
        self.preferences = Preferences::default();
        self.save()
    }

    /// Returns the list of excluded application names.
    pub fn get_excluded_apps(&self) -> &[String] {
        &self.preferences.excluded_apps
    }

    /// Adds an app to the exclusion list. Returns error if already present.
    pub fn add_excluded_app(&mut self, app: String) -> Result<(), PreferencesError> {
        if self.preferences.excluded_apps.iter().any(|a| a == &app) {
            return Err(PreferencesError::AppAlreadyExcluded(app));
        }
        self.preferences.excluded_apps.push(app);
        self.save()
    }

    /// Removes an app from the exclusion list. Returns whether it was found.
    pub fn remove_excluded_app(&mut self, app: &str) -> Result<bool, PreferencesError> {
        let len_before = self.preferences.excluded_apps.len();
        self.preferences.excluded_apps.retain(|a| a != app);
        let removed = self.preferences.excluded_apps.len() < len_before;
        if removed {
            self.save()?;
        }
        Ok(removed)
    }

    /// Validates preferences values.
    fn validate(prefs: &Preferences) -> Result<(), PreferencesError> {
        if prefs.backup_interval_hours == 0 {
            return Err(PreferencesError::Validation(
                "Backup interval must be greater than 0".to_string(),
            ));
        }
        if prefs.max_backups == 0 {
            return Err(PreferencesError::Validation(
                "Max backups must be greater than 0".to_string(),
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_returns_defaults_when_no_file() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("prefs.json");
        let mgr = PreferencesManager::new(path).unwrap();
        assert!(mgr.get().enabled);
    }

    #[test]
    fn test_save_and_load_roundtrip() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("prefs.json");
        let mgr = PreferencesManager::new(path.clone()).unwrap();
        mgr.save().unwrap();

        let loaded = PreferencesManager::load(&path).unwrap();
        assert_eq!(*mgr.get(), loaded);
    }

    #[test]
    fn test_update_saves_to_disk() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("prefs.json");
        let mut mgr = PreferencesManager::new(path.clone()).unwrap();

        let mut prefs = Preferences::default();
        prefs.play_sound = true;
        mgr.update(prefs).unwrap();

        let loaded = PreferencesManager::load(&path).unwrap();
        assert!(loaded.play_sound);
    }

    #[test]
    fn test_update_rejects_zero_backup_interval() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("prefs.json");
        let mut mgr = PreferencesManager::new(path).unwrap();

        let mut prefs = Preferences::default();
        prefs.backup_interval_hours = 0;
        let result = mgr.update(prefs);
        assert!(result.is_err());
    }

    #[test]
    fn test_update_rejects_zero_max_backups() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("prefs.json");
        let mut mgr = PreferencesManager::new(path).unwrap();

        let mut prefs = Preferences::default();
        prefs.max_backups = 0;
        let result = mgr.update(prefs);
        assert!(result.is_err());
    }

    #[test]
    fn test_reset_to_defaults() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("prefs.json");
        let mut mgr = PreferencesManager::new(path).unwrap();

        let mut prefs = Preferences::default();
        prefs.play_sound = true;
        prefs.max_backups = 99;
        mgr.update(prefs).unwrap();

        mgr.reset_to_defaults().unwrap();
        assert!(!mgr.get().play_sound);
        assert_eq!(mgr.get().max_backups, 10);
    }

    #[test]
    fn test_excluded_apps_crud() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("prefs.json");
        let mut mgr = PreferencesManager::new(path).unwrap();

        assert!(mgr.get_excluded_apps().is_empty());

        mgr.add_excluded_app("1password".to_string()).unwrap();
        assert_eq!(mgr.get_excluded_apps(), &["1password"]);

        mgr.add_excluded_app("keepass".to_string()).unwrap();
        assert_eq!(mgr.get_excluded_apps().len(), 2);

        // Duplicate should fail
        let result = mgr.add_excluded_app("1password".to_string());
        assert!(result.is_err());

        let removed = mgr.remove_excluded_app("1password").unwrap();
        assert!(removed);
        assert_eq!(mgr.get_excluded_apps(), &["keepass"]);

        let removed = mgr.remove_excluded_app("nonexistent").unwrap();
        assert!(!removed);
    }

    #[test]
    fn test_load_nonexistent_returns_defaults() {
        let path = Path::new("/tmp/does_not_exist_muttontext_test.json");
        let prefs = PreferencesManager::load(path).unwrap();
        assert_eq!(prefs, Preferences::default());
    }

    #[test]
    fn test_preferences_error_display() {
        let err = PreferencesError::Validation("bad".to_string());
        assert!(format!("{err}").contains("bad"));

        let err = PreferencesError::AppAlreadyExcluded("app".to_string());
        assert!(format!("{err}").contains("app"));
    }
}
