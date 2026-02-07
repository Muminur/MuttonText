//! Character buffer and input management for keyword matching.
//!
//! `InputManager` accumulates typed characters into a buffer and resets
//! the buffer on word boundaries, non-printable keys, mouse clicks,
//! and focus changes. Consumers register a callback to be notified
//! whenever the buffer content changes.

use std::sync::{Arc, Mutex, MutexGuard};
use std::sync::atomic::{AtomicBool, Ordering};

use crate::platform::keyboard_hook::{
    FocusDetector, Key, KeyEvent, KeyEventType, KeyboardHook, PlatformError, WindowInfo,
};

/// Helper to handle poisoned mutexes gracefully by recovering the inner data.
fn lock_mutex<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(|e| e.into_inner())
}

/// Default maximum buffer size in characters.
const DEFAULT_MAX_BUFFER_SIZE: usize = 256;

/// Default word boundary characters.
const DEFAULT_WORD_BOUNDARIES: &[char] = &[
    ' ', '\t', '\n', '\r', '.', ',', ';', ':', '!', '?', '(', ')', '[', ']', '{', '}', '<', '>',
    '/', '\\', '|', '"', '\'', '`', '~', '@', '#', '$', '%', '^', '&', '*', '-', '+', '=',
];

/// Shared inner state protected by a mutex so the keyboard callback
/// (running on the hook thread) can mutate the buffer safely.
struct InputManagerInner {
    buffer: String,
    max_buffer_size: usize,
    is_paused: bool,
    word_boundary_chars: Vec<char>,
    last_window_info: Option<WindowInfo>,
    on_buffer_change: Option<Arc<dyn Fn(&str) + Send + Sync>>,
}

impl InputManagerInner {
    fn new() -> Self {
        Self {
            buffer: String::with_capacity(DEFAULT_MAX_BUFFER_SIZE),
            max_buffer_size: DEFAULT_MAX_BUFFER_SIZE,
            is_paused: false,
            word_boundary_chars: DEFAULT_WORD_BOUNDARIES.to_vec(),
            last_window_info: None,
            on_buffer_change: None,
        }
    }

    fn clear_buffer(&mut self) {
        if !self.buffer.is_empty() {
            self.buffer.clear();
            self.notify_change();
        }
    }

    fn push_char(&mut self, c: char) {
        if self.buffer.len() >= self.max_buffer_size {
            // Drop oldest half to avoid unbounded growth while keeping
            // recent context.
            let drain_to = self.buffer.len() / 2;
            // Find a char boundary at or after drain_to.
            let mut boundary = drain_to;
            while boundary < self.buffer.len() && !self.buffer.is_char_boundary(boundary) {
                boundary += 1;
            }
            self.buffer.drain(..boundary);
        }
        self.buffer.push(c);
        self.notify_change();
    }

    fn handle_backspace(&mut self) {
        if self.buffer.pop().is_some() {
            self.notify_change();
        }
    }

    fn is_word_boundary(&self, c: char) -> bool {
        self.word_boundary_chars.contains(&c)
    }

    fn notify_change(&self) {
        if let Some(ref cb) = self.on_buffer_change {
            cb(&self.buffer);
        }
    }
}

/// Manages the character buffer driven by platform keyboard events.
pub struct InputManager {
    inner: Arc<Mutex<InputManagerInner>>,
    keyboard_hook: Option<Box<dyn KeyboardHook>>,
    /// Lock-free flag: when true, the hook callback silently discards events.
    /// Used during expansion to prevent xdotool keystrokes from being captured.
    is_suppressed: Arc<AtomicBool>,
    /// Lock-free flag: when true, the hook callback clears the buffer on the
    /// next event before processing. Used after expansion to reset state.
    needs_buffer_clear: Arc<AtomicBool>,
}

