//! Persistence for user preferences.
//!
//! Reads and writes `preferences.json` with atomic writes, file locking,
//! and schema version migration support.

use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;

use fs2::FileExt;
use serde_json::Value;
use tracing;

use crate::models::preferences::Preferences;

use super::storage::StorageError;

/// Current schema version for the preferences on-disk format.
const CURRENT_SCHEMA_VERSION: u32 = 1;

/// Key used in the JSON envelope for schema version.
const SCHEMA_VERSION_KEY: &str = "schemaVersion";

/// Manages loading and saving user preferences to disk.
pub struct PreferencesStorage {
    path: PathBuf,
}

impl PreferencesStorage {
    /// Creates a new `PreferencesStorage` that reads from and writes to `path`.
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    /// Loads preferences from disk.
    ///
    /// If the file does not exist, returns `Preferences::default()`.
    /// Acquires a shared file lock during the read.
    /// Performs schema migration if the on-disk version is older.
    pub fn load(&self) -> Result<Preferences, StorageError> {
        if !self.path.exists() {
            tracing::info!("Preferences file not found, returning defaults");
            return Ok(Preferences::default());
        }

        let file = File::open(&self.path)?;
        file.lock_shared()
            .map_err(|_| StorageError::FileLocked)?;

        let content = fs::read_to_string(&self.path)?;
        drop(file);

        let mut json_value: Value = serde_json::from_str(&content)?;
        let on_disk_version = json_value
            .get(SCHEMA_VERSION_KEY)
            .and_then(|v| v.as_u64())
            .unwrap_or(1) as u32;

        if on_disk_version < CURRENT_SCHEMA_VERSION {
            tracing::info!(
                from = on_disk_version,
                to = CURRENT_SCHEMA_VERSION,
                "Migrating preferences schema"
            );
            json_value =
                migrate_preferences(json_value, on_disk_version, CURRENT_SCHEMA_VERSION)?;
        }

        let prefs: Preferences = serde_json::from_value(json_value)?;
        Ok(prefs)
    }

    /// Saves preferences to disk.
    ///
    /// Performs an atomic write: writes to a temporary file, fsyncs, then renames.
    /// Embeds the current schema version in the output JSON.
    pub fn save(&self, prefs: &Preferences) -> Result<(), StorageError> {
        let mut json_value = serde_json::to_value(prefs)?;
        if let Some(obj) = json_value.as_object_mut() {
            obj.insert(
                SCHEMA_VERSION_KEY.to_string(),
                Value::Number(CURRENT_SCHEMA_VERSION.into()),
            );
        }

        let json_string = serde_json::to_string_pretty(&json_value)?;
        atomic_write(&self.path, json_string.as_bytes())?;
        Ok(())
    }
}

/// Migrates a preferences JSON value from one schema version to another.
pub fn migrate_preferences(
    mut value: Value,
    from: u32,
    to: u32,
) -> Result<Value, StorageError> {
    let mut current = from;
    while current < to {
        value = migrate_preferences_step(value, current)?;
        current += 1;
    }
    Ok(value)
}

/// Performs a single preferences migration step.
fn migrate_preferences_step(_value: Value, version: u32) -> Result<Value, StorageError> {
    match version {
        // Future migrations go here.
        _ => Err(StorageError::MigrationFailed(format!(
            "No preferences migration from version {version} to {}",
            version + 1
        ))),
    }
}

/// Writes data to a file atomically (same implementation as combo_storage).
fn atomic_write(path: &std::path::Path, data: &[u8]) -> Result<(), StorageError> {
    let tmp_path = path.with_extension("tmp");

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    {
        let file = File::create(&tmp_path)?;
        file.lock_exclusive()
            .map_err(|_| StorageError::FileLocked)?;

        let mut writer = std::io::BufWriter::new(&file);
        writer.write_all(data)?;
        writer.flush()?;
        file.sync_all()?;
    }

    fs::rename(&tmp_path, path)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_save_and_load_roundtrip() {
        let tmp = tempfile::tempdir().expect("create temp dir");
        let path = tmp.path().join("preferences.json");
        let storage = PreferencesStorage::new(path);

        let prefs = Preferences::default();
        storage.save(&prefs).expect("save");
        let loaded = storage.load().expect("load");

        assert_eq!(loaded.enabled, prefs.enabled);
        assert_eq!(loaded.theme, prefs.theme);
        assert_eq!(loaded.max_backups, prefs.max_backups);
    }

    #[test]
    fn test_load_nonexistent_returns_default() {
        let tmp = tempfile::tempdir().expect("create temp dir");
        let path = tmp.path().join("no_such_file.json");
        let storage = PreferencesStorage::new(path);

        let loaded = storage.load().expect("load default");
        assert!(loaded.enabled);
    }

    #[test]
    fn test_save_creates_parent_directories() {
        let tmp = tempfile::tempdir().expect("create temp dir");
        let path = tmp.path().join("nested").join("preferences.json");
        let storage = PreferencesStorage::new(path.clone());

        storage.save(&Preferences::default()).expect("save");
        assert!(path.exists());
    }

    #[test]
    fn test_saved_json_contains_schema_version() {
        let tmp = tempfile::tempdir().expect("create temp dir");
        let path = tmp.path().join("preferences.json");
        let storage = PreferencesStorage::new(path.clone());

        storage.save(&Preferences::default()).expect("save");

        let content = fs::read_to_string(&path).expect("read");
        let json: Value = serde_json::from_str(&content).expect("parse");
        assert_eq!(
            json.get(SCHEMA_VERSION_KEY).and_then(|v| v.as_u64()),
            Some(CURRENT_SCHEMA_VERSION as u64)
        );
    }

    #[test]
    fn test_saved_json_is_pretty_printed() {
        let tmp = tempfile::tempdir().expect("create temp dir");
        let path = tmp.path().join("preferences.json");
        let storage = PreferencesStorage::new(path.clone());

        storage.save(&Preferences::default()).expect("save");

        let content = fs::read_to_string(&path).expect("read");
        assert!(content.contains('\n'));
    }

    #[test]
    fn test_custom_preferences_roundtrip() {
        let tmp = tempfile::tempdir().expect("create temp dir");
        let path = tmp.path().join("preferences.json");
        let storage = PreferencesStorage::new(path);

        let mut prefs = Preferences::default();
        prefs.play_sound = true;
        prefs.max_backups = 42;
        prefs.excluded_apps = vec!["keepass".to_string()];

        storage.save(&prefs).expect("save");
        let loaded = storage.load().expect("load");

        assert!(loaded.play_sound);
        assert_eq!(loaded.max_backups, 42);
        assert_eq!(loaded.excluded_apps, vec!["keepass".to_string()]);
    }

    #[test]
    fn test_no_tmp_file_remains_after_save() {
        let tmp = tempfile::tempdir().expect("create temp dir");
        let path = tmp.path().join("preferences.json");
        let storage = PreferencesStorage::new(path.clone());

        storage.save(&Preferences::default()).expect("save");
        assert!(!path.with_extension("tmp").exists());
    }

    #[test]
    fn test_migrate_preferences_no_op_when_equal() {
        let value = serde_json::json!({"enabled": true});
        let result = migrate_preferences(value.clone(), 1, 1);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), value);
    }
}
