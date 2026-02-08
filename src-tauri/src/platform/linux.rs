//! Linux (X11/Wayland) keyboard hook and focus detection.
//!
//! Uses the `rdev` crate for system-wide keyboard listening.
//! Focus detection uses `xdotool` and `xprop` commands to query X11 windows.
//!
//! # Wayland Limitations
//!
//! Wayland's security model restricts global keyboard listening.
//! Under pure Wayland (no XWayland), `rdev` may not receive events
//! unless the compositor provides a protocol like
//! `zwp_input_method_v2` or `wlr-input-inhibitor`. Users on Wayland
//! may need to run MuttonText under XWayland or grant special
//! compositor permissions. This is a known limitation shared by all
//! text expanders on Wayland.

#![cfg(target_os = "linux")]

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;

use crate::platform::keyboard_hook::{
    FocusDetector, KeyEvent, KeyEventType, KeyboardHook, Modifiers, PlatformError,
    WindowInfo,
};
use crate::platform::rdev_common::{is_modifier, rdev_key_to_key};

// ---------------------------------------------------------------------------
// LinuxKeyboardHook
// ---------------------------------------------------------------------------

/// Linux keyboard hook backed by `rdev::listen`.
///
/// # Limitation: Cannot be restarted
///
/// Due to rdev's internal implementation, once `stop()` is called, the hook
/// cannot be cleanly restarted. Attempting to start again will return an error.
/// To re-enable the hook after stopping, create a new instance.
pub struct LinuxKeyboardHook {
    running: Arc<AtomicBool>,
    /// Track if hook was ever started (even if later stopped).
    /// rdev::listen cannot be cleanly stopped and restarted.
    started_once: AtomicBool,
}

impl LinuxKeyboardHook {
    pub fn new() -> Self {
        Self {
            running: Arc::new(AtomicBool::new(false)),
            started_once: AtomicBool::new(false),
        }
    }
}

impl Default for LinuxKeyboardHook {
    fn default() -> Self {
        Self::new()
    }
}

impl KeyboardHook for LinuxKeyboardHook {
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
        let callback: Arc<dyn Fn(KeyEvent) + Send + Sync> = Arc::from(callback);

        thread::Builder::new()
            .name("muttontext-keyboard-hook".into())
            .spawn(move || {
                tracing::info!("Linux keyboard hook thread started");
                // rdev::listen blocks until an error occurs.
                if let Err(e) = rdev::listen(move |event| {
                    if !running.load(Ordering::SeqCst) {
                        return;
                    }
                    let (event_type, rdev_key) = match event.event_type {
                        rdev::EventType::KeyPress(k) => (KeyEventType::Press, k),
                        rdev::EventType::KeyRelease(k) => (KeyEventType::Release, k),
                        _ => return, // ignore mouse etc.
                    };
                    if is_modifier(&rdev_key) {
                        return; // don't forward bare modifier presses
                    }
                    let key = rdev_key_to_key(&rdev_key);
                    // NOTE: rdev does not provide modifier state directly;
                    // a full implementation would track it ourselves. For now
                    // we pass empty modifiers — the InputManager does not rely
                    // on modifiers for buffer management.
                    let ke = KeyEvent::new(key, event_type, Modifiers::default());
                    callback(ke);
                }) {
                    tracing::error!("rdev listen error: {:?}", e);
                }
            })
            .map_err(|e| PlatformError::Internal(e.to_string()))?;

        tracing::info!("LinuxKeyboardHook started");
        Ok(())
    }

    fn stop(&mut self) -> Result<(), PlatformError> {
        if !self.running.load(Ordering::SeqCst) {
            return Err(PlatformError::NotRunning);
        }
        self.running.store(false, Ordering::SeqCst);
        // rdev::listen does not provide a clean stop mechanism; setting the
        // flag causes the callback to become a no-op. The thread will exit
        // when the OS delivers the next event or on process shutdown.
        tracing::info!("LinuxKeyboardHook stopped");
        Ok(())
    }

    fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }
}

