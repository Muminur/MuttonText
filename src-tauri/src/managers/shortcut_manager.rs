//! Global keyboard shortcut management for the picker window.

use std::sync::{Arc, Mutex};
use thiserror::Error;

/// Errors that can occur during shortcut registration.
#[derive(Debug, Error, Clone, PartialEq)]
pub enum ShortcutError {
    #[error("Invalid shortcut format: {0}")]
    InvalidFormat(String),
    #[error("Shortcut already registered: {0}")]
    AlreadyRegistered(String),
    #[error("Failed to register shortcut: {0}")]
    RegistrationFailed(String),
    #[error("Failed to unregister shortcut: {0}")]
    UnregistrationFailed(String),
    #[error("No shortcut is currently registered")]
    NoShortcutRegistered,
}

/// Callback type for when a shortcut is pressed.
pub type ShortcutCallback = Arc<dyn Fn() + Send + Sync>;

/// Manages global keyboard shortcuts for the picker window.
///
/// This manager handles registration and unregistration of global shortcuts
/// and invokes callbacks when shortcuts are triggered.
pub struct ShortcutManager {
    /// Currently registered shortcut string (e.g., "Ctrl+Shift+Space").
    registered_shortcut: Option<String>,
    /// Callback to invoke when the shortcut is pressed.
    callback: Option<ShortcutCallback>,
    /// Whether the manager is enabled.
    enabled: bool,
}

impl ShortcutManager {
    /// Creates a new `ShortcutManager` with no shortcuts registered.
    pub fn new() -> Self {
        Self {
            registered_shortcut: None,
            callback: None,
            enabled: true,
        }
    }

    /// Returns the default picker shortcut.
    pub fn default_shortcut() -> String {
        "Ctrl+Shift+Space".to_string()
    }

    /// Validates the shortcut format.
    ///
    /// Valid formats:
    /// - "Ctrl+Shift+Space"
    /// - "Alt+K"
    /// - "Ctrl+Alt+F12"
    /// - etc.
    fn validate_shortcut(shortcut: &str) -> Result<(), ShortcutError> {
        if shortcut.trim().is_empty() {
            return Err(ShortcutError::InvalidFormat(
                "Shortcut cannot be empty".to_string(),
            ));
        }

        // Basic validation - should contain at least one modifier and a key
        let parts: Vec<&str> = shortcut.split('+').collect();
        if parts.len() < 2 {
            return Err(ShortcutError::InvalidFormat(
                format!("Shortcut must contain at least one modifier and a key: {}", shortcut),
            ));
        }

        // Check for valid modifiers
        let valid_modifiers = ["Ctrl", "Alt", "Shift", "Super", "Meta", "Cmd", "Win"];
        let key = parts.last().unwrap();

        for part in parts.iter().take(parts.len() - 1) {
            if !valid_modifiers.contains(part) {
                return Err(ShortcutError::InvalidFormat(
                    format!("Invalid modifier '{}' in shortcut: {}", part, shortcut),
                ));
            }
        }

        // Key should not be empty
        if key.trim().is_empty() {
            return Err(ShortcutError::InvalidFormat(
                format!("Key cannot be empty in shortcut: {}", shortcut),
            ));
        }

        Ok(())
    }

    /// Registers a global shortcut.
    ///
    /// If a shortcut is already registered, it will be unregistered first.
    /// Returns an error if the shortcut format is invalid or registration fails.
    pub fn register_picker_shortcut(&mut self, shortcut: &str) -> Result<(), ShortcutError> {
        // Validate format
        Self::validate_shortcut(shortcut)?;

        // Unregister existing shortcut if any
        if self.registered_shortcut.is_some() {
            self.unregister_picker_shortcut()?;
        }

        // In a real implementation, this would use Tauri's global shortcut plugin
        // For now, we just track the registered shortcut
        tracing::info!("Registering global shortcut: {}", shortcut);

        self.registered_shortcut = Some(shortcut.to_string());

        Ok(())
    }

    /// Unregisters the currently registered shortcut.
    ///
    /// Returns an error if no shortcut is registered or unregistration fails.
    pub fn unregister_picker_shortcut(&mut self) -> Result<(), ShortcutError> {
        if self.registered_shortcut.is_none() {
            return Err(ShortcutError::NoShortcutRegistered);
        }

        let shortcut = self.registered_shortcut.take().unwrap();

        tracing::info!("Unregistering global shortcut: {}", shortcut);

        // In a real implementation, this would call the platform-specific unregister API
        // For now, we just clear the tracking

        Ok(())
    }

    /// Sets the callback to invoke when the shortcut is pressed.
    pub fn set_shortcut_callback<F>(&mut self, callback: F)
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.callback = Some(Arc::new(callback));
    }

    /// Gets the currently registered shortcut, if any.
    pub fn get_registered_shortcut(&self) -> Option<&str> {
        self.registered_shortcut.as_deref()
    }

    /// Enables or disables the shortcut manager.
    ///
    /// When disabled, shortcuts will not trigger callbacks even if registered.
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Returns whether the manager is enabled.
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Triggers the callback (used for testing and internal implementation).
    #[cfg(test)]
    pub fn trigger_for_testing(&self) {
        if !self.enabled {
            return;
        }

        if let Some(ref callback) = self.callback {
            callback();
        }
    }
}