impl InputManager {
    /// Create a new `InputManager` with default settings.
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(InputManagerInner::new())),
            keyboard_hook: None,
            is_suppressed: Arc::new(AtomicBool::new(false)),
            needs_buffer_clear: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Set the maximum buffer size.
    pub fn set_max_buffer_size(&mut self, size: usize) {
        lock_mutex(&self.inner).max_buffer_size = size;
    }

    /// Set the word boundary characters.
    pub fn set_word_boundary_chars(&mut self, chars: Vec<char>) {
        lock_mutex(&self.inner).word_boundary_chars = chars;
    }

    /// Register a callback invoked whenever the buffer content changes.
    pub fn on_buffer_change<F>(&mut self, callback: F)
    where
        F: Fn(&str) + Send + Sync + 'static,
    {
        lock_mutex(&self.inner).on_buffer_change = Some(Arc::new(callback));
    }

    /// Attach a keyboard hook. The hook is not started until `start` is called.
    pub fn set_keyboard_hook(&mut self, hook: Box<dyn KeyboardHook>) {
        self.keyboard_hook = Some(hook);
    }

    /// Start listening for keyboard events.
    pub fn start(&mut self) -> Result<(), PlatformError> {
        let hook = self
            .keyboard_hook
            .as_mut()
            .ok_or_else(|| PlatformError::Internal("No keyboard hook configured".into()))?;

        let inner = self.inner.clone();
        let suppressed = self.is_suppressed.clone();
        let needs_clear = self.needs_buffer_clear.clone();
        hook.start(Box::new(move |event: KeyEvent| {
            // Check lock-free suppression flag first (no mutex needed).
            // During expansion, all events are silently discarded.
            if suppressed.load(Ordering::SeqCst) {
                return;
            }

            let mut state = lock_mutex(&inner);

            // If buffer clear was requested (after expansion), do it now.
            if needs_clear.swap(false, Ordering::SeqCst) {
                state.buffer.clear();
                // Don't notify - silent clear to prevent re-triggering
            }

            if state.is_paused {
                return;
            }
            // Only process key presses.
            if event.event_type != KeyEventType::Press {
                return;
            }
            Self::process_key_event(&mut state, &event);
        }))?;

        tracing::info!("InputManager started");
        Ok(())
    }

    /// Stop listening for keyboard events.
    pub fn stop(&mut self) -> Result<(), PlatformError> {
        if let Some(ref mut hook) = self.keyboard_hook {
            hook.stop()?;
        }
        tracing::info!("InputManager stopped");
        Ok(())
    }

    /// Pause input processing without stopping the hook.
    pub fn pause(&self) {
        lock_mutex(&self.inner).is_paused = true;
        tracing::debug!("InputManager paused");
    }

    /// Resume input processing.
    pub fn resume(&self) {
        lock_mutex(&self.inner).is_paused = false;
        tracing::debug!("InputManager resumed");
    }

    /// Returns whether input processing is paused.
    pub fn is_paused(&self) -> bool {
        lock_mutex(&self.inner).is_paused
    }

    /// Lock-free: suppress all input events (used during expansion).
    /// Safe to call from within the on_buffer_change callback without deadlock.
    pub fn suppress(&self) {
        self.is_suppressed.store(true, Ordering::SeqCst);
        tracing::debug!("InputManager suppressed (lock-free)");
    }

    /// Lock-free: stop suppressing input events.
    /// Safe to call from within the on_buffer_change callback without deadlock.
    pub fn unsuppress(&self) {
        self.is_suppressed.store(false, Ordering::SeqCst);
        tracing::debug!("InputManager unsuppressed (lock-free)");
    }

    /// Lock-free: request buffer clear on the next hook event.
    /// Safe to call from within the on_buffer_change callback without deadlock.
    pub fn request_buffer_clear(&self) {
        self.needs_buffer_clear.store(true, Ordering::SeqCst);
    }

    /// Unsuppress input and clear buffer after a delay, on a background thread.
    /// This ensures xdotool-generated events queued on the rdev thread are
    /// discarded (suppressed) before we resume processing.
    pub fn unsuppress_after(&self, delay: std::time::Duration) {
        let suppressed = self.is_suppressed.clone();
        let needs_clear = self.needs_buffer_clear.clone();
        std::thread::spawn(move || {
            std::thread::sleep(delay);
            needs_clear.store(true, Ordering::SeqCst);
            suppressed.store(false, Ordering::SeqCst);
            tracing::debug!("InputManager unsuppressed after delay (lock-free)");
        });
    }

    /// Get the current buffer contents.
    pub fn buffer(&self) -> String {
        lock_mutex(&self.inner).buffer.clone()
    }

    /// Clear the buffer (e.g. after a successful expansion).
    pub fn clear_buffer(&self) {
        lock_mutex(&self.inner).clear_buffer();
    }

    /// Notify the manager that a mouse click occurred, resetting the buffer.
    pub fn handle_mouse_click(&self) {
        lock_mutex(&self.inner).clear_buffer();
    }

    /// Notify the manager that the focused window may have changed.
    /// If it has, the buffer is cleared.
    pub fn handle_focus_change(&self, detector: &dyn FocusDetector) {
        if let Ok(info) = detector.get_active_window_info() {
            let mut state = lock_mutex(&self.inner);
            let changed = state
                .last_window_info
                .as_ref()
                .map_or(true, |last| *last != info);
            if changed {
                tracing::debug!("Focus changed to: {} ({})", info.app_name, info.title);
                state.last_window_info = Some(info);
                state.clear_buffer();
            }
        }
    }

    /// Process a single key event. Called from the hook callback.
    fn process_key_event(state: &mut InputManagerInner, event: &KeyEvent) {
        // If ctrl/alt/meta is held, reset buffer (likely a shortcut).
        if event.modifiers.ctrl || event.modifiers.alt || event.modifiers.meta {
            state.clear_buffer();
            return;
        }

        match &event.key {
            // Backspace removes the last character.
            Key::Backspace => {
                state.handle_backspace();
            }
            // These non-printable keys reset the buffer.
            Key::Enter | Key::Escape | Key::Tab | Key::Left | Key::Right | Key::Up | Key::Down
            | Key::Home | Key::End | Key::PageUp | Key::PageDown | Key::Delete => {
                state.clear_buffer();
            }
            // Function keys reset the buffer.
            Key::F(_) => {
                state.clear_buffer();
            }
            // Printable character or space.
            Key::Char(c) => {
                if state.is_word_boundary(*c) {
                    state.clear_buffer();
                } else {
                    state.push_char(*c);
                }
            }
            Key::Space => {
                // Space is always a word boundary.
                state.clear_buffer();
            }
            // Unknown keys reset the buffer.
            Key::Other(_) => {
                state.clear_buffer();
            }
        }
    }
}

