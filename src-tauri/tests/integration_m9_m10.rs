//! Integration tests for Milestones 9 & 10.
//!
//! Tests cross-manager interactions for import/export roundtrip, backup/restore
//! integrity, preferences persistence, emoji in variable evaluation, update
//! version comparison edge cases, and backup retention enforcement.

use std::cmp::Ordering;

use tempfile::TempDir;

use muttontext_lib::managers::backup_manager::BackupManager;
use muttontext_lib::managers::export_manager::{ExportFormat, ExportManager};
use muttontext_lib::managers::import_manager::{ConflictResolution, ImportManager};
use muttontext_lib::managers::preferences_manager::PreferencesManager;
use muttontext_lib::managers::update_manager::{UpdateManager, VersionInfo};
use muttontext_lib::managers::variable_evaluator::{EvalContext, VariableEvaluator};
use muttontext_lib::models::{ComboBuilder, Group, Preferences};

// ─── Import → Export Roundtrip ──────────────────────────────────────────────

#[test]
fn test_import_export_roundtrip_muttontext_json() {
    let group = Group::new("Email Signatures");
    let combo1 = ComboBuilder::new()
        .name("Signature")
        .keyword("sig")
        .snippet("Best regards,\nJohn Doe")
        .group_id(group.id)
        .build()
        .unwrap();
    let combo2 = ComboBuilder::new()
        .name("Address")
        .keyword("addr")
        .snippet("123 Main Street")
        .group_id(group.id)
        .build()
        .unwrap();

    // Export to JSON
    let json = ExportManager::export_muttontext_json(&[combo1.clone(), combo2.clone()], &[group.clone()])
        .unwrap();

    // Import back
    let result = ImportManager::import_muttontext_json(&json).unwrap();
    assert_eq!(result.imported_count, 2);
    assert_eq!(result.groups.len(), 1);
    assert_eq!(result.groups[0].name, "Email Signatures");
    assert_eq!(result.combos[0].keyword, "sig");
    assert_eq!(result.combos[1].keyword, "addr");
    assert_eq!(result.combos[0].snippet, "Best regards,\nJohn Doe");
}

#[test]
fn test_import_export_roundtrip_textexpander_csv() {
    let combo = ComboBuilder::new()
        .name("Greeting")
        .keyword("hi")
        .snippet("Hello, how are you?")
        .build()
        .unwrap();

    // Export to TextExpander CSV
    let csv = ExportManager::export_textexpander_csv(&[combo.clone()]).unwrap();

    // Import back
    let result = ImportManager::import_textexpander_csv(&csv, ConflictResolution::Skip).unwrap();
    assert_eq!(result.imported_count, 1);
    assert_eq!(result.combos[0].keyword, "hi");
    assert_eq!(result.combos[0].snippet, "Hello, how are you?");
    assert_eq!(result.combos[0].name, "Greeting");
}

#[test]
fn test_import_export_all_formats_detect() {
    let group = Group::new("G");
    let combo = ComboBuilder::new()
        .keyword("test")
        .snippet("value")
        .group_id(group.id)
        .build()
        .unwrap();

    // MuttonText JSON roundtrip with format detection
    let json = ExportManager::export_to_format(&[combo.clone()], &[group.clone()], ExportFormat::MuttonTextJson)
        .unwrap();
    let preview = ImportManager::preview_import(&json).unwrap();
    assert_eq!(preview.combo_count, 1);
    assert_eq!(preview.group_count, 1);
}

// ─── Backup → Restore Data Integrity ────────────────────────────────────────

#[test]
fn test_backup_restore_data_integrity() {
    let dir = TempDir::new().unwrap();
    let mgr = BackupManager::new(dir.path().to_path_buf(), 10);

    let group = Group::new("Important");
    let combo = ComboBuilder::new()
        .name("Sig")
        .keyword("sig")
        .snippet("Regards, Alice")
        .group_id(group.id)
        .build()
        .unwrap();
    let prefs = serde_json::json!({
        "theme": "dark",
        "playSound": true,
        "backupIntervalHours": 12
    });

    // Create backup
    let info = mgr.create_backup(&[combo.clone()], &[group.clone()], &prefs).unwrap();
    assert_eq!(info.combo_count, 1);

    // Restore and verify all fields
    let data = mgr.restore_backup(&info.id).unwrap();
    assert_eq!(data.combos.len(), 1);
    assert_eq!(data.combos[0].keyword, "sig");
    assert_eq!(data.combos[0].snippet, "Regards, Alice");
    assert_eq!(data.combos[0].name, "Sig");
    assert_eq!(data.combos[0].group_id, group.id);
    assert_eq!(data.groups.len(), 1);
    assert_eq!(data.groups[0].name, "Important");
    assert_eq!(data.groups[0].id, group.id);
    assert_eq!(data.preferences["theme"], "dark");
    assert_eq!(data.preferences["playSound"], true);
    assert_eq!(data.preferences["backupIntervalHours"], 12);
    assert_eq!(data.metadata.version, "1.0");
}

