//! Caching system for incremental linting
//!
//! Caches lint results by file content hash to dramatically speed up
//! subsequent runs when files haven't changed.

use crate::diagnostic::Diagnostic;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

/// Cache entry for a single file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry {
    /// Content hash of the file
    pub content_hash: String,
    /// Last modified time (Unix timestamp)
    pub mtime: u64,
    /// File size in bytes
    pub size: u64,
    /// Cached diagnostics
    pub diagnostics: Vec<CachedDiagnostic>,
    /// Rules that were applied (for invalidation if rules change)
    pub rule_versions: HashMap<String, String>,
}

/// Simplified diagnostic for caching
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedDiagnostic {
    pub rule_id: String,
    pub message: String,
    pub severity: String,
    pub line: usize,
    pub column: usize,
    pub source_line: Option<String>,
}

impl CachedDiagnostic {
    /// Convert from a full Diagnostic
    pub fn from_diagnostic(diag: &Diagnostic) -> Self {
        Self {
            rule_id: diag.rule_id.clone(),
            message: diag.message.clone(),
            severity: format!("{:?}", diag.severity),
            line: diag.location.line,
            column: diag.location.column,
            source_line: diag.source_line.clone(),
        }
    }

    /// Convert back to a full Diagnostic
    pub fn to_diagnostic(&self, file: PathBuf) -> Diagnostic {
        use crate::diagnostic::{Location, Severity};

        Diagnostic {
            rule_id: self.rule_id.clone(),
            message: self.message.clone(),
            severity: match self.severity.to_lowercase().as_str() {
                "error" => Severity::Error,
                "warning" => Severity::Warning,
                _ => Severity::Info,
            },
            location: Location::new(file, self.line, self.column),
            source_line: self.source_line.clone(),
            context_before: vec![],
            context_after: vec![],
            help: None,
            fix: None,
            notes: vec![],
        }
    }
}

/// Lint cache for incremental runs
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct LintCache {
    /// Cache version (bump to invalidate all caches)
    pub version: u32,
    /// Entries by file path
    pub entries: HashMap<String, CacheEntry>,
    /// Configuration hash (invalidate if config changes)
    #[serde(default)]
    pub config_hash: String,
}

impl LintCache {
    /// Current cache format version
    const CACHE_VERSION: u32 = 1;

    /// Create a new empty cache
    pub fn new() -> Self {
        Self {
            version: Self::CACHE_VERSION,
            entries: HashMap::new(),
            config_hash: String::new(),
        }
    }

    /// Load cache from file
    pub fn load(path: &Path) -> Result<Self, std::io::Error> {
        let file = fs::File::open(path)?;
        let mut reader = std::io::BufReader::new(file);
        let mut content = Vec::new();
        reader.read_to_end(&mut content)?;

        let cache: Self = serde_json::from_slice(&content)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

        // Invalidate if version mismatch
        if cache.version != Self::CACHE_VERSION {
            return Ok(Self::new());
        }

        Ok(cache)
    }

    /// Save cache to file
    pub fn save(&self, path: &Path) -> Result<(), std::io::Error> {
        // Create parent directories if needed
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let content = serde_json::to_vec(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

        let mut file = fs::File::create(path)?;
        file.write_all(&content)?;
        Ok(())
    }

    /// Set config hash (call before using cache)
    pub fn set_config_hash(&mut self, hash: &str) {
        if self.config_hash != hash {
            // Config changed, invalidate all entries
            self.entries.clear();
            self.config_hash = hash.to_string();
        }
    }

    /// Check if a file is cached and still valid
    pub fn get(&self, file: &Path) -> Option<&CacheEntry> {
        let key = file.to_string_lossy().to_string();
        let entry = self.entries.get(&key)?;

        // Validate entry is still fresh
        if let Ok(metadata) = fs::metadata(file) {
            let mtime = metadata
                .modified()
                .ok()
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|d| d.as_secs())
                .unwrap_or(0);

            let size = metadata.len();

            // Quick check: mtime and size
            if entry.mtime == mtime && entry.size == size {
                return Some(entry);
            }

            // Slower check: content hash
            if let Ok(content_hash) = hash_file(file) {
                if entry.content_hash == content_hash {
                    return Some(entry);
                }
            }
        }

        None
    }