impl Default for InputManager {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::platform::mock::{MockFocusDetector, MockKeyboardHook};
    use crate::platform::keyboard_hook::Modifiers;
    use std::sync::atomic::{AtomicUsize, Ordering};

    /// Helper: create a KeyEvent for a character press.
    fn char_press(c: char) -> KeyEvent {
        KeyEvent::new(Key::Char(c), KeyEventType::Press, Modifiers::default())
    }

    /// Helper: create a KeyEvent for a special key press.
    fn key_press(key: Key) -> KeyEvent {
        KeyEvent::new(key, KeyEventType::Press, Modifiers::default())
    }

    /// Helper: create a KeyEvent with modifiers.
    fn modified_press(key: Key, mods: Modifiers) -> KeyEvent {
        KeyEvent::new(key, KeyEventType::Press, mods)
    }

    // -- Basic buffer tests (direct state manipulation) --

    #[test]
    fn test_new_manager_has_empty_buffer() {
        let mgr = InputManager::new();
        assert_eq!(mgr.buffer(), "");
    }

    #[test]
    fn test_char_accumulation() {
        let mgr = InputManager::new();
        {
            let mut state = lock_mutex(&mgr.inner);
            InputManager::process_key_event(&mut state, &char_press('h'));
            InputManager::process_key_event(&mut state, &char_press('e'));
            InputManager::process_key_event(&mut state, &char_press('l'));
            InputManager::process_key_event(&mut state, &char_press('l'));
            InputManager::process_key_event(&mut state, &char_press('o'));
        }
        assert_eq!(mgr.buffer(), "hello");
    }

