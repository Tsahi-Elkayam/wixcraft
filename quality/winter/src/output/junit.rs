//! JUnit XML output formatter
//!
//! Outputs diagnostics in JUnit XML format for CI/CD integration.

use crate::diagnostic::{Diagnostic, Severity};
use crate::engine::LintResult;
use super::OutputFormatter;

/// Formatter for JUnit XML output
pub struct JUnitFormatter {
    /// Test suite name
    pub suite_name: String,
}

impl JUnitFormatter {
    /// Create a new JUnit formatter
    pub fn new() -> Self {
        Self {
            suite_name: "winter-lint".to_string(),
        }
    }

    /// Set the test suite name
    pub fn with_suite_name(mut self, name: &str) -> Self {
        self.suite_name = name.to_string();
        self
    }

    fn escape_xml(s: &str) -> String {
        s.replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
            .replace('\'', "&apos;")
    }
}

impl Default for JUnitFormatter {
    fn default() -> Self {
        Self::new()
    }
}

impl OutputFormatter for JUnitFormatter {
    fn format(&self, result: &LintResult) -> String {
        let mut xml = String::new();
        xml.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");

        let total_tests = result.diagnostics.len();
        let failures = result.error_count;

        xml.push_str(&format!(
            "<testsuite name=\"{}\" tests=\"{}\" failures=\"{}\" errors=\"0\" time=\"{:.3}\">\n",
            Self::escape_xml(&self.suite_name),
            total_tests,
            failures,
            result.duration.as_secs_f64()
        ));

        // Group diagnostics by file
        let mut by_file: std::collections::HashMap<String, Vec<&Diagnostic>> =
            std::collections::HashMap::new();
        for diag in &result.diagnostics {
            by_file
                .entry(diag.location.file.display().to_string())
                .or_default()
                .push(diag);
        }

        for (file, diags) in &by_file {
            for diag in diags {
                xml.push_str(&format!(
                    "  <testcase name=\"{}\" classname=\"{}\">\n",
                    Self::escape_xml(&diag.rule_id),
                    Self::escape_xml(file)
                ));

                let failure_type = match diag.severity {
                    Severity::Error => "error",
                    Severity::Warning => "warning",
                    Severity::Info => "info",
                };

                xml.push_str(&format!(
                    "    <failure type=\"{}\" message=\"{}\">\n",
                    failure_type,
                    Self::escape_xml(&diag.message)
                ));
                xml.push_str(&format!(
                    "{}:{}:{}: {}\n",
                    file,
                    diag.location.line,
                    diag.location.column,
                    Self::escape_xml(&diag.message)
                ));
                xml.push_str("    </failure>\n");
                xml.push_str("  </testcase>\n");
            }
        }

        xml.push_str("</testsuite>\n");
        xml
    }

    fn format_diagnostic(&self, diagnostic: &Diagnostic) -> String {
        format!(
            "<testcase name=\"{}\" classname=\"{}\"><failure>{}</failure></testcase>",
            Self::escape_xml(&diagnostic.rule_id),
            Self::escape_xml(&diagnostic.location.file.display().to_string()),
            Self::escape_xml(&diagnostic.message)
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagnostic::Location;
    use std::path::PathBuf;

    #[test]
    fn test_junit_format() {
        let formatter = JUnitFormatter::new();
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
        assert!(output.contains("<?xml version"));
        assert!(output.contains("<testsuite"));
        assert!(output.contains("<testcase"));
        assert!(output.contains("<failure"));
    }

    #[test]
    fn test_xml_escaping() {
        assert_eq!(JUnitFormatter::escape_xml("<>&\"'"), "&lt;&gt;&amp;&quot;&apos;");
    }
}
