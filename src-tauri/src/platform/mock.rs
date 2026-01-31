//! Mock implementations of platform traits for testing.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};

use crate::platform::keyboard_hook::{
    FocusDetector, KeyEvent, KeyboardHook, PlatformError, WindowInfo,
};

/// Helper to handle poisoned mutexes gracefully by recovering the inner data.
fn lock_mutex<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(|e| e.into_inner())
}

// ---------------------------------------------------------------------------
// MockKeyboardHook
// ---------------------------------------------------------------------------

/// A keyboard hook that records calls and lets tests inject events.
pub struct MockKeyboardHook {
    running: Arc<AtomicBool>,
    callback: Arc<Mutex<Option<Box<dyn Fn(KeyEvent) + Send + Sync>>>>,
}

impl MockKeyboardHook {
    pub fn new() -> Self {
        Self {
            running: Arc::new(AtomicBool::new(false)),
            callback: Arc::new(Mutex::new(None)),
        }
    }

    /// Simulate a key event as if it came from the OS.
    pub fn inject_event(&self, event: KeyEvent) {
        let cb = lock_mutex(&self.callback);
        if let Some(ref f) = *cb {
            f(event);
        }
    }
}

impl Default for MockKeyboardHook {
    fn default() -> Self {
        Self::new()
    }
}

impl KeyboardHook for MockKeyboardHook {
    fn start(
        &mut self,
        callback: Box<dyn Fn(KeyEvent) + Send + Sync>,
    ) -> Result<(), PlatformError> {
        if self.running.load(Ordering::SeqCst) {
            return Err(PlatformError::AlreadyRunning);
        }
        *lock_mutex(&self.callback) = Some(callback);
        self.running.store(true, Ordering::SeqCst);
        tracing::info!("MockKeyboardHook started");
        Ok(())
    }

    fn stop(&mut self) -> Result<(), PlatformError> {
        if !self.running.load(Ordering::SeqCst) {
            return Err(PlatformError::NotRunning);
        }
        self.running.store(false, Ordering::SeqCst);
        *lock_mutex(&self.callback) = None;
        tracing::info!("MockKeyboardHook stopped");
        Ok(())
    }

    fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }
}

// ---------------------------------------------------------------------------
// MockFocusDetector
// ---------------------------------------------------------------------------

/// A focus detector that returns a configurable `WindowInfo`.
pub struct MockFocusDetector {
    info: Arc<Mutex<WindowInfo>>,
}

impl MockFocusDetector {
    pub fn new() -> Self {
        Self {
            info: Arc::new(Mutex::new(WindowInfo::default())),
        }
    }

    /// Set the window info that will be returned by `get_active_window_info`.
    pub fn set_window_info(&self, info: WindowInfo) {
        *lock_mutex(&self.info) = info;
    }
}

impl Default for MockFocusDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl FocusDetector for MockFocusDetector {
    fn get_active_window_info(&self) -> Result<WindowInfo, PlatformError> {
        Ok(lock_mutex(&self.info).clone())
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::platform::keyboard_hook::{Key, KeyEventType, Modifiers};
    use std::sync::atomic::AtomicUsize;

    #[test]
    fn test_mock_hook_start_stop() {
        let mut hook = MockKeyboardHook::new();
        assert!(!hook.is_running());

        hook.start(Box::new(|_| {})).unwrap();
        assert!(hook.is_running());

        // Double start should error
        let err = hook.start(Box::new(|_| {}));
        assert!(err.is_err());

        hook.stop().unwrap();
        assert!(!hook.is_running());

        // Double stop should error
        let err = hook.stop();
        assert!(err.is_err());
    }

    #[test]
    fn test_mock_hook_inject_event() {
        let mut hook = MockKeyboardHook::new();
        let count = Arc::new(AtomicUsize::new(0));
        let count_clone = count.clone();

        hook.start(Box::new(move |_ev| {
            count_clone.fetch_add(1, Ordering::SeqCst);
        }))
        .unwrap();

        let ev = KeyEvent::new(Key::Char('a'), KeyEventType::Press, Modifiers::default());
        hook.inject_event(ev);
        assert_eq!(count.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn test_mock_focus_detector_default() {
        let det = MockFocusDetector::new();
        let info = det.get_active_window_info().unwrap();
        assert_eq!(info, WindowInfo::default());
    }

    #[test]
    fn test_mock_focus_detector_set_info() {
        let det = MockFocusDetector::new();
        let custom = WindowInfo {
            title: "My App".into(),
            app_name: "myapp".into(),
            process_id: Some(1234),
        };
        det.set_window_info(custom.clone());
        assert_eq!(det.get_active_window_info().unwrap(), custom);
    }
}