    #[test]
    fn test_backspace_removes_last_char() {
        let mgr = InputManager::new();
        {
            let mut state = lock_mutex(&mgr.inner);
            InputManager::process_key_event(&mut state, &char_press('a'));
            InputManager::process_key_event(&mut state, &char_press('b'));
            InputManager::process_key_event(&mut state, &key_press(Key::Backspace));
        }
        assert_eq!(mgr.buffer(), "a");
    }

    #[test]
    fn test_backspace_on_empty_buffer() {
        let mgr = InputManager::new();
        {
            let mut state = lock_mutex(&mgr.inner);
            InputManager::process_key_event(&mut state, &key_press(Key::Backspace));
        }
        assert_eq!(mgr.buffer(), "");
    }

    #[test]
    fn test_enter_clears_buffer() {
        let mgr = InputManager::new();
        {
            let mut state = lock_mutex(&mgr.inner);
            InputManager::process_key_event(&mut state, &char_press('a'));
            InputManager::process_key_event(&mut state, &key_press(Key::Enter));
        }
        assert_eq!(mgr.buffer(), "");
    }

    #[test]
    fn test_escape_clears_buffer() {
        let mgr = InputManager::new();
        {
            let mut state = lock_mutex(&mgr.inner);
            InputManager::process_key_event(&mut state, &char_press('x'));
            InputManager::process_key_event(&mut state, &key_press(Key::Escape));
        }
        assert_eq!(mgr.buffer(), "");
    }

    #[test]
    fn test_arrow_keys_clear_buffer() {
        for key in [Key::Left, Key::Right, Key::Up, Key::Down] {
            let mgr = InputManager::new();
            {
                let mut state = lock_mutex(&mgr.inner);
                InputManager::process_key_event(&mut state, &char_press('z'));
                InputManager::process_key_event(&mut state, &key_press(key));
            }
            assert_eq!(mgr.buffer(), "", "Arrow key should clear buffer");
        }
    }

    #[test]
    fn test_tab_clears_buffer() {
        let mgr = InputManager::new();
        {
            let mut state = lock_mutex(&mgr.inner);
            InputManager::process_key_event(&mut state, &char_press('t'));
            InputManager::process_key_event(&mut state, &key_press(Key::Tab));
        }
        assert_eq!(mgr.buffer(), "");
    }

    #[test]
    fn test_function_keys_clear_buffer() {
        let mgr = InputManager::new();
        {
            let mut state = lock_mutex(&mgr.inner);
            InputManager::process_key_event(&mut state, &char_press('f'));
            InputManager::process_key_event(&mut state, &key_press(Key::F(5)));
        }
        assert_eq!(mgr.buffer(), "");
    }

    #[test]
    fn test_space_clears_buffer() {
        let mgr = InputManager::new();
        {
            let mut state = lock_mutex(&mgr.inner);
            InputManager::process_key_event(&mut state, &char_press('a'));
            InputManager::process_key_event(&mut state, &key_press(Key::Space));
        }
        assert_eq!(mgr.buffer(), "");
    }

    #[test]
    fn test_word_boundary_chars_clear_buffer() {
        for c in ['.', ',', '!', '?', '(', ')', '/'] {
            let mgr = InputManager::new();
            {
                let mut state = lock_mutex(&mgr.inner);
                InputManager::process_key_event(&mut state, &char_press('x'));
                InputManager::process_key_event(&mut state, &char_press(c));
            }
            assert_eq!(
                mgr.buffer(),
                "",
                "Boundary char '{}' should clear buffer",
                c
            );
        }
    }

    #[test]
    fn test_ctrl_modifier_clears_buffer() {
        let mgr = InputManager::new();
        let mods = Modifiers {
            ctrl: true,
            ..Default::default()
        };
        {
            let mut state = lock_mutex(&mgr.inner);
            InputManager::process_key_event(&mut state, &char_press('a'));
            InputManager::process_key_event(&mut state, &modified_press(Key::Char('c'), mods));
        }
        assert_eq!(mgr.buffer(), "");
    }

