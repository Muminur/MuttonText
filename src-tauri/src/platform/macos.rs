//! macOS keyboard hook and focus detection.
//!
//! Provides two keyboard hook implementations:
//!
//! - `MacOSKeyboardHook`: Uses a direct CoreGraphics `CGEventTap` via FFI.
//!   Requires Accessibility permissions (System Preferences → Security &
//!   Privacy → Privacy → Accessibility). Kept as a fallback.
//!
//! - `IOHIDKeyboardHook` (default): Uses IOKit's `IOHIDManager` API to
//!   monitor keyboard input. Does NOT require Accessibility permissions,
//!   making it work with ad-hoc signed apps on macOS Ventura and later.

#![cfg(target_os = "macos")]

use std::os::raw::c_void;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;

use crate::platform::keyboard_hook::{
    FocusDetector, Key, KeyEvent, KeyEventType, KeyboardHook, Modifiers, PlatformError,
    WindowInfo,
};

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
/// Uses the macOS `AXIsProcessTrusted` API via the
/// `ApplicationServices` framework to check if THIS process has
/// accessibility permissions. This is the correct API — the previous
/// AppleScript approach incorrectly checked System Events' trust status
/// instead of the running app's.
///
/// # Returns
///
/// `PermissionStatus::Granted` if permissions are granted,
/// `PermissionStatus::Denied` if denied,
/// `PermissionStatus::Unknown` if the check fails.
pub fn check_accessibility_permission() -> PermissionStatus {
    // AXIsProcessTrusted() returns a Boolean (u8) indicating whether this
    // process is trusted for accessibility.  It is part of the
    // ApplicationServices umbrella framework which is always available on
    // macOS.
    #[link(name = "ApplicationServices", kind = "framework")]
    extern "C" {
        fn AXIsProcessTrusted() -> u8; // Boolean (0 = false, 1 = true)
    }

    // SAFETY: AXIsProcessTrusted is a well-defined C function with no
    // parameters and no side effects — it simply reads the TCC database.
    let trusted = unsafe { AXIsProcessTrusted() };

    if trusted != 0 {
        tracing::debug!("Accessibility permissions are granted");
        PermissionStatus::Granted
    } else {
        tracing::warn!("Accessibility permissions are not granted");
        PermissionStatus::Denied
    }
}

/// Check accessibility permissions and optionally prompt the user.
///
/// When `prompt` is `true` and the app is not yet trusted, macOS will
/// show the standard system dialog asking the user to grant Accessibility
/// access. The app will automatically appear in System Settings →
/// Privacy & Security → Accessibility.
///
/// # Returns
///
/// `PermissionStatus::Granted` if already trusted,
/// `PermissionStatus::Denied` if not trusted (dialog shown when `prompt` is true).
pub fn check_accessibility_permission_with_prompt(prompt: bool) -> PermissionStatus {
    #[link(name = "ApplicationServices", kind = "framework")]
    extern "C" {
        fn AXIsProcessTrustedWithOptions(options: *const std::ffi::c_void) -> u8;
    }

    #[link(name = "CoreFoundation", kind = "framework")]
    extern "C" {
        fn CFDictionaryCreate(
            allocator: *const std::ffi::c_void,
            keys: *const *const std::ffi::c_void,
            values: *const *const std::ffi::c_void,
            num_values: isize,
            key_callbacks: *const std::ffi::c_void,
            value_callbacks: *const std::ffi::c_void,
            ) -> *const std::ffi::c_void;
        fn CFRelease(cf: *const std::ffi::c_void);

        // Well-known CF constants
        static kCFBooleanTrue: *const std::ffi::c_void;
        static kCFBooleanFalse: *const std::ffi::c_void;
        // These are CFDictionaryKeyCallBacks (48 bytes) and CFDictionaryValueCallBacks (40 bytes)
        // structs. We only need their addresses, so we declare them as opaque byte arrays.
        static kCFTypeDictionaryKeyCallBacks: [u8; 48];
        static kCFTypeDictionaryValueCallBacks: [u8; 40];
    }

    // The key string "AXTrustedCheckOptionPrompt" is defined in
    // HIServices/AXUIElement.h.  We build it manually via CoreFoundation.
    #[link(name = "CoreFoundation", kind = "framework")]
    extern "C" {
        fn CFStringCreateWithCString(
            alloc: *const std::ffi::c_void,
            c_str: *const u8,
            encoding: u32,
        ) -> *const std::ffi::c_void;
    }

    const K_CF_STRING_ENCODING_UTF8: u32 = 0x0800_0100;

    unsafe {
        let key_str = b"AXTrustedCheckOptionPrompt\0";
        let key = CFStringCreateWithCString(
            std::ptr::null(),
            key_str.as_ptr(),
            K_CF_STRING_ENCODING_UTF8,
        );

        let value = if prompt {
            kCFBooleanTrue
        } else {
            kCFBooleanFalse
        };

        let keys = [key];
        let values = [value];
        let dict = CFDictionaryCreate(
            std::ptr::null(),
            keys.as_ptr(),
            values.as_ptr(),
            1,
            kCFTypeDictionaryKeyCallBacks.as_ptr() as *const std::ffi::c_void,
            kCFTypeDictionaryValueCallBacks.as_ptr() as *const std::ffi::c_void,
        );

        let trusted = AXIsProcessTrustedWithOptions(dict);

        CFRelease(dict);
        CFRelease(key);

        if trusted != 0 {
            tracing::debug!("Accessibility permissions are granted");
            PermissionStatus::Granted
        } else {
            tracing::warn!("Accessibility permissions are not granted — user has been prompted");
            PermissionStatus::Denied
        }
    }
}

