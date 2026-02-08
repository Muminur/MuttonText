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
        const MAX_EXCLUDED_APPS: usize = 100;
        if self.preferences.excluded_apps.len() >= MAX_EXCLUDED_APPS {
            return Err(PreferencesError::Validation(
                "Maximum of 100 excluded apps reached".to_string(),
            ));
        }
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
        if prefs.backup_interval_hours > 8760 {
            return Err(PreferencesError::Validation(
                "Backup interval cannot exceed 8760 hours (1 year)".to_string(),
            ));
        }
        if prefs.max_backups == 0 {
            return Err(PreferencesError::Validation(
                "Max backups must be greater than 0".to_string(),
            ));
        }
        if prefs.max_backups > 1000 {
            return Err(PreferencesError::Validation(
                "Max backups cannot exceed 1000".to_string(),
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

    // ── Excluded Apps Limit ──────────────────────────────────────

    #[test]
    fn test_add_excluded_app_max_limit() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("prefs.json");
        let mut mgr = PreferencesManager::new(path).unwrap();

        // Add 100 apps (the limit)
        for i in 0..100 {
            mgr.add_excluded_app(format!("app{}", i)).unwrap();
        }

        // The 101st should fail
        let result = mgr.add_excluded_app("app101".to_string());
        assert!(result.is_err());
        match result.unwrap_err() {
            PreferencesError::Validation(msg) => {
                assert!(msg.contains("100 excluded apps"));
            }
            _ => panic!("Expected Validation error"),
        }
    }

    // ── Preferences Bounds ───────────────────────────────────────

    #[test]
    fn test_update_rejects_excessive_backup_interval() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("prefs.json");
        let mut mgr = PreferencesManager::new(path).unwrap();

        let mut prefs = Preferences::default();
        prefs.backup_interval_hours = 8761; // Exceeds 8760 (1 year)
        let result = mgr.update(prefs);
        assert!(result.is_err());
        match result.unwrap_err() {
            PreferencesError::Validation(msg) => {
                assert!(msg.contains("8760"));
            }
            _ => panic!("Expected Validation error"),
        }
    }

    #[test]
    fn test_update_rejects_excessive_max_backups() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("prefs.json");
        let mut mgr = PreferencesManager::new(path).unwrap();

        let mut prefs = Preferences::default();
        prefs.max_backups = 1001; // Exceeds 1000
        let result = mgr.update(prefs);
        assert!(result.is_err());
        match result.unwrap_err() {
            PreferencesError::Validation(msg) => {
                assert!(msg.contains("1000"));
            }
            _ => panic!("Expected Validation error"),
        }
    }

    #[test]
    fn test_update_accepts_max_valid_values() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("prefs.json");
        let mut mgr = PreferencesManager::new(path).unwrap();

        let mut prefs = Preferences::default();
        prefs.backup_interval_hours = 8760; // Exactly at limit
        prefs.max_backups = 1000; // Exactly at limit
        let result = mgr.update(prefs);
        assert!(result.is_ok());
    }

    // ── Per-Field Save/Load Roundtrip ───────────────────────────────

    #[test]
    fn test_every_field_save_load_roundtrip() {
        use crate::models::preferences::{PasteMethod, Theme};
        use crate::models::matching::MatchingMode;

        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("prefs.json");
        let mut mgr = PreferencesManager::new(path.clone()).unwrap();

        let custom = Preferences {
            enabled: false,
            play_sound: true,
            show_system_tray: false,
            start_at_login: true,
            start_minimized: true,
            default_matching_mode: MatchingMode::Loose,
            default_case_sensitive: true,
            combo_trigger_shortcut: "Ctrl+Shift+E".to_string(),
            picker_shortcut: "Ctrl+Alt+P".to_string(),
            paste_method: PasteMethod::SimulateKeystrokes,
            theme: Theme::Dark,
            backup_enabled: false,
            backup_interval_hours: 48,
            max_backups: 25,
            auto_check_updates: false,
            excluded_apps: vec!["1password".to_string(), "keepass".to_string()],
        };
        mgr.update(custom.clone()).unwrap();

        let loaded = PreferencesManager::load(&path).unwrap();
        assert_eq!(loaded.enabled, false);
        assert_eq!(loaded.play_sound, true);
        assert_eq!(loaded.show_system_tray, false);
        assert_eq!(loaded.start_at_login, true);
        assert_eq!(loaded.start_minimized, true);
        assert_eq!(loaded.default_matching_mode, MatchingMode::Loose);
        assert_eq!(loaded.default_case_sensitive, true);
        assert_eq!(loaded.combo_trigger_shortcut, "Ctrl+Shift+E");
        assert_eq!(loaded.picker_shortcut, "Ctrl+Alt+P");
        assert_eq!(loaded.paste_method, PasteMethod::SimulateKeystrokes);
        assert_eq!(loaded.theme, Theme::Dark);
        assert_eq!(loaded.backup_enabled, false);
        assert_eq!(loaded.backup_interval_hours, 48);
        assert_eq!(loaded.max_backups, 25);
        assert_eq!(loaded.auto_check_updates, false);
        assert_eq!(loaded.excluded_apps, vec!["1password", "keepass"]);
    }

    #[test]
    fn test_paste_method_xdotool_type_roundtrip() {
        use crate::models::preferences::PasteMethod;

        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("prefs.json");
        let mut mgr = PreferencesManager::new(path.clone()).unwrap();

        let mut prefs = Preferences::default();
        prefs.paste_method = PasteMethod::XdotoolType;
        mgr.update(prefs).unwrap();

        let loaded = PreferencesManager::load(&path).unwrap();
        assert_eq!(loaded.paste_method, PasteMethod::XdotoolType);
    }

    #[test]
    fn test_theme_variants_roundtrip() {
        use crate::models::preferences::Theme;

        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("prefs.json");
        let mut mgr = PreferencesManager::new(path.clone()).unwrap();

        for theme in [Theme::System, Theme::Light, Theme::Dark] {
            let mut prefs = Preferences::default();
            prefs.theme = theme;
            mgr.update(prefs).unwrap();

            let loaded = PreferencesManager::load(&path).unwrap();
            assert_eq!(loaded.theme, theme, "Theme {:?} failed roundtrip", theme);
        }
    }

    #[test]
    fn test_paste_method_variants_roundtrip() {
        use crate::models::preferences::PasteMethod;

        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("prefs.json");
        let mut mgr = PreferencesManager::new(path.clone()).unwrap();

        for method in [PasteMethod::Clipboard, PasteMethod::SimulateKeystrokes, PasteMethod::XdotoolType] {
            let mut prefs = Preferences::default();
            prefs.paste_method = method;
            mgr.update(prefs).unwrap();

            let loaded = PreferencesManager::load(&path).unwrap();
            assert_eq!(loaded.paste_method, method, "PasteMethod {:?} failed roundtrip", method);
        }
    }

    #[test]
    fn test_matching_mode_variants_roundtrip() {
        use crate::models::matching::MatchingMode;

        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("prefs.json");
        let mut mgr = PreferencesManager::new(path.clone()).unwrap();

        for mode in [MatchingMode::Strict, MatchingMode::Loose] {
            let mut prefs = Preferences::default();
            prefs.default_matching_mode = mode;
            mgr.update(prefs).unwrap();

            let loaded = PreferencesManager::load(&path).unwrap();
            assert_eq!(loaded.default_matching_mode, mode, "MatchingMode {:?} failed roundtrip", mode);
        }
    }

    #[test]
    fn test_defaults_are_correct() {
        use crate::models::preferences::{PasteMethod, Theme};
        use crate::models::matching::MatchingMode;

        let prefs = Preferences::default();
        assert_eq!(prefs.enabled, true);
        assert_eq!(prefs.play_sound, false);
        assert_eq!(prefs.show_system_tray, true);
        assert_eq!(prefs.start_at_login, false);
        assert_eq!(prefs.start_minimized, false);
        assert_eq!(prefs.default_matching_mode, MatchingMode::Strict);
        assert_eq!(prefs.default_case_sensitive, false);
        assert_eq!(prefs.combo_trigger_shortcut, "");
        assert_eq!(prefs.picker_shortcut, "Ctrl+Shift+Space");
        assert_eq!(prefs.paste_method, PasteMethod::Clipboard);
        assert_eq!(prefs.theme, Theme::System);
        assert_eq!(prefs.backup_enabled, true);
        assert_eq!(prefs.backup_interval_hours, 24);
        assert_eq!(prefs.max_backups, 10);
        assert_eq!(prefs.auto_check_updates, true);
        assert!(prefs.excluded_apps.is_empty());
    }

    #[test]
    fn test_reset_restores_all_fields() {
        use crate::models::preferences::{PasteMethod, Theme};
        use crate::models::matching::MatchingMode;

        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("prefs.json");
        let mut mgr = PreferencesManager::new(path).unwrap();

        let custom = Preferences {
            enabled: false,
            play_sound: true,
            show_system_tray: false,
            start_at_login: true,
            start_minimized: true,
            default_matching_mode: MatchingMode::Loose,
            default_case_sensitive: true,
            combo_trigger_shortcut: "Ctrl+X".to_string(),
            picker_shortcut: "Alt+P".to_string(),
            paste_method: PasteMethod::XdotoolType,
            theme: Theme::Dark,
            backup_enabled: false,
            backup_interval_hours: 72,
            max_backups: 50,
            auto_check_updates: false,
            excluded_apps: vec!["app1".to_string()],
        };
        mgr.update(custom).unwrap();

        mgr.reset_to_defaults().unwrap();

        let prefs = mgr.get();
        assert_eq!(*prefs, Preferences::default());
    }
}
