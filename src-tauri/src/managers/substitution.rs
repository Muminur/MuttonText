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

/// Chunk size for large snippet pasting (MT-1104).
const PASTE_CHUNK_SIZE: usize = 500;

/// Threshold above which snippets are pasted in chunks (MT-1104).
const CHUNKED_PASTE_THRESHOLD: usize = 1000;

/// Default substitution timeout in seconds (MT-1103).
const DEFAULT_SUBSTITUTION_TIMEOUT_SECS: u64 = 5;

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
    #[error("Target window lost focus during substitution")]
    FocusLost,
    #[error("Substitution timed out after {0} seconds")]
    Timeout(u64),
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
    /// Timeout for the entire substitution operation in seconds (MT-1103).
    pub timeout_secs: u64,
    /// Delay between chunks when pasting large snippets, in milliseconds (MT-1104).
    pub chunk_delay_ms: u64,
}

/// Trait for checking if the target window still has focus (MT-1103).
///
/// Implementations can query the OS for the current foreground window.
/// The default implementation always returns true (no focus check).
pub trait FocusChecker: Send {
    /// Returns true if the target window is still focused.
    fn is_target_focused(&self) -> bool;
}

/// Default focus checker that always reports focused (no-op).
pub struct NoOpFocusChecker;

impl FocusChecker for NoOpFocusChecker {
    fn is_target_focused(&self) -> bool {
        true
    }
}