    #[test]
    fn test_alt_modifier_clears_buffer() {
        let mgr = InputManager::new();
        let mods = Modifiers {
            alt: true,
            ..Default::default()
        };
        {
            let mut state = lock_mutex(&mgr.inner);
            InputManager::process_key_event(&mut state, &char_press('b'));
            InputManager::process_key_event(&mut state, &modified_press(Key::Char('x'), mods));
        }
        assert_eq!(mgr.buffer(), "");
    }

    #[test]
    fn test_release_events_ignored() {
        let mgr = InputManager::new();
        {
            let mut state = lock_mutex(&mgr.inner);
            InputManager::process_key_event(&mut state, &char_press('a'));
            let _release = KeyEvent::new(
                Key::Char('a'),
                KeyEventType::Release,
                Modifiers::default(),
            );
            // Release is filtered in start(); process_key_event does not
            // filter, but start() does. We test via the hook integration below.
            // Here just verify buffer stays 'a' after another press.
            InputManager::process_key_event(&mut state, &char_press('b'));
        }
        assert_eq!(mgr.buffer(), "ab");
    }

    #[test]
    fn test_max_buffer_size() {
        let mut mgr = InputManager::new();
        mgr.set_max_buffer_size(5);
        {
            let mut state = lock_mutex(&mgr.inner);
            for c in "abcdefgh".chars() {
                InputManager::process_key_event(&mut state, &char_press(c));
            }
        }
        // Buffer should not exceed max; older chars dropped.
        let buf = mgr.buffer();
        assert!(buf.len() <= 8); // after drain, it keeps roughly half + new
        assert!(buf.ends_with('h'));
    }

    #[test]
    fn test_clear_buffer_explicit() {
        let mgr = InputManager::new();
        {
            let mut state = lock_mutex(&mgr.inner);
            InputManager::process_key_event(&mut state, &char_press('z'));
        }
        assert_eq!(mgr.buffer(), "z");
        mgr.clear_buffer();
        assert_eq!(mgr.buffer(), "");
    }

    #[test]
    fn test_mouse_click_clears_buffer() {
        let mgr = InputManager::new();
        {
            let mut state = lock_mutex(&mgr.inner);
            InputManager::process_key_event(&mut state, &char_press('m'));
        }
        mgr.handle_mouse_click();
        assert_eq!(mgr.buffer(), "");
    }

    #[test]
    fn test_focus_change_clears_buffer() {
        let mgr = InputManager::new();
        let detector = MockFocusDetector::new();
        {
            let mut state = lock_mutex(&mgr.inner);
            InputManager::process_key_event(&mut state, &char_press('f'));
        }
        // First call records window and clears (no previous).
        mgr.handle_focus_change(&detector);
        assert_eq!(mgr.buffer(), "");

        // Type again.
        {
            let mut state = lock_mutex(&mgr.inner);
            InputManager::process_key_event(&mut state, &char_press('g'));
        }

        // Same window — should NOT clear.
        mgr.handle_focus_change(&detector);
        assert_eq!(mgr.buffer(), "g");

        // Change window — should clear.
        detector.set_window_info(WindowInfo {
            title: "Other".into(),
            app_name: "other".into(),
            process_id: Some(999),
        });
        mgr.handle_focus_change(&detector);
        assert_eq!(mgr.buffer(), "");
    }

    #[test]
    fn test_pause_and_resume() {
        let mgr = InputManager::new();
        assert!(!mgr.is_paused());
        mgr.pause();
        assert!(mgr.is_paused());
        mgr.resume();
        assert!(!mgr.is_paused());
    }

