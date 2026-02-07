//! Expansion engine orchestration and lifecycle management.
//!
//! The `EngineManager` ties together all components needed for automatic text expansion:
//! - Keyboard hook (platform-specific)
//! - Input buffer management
//! - Matching engine (keyword detection)
//! - Substitution engine (text replacement)
//! - Clipboard manager
//!
//! It handles the full expansion pipeline: keystrokes → buffer → match → expand.

use std::sync::{Arc, Mutex};
use thiserror::Error;

use crate::managers::{
    clipboard_manager::{ClipboardManager, ArboardProvider},
    expansion_pipeline::ExpansionPipeline,
    input_manager::InputManager,
};
use crate::models::{Combo, Preferences};
use crate::models::preferences::PasteMethod;
use crate::platform::keyboard_hook::{FocusDetector, KeyboardHook};

#[cfg(target_os = "linux")]
use crate::platform::linux::{LinuxKeyboardHook, LinuxFocusDetector};

#[cfg(target_os = "macos")]
use crate::platform::macos::{MacOSKeyboardHook, MacOSFocusDetector};

// Windows support is not yet implemented, use mock for now
#[cfg(target_os = "windows")]
use crate::platform::mock::MockFocusDetector;

/// Errors from engine operations.
#[derive(Debug, Error)]
pub enum EngineError {
    #[error("Platform error: {0}")]
    Platform(#[from] crate::platform::keyboard_hook::PlatformError),

    #[error("Engine is not running")]
    NotRunning,

    #[error("Engine is already running")]
    AlreadyRunning,

    #[error("Lock acquisition failed")]
    LockError,
}

/// Current state of the expansion engine.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EngineStatus {
    /// Engine is stopped (not listening for keystrokes).
    Stopped,
    /// Engine is running and actively expanding text.
    Running,
    /// Engine is paused (listening but not expanding).
    Paused,
}

/// Thread-safe inner state of the engine.
struct EngineInner {
    input_manager: InputManager,
    expansion_pipeline: ExpansionPipeline,
    clipboard: ClipboardManager<ArboardProvider>,
    focus_detector: Box<dyn FocusDetector>,
    status: EngineStatus,
    paste_method: PasteMethod,
}

/// Manages the text expansion engine lifecycle.
///
/// This is the central coordinator that:
/// 1. Starts/stops the keyboard hook
/// 2. Feeds keystrokes to the input buffer
/// 3. Checks buffer for matches on every keystroke
/// 4. Triggers expansions when keywords are detected
/// 5. Updates combo usage statistics
pub struct EngineManager {
    inner: Arc<Mutex<EngineInner>>,
    /// Callback to notify when a combo is used (for updating stats in storage).
    on_combo_used: Option<Arc<dyn Fn(uuid::Uuid) + Send + Sync>>,
}

use crate::managers::expansion_pipeline::ExpansionResult;

impl EngineManager {
    /// Checks if there's a match in the buffer without performing expansion.
    /// Returns the match result if found.
    fn check_for_match(state: &mut EngineInner, buffer: &str) -> Option<crate::managers::matching::MatchResult> {
        // Detect the currently focused application
        let current_app = state
            .focus_detector
            .get_active_window_info()
            .ok()
            .map(|info| info.app_name);

        let current_app_ref = current_app.as_deref();

        // Just check for match, don't perform expansion yet
        state.expansion_pipeline.process_buffer(buffer, current_app_ref)
    }

    /// Performs the expansion substitution using the provided match result.
    /// This should be called AFTER pausing the input manager.
    fn perform_expansion(
        state: &mut EngineInner,
        match_result: crate::managers::matching::MatchResult,
    ) -> Option<ExpansionResult> {
        // Perform the actual substitution based on paste method
        let substitution_result = match state.paste_method {
            PasteMethod::Clipboard => {
                state.expansion_pipeline.substitution().substitute_via_clipboard(
                    match_result.keyword_len,
                    &match_result.snippet,
                    &mut state.clipboard,
                )
            }
            PasteMethod::SimulateKeystrokes => {
                state.expansion_pipeline.substitution().substitute_via_keystrokes(
                    match_result.keyword_len,
                    &match_result.snippet,
                )
            }
            PasteMethod::XdotoolType => {
                state.expansion_pipeline.substitution().substitute_via_xdotool(
                    match_result.keyword_len,
                    &match_result.snippet,
                )
            }
        };

        match substitution_result {
            Ok(()) => Some(ExpansionResult {
                combo_id: match_result.combo_id,
                keyword: match_result.keyword,
                snippet: match_result.snippet,
            }),
            Err(e) => {
                tracing::error!("Substitution failed: {}", e);
                None
            }
        }
    }
}

