//! Incremental analysis with caching
//!
//! Caches analysis results and only re-analyzes files that have changed.
//! Uses file content hashes to detect changes.
//!
//! # Cache Structure
//!
//! ```text
//! .wixanalyzer-cache/
//! ├── index.json          # Cache index with file hashes
//! └── results/
//!     ├── abc123.json     # Cached results by content hash
//!     └── def456.json
//! ```
//!
//! # Usage
//!
//! ```rust,ignore
//! let mut cache = AnalysisCache::new(".wixanalyzer-cache")?;
//!
//! if let Some(cached) = cache.get(&file_path) {
//!     // Use cached results
//! } else {
//!     let result = analyze(&doc);
//!     cache.put(&file_path, &source, &result)?;
//! }
//!
//! cache.save()?;
//! ```

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use crate::core::AnalysisResult;

/// Default cache directory name
pub const CACHE_DIR_NAME: &str = ".wixanalyzer-cache";

/// Cache version (bump when cache format changes)
const CACHE_VERSION: u32 = 1;

/// Maximum cache age in days before auto-cleanup
const MAX_CACHE_AGE_DAYS: u64 = 7;

/// Cache index entry
#[derive(Debug, Clone, Serialize, Deserialize)]
struct CacheEntry {
    /// Content hash of the file
    content_hash: String,
    /// Last analyzed timestamp
    analyzed_at: u64,
    /// Tool version that analyzed it
    tool_version: String,
    /// Path to cached result file
    result_file: String,
}

/// Cache index
#[derive(Debug, Clone, Serialize, Deserialize)]
struct CacheIndex {
    /// Cache format version
    version: u32,
    /// Cache creation time
    created_at: u64,
    /// Entries by file path
    entries: HashMap<String, CacheEntry>,
}

impl Default for CacheIndex {
    fn default() -> Self {
        Self {
            version: CACHE_VERSION,
            created_at: current_timestamp(),
            entries: HashMap::new(),
        }
    }
}

/// Analysis cache manager
#[derive(Debug)]
pub struct AnalysisCache {
    /// Cache directory path
    cache_dir: PathBuf,
    /// Index of cached files
    index: CacheIndex,
    /// Whether index has been modified
    dirty: bool,
    /// Statistics
    stats: CacheStats,
}

/// Cache statistics
#[derive(Debug, Clone, Default)]
pub struct CacheStats {
    /// Number of cache hits
    pub hits: usize,
    /// Number of cache misses
    pub misses: usize,
    /// Files analyzed (not from cache)
    pub analyzed: usize,
    /// Files skipped (from cache)
    pub skipped: usize,
    /// Bytes saved by caching
    pub bytes_saved: usize,
}

impl CacheStats {
    /// Calculate hit rate percentage
    pub fn hit_rate(&self) -> f64 {
        let total = self.hits + self.misses;
        if total == 0 {
            0.0
        } else {
            (self.hits as f64 / total as f64) * 100.0
        }
    }
}

impl AnalysisCache {
    /// Create or load cache from directory
    pub fn new(cache_dir: impl Into<PathBuf>) -> Result<Self, CacheError> {
        let cache_dir = cache_dir.into();

        // Create cache directory structure
        fs::create_dir_all(&cache_dir)
            .map_err(|e| CacheError::IoError(cache_dir.clone(), e.to_string()))?;
        fs::create_dir_all(cache_dir.join("results"))
            .map_err(|e| CacheError::IoError(cache_dir.clone(), e.to_string()))?;

        // Load or create index
        let index_path = cache_dir.join("index.json");
        let index = if index_path.exists() {
            let content = fs::read_to_string(&index_path)
                .map_err(|e| CacheError::IoError(index_path.clone(), e.to_string()))?;

            let index: CacheIndex = serde_json::from_str(&content)
                .map_err(|e| CacheError::ParseError(index_path.clone(), e.to_string()))?;

            // Check version compatibility
            if index.version != CACHE_VERSION {
                // Incompatible version, start fresh
                CacheIndex::default()
            } else {
                index
            }
        } else {
            CacheIndex::default()
        };

        Ok(Self {
            cache_dir,
            index,
            dirty: false,
            stats: CacheStats::default(),
        })
    }

    /// Create cache in default location relative to a directory
    pub fn in_directory(dir: impl AsRef<Path>) -> Result<Self, CacheError> {
        Self::new(dir.as_ref().join(CACHE_DIR_NAME))
    }

    /// Get cached result for a file
    pub fn get(&mut self, file_path: &Path, source: &str) -> Option<AnalysisResult> {
        let key = self.path_key(file_path);
        let content_hash = self.hash_content(source);

        if let Some(entry) = self.index.entries.get(&key) {
            // Check if hash matches
            if entry.content_hash == content_hash && entry.tool_version == env!("CARGO_PKG_VERSION")
            {
                // Load cached result
                let result_path = self.cache_dir.join("results").join(&entry.result_file);
                if let Ok(content) = fs::read_to_string(&result_path) {
                    if let Ok(result) = serde_json::from_str(&content) {
                        self.stats.hits += 1;
                        self.stats.skipped += 1;
                        self.stats.bytes_saved += source.len();
                        return Some(result);
                    }
                }
            }
        }

        self.stats.misses += 1;
        None
    }