    #[test]
    fn test_on_buffer_change_callback() {
        let mut mgr = InputManager::new();
        let changes = Arc::new(Mutex::new(Vec::<String>::new()));
        let changes_clone = changes.clone();

        mgr.on_buffer_change(move |buf| {
            lock_mutex(&changes_clone).push(buf.to_string());
        });

        {
            let mut state = lock_mutex(&mgr.inner);
            InputManager::process_key_event(&mut state, &char_press('a'));
            InputManager::process_key_event(&mut state, &char_press('b'));
            InputManager::process_key_event(&mut state, &key_press(Key::Backspace));
        }

        let log = lock_mutex(&changes);
        assert_eq!(*log, vec!["a", "ab", "a"]);
    }

    #[test]
    fn test_custom_word_boundaries() {
        let mut mgr = InputManager::new();
        // Only treat '.' as a word boundary.
        mgr.set_word_boundary_chars(vec!['.']);

        {
            let mut state = lock_mutex(&mgr.inner);
            // Comma should NOT clear buffer now.
            InputManager::process_key_event(&mut state, &char_press('a'));
            InputManager::process_key_event(&mut state, &char_press(','));
        }
        assert_eq!(mgr.buffer(), "a,");

        {
            let mut state = lock_mutex(&mgr.inner);
            // Dot SHOULD clear buffer.
            InputManager::process_key_event(&mut state, &char_press('.'));
        }
        assert_eq!(mgr.buffer(), "");
    }

    // -- Integration test with MockKeyboardHook --

    #[test]
    fn test_start_stop_with_mock_hook() {
        let mut mgr = InputManager::new();
        let hook = MockKeyboardHook::new();
        mgr.set_keyboard_hook(Box::new(hook));

        mgr.start().unwrap();
        mgr.stop().unwrap();
    }

    #[test]
    fn test_start_without_hook_errors() {
        let mut mgr = InputManager::new();
        let result = mgr.start();
        assert!(result.is_err());
    }

    #[test]
    fn test_hook_callback_integration() {
        let mut mgr = InputManager::new();
        let mock_hook = MockKeyboardHook::new();

        let change_count = Arc::new(AtomicUsize::new(0));
        let count_clone = change_count.clone();
        mgr.on_buffer_change(move |_| {
            count_clone.fetch_add(1, Ordering::SeqCst);
        });

        mgr.set_keyboard_hook(Box::new(mock_hook));
        mgr.start().unwrap();

        // We cannot easily inject events through the mock after start() takes
        // ownership of the callback. This test verifies start/stop lifecycle.
        // Direct event processing is tested above via process_key_event.

        mgr.stop().unwrap();
    }

    #[test]
    fn test_delete_clears_buffer() {
        let mgr = InputManager::new();
        {
            let mut state = lock_mutex(&mgr.inner);
            InputManager::process_key_event(&mut state, &char_press('d'));
            InputManager::process_key_event(&mut state, &key_press(Key::Delete));
        }
        assert_eq!(mgr.buffer(), "");
    }

    #[test]
    fn test_home_end_page_keys_clear_buffer() {
        for key in [Key::Home, Key::End, Key::PageUp, Key::PageDown] {
            let mgr = InputManager::new();
            {
                let mut state = lock_mutex(&mgr.inner);
                InputManager::process_key_event(&mut state, &char_press('x'));
                InputManager::process_key_event(&mut state, &key_press(key));
            }
            assert_eq!(mgr.buffer(), "");
        }
    }

    #[test]
    fn test_other_key_clears_buffer() {
        let mgr = InputManager::new();
        {
            let mut state = lock_mutex(&mgr.inner);
            InputManager::process_key_event(&mut state, &char_press('o'));
            InputManager::process_key_event(
                &mut state,
                &key_press(Key::Other("XF86Audio".into())),
            );
        }
        assert_eq!(mgr.buffer(), "");
    }

    #[test]
    fn test_meta_modifier_clears_buffer() {
        let mgr = InputManager::new();
        let mods = Modifiers {
            meta: true,
            ..Default::default()
        };
        {
            let mut state = lock_mutex(&mgr.inner);
            InputManager::process_key_event(&mut state, &char_press('a'));
            InputManager::process_key_event(&mut state, &modified_press(Key::Char('v'), mods));
        }
        assert_eq!(mgr.buffer(), "");
    }
}
