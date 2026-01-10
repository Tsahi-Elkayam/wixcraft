//! File watcher for continuous linting
//!
//! Watches files for changes and re-runs linting automatically.

use notify::{RecommendedWatcher, RecursiveMode};
use notify_debouncer_mini::{new_debouncer, DebouncedEvent, Debouncer};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver};
use std::time::Duration;

/// File watcher for continuous linting
pub struct Watcher {
    /// Debounced watcher
    _debouncer: Debouncer<RecommendedWatcher>,
    /// Event receiver
    receiver: Receiver<Result<Vec<DebouncedEvent>, notify::Error>>,
    /// Watched paths
    paths: Vec<PathBuf>,
    /// File extensions to watch
    extensions: Vec<String>,
}

/// Watch event
#[derive(Debug, Clone)]
pub struct WatchEvent {
    /// Changed file paths
    pub paths: Vec<PathBuf>,
    /// Event kind
    pub kind: WatchEventKind,
}

/// Kind of watch event
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WatchEventKind {
    /// File was modified
    Modified,
    /// File was created
    Created,
    /// File was deleted
    Deleted,
}

impl Watcher {
    /// Create a new watcher for the given paths
    pub fn new(paths: &[PathBuf], extensions: &[&str]) -> Result<Self, notify::Error> {
        let (tx, rx) = mpsc::channel();

        // Create debouncer with 300ms delay
        let mut debouncer = new_debouncer(Duration::from_millis(300), tx)?;

        // Watch all paths
        for path in paths {
            let watch_path = if path.is_file() {
                path.parent().unwrap_or(Path::new("."))
            } else {
                path.as_path()
            };

            debouncer
                .watcher()
                .watch(watch_path, RecursiveMode::Recursive)?;
        }

        Ok(Self {
            _debouncer: debouncer,
            receiver: rx,
            paths: paths.to_vec(),
            extensions: extensions.iter().map(|s| s.to_string()).collect(),
        })
    }

    /// Wait for the next change event
    pub fn wait(&self) -> Option<WatchEvent> {
        match self.receiver.recv() {
            Ok(Ok(events)) => {
                let mut changed_paths = Vec::new();

                for event in events {
                    let path = &event.path;

                    // Filter by extension
                    if !self.extensions.is_empty() {
                        if let Some(ext) = path.extension() {
                            let ext_str = ext.to_string_lossy().to_lowercase();
                            if !self.extensions.iter().any(|e| e.to_lowercase() == ext_str) {
                                continue;
                            }
                        } else {
                            continue;
                        }
                    }

                    // Check if path matches our watched patterns
                    if self.matches_watched_path(path) && !changed_paths.contains(path) {
                        changed_paths.push(path.clone());
                    }
                }

                if changed_paths.is_empty() {
                    None
                } else {
                    Some(WatchEvent {
                        paths: changed_paths,
                        kind: WatchEventKind::Modified,
                    })
                }
            }
            Ok(Err(_)) | Err(_) => None,
        }
    }

    /// Try to get a change event without blocking
    pub fn try_recv(&self) -> Option<WatchEvent> {
        match self.receiver.try_recv() {
            Ok(Ok(events)) => {
                let changed_paths: Vec<PathBuf> = events
                    .into_iter()
                    .filter_map(|e| {
                        let path = e.path;
                        if self.matches_watched_path(&path) && self.matches_extension(&path) {
                            Some(path)
                        } else {
                            None
                        }
                    })
                    .collect();

                if changed_paths.is_empty() {
                    None
                } else {
                    Some(WatchEvent {
                        paths: changed_paths,
                        kind: WatchEventKind::Modified,
                    })
                }
            }
            _ => None,
        }
    }

    /// Check if a path matches our watched paths
    fn matches_watched_path(&self, path: &Path) -> bool {
        for watched in &self.paths {
            if watched.is_file() {
                if path == watched {
                    return true;
                }
            } else if path.starts_with(watched) {
                return true;
            }
        }
        false
    }

    /// Check if path has a watched extension
    fn matches_extension(&self, path: &Path) -> bool {
        if self.extensions.is_empty() {
            return true;
        }

        if let Some(ext) = path.extension() {
            let ext_str = ext.to_string_lossy().to_lowercase();
            self.extensions.iter().any(|e| e.to_lowercase() == ext_str)
        } else {
            false
        }
    }

    /// Get the watched paths
    pub fn watched_paths(&self) -> &[PathBuf] {
        &self.paths
    }

    /// Get the watched extensions
    pub fn watched_extensions(&self) -> &[String] {
        &self.extensions
    }
}

/// Run a function in watch mode
pub fn watch_and_run<F>(
    paths: &[PathBuf],
    extensions: &[&str],
    clear_screen: bool,
    mut callback: F,
) -> Result<(), notify::Error>
where
    F: FnMut(&[PathBuf]),
{
    let watcher = Watcher::new(paths, extensions)?;

    // Initial run
    if clear_screen {
        print!("\x1B[2J\x1B[1;1H"); // Clear screen
    }
    callback(paths);

    // Watch loop
    loop {
        if let Some(event) = watcher.wait() {
            if clear_screen {
                print!("\x1B[2J\x1B[1;1H"); // Clear screen
            }
            callback(&event.paths);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_watcher_creation() {
        let temp = TempDir::new().unwrap();
        let watcher = Watcher::new(&[temp.path().to_path_buf()], &["wxs"]);
        assert!(watcher.is_ok());
    }

    #[test]
    fn test_extension_filter() {
        let temp = TempDir::new().unwrap();
        let watcher = Watcher::new(&[temp.path().to_path_buf()], &["wxs", "xml"]).unwrap();

        assert!(watcher.matches_extension(Path::new("test.wxs")));
        assert!(watcher.matches_extension(Path::new("test.xml")));
        assert!(watcher.matches_extension(Path::new("test.WXS"))); // Case insensitive
        assert!(!watcher.matches_extension(Path::new("test.txt")));
    }

    #[test]
    fn test_empty_extension_filter() {
        let temp = TempDir::new().unwrap();
        let watcher = Watcher::new(&[temp.path().to_path_buf()], &[]).unwrap();

        // Empty filter means all files match
        assert!(watcher.matches_extension(Path::new("test.wxs")));
        assert!(watcher.matches_extension(Path::new("test.txt")));
    }

    #[test]
    fn test_path_matching() {
        let temp = TempDir::new().unwrap();
        let watched_file = temp.path().join("test.wxs");
        fs::write(&watched_file, "<Wix/>").unwrap();

        let watcher = Watcher::new(&[watched_file.clone()], &["wxs"]).unwrap();

        assert!(watcher.matches_watched_path(&watched_file));
        assert!(!watcher.matches_watched_path(Path::new("/other/file.wxs")));
    }

    #[test]
    fn test_directory_matching() {
        let temp = TempDir::new().unwrap();
        let subfile = temp.path().join("subdir/test.wxs");

        let watcher = Watcher::new(&[temp.path().to_path_buf()], &["wxs"]).unwrap();

        assert!(watcher.matches_watched_path(&subfile));
    }
}
