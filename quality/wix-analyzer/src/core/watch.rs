//! Watch mode for continuous file analysis
//!
//! Provides file system watching capabilities for development workflows.
//! When files change, they are automatically re-analyzed.
//!
//! # Example
//!
//! ```no_run
//! use wix_analyzer::core::watch::{FileWatcher, WatchEvent};
//! use std::path::Path;
//!
//! let mut watcher = FileWatcher::new(Path::new("./src")).unwrap();
//! for event in watcher {
//!     match event {
//!         WatchEvent::FileChanged(path) => println!("Changed: {:?}", path),
//!         WatchEvent::FileCreated(path) => println!("Created: {:?}", path),
//!         WatchEvent::FileDeleted(path) => println!("Deleted: {:?}", path),
//!         WatchEvent::Error(e) => eprintln!("Error: {}", e),
//!     }
//! }
//! ```

use notify::{
    Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher as NotifyWatcher,
};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver, Sender};
use std::time::Duration;

/// Events emitted by the file watcher
#[derive(Debug, Clone)]
pub enum WatchEvent {
    /// A WiX file was modified
    FileChanged(PathBuf),
    /// A new WiX file was created
    FileCreated(PathBuf),
    /// A WiX file was deleted
    FileDeleted(PathBuf),
    /// An error occurred
    Error(String),
}

/// File watcher error types
#[derive(Debug)]
pub enum WatchError {
    /// Failed to create watcher
    InitError(String),
    /// Failed to watch path
    WatchError(String),
    /// Channel error
    ChannelError(String),
}

impl std::fmt::Display for WatchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InitError(msg) => write!(f, "Failed to initialize watcher: {}", msg),
            Self::WatchError(msg) => write!(f, "Failed to watch path: {}", msg),
            Self::ChannelError(msg) => write!(f, "Channel error: {}", msg),
        }
    }
}

impl std::error::Error for WatchError {}

/// File system watcher for WiX files
pub struct FileWatcher {
    _watcher: RecommendedWatcher,
    receiver: Receiver<WatchEvent>,
    extensions: Vec<String>,
}

impl FileWatcher {
    /// Create a new file watcher for the given path
    ///
    /// By default, watches for `.wxs`, `.wxi`, and `.wxl` files.
    pub fn new(path: &Path) -> Result<Self, WatchError> {
        Self::with_extensions(path, vec!["wxs", "wxi", "wxl"])
    }

    /// Create a new file watcher with custom file extensions
    pub fn with_extensions(path: &Path, extensions: Vec<&str>) -> Result<Self, WatchError> {
        let (tx, rx) = mpsc::channel();
        let extensions: Vec<String> = extensions.iter().map(|s| s.to_string()).collect();
        let ext_clone = extensions.clone();

        let watcher = RecommendedWatcher::new(
            move |res: Result<Event, notify::Error>| {
                Self::handle_event(res, &tx, &ext_clone);
            },
            Config::default().with_poll_interval(Duration::from_secs(1)),
        )
        .map_err(|e| WatchError::InitError(e.to_string()))?;

        let mut file_watcher = Self {
            _watcher: watcher,
            receiver: rx,
            extensions,
        };

        file_watcher.watch(path)?;

        Ok(file_watcher)
    }

    /// Start watching a path
    pub fn watch(&mut self, path: &Path) -> Result<(), WatchError> {
        self._watcher
            .watch(path, RecursiveMode::Recursive)
            .map_err(|e| WatchError::WatchError(e.to_string()))
    }

    /// Stop watching a path
    pub fn unwatch(&mut self, path: &Path) -> Result<(), WatchError> {
        self._watcher
            .unwatch(path)
            .map_err(|e| WatchError::WatchError(e.to_string()))
    }

    /// Check if a path has a watched extension
    fn is_watched_file(path: &Path, extensions: &[String]) -> bool {
        path.extension()
            .and_then(|e| e.to_str())
            .map(|e| extensions.iter().any(|ext| ext.eq_ignore_ascii_case(e)))
            .unwrap_or(false)
    }

