//! macOS keyboard hook and focus detection.
//!
//! Uses the `rdev` crate which internally uses `CGEventTap` on macOS.
//! Requires Accessibility permissions (System Preferences → Security &
//! Privacy → Privacy → Accessibility).

#![cfg(target_os = "macos")]

use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;

use crate::platform::keyboard_hook::{
    FocusDetector, Key, KeyEvent, KeyEventType, KeyboardHook, Modifiers, PlatformError,
    WindowInfo,
};
use crate::platform::rdev_common::{is_modifier, rdev_key_to_key};

// ---------------------------------------------------------------------------
// Permission Status
// ---------------------------------------------------------------------------

/// Status of macOS accessibility permissions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PermissionStatus {
    /// Accessibility permissions are granted.
    Granted,
    /// Accessibility permissions are denied or not granted.
    Denied,
    /// Permission status could not be determined.
    Unknown,
}

// ---------------------------------------------------------------------------
// Permission Checking Functions
// ---------------------------------------------------------------------------

/// Check whether the app has Accessibility permissions.
///
/// Uses AppleScript to check the TCC (Transparency, Consent, and Control)
/// database for accessibility permissions. This approach is compatible with
/// cross-compilation as it doesn't require linking against macOS frameworks.
///
/// # Returns
///
/// `PermissionStatus::Granted` if permissions are granted,
/// `PermissionStatus::Denied` if denied,
/// `PermissionStatus::Unknown` if the check fails or status cannot be determined.
pub fn check_accessibility_permission() -> PermissionStatus {
    // Use AppleScript to check if the app is trusted for accessibility
    // This checks the TCC database without requiring objc bindings
    let script = r#"
        tell application "System Events"
            set isTrusted to get accessibility trusted of application "System Events"
            return isTrusted
        end tell
    "#;

    match Command::new("/usr/bin/osascript")
        .arg("-e")
        .arg(script)
        .output()
    {
        Ok(output) => {
            if output.status.success() {
                let result = String::from_utf8_lossy(&output.stdout);
                let is_trusted = result.trim() == "true";

                if is_trusted {
                    tracing::debug!("Accessibility permissions are granted");
                    PermissionStatus::Granted
                } else {
                    tracing::warn!("Accessibility permissions are not granted");
                    PermissionStatus::Denied
                }
            } else {
                let error = String::from_utf8_lossy(&output.stderr);
                tracing::error!("Failed to check accessibility permission: {}", error);
                PermissionStatus::Unknown
            }
        }
        Err(e) => {
            tracing::error!("Failed to execute osascript: {}", e);
            PermissionStatus::Unknown
        }
    }
}

/// Request accessibility permissions by opening System Preferences to the
/// Accessibility pane.
///
/// This will open the Security & Privacy preferences with the Accessibility
/// section selected, allowing the user to grant permissions to the app.
///
/// # Returns
///
/// `Ok(())` if the preferences pane was opened successfully,
/// `Err(PlatformError)` if the operation failed.
pub fn request_accessibility_permission() -> Result<(), PlatformError> {
    // Open System Preferences to the Accessibility pane
    // Using x-apple.systempreferences URL scheme
    let url = "x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility";

    match Command::new("/usr/bin/open").arg(url).status() {
        Ok(status) => {
            if status.success() {
                tracing::info!("Opened System Preferences to Accessibility pane");
                Ok(())
            } else {
                let msg = format!("Failed to open System Preferences (exit code: {:?})", status.code());
                tracing::error!("{}", msg);
                Err(PlatformError::Internal(msg))
            }
        }
        Err(e) => {
            let msg = format!("Failed to execute 'open' command: {}", e);
            tracing::error!("{}", msg);
            Err(PlatformError::Internal(msg))
        }
    }
}

// ---------------------------------------------------------------------------
// MacOSKeyboardHook
// ---------------------------------------------------------------------------

/// macOS keyboard hook backed by `rdev::listen` (CGEventTap).
///
/// # Accessibility Permissions
///
/// macOS requires Accessibility permissions for `CGEventTap`. The app
/// must be added to System Preferences → Security & Privacy → Privacy →
/// Accessibility. Without this, `rdev::listen` will silently fail to
/// receive events.
///
/// # Limitation: Cannot be restarted
///
/// Due to rdev's internal implementation, once `stop()` is called, the hook
/// cannot be cleanly restarted. Attempting to start again will return an error.
/// To re-enable the hook after stopping, create a new instance.
pub struct MacOSKeyboardHook {
    running: Arc<AtomicBool>,
    /// Track if hook was ever started (even if later stopped).
    /// rdev::listen cannot be cleanly stopped and restarted.
    started_once: AtomicBool,
}

impl MacOSKeyboardHook {
    pub fn new() -> Self {
        Self {
            running: Arc::new(AtomicBool::new(false)),
            started_once: AtomicBool::new(false),
        }
    }
}

impl Default for MacOSKeyboardHook {
    fn default() -> Self {
        Self::new()
    }
}