/// Request accessibility permissions by triggering the macOS system prompt.
///
/// This uses `AXIsProcessTrustedWithOptions` with the prompt option set to
/// `true`, which causes macOS to show the standard Accessibility permission
/// dialog and automatically adds the app to System Settings → Privacy &
/// Security → Accessibility.
///
/// # Returns
///
/// `Ok(())` always — the system dialog is shown asynchronously and the user
/// must grant permission and restart the app for it to take effect.
pub fn request_accessibility_permission() -> Result<(), PlatformError> {
    let status = check_accessibility_permission_with_prompt(true);
    tracing::info!(
        "Requested accessibility permission via system prompt (current status: {:?})",
        status
    );
    Ok(())
}

// ---------------------------------------------------------------------------
// MacOSKeyboardHook
// ---------------------------------------------------------------------------

/// macOS keyboard hook backed by a direct CoreGraphics `CGEventTap`.
///
/// # Accessibility Permissions
///
/// macOS requires Accessibility permissions for `CGEventTap`. The app
/// must be added to System Preferences → Security & Privacy → Privacy →
/// Accessibility. Without this, `CGEventTapCreate` may return null or
/// the tap will silently receive zero events.
///
/// # Limitation: Cannot be restarted
///
/// Once `stop()` is called, the hook cannot be cleanly restarted.
/// Attempting to start again will return an error.
/// To re-enable the hook after stopping, create a new instance.
pub struct MacOSKeyboardHook {
    running: Arc<AtomicBool>,
    /// Track if hook was ever started (even if later stopped).
    /// The CGEventTap run loop cannot be cleanly stopped and restarted.
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
        if self.started_once.load(Ordering::SeqCst) {
            return Err(PlatformError::Internal(
                "Hook cannot be restarted after stop(); create a new instance".into(),
            ));
        }
        self.running.store(true, Ordering::SeqCst);
        self.started_once.store(true, Ordering::SeqCst);
        let running = self.running.clone();
        let callback: Arc<dyn Fn(KeyEvent) + Send + Sync> = Arc::from(callback);

        let ctrl_down = Arc::new(AtomicBool::new(false));
        let alt_down = Arc::new(AtomicBool::new(false));
        let meta_down = Arc::new(AtomicBool::new(false));
        let shift_down = Arc::new(AtomicBool::new(false));

        let ctrl = ctrl_down.clone();
        let alt = alt_down.clone();
        let meta = meta_down.clone();
        let shift = shift_down.clone();