    /// Store diagnostics for a file
    pub fn put(&mut self, file: &Path, diagnostics: &[Diagnostic], rule_versions: HashMap<String, String>) {
        let key = file.to_string_lossy().to_string();

        // Get file metadata
        let (mtime, size) = fs::metadata(file)
            .map(|m| {
                let mtime = m
                    .modified()
                    .ok()
                    .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                    .map(|d| d.as_secs())
                    .unwrap_or(0);
                (mtime, m.len())
            })
            .unwrap_or((0, 0));

        // Hash file content
        let content_hash = hash_file(file).unwrap_or_default();

        let entry = CacheEntry {
            content_hash,
            mtime,
            size,
            diagnostics: diagnostics.iter().map(CachedDiagnostic::from_diagnostic).collect(),
            rule_versions,
        };

        self.entries.insert(key, entry);
    }

    /// Remove entries for files that no longer exist
    pub fn prune(&mut self) {
        self.entries.retain(|path, _| Path::new(path).exists());
    }

    /// Get cache statistics
    pub fn stats(&self) -> CacheStats {
        CacheStats {
            total_entries: self.entries.len(),
            total_diagnostics: self.entries.values().map(|e| e.diagnostics.len()).sum(),
        }
    }

    /// Clear all entries
    pub fn clear(&mut self) {
        self.entries.clear();
    }
}

/// Cache statistics
#[derive(Debug)]
pub struct CacheStats {
    pub total_entries: usize,
    pub total_diagnostics: usize,
}

/// Hash a file's content
fn hash_file(path: &Path) -> Result<String, std::io::Error> {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let content = fs::read(path)?;
    let mut hasher = DefaultHasher::new();
    content.hash(&mut hasher);
    Ok(format!("{:x}", hasher.finish()))
}

/// Hash a config for cache invalidation
pub fn hash_config(config: &impl serde::Serialize) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let json = serde_json::to_string(config).unwrap_or_default();
    let mut hasher = DefaultHasher::new();
    json.hash(&mut hasher);
    format!("{:x}", hasher.finish())
}

