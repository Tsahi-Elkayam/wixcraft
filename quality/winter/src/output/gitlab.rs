//! GitLab CI output formatter
//!
//! Outputs diagnostics in GitLab Code Quality format (JSON).
//! https://docs.gitlab.com/ee/ci/testing/code_quality.html

use super::OutputFormatter;
use crate::diagnostic::{Diagnostic, Severity};
use crate::engine::LintResult;
use serde::Serialize;

/// GitLab Code Quality issue
#[derive(Serialize)]
struct GitLabIssue {
    description: String,
    check_name: String,
    fingerprint: String,
    severity: String,
    location: GitLabLocation,
}

#[derive(Serialize)]
struct GitLabLocation {
    path: String,
    lines: GitLabLines,
}

#[derive(Serialize)]
struct GitLabLines {
    begin: usize,
}

/// Formatter for GitLab Code Quality JSON output
pub struct GitlabFormatter;

impl GitlabFormatter {
    /// Create a new GitLab formatter
    pub fn new() -> Self {
        Self
    }

    fn severity_to_gitlab(severity: Severity) -> &'static str {
        match severity {
            Severity::Error => "critical",
            Severity::Warning => "major",
            Severity::Info => "minor",
        }
    }

    fn generate_fingerprint(diag: &Diagnostic) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        diag.rule_id.hash(&mut hasher);
        diag.location.file.hash(&mut hasher);
        diag.location.line.hash(&mut hasher);
        diag.message.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }
}

impl Default for GitlabFormatter {
    fn default() -> Self {
        Self::new()
    }
}

impl OutputFormatter for GitlabFormatter {
    fn format(&self, result: &LintResult) -> String {
        let issues: Vec<GitLabIssue> = result
            .diagnostics
            .iter()
            .map(|diag| GitLabIssue {
                description: diag.message.clone(),
                check_name: diag.rule_id.clone(),
                fingerprint: Self::generate_fingerprint(diag),
                severity: Self::severity_to_gitlab(diag.severity).to_string(),
                location: GitLabLocation {
                    path: diag.location.file.display().to_string(),
                    lines: GitLabLines {
                        begin: diag.location.line,
                    },
                },
            })
            .collect();

        serde_json::to_string_pretty(&issues).unwrap_or_else(|_| "[]".to_string())
    }

    fn format_diagnostic(&self, diagnostic: &Diagnostic) -> String {
        let issue = GitLabIssue {
            description: diagnostic.message.clone(),
            check_name: diagnostic.rule_id.clone(),
            fingerprint: Self::generate_fingerprint(diagnostic),
            severity: Self::severity_to_gitlab(diagnostic.severity).to_string(),
            location: GitLabLocation {
                path: diagnostic.location.file.display().to_string(),
                lines: GitLabLines {
                    begin: diagnostic.location.line,
                },
            },
        };

        serde_json::to_string(&issue).unwrap_or_else(|_| "{}".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagnostic::Location;
    use std::path::PathBuf;

    #[test]
    fn test_gitlab_format() {
        let formatter = GitlabFormatter::new();
        let result = LintResult {
            diagnostics: vec![Diagnostic {
                rule_id: "test-rule".to_string(),
                message: "Test error".to_string(),
                severity: Severity::Error,
                location: Location::new(PathBuf::from("test.wxs"), 10, 5),
                source_line: None,
                context_before: vec![],
                context_after: vec![],
                help: None,
                fix: None,
                notes: vec![],
            }],
            files_processed: 1,
            files_with_errors: 1,
            files_with_warnings: 0,
            error_count: 1,
            warning_count: 0,
            info_count: 0,
            duration: std::time::Duration::from_millis(100),
            rule_timings: std::collections::HashMap::new(),
        };

        let output = formatter.format(&result);
        assert!(output.contains("\"severity\": \"critical\""));
        assert!(output.contains("\"check_name\": \"test-rule\""));
        assert!(output.contains("\"path\": \"test.wxs\""));
    }
}
