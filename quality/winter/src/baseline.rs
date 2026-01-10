//! Baseline system for hold-the-line linting
//!
//! Allows adopting linting without fixing legacy code first.
//! New issues are reported, but existing (baselined) issues are ignored.

use crate::diagnostic::Diagnostic;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

/// A baselined issue that should be ignored
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct BaselineIssue {
    /// Rule ID
    pub rule_id: String,
    /// File path (relative to baseline file)
    pub file: String,
    /// Line number (may shift over time)
    pub line: usize,
    /// Content hash for better matching
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_hash: Option<String>,
    /// Message fingerprint
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message_hash: Option<String>,
}

impl BaselineIssue {
    /// Create a new baseline issue from a diagnostic
    pub fn from_diagnostic(diag: &Diagnostic, base_path: &Path) -> Self {
        let relative_file = diag
            .location
            .file
            .strip_prefix(base_path)
            .unwrap_or(&diag.location.file)
            .to_string_lossy()
            .to_string();

        Self {
            rule_id: diag.rule_id.clone(),
            file: relative_file,
            line: diag.location.line,
            content_hash: diag.source_line.as_ref().map(|s| hash_string(s)),
            message_hash: Some(hash_string(&diag.message)),
        }
    }

    /// Create a fingerprint for fuzzy matching
    fn fingerprint(&self) -> String {
        format!("{}:{}", self.rule_id, self.file)
    }
}

/// Baseline containing all ignored issues
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Baseline {
    /// Version of the baseline format
    #[serde(default = "default_version")]
    pub version: String,
    /// Base path for relative file paths
    #[serde(skip)]
    pub base_path: PathBuf,
    /// All baselined issues
    pub issues: Vec<BaselineIssue>,
    /// When the baseline was created
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
    /// When the baseline was last updated
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<String>,
}

fn default_version() -> String {
    "1".to_string()
}

impl Baseline {
    /// Create an empty baseline
    pub fn new() -> Self {
        Self {
            version: default_version(),
            base_path: PathBuf::new(),
            issues: Vec::new(),
            created_at: Some(current_timestamp()),
            updated_at: None,
        }
    }

    /// Load baseline from a JSON file
    pub fn load(path: &Path) -> Result<Self, std::io::Error> {
        let content = std::fs::read_to_string(path)?;
        let mut baseline: Self = serde_json::from_str(&content)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        baseline.base_path = path.parent().unwrap_or(Path::new(".")).to_path_buf();
        Ok(baseline)
    }

    /// Save baseline to a JSON file
    pub fn save(&mut self, path: &Path) -> Result<(), std::io::Error> {
        self.updated_at = Some(current_timestamp());
        let content = serde_json::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        std::fs::write(path, content)
    }

    /// Add diagnostics to the baseline
    pub fn add_diagnostics(&mut self, diagnostics: &[Diagnostic]) {
        for diag in diagnostics {
            let issue = BaselineIssue::from_diagnostic(diag, &self.base_path);
            if !self.issues.contains(&issue) {
                self.issues.push(issue);
            }
        }
    }

    /// Filter diagnostics, removing baselined issues
    pub fn filter_diagnostics(&self, diagnostics: Vec<Diagnostic>) -> Vec<Diagnostic> {
        let index = self.build_index();

        diagnostics
            .into_iter()
            .filter(|diag| !self.is_baselined(diag, &index))
            .collect()
    }

    /// Check if a diagnostic is baselined
    fn is_baselined(&self, diag: &Diagnostic, index: &HashMap<String, Vec<&BaselineIssue>>) -> bool {
        let relative_file = diag
            .location
            .file
            .strip_prefix(&self.base_path)
            .unwrap_or(&diag.location.file)
            .to_string_lossy()
            .to_string();

        let fingerprint = format!("{}:{}", diag.rule_id, relative_file);

        if let Some(candidates) = index.get(&fingerprint) {
            for candidate in candidates {
                // Exact line match
                if candidate.line == diag.location.line {
                    return true;
                }

                // Fuzzy match by content hash
                if let (Some(baseline_hash), Some(source_line)) =
                    (&candidate.content_hash, &diag.source_line)
                {
                    if *baseline_hash == hash_string(source_line) {
                        return true;
                    }
                }

                // Fuzzy match by message hash
                if let Some(baseline_msg_hash) = &candidate.message_hash {
                    if *baseline_msg_hash == hash_string(&diag.message) {
                        return true;
                    }
                }
            }
        }

        false
    }

    /// Build an index for fast lookup
    fn build_index(&self) -> HashMap<String, Vec<&BaselineIssue>> {
        let mut index: HashMap<String, Vec<&BaselineIssue>> = HashMap::new();

        for issue in &self.issues {
            let fingerprint = issue.fingerprint();
            index.entry(fingerprint).or_default().push(issue);
        }

        index
    }

    /// Get count of baselined issues
    pub fn issue_count(&self) -> usize {
        self.issues.len()
    }

    /// Get count of baselined issues by severity
    pub fn count_by_rule(&self) -> HashMap<String, usize> {
        let mut counts: HashMap<String, usize> = HashMap::new();
        for issue in &self.issues {
            *counts.entry(issue.rule_id.clone()).or_default() += 1;
        }
        counts
    }

