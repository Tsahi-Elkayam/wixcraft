//! Baseline file support for wix-analyzer
//!
//! Baselines allow tracking and ignoring existing issues in legacy code.
//! New issues are still reported, but issues that match the baseline are filtered out.
//!
//! # Baseline File Format
//!
//! ```json
//! {
//!   "version": 1,
//!   "created": "2024-01-15T10:30:00Z",
//!   "tool_version": "0.1.0",
//!   "issues": [
//!     {
//!       "fingerprint": "abc123...",
//!       "rule_id": "SEC-001",
//!       "file": "src/product.wxs",
//!       "line": 42,
//!       "message_hash": "def456..."
//!     }
//!   ]
//! }
//! ```

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashSet;
use std::fs;
use std::path::Path;

use crate::core::{AnalysisResult, Diagnostic};

/// Baseline file format version
const BASELINE_VERSION: u32 = 1;

/// Default baseline file name
pub const BASELINE_FILE_NAME: &str = ".wixanalyzer-baseline.json";

/// A baseline entry representing a known issue
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct BaselineEntry {
    /// Unique fingerprint for this issue
    pub fingerprint: String,
    /// Rule ID
    pub rule_id: String,
    /// Relative file path
    pub file: String,
    /// Line number (for reference only, not used in matching)
    pub line: usize,
    /// Hash of the message (for detecting rule changes)
    pub message_hash: String,
}

impl BaselineEntry {
    /// Create a baseline entry from a diagnostic
    pub fn from_diagnostic(diag: &Diagnostic, base_path: Option<&Path>) -> Self {
        let file = match base_path {
            Some(base) => diag.location.file
                .strip_prefix(base)
                .unwrap_or(&diag.location.file)
                .to_string_lossy()
                .to_string(),
            None => diag.location.file.to_string_lossy().to_string(),
        };

        let fingerprint = Self::compute_fingerprint(
            &diag.rule_id,
            &file,
            diag.location.range.start.line,
            &diag.message,
        );

        let message_hash = Self::hash_message(&diag.message);

        Self {
            fingerprint,
            rule_id: diag.rule_id.clone(),
            file,
            line: diag.location.range.start.line,
            message_hash,
        }
    }

    /// Compute a fingerprint for matching issues
    /// Uses rule_id + normalized file path + line region (not exact line)
    fn compute_fingerprint(rule_id: &str, file: &str, line: usize, message: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(rule_id.as_bytes());
        hasher.update(b"|");
        // Normalize path separators
        hasher.update(file.replace('\\', "/").as_bytes());
        hasher.update(b"|");
        // Use line region (group of 5 lines) to handle minor line shifts
        let line_region = line / 5;
        hasher.update(line_region.to_string().as_bytes());
        hasher.update(b"|");
        // Include first 50 chars of message for semantic matching
        let msg_prefix: String = message.chars().take(50).collect();
        hasher.update(msg_prefix.as_bytes());

        format!("{:x}", hasher.finalize())
    }

    fn hash_message(message: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(message.as_bytes());
        format!("{:x}", hasher.finalize())[..16].to_string()
    }
}

/// Baseline file structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Baseline {
    /// Format version
    pub version: u32,
    /// Creation timestamp (ISO 8601)
    pub created: String,
    /// Tool version that created this baseline
    pub tool_version: String,
    /// Description (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Baseline entries
    pub issues: Vec<BaselineEntry>,
}

impl Default for Baseline {
    fn default() -> Self {
        Self::new()
    }
}

impl Baseline {
    /// Create a new empty baseline
    pub fn new() -> Self {
        Self {
            version: BASELINE_VERSION,
            created: chrono::Utc::now().to_rfc3339(),
            tool_version: env!("CARGO_PKG_VERSION").to_string(),
            description: None,
            issues: Vec::new(),
        }
    }

    /// Create baseline from analysis results
    pub fn from_results(results: &[AnalysisResult], base_path: Option<&Path>) -> Self {
        let mut baseline = Self::new();

        for result in results {
            for diag in &result.diagnostics {
                baseline.issues.push(BaselineEntry::from_diagnostic(diag, base_path));
            }
        }

        baseline
    }