    fn handle_event(
        res: Result<Event, notify::Error>,
        tx: &Sender<WatchEvent>,
        extensions: &[String],
    ) {
        match res {
            Ok(event) => {
                for path in event.paths {
                    if !Self::is_watched_file(&path, extensions) {
                        continue;
                    }

                    let watch_event = match event.kind {
                        EventKind::Create(_) => WatchEvent::FileCreated(path),
                        EventKind::Modify(_) => WatchEvent::FileChanged(path),
                        EventKind::Remove(_) => WatchEvent::FileDeleted(path),
                        _ => continue,
                    };

                    let _ = tx.send(watch_event);
                }
            }
            Err(e) => {
                let _ = tx.send(WatchEvent::Error(e.to_string()));
            }
        }
    }

    /// Try to receive the next event without blocking
    pub fn try_recv(&self) -> Option<WatchEvent> {
        self.receiver.try_recv().ok()
    }

    /// Receive the next event, blocking until one is available
    pub fn recv(&self) -> Result<WatchEvent, WatchError> {
        self.receiver
            .recv()
            .map_err(|e| WatchError::ChannelError(e.to_string()))
    }

    /// Receive the next event with a timeout
    pub fn recv_timeout(&self, timeout: Duration) -> Option<WatchEvent> {
        self.receiver.recv_timeout(timeout).ok()
    }

    /// Get the watched extensions
    pub fn extensions(&self) -> &[String] {
        &self.extensions
    }
}

impl Iterator for FileWatcher {
    type Item = WatchEvent;

    fn next(&mut self) -> Option<Self::Item> {
        self.recv().ok()
    }
}

/// Watch configuration
#[derive(Debug, Clone)]
pub struct WatchConfig {
    /// File extensions to watch
    pub extensions: Vec<String>,
    /// Debounce duration in milliseconds
    pub debounce_ms: u64,
    /// Whether to clear screen on each run
    pub clear_screen: bool,
    /// Whether to run on startup
    pub run_on_start: bool,
}

impl Default for WatchConfig {
    fn default() -> Self {
        Self {
            extensions: vec!["wxs".to_string(), "wxi".to_string(), "wxl".to_string()],
            debounce_ms: 300,
            clear_screen: true,
            run_on_start: true,
        }
    }
}

impl WatchConfig {
    /// Create a new watch config with default settings
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the extensions to watch
    pub fn with_extensions(mut self, extensions: Vec<&str>) -> Self {
        self.extensions = extensions.into_iter().map(String::from).collect();
        self
    }

    /// Set the debounce duration
    pub fn with_debounce(mut self, ms: u64) -> Self {
        self.debounce_ms = ms;
        self
    }

    /// Set whether to clear screen
    pub fn with_clear_screen(mut self, clear: bool) -> Self {
        self.clear_screen = clear;
        self
    }

