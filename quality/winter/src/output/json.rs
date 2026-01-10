//! JSON output formatter

use super::OutputFormatter;
use crate::diagnostic::Diagnostic;
use crate::engine::LintResult;
use serde::Serialize;

/// JSON formatter for machine-readable output
#[derive(Default)]
pub struct JsonFormatter {
    /// Pretty print with indentation
    pub pretty: bool,
}

impl JsonFormatter {
    /// Create a new JSON formatter
    pub fn new() -> Self {
        Self::default()
    }

    /// Enable pretty printing
    pub fn pretty(mut self) -> Self {
        self.pretty = true;
        self
    }
}

#[derive(Serialize)]
struct JsonOutput<'a> {
    diagnostics: Vec<JsonDiagnostic<'a>>,
    summary: JsonSummary,
}

#[derive(Serialize)]
struct JsonDiagnostic<'a> {
    rule_id: &'a str,
    severity: &'a str,
    message: &'a str,
    file: String,
    line: usize,
    column: usize,
    length: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    source_line: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    help: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    fix: Option<JsonFix<'a>>,
}

#[derive(Serialize)]
struct JsonFix<'a> {
    description: &'a str,
    replacement: &'a str,
}

#[derive(Serialize)]
struct JsonSummary {
    files_processed: usize,
    files_with_errors: usize,
    files_with_warnings: usize,
    error_count: usize,
    warning_count: usize,
    info_count: usize,
    duration_ms: u128,
}

impl OutputFormatter for JsonFormatter {
    fn format(&self, result: &LintResult) -> String {
        let diagnostics: Vec<JsonDiagnostic> = result
            .diagnostics
            .iter()
            .map(|d| JsonDiagnostic {
                rule_id: &d.rule_id,
                severity: match d.severity {
                    crate::diagnostic::Severity::Error => "error",
                    crate::diagnostic::Severity::Warning => "warning",
                    crate::diagnostic::Severity::Info => "info",
                },
                message: &d.message,
                file: d.location.file.display().to_string(),
                line: d.location.line,
                column: d.location.column,
                length: d.location.length,
                source_line: d.source_line.as_deref(),
                help: d.help.as_deref(),
                fix: d.fix.as_ref().map(|f| JsonFix {
                    description: &f.description,
                    replacement: &f.replacement,
                }),
            })
            .collect();

        let output = JsonOutput {
            diagnostics,
            summary: JsonSummary {
                files_processed: result.files_processed,
                files_with_errors: result.files_with_errors,
                files_with_warnings: result.files_with_warnings,
                error_count: result.error_count,
                warning_count: result.warning_count,
                info_count: result.info_count,
                duration_ms: result.duration.as_millis(),
            },
        };

        if self.pretty {
            serde_json::to_string_pretty(&output).unwrap_or_default()
        } else {
            serde_json::to_string(&output).unwrap_or_default()
        }
    }

    fn format_diagnostic(&self, diagnostic: &Diagnostic) -> String {
        let json_diag = JsonDiagnostic {
            rule_id: &diagnostic.rule_id,
            severity: match diagnostic.severity {
                crate::diagnostic::Severity::Error => "error",
                crate::diagnostic::Severity::Warning => "warning",
                crate::diagnostic::Severity::Info => "info",
            },
            message: &diagnostic.message,
            file: diagnostic.location.file.display().to_string(),
            line: diagnostic.location.line,
            column: diagnostic.location.column,
            length: diagnostic.location.length,
            source_line: diagnostic.source_line.as_deref(),
            help: diagnostic.help.as_deref(),
            fix: diagnostic.fix.as_ref().map(|f| JsonFix {
                description: &f.description,
                replacement: &f.replacement,
            }),
        };

        if self.pretty {
            serde_json::to_string_pretty(&json_diag).unwrap_or_default()
        } else {
            serde_json::to_string(&json_diag).unwrap_or_default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagnostic::{Location, Severity};
    use std::path::PathBuf;

    #[test]
    fn test_json_format_diagnostic() {
        let formatter = JsonFormatter::new();
        let diag = Diagnostic::new(
            "test-rule",
            Severity::Error,
            "Test message",
            Location::new(PathBuf::from("test.wxs"), 10, 5),
        );

        let output = formatter.format_diagnostic(&diag);
        assert!(output.contains("\"rule_id\":\"test-rule\""));
        assert!(output.contains("\"severity\":\"error\""));
        assert!(output.contains("\"line\":10"));
    }

    #[test]
    fn test_json_format_result() {
        let formatter = JsonFormatter::new();
        let result = LintResult {
            diagnostics: vec![],
            files_processed: 5,
            error_count: 2,
            warning_count: 3,
            ..Default::default()
        };

        let output = formatter.format(&result);
        assert!(output.contains("\"files_processed\":5"));
        assert!(output.contains("\"error_count\":2"));
        assert!(output.contains("\"warning_count\":3"));
    }

    #[test]
    fn test_json_pretty() {
        let formatter = JsonFormatter::new().pretty();
        let diag = Diagnostic::new(
            "test",
            Severity::Warning,
            "msg",
            Location::new(PathBuf::from("f.wxs"), 1, 1),
        );

        let output = formatter.format_diagnostic(&diag);
        assert!(output.contains('\n')); // Pretty printed has newlines
    }
}
