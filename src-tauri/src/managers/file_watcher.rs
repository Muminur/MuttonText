//! File watcher stub for detecting external changes to data files.
//!
//! This module defines the interface for watching config files for changes
//! made by external processes. The actual implementation using the `notify`
//! crate will be added in a future milestone.

use std::path::PathBuf;

use tracing;

// TODO: Add `notify` crate to Cargo.toml when implementing:
//   notify = { version = "6", features = ["macos_fsevent"] }

/// Callback type invoked when a watched file changes.
pub type OnChangeCallback = Box<dyn Fn(&PathBuf) + Send + Sync>;

/// Watches files for external modifications and invokes a callback on change.
///
/// # Future Implementation
///
/// Will use the `notify` crate to receive filesystem events efficiently:
/// - Linux: inotify
/// - macOS: FSEvents
/// - Windows: ReadDirectoryChangesW
pub struct FileWatcher {
    /// Paths currently being watched.
    watched_paths: Vec<PathBuf>,
    /// Callback to invoke when a watched file changes.
    _callback: Option<OnChangeCallback>,
    // TODO: Add notify::RecommendedWatcher field.
}

impl FileWatcher {
    /// Creates a new `FileWatcher` with no watched paths.
    pub fn new() -> Self {
        Self {
            watched_paths: Vec::new(),
            _callback: None,
        }
    }

    /// Registers a path to be watched for changes.
    ///
    /// # Stub
    /// Currently stores the path but does not start actual filesystem monitoring.
    pub fn watch(&mut self, path: PathBuf) {
        // TODO: Register path with notify::Watcher.
        tracing::debug!(?path, "FileWatcher: registered path (stub)");
        self.watched_paths.push(path);
    }

    /// Sets the callback to be invoked when any watched file changes.
    ///
    /// # Stub
    /// Currently stores the callback but does not wire it to filesystem events.
    pub fn on_change(&mut self, callback: OnChangeCallback) {
        // TODO: Wire callback to notify::Watcher event handler.
        self._callback = Some(callback);
    }

    /// Returns the list of currently watched paths.
    pub fn watched_paths(&self) -> &[PathBuf] {
        &self.watched_paths
    }

    // TODO: Add `stop()` method to unregister all watchers.
    // TODO: Add debouncing to avoid rapid-fire callbacks.
}

impl Default for FileWatcher {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_file_watcher_has_no_paths() {
        let watcher = FileWatcher::new();
        assert!(watcher.watched_paths().is_empty());
    }

    #[test]
    fn test_watch_adds_path() {
        let mut watcher = FileWatcher::new();
        watcher.watch(PathBuf::from("/tmp/test.json"));
        assert_eq!(watcher.watched_paths().len(), 1);
    }

    #[test]
    fn test_watch_multiple_paths() {
        let mut watcher = FileWatcher::new();
        watcher.watch(PathBuf::from("/tmp/a.json"));
        watcher.watch(PathBuf::from("/tmp/b.json"));
        assert_eq!(watcher.watched_paths().len(), 2);
    }

    #[test]
    fn test_on_change_accepts_callback() {
        let mut watcher = FileWatcher::new();
        watcher.on_change(Box::new(|_path| {
            // Stub callback - does nothing in test.
        }));
        // No panic means success.
    }

    #[test]
    fn test_default_creates_empty_watcher() {
        let watcher = FileWatcher::default();
        assert!(watcher.watched_paths().is_empty());
    }
}