impl Default for ShortcutManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicBool, Ordering};

    #[test]
    fn test_new_manager_no_shortcuts() {
        let manager = ShortcutManager::new();
        assert!(manager.get_registered_shortcut().is_none());
        assert!(manager.is_enabled());
    }

    #[test]
    fn test_default_shortcut() {
        let shortcut = ShortcutManager::default_shortcut();
        assert_eq!(shortcut, "Ctrl+Shift+Space");
    }

    #[test]
    fn test_validate_shortcut_valid() {
        assert!(ShortcutManager::validate_shortcut("Ctrl+Shift+Space").is_ok());
        assert!(ShortcutManager::validate_shortcut("Alt+K").is_ok());
        assert!(ShortcutManager::validate_shortcut("Ctrl+Alt+F12").is_ok());
        assert!(ShortcutManager::validate_shortcut("Shift+A").is_ok());
    }

    #[test]
    fn test_validate_shortcut_empty() {
        let result = ShortcutManager::validate_shortcut("");
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            ShortcutError::InvalidFormat("Shortcut cannot be empty".to_string())
        );
    }

    #[test]
    fn test_validate_shortcut_no_modifier() {
        let result = ShortcutManager::validate_shortcut("Space");
        assert!(result.is_err());
        match result.unwrap_err() {
            ShortcutError::InvalidFormat(msg) => {
                assert!(msg.contains("at least one modifier"));
            }
            _ => panic!("Expected InvalidFormat error"),
        }
    }

    #[test]
    fn test_validate_shortcut_invalid_modifier() {
        let result = ShortcutManager::validate_shortcut("Invalid+Space");
        assert!(result.is_err());
        match result.unwrap_err() {
            ShortcutError::InvalidFormat(msg) => {
                assert!(msg.contains("Invalid modifier"));
            }
            _ => panic!("Expected InvalidFormat error"),
        }
    }

    #[test]
    fn test_register_shortcut_success() {
        let mut manager = ShortcutManager::new();
        let result = manager.register_picker_shortcut("Ctrl+Shift+Space");

        assert!(result.is_ok());
        assert_eq!(
            manager.get_registered_shortcut(),
            Some("Ctrl+Shift+Space")
        );
    }

    #[test]
    fn test_register_shortcut_invalid_format() {
        let mut manager = ShortcutManager::new();
        let result = manager.register_picker_shortcut("InvalidShortcut");

        assert!(result.is_err());
        assert!(manager.get_registered_shortcut().is_none());
    }

    #[test]
    fn test_register_replaces_existing() {
        let mut manager = ShortcutManager::new();

        manager.register_picker_shortcut("Ctrl+Shift+Space").unwrap();
        assert_eq!(
            manager.get_registered_shortcut(),
            Some("Ctrl+Shift+Space")
        );

        manager.register_picker_shortcut("Alt+K").unwrap();
        assert_eq!(manager.get_registered_shortcut(), Some("Alt+K"));
    }

    #[test]
    fn test_unregister_shortcut_success() {
        let mut manager = ShortcutManager::new();

        manager.register_picker_shortcut("Ctrl+Shift+Space").unwrap();
        let result = manager.unregister_picker_shortcut();

        assert!(result.is_ok());
        assert!(manager.get_registered_shortcut().is_none());
    }

    #[test]
    fn test_unregister_when_none_registered() {
        let mut manager = ShortcutManager::new();
        let result = manager.unregister_picker_shortcut();

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), ShortcutError::NoShortcutRegistered);
    }

    #[test]
    fn test_set_shortcut_callback() {
        let mut manager = ShortcutManager::new();
        let called = Arc::new(AtomicBool::new(false));
        let called_clone = called.clone();

        manager.set_shortcut_callback(move || {
            called_clone.store(true, Ordering::SeqCst);
        });

        // Trigger the callback
        manager.trigger_for_testing();

        assert!(called.load(Ordering::SeqCst));
    }

    #[test]
    fn test_callback_not_triggered_when_disabled() {
        let mut manager = ShortcutManager::new();
        let called = Arc::new(AtomicBool::new(false));
        let called_clone = called.clone();

        manager.set_shortcut_callback(move || {
            called_clone.store(true, Ordering::SeqCst);
        });

        manager.set_enabled(false);
        assert!(!manager.is_enabled());

        manager.trigger_for_testing();

        assert!(!called.load(Ordering::SeqCst));
    }

    #[test]
    fn test_enable_disable() {
        let mut manager = ShortcutManager::new();

        assert!(manager.is_enabled());

        manager.set_enabled(false);
        assert!(!manager.is_enabled());

        manager.set_enabled(true);
        assert!(manager.is_enabled());
    }

    #[test]
    fn test_multiple_callbacks() {
        let mut manager = ShortcutManager::new();
        let count = Arc::new(Mutex::new(0));

        // Set first callback
        let count_clone = count.clone();
        manager.set_shortcut_callback(move || {
            let mut c = count_clone.lock().unwrap();
            *c += 1;
        });

        manager.trigger_for_testing();
        assert_eq!(*count.lock().unwrap(), 1);

        // Set second callback (should replace first)
        let count_clone2 = count.clone();
        manager.set_shortcut_callback(move || {
            let mut c = count_clone2.lock().unwrap();
            *c += 10;
        });

        manager.trigger_for_testing();
        // Should be 11 (1 from first trigger + 10 from second)
        assert_eq!(*count.lock().unwrap(), 11);
    }

    #[test]
    fn test_default_trait() {
        let manager = ShortcutManager::default();
        assert!(manager.get_registered_shortcut().is_none());
        assert!(manager.is_enabled());
    }

    #[test]
    fn test_validate_with_whitespace() {
        let result = ShortcutManager::validate_shortcut("   ");
        assert!(result.is_err());
    }

    #[test]
    fn test_register_empty_shortcut() {
        let mut manager = ShortcutManager::new();
        let result = manager.register_picker_shortcut("");
        assert!(result.is_err());
    }
}