// ---------------------------------------------------------------------------
// LinuxFocusDetector
// ---------------------------------------------------------------------------

/// Focus detector for Linux using X11 tools.
///
/// Uses `xdotool` and `xprop` commands to query the active window.
/// Falls back to "Unknown" if the tools are not available or the
/// environment is not X11.
pub struct LinuxFocusDetector;

impl LinuxFocusDetector {
    pub fn new() -> Self {
        Self
    }

    /// Get the active window ID using xdotool.
    fn get_active_window_id() -> Option<String> {
        use std::process::Command;

        let output = Command::new("xdotool")
            .args(["getactivewindow"])
            .output()
            .ok()?;

        if output.status.success() {
            let window_id = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !window_id.is_empty() {
                return Some(window_id);
            }
        }
        None
    }

    /// Get window title using xprop with the window ID.
    fn get_window_title(window_id: &str) -> Option<String> {
        use std::process::Command;

        // Validate window_id contains only ASCII digits (no command injection)
        if !window_id.chars().all(|c| c.is_ascii_digit()) {
            tracing::warn!("Invalid window_id format: {}", window_id);
            return None;
        }

        let output = Command::new("/usr/bin/xprop")
            .args(["-id", window_id, "_NET_WM_NAME", "WM_NAME"])
            .output()
            .ok()?;

        if output.status.success() {
            let text = String::from_utf8_lossy(&output.stdout);

            // xprop returns lines like:
            // _NET_WM_NAME(UTF8_STRING) = "Window Title"
            // WM_NAME(STRING) = "Window Title"
            // We prefer _NET_WM_NAME as it supports UTF-8
            for line in text.lines() {
                if line.contains("_NET_WM_NAME") || line.contains("WM_NAME") {
                    if let Some(title) = Self::extract_quoted_value(line) {
                        return Some(title);
                    }
                }
            }
        }
        None
    }

    /// Get window class (app name) using xprop with the window ID.
    fn get_window_class(window_id: &str) -> Option<String> {
        use std::process::Command;

        // Validate window_id contains only ASCII digits (no command injection)
        if !window_id.chars().all(|c| c.is_ascii_digit()) {
            tracing::warn!("Invalid window_id format: {}", window_id);
            return None;
        }

        let output = Command::new("/usr/bin/xprop")
            .args(["-id", window_id, "WM_CLASS"])
            .output()
            .ok()?;

        if output.status.success() {
            let text = String::from_utf8_lossy(&output.stdout);

            // xprop returns: WM_CLASS(STRING) = "instance", "class"
            // We want the class (second value)
            if let Some(class) = Self::extract_window_class(&text) {
                return Some(class);
            }
        }
        None
    }

    /// Get process ID using xprop with the window ID.
    fn get_window_pid(window_id: &str) -> Option<u32> {
        use std::process::Command;

        // Validate window_id contains only ASCII digits (no command injection)
        if !window_id.chars().all(|c| c.is_ascii_digit()) {
            tracing::warn!("Invalid window_id format: {}", window_id);
            return None;
        }

        let output = Command::new("/usr/bin/xprop")
            .args(["-id", window_id, "_NET_WM_PID"])
            .output()
            .ok()?;

        if output.status.success() {
            let text = String::from_utf8_lossy(&output.stdout);

            // xprop returns: _NET_WM_PID(CARDINAL) = 12345
            for part in text.split('=') {
                if let Ok(pid) = part.trim().parse::<u32>() {
                    return Some(pid);
                }
            }
        }
        None
    }

    /// Extract a quoted string value from xprop output.
    /// Example: `_NET_WM_NAME(UTF8_STRING) = "Firefox"` → `"Firefox"`
    fn extract_quoted_value(line: &str) -> Option<String> {
        // Find the part after '='
        let after_eq = line.split('=').nth(1)?;

        // Find content between quotes
        let start = after_eq.find('"')? + 1;
        let end = after_eq[start..].find('"')? + start;

        Some(after_eq[start..end].to_string())
    }

