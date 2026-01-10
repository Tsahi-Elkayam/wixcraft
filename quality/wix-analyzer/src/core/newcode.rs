//! New Code Period support for "Clean as You Code" methodology
//!
//! Focuses quality gates on new/changed code rather than legacy code.
//! Supports multiple new code definitions:
//! - **previous_version**: Code changed since last version tag
//! - **days**: Code changed in the last N days
//! - **reference_branch**: Code different from a reference branch
//! - **date**: Code changed since a specific date
//!
//! # Usage
//!
//! ```bash
//! # Focus on code changed in the last 30 days
//! wix-analyzer --new-code-period days:30 src/
//!
//! # Focus on code changed since last release
//! wix-analyzer --new-code-period previous_version src/
//!
//! # Focus on code different from main branch
//! wix-analyzer --new-code-period reference_branch:main src/
//! ```

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::process::Command;

/// New code period definition
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum NewCodePeriod {
    /// All code - no new code filtering
    AllCode,
    /// Code changed since last version/tag
    PreviousVersion {
        /// Optional version pattern (e.g., "v*")
        #[serde(default)]
        pattern: Option<String>,
    },
    /// Code changed in the last N days
    Days {
        /// Number of days (default: 30, max: 90)
        days: u32,
    },
    /// Code different from a reference branch
    ReferenceBranch {
        /// Branch name (e.g., "main", "master", "develop")
        branch: String,
    },
    /// Code changed since a specific date
    Date {
        /// ISO 8601 date string
        since: String,
    },
}

impl Default for NewCodePeriod {
    fn default() -> Self {
        Self::AllCode
    }
}

impl std::str::FromStr for NewCodePeriod {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.splitn(2, ':').collect();
        match parts[0].to_lowercase().as_str() {
            "all" | "all_code" => Ok(Self::AllCode),
            "previous_version" | "version" => Ok(Self::PreviousVersion {
                pattern: parts.get(1).map(|s| s.to_string()),
            }),
            "days" => {
                let days = parts
                    .get(1)
                    .ok_or("days requires a number (e.g., days:30)")?
                    .parse()
                    .map_err(|_| "Invalid number of days")?;
                if days > 90 {
                    return Err("Maximum 90 days allowed".to_string());
                }
                Ok(Self::Days { days })
            }
            "reference_branch" | "branch" => {
                let branch = parts
                    .get(1)
                    .ok_or("reference_branch requires a branch name")?
                    .to_string();
                Ok(Self::ReferenceBranch { branch })
            }
            "date" => {
                let since = parts
                    .get(1)
                    .ok_or("date requires an ISO 8601 date")?
                    .to_string();
                Ok(Self::Date { since })
            }
            _ => Err(format!(
                "Unknown new code period type: {}. Valid types: all, previous_version, days, reference_branch, date",
                parts[0]
            )),
        }
    }
}

impl std::fmt::Display for NewCodePeriod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AllCode => write!(f, "all"),
            Self::PreviousVersion { pattern: Some(p) } => write!(f, "previous_version:{}", p),
            Self::PreviousVersion { pattern: None } => write!(f, "previous_version"),
            Self::Days { days } => write!(f, "days:{}", days),
            Self::ReferenceBranch { branch } => write!(f, "reference_branch:{}", branch),
            Self::Date { since } => write!(f, "date:{}", since),
        }
    }
}

/// New code detection result
#[derive(Debug, Clone)]
pub struct NewCodeResult {
    /// Files considered "new" code
    pub new_code_files: HashSet<PathBuf>,
    /// Lines within files that are "new" (file -> set of line numbers)
    pub new_code_lines: std::collections::HashMap<PathBuf, HashSet<usize>>,
    /// Reference point description
    pub reference: String,
    /// Start date of new code period
    pub since: Option<DateTime<Utc>>,
}

impl NewCodeResult {
    /// Create an empty result (all code is new)
    pub fn all_code() -> Self {
        Self {
            new_code_files: HashSet::new(),
            new_code_lines: std::collections::HashMap::new(),
            reference: "all".to_string(),
            since: None,
        }
    }

    /// Check if a file is considered new code
    pub fn is_new_code_file(&self, path: &Path) -> bool {
        // If no files tracked, all files are new
        if self.new_code_files.is_empty() && self.new_code_lines.is_empty() {
            return true;
        }
        self.new_code_files.contains(path)
    }

    /// Check if a specific line is new code
    pub fn is_new_code_line(&self, path: &Path, line: usize) -> bool {
        // If no line tracking, check file-level
        if self.new_code_lines.is_empty() {
            return self.is_new_code_file(path);
        }

        self.new_code_lines
            .get(path)
            .map(|lines| lines.contains(&line))
            .unwrap_or(false)
    }