#[test]
fn test_backup_restore_empty_library() {
    let dir = TempDir::new().unwrap();
    let mgr = BackupManager::new(dir.path().to_path_buf(), 10);
    let prefs = serde_json::json!({});

    let info = mgr.create_backup(&[], &[], &prefs).unwrap();
    assert_eq!(info.combo_count, 0);

    let data = mgr.restore_backup(&info.id).unwrap();
    assert!(data.combos.is_empty());
    assert!(data.groups.is_empty());
}

// ─── Preferences Save/Load with Non-Default Values ─────────────────────────

#[test]
fn test_preferences_save_load_non_defaults() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("prefs.json");

    let mut mgr = PreferencesManager::new(path.clone()).unwrap();

    // Modify many fields from their defaults
    let mut prefs = Preferences::default();
    prefs.enabled = false;
    prefs.play_sound = true;
    prefs.show_system_tray = false;
    prefs.start_at_login = true;
    prefs.start_minimized = true;
    prefs.default_case_sensitive = true;
    prefs.backup_interval_hours = 48;
    prefs.max_backups = 25;
    prefs.auto_check_updates = false;
    prefs.excluded_apps = vec!["1password".to_string(), "keepass".to_string()];

    mgr.update(prefs).unwrap();

    // Load from disk in a fresh manager
    let mgr2 = PreferencesManager::new(path).unwrap();
    let loaded = mgr2.get();
    assert!(!loaded.enabled);
    assert!(loaded.play_sound);
    assert!(!loaded.show_system_tray);
    assert!(loaded.start_at_login);
    assert!(loaded.start_minimized);
    assert!(loaded.default_case_sensitive);
    assert_eq!(loaded.backup_interval_hours, 48);
    assert_eq!(loaded.max_backups, 25);
    assert!(!loaded.auto_check_updates);
    assert_eq!(loaded.excluded_apps, vec!["1password", "keepass"]);
}

// ─── Emoji Expansion Within Variable Evaluation Context ─────────────────────

#[test]
fn test_emoji_in_variable_context() {
    // Simulates a combo whose snippet contains emoji text alongside variables.
    // The variable evaluator should preserve emoji characters untouched.
    let evaluator = VariableEvaluator::new();
    let mut ctx = EvalContext::new("world".to_string(), |_| None);

    let result = evaluator
        .evaluate("Hello #{clipboard}! \u{1F600}\u{1F389}", &mut ctx)
        .unwrap();
    assert_eq!(result.text, "Hello world! \u{1F600}\u{1F389}");
}

#[test]
fn test_emoji_in_combo_reference() {
    // Combo references a snippet with emoji
    let evaluator = VariableEvaluator::new();
    let mut ctx = EvalContext::new(String::new(), |kw| match kw {
        "wave" => Some("\u{1F44B} Hello!".to_string()),
        _ => None,
    });

    let result = evaluator.evaluate("#{combo:wave}", &mut ctx).unwrap();
    assert_eq!(result.text, "\u{1F44B} Hello!");
}

#[test]
fn test_emoji_with_upper_lower_transforms() {
    let evaluator = VariableEvaluator::new();
    let mut ctx = EvalContext::new(String::new(), |kw| match kw {
        "msg" => Some("Hello \u{1F600} World".to_string()),
        _ => None,
    });

    let result = evaluator.evaluate("#{upper:msg}", &mut ctx).unwrap();
    assert_eq!(result.text, "HELLO \u{1F600} WORLD");

    let mut ctx2 = EvalContext::new(String::new(), |kw| match kw {
        "msg" => Some("Hello \u{1F600} World".to_string()),
        _ => None,
    });
    let result2 = evaluator.evaluate("#{lower:msg}", &mut ctx2).unwrap();
    assert_eq!(result2.text, "hello \u{1F600} world");
}

// ─── Update Version Comparison Edge Cases ───────────────────────────────────