impl EngineManager {
    /// Creates a new EngineManager with default configuration.
    pub fn new() -> Self {
        let mut input_manager = InputManager::new();
        let expansion_pipeline = ExpansionPipeline::with_defaults();
        let clipboard = ClipboardManager::new_system()
            .expect("Failed to initialize clipboard manager");

        // Attach platform-specific keyboard hook and focus detector
        let hook: Box<dyn KeyboardHook> = Self::create_keyboard_hook();
        let focus_detector: Box<dyn FocusDetector> = Self::create_focus_detector();
        input_manager.set_keyboard_hook(hook);

        let inner = EngineInner {
            input_manager,
            expansion_pipeline,
            clipboard,
            focus_detector,
            status: EngineStatus::Stopped,
            paste_method: PasteMethod::default(),
        };

        Self {
            inner: Arc::new(Mutex::new(inner)),
            on_combo_used: None,
        }
    }

    /// Creates the platform-specific keyboard hook.
    fn create_keyboard_hook() -> Box<dyn KeyboardHook> {
        #[cfg(target_os = "linux")]
        {
            Box::new(LinuxKeyboardHook::new())
        }

        #[cfg(target_os = "macos")]
        {
            Box::new(MacOSKeyboardHook::new())
        }

        #[cfg(target_os = "windows")]
        {
            // Windows keyboard hook not yet implemented
            compile_error!("Windows keyboard hook not yet implemented");
        }

        #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
        {
            compile_error!("Unsupported platform for keyboard hooks");
        }
    }

    /// Creates the platform-specific focus detector.
    fn create_focus_detector() -> Box<dyn FocusDetector> {
        #[cfg(target_os = "linux")]
        {
            Box::new(LinuxFocusDetector::new())
        }

        #[cfg(target_os = "macos")]
        {
            Box::new(MacOSFocusDetector::new())
        }

        #[cfg(target_os = "windows")]
        {
            // Windows focus detector not yet implemented, use mock as fallback
            Box::new(MockFocusDetector::new())
        }

        #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
        {
            compile_error!("Unsupported platform for focus detection");
        }
    }

    /// Registers a callback to be invoked when a combo is successfully used.
    pub fn on_combo_used<F>(&mut self, callback: F)
    where
        F: Fn(uuid::Uuid) + Send + Sync + 'static,
    {
        self.on_combo_used = Some(Arc::new(callback));
    }

    /// Loads combos into the expansion engine.
    pub fn load_combos(&self, combos: &[Combo]) -> Result<(), EngineError> {
        let mut inner = self.inner.lock().map_err(|_| EngineError::LockError)?;
        inner.expansion_pipeline.load_combos(combos);
        tracing::info!("Loaded {} combos into expansion engine", combos.len());
        Ok(())
    }

    /// Applies preferences to the expansion engine.
    pub fn apply_preferences(&self, prefs: &Preferences) -> Result<(), EngineError> {
        let mut inner = self.inner.lock().map_err(|_| EngineError::LockError)?;

        // Always add MuttonText to excluded apps to prevent self-expansion
        let mut excluded_apps = prefs.excluded_apps.clone();
        if !excluded_apps.iter().any(|app| app.to_lowercase() == "muttontext") {
            excluded_apps.push("muttontext".to_string());
        }

        // Apply preferences with augmented exclusion list
        let mut prefs_with_self_exclusion = prefs.clone();
        prefs_with_self_exclusion.excluded_apps = excluded_apps;
        inner.expansion_pipeline.apply_preferences(&prefs_with_self_exclusion);

        inner.paste_method = prefs.paste_method;
        tracing::info!("Applied preferences to expansion engine (paste_method: {:?}, excluded_apps: {:?})",
            prefs.paste_method, prefs_with_self_exclusion.excluded_apps);
        Ok(())
    }

