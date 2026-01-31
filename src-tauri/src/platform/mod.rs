//! Platform-specific implementations for keyboard hooking and focus detection.
//!
//! This module defines cross-platform traits (`KeyboardHook`, `FocusDetector`)
//! and provides platform-specific implementations conditionally compiled for
//! Linux and macOS. A `mock` module is always available for testing.

pub mod keyboard_hook;
pub mod mock;

#[cfg(any(target_os = "linux", target_os = "macos"))]
pub(crate) mod rdev_common;

#[cfg(target_os = "linux")]
pub mod linux;

#[cfg(target_os = "macos")]
pub mod macos;

// Re-export core types for convenience.
pub use keyboard_hook::{
    FocusDetector, Key, KeyEvent, KeyEventType, KeyboardHook, Modifiers, MouseEvent,
    MouseEventType, PlatformError, WindowInfo,
};

// Re-export mock types.
pub use mock::{MockFocusDetector, MockKeyboardHook};

// Re-export platform implementations.
#[cfg(target_os = "linux")]
pub use linux::{
    detect_wayland_status, is_xwayland_available, LinuxFocusDetector, LinuxKeyboardHook,
    WaylandStatus,
};

#[cfg(target_os = "macos")]
pub use macos::{
    check_accessibility_permission, request_accessibility_permission, MacOSFocusDetector,
    MacOSKeyboardHook, PermissionStatus,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_platform_module_loads() {
        // Verify re-exports compile
        let _m = Modifiers::default();
        let _w = WindowInfo::default();
    }

    #[test]
    fn test_mock_types_accessible() {
        let _hook = MockKeyboardHook::new();
        let _det = MockFocusDetector::new();
    }
}
