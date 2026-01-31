//! Integration tests for picker window flow.
//!
//! Tests the complete flow from shortcut registration to combo search
//! and picker window operations.

use muttontext_lib::managers::combo_manager::ComboManager;
use muttontext_lib::managers::combo_storage::ComboStorage;
use muttontext_lib::managers::shortcut_manager::ShortcutManager;
use muttontext_lib::models::matching::MatchingMode;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use tempfile::tempdir;

fn make_test_combo_manager() -> ComboManager {
    let tmp = tempdir().expect("tempdir");
    let path = tmp.path().join("combos.json");
    let storage = ComboStorage::new(path);
    let mut manager = ComboManager::new(storage).expect("ComboManager::new");

    // Create test groups
    let default_group = manager.get_all_groups()[0].clone();
    let dev_group = manager
        .create_group("Development".to_string(), "Dev snippets".to_string())
        .unwrap();

    // Create test combos
    manager
        .create_combo(
            "Email Signature".to_string(),
            "sig".to_string(),
            "Best regards,\nJohn Doe".to_string(),
            default_group.id,
            MatchingMode::Strict,
            false,
        )
        .unwrap();

    manager
        .create_combo(
            "Copyright Notice".to_string(),
            "copyright".to_string(),
            "Copyright (c) 2024".to_string(),
            dev_group.id,
            MatchingMode::Strict,
            false,
        )
        .unwrap();

    manager
        .create_combo(
            "Debug Log".to_string(),
            "dbg".to_string(),
            "console.log('DEBUG:', );".to_string(),
            dev_group.id,
            MatchingMode::Strict,
            false,
        )
        .unwrap();

    manager
}

#[test]
fn test_complete_picker_flow() {
    // 1. Create managers
    let combo_manager = make_test_combo_manager();
    let mut shortcut_manager = ShortcutManager::new();

    // 2. Get default shortcut
    let default_shortcut = ShortcutManager::default_shortcut();
    assert_eq!(default_shortcut, "Ctrl+Shift+Space");

    // 3. Register the picker shortcut
    let result = shortcut_manager.register_picker_shortcut(&default_shortcut);
    assert!(result.is_ok());

    // 4. Verify shortcut is registered
    let registered = shortcut_manager.get_registered_shortcut();
    assert_eq!(registered, Some(default_shortcut.as_str()));

    // 5. Search for combos (manual implementation of search logic)
    let combos = combo_manager.get_all_combos();
    let groups = combo_manager.get_all_groups();
    let query = "sig";

    let matching_combos: Vec<_> = combos
        .iter()
        .filter(|c| c.enabled && c.keyword.to_lowercase().contains(&query.to_lowercase()))
        .collect();

    assert!(matching_combos.len() >= 1);
    assert_eq!(matching_combos[0].keyword, "sig");

    // 6. Unregister shortcut
    let result = shortcut_manager.unregister_picker_shortcut();
    assert!(result.is_ok());

    // 7. Verify shortcut is unregistered
    let registered = shortcut_manager.get_registered_shortcut();
    assert_eq!(registered, None);
}

#[test]
fn test_shortcut_callback_integration() {
    let mut shortcut_manager = ShortcutManager::new();
    let call_count = Arc::new(AtomicUsize::new(0));

    // Set up callback
    let count_clone = call_count.clone();
    shortcut_manager.set_shortcut_callback(move || {
        count_clone.fetch_add(1, Ordering::SeqCst);
    });

    // Register shortcut
    shortcut_manager
        .register_picker_shortcut("Ctrl+Shift+Space")
        .unwrap();

    // Simulate shortcut trigger (in real app, this would come from the OS)
    shortcut_manager.trigger_for_testing();

    // Verify callback was called
    assert_eq!(call_count.load(Ordering::SeqCst), 1);
}

