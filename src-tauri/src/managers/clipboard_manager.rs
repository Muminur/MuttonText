//! Clipboard management for MuttonText.
//!
//! Provides clipboard read/write with preserve/restore semantics so that
//! the user's clipboard content is not destroyed during snippet expansion.

use std::thread;
use std::time::Duration;

use thiserror::Error;

/// Errors arising from clipboard operations.
#[derive(Debug, Error)]
pub enum ClipboardError {
    #[error("Failed to access clipboard: {0}")]
    AccessFailed(String),
    #[error("Failed to read clipboard: {0}")]
    ReadFailed(String),
    #[error("Failed to write to clipboard: {0}")]
    WriteFailed(String),
    #[error("No preserved clipboard content to restore")]
    NothingToRestore,
}

/// Trait abstracting clipboard operations for testability.
pub trait ClipboardProvider: Send {
    /// Reads the current clipboard text content.
    fn read_text(&mut self) -> Result<String, ClipboardError>;
    /// Writes text to the clipboard.
    fn write_text(&mut self, text: &str) -> Result<(), ClipboardError>;
}

/// Real clipboard provider using arboard.
pub struct ArboardProvider {
    clipboard: arboard::Clipboard,
}

impl ArboardProvider {
    /// Creates a new arboard-backed clipboard provider.
    pub fn new() -> Result<Self, ClipboardError> {
        let clipboard = arboard::Clipboard::new()
            .map_err(|e| ClipboardError::AccessFailed(e.to_string()))?;
        Ok(Self { clipboard })
    }
}

impl ClipboardProvider for ArboardProvider {
    fn read_text(&mut self) -> Result<String, ClipboardError> {
        self.clipboard
            .get_text()
            .map_err(|e| ClipboardError::ReadFailed(e.to_string()))
    }

    fn write_text(&mut self, text: &str) -> Result<(), ClipboardError> {
        self.clipboard
            .set_text(text)
            .map_err(|e| ClipboardError::WriteFailed(e.to_string()))
    }
}

/// Manages clipboard operations with preserve/restore capability.
pub struct ClipboardManager<P: ClipboardProvider> {
    provider: P,
    preserved: Option<String>,
}

impl ClipboardManager<ArboardProvider> {
    /// Creates a new `ClipboardManager` backed by the system clipboard.
    pub fn new_system() -> Result<Self, ClipboardError> {
        Ok(Self {
            provider: ArboardProvider::new()?,
            preserved: None,
        })
    }
}

impl<P: ClipboardProvider> ClipboardManager<P> {
    /// Creates a new `ClipboardManager` with the given provider.
    pub fn new(provider: P) -> Self {
        Self {
            provider,
            preserved: None,
        }
    }

    /// Reads current clipboard text.
    pub fn read(&mut self) -> Result<String, ClipboardError> {
        tracing::debug!("Reading clipboard");
        self.provider.read_text()
    }

    /// Writes text to the clipboard.
    pub fn write(&mut self, text: &str) -> Result<(), ClipboardError> {
        tracing::debug!("Writing to clipboard: {} chars", text.len());
        self.provider.write_text(text)
    }

    /// Saves the current clipboard content for later restoration.
    pub fn preserve(&mut self) -> Result<(), ClipboardError> {
        let content = self.provider.read_text().unwrap_or_default();
        tracing::debug!("Preserving clipboard: {} chars", content.len());
        self.preserved = Some(content);
        Ok(())
    }

    /// Restores previously preserved clipboard content.
    pub fn restore(&mut self) -> Result<(), ClipboardError> {
        match self.preserved.take() {
            Some(content) => {
                tracing::debug!("Restoring clipboard: {} chars", content.len());
                self.provider.write_text(&content)
            }
            None => Err(ClipboardError::NothingToRestore),
        }
    }

    /// Returns true if there is preserved content waiting to be restored.
    pub fn has_preserved(&self) -> bool {
        self.preserved.is_some()
    }

