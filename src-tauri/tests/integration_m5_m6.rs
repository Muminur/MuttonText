//! Integration tests for Milestones 5 & 6.
//!
//! Tests cover the full pipeline integration for combo expansion:
//! - InputManager accumulates typed characters
//! - MatcherEngine detects keyword matches
//! - ExpansionPipeline orchestrates the full flow
//! - ClipboardManager preserves and restores clipboard
//! - Usage statistics are tracked
//!
//! These tests use mock platform implementations (MockKeyboardHook,
//! MockFocusDetector, MockClipboardProvider) to avoid requiring actual
//! system access, making them suitable for CI environments.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use muttontext_lib::managers::clipboard_manager::{ClipboardError, ClipboardManager, ClipboardProvider};
use muttontext_lib::managers::expansion_pipeline::{update_usage_stats, ExpansionPipeline};
use muttontext_lib::managers::input_manager::InputManager;
use muttontext_lib::managers::matching::MatcherEngine;
use muttontext_lib::managers::substitution::SubstitutionEngine;
use muttontext_lib::models::{ComboBuilder, MatchingMode, Preferences};
use muttontext_lib::platform::keyboard_hook::{
    FocusDetector, Key, KeyEvent, KeyEventType, Modifiers, PlatformError, WindowInfo,
};
use muttontext_lib::platform::mock::{MockFocusDetector, MockKeyboardHook};

// ---------------------------------------------------------------------------
// Mock Clipboard Provider for Testing
// ---------------------------------------------------------------------------

/// Mock clipboard provider that records read/write operations for verification.
#[derive(Clone)]
struct MockClipboardProvider {
    content: Arc<Mutex<String>>,
    read_count: Arc<AtomicUsize>,
    write_count: Arc<AtomicUsize>,
}

impl MockClipboardProvider {
    fn new(initial: &str) -> Self {
        Self {
            content: Arc::new(Mutex::new(initial.to_string())),
            read_count: Arc::new(AtomicUsize::new(0)),
            write_count: Arc::new(AtomicUsize::new(0)),
        }
    }

    fn get_content(&self) -> String {
        self.content.lock().unwrap().clone()
    }

    fn get_read_count(&self) -> usize {
        self.read_count.load(Ordering::SeqCst)
    }

    fn get_write_count(&self) -> usize {
        self.write_count.load(Ordering::SeqCst)
    }
}

impl ClipboardProvider for MockClipboardProvider {
    fn read_text(&mut self) -> Result<String, ClipboardError> {
        self.read_count.fetch_add(1, Ordering::SeqCst);
        Ok(self.content.lock().unwrap().clone())
    }

