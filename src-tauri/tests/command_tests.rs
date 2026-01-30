//! Integration tests for Tauri command layer.
//!
//! These tests exercise the ComboManager (which underpins all commands)
//! directly, since Tauri commands are thin wrappers that parse strings
//! and delegate to ComboManager methods.

use muttontext_lib::managers::combo_manager::ComboManager;
use muttontext_lib::managers::combo_storage::ComboStorage;
use muttontext_lib::models::group::Group;
use muttontext_lib::models::library::ComboLibrary;
use muttontext_lib::models::matching::MatchingMode;

use tempfile::tempdir;
use uuid::Uuid;

fn make_manager() -> ComboManager {
    let tmp = tempdir().expect("tempdir");
    let path = tmp.into_path().join("combos.json");
    let storage = ComboStorage::new(path);
    let mut library = ComboLibrary::new("1.0");
    let group = Group::new("Default");
    library.add_group(group);
    ComboManager::with_library(library, storage)
}

fn default_group_id(mgr: &ComboManager) -> Uuid {
    mgr.get_all_groups()[0].id
}

// ── Combo CRUD integration tests ────────────────────────────────

#[test]
fn test_full_combo_lifecycle() {
    let mut mgr = make_manager();
    let gid = default_group_id(&mgr);

    // Create
    let combo = mgr
        .create_combo(
            "Greeting".into(),
            "greet".into(),
            "Hello, World!".into(),
            gid,
            MatchingMode::Strict,
            false,
        )
        .expect("create combo");

    assert_eq!(combo.name, "Greeting");
    assert_eq!(combo.keyword, "greet");
    assert!(combo.enabled);

    // Read
    let fetched = mgr.get_combo(combo.id).expect("combo should exist");
    assert_eq!(fetched.id, combo.id);

    // Update
    let updated = mgr
        .update_combo(
            combo.id,
            Some("Hi".into()),
            Some("hi".into()),
            None,
            None,
            Some(MatchingMode::Loose),
            None,
            None,
        )
        .expect("update combo");
    assert_eq!(updated.name, "Hi");
    assert_eq!(updated.keyword, "hi");
    assert_eq!(updated.matching_mode, MatchingMode::Loose);

    // Duplicate
    let dup = mgr.duplicate_combo(combo.id).expect("duplicate");
    assert_ne!(dup.id, combo.id);
    assert_eq!(dup.keyword, "hi");
    assert_eq!(dup.name, "Hi (copy)");
    assert_eq!(mgr.get_all_combos().len(), 2);

    // Toggle
    let state = mgr.toggle_combo(combo.id).expect("toggle");
    assert!(!state);
    let state = mgr.toggle_combo(combo.id).expect("toggle back");
    assert!(state);

    // Delete
    mgr.delete_combo(dup.id).expect("delete duplicate");
    assert_eq!(mgr.get_all_combos().len(), 1);
    mgr.delete_combo(combo.id).expect("delete original");
    assert!(mgr.get_all_combos().is_empty());
}

#[test]
fn test_combo_create_fails_with_invalid_group() {
    let mut mgr = make_manager();
    let result = mgr.create_combo(
        "Bad".into(),
        "bad".into(),
        "text".into(),
        Uuid::new_v4(),
        MatchingMode::Strict,
        false,
    );
    assert!(result.is_err());
}

#[test]
fn test_combo_create_fails_with_empty_keyword() {
    let mut mgr = make_manager();
    let gid = default_group_id(&mgr);
    let result = mgr.create_combo(
        "Empty".into(),
        "".into(),
        "text".into(),
        gid,
        MatchingMode::Strict,
        false,
    );
    assert!(result.is_err());
}

#[test]
fn test_combo_update_not_found() {
    let mut mgr = make_manager();
    let result = mgr.update_combo(
        Uuid::new_v4(),
        Some("Name".into()),
        None,
        None,
        None,
        None,
        None,
        None,
    );
    assert!(result.is_err());
}

#[test]
fn test_combo_delete_not_found() {
    let mut mgr = make_manager();
    assert!(mgr.delete_combo(Uuid::new_v4()).is_err());
}

#[test]
fn test_combo_duplicate_not_found() {
    let mut mgr = make_manager();
    assert!(mgr.duplicate_combo(Uuid::new_v4()).is_err());
}

#[test]
fn test_combo_toggle_not_found() {
    let mut mgr = make_manager();
    assert!(mgr.toggle_combo(Uuid::new_v4()).is_err());
}

#[test]
fn test_move_combo_to_group() {
    let mut mgr = make_manager();
    let gid = default_group_id(&mgr);
    let g2 = mgr
        .create_group("Other".into(), "desc".into())
        .expect("create group");

    let combo = mgr
        .create_combo(
            "Move".into(),
            "mv".into(),
            "text".into(),
            gid,
            MatchingMode::Strict,
            false,
        )
        .expect("create combo");

    mgr.move_combo_to_group(combo.id, g2.id)
        .expect("move combo");
    assert_eq!(mgr.get_combo(combo.id).unwrap().group_id, g2.id);
}