    /// Total new code file count
    pub fn new_file_count(&self) -> usize {
        self.new_code_files.len()
    }
}

/// New code period detector
#[derive(Debug)]
pub struct NewCodeDetector {
    /// Working directory
    workdir: PathBuf,
    /// File extensions to track
    extensions: HashSet<String>,
}

impl NewCodeDetector {
    /// Create a new detector
    pub fn new(workdir: impl Into<PathBuf>) -> Self {
        let mut extensions = HashSet::new();
        extensions.insert("wxs".to_string());
        extensions.insert("wxi".to_string());
        extensions.insert("wxl".to_string());

        Self {
            workdir: workdir.into(),
            extensions,
        }
    }

    /// Detect new code based on period definition
    pub fn detect(&self, period: &NewCodePeriod) -> Result<NewCodeResult, NewCodeError> {
        match period {
            NewCodePeriod::AllCode => Ok(NewCodeResult::all_code()),
            NewCodePeriod::PreviousVersion { pattern } => self.detect_since_version(pattern.as_deref()),
            NewCodePeriod::Days { days } => self.detect_since_days(*days),
            NewCodePeriod::ReferenceBranch { branch } => self.detect_since_branch(branch),
            NewCodePeriod::Date { since } => self.detect_since_date(since),
        }
    }

    /// Detect code changed since last version tag
    fn detect_since_version(&self, pattern: Option<&str>) -> Result<NewCodeResult, NewCodeError> {
        let pattern = pattern.unwrap_or("v*");

        // Find latest tag matching pattern
        let output = Command::new("git")
            .args(["describe", "--tags", "--match", pattern, "--abbrev=0"])
            .current_dir(&self.workdir)
            .output()
            .map_err(|e| NewCodeError::GitError(e.to_string()))?;

        if !output.status.success() {
            return Err(NewCodeError::NoVersionFound(pattern.to_string()));
        }

        let tag = String::from_utf8_lossy(&output.stdout).trim().to_string();
        self.detect_since_ref(&tag, format!("version {}", tag))
    }

    /// Detect code changed in the last N days
    fn detect_since_days(&self, days: u32) -> Result<NewCodeResult, NewCodeError> {
        let since = Utc::now() - Duration::days(days as i64);
        let since_str = since.format("%Y-%m-%d").to_string();

        self.detect_since_date(&since_str)
    }

    /// Detect code different from a reference branch
    fn detect_since_branch(&self, branch: &str) -> Result<NewCodeResult, NewCodeError> {
        // Find merge-base
        let output = Command::new("git")
            .args(["merge-base", "HEAD", branch])
            .current_dir(&self.workdir)
            .output()
            .map_err(|e| NewCodeError::GitError(e.to_string()))?;

        if !output.status.success() {
            return Err(NewCodeError::BranchNotFound(branch.to_string()));
        }

        let merge_base = String::from_utf8_lossy(&output.stdout).trim().to_string();
        self.detect_since_ref(&merge_base, format!("branch {}", branch))
    }

    /// Detect code changed since a specific date
    fn detect_since_date(&self, date: &str) -> Result<NewCodeResult, NewCodeError> {
        // Parse date
        let since = DateTime::parse_from_rfc3339(&format!("{}T00:00:00Z", date))
            .or_else(|_| DateTime::parse_from_rfc3339(date))
            .map_err(|_| NewCodeError::InvalidDate(date.to_string()))?
            .with_timezone(&Utc);

        // Get files changed since date
        let output = Command::new("git")
            .args(["log", "--since", date, "--name-only", "--pretty=format:"])
            .current_dir(&self.workdir)
            .output()
            .map_err(|e| NewCodeError::GitError(e.to_string()))?;

        let mut new_files = HashSet::new();

        for line in String::from_utf8_lossy(&output.stdout).lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            let path = PathBuf::from(line);
            if let Some(ext) = path.extension() {
                if self.extensions.contains(&ext.to_string_lossy().to_string()) {
                    new_files.insert(path);
                }
            }
        }