    /// Reads clipboard text with retry logic.
    ///
    /// Retries up to `retries` times with `delay` between attempts.
    /// This helps on Windows where clipboard access can transiently fail
    /// if another application has it open.
    pub fn read_with_retry(
        &mut self,
        retries: u32,
        delay: Duration,
    ) -> Result<String, ClipboardError> {
        let mut last_err = None;
        for attempt in 0..=retries {
            match self.provider.read_text() {
                Ok(text) => return Ok(text),
                Err(e) => {
                    tracing::debug!(
                        "Clipboard read attempt {}/{} failed: {}",
                        attempt + 1,
                        retries + 1,
                        e
                    );
                    last_err = Some(e);
                    if attempt < retries {
                        thread::sleep(delay);
                    }
                }
            }
        }
        Err(last_err.unwrap_or(ClipboardError::ReadFailed(
            "All retries exhausted".to_string(),
        )))
    }

    /// Writes text to clipboard with retry logic.
    pub fn write_with_retry(
        &mut self,
        text: &str,
        retries: u32,
        delay: Duration,
    ) -> Result<(), ClipboardError> {
        let mut last_err = None;
        for attempt in 0..=retries {
            match self.provider.write_text(text) {
                Ok(()) => return Ok(()),
                Err(e) => {
                    tracing::debug!(
                        "Clipboard write attempt {}/{} failed: {}",
                        attempt + 1,
                        retries + 1,
                        e
                    );
                    last_err = Some(e);
                    if attempt < retries {
                        thread::sleep(delay);
                    }
                }
            }
        }
        Err(last_err.unwrap_or(ClipboardError::WriteFailed(
            "All retries exhausted".to_string(),
        )))
    }
}

/// Default retry count for clipboard operations.
pub const CLIPBOARD_RETRY_COUNT: u32 = 3;

/// Default delay between clipboard retry attempts.
pub const CLIPBOARD_RETRY_DELAY: Duration = Duration::from_millis(50);

/// RAII guard that saves clipboard content on creation and restores on drop.
///
/// Use this to temporarily commandeer the clipboard for snippet insertion
/// while guaranteeing the user's original content is restored afterward.
pub struct ClipboardGuard<'a, P: ClipboardProvider> {
    manager: &'a mut ClipboardManager<P>,
    saved: Option<String>,
    restored: bool,
}

impl<'a, P: ClipboardProvider> ClipboardGuard<'a, P> {
    /// Creates a new guard, saving the current clipboard content.
    /// If reading fails, saves an empty string.
    pub fn new(manager: &'a mut ClipboardManager<P>) -> Self {
        let saved = manager
            .read_with_retry(CLIPBOARD_RETRY_COUNT, CLIPBOARD_RETRY_DELAY)
            .unwrap_or_default();
        tracing::debug!("ClipboardGuard: saved {} chars", saved.len());
        Self {
            manager,
            saved: Some(saved),
            restored: false,
        }
    }

    /// Access the underlying clipboard manager for writes.
    pub fn manager_mut(&mut self) -> &mut ClipboardManager<P> {
        self.manager
    }

    /// Explicitly restore clipboard content. Called automatically on drop,
    /// but can be called early if you need error handling.
    pub fn restore(&mut self) -> Result<(), ClipboardError> {
        if self.restored {
            return Ok(());
        }
        self.restored = true;
        if let Some(content) = self.saved.take() {
            self.manager
                .write_with_retry(&content, CLIPBOARD_RETRY_COUNT, CLIPBOARD_RETRY_DELAY)
        } else {
            Ok(())
        }
    }
}

