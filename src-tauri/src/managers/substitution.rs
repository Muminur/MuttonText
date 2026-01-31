//! Substitution engine for MuttonText.
//!
//! Handles deleting the typed keyword (via backspace key events) and inserting
//! the expanded snippet (via clipboard paste or simulated keystrokes).

use std::thread;
use std::time::Duration;

use rdev::{simulate, EventType, Key};
use thiserror::Error;

use crate::managers::clipboard_manager::{ClipboardError, ClipboardManager, ClipboardProvider};

/// Maximum allowed keyword length to prevent excessive backspace simulation.
const MAX_KEYWORD_LENGTH: usize = 256;

/// Maximum allowed snippet size to prevent excessive keystroke simulation.
const MAX_SNIPPET_SIZE: usize = 100_000;

/// Errors arising from substitution operations.
#[derive(Debug, Error)]
pub enum SubstitutionError {
    #[error("Failed to simulate key event: {0}")]
    SimulationFailed(String),
    #[error("Clipboard error during substitution: {0}")]
    Clipboard(#[from] ClipboardError),
    #[error("Keyword length {0} exceeds maximum of {1}")]
    KeywordTooLong(usize, usize),
    #[error("Snippet size {0} exceeds maximum of {1}")]
    SnippetTooLarge(usize, usize),
}

/// Configuration for the substitution engine.
#[derive(Debug, Clone)]
pub struct SubstitutionConfig {
    /// Delay between simulated key events in milliseconds.
    pub key_delay_ms: u64,
    /// Delay after paste before restoring clipboard, in milliseconds.
    pub paste_restore_delay_ms: u64,
    /// Whether to use Shift+Insert instead of Ctrl+V for pasting.
    pub use_shift_insert: bool,
}

impl Default for SubstitutionConfig {
    fn default() -> Self {
        Self {
            key_delay_ms: 5,
            paste_restore_delay_ms: 50,
            use_shift_insert: false,
        }
    }
}

/// Sends a single key event via rdev, with a configurable delay.
fn send_key_event(event_type: EventType, delay: Duration) -> Result<(), SubstitutionError> {
    simulate(&event_type).map_err(|e| {
        SubstitutionError::SimulationFailed(format!("{:?}", e))
    })?;
    thread::sleep(delay);
    Ok(())
}

/// Sends a key press (down + up) with delay.
fn press_key(key: Key, delay: Duration) -> Result<(), SubstitutionError> {
    send_key_event(EventType::KeyPress(key), delay)?;
    send_key_event(EventType::KeyRelease(key), delay)?;
    Ok(())
}

/// Deletes `count` characters by sending backspace key events.
pub fn delete_keyword(count: usize, config: &SubstitutionConfig) -> Result<(), SubstitutionError> {
    if count > MAX_KEYWORD_LENGTH {
        return Err(SubstitutionError::KeywordTooLong(count, MAX_KEYWORD_LENGTH));
    }
    tracing::debug!("Deleting {} characters via backspace", count);
    let delay = Duration::from_millis(config.key_delay_ms);
    for _ in 0..count {
        press_key(Key::Backspace, delay)?;
    }
    Ok(())
}

/// Inserts text by writing it to the clipboard and simulating paste.
///
/// Preserves and restores the user's clipboard content.
pub fn insert_via_clipboard<P: ClipboardProvider>(
    text: &str,
    clipboard_mgr: &mut ClipboardManager<P>,
    config: &SubstitutionConfig,
) -> Result<(), SubstitutionError> {
    tracing::debug!("Inserting via clipboard: {} chars", text.len());

    // Preserve current clipboard
    clipboard_mgr.preserve()?;

    // Write snippet to clipboard
    clipboard_mgr.write(text)?;

    // Small delay to ensure clipboard is ready
    thread::sleep(Duration::from_millis(config.key_delay_ms));

    // Simulate paste
    let delay = Duration::from_millis(config.key_delay_ms);
    let paste_result = if config.use_shift_insert {
        send_key_event(EventType::KeyPress(Key::ShiftLeft), delay)
            .and_then(|_| press_key(Key::Insert, delay))
            .and_then(|_| send_key_event(EventType::KeyRelease(Key::ShiftLeft), delay))
    } else {
        // Use Cmd+V on macOS, Ctrl+V on other platforms
        let paste_modifier = if cfg!(target_os = "macos") {
            Key::MetaLeft
        } else {
            Key::ControlLeft
        };
        send_key_event(EventType::KeyPress(paste_modifier), delay)
            .and_then(|_| press_key(Key::KeyV, delay))
            .and_then(|_| send_key_event(EventType::KeyRelease(paste_modifier), delay))
    };

    // Wait for paste to complete before restoring clipboard
    thread::sleep(Duration::from_millis(config.paste_restore_delay_ms));

    // Always restore clipboard, regardless of paste success/failure
    let restore_result = clipboard_mgr.restore();

    // Now propagate any errors (paste first, then restore)
    paste_result?;
    restore_result?;

    Ok(())
}

/// Inserts text by simulating individual keystrokes via rdev.
///
/// This is slower but does not disturb the clipboard.
pub fn insert_via_keystrokes(text: &str, config: &SubstitutionConfig) -> Result<(), SubstitutionError> {
    if text.len() > MAX_SNIPPET_SIZE {
        return Err(SubstitutionError::SnippetTooLarge(text.len(), MAX_SNIPPET_SIZE));
    }
    tracing::debug!("Inserting via keystrokes: {} chars", text.len());
    let delay = Duration::from_millis(config.key_delay_ms);

    for ch in text.chars() {
        // rdev supports KeyPress with unicode characters
        send_key_event(EventType::KeyPress(Key::Unknown(ch as u32)), delay)?;
        send_key_event(EventType::KeyRelease(Key::Unknown(ch as u32)), delay)?;
    }

    Ok(())
}

/// Represents a complete substitution operation.
pub struct SubstitutionEngine {
    config: SubstitutionConfig,
}

impl SubstitutionEngine {
    /// Creates a new substitution engine with the given configuration.
    pub fn new(config: SubstitutionConfig) -> Self {
        Self { config }
    }

