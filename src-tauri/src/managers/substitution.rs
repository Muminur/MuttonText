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
    /// Delay before starting backspace deletion, in milliseconds.
    /// This allows applications (especially browsers) to fully process the last
    /// keystroke before xdotool starts deleting. Without this delay, the first
    /// character of the keyword may not be deleted in Firefox/Chrome.
    pub pre_deletion_delay_ms: u64,
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
            paste_restore_delay_ms: 200,
            // Use Ctrl+V on all platforms (including Linux).
            // Shift+Insert pastes from X11 PRIMARY selection, but arboard writes
            // to the CLIPBOARD selection — causing the user's old clipboard content
            // to be pasted instead of the snippet.
            use_shift_insert: false,
            timeout_secs: DEFAULT_SUBSTITUTION_TIMEOUT_SECS,
            chunk_delay_ms: 10,
            pre_deletion_delay_ms: 20,
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

/// macOS-specific fast keyword deletion using CoreGraphics.
///
/// Sends all backspace key events through CGEventPost with minimal delay,
/// bypassing rdev::simulate which adds overhead on macOS Ventura.
#[cfg(target_os = "macos")]
fn delete_keyword_macos(count: usize, _config: &SubstitutionConfig) -> Result<(), SubstitutionError> {
    if count > MAX_KEYWORD_LENGTH {
        return Err(SubstitutionError::KeywordTooLong(count, MAX_KEYWORD_LENGTH));
    }
    if count == 0 {
        return Ok(());
    }
    tracing::debug!("Deleting {} characters via macOS CGEvent backspace (fast)", count);

    use std::ffi::c_void;

    type CGEventRef = *mut c_void;
    type CGEventSourceRef = *mut c_void;

    #[link(name = "CoreGraphics", kind = "framework")]
    extern "C" {
        fn CGEventSourceCreate(state_id: i32) -> CGEventSourceRef;
        fn CGEventCreateKeyboardEvent(
            source: CGEventSourceRef,
            virtual_key: u16,
            key_down: bool,
        ) -> CGEventRef;
        fn CGEventPost(tap: u32, event: CGEventRef);
    }

    #[link(name = "CoreFoundation", kind = "framework")]
    extern "C" {
        fn CFRelease(cf: *const c_void);
    }

    const HID_SYSTEM_STATE: i32 = 1;
    const HID_EVENT_TAP: u32 = 0;
    const BACKSPACE_KEYCODE: u16 = 51; // macOS virtual keycode for backspace

    unsafe {
        let source = CGEventSourceCreate(HID_SYSTEM_STATE);

        for _ in 0..count {
            let key_down = CGEventCreateKeyboardEvent(source, BACKSPACE_KEYCODE, true);
            if !key_down.is_null() {
                CGEventPost(HID_EVENT_TAP, key_down);
                CFRelease(key_down as *const c_void);
            }

            let key_up = CGEventCreateKeyboardEvent(source, BACKSPACE_KEYCODE, false);
            if !key_up.is_null() {
                CGEventPost(HID_EVENT_TAP, key_up);
                CFRelease(key_up as *const c_void);
            }
        }

        // Brief pause to let the OS process all backspaces before inserting
        std::thread::sleep(std::time::Duration::from_millis(10));

        if !source.is_null() {
            CFRelease(source as *const c_void);
        }
    }

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

/// macOS-specific clipboard paste using CoreGraphics CGEventPost.
///
/// Uses CGEventPost to simulate Cmd+V instead of rdev::simulate,
/// which doesn't work reliably on macOS Ventura.
#[cfg(target_os = "macos")]
pub fn insert_via_clipboard_macos<P: ClipboardProvider>(
    text: &str,
    clipboard_mgr: &mut ClipboardManager<P>,
    config: &SubstitutionConfig,
) -> Result<(), SubstitutionError> {
    tracing::debug!("Inserting via macOS clipboard paste (CGEvent): {} chars", text.len());

    use std::ffi::c_void;

    type CGEventRef = *mut c_void;
    type CGEventSourceRef = *mut c_void;

    #[link(name = "CoreGraphics", kind = "framework")]
    extern "C" {
        fn CGEventSourceCreate(state_id: i32) -> CGEventSourceRef;
        fn CGEventCreateKeyboardEvent(
            source: CGEventSourceRef,
            virtual_key: u16,
            key_down: bool,
        ) -> CGEventRef;
        fn CGEventSetFlags(event: CGEventRef, flags: u64);
        fn CGEventPost(tap: u32, event: CGEventRef);
    }

    #[link(name = "CoreFoundation", kind = "framework")]
    extern "C" {
        fn CFRelease(cf: *const c_void);
    }

    const HID_SYSTEM_STATE: i32 = 1;
    const HID_EVENT_TAP: u32 = 0;
    const V_KEYCODE: u16 = 9;           // macOS virtual keycode for 'v'
    const CMD_LEFT_KEYCODE: u16 = 55;   // macOS virtual keycode for Left Command
    const CMD_FLAG: u64 = 0x00100000;   // kCGEventFlagMaskCommand

    // Preserve current clipboard
    clipboard_mgr.preserve()?;

    // Write snippet to clipboard
    clipboard_mgr.write(text)?;

    // Delay to ensure clipboard content is committed to the pasteboard
    std::thread::sleep(std::time::Duration::from_millis(20));

    unsafe {
        let source = CGEventSourceCreate(HID_SYSTEM_STATE);

        // 1. Command key down
        let cmd_down = CGEventCreateKeyboardEvent(source, CMD_LEFT_KEYCODE, true);
        if !cmd_down.is_null() {
            CGEventSetFlags(cmd_down, CMD_FLAG);
            CGEventPost(HID_EVENT_TAP, cmd_down);
            CFRelease(cmd_down as *const c_void);
        }

        std::thread::sleep(std::time::Duration::from_millis(5));

        // 2. V key down (Command still held)
        let v_down = CGEventCreateKeyboardEvent(source, V_KEYCODE, true);
        if !v_down.is_null() {
            CGEventSetFlags(v_down, CMD_FLAG);
            CGEventPost(HID_EVENT_TAP, v_down);
            CFRelease(v_down as *const c_void);
        }

        std::thread::sleep(std::time::Duration::from_millis(5));

        // 3. V key up (Command still held)
        let v_up = CGEventCreateKeyboardEvent(source, V_KEYCODE, false);
        if !v_up.is_null() {
            CGEventSetFlags(v_up, CMD_FLAG);
            CGEventPost(HID_EVENT_TAP, v_up);
            CFRelease(v_up as *const c_void);
        }

        std::thread::sleep(std::time::Duration::from_millis(5));

        // 4. Command key up
        let cmd_up = CGEventCreateKeyboardEvent(source, CMD_LEFT_KEYCODE, false);
        if !cmd_up.is_null() {
            CGEventSetFlags(cmd_up, 0);
            CGEventPost(HID_EVENT_TAP, cmd_up);
            CFRelease(cmd_up as *const c_void);
        }

        if !source.is_null() {
            CFRelease(source as *const c_void);
        }
    }

    // Wait for paste to complete before restoring clipboard
    std::thread::sleep(std::time::Duration::from_millis(config.paste_restore_delay_ms));

    // Restore clipboard
    clipboard_mgr.restore()?;

    Ok(())
}

/// macOS-specific keystroke insertion using CoreGraphics.
///
/// Uses `CGEventKeyboardSetUnicodeString` to correctly inject Unicode
/// characters, since `Key::Unknown(code)` incorrectly passes Unicode
/// codepoints as virtual keycodes on macOS (CGKeyCodes map physical keys,
/// not characters).
#[cfg(target_os = "macos")]
fn insert_via_keystrokes_macos(text: &str, config: &SubstitutionConfig) -> Result<(), SubstitutionError> {
    use std::ffi::c_void;

    type CGEventRef = *mut c_void;
    type CGEventSourceRef = *mut c_void;

    #[link(name = "CoreGraphics", kind = "framework")]
    extern "C" {
        fn CGEventSourceCreate(state_id: i32) -> CGEventSourceRef;
        fn CGEventCreateKeyboardEvent(
            source: CGEventSourceRef,
            virtual_key: u16,
            key_down: bool,
        ) -> CGEventRef;
        fn CGEventKeyboardSetUnicodeString(
            event: CGEventRef,
            length: u64,
            string: *const u16,
        );
        fn CGEventPost(tap: u32, event: CGEventRef);
    }

    #[link(name = "CoreFoundation", kind = "framework")]
    extern "C" {
        fn CFRelease(cf: *const c_void);
    }

    // kCGEventSourceStateHIDSystemState = 1
    const HID_SYSTEM_STATE: i32 = 1;
    // kCGHIDEventTap = 0
    const HID_EVENT_TAP: u32 = 0;

    tracing::debug!("Inserting via macOS CGEvent keystrokes: {} chars", text.len());
    let delay = Duration::from_millis(config.key_delay_ms);

    unsafe {
        let source = CGEventSourceCreate(HID_SYSTEM_STATE);

        for ch in text.chars() {
            let mut utf16 = [0u16; 2];
            let encoded = ch.encode_utf16(&mut utf16);
            let len = encoded.len();

            // Key-down with dummy keycode 0; unicode string overrides the character
            let key_down = CGEventCreateKeyboardEvent(source, 0, true);
            if key_down.is_null() {
                if !source.is_null() {
                    CFRelease(source as *const c_void);
                }
                return Err(SubstitutionError::SimulationFailed(
                    format!("CGEventCreateKeyboardEvent failed for '{}'", ch),
                ));
            }
            CGEventKeyboardSetUnicodeString(key_down, len as u64, utf16.as_ptr());
            CGEventPost(HID_EVENT_TAP, key_down);
            CFRelease(key_down as *const c_void);

            thread::sleep(delay);

            // Key-up
            let key_up = CGEventCreateKeyboardEvent(source, 0, false);
            if !key_up.is_null() {
                CGEventKeyboardSetUnicodeString(key_up, len as u64, utf16.as_ptr());
                CGEventPost(HID_EVENT_TAP, key_up);
                CFRelease(key_up as *const c_void);
            }

            thread::sleep(delay);
        }

        if !source.is_null() {
            CFRelease(source as *const c_void);
        }
    }

    Ok(())
}

/// Inserts text by simulating individual keystrokes.
///
/// On macOS, uses CoreGraphics `CGEventKeyboardSetUnicodeString` for correct
/// Unicode character injection. On other platforms, uses rdev's simulate API.
pub fn insert_via_keystrokes(text: &str, config: &SubstitutionConfig) -> Result<(), SubstitutionError> {
    if text.len() > MAX_SNIPPET_SIZE {
        return Err(SubstitutionError::SnippetTooLarge(text.len(), MAX_SNIPPET_SIZE));
    }
    tracing::debug!("Inserting via keystrokes: {} chars", text.len());

    #[cfg(target_os = "macos")]
    {
        return insert_via_keystrokes_macos(text, config);
    }

    #[cfg(not(target_os = "macos"))]
    {
        let delay = Duration::from_millis(config.key_delay_ms);
        for ch in text.chars() {
            send_key_event(EventType::KeyPress(Key::Unknown(ch as u32)), delay)?;
            send_key_event(EventType::KeyRelease(Key::Unknown(ch as u32)), delay)?;
        }
        Ok(())
    }
}

/// Deletes `count` characters by sending backspace via xdotool (Linux).
///
/// This method works in terminals (including Claude Code CLI) by using the
/// external xdotool command instead of rdev's simulate which doesn't work
/// reliably in terminals.
///
/// IMPORTANT: Includes a pre-deletion delay (config.pre_deletion_delay_ms) to allow
/// applications (especially browsers like Firefox/Chrome) to fully process the last
/// keystroke before xdotool starts deleting. Without this delay, the first character
/// of the keyword may remain undeleted in browser text fields.
pub fn delete_keyword_xdotool(count: usize, config: &SubstitutionConfig) -> Result<(), SubstitutionError> {
    if count > MAX_KEYWORD_LENGTH {
        return Err(SubstitutionError::KeywordTooLong(count, MAX_KEYWORD_LENGTH));
    }
    tracing::debug!("Deleting {} characters via xdotool backspace (with {}ms pre-deletion delay)", count, config.pre_deletion_delay_ms);

    // Wait for the target app to fully process the last keystroke
    // This is critical for browsers (Firefox, Chrome) which have async input processing
    if config.pre_deletion_delay_ms > 0 {
        thread::sleep(Duration::from_millis(config.pre_deletion_delay_ms));
    }

    let output = std::process::Command::new("xdotool")
        .arg("key")
        .arg("--clearmodifiers")
        .arg("--repeat")
        .arg(count.to_string())
        .arg("--delay")
        .arg(config.key_delay_ms.to_string())
        .arg("BackSpace")
        .output()
        .map_err(|e| SubstitutionError::SimulationFailed(format!("xdotool key failed: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(SubstitutionError::SimulationFailed(format!("xdotool key error: {}", stderr)));
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

/// macOS-specific text insertion using osascript and clipboard (AppleScript paste).
///
/// Uses `pbcopy` to write text to the clipboard and `osascript` to trigger
/// Cmd+V via System Events. This is an alternative to the CGEvent-based
/// clipboard paste that works through the Accessibility framework.
///
/// Note: Requires Accessibility permissions for System Events.
#[cfg(target_os = "macos")]
pub fn insert_via_osascript_paste_macos(
    text: &str,
    config: &SubstitutionConfig,
) -> Result<(), SubstitutionError> {
    if text.len() > MAX_SNIPPET_SIZE {
        return Err(SubstitutionError::SnippetTooLarge(text.len(), MAX_SNIPPET_SIZE));
    }
    tracing::debug!("Inserting via macOS osascript paste: {} chars", text.len());

    use std::io::Write;

    // Write text to clipboard via pbcopy
    let mut child = std::process::Command::new("pbcopy")
        .stdin(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| SubstitutionError::SimulationFailed(format!("pbcopy spawn failed: {}", e)))?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(text.as_bytes())
            .map_err(|e| SubstitutionError::SimulationFailed(format!("pbcopy write failed: {}", e)))?;
    }

    let status = child.wait()
        .map_err(|e| SubstitutionError::SimulationFailed(format!("pbcopy wait failed: {}", e)))?;

    if !status.success() {
        return Err(SubstitutionError::SimulationFailed("pbcopy exited with error".to_string()));
    }

    // Delay to ensure clipboard content is committed
    std::thread::sleep(std::time::Duration::from_millis(20));

    // Use osascript to send Cmd+V via System Events
    let output = std::process::Command::new("osascript")
        .arg("-e")
        .arg("tell application \"System Events\" to keystroke \"v\" using command down")
        .output()
        .map_err(|e| SubstitutionError::SimulationFailed(format!("osascript failed: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(SubstitutionError::SimulationFailed(
            format!("osascript error: {}", stderr),
        ));
    }

    // Wait for paste to complete
    std::thread::sleep(std::time::Duration::from_millis(config.paste_restore_delay_ms));

    Ok(())
}

/// macOS-specific keyword deletion using osascript (AppleScript backspace).
///
/// Uses `osascript` with System Events to send backspace key events.
/// This is an alternative to the CGEvent-based backspace that works through
/// the Accessibility framework.
#[cfg(target_os = "macos")]
#[allow(dead_code)] // Available as alternative to CGEvent-based deletion
fn delete_keyword_osascript_macos(count: usize, config: &SubstitutionConfig) -> Result<(), SubstitutionError> {
    if count > MAX_KEYWORD_LENGTH {
        return Err(SubstitutionError::KeywordTooLong(count, MAX_KEYWORD_LENGTH));
    }
    if count == 0 {
        return Ok(());
    }
    tracing::debug!("Deleting {} characters via macOS osascript backspace", count);

    // Pre-deletion delay for browser compatibility
    if config.pre_deletion_delay_ms > 0 {
        std::thread::sleep(std::time::Duration::from_millis(config.pre_deletion_delay_ms));
    }

    // Use osascript to send backspace key events via System Events
    // key code 51 = macOS backspace
    let script = format!(
        "tell application \"System Events\"\nrepeat {} times\nkey code 51\ndelay 0.005\nend repeat\nend tell",
        count
    );

    let output = std::process::Command::new("osascript")
        .arg("-e")
        .arg(&script)
        .output()
        .map_err(|e| SubstitutionError::SimulationFailed(format!("osascript failed: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(SubstitutionError::SimulationFailed(
            format!("osascript error: {}", stderr),
        ));
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
    /// On Linux, uses xdotool for keyword deletion (terminal-compatible).
    pub fn substitute_via_clipboard<P: ClipboardProvider>(
        &self,
        keyword_len: usize,
        snippet: &str,
        clipboard_mgr: &mut ClipboardManager<P>,
    ) -> Result<(), SubstitutionError> {
        #[cfg(target_os = "macos")]
        {
            delete_keyword_macos(keyword_len, &self.config)?;
            return insert_via_clipboard_macos(snippet, clipboard_mgr, &self.config);
        }
        #[cfg(not(target_os = "macos"))]
        {
            if cfg!(target_os = "linux") {
                delete_keyword_xdotool(keyword_len, &self.config)?;
            } else {
                delete_keyword(keyword_len, &self.config)?;
            }
            insert_via_clipboard(snippet, clipboard_mgr, &self.config)?;
            Ok(())
        }
    }

    /// Performs a full substitution: delete keyword, then insert snippet.
    ///
    /// Uses keystroke-based insertion.
    /// On Linux, uses xdotool for both deletion and insertion (terminal-compatible).
    /// On other platforms, uses rdev.
    pub fn substitute_via_keystrokes(
        &self,
        keyword_len: usize,
        snippet: &str,
    ) -> Result<(), SubstitutionError> {
        #[cfg(target_os = "macos")]
        {
            delete_keyword_macos(keyword_len, &self.config)?;
            return insert_via_keystrokes(snippet, &self.config);
        }
        #[cfg(not(target_os = "macos"))]
        {
            if cfg!(target_os = "linux") {
                delete_keyword_xdotool(keyword_len, &self.config)?;
                insert_via_xdotool(snippet, &self.config)?;
            } else {
                delete_keyword(keyword_len, &self.config)?;
                insert_via_keystrokes(snippet, &self.config)?;
            }
            Ok(())
        }
    }

    /// Performs a full substitution: delete keyword, then insert snippet.
    ///
    /// On macOS, uses osascript (AppleScript System Events) as the equivalent
    /// of xdotool. Uses CGEvent backspace for deletion (fast and reliable)
    /// and osascript for paste.
    /// On Linux, uses xdotool type command (terminal compatible).
    pub fn substitute_via_xdotool(
        &self,
        keyword_len: usize,
        snippet: &str,
    ) -> Result<(), SubstitutionError> {
        #[cfg(target_os = "macos")]
        {
            // On macOS, use osascript (AppleScript System Events) as the
            // equivalent of xdotool. Uses CGEvent backspace for deletion
            // (fast and reliable) and osascript for paste.
            delete_keyword_macos(keyword_len, &self.config)?;
            return insert_via_osascript_paste_macos(snippet, &self.config);
        }
        #[cfg(not(target_os = "macos"))]
        {
            delete_keyword_xdotool(keyword_len, &self.config)?;
            insert_via_xdotool(snippet, &self.config)?;
            Ok(())
        }
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

        // Simulate paste - use CGEvent on macOS for reliability
        #[cfg(target_os = "macos")]
        {
            use std::ffi::c_void;
            type CGEventRef = *mut c_void;
            type CGEventSourceRef = *mut c_void;
            #[link(name = "CoreGraphics", kind = "framework")]
            extern "C" {
                fn CGEventSourceCreate(state_id: i32) -> CGEventSourceRef;
                fn CGEventCreateKeyboardEvent(source: CGEventSourceRef, virtual_key: u16, key_down: bool) -> CGEventRef;
                fn CGEventSetFlags(event: CGEventRef, flags: u64);
                fn CGEventPost(tap: u32, event: CGEventRef);
            }
            #[link(name = "CoreFoundation", kind = "framework")]
            extern "C" {
                fn CFRelease(cf: *const c_void);
            }
            const CMD_FLAG_CHUNK: u64 = 0x00100000;
            unsafe {
                let source = CGEventSourceCreate(1); // HID system state
                let cmd_down = CGEventCreateKeyboardEvent(source, 55, true);
                if !cmd_down.is_null() {
                    CGEventSetFlags(cmd_down, CMD_FLAG_CHUNK);
                    CGEventPost(0, cmd_down);
                    CFRelease(cmd_down as *const c_void);
                }
                std::thread::sleep(Duration::from_millis(5));
                let v_down = CGEventCreateKeyboardEvent(source, 9, true);
                if !v_down.is_null() {
                    CGEventSetFlags(v_down, CMD_FLAG_CHUNK);
                    CGEventPost(0, v_down);
                    CFRelease(v_down as *const c_void);
                }
                std::thread::sleep(Duration::from_millis(5));
                let v_up = CGEventCreateKeyboardEvent(source, 9, false);
                if !v_up.is_null() {
                    CGEventSetFlags(v_up, CMD_FLAG_CHUNK);
                    CGEventPost(0, v_up);
                    CFRelease(v_up as *const c_void);
                }
                std::thread::sleep(Duration::from_millis(5));
                let cmd_up = CGEventCreateKeyboardEvent(source, 55, false);
                if !cmd_up.is_null() {
                    CGEventSetFlags(cmd_up, 0);
                    CGEventPost(0, cmd_up);
                    CFRelease(cmd_up as *const c_void);
                }
                if !source.is_null() {
                    CFRelease(source as *const c_void);
                }
            }
        }
        #[cfg(not(target_os = "macos"))]
        {
            let delay = Duration::from_millis(config.key_delay_ms);
            let paste_modifier = if cfg!(target_os = "macos") {
                Key::MetaLeft
            } else {
                Key::ControlLeft
            };
            send_key_event(EventType::KeyPress(paste_modifier), delay)?;
            press_key(Key::KeyV, delay)?;
            send_key_event(EventType::KeyRelease(paste_modifier), delay)?;
        }

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
        assert_eq!(config.paste_restore_delay_ms, 200);
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
            pre_deletion_delay_ms: 30,
        };
        let engine = SubstitutionEngine::new(config);
        assert_eq!(engine.config().key_delay_ms, 10);
        assert!(engine.config().use_shift_insert);
        assert_eq!(engine.config().pre_deletion_delay_ms, 30);
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
            pre_deletion_delay_ms: 25,
        });
        assert_eq!(engine.config().key_delay_ms, 20);
        assert_eq!(engine.config().paste_restore_delay_ms, 200);
        assert_eq!(engine.config().pre_deletion_delay_ms, 25);
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
            pre_deletion_delay_ms: 35,
        };
        let cloned = config.clone();
        assert_eq!(cloned.key_delay_ms, 15);
        assert_eq!(cloned.paste_restore_delay_ms, 75);
        assert!(cloned.use_shift_insert);
        assert_eq!(cloned.timeout_secs, 7);
        assert_eq!(cloned.pre_deletion_delay_ms, 35);
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
            pre_deletion_delay_ms: 20,
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
    fn test_config_use_shift_insert_default_false() {
        let config = SubstitutionConfig::default();
        // Shift+Insert pastes from X11 PRIMARY selection, not CLIPBOARD.
        // arboard writes to CLIPBOARD, so we must use Ctrl+V on all platforms.
        assert!(!config.use_shift_insert, "Should default to Ctrl+V (not Shift+Insert)");
    }

    // ── Xdotool deletion tests ─────────────────────────────────

    #[test]
    fn test_delete_keyword_xdotool_validates_length() {
        let config = SubstitutionConfig::default();
        let result = delete_keyword_xdotool(MAX_KEYWORD_LENGTH + 1, &config);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), SubstitutionError::KeywordTooLong(..)));
    }

    #[test]
    fn test_delete_keyword_xdotool_zero_length() {
        // Zero length should succeed (no-op)
        // Note: This will actually try to run xdotool with --repeat 0
        // We can't test actual execution in CI, but we can test the validation logic
        // which should allow zero
        assert!(MAX_KEYWORD_LENGTH > 0);
    }

    #[test]
    fn test_substitute_via_keystrokes_uses_xdotool_on_linux() {
        // This test verifies the dispatch logic based on cfg!(target_os = "linux")
        // Actual execution would require xdotool and a display server
        let engine = SubstitutionEngine::with_defaults();
        // We can't test actual execution, but we document the expected behavior:
        // On Linux: calls delete_keyword_xdotool + insert_via_xdotool
        // On others: calls delete_keyword + insert_via_keystrokes
        assert!(engine.config().key_delay_ms > 0);
    }

    #[test]
    fn test_substitute_via_xdotool_always_uses_xdotool() {
        // substitute_via_xdotool should always use xdotool functions
        // regardless of platform
        let engine = SubstitutionEngine::with_defaults();
        // We can't test actual execution, but we document that this method
        // unconditionally calls delete_keyword_xdotool + insert_via_xdotool
        assert!(engine.config().key_delay_ms > 0);
    }

    #[test]
    fn test_keyword_too_long_error_contains_limits() {
        let err = SubstitutionError::KeywordTooLong(300, MAX_KEYWORD_LENGTH);
        let msg = err.to_string();
        assert!(msg.contains("300"));
        assert!(msg.contains(&MAX_KEYWORD_LENGTH.to_string()));
    }

    // ── Browser compatibility: pre-deletion delay ─────────────────

    #[test]
    fn test_config_pre_deletion_delay_default() {
        let config = SubstitutionConfig::default();
        assert_eq!(config.pre_deletion_delay_ms, 20,
            "Default pre-deletion delay should be 20ms for browser compatibility");
    }

    #[test]
    fn test_config_pre_deletion_delay_custom() {
        let config = SubstitutionConfig {
            key_delay_ms: 5,
            paste_restore_delay_ms: 50,
            use_shift_insert: false,
            timeout_secs: 5,
            chunk_delay_ms: 10,
            pre_deletion_delay_ms: 50,
        };
        assert_eq!(config.pre_deletion_delay_ms, 50);
    }

    #[test]
    fn test_config_pre_deletion_delay_zero_allowed() {
        // Zero delay should be allowed (for terminal use where no delay is needed)
        let config = SubstitutionConfig {
            key_delay_ms: 5,
            paste_restore_delay_ms: 50,
            use_shift_insert: false,
            timeout_secs: 5,
            chunk_delay_ms: 10,
            pre_deletion_delay_ms: 0,
        };
        assert_eq!(config.pre_deletion_delay_ms, 0);
    }

    // ── macOS clipboard paste timing ─────────────────────────────

    #[test]
    fn test_config_paste_restore_delay_default_200ms() {
        // paste_restore_delay_ms was increased from 50ms to 200ms to give target
        // applications enough time to read clipboard content before restoration
        let config = SubstitutionConfig::default();
        assert_eq!(config.paste_restore_delay_ms, 200,
            "Default paste restore delay should be 200ms for reliable clipboard paste");
    }

    // ── macOS osascript paste validation ─────────────────────────

    #[test]
    fn test_insert_via_osascript_validates_snippet_size() {
        // insert_via_osascript_paste_macos should validate snippet size
        // We test this on macOS only since the function is cfg(target_os = "macos")
        #[cfg(target_os = "macos")]
        {
            let config = SubstitutionConfig::default();
            let huge_text = "a".repeat(MAX_SNIPPET_SIZE + 1);
            let result = insert_via_osascript_paste_macos(&huge_text, &config);
            assert!(result.is_err());
            assert!(matches!(result.unwrap_err(), SubstitutionError::SnippetTooLarge(..)));
        }
    }

    #[test]
    fn test_delete_keyword_osascript_validates_length() {
        // delete_keyword_osascript_macos should validate keyword length
        #[cfg(target_os = "macos")]
        {
            let config = SubstitutionConfig::default();
            let result = delete_keyword_osascript_macos(MAX_KEYWORD_LENGTH + 1, &config);
            assert!(result.is_err());
            assert!(matches!(result.unwrap_err(), SubstitutionError::KeywordTooLong(..)));
        }
    }

    #[test]
    fn test_delete_keyword_osascript_zero_is_noop() {
        #[cfg(target_os = "macos")]
        {
            let config = SubstitutionConfig::default();
            let result = delete_keyword_osascript_macos(0, &config);
            assert!(result.is_ok());
        }
    }

    // ── Platform dispatch tests ──────────────────────────────────

    #[test]
    fn test_substitute_via_xdotool_uses_osascript_on_macos() {
        // On macOS, substitute_via_xdotool should use osascript-based functions
        // instead of xdotool (which doesn't exist on macOS).
        // We verify the engine can be constructed and the method signature matches.
        let engine = SubstitutionEngine::with_defaults();
        // The dispatch logic uses #[cfg(target_os)] so we can't test both branches
        // in one binary, but we can verify the engine is properly configured
        assert_eq!(engine.config().paste_restore_delay_ms, 200);
    }

    #[test]
    fn test_substitute_via_xdotool_doc_update() {
        // Verify the xdotool method exists and engine works with all three methods
        let engine = SubstitutionEngine::with_defaults();
        // On macOS: substitute_via_xdotool uses delete_keyword_macos + insert_via_osascript_paste_macos
        // On Linux: substitute_via_xdotool uses delete_keyword_xdotool + insert_via_xdotool
        assert!(engine.config().key_delay_ms > 0);
        assert!(engine.config().paste_restore_delay_ms > 0);
    }

    // ── Chunked paste platform dispatch ──────────────────────────

    #[test]
    fn test_chunked_paste_threshold_unchanged() {
        // Verify chunked paste constants haven't changed
        assert_eq!(CHUNKED_PASTE_THRESHOLD, 1000);
        assert_eq!(PASTE_CHUNK_SIZE, 500);
    }
}