    /// Extract window class from WM_CLASS output.
    /// Example: `WM_CLASS(STRING) = "firefox", "Firefox"` → `"Firefox"`
    fn extract_window_class(text: &str) -> Option<String> {
        // Find the part after '='
        let after_eq = text.split('=').nth(1)?;

        // Split by comma to get both instance and class
        let parts: Vec<&str> = after_eq.split(',').collect();

        // We want the second part (class), or fall back to first (instance)
        let class_part = parts.get(1).or_else(|| parts.first())?;

        // Extract the quoted value
        let trimmed = class_part.trim();
        let start = trimmed.find('"')? + 1;
        let end = trimmed[start..].find('"')? + start;

        Some(trimmed[start..end].to_string())
    }
}

impl Default for LinuxFocusDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl FocusDetector for LinuxFocusDetector {
    fn get_active_window_info(&self) -> Result<WindowInfo, PlatformError> {
        // Try to get window information via X11 tools
        if let Some(window_id) = Self::get_active_window_id() {
            let title = Self::get_window_title(&window_id)
                .unwrap_or_else(|| "Unknown".to_string());
            let app_name = Self::get_window_class(&window_id)
                .unwrap_or_else(|| "Unknown".to_string());
            let process_id = Self::get_window_pid(&window_id);

            return Ok(WindowInfo {
                title,
                app_name,
                process_id,
            });
        }

        // Fall back to default if X11 tools are not available
        tracing::debug!("Failed to get active window info; xdotool/xprop may not be installed or X11 not available");
        Ok(WindowInfo::default())
    }
}

// ---------------------------------------------------------------------------
// Wayland Detection & Status
// ---------------------------------------------------------------------------

/// Represents the current Wayland session status.
///
/// This enum helps determine what input method approach is available:
/// - **NotAvailable**: Not running under Wayland at all (pure X11)
/// - **XWaylandFallback**: Running under Wayland but XWayland is available;
///   rdev can use X11 API via XWayland compatibility layer
/// - **NativePortal**: Running under native Wayland; would need Portal API
///   (ashpd) for global input (not yet implemented)
/// - **Unknown**: Cannot determine session type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WaylandStatus {
    /// Not running under Wayland (pure X11 session).
    NotAvailable,
    /// Running under Wayland but XWayland is available as fallback.
    XWaylandFallback,
    /// Running under native Wayland (Portal API would be required).
    NativePortal,
    /// Session type could not be determined.
    Unknown,
}

/// Detects the current Wayland session status.
///
/// Checks environment variables to determine if we're running under Wayland
/// and whether XWayland is available as a fallback.
///
/// # Detection Logic
///
/// 1. Check `$XDG_SESSION_TYPE`: if "wayland", we're under Wayland
/// 2. Check `$WAYLAND_DISPLAY`: if set, confirms Wayland is running
/// 3. Check `$DISPLAY`: if set while Wayland is active, XWayland is available
///
/// # Returns
///
/// - `NotAvailable`: Neither Wayland nor XWayland detected (pure X11)
/// - `XWaylandFallback`: Wayland is running AND XWayland is available
/// - `NativePortal`: Wayland is running but no XWayland (needs Portal API)
/// - `Unknown`: Cannot determine session type
///
/// # Compositor-Specific Notes
///
/// ## GNOME (Mutter)
/// - XWayland enabled by default
/// - Global input requires Portal API or GNOME Shell extensions
/// - rdev works via XWayland fallback
///
/// ## KDE Plasma (KWin)
/// - XWayland enabled by default
/// - Global input requires Portal API or KWin scripts
/// - rdev works via XWayland fallback
///
/// ## Sway (wlroots-based)
/// - XWayland optional but usually enabled
/// - Global input requires wlr-protocols or Portal API
/// - rdev works via XWayland if enabled
///
/// ## Future Work: Native Wayland Support
///
/// For pure Wayland compositors without XWayland, we would need:
/// - Portal API via `ashpd` crate for global shortcuts
/// - Text input protocol (`zwp_input_method_v2`) for snippet insertion
/// - This is deferred to a future milestone
pub fn detect_wayland_status() -> WaylandStatus {
    use std::env;

    // Check if we're in a Wayland session
    let session_type = env::var("XDG_SESSION_TYPE").unwrap_or_default();
    let wayland_display = env::var("WAYLAND_DISPLAY").ok();

    let is_wayland = session_type == "wayland" || wayland_display.is_some();

    if !is_wayland {
        // Pure X11 or unknown session
        return WaylandStatus::NotAvailable;
    }

    // We're under Wayland - check if XWayland is available
    if is_xwayland_available() {
        WaylandStatus::XWaylandFallback
    } else {
        WaylandStatus::NativePortal
    }
}

