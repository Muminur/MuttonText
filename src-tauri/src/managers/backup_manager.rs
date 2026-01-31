//! Backup and restore functionality for combos, groups, and preferences.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use thiserror::Error;

use crate::models::combo::Combo;
use crate::models::group::Group;

/// Errors that can occur during backup operations.
#[derive(Debug, Error)]
pub enum BackupError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Serialization error: {0}")]
    Serialization(String),
    #[error("Backup not found: {0}")]
    NotFound(String),
    #[error("Invalid backup file: {0}")]
    InvalidBackup(String),
}

/// Information about a stored backup.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupInfo {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub size_bytes: u64,
    pub combo_count: usize,
    pub path: PathBuf,
}

/// Metadata stored inside each backup file.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupMetadata {
    pub version: String,
    pub created_at: DateTime<Utc>,
    pub app_version: String,
}

/// Full backup data including metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupData {
    pub metadata: BackupMetadata,
    pub combos: Vec<Combo>,
    pub groups: Vec<Group>,
    pub preferences: serde_json::Value,
}

/// Manages backup creation, restoration, and retention.
pub struct BackupManager {
    pub backup_dir: PathBuf,
    pub max_backups: u32,
    pub auto_interval_hours: u32,
}

impl BackupManager {
    pub fn new(backup_dir: PathBuf, max_backups: u32) -> Self {
        Self {
            backup_dir,
            max_backups,
            auto_interval_hours: 24,
        }
    }

    /// Create a backup file containing combos, groups, and preferences.
    pub fn create_backup(
        &self,
        combos: &[Combo],
        groups: &[Group],
        preferences: &serde_json::Value,
    ) -> Result<BackupInfo, BackupError> {
        std::fs::create_dir_all(&self.backup_dir)?;

        let now = Utc::now();
        let id = now.format("%Y%m%d_%H%M%S_%3f").to_string();
        let filename = format!("{}.btbackup", id);
        let path = self.backup_dir.join(&filename);

        let data = BackupData {
            metadata: BackupMetadata {
                version: "1.0".to_string(),
                created_at: now,
                app_version: env!("CARGO_PKG_VERSION").to_string(),
            },
            combos: combos.to_vec(),
            groups: groups.to_vec(),
            preferences: preferences.clone(),
        };

        let json = serde_json::to_string_pretty(&data)
            .map_err(|e| BackupError::Serialization(e.to_string()))?;
        std::fs::write(&path, &json)?;

        let size_bytes = json.len() as u64;

        Ok(BackupInfo {
            id,
            timestamp: now,
            size_bytes,
            combo_count: combos.len(),
            path,
        })
    }

    /// Restore a backup by its ID.
    pub fn restore_backup(&self, backup_id: &str) -> Result<BackupData, BackupError> {
        let filename = format!("{}.btbackup", backup_id);
        let path = self.backup_dir.join(&filename);

        if !path.exists() {
            return Err(BackupError::NotFound(backup_id.to_string()));
        }

        let content = std::fs::read_to_string(&path)?;
        let data: BackupData = serde_json::from_str(&content)
            .map_err(|e| BackupError::InvalidBackup(e.to_string()))?;

        Ok(data)
    }

    /// List all available backups, sorted by timestamp descending (newest first).
    pub fn list_backups(&self) -> Result<Vec<BackupInfo>, BackupError> {
        if !self.backup_dir.exists() {
            return Ok(Vec::new());
        }

        let mut backups = Vec::new();
        for entry in std::fs::read_dir(&self.backup_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("btbackup") {
                let id = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("")
                    .to_string();

                let content = std::fs::read_to_string(&path)?;
                let metadata = entry.metadata()?;

                if let Ok(data) = serde_json::from_str::<BackupData>(&content) {
                    backups.push(BackupInfo {
                        id,
                        timestamp: data.metadata.created_at,
                        size_bytes: metadata.len(),
                        combo_count: data.combos.len(),
                        path,
                    });
                }
            }
        }

        backups.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        Ok(backups)
    }

    /// Delete a backup by its ID.
    pub fn delete_backup(&self, backup_id: &str) -> Result<(), BackupError> {
        let filename = format!("{}.btbackup", backup_id);
        let path = self.backup_dir.join(&filename);

        if !path.exists() {
            return Err(BackupError::NotFound(backup_id.to_string()));
        }

        std::fs::remove_file(&path)?;
        Ok(())
    }

    /// Remove old backups beyond `max_backups`, returning the count deleted.
    pub fn enforce_retention(&self) -> Result<usize, BackupError> {
        let backups = self.list_backups()?;
        let max = self.max_backups as usize;

        if backups.len() <= max {
            return Ok(0);
        }

        let to_delete = &backups[max..];
        let count = to_delete.len();

        for backup in to_delete {
            if backup.path.exists() {
                std::fs::remove_file(&backup.path)?;
            }
        }

        Ok(count)
    }

