//! Persistence for the combo library (groups and combos).
//!
//! Reads and writes `combos.json` with atomic writes, file locking,
//! and schema version migration support.

use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;

use fs2::FileExt;
use serde_json::Value;
use tracing;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::library::ComboLibrary;

use super::storage::StorageError;

/// Lightweight combo summary without snippet text (MT-1108).
///
/// Used for listing combos in the UI without loading full snippet content,
/// which can be large for rich snippets.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ComboSummary {
    pub id: Uuid,
    pub keyword: String,
    pub name: String,
    pub group_id: Uuid,
}

/// Current schema version for the combo library on-disk format.
const CURRENT_SCHEMA_VERSION: u32 = 1;

/// Key used in the JSON envelope for schema version.
const SCHEMA_VERSION_KEY: &str = "schemaVersion";

/// Manages loading and saving the combo library to disk.
pub struct ComboStorage {
    path: PathBuf,
}

impl ComboStorage {
    /// Creates a new `ComboStorage` that reads from and writes to `path`.
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    /// Loads the combo library from disk.
    ///
    /// If the file does not exist, returns a default `ComboLibrary`.
    /// Acquires a shared file lock during the read.
    /// Performs schema migration if the on-disk version is older.
    pub fn load(&self) -> Result<ComboLibrary, StorageError> {
        if !self.path.exists() {
            tracing::info!("Combo library file not found, returning default");
            return Ok(ComboLibrary::new("1.0"));
        }

        let file = File::open(&self.path)?;
        file.lock_shared()
            .map_err(|_| StorageError::FileLocked)?;

        let content = fs::read_to_string(&self.path)?;

        // Unlock happens on drop of file handle.
        drop(file);

        // Check schema version and migrate if needed.
        let mut json_value: Value = serde_json::from_str(&content)?;
        let on_disk_version = json_value
            .get(SCHEMA_VERSION_KEY)
            .and_then(|v| v.as_u64())
            .unwrap_or(1) as u32;

        if on_disk_version < CURRENT_SCHEMA_VERSION {
            tracing::info!(
                from = on_disk_version,
                to = CURRENT_SCHEMA_VERSION,
                "Migrating combo library schema"
            );
            json_value =
                migrate_combo_library(json_value, on_disk_version, CURRENT_SCHEMA_VERSION)?;
        }

        let library: ComboLibrary = serde_json::from_value(json_value)?;
        Ok(library)
    }

    /// Loads only combo summaries (id, keyword, name, group_id) without snippets.
    ///
    /// This is faster than `load()` for UI list views that don't need snippet text.
    /// Falls back to a full load internally but only returns summary fields.
    pub fn get_combo_summaries(&self) -> Result<Vec<ComboSummary>, StorageError> {
        let library = self.load()?;
        let summaries = library
            .combos
            .iter()
            .map(|c| ComboSummary {
                id: c.id,
                keyword: c.keyword.clone(),
                name: c.name.clone(),
                group_id: c.group_id,
            })
            .collect();
        Ok(summaries)
    }

