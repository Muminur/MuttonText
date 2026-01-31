//! Platform-agnostic keyboard hook traits and types.
//!
//! Defines the `KeyboardHook` and `FocusDetector` traits along with
//! shared data types (`KeyEvent`, `Key`, `Modifiers`, `WindowInfo`)
//! and the `PlatformError` error type.

use std::fmt;
use thiserror::Error;

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

/// Errors originating from the platform layer.
#[derive(Debug, Error)]
pub enum PlatformError {
    #[error("Hook already running")]
    AlreadyRunning,
    #[error("Hook not running")]
    NotRunning,
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
    #[error("Platform not supported: {0}")]
    NotSupported(String),
    #[error("Internal platform error: {0}")]
    Internal(String),
}

// ---------------------------------------------------------------------------
// Key types
// ---------------------------------------------------------------------------

/// A physical or logical key.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Key {
    Char(char),
    Backspace,
    Enter,
    Tab,
    Escape,
    Space,
    Delete,
    Left,
    Right,
    Up,
    Down,
    Home,
    End,
    PageUp,
    PageDown,
    F(u8),
    Other(String),
}

impl fmt::Display for Key {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Key::Char(c) => write!(f, "{}", c),
            Key::Backspace => write!(f, "Backspace"),
            Key::Enter => write!(f, "Enter"),
            Key::Tab => write!(f, "Tab"),
            Key::Escape => write!(f, "Escape"),
            Key::Space => write!(f, "Space"),
            Key::Delete => write!(f, "Delete"),
            Key::Left => write!(f, "Left"),
            Key::Right => write!(f, "Right"),
            Key::Up => write!(f, "Up"),
            Key::Down => write!(f, "Down"),
            Key::Home => write!(f, "Home"),
            Key::End => write!(f, "End"),
            Key::PageUp => write!(f, "PageUp"),
            Key::PageDown => write!(f, "PageDown"),
            Key::F(n) => write!(f, "F{}", n),
            Key::Other(s) => write!(f, "{}", s),
        }
    }
}

/// Whether a key was pressed or released.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyEventType {
    Press,
    Release,
}

/// Active modifier keys at the time of an event.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Modifiers {
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
    pub meta: bool,
}

impl Modifiers {
    /// Returns `true` when no modifier is held.
    pub fn is_empty(&self) -> bool {
        !self.ctrl && !self.alt && !self.shift && !self.meta
    }

    /// Returns `true` when any modifier is held.
    pub fn any(&self) -> bool {
        self.ctrl || self.alt || self.shift || self.meta
    }
}

/// A keyboard event produced by the platform hook.
#[derive(Debug, Clone)]
pub struct KeyEvent {
    pub key: Key,
    pub event_type: KeyEventType,
    pub modifiers: Modifiers,
    pub timestamp: std::time::Instant,
}

impl KeyEvent {
    /// Convenience constructor.
    pub fn new(key: Key, event_type: KeyEventType, modifiers: Modifiers) -> Self {
        Self {
            key,
            event_type,
            modifiers,
            timestamp: std::time::Instant::now(),
        }
    }

    /// Returns the character if this is a `Press` of a printable `Key::Char`
    /// with no ctrl/alt/meta modifiers held.
    pub fn printable_char(&self) -> Option<char> {
        if self.event_type != KeyEventType::Press {
            return None;
        }
        if self.modifiers.ctrl || self.modifiers.alt || self.modifiers.meta {
            return None;
        }
        match &self.key {
            Key::Char(c) => Some(*c),
            Key::Space => Some(' '),
            _ => None,
        }
    }
}

// ---------------------------------------------------------------------------
// Mouse event (simplified)
// ---------------------------------------------------------------------------

/// Simplified mouse event for buffer-reset purposes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseEventType {
    Click,
}

/// A mouse event produced by the platform hook.
#[derive(Debug, Clone)]
pub struct MouseEvent {
    pub event_type: MouseEventType,
    pub timestamp: std::time::Instant,
}

// ---------------------------------------------------------------------------
// Window info / focus detection
// ---------------------------------------------------------------------------

/// Information about the currently focused window.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WindowInfo {
    pub title: String,
    pub app_name: String,
    pub process_id: Option<u32>,
}

impl Default for WindowInfo {
    fn default() -> Self {
        Self {
            title: "Unknown".to_string(),
            app_name: "Unknown".to_string(),
            process_id: None,
        }
    }
}

// ---------------------------------------------------------------------------
// Traits
// ---------------------------------------------------------------------------

/// A system-wide keyboard listener.
pub trait KeyboardHook: Send + Sync {
    /// Start listening for keyboard events. The callback is invoked on every
    /// key press/release.
    fn start(
        &mut self,
        callback: Box<dyn Fn(KeyEvent) + Send + Sync>,
    ) -> Result<(), PlatformError>;

    /// Stop the keyboard hook.
    fn stop(&mut self) -> Result<(), PlatformError>;

    /// Returns `true` if the hook is currently active.
    fn is_running(&self) -> bool;
}

/// Detects which window currently has focus.
pub trait FocusDetector: Send + Sync {
    /// Returns information about the currently active/focused window.
    fn get_active_window_info(&self) -> Result<WindowInfo, PlatformError>;
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_display() {
        assert_eq!(Key::Char('a').to_string(), "a");
        assert_eq!(Key::Backspace.to_string(), "Backspace");
        assert_eq!(Key::F(12).to_string(), "F12");
        assert_eq!(Key::Other("XF86Audio".into()).to_string(), "XF86Audio");
    }

    #[test]
    fn test_modifiers_empty() {
        let m = Modifiers::default();
        assert!(m.is_empty());
        assert!(!m.any());
    }

    #[test]
    fn test_modifiers_any() {
        let m = Modifiers { ctrl: true, ..Default::default() };
        assert!(!m.is_empty());
        assert!(m.any());
    }

    #[test]
    fn test_key_event_printable_char() {
        let ev = KeyEvent::new(Key::Char('x'), KeyEventType::Press, Modifiers::default());
        assert_eq!(ev.printable_char(), Some('x'));
    }

    #[test]
    fn test_key_event_printable_char_space() {
        let ev = KeyEvent::new(Key::Space, KeyEventType::Press, Modifiers::default());
        assert_eq!(ev.printable_char(), Some(' '));
    }

    #[test]
    fn test_key_event_not_printable_on_release() {
        let ev = KeyEvent::new(Key::Char('x'), KeyEventType::Release, Modifiers::default());
        assert_eq!(ev.printable_char(), None);
    }

    #[test]
    fn test_key_event_not_printable_with_ctrl() {
        let mods = Modifiers { ctrl: true, ..Default::default() };
        let ev = KeyEvent::new(Key::Char('c'), KeyEventType::Press, mods);
        assert_eq!(ev.printable_char(), None);
    }

    #[test]
    fn test_key_event_not_printable_non_char() {
        let ev = KeyEvent::new(Key::Backspace, KeyEventType::Press, Modifiers::default());
        assert_eq!(ev.printable_char(), None);
    }

    #[test]
    fn test_window_info_default() {
        let info = WindowInfo::default();
        assert_eq!(info.title, "Unknown");
        assert_eq!(info.app_name, "Unknown");
        assert_eq!(info.process_id, None);
    }

    #[test]
    fn test_platform_error_display() {
        let e = PlatformError::AlreadyRunning;
        assert_eq!(e.to_string(), "Hook already running");
    }
}
