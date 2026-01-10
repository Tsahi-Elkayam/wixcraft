//! Diff-aware analysis for PR/CI pipelines
//!
//! Analyzes only changed files to provide faster feedback in CI environments.
//! Supports multiple diff sources:
//! - Git diff (branch comparison)
//! - File list (explicit list of changed files)
//! - Stdin (piped from other tools)
//!
//! # Usage
//!
//! ```bash
//! # Analyze only files changed since main branch
//! wix-analyzer --diff-base main src/
//!
//! # Analyze specific changed files
//! wix-analyzer --changed-files product.wxs,features.wxs
//!
//! # Pipe from git
//! git diff --name-only main | wix-analyzer --stdin-files
//! ```

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Diff source for determining changed files
#[derive(Debug, Clone)]
#[derive(Default)]
pub enum DiffSource {
    /// Git diff against a branch/commit
    GitBranch(String),
    /// Git diff against HEAD~N
    GitHead(usize),
    /// Explicit list of files
    FileList(Vec<PathBuf>),
    /// Files newer than a timestamp
    ModifiedSince(std::time::SystemTime),
    /// All files (no filtering)
    #[default]
    All,
}


/// Result of diff detection
#[derive(Debug, Clone)]
pub struct DiffResult {
    /// Files that were added
    pub added: Vec<PathBuf>,
    /// Files that were modified
    pub modified: Vec<PathBuf>,
    /// Files that were deleted
    pub deleted: Vec<PathBuf>,
    /// Base reference (branch/commit) used
    pub base_ref: Option<String>,
}

impl DiffResult {
    /// Create an empty diff result
    pub fn empty() -> Self {
        Self {
            added: Vec::new(),
            modified: Vec::new(),
            deleted: Vec::new(),
            base_ref: None,
        }
    }

    /// Get all changed files (added + modified)
    pub fn changed_files(&self) -> Vec<&PathBuf> {
        self.added.iter().chain(self.modified.iter()).collect()
    }

    /// Check if a file was changed
    pub fn is_changed(&self, path: &Path) -> bool {
        self.added.iter().any(|p| p == path) || self.modified.iter().any(|p| p == path)
    }

    /// Total number of changed files
    pub fn changed_count(&self) -> usize {
        self.added.len() + self.modified.len()
    }

    /// Check if any files changed
    pub fn has_changes(&self) -> bool {
        !self.added.is_empty() || !self.modified.is_empty()
    }
}

/// Diff detector for finding changed files
#[derive(Debug, Default)]
pub struct DiffDetector {
    /// Working directory
    workdir: Option<PathBuf>,
    /// File extensions to include
    extensions: HashSet<String>,
}

impl DiffDetector {
    /// Create a new diff detector
    pub fn new() -> Self {
        let mut extensions = HashSet::new();
        extensions.insert("wxs".to_string());
        extensions.insert("wxi".to_string());
        extensions.insert("wxl".to_string());

        Self {
            workdir: None,
            extensions,
        }
    }

    /// Set working directory
    pub fn with_workdir(mut self, workdir: impl Into<PathBuf>) -> Self {
        self.workdir = Some(workdir.into());
        self
    }

    /// Add file extension to filter
    pub fn with_extension(mut self, ext: impl Into<String>) -> Self {
        self.extensions.insert(ext.into());
        self
    }

    /// Detect changed files from a diff source
    pub fn detect(&self, source: &DiffSource) -> Result<DiffResult, DiffError> {
        match source {
            DiffSource::GitBranch(branch) => self.detect_git_branch(branch),
            DiffSource::GitHead(n) => self.detect_git_head(*n),
            DiffSource::FileList(files) => self.detect_file_list(files),
            DiffSource::ModifiedSince(time) => self.detect_modified_since(time),
            DiffSource::All => Ok(DiffResult::empty()),
        }
    }

    /// Detect changes from git branch comparison
    fn detect_git_branch(&self, branch: &str) -> Result<DiffResult, DiffError> {
        let workdir = self.workdir.as_deref().unwrap_or(Path::new("."));

        // Get merge-base to find common ancestor
        let merge_base = Command::new("git")
            .args(["merge-base", "HEAD", branch])
            .current_dir(workdir)
            .output()
            .map_err(|e| DiffError::GitError(e.to_string()))?;

        if !merge_base.status.success() {
            return Err(DiffError::GitError(format!(
                "Failed to find merge-base with {}: {}",
                branch,
                String::from_utf8_lossy(&merge_base.stderr)
            )));
        }

        let base_commit = String::from_utf8_lossy(&merge_base.stdout)
            .trim()
            .to_string();

        // Get diff against merge-base
        let output = Command::new("git")
            .args(["diff", "--name-status", &base_commit])
            .current_dir(workdir)
            .output()
            .map_err(|e| DiffError::GitError(e.to_string()))?;

        if !output.status.success() {
            return Err(DiffError::GitError(
                String::from_utf8_lossy(&output.stderr).to_string(),
            ));
        }

        self.parse_git_diff_output(
            &String::from_utf8_lossy(&output.stdout),
            Some(branch.to_string()),
        )
    }

