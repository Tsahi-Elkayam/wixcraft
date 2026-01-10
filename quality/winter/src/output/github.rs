//! GitHub Actions output formatter
//!
//! Outputs diagnostics in GitHub Actions workflow command format:
//! ::warning file={name},line={line},col={col}::{message}

use super::OutputFormatter;
use crate::diagnostic::{Diagnostic, Severity};
use crate::engine::LintResult;

/// Formatter for GitHub Actions annotations
pub struct GithubFormatter {
    /// Whether to include summary
    pub show_summary: bool,
}

impl GithubFormatter {
    /// Create a new GitHub formatter
    pub fn new() -> Self {
        Self { show_summary: true }
    }

    /// Disable summary output
    pub fn without_summary(mut self) -> Self {
        self.show_summary = false;
        self
    }
}

impl Default for GithubFormatter {
    fn default() -> Self {
        Self::new()
    }
}

impl OutputFormatter for GithubFormatter {
    fn format(&self, result: &LintResult) -> String {
        let mut output = String::new();

        // Output each diagnostic as a GitHub annotation
        for diag in &result.diagnostics {
            output.push_str(&self.format_diagnostic(diag));
            output.push('\n');
        }

        // Output summary as a notice if enabled
        if self.show_summary && !result.diagnostics.is_empty() {
            output.push_str(&format!(
                "::notice::Linting complete: {} error(s), {} warning(s), {} info(s) in {} file(s)\n",
                result.error_count,
                result.warning_count,
                result.info_count,
                result.files_processed
            ));
        }

        // Group annotations by file for better readability
        if !result.diagnostics.is_empty() {
            output.push_str("::group::Lint Summary\n");
            output.push_str(&format!("Files checked: {}\n", result.files_processed));
            output.push_str(&format!("Errors: {}\n", result.error_count));
            output.push_str(&format!("Warnings: {}\n", result.warning_count));
            output.push_str(&format!("Info: {}\n", result.info_count));
            output.push_str("::endgroup::\n");
        }

        output
    }

    fn format_diagnostic(&self, diagnostic: &Diagnostic) -> String {
        let level = match diagnostic.severity {
            Severity::Error => "error",
            Severity::Warning => "warning",
            Severity::Info => "notice",
        };

        let file = diagnostic.location.file.display();
        let line = diagnostic.location.line;
        let col = diagnostic.location.column;

        // Escape special characters in message
        let message = diagnostic
            .message
            .replace('%', "%25")
            .replace('\r', "%0D")
            .replace('\n', "%0A");

        // Format: ::warning file={name},line={line},col={col},title={title}::{message}
        format!(
            "::{} file={},line={},col={},title={}::{}",
            level,
            file,
            line,
            col.max(1), // GitHub requires col >= 1
            diagnostic.rule_id,
            message
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagnostic::Location;
    use std::path::PathBuf;

    fn make_diagnostic(
        severity: Severity,
        rule: &str,
        file: &str,
        line: usize,
        msg: &str,
    ) -> Diagnostic {
        Diagnostic {
            rule_id: rule.to_string(),
            message: msg.to_string(),
            severity,
            location: Location::new(PathBuf::from(file), line, 5),
            source_line: None,
            context_before: vec![],
            context_after: vec![],
            help: None,
            fix: None,
            notes: vec![],
        }
    }

    #[test]
    fn test_format_error() {
        let formatter = GithubFormatter::new();
        let diag = make_diagnostic(
            Severity::Error,
            "test-rule",
            "src/test.wxs",
            10,
            "Error message",
        );

        let output = formatter.format_diagnostic(&diag);
        assert!(output.starts_with("::error"));
        assert!(output.contains("file=src/test.wxs"));
        assert!(output.contains("line=10"));
        assert!(output.contains("title=test-rule"));
        assert!(output.contains("Error message"));
    }

    #[test]
    fn test_format_warning() {
        let formatter = GithubFormatter::new();
        let diag = make_diagnostic(
            Severity::Warning,
            "test-rule",
            "src/test.wxs",
            20,
            "Warning message",
        );

        let output = formatter.format_diagnostic(&diag);
        assert!(output.starts_with("::warning"));
    }

    #[test]
    fn test_format_info() {
        let formatter = GithubFormatter::new();
        let diag = make_diagnostic(
            Severity::Info,
            "test-rule",
            "src/test.wxs",
            30,
            "Info message",
        );

        let output = formatter.format_diagnostic(&diag);
        assert!(output.starts_with("::notice"));
    }

    #[test]
    fn test_escape_newlines() {
        let formatter = GithubFormatter::new();
        let diag = make_diagnostic(Severity::Error, "test", "test.wxs", 1, "Line1\nLine2");

        let output = formatter.format_diagnostic(&diag);
        assert!(output.contains("%0A"));
        assert!(!output.contains('\n'));
    }

    #[test]
    fn test_format_result() {
        let formatter = GithubFormatter::new();
        let result = LintResult {
            diagnostics: vec![
                make_diagnostic(Severity::Error, "rule1", "file.wxs", 1, "Error"),
                make_diagnostic(Severity::Warning, "rule2", "file.wxs", 2, "Warning"),
            ],
            files_processed: 1,
            files_with_errors: 1,
            files_with_warnings: 1,
            error_count: 1,
            warning_count: 1,
            info_count: 0,
            duration: std::time::Duration::from_millis(100),
            rule_timings: std::collections::HashMap::new(),
        };

        let output = formatter.format(&result);
        assert!(output.contains("::error"));
        assert!(output.contains("::warning"));
        assert!(output.contains("::group::"));
        assert!(output.contains("::endgroup::"));
    }
}