    /// Set whether to run on startup
    pub fn with_run_on_start(mut self, run: bool) -> Self {
        self.run_on_start = run;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_watch_config_default() {
        let config = WatchConfig::default();
        assert_eq!(config.extensions, vec!["wxs", "wxi", "wxl"]);
        assert_eq!(config.debounce_ms, 300);
        assert!(config.clear_screen);
        assert!(config.run_on_start);
    }

    #[test]
    fn test_watch_config_builder() {
        let config = WatchConfig::new()
            .with_extensions(vec!["wxs"])
            .with_debounce(500)
            .with_clear_screen(false)
            .with_run_on_start(false);

        assert_eq!(config.extensions, vec!["wxs"]);
        assert_eq!(config.debounce_ms, 500);
        assert!(!config.clear_screen);
        assert!(!config.run_on_start);
    }

    #[test]
    fn test_is_watched_file() {
        let extensions = vec!["wxs".to_string(), "wxi".to_string()];

        assert!(FileWatcher::is_watched_file(
            Path::new("test.wxs"),
            &extensions
        ));
        assert!(FileWatcher::is_watched_file(
            Path::new("test.WXS"),
            &extensions
        ));
        assert!(FileWatcher::is_watched_file(
            Path::new("test.wxi"),
            &extensions
        ));
        assert!(!FileWatcher::is_watched_file(
            Path::new("test.xml"),
            &extensions
        ));
        assert!(!FileWatcher::is_watched_file(
            Path::new("test"),
            &extensions
        ));
    }

    #[test]
    fn test_watch_error_display() {
        let err = WatchError::InitError("test".to_string());
        assert!(err.to_string().contains("initialize watcher"));

        let err = WatchError::WatchError("test".to_string());
        assert!(err.to_string().contains("Failed to watch"));

        let err = WatchError::ChannelError("test".to_string());
        assert!(err.to_string().contains("Channel error"));
    }

    #[test]
    fn test_watch_event_variants() {
        let path = PathBuf::from("test.wxs");

        let event = WatchEvent::FileChanged(path.clone());
        if let WatchEvent::FileChanged(p) = event {
            assert_eq!(p, path);
        } else {
            panic!("Expected FileChanged");
        }

        let event = WatchEvent::FileCreated(path.clone());
        if let WatchEvent::FileCreated(p) = event {
            assert_eq!(p, path);
        } else {
            panic!("Expected FileCreated");
        }

        let event = WatchEvent::FileDeleted(path.clone());
        if let WatchEvent::FileDeleted(p) = event {
            assert_eq!(p, path);
        } else {
            panic!("Expected FileDeleted");
        }

        let event = WatchEvent::Error("test error".to_string());
        if let WatchEvent::Error(msg) = event {
            assert_eq!(msg, "test error");
        } else {
            panic!("Expected Error");
        }
    }

    #[test]
    fn test_file_watcher_creation() {
        let temp_dir = TempDir::new().unwrap();
        let watcher = FileWatcher::new(temp_dir.path());
        assert!(watcher.is_ok());
    }

    #[test]
    fn test_file_watcher_extensions() {
        let temp_dir = TempDir::new().unwrap();
        let watcher = FileWatcher::with_extensions(temp_dir.path(), vec!["wxs", "xml"]).unwrap();
        assert_eq!(watcher.extensions(), &["wxs", "xml"]);
    }

    #[test]
    fn test_file_watcher_try_recv_empty() {
        let temp_dir = TempDir::new().unwrap();
        let watcher = FileWatcher::new(temp_dir.path()).unwrap();
        assert!(watcher.try_recv().is_none());
    }

    #[test]
    fn test_file_watcher_recv_timeout() {
        let temp_dir = TempDir::new().unwrap();
        let watcher = FileWatcher::new(temp_dir.path()).unwrap();
        let result = watcher.recv_timeout(Duration::from_millis(10));
        assert!(result.is_none());
    }

    // Note: Tests that actually trigger file events are flaky in CI
    // because file system events are not guaranteed to be delivered
    // immediately or in order. These are tested manually.

    #[test]
    fn test_file_watcher_unwatch() {
        let temp_dir = TempDir::new().unwrap();

        let mut watcher = FileWatcher::new(temp_dir.path()).unwrap();

        // Should be able to unwatch a watched path (may fail if already unwatched during drop)
        // This test just verifies the method exists and can be called
        let _ = watcher.unwatch(temp_dir.path());
    }

    #[test]
    fn test_file_watcher_watch_additional_path() {
        let temp_dir1 = TempDir::new().unwrap();
        let temp_dir2 = TempDir::new().unwrap();

        let mut watcher = FileWatcher::new(temp_dir1.path()).unwrap();

        // Should be able to watch additional paths
        let result = watcher.watch(temp_dir2.path());
        assert!(result.is_ok());
    }
}
