//! Integration tests for the persistence layer.
//!
//! Tests cover:
//! - Save and load roundtrip for ComboLibrary
//! - Save and load roundtrip for Preferences
//! - Loading non-existent file returns defaults
//! - Atomic write integrity (no temp file remains)
//! - File path resolution
//! - Schema version embedding

use std::fs;

use muttontext_lib::managers::{ComboStorage, PreferencesStorage, storage};
use muttontext_lib::models::{ComboBuilder, Group, ComboLibrary, Preferences};

// ── ComboLibrary persistence ────────────────────────────────────────

#[test]
fn test_combo_library_save_load_roundtrip() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let path = tmp.path().join("combos.json");
    let storage = ComboStorage::new(path);

    let mut library = ComboLibrary::new("1.0");
    let group = Group::new("Integration Test Group");
    let combo = ComboBuilder::new()
        .name("Test Combo")
        .keyword("itest")
        .snippet("Integration test snippet")
        .group_id(group.id)
        .build()
        .expect("build combo");
    library.add_group(group);
    library.add_combo(combo);

    storage.save(&library).expect("save");
    let loaded = storage.load().expect("load");

    assert_eq!(loaded.groups.len(), 1);
    assert_eq!(loaded.groups[0].name, "Integration Test Group");
    assert_eq!(loaded.combos.len(), 1);
    assert_eq!(loaded.combos[0].keyword, "itest");
    assert_eq!(loaded.combos[0].snippet, "Integration test snippet");
}

#[test]
fn test_combo_library_load_nonexistent_returns_default() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let path = tmp.path().join("nonexistent_combos.json");
    let storage = ComboStorage::new(path);

    let loaded = storage.load().expect("load default");
    assert!(loaded.combos.is_empty());
}

#[test]
fn test_combo_library_multiple_groups_and_combos() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let path = tmp.path().join("combos.json");
    let storage = ComboStorage::new(path);

    let mut library = ComboLibrary::new("1.0");
    let g1 = Group::new("Work");
    let g2 = Group::new("Personal");

    let c1 = ComboBuilder::new()
        .keyword("wsig")
        .snippet("Work signature")
        .group_id(g1.id)
        .build()
        .unwrap();
    let c2 = ComboBuilder::new()
        .keyword("psig")
        .snippet("Personal signature")
        .group_id(g2.id)
        .build()
        .unwrap();

    library.add_group(g1);
    library.add_group(g2);
    library.add_combo(c1);
    library.add_combo(c2);

    storage.save(&library).expect("save");
    let loaded = storage.load().expect("load");

    assert_eq!(loaded.groups.len(), 2);
    assert_eq!(loaded.combos.len(), 2);
}

// ── Preferences persistence ─────────────────────────────────────────

#[test]
fn test_preferences_save_load_roundtrip() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let path = tmp.path().join("preferences.json");
    let storage = PreferencesStorage::new(path);

    let mut prefs = Preferences::default();
    prefs.play_sound = true;
    prefs.max_backups = 25;
    prefs.excluded_apps = vec!["1password".to_string(), "keepass".to_string()];

    storage.save(&prefs).expect("save");
    let loaded = storage.load().expect("load");

    assert!(loaded.play_sound);
    assert_eq!(loaded.max_backups, 25);
    assert_eq!(loaded.excluded_apps.len(), 2);
}

#[test]
fn test_preferences_load_nonexistent_returns_default() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let path = tmp.path().join("nonexistent_prefs.json");
    let storage = PreferencesStorage::new(path);

    let loaded = storage.load().expect("load default");
    assert!(loaded.enabled);
    assert!(!loaded.play_sound);
}

// ── Atomic write integrity ──────────────────────────────────────────

#[test]
fn test_atomic_write_no_temp_file_after_combo_save() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let path = tmp.path().join("combos.json");
    let storage = ComboStorage::new(path.clone());

    storage.save(&ComboLibrary::new("1.0")).expect("save");

    assert!(path.exists());
    assert!(!path.with_extension("tmp").exists());
}

#[test]
fn test_atomic_write_no_temp_file_after_prefs_save() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let path = tmp.path().join("preferences.json");
    let storage = PreferencesStorage::new(path.clone());

    storage.save(&Preferences::default()).expect("save");

    assert!(path.exists());
    assert!(!path.with_extension("tmp").exists());
}

// ── File path resolution ────────────────────────────────────────────

#[test]
fn test_config_dir_ends_with_muttontext() {
    if let Ok(config) = storage::get_config_dir() {
        let name = config.file_name().unwrap().to_str().unwrap();
        assert_eq!(name, "muttontext");
    }
}

#[test]
fn test_combos_path_is_inside_config_dir() {
    if let (Ok(config), Ok(combos)) = (storage::get_config_dir(), storage::get_combos_path()) {
        assert!(combos.starts_with(&config));
    }
}

#[test]
fn test_preferences_path_is_inside_config_dir() {
    if let (Ok(config), Ok(prefs)) = (storage::get_config_dir(), storage::get_preferences_path()) {
        assert!(prefs.starts_with(&config));
    }
}

#[test]
fn test_backups_dir_is_inside_config_dir() {
    if let (Ok(config), Ok(backups)) = (storage::get_config_dir(), storage::get_backups_dir()) {
        assert!(backups.starts_with(&config));
    }
}

#[test]
fn test_logs_dir_is_inside_config_dir() {
    if let (Ok(config), Ok(logs)) = (storage::get_config_dir(), storage::get_logs_dir()) {
        assert!(logs.starts_with(&config));
    }
}

// ── Schema version embedding ────────────────────────────────────────

#[test]
fn test_combo_file_contains_schema_version() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let path = tmp.path().join("combos.json");
    let storage = ComboStorage::new(path.clone());

    storage.save(&ComboLibrary::new("1.0")).expect("save");

    let content = fs::read_to_string(&path).expect("read");
    let json: serde_json::Value = serde_json::from_str(&content).expect("parse");
    assert!(json.get("schemaVersion").is_some());
    assert_eq!(json["schemaVersion"].as_u64(), Some(1));
}

#[test]
fn test_preferences_file_contains_schema_version() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let path = tmp.path().join("preferences.json");
    let storage = PreferencesStorage::new(path.clone());

    storage.save(&Preferences::default()).expect("save");

    let content = fs::read_to_string(&path).expect("read");
    let json: serde_json::Value = serde_json::from_str(&content).expect("parse");
    assert!(json.get("schemaVersion").is_some());
    assert_eq!(json["schemaVersion"].as_u64(), Some(1));
}