    /// Creates a new substitution engine with default configuration.
    pub fn with_defaults() -> Self {
        Self {
            config: SubstitutionConfig::default(),
        }
    }

    /// Returns a reference to the current configuration.
    pub fn config(&self) -> &SubstitutionConfig {
        &self.config
    }

    /// Updates the configuration.
    pub fn set_config(&mut self, config: SubstitutionConfig) {
        self.config = config;
    }

    /// Performs a full substitution: delete keyword, then insert snippet.
    ///
    /// Uses clipboard-based insertion.
    pub fn substitute_via_clipboard<P: ClipboardProvider>(
        &self,
        keyword_len: usize,
        snippet: &str,
        clipboard_mgr: &mut ClipboardManager<P>,
    ) -> Result<(), SubstitutionError> {
        delete_keyword(keyword_len, &self.config)?;
        insert_via_clipboard(snippet, clipboard_mgr, &self.config)?;
        Ok(())
    }

    /// Performs a full substitution: delete keyword, then insert snippet.
    ///
    /// Uses keystroke-based insertion.
    pub fn substitute_via_keystrokes(
        &self,
        keyword_len: usize,
        snippet: &str,
    ) -> Result<(), SubstitutionError> {
        delete_keyword(keyword_len, &self.config)?;
        insert_via_keystrokes(snippet, &self.config)?;
        Ok(())
    }
}

impl Default for SubstitutionEngine {
    fn default() -> Self {
        Self::with_defaults()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: We cannot run actual rdev::simulate in unit tests (no display server
    // in CI), so tests focus on configuration, engine creation, and logic paths.
    // Integration/E2E tests cover actual key simulation.

    #[test]
    fn test_config_defaults() {
        let config = SubstitutionConfig::default();
        assert_eq!(config.key_delay_ms, 5);
        assert_eq!(config.paste_restore_delay_ms, 50);
        assert!(!config.use_shift_insert);
    }

    #[test]
    fn test_engine_creation_default() {
        let engine = SubstitutionEngine::with_defaults();
        assert_eq!(engine.config().key_delay_ms, 5);
    }

    #[test]
    fn test_engine_creation_custom() {
        let config = SubstitutionConfig {
            key_delay_ms: 10,
            paste_restore_delay_ms: 100,
            use_shift_insert: true,
        };
        let engine = SubstitutionEngine::new(config);
        assert_eq!(engine.config().key_delay_ms, 10);
        assert!(engine.config().use_shift_insert);
    }

    #[test]
    fn test_engine_set_config() {
        let mut engine = SubstitutionEngine::with_defaults();
        assert_eq!(engine.config().key_delay_ms, 5);

        engine.set_config(SubstitutionConfig {
            key_delay_ms: 20,
            paste_restore_delay_ms: 200,
            use_shift_insert: false,
        });
        assert_eq!(engine.config().key_delay_ms, 20);
        assert_eq!(engine.config().paste_restore_delay_ms, 200);
    }

    #[test]
    fn test_engine_default_trait() {
        let engine = SubstitutionEngine::default();
        assert_eq!(engine.config().key_delay_ms, 5);
    }

    #[test]
    fn test_config_clone() {
        let config = SubstitutionConfig {
            key_delay_ms: 15,
            paste_restore_delay_ms: 75,
            use_shift_insert: true,
        };
        let cloned = config.clone();
        assert_eq!(cloned.key_delay_ms, 15);
        assert_eq!(cloned.paste_restore_delay_ms, 75);
        assert!(cloned.use_shift_insert);
    }

    #[test]
    fn test_substitution_error_display() {
        let err = SubstitutionError::SimulationFailed("test".to_string());
        assert!(err.to_string().contains("test"));

        let err = SubstitutionError::Clipboard(ClipboardError::NothingToRestore);
        assert!(err.to_string().contains("clipboard"));
    }
}
