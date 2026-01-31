//! Update checking logic (no HTTP -- just version comparison and skip logic).

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use thiserror::Error;

/// Errors that can occur in update operations.
#[derive(Debug, Error)]
pub enum UpdateError {
    #[error("Invalid version string: {0}")]
    InvalidVersion(String),
}

/// Information about a software version.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VersionInfo {
    pub version: String,
    pub release_url: String,
    pub release_notes: String,
    pub published_at: String,
}

/// Manages update checking logic.
pub struct UpdateManager {
    pub current_version: String,
    pub skipped_versions: Vec<String>,
    pub last_check: Option<DateTime<Utc>>,
}

impl UpdateManager {
    pub fn new(current_version: String) -> Self {
        Self {
            current_version,
            skipped_versions: Vec::new(),
            last_check: None,
        }
    }

    /// Check if the given latest version is newer than current and not skipped.
    pub fn check_update_available(&self, latest: &VersionInfo) -> bool {
        if self.is_version_skipped(&latest.version) {
            return false;
        }
        matches!(
            Self::compare_versions(&self.current_version, &latest.version),
            Ok(Ordering::Less)
        )
    }

    /// Mark a version to be skipped.
    pub fn skip_version(&mut self, version: &str) {
        if !self.skipped_versions.contains(&version.to_string()) {
            self.skipped_versions.push(version.to_string());
        }
    }

    /// Check if a version has been skipped.
    pub fn is_version_skipped(&self, version: &str) -> bool {
        self.skipped_versions.contains(&version.to_string())
    }

    /// Check if enough time has elapsed since the last check.
    pub fn should_check(&self, interval_hours: u32) -> bool {
        match self.last_check {
            None => true,
            Some(last) => {
                let elapsed = Utc::now() - last;
                elapsed.num_hours() >= interval_hours as i64
            }
        }
    }

    /// Parse a semver string into (major, minor, patch).
    pub fn parse_version(version: &str) -> Result<(u32, u32, u32), UpdateError> {
        let v = version.strip_prefix('v').unwrap_or(version);
        // Strip any pre-release suffix (e.g. "-beta.1")
        let v = v.split('-').next().unwrap_or(v);
        let parts: Vec<&str> = v.split('.').collect();
        if parts.len() != 3 {
            return Err(UpdateError::InvalidVersion(version.to_string()));
        }
        let major = parts[0]
            .parse::<u32>()
            .map_err(|_| UpdateError::InvalidVersion(version.to_string()))?;
        let minor = parts[1]
            .parse::<u32>()
            .map_err(|_| UpdateError::InvalidVersion(version.to_string()))?;
        let patch = parts[2]
            .parse::<u32>()
            .map_err(|_| UpdateError::InvalidVersion(version.to_string()))?;
        Ok((major, minor, patch))
    }