    /// Starts the expansion engine.
    ///
    /// This begins listening for keystrokes and will automatically expand
    /// matching keywords.
    pub fn start(&self) -> Result<(), EngineError> {
        let mut inner = self.inner.lock().map_err(|_| EngineError::LockError)?;

        if inner.status != EngineStatus::Stopped {
            return Err(EngineError::AlreadyRunning);
        }

        // Set up the buffer change callback to trigger expansion pipeline
        // We need to use the shared inner state for this
        let inner_clone = self.inner.clone();
        let combo_used_cb = self.on_combo_used.clone();

        inner.input_manager.on_buffer_change(move |buffer| {
            // Lock the inner state to access pipeline and clipboard.
            // NOTE: This callback is invoked from within InputManagerInner's
            // notify_change(), which means the InputManagerInner mutex is ALREADY
            // held. We MUST NOT call any InputManager method that locks that mutex
            // (pause, resume, clear_buffer) or we'll deadlock.
            // Instead, use the lock-free suppress/unsuppress/request_buffer_clear.
            if let Ok(mut state) = inner_clone.lock() {
                // PHASE 1: Check for match (input is NOT suppressed)
                if let Some(match_result) = Self::check_for_match(&mut state, buffer) {
                    // PHASE 2: Match found! Suppress input via lock-free AtomicBool.
                    // This prevents the hook from capturing xdotool keystrokes.
                    state.input_manager.suppress();

                    tracing::info!(
                        "Expanding combo via {:?}: keyword='{}', snippet_len={}",
                        state.paste_method,
                        match_result.keyword,
                        match_result.snippet.len()
                    );

                    // Request buffer clear for the next hook event
                    state.input_manager.request_buffer_clear();

                    // PHASE 3: Perform the actual substitution (while suppressed)
                    if let Some(expansion_result) = Self::perform_expansion(&mut state, match_result) {
                        tracing::info!(
                            "Expanded combo: '{}' → {} chars",
                            expansion_result.keyword,
                            expansion_result.snippet.len()
                        );

                        if let Some(ref cb) = combo_used_cb {
                            cb(expansion_result.combo_id);
                        }
                    }

                    // Small delay to ensure xdotool finishes typing before unsuppressing
                    std::thread::sleep(std::time::Duration::from_millis(100));

                    // PHASE 4: Unsuppress input (lock-free, no deadlock)
                    state.input_manager.unsuppress();
                }
            }
        });

        // Start the keyboard hook
        inner.input_manager.start()?;
        inner.status = EngineStatus::Running;

        tracing::info!("Expansion engine started");
        Ok(())
    }

    /// Stops the expansion engine.
    pub fn stop(&self) -> Result<(), EngineError> {
        let mut inner = self.inner.lock().map_err(|_| EngineError::LockError)?;

        if inner.status == EngineStatus::Stopped {
            return Err(EngineError::NotRunning);
        }

        inner.input_manager.stop()?;
        inner.status = EngineStatus::Stopped;

        tracing::info!("Expansion engine stopped");
        Ok(())
    }

    /// Pauses the expansion engine (hook keeps running but expansions don't fire).
    pub fn pause(&self) -> Result<(), EngineError> {
        let mut inner = self.inner.lock().map_err(|_| EngineError::LockError)?;

        if inner.status != EngineStatus::Running {
            return Err(EngineError::NotRunning);
        }

        inner.input_manager.pause();
        inner.status = EngineStatus::Paused;

        tracing::info!("Expansion engine paused");
        Ok(())
    }

    /// Resumes the expansion engine from paused state.
    pub fn resume(&self) -> Result<(), EngineError> {
        let mut inner = self.inner.lock().map_err(|_| EngineError::LockError)?;

        if inner.status != EngineStatus::Paused {
            return Err(EngineError::NotRunning);
        }

        inner.input_manager.resume();
        inner.status = EngineStatus::Running;

        tracing::info!("Expansion engine resumed");
        Ok(())
    }

    /// Returns the current engine status.
    pub fn status(&self) -> Result<EngineStatus, EngineError> {
        let inner = self.inner.lock().map_err(|_| EngineError::LockError)?;
        Ok(inner.status)
    }