    /// Detect changes from HEAD~n
    fn detect_git_head(&self, n: usize) -> Result<DiffResult, DiffError> {
        let workdir = self.workdir.as_deref().unwrap_or(Path::new("."));

        let output = Command::new("git")
            .args(["diff", "--name-status", &format!("HEAD~{}", n)])
            .current_dir(workdir)
            .output()
            .map_err(|e| DiffError::GitError(e.to_string()))?;

        if !output.status.success() {
            return Err(DiffError::GitError(
                String::from_utf8_lossy(&output.stderr).to_string(),
            ));
        }

        self.parse_git_diff_output(
            &String::from_utf8_lossy(&output.stdout),
            Some(format!("HEAD~{}", n)),
        )
    }

    /// Parse git diff --name-status output
    fn parse_git_diff_output(
        &self,
        output: &str,
        base_ref: Option<String>,
    ) -> Result<DiffResult, DiffError> {
        let mut result = DiffResult {
            added: Vec::new(),
            modified: Vec::new(),
            deleted: Vec::new(),
            base_ref,
        };

        for line in output.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() < 2 {
                continue;
            }

            let status = parts[0];
            let path = PathBuf::from(parts[1]);

            // Filter by extension
            if let Some(ext) = path.extension() {
                if !self.extensions.contains(&ext.to_string_lossy().to_string()) {
                    continue;
                }
            } else {
                continue; // Skip files without extension
            }

            match status.chars().next() {
                Some('A') => result.added.push(path),
                Some('M') => result.modified.push(path),
                Some('D') => result.deleted.push(path),
                Some('R') => {
                    // Renamed: treat as modified (new path is parts[2] if present)
                    if parts.len() > 2 {
                        result.modified.push(PathBuf::from(parts[2]));
                    } else {
                        result.modified.push(path);
                    }
                }
                _ => result.modified.push(path), // Default to modified
            }
        }