#[test]
fn test_move_combo_to_nonexistent_group() {
    let mut mgr = make_manager();
    let gid = default_group_id(&mgr);
    let combo = mgr
        .create_combo(
            "Move".into(),
            "mv".into(),
            "text".into(),
            gid,
            MatchingMode::Strict,
            false,
        )
        .expect("create combo");
    assert!(mgr.move_combo_to_group(combo.id, Uuid::new_v4()).is_err());
}

// ── Group CRUD integration tests ────────────────────────────────

#[test]
fn test_full_group_lifecycle() {
    let mut mgr = make_manager();

    // Create
    let group = mgr
        .create_group("Dev".into(), "Development snippets".into())
        .expect("create group");
    assert_eq!(group.name, "Dev");
    assert_eq!(group.description, "Development snippets");
    assert_eq!(mgr.get_all_groups().len(), 2); // Default + Dev

    // Read
    let fetched = mgr.get_group(group.id).expect("group should exist");
    assert_eq!(fetched.id, group.id);

    // Update
    let updated = mgr
        .update_group(group.id, Some("Development".into()), None)
        .expect("update group");
    assert_eq!(updated.name, "Development");

    // Toggle
    let state = mgr.toggle_group(group.id).expect("toggle");
    assert!(!state);

    // Delete
    mgr.delete_group(group.id).expect("delete group");
    assert_eq!(mgr.get_all_groups().len(), 1); // Only Default remains
}

#[test]
fn test_delete_group_cascades_combos() {
    let mut mgr = make_manager();
    let gid = default_group_id(&mgr);

    mgr.create_combo(
        "A".into(),
        "aa".into(),
        "text".into(),
        gid,
        MatchingMode::Strict,
        false,
    )
    .expect("create combo");

    mgr.create_combo(
        "B".into(),
        "bb".into(),
        "text".into(),
        gid,
        MatchingMode::Strict,
        false,
    )
    .expect("create combo");

    assert_eq!(mgr.get_all_combos().len(), 2);
    mgr.delete_group(gid).expect("delete");
    assert!(mgr.get_all_combos().is_empty());
}

#[test]
fn test_toggle_group_cascades_to_combos() {
    let mut mgr = make_manager();
    let gid = default_group_id(&mgr);

    mgr.create_combo(
        "A".into(),
        "aa".into(),
        "text".into(),
        gid,
        MatchingMode::Strict,
        false,
    )
    .expect("create combo");

    // Disable group
    let state = mgr.toggle_group(gid).expect("toggle");
    assert!(!state);

    // All combos in that group should be disabled
    for combo in mgr.get_all_combos() {
        assert!(!combo.enabled);
    }

    // Re-enable
    let state = mgr.toggle_group(gid).expect("toggle back");
    assert!(state);
    for combo in mgr.get_all_combos() {
        assert!(combo.enabled);
    }
}

#[test]
fn test_group_update_not_found() {
    let mut mgr = make_manager();
    assert!(mgr
        .update_group(Uuid::new_v4(), Some("X".into()), None)
        .is_err());
}

#[test]
fn test_group_delete_not_found() {
    let mut mgr = make_manager();
    assert!(mgr.delete_group(Uuid::new_v4()).is_err());
}

#[test]
fn test_group_toggle_not_found() {
    let mut mgr = make_manager();
    assert!(mgr.toggle_group(Uuid::new_v4()).is_err());
}

// ── Persistence integration tests ───────────────────────────────

#[test]
fn test_changes_persist_to_disk() {
    let tmp = tempdir().expect("tempdir");
    let path = tmp.path().join("combos.json");

    // Create manager, add data, drop it
    {
        let storage = ComboStorage::new(path.clone());
        let mut library = ComboLibrary::new("1.0");
        library.add_group(Group::new("Default"));
        let mut mgr = ComboManager::with_library(library, storage);
        let gid = default_group_id(&mgr);
        mgr.create_combo(
            "Persist".into(),
            "per".into(),
            "text".into(),
            gid,
            MatchingMode::Strict,
            false,
        )
        .expect("create combo");
    }

    // Reload from same path
    let storage2 = ComboStorage::new(path);
    let mgr2 = ComboManager::new(storage2).expect("reload");
    assert_eq!(mgr2.get_all_combos().len(), 1);
    assert_eq!(mgr2.get_all_combos()[0].keyword, "per");
}

// ── Error type integration tests ────────────────────────────────

#[test]
fn test_command_error_from_manager_error() {
    use muttontext_lib::commands::error::CommandError;
    use muttontext_lib::managers::combo_manager::ComboManagerError;

    let id = Uuid::new_v4();
    let err: CommandError = ComboManagerError::ComboNotFound(id).into();
    assert_eq!(err.code, "COMBO_NOT_FOUND");

    let err: CommandError = ComboManagerError::GroupNotFound(id).into();
    assert_eq!(err.code, "GROUP_NOT_FOUND");
}

#[test]
fn test_command_error_serializes_to_json() {
    use muttontext_lib::commands::error::CommandError;

    let err = CommandError {
        code: "TEST_CODE".to_string(),
        message: "A test error".to_string(),
    };
    let json = serde_json::to_string(&err).expect("serialize");
    assert!(json.contains("\"code\":\"TEST_CODE\""));
    assert!(json.contains("\"message\":\"A test error\""));
}