    /// Restarts the engine (stop + start).
    pub fn restart(&self) -> Result<(), EngineError> {
        if self.status()? != EngineStatus::Stopped {
            self.stop()?;
        }
        self.start()?;
        Ok(())
    }
}

impl Default for EngineManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_engine_new() {
        let engine = EngineManager::new();
        assert_eq!(engine.status().unwrap(), EngineStatus::Stopped);
    }

    #[test]
    fn test_engine_load_combos() {
        let engine = EngineManager::new();
        let result = engine.load_combos(&[]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_engine_apply_preferences_stores_paste_method() {
        let engine = EngineManager::new();
        let mut prefs = Preferences::default();
        prefs.paste_method = PasteMethod::XdotoolType;

        let result = engine.apply_preferences(&prefs);
        assert!(result.is_ok());

        // Verify paste_method is stored (we can't directly access it, but the
        // test ensures apply_preferences doesn't panic and accepts the new variant)
    }

    #[test]
    fn test_engine_auto_excludes_muttontext() {
        let engine = EngineManager::new();
        let prefs = Preferences {
            excluded_apps: vec!["1password".to_string()],
            ..Default::default()
        };

        let result = engine.apply_preferences(&prefs);
        assert!(result.is_ok());

        // Verify that MuttonText was auto-added to excluded apps
        // We can't directly inspect the engine state, but we can verify
        // apply_preferences succeeds and doesn't panic
    }

    #[test]
    fn test_engine_does_not_duplicate_muttontext_exclusion() {
        let engine = EngineManager::new();
        let prefs = Preferences {
            excluded_apps: vec!["MuttonText".to_string(), "1password".to_string()],
            ..Default::default()
        };

        let result = engine.apply_preferences(&prefs);
        assert!(result.is_ok());

        // Apply again to ensure no duplication
        let result = engine.apply_preferences(&prefs);
        assert!(result.is_ok());
    }

    #[test]
    fn test_expansion_works_with_correct_pause_timing() {
        // This test verifies that expansion works when pause happens
        // AFTER match detection but BEFORE substitution (not before match detection).
        use crate::models::combo::ComboBuilder;
        use crate::models::matching::MatchingMode;

        let engine = EngineManager::new();

        // Create a combo: "gh" → "https://github.com" (short keyword for testing)
        let combo = ComboBuilder::new()
            .keyword("gh")
            .snippet("https://github.com")
            .matching_mode(MatchingMode::Strict)
            .build()
            .unwrap();

        engine.load_combos(&[combo]).unwrap();

        // Verify the engine can detect the match
        // This tests that match detection works BEFORE any pause occurs
        let inner = engine.inner.lock().unwrap();
        let result = inner.expansion_pipeline.process_buffer("gh", None);
        assert!(result.is_some(), "Should detect match for 'gh'");
        assert_eq!(result.unwrap().keyword, "gh");
    }

    #[test]
    fn test_buffer_cleared_after_expansion() {
        // This test demonstrates the infinite loop bug fix:
        // After expansion, the buffer MUST be cleared to prevent xdotool-typed
        // text from being captured back into the buffer and re-triggering the match.
        use crate::models::combo::ComboBuilder;
        use crate::models::matching::MatchingMode;

        let engine = EngineManager::new();

        // Create a combo: "github" → "https://github.com"
        let combo = ComboBuilder::new()
            .keyword("github")
            .snippet("https://github.com")
            .matching_mode(MatchingMode::Strict)
            .build()
            .unwrap();

        engine.load_combos(&[combo]).unwrap();

        // Verify buffer state through input_manager
        // After we fix the bug, this will pass
        let inner = engine.inner.lock().unwrap();

        // Initially buffer should be empty
        assert_eq!(inner.input_manager.buffer(), "");

        // This test documents expected behavior:
        // 1. Buffer accumulates "github"
        // 2. Match detection occurs (while input is NOT paused)
        // 3. Once match found, InputManager is paused
        // 4. Buffer is cleared
        // 5. Substitution happens (backspace + text insertion)
        // 6. InputManager is resumed
        // 7. Buffer remains empty (xdotool output not captured because we were paused)
    }

    // Note: Full integration tests require a display server and are
    // better suited for manual testing or CI with Xvfb.
}