        Ok(result)
    }

    /// Detect from explicit file list
    fn detect_file_list(&self, files: &[PathBuf]) -> Result<DiffResult, DiffError> {
        let mut result = DiffResult::empty();

        for path in files {
            // Filter by extension
            if let Some(ext) = path.extension() {
                if !self.extensions.contains(&ext.to_string_lossy().to_string()) {
                    continue;
                }
            } else {
                continue;
            }

            if path.exists() {
                result.modified.push(path.clone());
            } else {
                result.deleted.push(path.clone());
            }
        }

        Ok(result)
    }

    /// Detect files modified since a timestamp
    fn detect_modified_since(
        &self,
        since: &std::time::SystemTime,
    ) -> Result<DiffResult, DiffError> {
        let workdir = self.workdir.as_deref().unwrap_or(Path::new("."));
        let mut result = DiffResult::empty();

        self.walk_directory(workdir, since, &mut result)?;

        Ok(result)
    }

    fn walk_directory(
        &self,
        dir: &Path,
        since: &std::time::SystemTime,
        result: &mut DiffResult,
    ) -> Result<(), DiffError> {
        let entries = std::fs::read_dir(dir).map_err(|e| DiffError::IoError(e.to_string()))?;

        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();

            if path.is_dir() {
                // Skip hidden directories and common ignore patterns
                if let Some(name) = path.file_name() {
                    let name = name.to_string_lossy();
                    if name.starts_with('.') || name == "target" || name == "node_modules" {
                        continue;
                    }
                }
                self.walk_directory(&path, since, result)?;
            } else if path.is_file() {
                // Check extension
                if let Some(ext) = path.extension() {
                    if !self.extensions.contains(&ext.to_string_lossy().to_string()) {
                        continue;
                    }
                } else {
                    continue;
                }

                // Check modification time
                if let Ok(metadata) = path.metadata() {
                    if let Ok(modified) = metadata.modified() {
                        if modified > *since {
                            result.modified.push(path);
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Parse file list from stdin or string
    pub fn parse_file_list(input: &str) -> Vec<PathBuf> {
        input
            .lines()
            .map(|l| l.trim())
            .filter(|l| !l.is_empty())
            .map(PathBuf::from)
            .collect()
    }
}

/// Diff detection errors
#[derive(Debug)]
pub enum DiffError {
    GitError(String),
    IoError(String),
}

impl std::fmt::Display for DiffError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::GitError(msg) => write!(f, "Git error: {}", msg),
            Self::IoError(msg) => write!(f, "IO error: {}", msg),
        }
    }
}

impl std::error::Error for DiffError {}

/// Filter analysis results to only show issues in changed files/lines
pub fn filter_to_changed(
    results: &mut Vec<crate::core::AnalysisResult>,
    diff: &DiffResult,
) -> usize {
    let changed: HashSet<_> = diff.changed_files().iter().map(|p| p.as_path()).collect();
    let mut filtered = 0;

    for result in results.iter_mut() {
        let original_len = result.diagnostics.len();

        result
            .diagnostics
            .retain(|d| changed.contains(d.location.file.as_path()));

        filtered += original_len - result.diagnostics.len();
    }

    filtered
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diff_source_default() {
        let source: DiffSource = Default::default();
        assert!(matches!(source, DiffSource::All));
    }

    #[test]
    fn test_diff_result_empty() {
        let result = DiffResult::empty();
        assert!(!result.has_changes());
        assert_eq!(result.changed_count(), 0);
    }

    #[test]
    fn test_diff_result_changed_files() {
        let result = DiffResult {
            added: vec![PathBuf::from("new.wxs")],
            modified: vec![PathBuf::from("changed.wxs")],
            deleted: vec![PathBuf::from("removed.wxs")],
            base_ref: Some("main".to_string()),
        };

        assert!(result.has_changes());
        assert_eq!(result.changed_count(), 2);
        assert_eq!(result.changed_files().len(), 2);
        assert!(result.is_changed(Path::new("new.wxs")));
        assert!(result.is_changed(Path::new("changed.wxs")));
        assert!(!result.is_changed(Path::new("removed.wxs")));
    }

    #[test]
    fn test_diff_detector_new() {
        let detector = DiffDetector::new();
        assert!(detector.extensions.contains("wxs"));
        assert!(detector.extensions.contains("wxi"));
        assert!(detector.extensions.contains("wxl"));
    }

    #[test]
    fn test_diff_detector_with_extension() {
        let detector = DiffDetector::new().with_extension("xml");
        assert!(detector.extensions.contains("xml"));
    }

    #[test]
    fn test_diff_detector_with_workdir() {
        let detector = DiffDetector::new().with_workdir("/tmp");
        assert_eq!(detector.workdir, Some(PathBuf::from("/tmp")));
    }

    #[test]
    fn test_detect_file_list() {
        use std::fs::File;
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let existing = temp_dir.path().join("existing.wxs");
        File::create(&existing).unwrap();

        let missing = temp_dir.path().join("missing.wxs");

        let detector = DiffDetector::new();
        let result = detector
            .detect(&DiffSource::FileList(vec![
                existing.clone(),
                missing.clone(),
            ]))
            .unwrap();

        assert_eq!(result.modified.len(), 1);
        assert_eq!(result.deleted.len(), 1);
        assert_eq!(result.modified[0], existing);
        assert_eq!(result.deleted[0], missing);
    }

    #[test]
    fn test_detect_all() {
        let detector = DiffDetector::new();
        let result = detector.detect(&DiffSource::All).unwrap();
        assert!(!result.has_changes());
    }

    #[test]
    fn test_parse_file_list() {
        let input = "file1.wxs\nfile2.wxi\n\nfile3.wxl\n";
        let files = DiffDetector::parse_file_list(input);
        assert_eq!(files.len(), 3);
    }

    #[test]
    fn test_parse_git_diff_output() {
        let detector = DiffDetector::new();
        let output = "A\tsrc/new.wxs\nM\tsrc/changed.wxs\nD\tsrc/deleted.wxs\nM\tsrc/other.txt\n";

        let result = detector
            .parse_git_diff_output(output, Some("main".to_string()))
            .unwrap();

        assert_eq!(result.added.len(), 1);
        assert_eq!(result.modified.len(), 1); // other.txt filtered out
        assert_eq!(result.deleted.len(), 1);
        assert_eq!(result.base_ref, Some("main".to_string()));
    }

    #[test]
    fn test_parse_git_diff_output_rename() {
        let detector = DiffDetector::new();
        let output = "R100\told.wxs\tnew.wxs\n";

        let result = detector.parse_git_diff_output(output, None).unwrap();

        assert_eq!(result.modified.len(), 1);
        assert_eq!(result.modified[0], PathBuf::from("new.wxs"));
    }

    #[test]
    fn test_diff_error_display() {
        let err = DiffError::GitError("branch not found".to_string());
        assert!(err.to_string().contains("Git error"));
        assert!(err.to_string().contains("branch not found"));
    }

    #[test]
    fn test_filter_to_changed() {
        use crate::core::{AnalysisResult, Category, Diagnostic, Location, Position, Range};

        let diff = DiffResult {
            added: vec![],
            modified: vec![PathBuf::from("changed.wxs")],
            deleted: vec![],
            base_ref: None,
        };

        let mut results = vec![{
            let mut r = AnalysisResult::new();
            r.add(Diagnostic::error(
                "VAL-001",
                Category::Validation,
                "Error in changed file",
                Location::new(
                    PathBuf::from("changed.wxs"),
                    Range::new(Position::new(1, 1), Position::new(1, 10)),
                ),
            ));
            r.add(Diagnostic::error(
                "VAL-002",
                Category::Validation,
                "Error in unchanged file",
                Location::new(
                    PathBuf::from("unchanged.wxs"),
                    Range::new(Position::new(1, 1), Position::new(1, 10)),
                ),
            ));
            r
        }];

        let filtered = filter_to_changed(&mut results, &diff);

        assert_eq!(filtered, 1);
        assert_eq!(results[0].len(), 1);
        assert_eq!(results[0].diagnostics[0].rule_id, "VAL-001");
    }
}