#[test]
fn test_search_across_multiple_fields() {
    let manager = make_test_combo_manager();
    let combos = manager.get_all_combos();

    // Test keyword search
    let keyword_matches: Vec<_> = combos
        .iter()
        .filter(|c| c.enabled && c.keyword.to_lowercase().contains("sig"))
        .collect();
    assert!(keyword_matches.len() >= 1);

    // Test name search
    let name_matches: Vec<_> = combos
        .iter()
        .filter(|c| c.enabled && c.name.to_lowercase().contains("copyright"))
        .collect();
    assert_eq!(name_matches.len(), 1);

    // Test snippet search
    let snippet_matches: Vec<_> = combos
        .iter()
        .filter(|c| c.enabled && c.snippet.to_lowercase().contains("debug"))
        .collect();
    assert_eq!(snippet_matches.len(), 1);

    // Test no match
    let no_matches: Vec<_> = combos
        .iter()
        .filter(|c| c.enabled && (
            c.keyword.to_lowercase().contains("xyz123") ||
            c.name.to_lowercase().contains("xyz123") ||
            c.snippet.to_lowercase().contains("xyz123")
        ))
        .collect();
    assert_eq!(no_matches.len(), 0);
}

#[test]
fn test_shortcut_conflict_handling() {
    let mut shortcut_manager = ShortcutManager::new();

    // Register first shortcut
    shortcut_manager
        .register_picker_shortcut("Ctrl+Shift+A")
        .unwrap();

    // Register second shortcut (should replace first)
    shortcut_manager
        .register_picker_shortcut("Ctrl+Shift+B")
        .unwrap();

    // Verify only second shortcut is registered
    let registered = shortcut_manager.get_registered_shortcut();
    assert_eq!(registered, Some("Ctrl+Shift+B"));
}

#[test]
fn test_search_relevance_ranking() {
    let tmp = tempdir().expect("tempdir");
    let path = tmp.path().join("combos.json");
    let storage = ComboStorage::new(path);
    let mut manager = ComboManager::new(storage).expect("ComboManager::new");
    let default_group = manager.get_all_groups()[0].clone();

    // Create combos with different relevance levels for query "test"
    manager
        .create_combo(
            "Other Name".to_string(),
            "test".to_string(), // Exact keyword match - highest priority
            "snippet".to_string(),
            default_group.id,
            MatchingMode::Strict,
            false,
        )
        .unwrap();

    manager
        .create_combo(
            "Test Name".to_string(), // Name starts with query - medium priority
            "other".to_string(),
            "snippet".to_string(),
            default_group.id,
            MatchingMode::Strict,
            false,
        )
        .unwrap();

    manager
        .create_combo(
            "Name".to_string(),
            "kw".to_string(),
            "This is a test snippet".to_string(), // Snippet contains - lowest priority
            default_group.id,
            MatchingMode::Strict,
            false,
        )
        .unwrap();

    let combos = manager.get_all_combos();
    let query = "test";

    // Manually score combos (simplified version of search_combos logic)
    let mut scored: Vec<_> = combos
        .iter()
        .filter(|c| c.enabled)
        .filter_map(|c| {
            let keyword_lower = c.keyword.to_lowercase();
            let name_lower = c.name.to_lowercase();
            let snippet_lower = c.snippet.to_lowercase();
            let query_lower = query.to_lowercase();

            let score = if keyword_lower == query_lower {
                1000
            } else if name_lower.starts_with(&query_lower) {
                800
            } else if snippet_lower.contains(&query_lower) {
                500
            } else {
                return None;
            };

            Some((score, c))
        })
        .collect();

    scored.sort_by(|a, b| b.0.cmp(&a.0));

    // Should return all 3, with keyword match first
    assert_eq!(scored.len(), 3);
    assert_eq!(scored[0].1.keyword, "test");
    assert_eq!(scored[1].1.name, "Test Name");
}

#[test]
fn test_picker_flow_with_disabled_shortcuts() {
    let mut shortcut_manager = ShortcutManager::new();
    let call_count = Arc::new(AtomicUsize::new(0));

    // Set up callback
    let count_clone = call_count.clone();
    shortcut_manager.set_shortcut_callback(move || {
        count_clone.fetch_add(1, Ordering::SeqCst);
    });

    // Register shortcut
    shortcut_manager
        .register_picker_shortcut("Ctrl+Shift+Space")
        .unwrap();

    // Disable shortcuts
    shortcut_manager.set_enabled(false);
    assert!(!shortcut_manager.is_enabled());

    // Try to trigger (should not call callback)
    shortcut_manager.trigger_for_testing();

    // Callback should not have been called
    assert_eq!(call_count.load(Ordering::SeqCst), 0);

    // Re-enable and try again
    shortcut_manager.set_enabled(true);
    shortcut_manager.trigger_for_testing();

    // Now callback should be called
    assert_eq!(call_count.load(Ordering::SeqCst), 1);
}