/// Checks if XWayland is available as a fallback.
///
/// XWayland allows X11 applications to run under Wayland compositors,
/// which means `rdev` can hook into keyboard events via the X11 API.
///
/// # Detection
///
/// Checks for the `$DISPLAY` environment variable, which is set by
/// XWayland when it's running. If present alongside Wayland indicators,
/// we can use X11-based input monitoring.
///
/// # Returns
///
/// `true` if `$DISPLAY` is set (XWayland available), `false` otherwise.
pub fn is_xwayland_available() -> bool {
    std::env::var("DISPLAY").is_ok()
}

// ---------------------------------------------------------------------------
// Wayland Keyboard Hook (Future Work)
// ---------------------------------------------------------------------------

/// Placeholder for a future native Wayland keyboard hook.
///
/// # Wayland Security Model Limitations
///
/// Wayland's security model restricts global input listening for privacy
/// and security reasons. Unlike X11, applications cannot arbitrarily
/// capture keyboard events system-wide.
///
/// ## Current Status (v1.0)
///
/// MuttonText works under Wayland via **XWayland fallback**:
/// - Most compositors (GNOME, KDE, Sway) ship with XWayland enabled
/// - `rdev` hooks into the X11 compatibility layer
/// - This provides full keyboard monitoring functionality
///
/// ## Future: Native Wayland Support
///
/// For pure Wayland (no XWayland), we would need:
///
/// ### Option 1: Portal API (Recommended)
/// - Use `org.freedesktop.portal.GlobalShortcuts` for trigger shortcuts
/// - Use `org.freedesktop.portal.InputCapture` (if available) for monitoring
/// - Implemented via `ashpd` crate
/// - Requires user permission grant via desktop environment
///
/// ### Option 2: Input Method Protocol
/// - Use `zwp_input_method_v2` Wayland protocol
/// - Requires compositor support (not universal)
/// - More complex implementation
///
/// ### Option 3: Compositor-Specific Extensions
/// - GNOME: Shell extensions with custom D-Bus API
/// - KDE: KWin scripts
/// - Sway: wlr-input-inhibitor protocol
/// - Not portable across compositors
///
/// ## Testing on Wayland
///
/// To test Wayland support, check session type:
/// ```bash
/// echo $XDG_SESSION_TYPE  # Should output "wayland"
/// echo $WAYLAND_DISPLAY   # Should be set (e.g., "wayland-0")
/// echo $DISPLAY           # If set, XWayland is available
/// ```
///
/// If `$DISPLAY` is not set, you're on pure Wayland and MuttonText
/// will need the Portal API implementation (future work).
///
/// ## Compositor Compatibility
///
/// | Compositor | XWayland Default | Native Portal Support |
/// |------------|------------------|------------------------|
/// | GNOME      | ✅ Yes           | ⏳ Partial (v43+)      |
/// | KDE Plasma | ✅ Yes           | ⏳ Partial (v5.27+)    |
/// | Sway       | ✅ Yes (optional)| ❌ Limited             |
/// | Hyprland   | ✅ Yes           | ❌ Limited             |
/// | Cosmic     | ⏳ TBD           | ⏳ TBD                 |
///
/// **Recommendation for users:** Ensure XWayland is enabled in your compositor
/// settings if MuttonText doesn't work out of the box.
pub struct WaylandKeyboardHook;