    /// Check whether an automatic backup should be created.
    pub fn should_auto_backup(&self, last_backup: Option<DateTime<Utc>>) -> bool {
        match last_backup {
            None => true,
            Some(last) => {
                let elapsed = Utc::now() - last;
                elapsed.num_hours() >= self.auto_interval_hours as i64
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn make_manager(dir: &TempDir) -> BackupManager {
        BackupManager::new(dir.path().to_path_buf(), 3)
    }

    fn sample_combos() -> Vec<Combo> {
        use crate::models::combo::ComboBuilder;
        vec![ComboBuilder::new()
            .keyword("sig")
            .snippet("hello")
            .build()
            .unwrap()]
    }

    fn sample_groups() -> Vec<Group> {
        vec![Group::new("Test")]
    }

    // ── Create & Restore Roundtrip ───────────────────────────────

    #[test]
    fn test_create_and_restore_backup() {
        let dir = TempDir::new().unwrap();
        let mgr = make_manager(&dir);
        let prefs = serde_json::json!({"theme": "dark"});

        let info = mgr.create_backup(&sample_combos(), &sample_groups(), &prefs).unwrap();
        assert_eq!(info.combo_count, 1);
        assert!(info.path.exists());

        let data = mgr.restore_backup(&info.id).unwrap();
        assert_eq!(data.combos.len(), 1);
        assert_eq!(data.combos[0].keyword, "sig");
        assert_eq!(data.groups[0].name, "Test");
        assert_eq!(data.preferences["theme"], "dark");
        assert_eq!(data.metadata.version, "1.0");
    }

    // ── List Backups ─────────────────────────────────────────────

    #[test]
    fn test_list_backups_empty() {
        let dir = TempDir::new().unwrap();
        let mgr = make_manager(&dir);
        let list = mgr.list_backups().unwrap();
        assert!(list.is_empty());
    }

    #[test]
    fn test_list_backups_sorted() {
        let dir = TempDir::new().unwrap();
        let mgr = make_manager(&dir);
        let prefs = serde_json::json!({});

        let info1 = mgr.create_backup(&sample_combos(), &[], &prefs).unwrap();
        std::thread::sleep(std::time::Duration::from_millis(10));
        let info2 = mgr.create_backup(&sample_combos(), &[], &prefs).unwrap();

        let list = mgr.list_backups().unwrap();
        assert_eq!(list.len(), 2);
        // Newest first
        assert_eq!(list[0].id, info2.id);
        assert_eq!(list[1].id, info1.id);
    }

    // ── Delete Backup ────────────────────────────────────────────

    #[test]
    fn test_delete_backup() {
        let dir = TempDir::new().unwrap();
        let mgr = make_manager(&dir);
        let prefs = serde_json::json!({});

        let info = mgr.create_backup(&sample_combos(), &[], &prefs).unwrap();
        assert!(info.path.exists());

        mgr.delete_backup(&info.id).unwrap();
        assert!(!info.path.exists());
    }

    #[test]
    fn test_delete_backup_not_found() {
        let dir = TempDir::new().unwrap();
        let mgr = make_manager(&dir);
        let result = mgr.delete_backup("nonexistent");
        assert!(result.is_err());
    }

    // ── Retention ────────────────────────────────────────────────

    #[test]
    fn test_enforce_retention() {
        let dir = TempDir::new().unwrap();
        let mgr = BackupManager::new(dir.path().to_path_buf(), 2);
        let prefs = serde_json::json!({});

        for _ in 0..4 {
            std::thread::sleep(std::time::Duration::from_millis(10));
            mgr.create_backup(&sample_combos(), &[], &prefs).unwrap();
        }

        let deleted = mgr.enforce_retention().unwrap();
        assert_eq!(deleted, 2);
        assert_eq!(mgr.list_backups().unwrap().len(), 2);
    }

    #[test]
    fn test_enforce_retention_no_delete_needed() {
        let dir = TempDir::new().unwrap();
        let mgr = make_manager(&dir);
        let prefs = serde_json::json!({});

        mgr.create_backup(&sample_combos(), &[], &prefs).unwrap();
        let deleted = mgr.enforce_retention().unwrap();
        assert_eq!(deleted, 0);
    }

    // ── Auto-Backup Timing ───────────────────────────────────────

    #[test]
    fn test_should_auto_backup_no_previous() {
        let dir = TempDir::new().unwrap();
        let mgr = make_manager(&dir);
        assert!(mgr.should_auto_backup(None));
    }

    #[test]
    fn test_should_auto_backup_recent() {
        let dir = TempDir::new().unwrap();
        let mgr = make_manager(&dir);
        assert!(!mgr.should_auto_backup(Some(Utc::now())));
    }

    #[test]
    fn test_should_auto_backup_old() {
        let dir = TempDir::new().unwrap();
        let mgr = make_manager(&dir);
        let old = Utc::now() - chrono::Duration::hours(25);
        assert!(mgr.should_auto_backup(Some(old)));
    }

    // ── Restore Not Found ────────────────────────────────────────

    #[test]
    fn test_restore_nonexistent() {
        let dir = TempDir::new().unwrap();
        let mgr = make_manager(&dir);
        let result = mgr.restore_backup("nonexistent");
        assert!(result.is_err());
    }

    // ── Error Display ────────────────────────────────────────────

    #[test]
    fn test_backup_error_display() {
        let err = BackupError::NotFound("test".to_string());
        assert_eq!(err.to_string(), "Backup not found: test");
    }

    // ── Serialization ────────────────────────────────────────────

    #[test]
    fn test_backup_info_serialization() {
        let info = BackupInfo {
            id: "20240101_120000_000".to_string(),
            timestamp: Utc::now(),
            size_bytes: 1024,
            combo_count: 5,
            path: PathBuf::from("/tmp/test.btbackup"),
        };
        let json = serde_json::to_string(&info).unwrap();
        assert!(json.contains("sizeBytes"));
        assert!(json.contains("comboCount"));
    }
}