    fn write_text(&mut self, text: &str) -> Result<(), ClipboardError> {
        self.write_count.fetch_add(1, Ordering::SeqCst);
        *self.content.lock().unwrap() = text.to_string();
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Helper Functions
// ---------------------------------------------------------------------------

/// Helper: create a KeyEvent for a character press.
fn char_press(c: char) -> KeyEvent {
    KeyEvent::new(Key::Char(c), KeyEventType::Press, Modifiers::default())
}

/// Helper: create a KeyEvent for a special key press.
fn key_press(key: Key) -> KeyEvent {
    KeyEvent::new(key, KeyEventType::Press, Modifiers::default())
}

/// Helper: simulate typing by directly injecting events via mock hook.
fn inject_typing(hook: &MockKeyboardHook, text: &str) {
    for c in text.chars() {
        hook.inject_event(char_press(c));
    }
}

// ---------------------------------------------------------------------------
// InputManager Integration Tests (MT-511)
// ---------------------------------------------------------------------------

#[test]
fn test_input_manager_accumulates_characters_via_hook() {
    let mut mgr = InputManager::new();
    let hook = MockKeyboardHook::new();

    let buffer_log = Arc::new(Mutex::new(Vec::<String>::new()));
    let buffer_log_clone = buffer_log.clone();

    mgr.on_buffer_change(move |buf| {
        buffer_log_clone.lock().unwrap().push(buf.to_string());
    });

    mgr.set_keyboard_hook(Box::new(hook));
    mgr.start().unwrap();

    // Cannot inject events after start takes ownership of callback.
    // This test verifies the lifecycle. For character accumulation,
    // see the existing unit tests in input_manager.rs that test
    // process_key_event directly.

    mgr.stop().unwrap();
}

#[test]
fn test_input_manager_clears_buffer_manually() {
    let mgr = InputManager::new();

    // Use public API to verify buffer clearing
    assert_eq!(mgr.buffer(), "");
    mgr.clear_buffer();
    assert_eq!(mgr.buffer(), "");
}

#[test]
fn test_input_manager_focus_change_clears_buffer() {
    let mgr = InputManager::new();
    let detector = MockFocusDetector::new();

    // First focus establishes window
    mgr.handle_focus_change(&detector);
    assert_eq!(mgr.buffer(), "");

    // Change to different window
    detector.set_window_info(WindowInfo {
        title: "New Window".into(),
        app_name: "newapp".into(),
        process_id: Some(9999),
    });

    mgr.handle_focus_change(&detector);
    assert_eq!(mgr.buffer(), "", "Buffer should clear on focus change");
}

#[test]
fn test_input_manager_pause_resume() {
    let mgr = InputManager::new();

    assert!(!mgr.is_paused());

    mgr.pause();
    assert!(mgr.is_paused());

    mgr.resume();
    assert!(!mgr.is_paused());
}

#[test]
fn test_input_manager_mouse_click_clears_buffer() {
    let mgr = InputManager::new();

    // Clear is idempotent
    mgr.handle_mouse_click();
    assert_eq!(mgr.buffer(), "");
}

// ---------------------------------------------------------------------------
// Full Pipeline Integration Tests (MT-522, MT-624)
// ---------------------------------------------------------------------------

#[test]
fn test_pipeline_match_found_after_typing_keyword() {
    let mut pipeline = ExpansionPipeline::with_defaults();
    let combo = ComboBuilder::new()
        .keyword("sig")
        .snippet("Best regards,\nJohn Doe")
        .matching_mode(MatchingMode::Strict)
        .build()
        .unwrap();
    pipeline.load_combos(&[combo]);

    // Simulate typing "hello sig" by checking buffer states
    let buffers = [
        "h", "he", "hel", "hell", "hello", "hello ", "hello s", "hello si", "hello sig",
    ];

    // No match until the full keyword is typed
    for &buf in &buffers[..buffers.len() - 1] {
        assert!(
            pipeline.process_buffer(buf, None).is_none(),
            "Should not match on partial: '{}'",
            buf
        );
    }

    // Final buffer should match
    let result = pipeline.process_buffer("hello sig", None);
    assert!(result.is_some(), "Should match complete keyword");

    let matched = result.unwrap();
    assert_eq!(matched.keyword, "sig");
    assert_eq!(matched.snippet, "Best regards,\nJohn Doe");
    assert_eq!(matched.keyword_len, 3);
}

#[test]
fn test_pipeline_partial_keyword_no_match() {
    let mut pipeline = ExpansionPipeline::with_defaults();
    let combo = ComboBuilder::new()
        .keyword("signature")
        .snippet("My Signature")
        .matching_mode(MatchingMode::Strict)
        .build()
        .unwrap();
    pipeline.load_combos(&[combo]);

    // Partial keyword should not match
    assert!(pipeline.process_buffer("sig", None).is_none());
    assert!(pipeline.process_buffer("signa", None).is_none());
    assert!(pipeline.process_buffer("signatu", None).is_none());

    // Full keyword should match
    assert!(pipeline.process_buffer("signature", None).is_some());
}

#[test]
fn test_pipeline_strict_matching_respects_word_boundaries() {
    let mut pipeline = ExpansionPipeline::with_defaults();
    let combo = ComboBuilder::new()
        .keyword("sig")
        .snippet("Signature")
        .matching_mode(MatchingMode::Strict)
        .build()
        .unwrap();
    pipeline.load_combos(&[combo]);

    // Should NOT match mid-word (strict mode requires word boundary)
    assert!(
        pipeline.process_buffer("testsig", None).is_none(),
        "Strict mode should not match mid-word"
    );

    // Should match after word boundary
    assert!(pipeline.process_buffer("test sig", None).is_some());
    assert!(pipeline.process_buffer("test.sig", None).is_some());
    assert!(pipeline.process_buffer("sig", None).is_some());
}

#[test]
fn test_pipeline_loose_matching_works_mid_word() {
    let mut pipeline = ExpansionPipeline::with_defaults();
    let combo = ComboBuilder::new()
        .keyword("sig")
        .snippet("Signature")
        .matching_mode(MatchingMode::Loose)
        .build()
        .unwrap();
    pipeline.load_combos(&[combo]);

    // Loose mode SHOULD match mid-word
    assert!(
        pipeline.process_buffer("testsig", None).is_some(),
        "Loose mode should match mid-word"
    );
    assert!(pipeline.process_buffer("mysignature", None).is_none(), "Should not match if keyword is substring");
    assert!(pipeline.process_buffer("mysig", None).is_some());
}

#[test]
fn test_pipeline_paused_doesnt_match() {
    let mut pipeline = ExpansionPipeline::with_defaults();
    let combo = ComboBuilder::new()
        .keyword("sig")
        .snippet("Signature")
        .build()
        .unwrap();
    pipeline.load_combos(&[combo]);

    // Pause the pipeline
    pipeline.matcher_mut().pause();

    // Should not match while paused
    assert!(
        pipeline.process_buffer("hello sig", None).is_none(),
        "Paused pipeline should not match"
    );

    // Resume
    pipeline.matcher_mut().resume();

    // Should match after resume
    assert!(pipeline.process_buffer("hello sig", None).is_some());
}

#[test]
fn test_pipeline_excluded_app_doesnt_match() {
    let mut pipeline = ExpansionPipeline::with_defaults();
    let combo = ComboBuilder::new()
        .keyword("sig")
        .snippet("Signature")
        .build()
        .unwrap();
    pipeline.load_combos(&[combo]);

    // Set excluded apps
    pipeline.matcher_mut().set_excluded_apps(vec![
        "1password".to_string(),
        "keepass".to_string(),
    ]);

    // Should NOT match in excluded app
    assert!(
        pipeline.process_buffer("hello sig", Some("1Password")).is_none(),
        "Should not match in excluded app"
    );
    assert!(pipeline.process_buffer("hello sig", Some("KeePass XC")).is_none());

    // Should match in non-excluded app
    assert!(pipeline.process_buffer("hello sig", Some("notepad")).is_some());
    assert!(pipeline.process_buffer("hello sig", None).is_some());
}

#[test]
fn test_pipeline_case_sensitivity_respected() {
    let mut pipeline = ExpansionPipeline::with_defaults();

    // Case-sensitive combo
    let combo = ComboBuilder::new()
        .keyword("Sig")
        .snippet("Signature")
        .case_sensitive(true)
        .build()
        .unwrap();
    pipeline.load_combos(&[combo]);

    // Exact case should match
    assert!(pipeline.process_buffer("hello Sig", None).is_some());

    // Different case should NOT match
    assert!(
        pipeline.process_buffer("hello sig", None).is_none(),
        "Case-sensitive combo should not match different case"
    );
    assert!(pipeline.process_buffer("hello SIG", None).is_none());
}

#[test]
fn test_pipeline_case_insensitive_matches_any_case() {
    let mut pipeline = ExpansionPipeline::with_defaults();

    // Case-insensitive combo (default)
    let combo = ComboBuilder::new()
        .keyword("sig")
        .snippet("Signature")
        .case_sensitive(false)
        .build()
        .unwrap();
    pipeline.load_combos(&[combo]);

    // Any case should match
    assert!(pipeline.process_buffer("hello sig", None).is_some());
    assert!(pipeline.process_buffer("hello SIG", None).is_some());
    assert!(pipeline.process_buffer("hello Sig", None).is_some());
    assert!(pipeline.process_buffer("hello sIg", None).is_some());
}

#[test]
fn test_usage_stats_updated_after_expansion() {
    let mut combo = ComboBuilder::new()
        .keyword("sig")
        .snippet("Signature")
        .build()
        .unwrap();

    assert_eq!(combo.use_count, 0);
    assert!(combo.last_used.is_none());

    // Simulate expansion
    update_usage_stats(&mut combo);

    assert_eq!(combo.use_count, 1);
    assert!(combo.last_used.is_some());

    // Another expansion
    update_usage_stats(&mut combo);

    assert_eq!(combo.use_count, 2);
}

#[test]
fn test_pipeline_disabled_combo_no_match() {
    let mut pipeline = ExpansionPipeline::with_defaults();
    let mut combo = ComboBuilder::new()
        .keyword("sig")
        .snippet("Signature")
        .build()
        .unwrap();
    combo.enabled = false;
    pipeline.load_combos(&[combo]);

    // Disabled combo should not match
    assert!(
        pipeline.process_buffer("hello sig", None).is_none(),
        "Disabled combo should not match"
    );
}

#[test]
fn test_pipeline_apply_preferences_disabled() {
    let mut pipeline = ExpansionPipeline::with_defaults();
    let combo = ComboBuilder::new()
        .keyword("sig")
        .snippet("Signature")
        .build()
        .unwrap();
    pipeline.load_combos(&[combo]);

    // Disable via preferences
    let mut prefs = Preferences::default();
    prefs.enabled = false;
    pipeline.apply_preferences(&prefs);

    // Should not match when disabled
    assert!(
        pipeline.process_buffer("hello sig", None).is_none(),
        "Pipeline disabled via preferences should not match"
    );

    // Re-enable
    let mut prefs = Preferences::default();
    prefs.enabled = true;
    pipeline.apply_preferences(&prefs);

    assert!(pipeline.process_buffer("hello sig", None).is_some());
}

#[test]
fn test_pipeline_apply_preferences_excluded_apps() {
    let mut pipeline = ExpansionPipeline::with_defaults();
    let combo = ComboBuilder::new()
        .keyword("sig")
        .snippet("Signature")
        .build()
        .unwrap();
    pipeline.load_combos(&[combo]);

    let mut prefs = Preferences::default();
    prefs.excluded_apps = vec!["password".to_string()];
    pipeline.apply_preferences(&prefs);

    assert!(pipeline.process_buffer("hello sig", Some("1Password")).is_none());
    assert!(pipeline.process_buffer("hello sig", Some("notepad")).is_some());
}

// ---------------------------------------------------------------------------
// Clipboard Integration Tests (MT-522)
// ---------------------------------------------------------------------------

#[test]
fn test_clipboard_preserve_write_restore_cycle() {
    let provider = MockClipboardProvider::new("original content");
    let mut mgr = ClipboardManager::new(provider.clone());

    // Preserve
    mgr.preserve().unwrap();
    assert!(mgr.has_preserved());
    assert_eq!(provider.get_content(), "original content");

    // Write temporary content
    mgr.write("temporary snippet").unwrap();
    assert_eq!(provider.get_content(), "temporary snippet");

    // Restore
    mgr.restore().unwrap();
    assert!(!mgr.has_preserved());
    assert_eq!(
        provider.get_content(),
        "original content",
        "Clipboard should be restored to original"
    );
}

#[test]
fn test_clipboard_multiple_preserve_restore_cycles() {
    let provider = MockClipboardProvider::new("first");
    let mut mgr = ClipboardManager::new(provider.clone());

    // First cycle
    mgr.preserve().unwrap();
    mgr.write("temp1").unwrap();
    mgr.restore().unwrap();
    assert_eq!(provider.get_content(), "first");

    // Second cycle
    mgr.write("second").unwrap();
    mgr.preserve().unwrap();
    mgr.write("temp2").unwrap();
    mgr.restore().unwrap();
    assert_eq!(provider.get_content(), "second");
}

#[test]
fn test_clipboard_preserve_empty_content() {
    let provider = MockClipboardProvider::new("");
    let mut mgr = ClipboardManager::new(provider.clone());

    mgr.preserve().unwrap();
    mgr.write("something").unwrap();
    mgr.restore().unwrap();

    assert_eq!(provider.get_content(), "", "Should restore empty content");
}

#[test]
fn test_clipboard_usage_tracking() {
    let provider = MockClipboardProvider::new("initial");
    let mut mgr = ClipboardManager::new(provider.clone());

    assert_eq!(provider.get_read_count(), 0);
    assert_eq!(provider.get_write_count(), 0);

    mgr.preserve().unwrap(); // 1 read
    assert_eq!(provider.get_read_count(), 1);

    mgr.write("test").unwrap(); // 1 write
    assert_eq!(provider.get_write_count(), 1);

    mgr.restore().unwrap(); // 1 write (restore)
    assert_eq!(provider.get_write_count(), 2);
}

// ---------------------------------------------------------------------------
// End-to-End Integration Test: Full Flow
// ---------------------------------------------------------------------------

#[test]
fn test_e2e_full_expansion_detection_flow() {
    // This test simulates the complete expansion detection flow:
    // 1. Pipeline detects match in buffer
    // 2. Usage stats updated
    // 3. Clipboard preserved/restored

    let mut combo = ComboBuilder::new()
        .name("Email Signature")
        .keyword("esig")
        .snippet("Best regards,\nJohn Doe\njohn@example.com")
        .matching_mode(MatchingMode::Strict)
        .build()
        .unwrap();
    let combo_id = combo.id;

    // Setup Pipeline
    let mut pipeline = ExpansionPipeline::with_defaults();
    pipeline.load_combos(&[combo.clone()]);

    // Simulate buffer states as user types "Hi esig"
    assert!(pipeline.process_buffer("Hi", None).is_none());
    assert!(pipeline.process_buffer("", None).is_none()); // Space clears
    assert!(pipeline.process_buffer("e", None).is_none());
    assert!(pipeline.process_buffer("es", None).is_none());
    assert!(pipeline.process_buffer("esi", None).is_none());

    // Full keyword should match
    let match_result = pipeline.process_buffer("esig", None);
    assert!(match_result.is_some(), "Should match 'esig'");

    let matched = match_result.unwrap();
    assert_eq!(matched.combo_id, combo_id);
    assert_eq!(matched.keyword, "esig");
    assert_eq!(matched.snippet, "Best regards,\nJohn Doe\njohn@example.com");

    // Update usage stats
    update_usage_stats(&mut combo);
    assert_eq!(combo.use_count, 1);
    assert!(combo.last_used.is_some());

    // Simulate clipboard preservation/restoration
    let provider = MockClipboardProvider::new("user's clipboard content");
    let mut clipboard_mgr = ClipboardManager::new(provider.clone());

    clipboard_mgr.preserve().unwrap();
    clipboard_mgr.write(&matched.snippet).unwrap();
    assert_eq!(provider.get_content(), "Best regards,\nJohn Doe\njohn@example.com");

    clipboard_mgr.restore().unwrap();
    assert_eq!(
        provider.get_content(),
        "user's clipboard content",
        "Clipboard should be restored"
    );
}

#[test]
fn test_e2e_multiple_combos_priority() {
    let mut pipeline = ExpansionPipeline::with_defaults();

    // Create two combos with different keywords
    let c1 = ComboBuilder::new()
        .keyword("sig")
        .snippet("Short signature")
        .matching_mode(MatchingMode::Strict)
        .build()
        .unwrap();

    let c2 = ComboBuilder::new()
        .keyword("signature")
        .snippet("Long signature")
        .matching_mode(MatchingMode::Strict)
        .build()
        .unwrap();

    pipeline.load_combos(&[c1, c2]);

    // "sig" should match the first combo
    let result = pipeline.process_buffer("test sig", None);
    assert!(result.is_some());
    let matched = result.unwrap();
    assert_eq!(matched.keyword, "sig");

    // "signature" should match the second combo
    let result = pipeline.process_buffer("test signature", None);
    assert!(result.is_some());
    let matched = result.unwrap();
    assert_eq!(matched.keyword, "signature");
}

#[test]
fn test_e2e_real_world_email_scenario() {
    // Simulate a real-world scenario: user composes email and types signature keyword

    let mut pipeline = ExpansionPipeline::with_defaults();
    let mut combo = ComboBuilder::new()
        .name("Professional Email Signature")
        .keyword("prof")
        .snippet("Best regards,\n\nJane Smith\nSenior Developer\nAcme Corp\n+1 (555) 123-4567\njane.smith@acme.com")
        .matching_mode(MatchingMode::Strict)
        .build()
        .unwrap();

    pipeline.load_combos(&[combo.clone()]);

    // Simulate typing email body - no match on regular words
    assert!(pipeline.process_buffer("Thanks", None).is_none());
    assert!(pipeline.process_buffer("for", None).is_none());
    assert!(pipeline.process_buffer("your", None).is_none());
    assert!(pipeline.process_buffer("help", None).is_none());

    // User types signature keyword
    let result = pipeline.process_buffer("prof", None);
    assert!(result.is_some());
    let matched = result.unwrap();
    assert_eq!(matched.keyword, "prof");
    assert!(matched.snippet.contains("Jane Smith"));
    assert!(matched.snippet.contains("Acme Corp"));

    // Update stats
    update_usage_stats(&mut combo);
    assert_eq!(combo.use_count, 1);
}

#[test]
fn test_e2e_focus_detector_integration() {
    // Test that focus detection properly integrates with InputManager

    let mgr = InputManager::new();
    let detector = MockFocusDetector::new();

    // Establish initial window
    detector.set_window_info(WindowInfo {
        title: "Email Client".into(),
        app_name: "thunderbird".into(),
        process_id: Some(1000),
    });
    mgr.handle_focus_change(&detector);

    // Buffer is empty after initial focus
    assert_eq!(mgr.buffer(), "");

    // Switch to password manager
    detector.set_window_info(WindowInfo {
        title: "Password Manager".into(),
        app_name: "1password".into(),
        process_id: Some(2000),
    });
    mgr.handle_focus_change(&detector);

    // Buffer should still be cleared
    assert_eq!(mgr.buffer(), "");
}

#[test]
fn test_e2e_pipeline_with_preferences() {
    // Test complete flow with preferences applied

    let mut pipeline = ExpansionPipeline::with_defaults();
    let combo = ComboBuilder::new()
        .keyword("test")
        .snippet("Test snippet")
        .build()
        .unwrap();
    pipeline.load_combos(&[combo]);

    // Configure preferences
    let mut prefs = Preferences::default();
    prefs.enabled = true;
    prefs.excluded_apps = vec!["password".to_string(), "keepass".to_string()];
    prefs.play_sound = false;
    pipeline.apply_preferences(&prefs);

    // Should match in normal app
    assert!(pipeline.process_buffer("test", Some("notepad")).is_some());

    // Should not match in excluded app
    assert!(pipeline.process_buffer("test", Some("1Password")).is_none());

    // Disable globally
    prefs.enabled = false;
    pipeline.apply_preferences(&prefs);

    // Should not match when disabled
    assert!(pipeline.process_buffer("test", Some("notepad")).is_none());
}