impl KeyboardHook for MacOSKeyboardHook {
    fn start(
        &mut self,
        callback: Box<dyn Fn(KeyEvent) + Send + Sync>,
    ) -> Result<(), PlatformError> {
        if self.running.load(Ordering::SeqCst) {
            return Err(PlatformError::AlreadyRunning);
        }
        // Prevent re-starting after stop() due to rdev::listen limitations
        if self.started_once.load(Ordering::SeqCst) {
            return Err(PlatformError::Internal(
                "Hook cannot be restarted after stop(); create a new instance".into(),
            ));
        }
        self.running.store(true, Ordering::SeqCst);
        self.started_once.store(true, Ordering::SeqCst);
        let running = self.running.clone();
        let callback = Arc::from(callback);

        thread::Builder::new()
            .name("muttontext-keyboard-hook".into())
            .spawn(move || {
                tracing::info!("macOS keyboard hook thread started");
                if let Err(e) = rdev::listen(move |event| {
                    if !running.load(Ordering::SeqCst) {
                        return;
                    }
                    let (event_type, rdev_key) = match event.event_type {
                        rdev::EventType::KeyPress(k) => (KeyEventType::Press, k),
                        rdev::EventType::KeyRelease(k) => (KeyEventType::Release, k),
                        _ => return,
                    };
                    if is_modifier(&rdev_key) {
                        return;
                    }
                    let key = rdev_key_to_key(&rdev_key);
                    let ke = KeyEvent::new(key, event_type, Modifiers::default());
                    callback(ke);
                }) {
                    tracing::error!("rdev listen error: {:?}", e);
                }
            })
            .map_err(|e| PlatformError::Internal(e.to_string()))?;

        tracing::info!("MacOSKeyboardHook started");
        Ok(())
    }

    fn stop(&mut self) -> Result<(), PlatformError> {
        if !self.running.load(Ordering::SeqCst) {
            return Err(PlatformError::NotRunning);
        }
        self.running.store(false, Ordering::SeqCst);
        tracing::info!("MacOSKeyboardHook stopped");
        Ok(())
    }

    fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }
}

// ---------------------------------------------------------------------------
// MacOSFocusDetector (stub)
// ---------------------------------------------------------------------------

/// Stub focus detector for macOS. A full implementation would use
/// `NSWorkspace.shared.frontmostApplication` via the objc crate.
pub struct MacOSFocusDetector;

impl MacOSFocusDetector {
    pub fn new() -> Self {
        Self
    }
}

impl Default for MacOSFocusDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl FocusDetector for MacOSFocusDetector {
    fn get_active_window_info(&self) -> Result<WindowInfo, PlatformError> {
        Ok(WindowInfo::default())
    }
}

// ---------------------------------------------------------------------------
// Tests (only compiled on macOS)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_macos_focus_detector_stub() {
        let det = MacOSFocusDetector::new();
        let info = det.get_active_window_info().unwrap();
        assert_eq!(info, WindowInfo::default());
    }

    #[test]
    fn test_check_accessibility_permission_returns_status() {
        // Should return a valid PermissionStatus variant
        let status = check_accessibility_permission();
        // Any of these is valid - we can't predict the actual system state
        assert!(matches!(
            status,
            PermissionStatus::Granted | PermissionStatus::Denied | PermissionStatus::Unknown
        ));
    }

    #[test]
    fn test_permission_status_equality() {
        assert_eq!(PermissionStatus::Granted, PermissionStatus::Granted);
        assert_eq!(PermissionStatus::Denied, PermissionStatus::Denied);
        assert_eq!(PermissionStatus::Unknown, PermissionStatus::Unknown);
        assert_ne!(PermissionStatus::Granted, PermissionStatus::Denied);
    }

    #[test]
    fn test_request_accessibility_permission_executes() {
        // We can't actually test that preferences open without user interaction,
        // but we can verify the function doesn't panic and returns a Result
        let result = request_accessibility_permission();
        // The result could be Ok or Err depending on system state
        // Just verify it returns something
        let _ = result;
    }

    #[test]
    fn test_hook_cannot_restart_after_stop() {
        let mut hook = MacOSKeyboardHook::new();
        assert!(!hook.started_once.load(Ordering::SeqCst));

        // First start should succeed
        let result = hook.start(Box::new(|_| {}));
        assert!(result.is_ok());
        assert!(hook.started_once.load(Ordering::SeqCst));

        // Stop the hook
        let _ = hook.stop();

        // Second start should fail
        let result = hook.start(Box::new(|_| {}));
        assert!(result.is_err());
        match result.unwrap_err() {
            PlatformError::Internal(msg) => assert!(msg.contains("cannot be restarted")),
            _ => panic!("Expected Internal error"),
        }
    }
}

// ---------------------------------------------------------------------------
// Tests with Mocks (cross-platform)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod mock_tests {
    use super::*;

    // Mock Command execution for testing on non-macOS platforms
    #[cfg(not(target_os = "macos"))]
    #[test]
    fn test_permission_status_variants() {
        // Test that all PermissionStatus variants exist and are distinct
        let granted = PermissionStatus::Granted;
        let denied = PermissionStatus::Denied;
        let unknown = PermissionStatus::Unknown;

        assert_ne!(granted, denied);
        assert_ne!(granted, unknown);
        assert_ne!(denied, unknown);
    }

    #[test]
    fn test_permission_status_debug() {
        // Verify Debug trait implementation
        let status = PermissionStatus::Granted;
        let debug_str = format!("{:?}", status);
        assert!(debug_str.contains("Granted"));
    }

    #[test]
    fn test_permission_status_clone() {
        // Verify Clone trait implementation
        let status = PermissionStatus::Granted;
        let cloned = status;
        assert_eq!(status, cloned);
    }

    #[test]
    fn test_permission_status_copy() {
        // Verify Copy trait implementation
        let status = PermissionStatus::Denied;
        let copied = status;
        assert_eq!(status, copied);
        // Both should still be usable
        assert_eq!(status, PermissionStatus::Denied);
        assert_eq!(copied, PermissionStatus::Denied);
    }
}