    /// Store result in cache
    pub fn put(
        &mut self,
        file_path: &Path,
        source: &str,
        result: &AnalysisResult,
    ) -> Result<(), CacheError> {
        let key = self.path_key(file_path);
        let content_hash = self.hash_content(source);
        let result_file = format!("{}.json", &content_hash[..16]);

        // Write result file
        let result_path = self.cache_dir.join("results").join(&result_file);
        let content =
            serde_json::to_string(result).map_err(|e| CacheError::SerializeError(e.to_string()))?;
        fs::write(&result_path, content)
            .map_err(|e| CacheError::IoError(result_path, e.to_string()))?;

        // Update index
        self.index.entries.insert(
            key,
            CacheEntry {
                content_hash,
                analyzed_at: current_timestamp(),
                tool_version: env!("CARGO_PKG_VERSION").to_string(),
                result_file,
            },
        );

        self.dirty = true;
        self.stats.analyzed += 1;

        Ok(())
    }

    /// Check if a file needs re-analysis
    pub fn needs_analysis(&self, file_path: &Path, source: &str) -> bool {
        let key = self.path_key(file_path);
        let content_hash = self.hash_content(source);

        match self.index.entries.get(&key) {
            Some(entry) => {
                entry.content_hash != content_hash
                    || entry.tool_version != env!("CARGO_PKG_VERSION")
            }
            None => true,
        }
    }

    /// Save cache index to disk
    pub fn save(&mut self) -> Result<(), CacheError> {
        if !self.dirty {
            return Ok(());
        }

        let index_path = self.cache_dir.join("index.json");
        let content = serde_json::to_string_pretty(&self.index)
            .map_err(|e| CacheError::SerializeError(e.to_string()))?;

        fs::write(&index_path, content)
            .map_err(|e| CacheError::IoError(index_path, e.to_string()))?;

        self.dirty = false;
        Ok(())
    }

    /// Get cache statistics
    pub fn stats(&self) -> &CacheStats {
        &self.stats
    }

    /// Clear all cached data
    pub fn clear(&mut self) -> Result<(), CacheError> {
        // Remove all result files
        let results_dir = self.cache_dir.join("results");
        if results_dir.exists() {
            fs::remove_dir_all(&results_dir)
                .map_err(|e| CacheError::IoError(results_dir.clone(), e.to_string()))?;
            fs::create_dir_all(&results_dir)
                .map_err(|e| CacheError::IoError(results_dir, e.to_string()))?;
        }

        // Reset index
        self.index = CacheIndex::default();
        self.dirty = true;

        Ok(())
    }

    /// Clean up stale cache entries
    pub fn cleanup(&mut self) -> Result<usize, CacheError> {
        let now = current_timestamp();
        let max_age = MAX_CACHE_AGE_DAYS * 24 * 60 * 60;
        let mut removed = 0;

        // Find stale entries
        let stale_keys: Vec<String> = self
            .index
            .entries
            .iter()
            .filter(|(_, entry)| now - entry.analyzed_at > max_age)
            .map(|(key, _)| key.clone())
            .collect();

        // Remove stale entries and their result files
        for key in stale_keys {
            if let Some(entry) = self.index.entries.remove(&key) {
                let result_path = self.cache_dir.join("results").join(&entry.result_file);
                let _ = fs::remove_file(&result_path);
                removed += 1;
            }
        }

        if removed > 0 {
            self.dirty = true;
        }

        Ok(removed)
    }

    /// Invalidate cache for a specific file
    pub fn invalidate(&mut self, file_path: &Path) {
        let key = self.path_key(file_path);
        if let Some(entry) = self.index.entries.remove(&key) {
            let result_path = self.cache_dir.join("results").join(&entry.result_file);
            let _ = fs::remove_file(&result_path);
            self.dirty = true;
        }
    }

    /// Get number of cached entries
    pub fn entry_count(&self) -> usize {
        self.index.entries.len()
    }

    /// Hash file content
    fn hash_content(&self, content: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// Convert file path to cache key
    fn path_key(&self, path: &Path) -> String {
        path.to_string_lossy().replace(['/', '\\'], "_")
    }
}

/// Get current timestamp in seconds
fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// Cache errors
#[derive(Debug)]
pub enum CacheError {
    IoError(PathBuf, String),
    ParseError(PathBuf, String),
    SerializeError(String),
}

impl std::fmt::Display for CacheError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IoError(path, msg) => {
                write!(f, "Cache I/O error at '{}': {}", path.display(), msg)
            }
            Self::ParseError(path, msg) => {
                write!(f, "Cache parse error at '{}': {}", path.display(), msg)
            }
            Self::SerializeError(msg) => write!(f, "Cache serialization error: {}", msg),
        }
    }
}

impl std::error::Error for CacheError {}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn make_result() -> AnalysisResult {
        use crate::core::{Category, Diagnostic, Location, Position, Range};