    /// Get unique files in the baseline
    pub fn files(&self) -> HashSet<&str> {
        self.issues.iter().map(|i| i.file.as_str()).collect()
    }

    /// Remove issues for files that no longer exist
    pub fn prune_missing_files(&mut self) {
        self.issues.retain(|issue| {
            let path = self.base_path.join(&issue.file);
            path.exists()
        });
    }

    /// Update baseline with new diagnostics (add new, keep existing)
    pub fn update(&mut self, diagnostics: &[Diagnostic]) {
        // Keep track of what's still active
        let mut active_issues: HashSet<BaselineIssue> = HashSet::new();

        for diag in diagnostics {
            let issue = BaselineIssue::from_diagnostic(diag, &self.base_path);
            active_issues.insert(issue);
        }

        // Keep only issues that are still active (or always keep all for hold-the-line)
        // For strict mode, we could prune fixed issues:
        // self.issues.retain(|i| active_issues.contains(i));

        // Add any new issues
        for issue in active_issues {
            if !self.issues.contains(&issue) {
                self.issues.push(issue);
            }
        }
    }
}

/// Simple string hash for content comparison
fn hash_string(s: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    s.trim().hash(&mut hasher);
    format!("{:x}", hasher.finish())
}

/// Get current timestamp
fn current_timestamp() -> String {
    // Simple ISO-8601 format without external deps
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs().to_string())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagnostic::{Location, Severity};

    fn make_diagnostic(rule: &str, file: &str, line: usize, msg: &str) -> Diagnostic {
        Diagnostic {
            rule_id: rule.to_string(),
            message: msg.to_string(),
            severity: Severity::Warning,
            location: Location::new(PathBuf::from(file), line, 0),
            source_line: Some(format!("line {} content", line)),
            context_before: vec![],
            context_after: vec![],
            help: None,
            fix: None,
            notes: vec![],
        }
    }

    #[test]
    fn test_baseline_new() {
        let baseline = Baseline::new();
        assert_eq!(baseline.version, "1");
        assert_eq!(baseline.issue_count(), 0);
    }

    #[test]
    fn test_add_diagnostics() {
        let mut baseline = Baseline::new();
        let diags = vec![
            make_diagnostic("rule1", "file.wxs", 10, "test message"),
            make_diagnostic("rule2", "file.wxs", 20, "another message"),
        ];

        baseline.add_diagnostics(&diags);
        assert_eq!(baseline.issue_count(), 2);
    }

    #[test]
    fn test_filter_baselined() {
        let mut baseline = Baseline::new();
        let diags = vec![
            make_diagnostic("rule1", "file.wxs", 10, "test message"),
        ];
        baseline.add_diagnostics(&diags);

        // Same diagnostic should be filtered
        let result = baseline.filter_diagnostics(diags.clone());
        assert!(result.is_empty());

        // Different diagnostic should pass through
        let new_diags = vec![
            make_diagnostic("rule2", "file.wxs", 15, "new message"),
        ];
        let result = baseline.filter_diagnostics(new_diags);
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn test_count_by_rule() {
        let mut baseline = Baseline::new();
        let diags = vec![
            make_diagnostic("rule1", "file1.wxs", 10, "msg1"),
            make_diagnostic("rule1", "file2.wxs", 20, "msg2"),
            make_diagnostic("rule2", "file1.wxs", 30, "msg3"),
        ];
        baseline.add_diagnostics(&diags);

        let counts = baseline.count_by_rule();
        assert_eq!(counts.get("rule1"), Some(&2));
        assert_eq!(counts.get("rule2"), Some(&1));
    }

    #[test]
    fn test_files() {
        let mut baseline = Baseline::new();
        let diags = vec![
            make_diagnostic("rule1", "file1.wxs", 10, "msg1"),
            make_diagnostic("rule1", "file2.wxs", 20, "msg2"),
            make_diagnostic("rule2", "file1.wxs", 30, "msg3"),
        ];
        baseline.add_diagnostics(&diags);

        let files = baseline.files();
        assert_eq!(files.len(), 2);
        assert!(files.contains("file1.wxs"));
        assert!(files.contains("file2.wxs"));
    }

    #[test]
    fn test_baseline_issue_from_diagnostic() {
        let diag = make_diagnostic("test-rule", "/path/to/file.wxs", 42, "test");
        let issue = BaselineIssue::from_diagnostic(&diag, Path::new("/path/to"));

        assert_eq!(issue.rule_id, "test-rule");
        assert_eq!(issue.file, "file.wxs");
        assert_eq!(issue.line, 42);
        assert!(issue.content_hash.is_some());
        assert!(issue.message_hash.is_some());
    }

    #[test]
    fn test_hash_string() {
        let hash1 = hash_string("hello world");
        let hash2 = hash_string("hello world");
        let hash3 = hash_string("different");

        assert_eq!(hash1, hash2);
        assert_ne!(hash1, hash3);
    }

    #[test]
    fn test_no_duplicates() {
        let mut baseline = Baseline::new();
        let diags = vec![
            make_diagnostic("rule1", "file.wxs", 10, "message"),
        ];

        baseline.add_diagnostics(&diags);
        baseline.add_diagnostics(&diags); // Add same again

        assert_eq!(baseline.issue_count(), 1);
    }
}