    /// Load baseline from file
    pub fn load(path: &Path) -> Result<Self, BaselineError> {
        let content = fs::read_to_string(path)
            .map_err(|e| BaselineError::ReadError(path.to_path_buf(), e.to_string()))?;

        let baseline: Self = serde_json::from_str(&content)
            .map_err(|e| BaselineError::ParseError(path.to_path_buf(), e.to_string()))?;

        if baseline.version > BASELINE_VERSION {
            return Err(BaselineError::UnsupportedVersion(baseline.version));
        }

        Ok(baseline)
    }

    /// Find and load baseline from directory or parents
    pub fn find_and_load(start_dir: &Path) -> Option<Self> {
        let mut current = start_dir.to_path_buf();

        loop {
            let baseline_path = current.join(BASELINE_FILE_NAME);
            if baseline_path.exists() {
                return Self::load(&baseline_path).ok();
            }

            if !current.pop() {
                break;
            }
        }

        None
    }

    /// Save baseline to file
    pub fn save(&self, path: &Path) -> Result<(), BaselineError> {
        let content = serde_json::to_string_pretty(self)
            .map_err(|e| BaselineError::SerializeError(e.to_string()))?;

        fs::write(path, content)
            .map_err(|e| BaselineError::WriteError(path.to_path_buf(), e.to_string()))?;

        Ok(())
    }

    /// Set description
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// Check if an issue is in the baseline
    pub fn contains(&self, diag: &Diagnostic, base_path: Option<&Path>) -> bool {
        let entry = BaselineEntry::from_diagnostic(diag, base_path);
        self.fingerprints().contains(&entry.fingerprint)
    }

    /// Get all fingerprints as a set for fast lookup
    pub fn fingerprints(&self) -> HashSet<String> {
        self.issues.iter().map(|e| e.fingerprint.clone()).collect()
    }

    /// Number of issues in baseline
    pub fn len(&self) -> usize {
        self.issues.len()
    }

    /// Check if baseline is empty
    pub fn is_empty(&self) -> bool {
        self.issues.is_empty()
    }

    /// Get statistics about the baseline
    pub fn stats(&self) -> BaselineStats {
        use std::collections::HashMap;

        let mut by_rule: HashMap<String, usize> = HashMap::new();
        let mut by_file: HashMap<String, usize> = HashMap::new();

        for entry in &self.issues {
            *by_rule.entry(entry.rule_id.clone()).or_default() += 1;
            *by_file.entry(entry.file.clone()).or_default() += 1;
        }

        BaselineStats {
            total_issues: self.issues.len(),
            unique_rules: by_rule.len(),
            unique_files: by_file.len(),
            by_rule,
            by_file,
        }
    }
}

/// Baseline statistics
#[derive(Debug, Clone)]
pub struct BaselineStats {
    pub total_issues: usize,
    pub unique_rules: usize,
    pub unique_files: usize,
    pub by_rule: std::collections::HashMap<String, usize>,
    pub by_file: std::collections::HashMap<String, usize>,
}

/// Baseline error types
#[derive(Debug)]
pub enum BaselineError {
    ReadError(std::path::PathBuf, String),
    WriteError(std::path::PathBuf, String),
    ParseError(std::path::PathBuf, String),
    SerializeError(String),
    UnsupportedVersion(u32),
}

impl std::fmt::Display for BaselineError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ReadError(path, msg) => {
                write!(f, "Failed to read baseline '{}': {}", path.display(), msg)
            }
            Self::WriteError(path, msg) => {
                write!(f, "Failed to write baseline '{}': {}", path.display(), msg)
            }
            Self::ParseError(path, msg) => {
                write!(f, "Failed to parse baseline '{}': {}", path.display(), msg)
            }
            Self::SerializeError(msg) => {
                write!(f, "Failed to serialize baseline: {}", msg)
            }
            Self::UnsupportedVersion(v) => {
                write!(f, "Unsupported baseline version: {} (max supported: {})", v, BASELINE_VERSION)
            }
        }
    }
}

impl std::error::Error for BaselineError {}