#[test]
fn test_version_comparison_v_prefix() {
    assert_eq!(
        UpdateManager::compare_versions("v1.0.0", "1.0.0").unwrap(),
        Ordering::Equal
    );
    assert_eq!(
        UpdateManager::compare_versions("v1.0.0", "v1.0.1").unwrap(),
        Ordering::Less
    );
}

#[test]
fn test_version_comparison_prerelease_stripped() {
    // Pre-release suffix is stripped, so 1.0.0-beta.1 == 1.0.0
    assert_eq!(
        UpdateManager::compare_versions("1.0.0-beta.1", "1.0.0").unwrap(),
        Ordering::Equal
    );
    assert_eq!(
        UpdateManager::compare_versions("1.0.0-alpha", "1.0.1-beta").unwrap(),
        Ordering::Less
    );
}

#[test]
fn test_version_comparison_large_numbers() {
    assert_eq!(
        UpdateManager::compare_versions("10.20.30", "10.20.31").unwrap(),
        Ordering::Less
    );
    assert_eq!(
        UpdateManager::compare_versions("99.99.99", "100.0.0").unwrap(),
        Ordering::Less
    );
}

#[test]
fn test_version_skip_then_newer() {
    let mut mgr = UpdateManager::new("1.0.0".to_string());
    mgr.skip_version("1.1.0");

    // Skipped version should not show as available
    let skipped = VersionInfo {
        version: "1.1.0".to_string(),
        release_url: String::new(),
        release_notes: String::new(),
        published_at: String::new(),
    };
    assert!(!mgr.check_update_available(&skipped));

    // But a newer version should still show
    let newer = VersionInfo {
        version: "1.2.0".to_string(),
        release_url: String::new(),
        release_notes: String::new(),
        published_at: String::new(),
    };
    assert!(mgr.check_update_available(&newer));
}

#[test]
fn test_version_parse_invalid_formats() {
    assert!(UpdateManager::parse_version("1.0").is_err());
    assert!(UpdateManager::parse_version("").is_err());
    assert!(UpdateManager::parse_version("a.b.c").is_err());
    assert!(UpdateManager::parse_version("1.2.3.4").is_err());
}

// ─── Backup Retention Enforcement ───────────────────────────────────────────

#[test]
fn test_backup_retention_enforcement_with_multiple_backups() {
    let dir = TempDir::new().unwrap();
    let max_backups = 3u32;
    let mgr = BackupManager::new(dir.path().to_path_buf(), max_backups);
    let prefs = serde_json::json!({});
    let combos = vec![
        ComboBuilder::new()
            .keyword("aa")
            .snippet("AA")
            .build()
            .unwrap(),
    ];

    // Create 6 backups
    let mut ids = Vec::new();
    for _ in 0..6 {
        std::thread::sleep(std::time::Duration::from_millis(10));
        let info = mgr.create_backup(&combos, &[], &prefs).unwrap();
        ids.push(info.id);
    }

    assert_eq!(mgr.list_backups().unwrap().len(), 6);

    // Enforce retention
    let deleted = mgr.enforce_retention().unwrap();
    assert_eq!(deleted, 3);

    let remaining = mgr.list_backups().unwrap();
    assert_eq!(remaining.len(), 3);

    // Verify the 3 newest remain (newest first in list)
    for backup in &remaining {
        // The remaining backups should be the last 3 created
        assert!(ids[3..].contains(&backup.id));
    }
}

#[test]
fn test_backup_retention_exactly_at_max() {
    let dir = TempDir::new().unwrap();
    let mgr = BackupManager::new(dir.path().to_path_buf(), 3);
    let prefs = serde_json::json!({});

    for _ in 0..3 {
        std::thread::sleep(std::time::Duration::from_millis(10));
        mgr.create_backup(&[], &[], &prefs).unwrap();
    }

    let deleted = mgr.enforce_retention().unwrap();
    assert_eq!(deleted, 0);
    assert_eq!(mgr.list_backups().unwrap().len(), 3);
}

#[test]
fn test_backup_retention_with_max_one() {
    let dir = TempDir::new().unwrap();
    let mgr = BackupManager::new(dir.path().to_path_buf(), 1);
    let prefs = serde_json::json!({});

    for _ in 0..4 {
        std::thread::sleep(std::time::Duration::from_millis(10));
        mgr.create_backup(&[], &[], &prefs).unwrap();
    }

    let deleted = mgr.enforce_retention().unwrap();
    assert_eq!(deleted, 3);
    assert_eq!(mgr.list_backups().unwrap().len(), 1);
}