/// Default cache file location
pub fn default_cache_path() -> PathBuf {
    dirs::cache_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("winter")
        .join("cache.json")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagnostic::{Location, Severity};
    use tempfile::TempDir;

    fn make_diagnostic(rule: &str, file: PathBuf, line: usize) -> Diagnostic {
        Diagnostic {
            rule_id: rule.to_string(),
            message: "test message".to_string(),
            severity: Severity::Warning,
            location: Location::new(file, line, 0),
            source_line: None,
            context_before: vec![],
            context_after: vec![],
            help: None,
            fix: None,
            notes: vec![],
        }
    }

    #[test]
    fn test_cache_new() {
        let cache = LintCache::new();
        assert_eq!(cache.version, LintCache::CACHE_VERSION);
        assert!(cache.entries.is_empty());
    }

    #[test]
    fn test_cache_put_get() {
        let temp = TempDir::new().unwrap();
        let test_file = temp.path().join("test.wxs");
        fs::write(&test_file, "<Wix/>").unwrap();

        let mut cache = LintCache::new();
        let diags = vec![make_diagnostic("rule1", test_file.clone(), 1)];

        cache.put(&test_file, &diags, HashMap::new());

        let entry = cache.get(&test_file);
        assert!(entry.is_some());
        assert_eq!(entry.unwrap().diagnostics.len(), 1);
    }

    #[test]
    fn test_cache_invalidation_on_change() {
        let temp = TempDir::new().unwrap();
        let test_file = temp.path().join("test.wxs");
        fs::write(&test_file, "<Wix/>").unwrap();

        let mut cache = LintCache::new();
        cache.put(&test_file, &[], HashMap::new());

        // Modify file
        fs::write(&test_file, "<Wix><Component/></Wix>").unwrap();

        // Should not find cached entry
        let entry = cache.get(&test_file);
        assert!(entry.is_none());
    }

    #[test]
    fn test_cache_save_load() {
        let temp = TempDir::new().unwrap();
        let cache_file = temp.path().join("cache.json");
        let test_file = temp.path().join("test.wxs");
        fs::write(&test_file, "<Wix/>").unwrap();

        let mut cache = LintCache::new();
        cache.put(&test_file, &[], HashMap::new());
        cache.save(&cache_file).unwrap();

        let loaded = LintCache::load(&cache_file).unwrap();
        assert_eq!(loaded.entries.len(), 1);
    }

    #[test]
    fn test_cache_prune() {
        let temp = TempDir::new().unwrap();
        let test_file = temp.path().join("test.wxs");
        fs::write(&test_file, "<Wix/>").unwrap();

        let mut cache = LintCache::new();
        cache.put(&test_file, &[], HashMap::new());

        // Delete the file
        fs::remove_file(&test_file).unwrap();

        cache.prune();
        assert!(cache.entries.is_empty());
    }

    #[test]
    fn test_cache_config_invalidation() {
        let mut cache = LintCache::new();
        // Set initial config hash
        cache.set_config_hash("hash1");

        cache.entries.insert("test".to_string(), CacheEntry {
            content_hash: "abc".to_string(),
            mtime: 0,
            size: 0,
            diagnostics: vec![],
            rule_versions: HashMap::new(),
        });

        // Same config hash - entries preserved
        cache.set_config_hash("hash1");
        assert!(!cache.entries.is_empty());

        // Different config hash - entries cleared
        cache.set_config_hash("hash2");
        assert!(cache.entries.is_empty());
    }

    #[test]
    fn test_cached_diagnostic_conversion() {
        let file = PathBuf::from("test.wxs");
        let diag = make_diagnostic("test-rule", file.clone(), 42);

        let cached = CachedDiagnostic::from_diagnostic(&diag);
        assert_eq!(cached.rule_id, "test-rule");
        assert_eq!(cached.line, 42);

        let restored = cached.to_diagnostic(file);
        assert_eq!(restored.rule_id, "test-rule");
        assert_eq!(restored.location.line, 42);
    }

    #[test]
    fn test_stats() {
        let mut cache = LintCache::new();
        cache.entries.insert("file1".to_string(), CacheEntry {
            content_hash: "a".to_string(),
            mtime: 0,
            size: 0,
            diagnostics: vec![
                CachedDiagnostic {
                    rule_id: "r1".to_string(),
                    message: "m".to_string(),
                    severity: "warning".to_string(),
                    line: 1,
                    column: 0,
                    source_line: None,
                },
            ],
            rule_versions: HashMap::new(),
        });
        cache.entries.insert("file2".to_string(), CacheEntry {
            content_hash: "b".to_string(),
            mtime: 0,
            size: 0,
            diagnostics: vec![
                CachedDiagnostic {
                    rule_id: "r1".to_string(),
                    message: "m".to_string(),
                    severity: "warning".to_string(),
                    line: 1,
                    column: 0,
                    source_line: None,
                },
                CachedDiagnostic {
                    rule_id: "r2".to_string(),
                    message: "m".to_string(),
                    severity: "error".to_string(),
                    line: 2,
                    column: 0,
                    source_line: None,
                },
            ],
            rule_versions: HashMap::new(),
        });

        let stats = cache.stats();
        assert_eq!(stats.total_entries, 2);
        assert_eq!(stats.total_diagnostics, 3);
    }

    #[test]
    fn test_hash_config() {
        #[derive(serde::Serialize)]
        struct TestConfig {
            value: i32,
        }

        let hash1 = hash_config(&TestConfig { value: 1 });
        let hash2 = hash_config(&TestConfig { value: 1 });
        let hash3 = hash_config(&TestConfig { value: 2 });

        assert_eq!(hash1, hash2);
        assert_ne!(hash1, hash3);
    }
}