/// Filter results against a baseline
pub fn filter_baseline(
    results: &mut Vec<AnalysisResult>,
    baseline: &Baseline,
    base_path: Option<&Path>,
) -> usize {
    let fingerprints = baseline.fingerprints();
    let mut filtered_count = 0;

    for result in results.iter_mut() {
        let original_len = result.diagnostics.len();

        result.diagnostics.retain(|diag| {
            let entry = BaselineEntry::from_diagnostic(diag, base_path);
            !fingerprints.contains(&entry.fingerprint)
        });

        filtered_count += original_len - result.diagnostics.len();
    }

    filtered_count
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{Category, Location, Position, Range};
    use std::path::PathBuf;

    fn make_diagnostic(rule_id: &str, file: &str, line: usize, message: &str) -> Diagnostic {
        Diagnostic::error(
            rule_id,
            Category::Security,
            message,
            Location::new(
                PathBuf::from(file),
                Range::new(Position::new(line, 1), Position::new(line, 10)),
            ),
        )
    }

    #[test]
    fn test_baseline_entry_from_diagnostic() {
        let diag = make_diagnostic("SEC-001", "src/test.wxs", 42, "Test message");
        let entry = BaselineEntry::from_diagnostic(&diag, None);

        assert_eq!(entry.rule_id, "SEC-001");
        assert_eq!(entry.file, "src/test.wxs");
        assert_eq!(entry.line, 42);
        assert!(!entry.fingerprint.is_empty());
        assert!(!entry.message_hash.is_empty());
    }

    #[test]
    fn test_baseline_entry_with_base_path() {
        let diag = make_diagnostic("SEC-001", "/project/src/test.wxs", 42, "Test message");
        let entry = BaselineEntry::from_diagnostic(&diag, Some(Path::new("/project")));

        assert_eq!(entry.file, "src/test.wxs");
    }

    #[test]
    fn test_fingerprint_stability() {
        let diag1 = make_diagnostic("SEC-001", "src/test.wxs", 42, "Test message");
        let diag2 = make_diagnostic("SEC-001", "src/test.wxs", 42, "Test message");

        let entry1 = BaselineEntry::from_diagnostic(&diag1, None);
        let entry2 = BaselineEntry::from_diagnostic(&diag2, None);

        assert_eq!(entry1.fingerprint, entry2.fingerprint);
    }

    #[test]
    fn test_fingerprint_line_tolerance() {
        // Lines within same region (div 5) should have same fingerprint
        let diag1 = make_diagnostic("SEC-001", "src/test.wxs", 42, "Test message");
        let diag2 = make_diagnostic("SEC-001", "src/test.wxs", 43, "Test message");

        let entry1 = BaselineEntry::from_diagnostic(&diag1, None);
        let entry2 = BaselineEntry::from_diagnostic(&diag2, None);

        assert_eq!(entry1.fingerprint, entry2.fingerprint);
    }

    #[test]
    fn test_fingerprint_different_rules() {
        let diag1 = make_diagnostic("SEC-001", "src/test.wxs", 42, "Test message");
        let diag2 = make_diagnostic("SEC-002", "src/test.wxs", 42, "Test message");

        let entry1 = BaselineEntry::from_diagnostic(&diag1, None);
        let entry2 = BaselineEntry::from_diagnostic(&diag2, None);

        assert_ne!(entry1.fingerprint, entry2.fingerprint);
    }

    #[test]
    fn test_baseline_new() {
        let baseline = Baseline::new();

        assert_eq!(baseline.version, BASELINE_VERSION);
        assert!(!baseline.created.is_empty());
        assert!(baseline.issues.is_empty());
    }

    #[test]
    fn test_baseline_from_results() {
        let mut result = AnalysisResult::new();
        result.add(make_diagnostic("SEC-001", "test.wxs", 10, "Error 1"));
        result.add(make_diagnostic("SEC-002", "test.wxs", 20, "Error 2"));

        let baseline = Baseline::from_results(&[result], None);

        assert_eq!(baseline.len(), 2);
    }

    #[test]
    fn test_baseline_contains() {
        let diag = make_diagnostic("SEC-001", "test.wxs", 42, "Test error");
        let mut result = AnalysisResult::new();
        result.add(diag.clone());

        let baseline = Baseline::from_results(&[result], None);

        assert!(baseline.contains(&diag, None));
    }

    #[test]
    fn test_baseline_not_contains() {
        let diag1 = make_diagnostic("SEC-001", "test.wxs", 42, "Test error");
        let diag2 = make_diagnostic("SEC-002", "other.wxs", 10, "Other error");

        let mut result = AnalysisResult::new();
        result.add(diag1);

        let baseline = Baseline::from_results(&[result], None);

        assert!(!baseline.contains(&diag2, None));
    }

    #[test]
    fn test_baseline_stats() {
        let mut result = AnalysisResult::new();
        result.add(make_diagnostic("SEC-001", "test.wxs", 10, "Error 1"));
        result.add(make_diagnostic("SEC-001", "test.wxs", 20, "Error 2"));
        result.add(make_diagnostic("SEC-002", "other.wxs", 30, "Error 3"));

        let baseline = Baseline::from_results(&[result], None);
        let stats = baseline.stats();

        assert_eq!(stats.total_issues, 3);
        assert_eq!(stats.unique_rules, 2);
        assert_eq!(stats.unique_files, 2);
        assert_eq!(stats.by_rule.get("SEC-001"), Some(&2));
        assert_eq!(stats.by_rule.get("SEC-002"), Some(&1));
    }

    #[test]
    fn test_filter_baseline() {
        let diag1 = make_diagnostic("SEC-001", "test.wxs", 42, "Baselined error");
        let diag2 = make_diagnostic("SEC-002", "test.wxs", 50, "New error");

        let mut baselined_result = AnalysisResult::new();
        baselined_result.add(diag1.clone());
        let baseline = Baseline::from_results(&[baselined_result], None);

        let mut results = vec![{
            let mut r = AnalysisResult::new();
            r.add(diag1);
            r.add(diag2);
            r
        }];

        let filtered = filter_baseline(&mut results, &baseline, None);

        assert_eq!(filtered, 1);
        assert_eq!(results[0].len(), 1);
        assert_eq!(results[0].diagnostics[0].rule_id, "SEC-002");
    }

    #[test]
    fn test_baseline_with_description() {
        let baseline = Baseline::new()
            .with_description("Initial baseline for legacy code");

        assert_eq!(baseline.description, Some("Initial baseline for legacy code".to_string()));
    }

    #[test]
    fn test_baseline_save_and_load() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("baseline.json");

        let mut result = AnalysisResult::new();
        result.add(make_diagnostic("SEC-001", "test.wxs", 42, "Test error"));

        let baseline = Baseline::from_results(&[result], None)
            .with_description("Test baseline");

        baseline.save(&path).unwrap();

        let loaded = Baseline::load(&path).unwrap();
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded.description, Some("Test baseline".to_string()));
    }

    #[test]
    fn test_baseline_error_display() {
        let err = BaselineError::ReadError(PathBuf::from("test.json"), "not found".to_string());
        assert!(err.to_string().contains("Failed to read"));
        assert!(err.to_string().contains("test.json"));

        let err = BaselineError::UnsupportedVersion(99);
        assert!(err.to_string().contains("Unsupported baseline version"));
    }

    #[test]
    fn test_baseline_find_and_load() {
        use tempfile::TempDir;
        use std::fs::File;
        use std::io::Write;

        let temp_dir = TempDir::new().unwrap();
        let baseline_path = temp_dir.path().join(BASELINE_FILE_NAME);
        {
            let mut f = File::create(&baseline_path).unwrap();
            writeln!(f, r#"{{"version": 1, "created": "2024-01-01T00:00:00Z", "tool_version": "0.1.0", "issues": []}}"#).unwrap();
        }

        let sub_dir = temp_dir.path().join("subdir");
        std::fs::create_dir(&sub_dir).unwrap();

        let loaded = Baseline::find_and_load(&sub_dir);
        assert!(loaded.is_some());
        assert!(loaded.unwrap().is_empty());
    }

    #[test]
    fn test_baseline_is_empty() {
        let baseline = Baseline::new();
        assert!(baseline.is_empty());
    }
}
