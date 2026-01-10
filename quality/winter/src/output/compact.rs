//! Compact output formatter
//!
//! One line per diagnostic, minimal output for scripting.

use super::OutputFormatter;
use crate::diagnostic::{Diagnostic, Severity};
use crate::engine::LintResult;

/// Compact one-line-per-error formatter
pub struct CompactFormatter {
    /// Show severity prefix
    pub show_severity: bool,
    /// Show rule ID
    pub show_rule: bool,
}

impl CompactFormatter {
    /// Create a new compact formatter
    pub fn new() -> Self {
        Self {
            show_severity: true,
            show_rule: true,
        }
    }

    /// Hide severity prefix
    pub fn without_severity(mut self) -> Self {
        self.show_severity = false;
        self
    }

    /// Hide rule ID
    pub fn without_rule(mut self) -> Self {
        self.show_rule = false;
        self
    }
}

impl Default for CompactFormatter {
    fn default() -> Self {
        Self::new()
    }
}

impl OutputFormatter for CompactFormatter {
    fn format(&self, result: &LintResult) -> String {
        let mut output = String::new();

        for diag in &result.diagnostics {
            output.push_str(&self.format_diagnostic(diag));
            output.push('\n');
        }

        output
    }

    fn format_diagnostic(&self, diagnostic: &Diagnostic) -> String {
        let mut parts = Vec::new();

        // file:line:col
        parts.push(format!(
            "{}:{}:{}",
            diagnostic.location.file.display(),
            diagnostic.location.line,
            diagnostic.location.column
        ));

        // severity
        if self.show_severity {
            let sev = match diagnostic.severity {
                Severity::Error => "error",
                Severity::Warning => "warning",
                Severity::Info => "info",
            };
            parts.push(sev.to_string());
        }

        // rule
        if self.show_rule {
            parts.push(diagnostic.rule_id.clone());
        }

        // message
        parts.push(diagnostic.message.clone());

        parts.join(": ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagnostic::Location;
    use std::path::PathBuf;

    #[test]
    fn test_compact_format() {
        let formatter = CompactFormatter::new();
        let diag = Diagnostic {
            rule_id: "test-rule".to_string(),
            message: "Error message".to_string(),
            severity: Severity::Error,
            location: Location::new(PathBuf::from("test.wxs"), 10, 5),
            source_line: None,
            context_before: vec![],
            context_after: vec![],
            help: None,
            fix: None,
            notes: vec![],
        };

        let output = formatter.format_diagnostic(&diag);
        assert_eq!(output, "test.wxs:10:5: error: test-rule: Error message");
    }

    #[test]
    fn test_compact_minimal() {
        let formatter = CompactFormatter::new().without_severity().without_rule();
        let diag = Diagnostic {
            rule_id: "test-rule".to_string(),
            message: "Error".to_string(),
            severity: Severity::Error,
            location: Location::new(PathBuf::from("test.wxs"), 1, 1),
            source_line: None,
            context_before: vec![],
            context_after: vec![],
            help: None,
            fix: None,
            notes: vec![],
        };

        let output = formatter.format_diagnostic(&diag);
        assert_eq!(output, "test.wxs:1:1: Error");
    }

    #[test]
    fn test_compact_result() {
        let formatter = CompactFormatter::new();
        let result = LintResult {
            diagnostics: vec![
                Diagnostic {
                    rule_id: "r1".to_string(),
                    message: "E1".to_string(),
                    severity: Severity::Error,
                    location: Location::new(PathBuf::from("f.wxs"), 1, 1),
                    source_line: None,
                    context_before: vec![],
                    context_after: vec![],
                    help: None,
                    fix: None,
                    notes: vec![],
                },
                Diagnostic {
                    rule_id: "r2".to_string(),
                    message: "E2".to_string(),
                    severity: Severity::Warning,
                    location: Location::new(PathBuf::from("f.wxs"), 2, 1),
                    source_line: None,
                    context_before: vec![],
                    context_after: vec![],
                    help: None,
                    fix: None,
                    notes: vec![],
                },
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
        let lines: Vec<_> = output.lines().collect();
        assert_eq!(lines.len(), 2);
    }
}