impl<P: ClipboardProvider> Drop for ClipboardGuard<'_, P> {
    fn drop(&mut self) {
        if !self.restored {
            if let Err(e) = self.restore() {
                tracing::warn!("ClipboardGuard: failed to restore clipboard on drop: {}", e);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    /// Mock clipboard provider for testing.
    struct MockProvider {
        content: Arc<Mutex<String>>,
        fail_read: bool,
        fail_write: bool,
    }

    impl MockProvider {
        fn new(initial: &str) -> Self {
            Self {
                content: Arc::new(Mutex::new(initial.to_string())),
                fail_read: false,
                fail_write: false,
            }
        }

        fn with_read_failure() -> Self {
            Self {
                content: Arc::new(Mutex::new(String::new())),
                fail_read: true,
                fail_write: false,
            }
        }

        fn with_write_failure() -> Self {
            Self {
                content: Arc::new(Mutex::new(String::new())),
                fail_read: false,
                fail_write: true,
            }
        }
    }

    impl ClipboardProvider for MockProvider {
        fn read_text(&mut self) -> Result<String, ClipboardError> {
            if self.fail_read {
                return Err(ClipboardError::ReadFailed("mock read failure".into()));
            }
            Ok(self.content.lock().unwrap().clone())
        }

        fn write_text(&mut self, text: &str) -> Result<(), ClipboardError> {
            if self.fail_write {
                return Err(ClipboardError::WriteFailed("mock write failure".into()));
            }
            *self.content.lock().unwrap() = text.to_string();
            Ok(())
        }
    }

    #[test]
    fn test_read_returns_content() {
        let mut mgr = ClipboardManager::new(MockProvider::new("hello"));
        assert_eq!(mgr.read().unwrap(), "hello");
    }

    #[test]
    fn test_write_updates_content() {
        let mut mgr = ClipboardManager::new(MockProvider::new(""));
        mgr.write("new content").unwrap();
        assert_eq!(mgr.read().unwrap(), "new content");
    }

    #[test]
    fn test_preserve_and_restore() {
        let mut mgr = ClipboardManager::new(MockProvider::new("original"));

        mgr.preserve().unwrap();
        assert!(mgr.has_preserved());

        mgr.write("temporary").unwrap();
        assert_eq!(mgr.read().unwrap(), "temporary");

        mgr.restore().unwrap();
        assert_eq!(mgr.read().unwrap(), "original");
        assert!(!mgr.has_preserved());
    }

    #[test]
    fn test_restore_without_preserve_fails() {
        let mut mgr = ClipboardManager::new(MockProvider::new("content"));
        let result = mgr.restore();
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ClipboardError::NothingToRestore));
    }

    #[test]
    fn test_preserve_replaces_previous() {
        let mut mgr = ClipboardManager::new(MockProvider::new("first"));
        mgr.preserve().unwrap();

        mgr.write("second").unwrap();
        mgr.preserve().unwrap();

        mgr.write("third").unwrap();
        mgr.restore().unwrap();
        assert_eq!(mgr.read().unwrap(), "second");
    }

    #[test]
    fn test_read_failure() {
        let mut mgr = ClipboardManager::new(MockProvider::with_read_failure());
        assert!(mgr.read().is_err());
    }

    #[test]
    fn test_write_failure() {
        let mut mgr = ClipboardManager::new(MockProvider::with_write_failure());
        assert!(mgr.write("text").is_err());
    }

    #[test]
    fn test_preserve_with_empty_clipboard() {
        let mut mgr = ClipboardManager::new(MockProvider::new(""));
        mgr.preserve().unwrap();
        mgr.write("something").unwrap();
        mgr.restore().unwrap();
        assert_eq!(mgr.read().unwrap(), "");
    }

    #[test]
    fn test_has_preserved_initially_false() {
        let mgr = ClipboardManager::new(MockProvider::new("x"));
        assert!(!mgr.has_preserved());
    }

    #[test]
    fn test_restore_clears_preserved() {
        let mut mgr = ClipboardManager::new(MockProvider::new("data"));
        mgr.preserve().unwrap();
        mgr.restore().unwrap();
        assert!(!mgr.has_preserved());
        // Second restore should fail
        assert!(mgr.restore().is_err());
    }

    #[test]
    fn test_preserve_when_read_fails_uses_empty() {
        let mut mgr = ClipboardManager::new(MockProvider::with_read_failure());
        // preserve should still succeed, using empty string as fallback
        mgr.preserve().unwrap();
        assert!(mgr.has_preserved());
    }

    // ── Retry logic tests ────────────────────────────────────────

    #[test]
    fn test_read_with_retry_succeeds_first_try() {
        let mut mgr = ClipboardManager::new(MockProvider::new("hello"));
        let result = mgr.read_with_retry(3, Duration::from_millis(1));
        assert_eq!(result.unwrap(), "hello");
    }

    #[test]
    fn test_read_with_retry_all_fail() {
        let mut mgr = ClipboardManager::new(MockProvider::with_read_failure());
        let result = mgr.read_with_retry(2, Duration::from_millis(1));
        assert!(result.is_err());
    }

    #[test]
    fn test_write_with_retry_succeeds_first_try() {
        let mut mgr = ClipboardManager::new(MockProvider::new(""));
        let result = mgr.write_with_retry("data", 3, Duration::from_millis(1));
        assert!(result.is_ok());
        assert_eq!(mgr.read().unwrap(), "data");
    }

    #[test]
    fn test_write_with_retry_all_fail() {
        let mut mgr = ClipboardManager::new(MockProvider::with_write_failure());
        let result = mgr.write_with_retry("data", 2, Duration::from_millis(1));
        assert!(result.is_err());
    }

    // ── Flaky provider for retry testing ─────────────────────────

    /// A provider that fails N times then succeeds.
    struct FlakyProvider {
        content: String,
        read_fails_remaining: std::cell::Cell<u32>,
        write_fails_remaining: std::cell::Cell<u32>,
    }

    impl FlakyProvider {
        fn new(initial: &str, read_fails: u32, write_fails: u32) -> Self {
            Self {
                content: initial.to_string(),
                read_fails_remaining: std::cell::Cell::new(read_fails),
                write_fails_remaining: std::cell::Cell::new(write_fails),
            }
        }
    }

    impl ClipboardProvider for FlakyProvider {
        fn read_text(&mut self) -> Result<String, ClipboardError> {
            let remaining = self.read_fails_remaining.get();
            if remaining > 0 {
                self.read_fails_remaining.set(remaining - 1);
                return Err(ClipboardError::ReadFailed("transient".into()));
            }
            Ok(self.content.clone())
        }

        fn write_text(&mut self, text: &str) -> Result<(), ClipboardError> {
            let remaining = self.write_fails_remaining.get();
            if remaining > 0 {
                self.write_fails_remaining.set(remaining - 1);
                return Err(ClipboardError::WriteFailed("transient".into()));
            }
            self.content = text.to_string();
            Ok(())
        }
    }

    #[test]
    fn test_read_with_retry_succeeds_after_failures() {
        let mut mgr = ClipboardManager::new(FlakyProvider::new("data", 2, 0));
        let result = mgr.read_with_retry(3, Duration::from_millis(1));
        assert_eq!(result.unwrap(), "data");
    }

    #[test]
    fn test_write_with_retry_succeeds_after_failures() {
        let mut mgr = ClipboardManager::new(FlakyProvider::new("", 0, 2));
        let result = mgr.write_with_retry("new", 3, Duration::from_millis(1));
        assert!(result.is_ok());
        assert_eq!(mgr.read().unwrap(), "new");
    }

    // ── ClipboardGuard tests ─────────────────────────────────────

    #[test]
    fn test_clipboard_guard_restores_on_drop() {
        let mut mgr = ClipboardManager::new(MockProvider::new("original"));
        {
            let mut guard = ClipboardGuard::new(&mut mgr);
            guard.manager_mut().write("temporary").unwrap();
            assert_eq!(guard.manager_mut().read().unwrap(), "temporary");
            // guard drops here, should restore "original"
        }
        assert_eq!(mgr.read().unwrap(), "original");
    }

    #[test]
    fn test_clipboard_guard_explicit_restore() {
        let mut mgr = ClipboardManager::new(MockProvider::new("original"));
        let mut guard = ClipboardGuard::new(&mut mgr);
        guard.manager_mut().write("temp").unwrap();
        guard.restore().unwrap();
        // Double restore should be no-op
        guard.restore().unwrap();
    }
}