impl WaylandKeyboardHook {
    #[allow(dead_code)]
    pub fn new() -> Result<Self, PlatformError> {
        Err(PlatformError::NotSupported(
            "Wayland global keyboard hooks require compositor-specific protocols \
             (e.g. zwp_input_method_v2) or Portal API. Use XWayland or the X11 backend."
                .into(),
        ))
    }
}

// ---------------------------------------------------------------------------
// Tests (only compiled on Linux)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linux_focus_detector_fallback() {
        // This test verifies that the focus detector returns gracefully
        // even if xdotool/xprop are not installed (returns default)
        let det = LinuxFocusDetector::new();
        let result = det.get_active_window_info();
        assert!(result.is_ok());
        // Can't assert exact content since it depends on environment
    }

    #[test]
    fn test_extract_quoted_value() {
        let line = r#"_NET_WM_NAME(UTF8_STRING) = "Firefox""#;
        assert_eq!(
            LinuxFocusDetector::extract_quoted_value(line),
            Some("Firefox".to_string())
        );

        let line = r#"WM_NAME(STRING) = "Terminal - bash""#;
        assert_eq!(
            LinuxFocusDetector::extract_quoted_value(line),
            Some("Terminal - bash".to_string())
        );

        let line = "invalid line";
        assert_eq!(LinuxFocusDetector::extract_quoted_value(line), None);
    }

    #[test]
    fn test_extract_window_class() {
        let text = r#"WM_CLASS(STRING) = "firefox", "Firefox""#;
        assert_eq!(
            LinuxFocusDetector::extract_window_class(text),
            Some("Firefox".to_string())
        );

        let text = r#"WM_CLASS(STRING) = "gnome-terminal-server", "Gnome-terminal""#;
        assert_eq!(
            LinuxFocusDetector::extract_window_class(text),
            Some("Gnome-terminal".to_string())
        );

        // Single class value (fallback to instance)
        let text = r#"WM_CLASS(STRING) = "code""#;
        assert_eq!(
            LinuxFocusDetector::extract_window_class(text),
            Some("code".to_string())
        );

        let text = "invalid";
        assert_eq!(LinuxFocusDetector::extract_window_class(text), None);
    }

    #[test]
    fn test_wayland_hook_not_supported() {
        let result = WaylandKeyboardHook::new();
        assert!(result.is_err());
    }

    #[test]
    fn test_hook_cannot_restart_after_stop() {
        let mut hook = LinuxKeyboardHook::new();
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

    // ---------------------------------------------------------------------------
    // Wayland Detection Tests
    // ---------------------------------------------------------------------------

    #[test]
    fn test_wayland_status_not_available_on_x11() {
        // Simulate pure X11 environment (no Wayland indicators)
        use std::env;

        // Save original values
        let orig_session_type = env::var("XDG_SESSION_TYPE").ok();
        let orig_wayland_display = env::var("WAYLAND_DISPLAY").ok();

        // Set X11 session
        env::remove_var("XDG_SESSION_TYPE");
        env::remove_var("WAYLAND_DISPLAY");
        env::set_var("DISPLAY", ":0");

        let status = detect_wayland_status();
        assert_eq!(status, WaylandStatus::NotAvailable);

        // Restore original values
        if let Some(val) = orig_session_type {
            env::set_var("XDG_SESSION_TYPE", val);
        } else {
            env::remove_var("XDG_SESSION_TYPE");
        }
        if let Some(val) = orig_wayland_display {
            env::set_var("WAYLAND_DISPLAY", val);
        } else {
            env::remove_var("WAYLAND_DISPLAY");
        }
    }

    #[test]
    fn test_wayland_status_xwayland_fallback() {
        use std::env;

        // Save original values
        let orig_session_type = env::var("XDG_SESSION_TYPE").ok();
        let orig_wayland_display = env::var("WAYLAND_DISPLAY").ok();
        let orig_display = env::var("DISPLAY").ok();

        // Simulate Wayland with XWayland
        env::set_var("XDG_SESSION_TYPE", "wayland");
        env::set_var("WAYLAND_DISPLAY", "wayland-0");
        env::set_var("DISPLAY", ":0");

        let status = detect_wayland_status();
        assert_eq!(status, WaylandStatus::XWaylandFallback);

        // Restore original values
        if let Some(val) = orig_session_type {
            env::set_var("XDG_SESSION_TYPE", val);
        } else {
            env::remove_var("XDG_SESSION_TYPE");
        }
        if let Some(val) = orig_wayland_display {
            env::set_var("WAYLAND_DISPLAY", val);
        } else {
            env::remove_var("WAYLAND_DISPLAY");
        }
        if let Some(val) = orig_display {
            env::set_var("DISPLAY", val);
        } else {
            env::remove_var("DISPLAY");
        }
    }

    #[test]
    fn test_wayland_status_native_portal() {
        use std::env;

        // Save original values
        let orig_session_type = env::var("XDG_SESSION_TYPE").ok();
        let orig_wayland_display = env::var("WAYLAND_DISPLAY").ok();
        let orig_display = env::var("DISPLAY").ok();

        // Simulate pure Wayland without XWayland
        env::set_var("XDG_SESSION_TYPE", "wayland");
        env::set_var("WAYLAND_DISPLAY", "wayland-0");
        env::remove_var("DISPLAY");

        let status = detect_wayland_status();
        assert_eq!(status, WaylandStatus::NativePortal);

        // Restore original values
        if let Some(val) = orig_session_type {
            env::set_var("XDG_SESSION_TYPE", val);
        } else {
            env::remove_var("XDG_SESSION_TYPE");
        }
        if let Some(val) = orig_wayland_display {
            env::set_var("WAYLAND_DISPLAY", val);
        } else {
            env::remove_var("WAYLAND_DISPLAY");
        }
        if let Some(val) = orig_display {
            env::set_var("DISPLAY", val);
        } else {
            env::remove_var("DISPLAY");
        }
    }

    #[test]
    fn test_wayland_status_via_wayland_display_var() {
        use std::env;

        // Save original values
        let orig_session_type = env::var("XDG_SESSION_TYPE").ok();
        let orig_wayland_display = env::var("WAYLAND_DISPLAY").ok();
        let orig_display = env::var("DISPLAY").ok();

        // Simulate detection via WAYLAND_DISPLAY only (no XDG_SESSION_TYPE)
        env::remove_var("XDG_SESSION_TYPE");
        env::set_var("WAYLAND_DISPLAY", "wayland-1");
        env::set_var("DISPLAY", ":1");

        let status = detect_wayland_status();
        assert_eq!(status, WaylandStatus::XWaylandFallback);

        // Restore original values
        if let Some(val) = orig_session_type {
            env::set_var("XDG_SESSION_TYPE", val);
        } else {
            env::remove_var("XDG_SESSION_TYPE");
        }
        if let Some(val) = orig_wayland_display {
            env::set_var("WAYLAND_DISPLAY", val);
        } else {
            env::remove_var("WAYLAND_DISPLAY");
        }
        if let Some(val) = orig_display {
            env::set_var("DISPLAY", val);
        } else {
            env::remove_var("DISPLAY");
        }
    }

    #[test]
    fn test_is_xwayland_available_when_display_set() {
        use std::env;

        // Save original value
        let orig_display = env::var("DISPLAY").ok();

        env::set_var("DISPLAY", ":0");
        assert!(is_xwayland_available());

        // Restore original value
        if let Some(val) = orig_display {
            env::set_var("DISPLAY", val);
        } else {
            env::remove_var("DISPLAY");
        }
    }

    #[test]
    fn test_is_xwayland_available_when_display_not_set() {
        use std::env;

        // Save original value
        let orig_display = env::var("DISPLAY").ok();

        env::remove_var("DISPLAY");
        assert!(!is_xwayland_available());

        // Restore original value
        if let Some(val) = orig_display {
            env::set_var("DISPLAY", val);
        } else {
            env::remove_var("DISPLAY");
        }
    }
}