impl Default for SubstitutionConfig {
    fn default() -> Self {
        Self {
            key_delay_ms: 5,
            paste_restore_delay_ms: 50,
            use_shift_insert: cfg!(target_os = "linux"),
            timeout_secs: DEFAULT_SUBSTITUTION_TIMEOUT_SECS,
            chunk_delay_ms: 10,
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

/// Inserts text using xdotool type command (Linux terminal compatible).
///
/// This method works in terminals (including Claude Code CLI) by using the
/// external xdotool command instead of rdev's Key::Unknown which doesn't work
/// in terminals.
pub fn insert_via_xdotool(text: &str, config: &SubstitutionConfig) -> Result<(), SubstitutionError> {
    if text.len() > MAX_SNIPPET_SIZE {
        return Err(SubstitutionError::SnippetTooLarge(text.len(), MAX_SNIPPET_SIZE));
    }
    tracing::debug!("Inserting via xdotool: {} chars", text.len());

    let output = std::process::Command::new("xdotool")
        .arg("type")
        .arg("--clearmodifiers")
        .arg("--delay")
        .arg(config.key_delay_ms.to_string())
        .arg("--")
        .arg(text)
        .output()
        .map_err(|e| SubstitutionError::SimulationFailed(format!("xdotool failed: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(SubstitutionError::SimulationFailed(format!("xdotool error: {}", stderr)));
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

    /// Performs a full substitution: delete keyword, then insert snippet.
    ///
    /// Uses xdotool type command (Linux terminal compatible).
    pub fn substitute_via_xdotool(
        &self,
        keyword_len: usize,
        snippet: &str,
    ) -> Result<(), SubstitutionError> {
        delete_keyword(keyword_len, &self.config)?;
        insert_via_xdotool(snippet, &self.config)?;
        Ok(())
    }
}

/// Checks focus before pasting and returns FocusLost error if target lost focus.
pub fn check_focus(checker: &dyn FocusChecker) -> Result<(), SubstitutionError> {
    if !checker.is_target_focused() {
        tracing::warn!("Target window lost focus during substitution");
        return Err(SubstitutionError::FocusLost);
    }
    Ok(())
}

/// Inserts a large text by splitting it into chunks and pasting each chunk
/// separately with a small delay between chunks (MT-1104).
///
/// This avoids overwhelming the target application's input buffer.
pub fn insert_via_clipboard_chunked<P: ClipboardProvider>(
    text: &str,
    clipboard_mgr: &mut ClipboardManager<P>,
    config: &SubstitutionConfig,
) -> Result<(), SubstitutionError> {
    if text.len() <= CHUNKED_PASTE_THRESHOLD {
        return insert_via_clipboard(text, clipboard_mgr, config);
    }

    tracing::debug!(
        "Chunked paste: {} chars in ~{} chunks",
        text.len(),
        (text.len() + PASTE_CHUNK_SIZE - 1) / PASTE_CHUNK_SIZE
    );

    // Preserve once at the start
    clipboard_mgr.preserve()?;

    let chars: Vec<char> = text.chars().collect();
    let mut offset = 0;

    while offset < chars.len() {
        let end = std::cmp::min(offset + PASTE_CHUNK_SIZE, chars.len());
        let chunk: String = chars[offset..end].iter().collect();

        clipboard_mgr.write(&chunk)?;
        thread::sleep(Duration::from_millis(config.key_delay_ms));

        // Simulate paste
        let delay = Duration::from_millis(config.key_delay_ms);
        let paste_modifier = if cfg!(target_os = "macos") {
            Key::MetaLeft
        } else {
            Key::ControlLeft
        };
        send_key_event(EventType::KeyPress(paste_modifier), delay)?;
        press_key(Key::KeyV, delay)?;
        send_key_event(EventType::KeyRelease(paste_modifier), delay)?;

        thread::sleep(Duration::from_millis(config.paste_restore_delay_ms));

        offset = end;

        // Inter-chunk delay
        if offset < chars.len() {
            thread::sleep(Duration::from_millis(config.chunk_delay_ms));
        }
    }

    // Restore clipboard
    let _ = clipboard_mgr.restore();

    Ok(())
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
        // use_shift_insert is platform-specific, test separately
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
            timeout_secs: 10,
            chunk_delay_ms: 20,
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
            timeout_secs: 5,
            chunk_delay_ms: 10,
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
            timeout_secs: 7,
            chunk_delay_ms: 15,
        };
        let cloned = config.clone();
        assert_eq!(cloned.key_delay_ms, 15);
        assert_eq!(cloned.paste_restore_delay_ms, 75);
        assert!(cloned.use_shift_insert);
        assert_eq!(cloned.timeout_secs, 7);
    }

    #[test]
    fn test_substitution_error_display() {
        let err = SubstitutionError::SimulationFailed("test".to_string());
        assert!(err.to_string().contains("test"));

        let err = SubstitutionError::Clipboard(ClipboardError::NothingToRestore);
        assert!(err.to_string().contains("clipboard"));
    }

    // ── MT-1103: Focus loss tests ──────────────────────────────

    struct AlwaysFocused;
    impl FocusChecker for AlwaysFocused {
        fn is_target_focused(&self) -> bool { true }
    }

    struct NeverFocused;
    impl FocusChecker for NeverFocused {
        fn is_target_focused(&self) -> bool { false }
    }

    #[test]
    fn test_check_focus_ok() {
        assert!(check_focus(&AlwaysFocused).is_ok());
    }

    #[test]
    fn test_check_focus_lost() {
        let result = check_focus(&NeverFocused);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), SubstitutionError::FocusLost));
    }

    #[test]
    fn test_focus_lost_error_display() {
        let err = SubstitutionError::FocusLost;
        assert!(err.to_string().contains("focus"));
    }

    #[test]
    fn test_timeout_error_display() {
        let err = SubstitutionError::Timeout(5);
        assert!(err.to_string().contains("5"));
    }

    // ── MT-1103: Timeout config ────────────────────────────────

    #[test]
    fn test_config_timeout_default() {
        let config = SubstitutionConfig::default();
        assert_eq!(config.timeout_secs, DEFAULT_SUBSTITUTION_TIMEOUT_SECS);
    }

    #[test]
    fn test_config_custom_timeout() {
        let config = SubstitutionConfig {
            key_delay_ms: 5,
            paste_restore_delay_ms: 50,
            use_shift_insert: false,
            timeout_secs: 10,
            chunk_delay_ms: 10,
        };
        assert_eq!(config.timeout_secs, 10);
    }

    // ── MT-1104: Constants ─────────────────────────────────────

    #[test]
    fn test_max_snippet_size_constant() {
        assert_eq!(MAX_SNIPPET_SIZE, 100_000);
    }

    #[test]
    fn test_chunked_paste_constants() {
        assert_eq!(PASTE_CHUNK_SIZE, 500);
        assert_eq!(CHUNKED_PASTE_THRESHOLD, 1000);
    }

    #[test]
    fn test_noop_focus_checker() {
        let checker = NoOpFocusChecker;
        assert!(checker.is_target_focused());
    }

    // ── Platform-specific defaults ─────────────────────────────

    #[test]
    fn test_config_use_shift_insert_default_linux() {
        let config = SubstitutionConfig::default();
        #[cfg(target_os = "linux")]
        assert!(config.use_shift_insert, "Linux should default to Shift+Insert");
        #[cfg(not(target_os = "linux"))]
        assert!(!config.use_shift_insert, "Non-Linux should default to Ctrl+V");
    }
}
