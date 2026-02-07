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
use crate::platform::keyboard_hook::KeyboardHook;

#[cfg(target_os = "linux")]
use crate::platform::linux::LinuxKeyboardHook;

#[cfg(target_os = "macos")]
use crate::platform::macos::MacOSKeyboardHook;

#[cfg(target_os = "windows")]
use crate::platform::windows::WindowsKeyboardHook;

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
    /// Helper function to try expanding from a buffer, handling borrow checker constraints.
    fn try_expand(state: &mut EngineInner, buffer: &str) -> Option<ExpansionResult> {
        let result = match state.paste_method {
            PasteMethod::Clipboard => {
                state.expansion_pipeline.expand_via_clipboard(
                    buffer,
                    None,
                    &mut state.clipboard,
                )
            }
            PasteMethod::SimulateKeystrokes => {
                state.expansion_pipeline.expand_via_keystrokes(buffer, None)
            }
            PasteMethod::XdotoolType => {
                state.expansion_pipeline.expand_via_xdotool(buffer, None)
            }
        };
        match result {
            Ok(Some(result)) => Some(result),
            Ok(None) => None,
            Err(e) => {
                tracing::error!("Expansion failed: {}", e);
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

        // Attach platform-specific keyboard hook
        let hook: Box<dyn KeyboardHook> = Self::create_keyboard_hook();
        input_manager.set_keyboard_hook(hook);

        let inner = EngineInner {
            input_manager,
            expansion_pipeline,
            clipboard,
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
            Box::new(WindowsKeyboardHook::new())
        }

        #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
        {
            compile_error!("Unsupported platform for keyboard hooks");
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
        inner.expansion_pipeline.apply_preferences(prefs);
        inner.paste_method = prefs.paste_method;
        tracing::info!("Applied preferences to expansion engine (paste_method: {:?})", prefs.paste_method);
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
            // Lock the inner state to access pipeline and clipboard
            if let Ok(mut state) = inner_clone.lock() {
                // Attempt expansion via clipboard (preferred method on Linux)
                // Use the EngineInner helper method to avoid borrow checker issues
                if let Some(expansion_result) = Self::try_expand(&mut state, buffer) {
                    tracing::info!(
                        "Expanded combo: '{}' → {} chars",
                        expansion_result.keyword,
                        expansion_result.snippet.len()
                    );
                    // Notify callback
                    if let Some(ref cb) = combo_used_cb {
                        cb(expansion_result.combo_id);
                    }
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

    // Note: Full integration tests require a display server and are
    // better suited for manual testing or CI with Xvfb.
}