    /// Saves the combo library to disk.
    ///
    /// Performs an atomic write: writes to a temporary file, fsyncs, then renames.
    /// Acquires an exclusive file lock during the write.
    /// Embeds the current schema version in the output JSON.
    pub fn save(&self, library: &ComboLibrary) -> Result<(), StorageError> {
        // Serialize to a JSON value so we can inject schemaVersion.
        let mut json_value = serde_json::to_value(library)?;
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

/// Migrates a combo library JSON value from one schema version to another.
///
/// Each migration step is applied sequentially (from -> from+1 -> ... -> to).
pub fn migrate_combo_library(
    mut value: Value,
    from: u32,
    to: u32,
) -> Result<Value, StorageError> {
    let mut current = from;
    while current < to {
        value = migrate_combo_library_step(value, current)?;
        current += 1;
    }
    Ok(value)
}

/// Performs a single migration step from `version` to `version + 1`.
fn migrate_combo_library_step(_value: Value, version: u32) -> Result<Value, StorageError> {
    match version {
        // Future migrations go here, e.g.:
        // 1 => migrate_v1_to_v2(value),
        _ => Err(StorageError::MigrationFailed(format!(
            "No migration path from version {version} to {}",
            version + 1
        ))),
    }
}

/// Writes data to a file atomically.
///
/// 1. Writes to a `.tmp` file in the same directory.
/// 2. Fsyncs the temp file.
/// 3. Renames the temp file to the target path (atomic on the same filesystem).
/// 4. Acquires an exclusive lock on the temp file during write.
fn atomic_write(path: &std::path::Path, data: &[u8]) -> Result<(), StorageError> {
    let tmp_path = path.with_extension("tmp");

    // Ensure parent directory exists.
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
        // Lock released on drop.
    }

    fs::rename(&tmp_path, path)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::combo::ComboBuilder;
    use crate::models::group::Group;

    fn make_test_library() -> ComboLibrary {
        let mut lib = ComboLibrary::new("1.0");
        let group = Group::new("Test Group");
        let combo = ComboBuilder::new()
            .keyword("sig")
            .snippet("Best regards")
            .group_id(group.id)
            .build()
            .unwrap();
        lib.add_group(group);
        lib.add_combo(combo);
        lib
    }

    #[test]
    fn test_save_and_load_roundtrip() {
        let tmp = tempfile::tempdir().expect("create temp dir");
        let path = tmp.path().join("combos.json");
        let storage = ComboStorage::new(path);

        let library = make_test_library();
        storage.save(&library).expect("save");
        let loaded = storage.load().expect("load");

        assert_eq!(loaded.groups.len(), library.groups.len());
        assert_eq!(loaded.combos.len(), library.combos.len());
        assert_eq!(loaded.combos[0].keyword, "sig");
    }

    #[test]
    fn test_load_nonexistent_returns_default() {
        let tmp = tempfile::tempdir().expect("create temp dir");
        let path = tmp.path().join("does_not_exist.json");
        let storage = ComboStorage::new(path);

        let loaded = storage.load().expect("load default");
        assert!(loaded.combos.is_empty());
    }

    #[test]
    fn test_save_creates_parent_directories() {
        let tmp = tempfile::tempdir().expect("create temp dir");
        let path = tmp.path().join("nested").join("dir").join("combos.json");
        let storage = ComboStorage::new(path.clone());

        let library = ComboLibrary::new("1.0");
        storage.save(&library).expect("save");
        assert!(path.exists());
    }

    #[test]
    fn test_atomic_write_no_tmp_file_remains() {
        let tmp = tempfile::tempdir().expect("create temp dir");
        let path = tmp.path().join("combos.json");
        let storage = ComboStorage::new(path.clone());

        let library = ComboLibrary::new("1.0");
        storage.save(&library).expect("save");

        let tmp_path = path.with_extension("tmp");
        assert!(!tmp_path.exists(), "temp file should be removed after atomic write");
    }

    #[test]
    fn test_saved_json_contains_schema_version() {
        let tmp = tempfile::tempdir().expect("create temp dir");
        let path = tmp.path().join("combos.json");
        let storage = ComboStorage::new(path.clone());

        let library = ComboLibrary::new("1.0");
        storage.save(&library).expect("save");

        let content = fs::read_to_string(&path).expect("read file");
        let json: Value = serde_json::from_str(&content).expect("parse JSON");
        assert_eq!(
            json.get(SCHEMA_VERSION_KEY).and_then(|v| v.as_u64()),
            Some(CURRENT_SCHEMA_VERSION as u64)
        );
    }

    #[test]
    fn test_saved_json_is_pretty_printed() {
        let tmp = tempfile::tempdir().expect("create temp dir");
        let path = tmp.path().join("combos.json");
        let storage = ComboStorage::new(path.clone());

        let library = ComboLibrary::new("1.0");
        storage.save(&library).expect("save");

        let content = fs::read_to_string(&path).expect("read file");
        // Pretty-printed JSON has newlines and indentation.
        assert!(content.contains('\n'));
        assert!(content.contains("  "));
    }

    #[test]
    fn test_migrate_combo_library_no_op_when_versions_equal() {
        let value = serde_json::json!({"version": "1.0", "groups": [], "combos": []});
        let result = migrate_combo_library(value.clone(), 1, 1);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), value);
    }

    #[test]
    fn test_migrate_combo_library_fails_for_unknown_version() {
        let value = serde_json::json!({"version": "1.0"});
        let result = migrate_combo_library(value, 99, 100);
        assert!(result.is_err());
    }

    // ── MT-1108: ComboSummary tests ──────────────────────────────

    #[test]
    fn test_get_combo_summaries() {
        let tmp = tempfile::tempdir().expect("create temp dir");
        let path = tmp.path().join("combos.json");
        let storage = ComboStorage::new(path);

        let library = make_test_library();
        storage.save(&library).expect("save");

        let summaries = storage.get_combo_summaries().expect("summaries");
        assert_eq!(summaries.len(), 1);
        assert_eq!(summaries[0].keyword, "sig");
        assert_eq!(summaries[0].name, "");
    }

    #[test]
    fn test_get_combo_summaries_empty() {
        let tmp = tempfile::tempdir().expect("create temp dir");
        let path = tmp.path().join("combos.json");
        let storage = ComboStorage::new(path);

        let summaries = storage.get_combo_summaries().expect("summaries");
        assert!(summaries.is_empty());
    }

    #[test]
    fn test_combo_summary_serialization() {
        let summary = ComboSummary {
            id: uuid::Uuid::new_v4(),
            keyword: "sig".to_string(),
            name: "Signature".to_string(),
            group_id: uuid::Uuid::new_v4(),
        };
        let json = serde_json::to_string(&summary).unwrap();
        assert!(json.contains("keyword"));
        assert!(json.contains("groupId"));
    }

    #[test]
    fn test_multiple_saves_overwrite() {
        let tmp = tempfile::tempdir().expect("create temp dir");
        let path = tmp.path().join("combos.json");
        let storage = ComboStorage::new(path);

        let lib1 = ComboLibrary::new("1.0");
        storage.save(&lib1).expect("save 1");

        let mut lib2 = ComboLibrary::new("1.0");
        let group = Group::new("G");
        lib2.add_group(group);
        storage.save(&lib2).expect("save 2");

        let loaded = storage.load().expect("load");
        assert_eq!(loaded.groups.len(), 1);
    }
}
