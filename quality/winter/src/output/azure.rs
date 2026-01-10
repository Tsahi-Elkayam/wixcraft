//! Azure DevOps output formatter
//!
//! Outputs diagnostics in Azure DevOps logging command format:
//! ##vso[task.logissue type=warning;sourcepath=...;linenumber=...;columnnumber=...;code=...]message

use super::OutputFormatter;
use crate::diagnostic::{Diagnostic, Severity};
use crate::engine::LintResult;

/// Formatter for Azure DevOps logging commands
pub struct AzureFormatter {
    /// Whether to include summary
    pub show_summary: bool,
}

impl AzureFormatter {
    /// Create a new Azure formatter
    pub fn new() -> Self {
        Self { show_summary: true }
    }

    /// Disable summary output
    pub fn without_summary(mut self) -> Self {
        self.show_summary = false;
        self
    }
}

impl Default for AzureFormatter {
    fn default() -> Self {
        Self::new()
    }
}

impl OutputFormatter for AzureFormatter {
    fn format(&self, result: &LintResult) -> String {
        let mut output = String::new();

        for diag in &result.diagnostics {
            output.push_str(&self.format_diagnostic(diag));
            output.push('\n');
        }

        if self.show_summary && !result.diagnostics.is_empty() {
            // Use Azure DevOps section grouping
            output.push_str("##[group]Lint Summary\n");
            output.push_str(&format!("Files checked: {}\n", result.files_processed));
            output.push_str(&format!("Errors: {}\n", result.error_count));
            output.push_str(&format!("Warnings: {}\n", result.warning_count));
            output.push_str(&format!("Info: {}\n", result.info_count));
            output.push_str("##[endgroup]\n");
        }

        output
    }

    fn format_diagnostic(&self, diagnostic: &Diagnostic) -> String {
        let issue_type = match diagnostic.severity {
            Severity::Error => "error",
            Severity::Warning => "warning",
            Severity::Info => "warning", // Azure doesn't have info, use warning
        };

        let file = diagnostic.location.file.display();
        let line = diagnostic.location.line;
        let col = diagnostic.location.column.max(1);

        // Escape special characters in message
        let message = diagnostic.message.replace('\r', "").replace('\n', " ");

        format!(
            "##vso[task.logissue type={};sourcepath={};linenumber={};columnnumber={};code={}]{}",
            issue_type, file, line, col, diagnostic.rule_id, message
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagnostic::Location;
    use std::path::PathBuf;

    #[test]
    fn test_azure_format_error() {
        let formatter = AzureFormatter::new();
        let diag = Diagnostic {
            rule_id: "test-rule".to_string(),
            message: "Error message".to_string(),
            severity: Severity::Error,
            location: Location::new(PathBuf::from("src/test.wxs"), 10, 5),
            source_line: None,
            context_before: vec![],
            context_after: vec![],
            help: None,
            fix: None,
            notes: vec![],
        };

        let output = formatter.format_diagnostic(&diag);
        assert!(output.starts_with("##vso[task.logissue type=error"));
        assert!(output.contains("sourcepath=src/test.wxs"));
        assert!(output.contains("linenumber=10"));
        assert!(output.contains("code=test-rule"));
    }

    #[test]
    fn test_azure_format_result() {
        let formatter = AzureFormatter::new();
        let result = LintResult {
            diagnostics: vec![Diagnostic {
                rule_id: "test-rule".to_string(),
                message: "Error".to_string(),
                severity: Severity::Error,
                location: Location::new(PathBuf::from("file.wxs"), 1, 1),
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
        assert!(output.contains("##vso[task.logissue"));
        assert!(output.contains("##[group]Lint Summary"));
        assert!(output.contains("##[endgroup]"));
    }
}