    /// Compare two semver strings.
    pub fn compare_versions(current: &str, latest: &str) -> Result<Ordering, UpdateError> {
        let c = Self::parse_version(current)?;
        let l = Self::parse_version(latest)?;
        Ok(c.cmp(&l))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── Version Parsing ──────────────────────────────────────────

    #[test]
    fn test_parse_version_basic() {
        assert_eq!(UpdateManager::parse_version("1.2.3").unwrap(), (1, 2, 3));
    }

    #[test]
    fn test_parse_version_with_v_prefix() {
        assert_eq!(UpdateManager::parse_version("v1.0.0").unwrap(), (1, 0, 0));
    }

    #[test]
    fn test_parse_version_invalid() {
        assert!(UpdateManager::parse_version("1.2").is_err());
        assert!(UpdateManager::parse_version("abc").is_err());
        assert!(UpdateManager::parse_version("1.2.x").is_err());
    }

    #[test]
    fn test_parse_version_prerelease_stripped() {
        assert_eq!(
            UpdateManager::parse_version("1.2.3-beta.1").unwrap(),
            (1, 2, 3)
        );
    }

    // ── Version Comparison ───────────────────────────────────────

    #[test]
    fn test_compare_versions_equal() {
        assert_eq!(
            UpdateManager::compare_versions("1.0.0", "1.0.0").unwrap(),
            Ordering::Equal
        );
    }

    #[test]
    fn test_compare_versions_less() {
        assert_eq!(
            UpdateManager::compare_versions("1.0.0", "1.0.1").unwrap(),
            Ordering::Less
        );
        assert_eq!(
            UpdateManager::compare_versions("1.0.0", "1.1.0").unwrap(),
            Ordering::Less
        );
        assert_eq!(
            UpdateManager::compare_versions("1.0.0", "2.0.0").unwrap(),
            Ordering::Less
        );
    }

    #[test]
    fn test_compare_versions_greater() {
        assert_eq!(
            UpdateManager::compare_versions("2.0.0", "1.0.0").unwrap(),
            Ordering::Greater
        );
    }

    // ── Update Available ─────────────────────────────────────────

    #[test]
    fn test_check_update_available_newer() {
        let mgr = UpdateManager::new("1.0.0".to_string());
        let info = VersionInfo {
            version: "1.1.0".to_string(),
            release_url: String::new(),
            release_notes: String::new(),
            published_at: String::new(),
        };
        assert!(mgr.check_update_available(&info));
    }

    #[test]
    fn test_check_update_available_same() {
        let mgr = UpdateManager::new("1.0.0".to_string());
        let info = VersionInfo {
            version: "1.0.0".to_string(),
            release_url: String::new(),
            release_notes: String::new(),
            published_at: String::new(),
        };
        assert!(!mgr.check_update_available(&info));
    }

    #[test]
    fn test_check_update_available_older() {
        let mgr = UpdateManager::new("2.0.0".to_string());
        let info = VersionInfo {
            version: "1.0.0".to_string(),
            release_url: String::new(),
            release_notes: String::new(),
            published_at: String::new(),
        };
        assert!(!mgr.check_update_available(&info));
    }

    #[test]
    fn test_check_update_available_skipped() {
        let mut mgr = UpdateManager::new("1.0.0".to_string());
        mgr.skip_version("1.1.0");
        let info = VersionInfo {
            version: "1.1.0".to_string(),
            release_url: String::new(),
            release_notes: String::new(),
            published_at: String::new(),
        };
        assert!(!mgr.check_update_available(&info));
    }

    // ── Skip Logic ───────────────────────────────────────────────

    #[test]
    fn test_skip_version() {
        let mut mgr = UpdateManager::new("1.0.0".to_string());
        assert!(!mgr.is_version_skipped("2.0.0"));
        mgr.skip_version("2.0.0");
        assert!(mgr.is_version_skipped("2.0.0"));
    }

    #[test]
    fn test_skip_version_idempotent() {
        let mut mgr = UpdateManager::new("1.0.0".to_string());
        mgr.skip_version("2.0.0");
        mgr.skip_version("2.0.0");
        assert_eq!(mgr.skipped_versions.len(), 1);
    }

    // ── Check Timing ─────────────────────────────────────────────

    #[test]
    fn test_should_check_no_previous() {
        let mgr = UpdateManager::new("1.0.0".to_string());
        assert!(mgr.should_check(24));
    }

    #[test]
    fn test_should_check_recent() {
        let mut mgr = UpdateManager::new("1.0.0".to_string());
        mgr.last_check = Some(Utc::now());
        assert!(!mgr.should_check(24));
    }

    #[test]
    fn test_should_check_old() {
        let mut mgr = UpdateManager::new("1.0.0".to_string());
        mgr.last_check = Some(Utc::now() - chrono::Duration::hours(25));
        assert!(mgr.should_check(24));
    }

    // ── Error Display ────────────────────────────────────────────

    #[test]
    fn test_update_error_display() {
        let err = UpdateError::InvalidVersion("bad".to_string());
        assert_eq!(err.to_string(), "Invalid version string: bad");
    }

    // ── Serialization ────────────────────────────────────────────

    #[test]
    fn test_version_info_serialization() {
        let info = VersionInfo {
            version: "1.0.0".to_string(),
            release_url: "https://example.com".to_string(),
            release_notes: "Bug fixes".to_string(),
            published_at: "2024-01-01".to_string(),
        };
        let json = serde_json::to_string(&info).unwrap();
        assert!(json.contains("releaseUrl"));
        assert!(json.contains("releaseNotes"));
        assert!(json.contains("publishedAt"));
        let back: VersionInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(back.version, "1.0.0");
    }
}