        thread::Builder::new()
            .name("muttontext-keyboard-hook".into())
            .spawn(move || {
                tracing::info!("macOS keyboard hook thread started");

                // Wait for accessibility permission before creating the event tap.
                #[link(name = "ApplicationServices", kind = "framework")]
                extern "C" {
                    fn AXIsProcessTrusted() -> u8;
                }

                loop {
                    if !running.load(Ordering::SeqCst) {
                        tracing::info!("MacOSKeyboardHook stopped while waiting for accessibility");
                        return;
                    }
                    let trusted = unsafe { AXIsProcessTrusted() };
                    if trusted != 0 {
                        tracing::info!("Accessibility permission confirmed — creating event tap");
                        break;
                    }
                    tracing::debug!("Waiting for accessibility permission before creating event tap...");
                    std::thread::sleep(std::time::Duration::from_secs(2));
                }

                // ---- Direct CGEventTap via CoreGraphics FFI ----
                use std::ffi::c_void;

                type CGEventTapProxy = *mut c_void;
                type CGEventRef = *mut c_void;
                type CFMachPortRef = *mut c_void;
                type CFRunLoopSourceRef = *mut c_void;
                type CFRunLoopRef = *mut c_void;

                // CGEventType constants
                const K_CG_EVENT_KEY_DOWN: u32 = 10;
                const K_CG_EVENT_KEY_UP: u32 = 11;
                const K_CG_EVENT_FLAGS_CHANGED: u32 = 12;
                const K_CG_EVENT_TAP_DISABLED_BY_TIMEOUT: u32 = 0xFFFFFFFE;

                // CGEventTapLocation
                const K_CG_SESSION_EVENT_TAP: u32 = 1;
                // CGEventTapPlacement
                const K_CG_HEAD_INSERT_EVENT_TAP: u32 = 0;
                // CGEventTapOptions
                const K_CG_EVENT_TAP_OPTION_LISTEN_ONLY: u32 = 1;

                // Event mask bits
                const EVENT_MASK: u64 = (1u64 << K_CG_EVENT_KEY_DOWN)
                    | (1u64 << K_CG_EVENT_KEY_UP)
                    | (1u64 << K_CG_EVENT_FLAGS_CHANGED);

                #[link(name = "CoreGraphics", kind = "framework")]
                extern "C" {
                    fn CGEventTapCreate(
                        tap: u32,
                        place: u32,
                        options: u32,
                        events_of_interest: u64,
                        callback: unsafe extern "C" fn(
                            CGEventTapProxy,
                            u32,
                            CGEventRef,
                            *mut c_void,
                        ) -> CGEventRef,
                        user_info: *mut c_void,
                    ) -> CFMachPortRef;
                    fn CGEventTapEnable(tap: CFMachPortRef, enable: bool);
                    fn CGEventGetIntegerValueField(event: CGEventRef, field: u32) -> i64;
                    fn CGEventGetFlags(event: CGEventRef) -> u64;
                }

                #[link(name = "CoreFoundation", kind = "framework")]
                extern "C" {
                    fn CFMachPortCreateRunLoopSource(
                        allocator: *const c_void,
                        port: CFMachPortRef,
                        order: i64,
                    ) -> CFRunLoopSourceRef;
                    fn CFRunLoopGetMain() -> CFRunLoopRef;
                    fn CFRunLoopAddSource(
                        rl: CFRunLoopRef,
                        source: CFRunLoopSourceRef,
                        mode: *const c_void,
                    );

                    static kCFRunLoopCommonModes: *const c_void;
                }

                // kCGKeyboardEventKeycode field
                const K_CG_KEYBOARD_EVENT_KEYCODE: u32 = 9;

                // Modifier flag masks
                const K_CG_EVENT_FLAG_MASK_SHIFT: u64 = 0x00020000;
                const K_CG_EVENT_FLAG_MASK_CONTROL: u64 = 0x00040000;
                const K_CG_EVENT_FLAG_MASK_ALTERNATE: u64 = 0x00080000;
                const K_CG_EVENT_FLAG_MASK_COMMAND: u64 = 0x00100000;

                /// Context for the CGEventTap callback
                struct TapContext {
                    callback: Arc<dyn Fn(KeyEvent) + Send + Sync>,
                    running: Arc<AtomicBool>,
                    ctrl: Arc<AtomicBool>,
                    alt: Arc<AtomicBool>,
                    meta: Arc<AtomicBool>,
                    shift: Arc<AtomicBool>,
                    tap: std::sync::Mutex<Option<CFMachPortRef>>,
                }

                /// Map macOS virtual keycode to our Key enum
                fn virtual_keycode_to_key(keycode: u16) -> Key {
                    match keycode {
                        0 => Key::Char('a'), 1 => Key::Char('s'), 2 => Key::Char('d'),
                        3 => Key::Char('f'), 4 => Key::Char('h'), 5 => Key::Char('g'),
                        6 => Key::Char('z'), 7 => Key::Char('x'), 8 => Key::Char('c'),
                        9 => Key::Char('v'), 11 => Key::Char('b'), 12 => Key::Char('q'),
                        13 => Key::Char('w'), 14 => Key::Char('e'), 15 => Key::Char('r'),
                        16 => Key::Char('y'), 17 => Key::Char('t'), 18 => Key::Char('1'),
                        19 => Key::Char('2'), 20 => Key::Char('3'), 21 => Key::Char('4'),
                        22 => Key::Char('6'), 23 => Key::Char('5'), 24 => Key::Char('='),
                        25 => Key::Char('9'), 26 => Key::Char('7'), 27 => Key::Char('-'),
                        28 => Key::Char('8'), 29 => Key::Char('0'), 30 => Key::Char(']'),
                        31 => Key::Char('o'), 32 => Key::Char('u'), 33 => Key::Char('['),
                        34 => Key::Char('i'), 35 => Key::Char('p'), 36 => Key::Enter,
                        37 => Key::Char('l'), 38 => Key::Char('j'), 39 => Key::Char('\''),
                        40 => Key::Char('k'), 41 => Key::Char(';'), 42 => Key::Char('\\'),
                        43 => Key::Char(','), 44 => Key::Char('/'), 45 => Key::Char('n'),
                        46 => Key::Char('m'), 47 => Key::Char('.'), 48 => Key::Tab,
                        49 => Key::Space, 50 => Key::Char('`'), 51 => Key::Backspace,
                        53 => Key::Escape, 76 => Key::Enter,
                        // Arrow keys
                        123 => Key::Left, 124 => Key::Right,
                        125 => Key::Down, 126 => Key::Up,
                        // Function keys
                        122 => Key::F(1), 120 => Key::F(2), 99 => Key::F(3),
                        118 => Key::F(4), 96 => Key::F(5), 97 => Key::F(6),
                        98 => Key::F(7), 100 => Key::F(8), 101 => Key::F(9),
                        109 => Key::F(10), 103 => Key::F(11), 111 => Key::F(12),
                        // Navigation
                        115 => Key::Home, 119 => Key::End,
                        116 => Key::PageUp, 121 => Key::PageDown,
                        117 => Key::Delete,
                        // Numpad
                        65 => Key::Char('.'), 67 => Key::Char('*'),
                        69 => Key::Char('+'), 75 => Key::Char('/'),
                        78 => Key::Char('-'), 81 => Key::Char('='),
                        82 => Key::Char('0'), 83 => Key::Char('1'),
                        84 => Key::Char('2'), 85 => Key::Char('3'),
                        86 => Key::Char('4'), 87 => Key::Char('5'),
                        88 => Key::Char('6'), 89 => Key::Char('7'),
                        91 => Key::Char('8'), 92 => Key::Char('9'),
                        _ => Key::Other(format!("mac_vk_{}", keycode)),
                    }
                }

                /// Returns true if the keycode is a modifier key
                fn is_modifier_keycode(keycode: u16) -> bool {
                    matches!(keycode, 54 | 55 | 56 | 57 | 58 | 59 | 60 | 61 | 62 | 63)
                }

                unsafe extern "C" fn tap_callback(
                    _proxy: CGEventTapProxy,
                    event_type: u32,
                    event: CGEventRef,
                    user_info: *mut c_void,
                ) -> CGEventRef {
                    if user_info.is_null() || event.is_null() {
                        return event;
                    }

                    let ctx = &*(user_info as *const TapContext);

                    if !ctx.running.load(Ordering::SeqCst) {
                        return event;
                    }

                    // Re-enable tap if disabled by timeout
                    if event_type == K_CG_EVENT_TAP_DISABLED_BY_TIMEOUT {
                        tracing::warn!("CGEventTap disabled by timeout — re-enabling");
                        if let Ok(guard) = ctx.tap.lock() {
                            if let Some(tap_ref) = *guard {
                                CGEventTapEnable(tap_ref, true);
                            }
                        }
                        return event;
                    }

                    let keycode = CGEventGetIntegerValueField(event, K_CG_KEYBOARD_EVENT_KEYCODE) as u16;
                    let flags = CGEventGetFlags(event);

                    match event_type {
                        K_CG_EVENT_KEY_DOWN | K_CG_EVENT_KEY_UP => {
                            // Skip modifier-only keys
                            if is_modifier_keycode(keycode) {
                                return event;
                            }

                            let key_event_type = if event_type == K_CG_EVENT_KEY_DOWN {
                                KeyEventType::Press
                            } else {
                                KeyEventType::Release
                            };

                            let key = virtual_keycode_to_key(keycode);
                            let modifiers = Modifiers {
                                ctrl: (flags & K_CG_EVENT_FLAG_MASK_CONTROL) != 0,
                                alt: (flags & K_CG_EVENT_FLAG_MASK_ALTERNATE) != 0,
                                meta: (flags & K_CG_EVENT_FLAG_MASK_COMMAND) != 0,
                                shift: (flags & K_CG_EVENT_FLAG_MASK_SHIFT) != 0,
                            };

                            tracing::debug!(
                                "CGEventTap event: {:?} (keycode={}, type={:?}, mods=ctrl:{} alt:{} meta:{} shift:{})",
                                key, keycode, key_event_type,
                                modifiers.ctrl, modifiers.alt, modifiers.meta, modifiers.shift
                            );

                            let ke = KeyEvent::new(key, key_event_type, modifiers);
                            (ctx.callback)(ke);
                        }
                        K_CG_EVENT_FLAGS_CHANGED => {
                            // Update modifier state from flags
                            ctx.ctrl.store((flags & K_CG_EVENT_FLAG_MASK_CONTROL) != 0, Ordering::SeqCst);
                            ctx.alt.store((flags & K_CG_EVENT_FLAG_MASK_ALTERNATE) != 0, Ordering::SeqCst);
                            ctx.meta.store((flags & K_CG_EVENT_FLAG_MASK_COMMAND) != 0, Ordering::SeqCst);
                            ctx.shift.store((flags & K_CG_EVENT_FLAG_MASK_SHIFT) != 0, Ordering::SeqCst);
                        }
                        _ => {}
                    }

                    event
                }

                // Create the tap context
                let ctx = Box::new(TapContext {
                    callback,
                    running: running.clone(),
                    ctrl,
                    alt,
                    meta,
                    shift,
                    tap: std::sync::Mutex::new(None),
                });
                let ctx_ptr = Box::into_raw(ctx);

                unsafe {
                    let tap = CGEventTapCreate(
                        K_CG_SESSION_EVENT_TAP,
                        K_CG_HEAD_INSERT_EVENT_TAP,
                        K_CG_EVENT_TAP_OPTION_LISTEN_ONLY,
                        EVENT_MASK,
                        tap_callback,
                        ctx_ptr as *mut c_void,
                    );

                    if tap.is_null() {
                        tracing::error!("CGEventTapCreate returned null — accessibility may not be granted");
                        running.store(false, Ordering::SeqCst);
                        let _ = Box::from_raw(ctx_ptr);
                        return;
                    }

                    tracing::info!("CGEventTap created successfully");

                    // Store tap ref so callback can re-enable if disabled
                    if let Ok(mut guard) = (*ctx_ptr).tap.lock() {
                        *guard = Some(tap);
                    }

                    let source = CFMachPortCreateRunLoopSource(std::ptr::null(), tap, 0);
                    if source.is_null() {
                        tracing::error!("CFMachPortCreateRunLoopSource failed");
                        running.store(false, Ordering::SeqCst);
                        let _ = Box::from_raw(ctx_ptr);
                        return;
                    }

                    let run_loop = CFRunLoopGetMain();
                    CFRunLoopAddSource(run_loop, source, kCFRunLoopCommonModes);
                    CGEventTapEnable(tap, true);

                    tracing::info!("CGEventTap enabled on main run loop — listening for keyboard events");

                    // Keep thread alive to maintain ownership of TapContext.
                    // The main run loop is already running (managed by Tauri/Cocoa),
                    // so we just wait here until stop() is called.
                    while running.load(Ordering::SeqCst) {
                        std::thread::sleep(std::time::Duration::from_secs(1));
                    }

                    // Cleanup after stop
                    tracing::info!("CGEventTap thread exiting — cleaning up");
                    let _ = Box::from_raw(ctx_ptr);
                }

                running.store(false, Ordering::SeqCst);
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
// IOHIDManager FFI
// ---------------------------------------------------------------------------

#[link(name = "IOKit", kind = "framework")]
extern "C" {
    fn IOHIDManagerCreate(allocator: *const c_void, options: u32) -> *mut c_void;
    fn IOHIDManagerSetDeviceMatching(manager: *mut c_void, matching: *const c_void);
    fn IOHIDManagerRegisterInputValueCallback(
        manager: *mut c_void,
        callback: unsafe extern "C" fn(*mut c_void, u32, *mut c_void, *mut c_void),
        context: *mut c_void,
    );
    fn IOHIDManagerScheduleWithRunLoop(
        manager: *mut c_void,
        run_loop: *mut c_void,
        mode: *const c_void,
    );
    fn IOHIDManagerOpen(manager: *mut c_void, options: u32) -> i32;
    fn IOHIDManagerClose(manager: *mut c_void, options: u32) -> i32;
    fn IOHIDManagerUnscheduleFromRunLoop(
        manager: *mut c_void,
        run_loop: *mut c_void,
        mode: *const c_void,
    );
    fn IOHIDValueGetElement(value: *mut c_void) -> *mut c_void;
    fn IOHIDValueGetIntegerValue(value: *mut c_void) -> i64;
    fn IOHIDElementGetUsagePage(element: *mut c_void) -> u32;
    fn IOHIDElementGetUsage(element: *mut c_void) -> u32;
}

#[link(name = "CoreFoundation", kind = "framework")]
extern "C" {
    fn CFRunLoopGetCurrent() -> *mut c_void;
    fn CFRunLoopRun();
    fn CFRunLoopStop(run_loop: *mut c_void);
    fn CFNumberCreate(
        allocator: *const c_void,
        the_type: i64,
        value_ptr: *const c_void,
    ) -> *const c_void;
    fn CFDictionaryCreate(
        allocator: *const c_void,
        keys: *const *const c_void,
        values: *const *const c_void,
        num_values: isize,
        key_callbacks: *const c_void,
        value_callbacks: *const c_void,
    ) -> *const c_void;
    fn CFRelease(cf: *const c_void);
    fn CFStringCreateWithCString(
        alloc: *const c_void,
        c_str: *const u8,
        encoding: u32,
    ) -> *const c_void;

    static kCFRunLoopDefaultMode: *const c_void;
    static kCFAllocatorDefault: *const c_void;
    static kCFTypeDictionaryKeyCallBacks: [u8; 48];
    static kCFTypeDictionaryValueCallBacks: [u8; 40];
}

/// CFNumber type constant for 32-bit signed integer.
const K_CF_NUMBER_SINT32_TYPE: i64 = 3;
/// CoreFoundation UTF-8 string encoding.
const K_CF_STRING_ENCODING_UTF8_HID: u32 = 0x0800_0100;

/// HID usage page for Generic Desktop controls.
const K_HID_USAGE_PAGE_GENERIC_DESKTOP: u32 = 0x01;
/// HID usage for Keyboard within Generic Desktop.
const K_HID_USAGE_KEYBOARD: u32 = 0x06;
/// HID usage page for Keyboard/Keypad.
const K_HID_USAGE_PAGE_KEYBOARD: u32 = 0x07;

// ---------------------------------------------------------------------------
// IOHIDKeyboardHook
// ---------------------------------------------------------------------------

/// Context passed through the IOHIDManager callback's `void* context` pointer.
struct HIDCallbackContext {
    callback: Arc<dyn Fn(KeyEvent) + Send + Sync>,
    running: Arc<AtomicBool>,
    shift_down: AtomicBool,
    ctrl_down: AtomicBool,
    alt_down: AtomicBool,
    meta_down: AtomicBool,
}

/// A wrapper around a raw `CFRunLoopRef` pointer that can be sent across
/// threads. `CFRunLoopStop` is documented by Apple as thread-safe, so
/// storing the pointer and calling `CFRunLoopStop` from another thread is
/// valid.
struct SendableRunLoop(*mut c_void);
unsafe impl Send for SendableRunLoop {}
unsafe impl Sync for SendableRunLoop {}

/// macOS keyboard hook backed by `IOHIDManager`.
///
/// Unlike `MacOSKeyboardHook` (which uses `CGEventTap` via rdev), this
/// implementation uses IOKit's HID Manager to receive keyboard events.
/// IOHIDManager does **not** require Accessibility permissions, making it
/// suitable for ad-hoc signed apps on macOS Ventura and later.
///
/// # How it works
///
/// 1. An `IOHIDManager` is created and configured to match keyboard devices
///    (Generic Desktop / Keyboard usage pair).
/// 2. An input-value callback is registered that fires for every HID event.
/// 3. The manager is scheduled on a `CFRunLoop` which runs on a dedicated
///    background thread.
/// 4. When `stop()` is called, the run loop is stopped and the manager is
///    closed.
pub struct IOHIDKeyboardHook {
    running: Arc<AtomicBool>,
    /// Handle to the run loop of the background thread so we can stop it.
    run_loop: Arc<std::sync::Mutex<Option<SendableRunLoop>>>,
}

impl IOHIDKeyboardHook {
    pub fn new() -> Self {
        Self {
            running: Arc::new(AtomicBool::new(false)),
            run_loop: Arc::new(std::sync::Mutex::new(None)),
        }
    }
}

impl Default for IOHIDKeyboardHook {
    fn default() -> Self {
        Self::new()
    }
}

/// Convert a USB HID keyboard usage code (usage page 0x07) to our `Key` enum.
fn hid_usage_to_key(usage: u32) -> Option<Key> {
    match usage {
        // Letters: 0x04 = 'a' through 0x1D = 'z'
        0x04..=0x1D => {
            let ch = (b'a' + (usage as u8 - 0x04)) as char;
            Some(Key::Char(ch))
        }
        // Numbers: 0x1E = '1' through 0x26 = '9', 0x27 = '0'
        0x1E..=0x26 => {
            let ch = (b'1' + (usage as u8 - 0x1E)) as char;
            Some(Key::Char(ch))
        }
        0x27 => Some(Key::Char('0')),
        // Special keys
        0x28 => Some(Key::Enter),
        0x29 => Some(Key::Escape),
        0x2A => Some(Key::Backspace),
        0x2B => Some(Key::Tab),
        0x2C => Some(Key::Space),
        // Punctuation and symbols
        0x2D => Some(Key::Char('-')),  // Hyphen
        0x2E => Some(Key::Char('=')),  // Equal
        0x2F => Some(Key::Char('[')),  // Left Bracket
        0x30 => Some(Key::Char(']')),  // Right Bracket
        0x31 => Some(Key::Char('\\')), // Backslash
        0x33 => Some(Key::Char(';')),  // Semicolon
        0x34 => Some(Key::Char('\'')),  // Quote
        0x35 => Some(Key::Char('`')),  // Grave Accent
        0x36 => Some(Key::Char(',')),  // Comma
        0x37 => Some(Key::Char('.')),  // Period
        0x38 => Some(Key::Char('/')),  // Slash
        // Function keys: 0x3A = F1 through 0x45 = F12
        0x3A..=0x45 => {
            let n = (usage - 0x3A + 1) as u8;
            Some(Key::F(n))
        }
        // Navigation
        0x49 => Some(Key::Other("Insert".into())),
        0x4A => Some(Key::Home),
        0x4B => Some(Key::PageUp),
        0x4C => Some(Key::Delete),
        0x4D => Some(Key::End),
        0x4E => Some(Key::PageDown),
        0x4F => Some(Key::Right),
        0x50 => Some(Key::Left),
        0x51 => Some(Key::Down),
        0x52 => Some(Key::Up),
        // Modifier keys — handled separately for tracking but returned as Other
        0xE0 => Some(Key::Other("LeftControl".into())),
        0xE1 => Some(Key::Other("LeftShift".into())),
        0xE2 => Some(Key::Other("LeftAlt".into())),
        0xE3 => Some(Key::Other("LeftGUI".into())),
        0xE4 => Some(Key::Other("RightControl".into())),
        0xE5 => Some(Key::Other("RightShift".into())),
        0xE6 => Some(Key::Other("RightAlt".into())),
        0xE7 => Some(Key::Other("RightGUI".into())),
        _ => None, // Unknown / unmapped usage
    }
}

/// Returns `true` if the HID usage code corresponds to a modifier key.
fn hid_usage_is_modifier(usage: u32) -> bool {
    matches!(usage, 0xE0..=0xE7)
}

/// The C callback registered with `IOHIDManagerRegisterInputValueCallback`.
///
/// # Safety
///
/// Called by the IOKit framework. `context` must be a valid pointer to a
/// `HIDCallbackContext` that outlives the callback registration.
unsafe extern "C" fn hid_input_value_callback(
    context: *mut c_void,
    _result: u32,
    _sender: *mut c_void,
    value: *mut c_void,
) {
    if context.is_null() || value.is_null() {
        return;
    }

    let ctx = &*(context as *const HIDCallbackContext);

    if !ctx.running.load(Ordering::SeqCst) {
        return;
    }

    let element = IOHIDValueGetElement(value);
    if element.is_null() {
        return;
    }

    let usage_page = IOHIDElementGetUsagePage(element);
    let usage = IOHIDElementGetUsage(element);
    let int_value = IOHIDValueGetIntegerValue(value);

    // We only care about keyboard/keypad events (usage page 0x07).
    if usage_page != K_HID_USAGE_PAGE_KEYBOARD {
        return;
    }

    let event_type = if int_value != 0 {
        KeyEventType::Press
    } else {
        KeyEventType::Release
    };

    // Track modifier state
    if hid_usage_is_modifier(usage) {
        let pressed = event_type == KeyEventType::Press;
        match usage {
            0xE0 | 0xE4 => ctx.ctrl_down.store(pressed, Ordering::SeqCst),
            0xE1 | 0xE5 => ctx.shift_down.store(pressed, Ordering::SeqCst),
            0xE2 | 0xE6 => ctx.alt_down.store(pressed, Ordering::SeqCst),
            0xE3 | 0xE7 => ctx.meta_down.store(pressed, Ordering::SeqCst),
            _ => {}
        }
        // Don't forward modifier-only events (matches MacOSKeyboardHook behavior)
        return;
    }

    if let Some(key) = hid_usage_to_key(usage) {
        tracing::debug!("HID event: {:?} (usage=0x{:02X}, type={:?})", key, usage, event_type);

        let modifiers = Modifiers {
            ctrl: ctx.ctrl_down.load(Ordering::SeqCst),
            shift: ctx.shift_down.load(Ordering::SeqCst),
            alt: ctx.alt_down.load(Ordering::SeqCst),
            meta: ctx.meta_down.load(Ordering::SeqCst),
        };

        let ke = KeyEvent::new(key, event_type, modifiers);
        (ctx.callback)(ke);
    }
}

/// Create a CoreFoundation number from an `i32` value.
///
/// # Safety
///
/// Calls CoreFoundation FFI. The returned pointer must be released with
/// `CFRelease` when no longer needed.
unsafe fn cf_number_create(value: i32) -> *const c_void {
    CFNumberCreate(
        kCFAllocatorDefault,
        K_CF_NUMBER_SINT32_TYPE,
        &value as *const i32 as *const c_void,
    )
}

/// Create the device matching dictionary for keyboard devices.
///
/// # Safety
///
/// Calls CoreFoundation FFI. The returned pointer must be released with
/// `CFRelease` when no longer needed.
unsafe fn create_keyboard_matching_dict() -> *const c_void {
    let page_key_str = b"DeviceUsagePage\0";
    let usage_key_str = b"DeviceUsage\0";

    let page_key = CFStringCreateWithCString(
        kCFAllocatorDefault,
        page_key_str.as_ptr(),
        K_CF_STRING_ENCODING_UTF8_HID,
    );
    let usage_key = CFStringCreateWithCString(
        kCFAllocatorDefault,
        usage_key_str.as_ptr(),
        K_CF_STRING_ENCODING_UTF8_HID,
    );

    let page_val = cf_number_create(K_HID_USAGE_PAGE_GENERIC_DESKTOP as i32);
    let usage_val = cf_number_create(K_HID_USAGE_KEYBOARD as i32);

    let keys = [page_key, usage_key];
    let values = [page_val, usage_val];

    let dict = CFDictionaryCreate(
        kCFAllocatorDefault,
        keys.as_ptr(),
        values.as_ptr(),
        2,
        kCFTypeDictionaryKeyCallBacks.as_ptr() as *const c_void,
        kCFTypeDictionaryValueCallBacks.as_ptr() as *const c_void,
    );

    // Release intermediate objects (dict retains them)
    CFRelease(page_key);
    CFRelease(usage_key);
    CFRelease(page_val);
    CFRelease(usage_val);

    dict
}

impl KeyboardHook for IOHIDKeyboardHook {
    fn start(
        &mut self,
        callback: Box<dyn Fn(KeyEvent) + Send + Sync>,
    ) -> Result<(), PlatformError> {
        if self.running.load(Ordering::SeqCst) {
            return Err(PlatformError::AlreadyRunning);
        }

        self.running.store(true, Ordering::SeqCst);
        let running = self.running.clone();
        let run_loop_handle = self.run_loop.clone();

        thread::Builder::new()
            .name("muttontext-iohid-hook".into())
            .spawn(move || {
                tracing::info!("IOHIDKeyboardHook thread started");

                // Build the callback context. It is heap-allocated and leaked
                // intentionally — it must live as long as the IOHIDManager
                // callback is registered (i.e., until the run loop exits).
                let ctx = Box::new(HIDCallbackContext {
                    callback: Arc::from(callback),
                    running: running.clone(),
                    shift_down: AtomicBool::new(false),
                    ctrl_down: AtomicBool::new(false),
                    alt_down: AtomicBool::new(false),
                    meta_down: AtomicBool::new(false),
                });
                let ctx_ptr = Box::into_raw(ctx) as *mut c_void;

                unsafe {
                    // 1. Create IOHIDManager
                    let manager = IOHIDManagerCreate(kCFAllocatorDefault, 0);
                    if manager.is_null() {
                        tracing::error!("Failed to create IOHIDManager");
                        running.store(false, Ordering::SeqCst);
                        // Reclaim context
                        let _ = Box::from_raw(ctx_ptr as *mut HIDCallbackContext);
                        return;
                    }

                    // 2. Create and set device matching dictionary
                    let matching = create_keyboard_matching_dict();
                    IOHIDManagerSetDeviceMatching(manager, matching);
                    CFRelease(matching);

                    // 3. Register input value callback
                    IOHIDManagerRegisterInputValueCallback(
                        manager,
                        hid_input_value_callback,
                        ctx_ptr,
                    );

                    // 4. Schedule with the current thread's run loop
                    let rl = CFRunLoopGetCurrent();
                    IOHIDManagerScheduleWithRunLoop(manager, rl, kCFRunLoopDefaultMode);

                    // Store the run loop handle so stop() can call CFRunLoopStop
                    if let Ok(mut guard) = run_loop_handle.lock() {
                        *guard = Some(SendableRunLoop(rl));
                    }

                    // 5. Open the manager — retry in a loop waiting for Input Monitoring permission
                    loop {
                        if !running.load(Ordering::SeqCst) {
                            tracing::info!("IOHIDKeyboardHook stopped while waiting for Input Monitoring permission");
                            IOHIDManagerUnscheduleFromRunLoop(manager, rl, kCFRunLoopDefaultMode);
                            let _ = Box::from_raw(ctx_ptr as *mut HIDCallbackContext);
                            return;
                        }
                        let open_result = IOHIDManagerOpen(manager, 0);
                        if open_result == 0 {
                            break; // Success — permission granted
                        }
                        tracing::warn!(
                            "IOHIDManagerOpen failed (error {}). Input Monitoring permission may not be granted. \
                             Grant in System Settings → Privacy & Security → Input Monitoring. Retrying in 3s...",
                            open_result
                        );
                        // Close and retry — IOHIDManagerOpen may leave the manager in a bad state
                        IOHIDManagerClose(manager, 0);
                        std::thread::sleep(std::time::Duration::from_secs(3));
                    }

                    tracing::info!("IOHIDManager opened successfully — listening for keyboard events");

                    // 6. Run the run loop (blocks until CFRunLoopStop is called)
                    CFRunLoopRun();

                    // Cleanup after run loop exits
                    tracing::info!("IOHIDKeyboardHook run loop exited — cleaning up");
                    IOHIDManagerClose(manager, 0);
                    IOHIDManagerUnscheduleFromRunLoop(manager, rl, kCFRunLoopDefaultMode);
                    // Deregister callback by passing null
                    IOHIDManagerRegisterInputValueCallback(
                        manager,
                        std::mem::transmute::<
                            *const c_void,
                            unsafe extern "C" fn(*mut c_void, u32, *mut c_void, *mut c_void),
                        >(std::ptr::null()),
                        std::ptr::null_mut(),
                    );
                    CFRelease(manager as *const c_void);

                    // Reclaim and drop the context
                    let _ = Box::from_raw(ctx_ptr as *mut HIDCallbackContext);
                }

                running.store(false, Ordering::SeqCst);
                tracing::info!("IOHIDKeyboardHook thread exiting");
            })
            .map_err(|e| PlatformError::Internal(e.to_string()))?;

        tracing::info!("IOHIDKeyboardHook started");
        Ok(())
    }

    fn stop(&mut self) -> Result<(), PlatformError> {
        if !self.running.load(Ordering::SeqCst) {
            return Err(PlatformError::NotRunning);
        }

        self.running.store(false, Ordering::SeqCst);

        // Stop the CFRunLoop to unblock the background thread
        if let Ok(mut guard) = self.run_loop.lock() {
            if let Some(SendableRunLoop(rl)) = guard.take() {
                unsafe {
                    CFRunLoopStop(rl);
                }
            }
        }

        tracing::info!("IOHIDKeyboardHook stopped");
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