        Ok(NewCodeResult {
            new_code_files: new_files,
            new_code_lines: std::collections::HashMap::new(),
            reference: format!("since {}", date),
            since: Some(since),
        })
    }

    /// Detect code changed since a git ref (commit, tag, branch)
    fn detect_since_ref(&self, git_ref: &str, reference: String) -> Result<NewCodeResult, NewCodeError> {
        // Get changed files
        let output = Command::new("git")
            .args(["diff", "--name-only", git_ref])
            .current_dir(&self.workdir)
            .output()
            .map_err(|e| NewCodeError::GitError(e.to_string()))?;

        let mut new_files = HashSet::new();

        for line in String::from_utf8_lossy(&output.stdout).lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            let path = PathBuf::from(line);
            if let Some(ext) = path.extension() {
                if self.extensions.contains(&ext.to_string_lossy().to_string()) {
                    new_files.insert(path);
                }
            }
        }

        // Get specific changed lines for line-level tracking
        let mut new_lines = std::collections::HashMap::new();

        let diff_output = Command::new("git")
            .args(["diff", "-U0", git_ref])
            .current_dir(&self.workdir)
            .output()
            .map_err(|e| NewCodeError::GitError(e.to_string()))?;

        self.parse_diff_for_lines(&String::from_utf8_lossy(&diff_output.stdout), &mut new_lines);

        // Get ref date for since field
        let date_output = Command::new("git")
            .args(["log", "-1", "--format=%aI", git_ref])
            .current_dir(&self.workdir)
            .output()
            .ok();

        let since = date_output
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .and_then(|s| DateTime::parse_from_rfc3339(s.trim()).ok())
            .map(|d| d.with_timezone(&Utc));

        Ok(NewCodeResult {
            new_code_files: new_files,
            new_code_lines: new_lines,
            reference,
            since,
        })
    }

    /// Parse git diff output to extract changed line numbers
    fn parse_diff_for_lines(
        &self,
        diff: &str,
        new_lines: &mut std::collections::HashMap<PathBuf, HashSet<usize>>,
    ) {
        let mut current_file: Option<PathBuf> = None;

        for line in diff.lines() {
            // New file header: +++ b/path/to/file.wxs
            if line.starts_with("+++ b/") {
                let path = PathBuf::from(&line[6..]);
                if let Some(ext) = path.extension() {
                    if self.extensions.contains(&ext.to_string_lossy().to_string()) {
                        current_file = Some(path);
                    } else {
                        current_file = None;
                    }
                }
            }
            // Hunk header: @@ -old,count +new,count @@
            else if line.starts_with("@@") && current_file.is_some() {
                if let Some((start, count)) = parse_hunk_header(line) {
                    let file = current_file.as_ref().unwrap();
                    let lines = new_lines.entry(file.clone()).or_default();
                    for i in start..(start + count) {
                        lines.insert(i);
                    }
                }
            }
        }
    }
}

/// Parse a hunk header to get new file line range
fn parse_hunk_header(header: &str) -> Option<(usize, usize)> {
    // Format: @@ -old_start,old_count +new_start,new_count @@
    let parts: Vec<&str> = header.split_whitespace().collect();
    if parts.len() < 3 {
        return None;
    }

    let new_range = parts[2].trim_start_matches('+');
    let range_parts: Vec<&str> = new_range.split(',').collect();

    let start: usize = range_parts[0].parse().ok()?;
    let count: usize = range_parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(1);

    Some((start, count))
}

/// Filter diagnostics to only new code
pub fn filter_to_new_code(
    results: &mut Vec<crate::core::AnalysisResult>,
    new_code: &NewCodeResult,
) -> usize {
    let mut filtered = 0;

    for result in results.iter_mut() {
        let original_len = result.diagnostics.len();

        result.diagnostics.retain(|d| {
            new_code.is_new_code_line(&d.location.file, d.location.range.start.line)
        });

        filtered += original_len - result.diagnostics.len();
    }

    filtered
}

/// New code detection errors
#[derive(Debug)]
pub enum NewCodeError {
    GitError(String),
    NoVersionFound(String),
    BranchNotFound(String),
    InvalidDate(String),
}

impl std::fmt::Display for NewCodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::GitError(msg) => write!(f, "Git error: {}", msg),
            Self::NoVersionFound(pattern) => {
                write!(f, "No version tag found matching pattern: {}", pattern)
            }
            Self::BranchNotFound(branch) => write!(f, "Branch not found: {}", branch),
            Self::InvalidDate(date) => write!(f, "Invalid date format: {}", date),
        }
    }
}