        let mut result = AnalysisResult::new();
        result.add(Diagnostic::error(
            "VAL-001",
            Category::Validation,
            "Test error",
            Location::new(
                PathBuf::from("test.wxs"),
                Range::new(Position::new(1, 1), Position::new(1, 10)),
            ),
        ));
        result
    }

    #[test]
    fn test_cache_new() {
        let temp_dir = TempDir::new().unwrap();
        let cache = AnalysisCache::new(temp_dir.path().join("cache")).unwrap();

        assert_eq!(cache.entry_count(), 0);
        assert_eq!(cache.stats().hits, 0);
    }

    #[test]
    fn test_cache_put_and_get() {
        let temp_dir = TempDir::new().unwrap();
        let mut cache = AnalysisCache::new(temp_dir.path().join("cache")).unwrap();

        let file_path = PathBuf::from("test.wxs");
        let source = "<Wix><Package /></Wix>";
        let result = make_result();

        // Put
        cache.put(&file_path, source, &result).unwrap();
        assert_eq!(cache.entry_count(), 1);

        // Get
        let cached = cache.get(&file_path, source);
        assert!(cached.is_some());
        assert_eq!(cached.unwrap().len(), 1);

        assert_eq!(cache.stats().hits, 1);
    }

    #[test]
    fn test_cache_miss_on_changed_content() {
        let temp_dir = TempDir::new().unwrap();
        let mut cache = AnalysisCache::new(temp_dir.path().join("cache")).unwrap();

        let file_path = PathBuf::from("test.wxs");
        let source1 = "<Wix><Package /></Wix>";
        let source2 = "<Wix><Package Name=\"Changed\" /></Wix>";
        let result = make_result();

        // Put with original source
        cache.put(&file_path, source1, &result).unwrap();

        // Get with different source (should miss)
        let cached = cache.get(&file_path, source2);
        assert!(cached.is_none());
        assert_eq!(cache.stats().misses, 1);
    }

    #[test]
    fn test_cache_needs_analysis() {
        let temp_dir = TempDir::new().unwrap();
        let mut cache = AnalysisCache::new(temp_dir.path().join("cache")).unwrap();

        let file_path = PathBuf::from("test.wxs");
        let source = "<Wix />";
        let result = make_result();

        // Before caching
        assert!(cache.needs_analysis(&file_path, source));

        // After caching
        cache.put(&file_path, source, &result).unwrap();
        assert!(!cache.needs_analysis(&file_path, source));

        // With changed content
        assert!(cache.needs_analysis(&file_path, "<Wix><Changed /></Wix>"));
    }

    #[test]
    fn test_cache_save_and_load() {
        let temp_dir = TempDir::new().unwrap();
        let cache_dir = temp_dir.path().join("cache");

        // Create and populate cache
        {
            let mut cache = AnalysisCache::new(&cache_dir).unwrap();
            let result = make_result();
            cache
                .put(Path::new("test.wxs"), "<Wix />", &result)
                .unwrap();
            cache.save().unwrap();
        }

        // Load cache
        {
            let mut cache = AnalysisCache::new(&cache_dir).unwrap();
            assert_eq!(cache.entry_count(), 1);

            let cached = cache.get(Path::new("test.wxs"), "<Wix />");
            assert!(cached.is_some());
        }
    }

    #[test]
    fn test_cache_clear() {
        let temp_dir = TempDir::new().unwrap();
        let mut cache = AnalysisCache::new(temp_dir.path().join("cache")).unwrap();

        let result = make_result();
        cache
            .put(Path::new("test.wxs"), "<Wix />", &result)
            .unwrap();
        assert_eq!(cache.entry_count(), 1);

        cache.clear().unwrap();
        assert_eq!(cache.entry_count(), 0);
    }

    #[test]
    fn test_cache_invalidate() {
        let temp_dir = TempDir::new().unwrap();
        let mut cache = AnalysisCache::new(temp_dir.path().join("cache")).unwrap();

        let file_path = PathBuf::from("test.wxs");
        let result = make_result();
        cache.put(&file_path, "<Wix />", &result).unwrap();

        assert_eq!(cache.entry_count(), 1);

        cache.invalidate(&file_path);
        assert_eq!(cache.entry_count(), 0);
    }

    #[test]
    fn test_cache_stats_hit_rate() {
        let mut stats = CacheStats::default();
        assert_eq!(stats.hit_rate(), 0.0);

        stats.hits = 7;
        stats.misses = 3;
        assert!((stats.hit_rate() - 70.0).abs() < 0.001);
    }

    #[test]
    fn test_cache_in_directory() {
        let temp_dir = TempDir::new().unwrap();
        let cache = AnalysisCache::in_directory(temp_dir.path()).unwrap();

        assert!(temp_dir.path().join(CACHE_DIR_NAME).exists());
        assert_eq!(cache.entry_count(), 0);
    }

    #[test]
    fn test_cache_error_display() {
        let err = CacheError::IoError(PathBuf::from("test"), "not found".to_string());
        assert!(err.to_string().contains("I/O error"));
        assert!(err.to_string().contains("test"));
    }
}