impl std::error::Error for NewCodeError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_code_period_from_str() {
        assert!(matches!("all".parse::<NewCodePeriod>().unwrap(), NewCodePeriod::AllCode));
        assert!(matches!(
            "previous_version".parse::<NewCodePeriod>().unwrap(),
            NewCodePeriod::PreviousVersion { pattern: None }
        ));
        assert!(matches!(
            "previous_version:v*".parse::<NewCodePeriod>().unwrap(),
            NewCodePeriod::PreviousVersion { pattern: Some(_) }
        ));
        assert!(matches!(
            "days:30".parse::<NewCodePeriod>().unwrap(),
            NewCodePeriod::Days { days: 30 }
        ));
        assert!(matches!(
            "reference_branch:main".parse::<NewCodePeriod>().unwrap(),
            NewCodePeriod::ReferenceBranch { branch: _ }
        ));
        assert!(matches!(
            "date:2024-01-01".parse::<NewCodePeriod>().unwrap(),
            NewCodePeriod::Date { since: _ }
        ));
    }

    #[test]
    fn test_new_code_period_from_str_errors() {
        assert!("days".parse::<NewCodePeriod>().is_err()); // Missing number
        assert!("days:abc".parse::<NewCodePeriod>().is_err()); // Invalid number
        assert!("days:100".parse::<NewCodePeriod>().is_err()); // Too many days
        assert!("unknown".parse::<NewCodePeriod>().is_err()); // Unknown type
    }

    #[test]
    fn test_new_code_period_display() {
        assert_eq!(NewCodePeriod::AllCode.to_string(), "all");
        assert_eq!(NewCodePeriod::Days { days: 30 }.to_string(), "days:30");
        assert_eq!(
            NewCodePeriod::ReferenceBranch { branch: "main".to_string() }.to_string(),
            "reference_branch:main"
        );
    }

    #[test]
    fn test_new_code_result_all_code() {
        let result = NewCodeResult::all_code();
        assert!(result.is_new_code_file(Path::new("any.wxs")));
        assert!(result.is_new_code_line(Path::new("any.wxs"), 42));
    }

    #[test]
    fn test_new_code_result_with_files() {
        let mut files = HashSet::new();
        files.insert(PathBuf::from("new.wxs"));

        let result = NewCodeResult {
            new_code_files: files,
            new_code_lines: std::collections::HashMap::new(),
            reference: "test".to_string(),
            since: None,
        };

        assert!(result.is_new_code_file(Path::new("new.wxs")));
        assert!(!result.is_new_code_file(Path::new("old.wxs")));
    }

    #[test]
    fn test_new_code_result_with_lines() {
        let mut files = HashSet::new();
        files.insert(PathBuf::from("changed.wxs"));

        let mut lines = std::collections::HashMap::new();
        let mut file_lines = HashSet::new();
        file_lines.insert(10);
        file_lines.insert(11);
        file_lines.insert(12);
        lines.insert(PathBuf::from("changed.wxs"), file_lines);

        let result = NewCodeResult {
            new_code_files: files,
            new_code_lines: lines,
            reference: "test".to_string(),
            since: None,
        };

        assert!(result.is_new_code_line(Path::new("changed.wxs"), 10));
        assert!(result.is_new_code_line(Path::new("changed.wxs"), 11));
        assert!(!result.is_new_code_line(Path::new("changed.wxs"), 5));
        assert!(!result.is_new_code_line(Path::new("other.wxs"), 10));
    }

    #[test]
    fn test_parse_hunk_header() {
        assert_eq!(parse_hunk_header("@@ -1,5 +1,10 @@"), Some((1, 10)));
        assert_eq!(parse_hunk_header("@@ -10 +15,3 @@ function"), Some((15, 3)));
        assert_eq!(parse_hunk_header("@@ -0,0 +1 @@"), Some((1, 1)));
    }

    #[test]
    fn test_new_code_error_display() {
        let err = NewCodeError::BranchNotFound("feature".to_string());
        assert!(err.to_string().contains("Branch not found"));
        assert!(err.to_string().contains("feature"));
    }

    #[test]
    fn test_filter_to_new_code() {
        use crate::core::{AnalysisResult, Category, Diagnostic, Location, Position, Range};

        let mut files = HashSet::new();
        files.insert(PathBuf::from("new.wxs"));

        let mut lines = std::collections::HashMap::new();
        let mut new_lines = HashSet::new();
        new_lines.insert(10);
        lines.insert(PathBuf::from("new.wxs"), new_lines);

        let new_code = NewCodeResult {
            new_code_files: files,
            new_code_lines: lines,
            reference: "test".to_string(),
            since: None,
        };

        let mut results = vec![{
            let mut r = AnalysisResult::new();
            // Issue on new line
            r.add(Diagnostic::error(
                "VAL-001",
                Category::Validation,
                "Error on new line",
                Location::new(
                    PathBuf::from("new.wxs"),
                    Range::new(Position::new(10, 1), Position::new(10, 10)),
                ),
            ));
            // Issue on old line
            r.add(Diagnostic::error(
                "VAL-002",
                Category::Validation,
                "Error on old line",
                Location::new(
                    PathBuf::from("new.wxs"),
                    Range::new(Position::new(5, 1), Position::new(5, 10)),
                ),
            ));
            r
        }];

        let filtered = filter_to_new_code(&mut results, &new_code);

        assert_eq!(filtered, 1);
        assert_eq!(results[0].len(), 1);
        assert_eq!(results[0].diagnostics[0].rule_id, "VAL-001");
    }

    #[test]
    fn test_new_code_period_default() {
        let period: NewCodePeriod = Default::default();
        assert!(matches!(period, NewCodePeriod::AllCode));
    }
}
