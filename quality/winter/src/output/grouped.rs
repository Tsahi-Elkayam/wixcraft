//! Grouped output formatter
//!
//! Groups diagnostics by file for better readability.

use crate::diagnostic::{Diagnostic, Severity};
use crate::engine::LintResult;
use super::OutputFormatter;
use std::collections::HashMap;

/// Formatter that groups diagnostics by file
pub struct GroupedFormatter {
    /// Show colors (when supported)
    pub use_colors: bool,
    /// Show source line context
    pub show_source: bool,
}

impl GroupedFormatter {
    /// Create a new grouped formatter
    pub fn new() -> Self {
        Self {
            use_colors: true,
            show_source: true,
        }
    }

    /// Disable colors
    pub fn without_colors(mut self) -> Self {
        self.use_colors = false;
        self
    }

    /// Disable source line display
    pub fn without_source(mut self) -> Self {
        self.show_source = false;
        self
    }

    fn severity_symbol(&self, severity: Severity) -> &'static str {
        match severity {
            Severity::Error => "E",
            Severity::Warning => "W",
            Severity::Info => "I",
        }
    }

    fn severity_color(&self, severity: Severity) -> &'static str {
        if !self.use_colors {
            return "";
        }
        match severity {
            Severity::Error => "\x1b[31m",   // Red
            Severity::Warning => "\x1b[33m", // Yellow
            Severity::Info => "\x1b[34m",    // Blue
        }
    }

    fn reset_color(&self) -> &'static str {
        if self.use_colors { "\x1b[0m" } else { "" }
    }
}

impl Default for GroupedFormatter {
    fn default() -> Self {
        Self::new()
    }
}

impl OutputFormatter for GroupedFormatter {
    fn format(&self, result: &LintResult) -> String {
        let mut output = String::new();

        // Group by file
        let mut by_file: HashMap<String, Vec<&Diagnostic>> = HashMap::new();
        for diag in &result.diagnostics {
            by_file
                .entry(diag.location.file.display().to_string())
                .or_default()
                .push(diag);
        }

        // Sort files alphabetically
        let mut files: Vec<_> = by_file.keys().collect();
        files.sort();

        for file in files {
            let diags = by_file.get(file).unwrap();

            // File header
            output.push_str(&format!("\n{}\n", file));
            output.push_str(&format!("{}\n", "─".repeat(file.len().min(80))));

            // Sort diagnostics by line number
            let mut sorted_diags = diags.clone();
            sorted_diags.sort_by_key(|d| d.location.line);

            for diag in sorted_diags {
                output.push_str(&self.format_diagnostic(diag));
                output.push('\n');
            }
        }

        // Summary
        if !result.diagnostics.is_empty() {
            output.push_str(&format!(
                "\n{} error(s), {} warning(s), {} info(s) in {} file(s)\n",
                result.error_count,
                result.warning_count,
                result.info_count,
                result.files_processed
            ));
        }

        output
    }

    fn format_diagnostic(&self, diagnostic: &Diagnostic) -> String {
        let mut output = String::new();

        let color = self.severity_color(diagnostic.severity);
        let reset = self.reset_color();
        let symbol = self.severity_symbol(diagnostic.severity);

        output.push_str(&format!(
            "  {}[{}]{} {}:{}: {} ({})\n",
            color,
            symbol,
            reset,
            diagnostic.location.line,
            diagnostic.location.column,
            diagnostic.message,
            diagnostic.rule_id
        ));

        if self.show_source {
            if let Some(source) = &diagnostic.source_line {
                output.push_str(&format!("      │ {}\n", source.trim_start()));
            }
        }

        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagnostic::Location;
    use std::path::PathBuf;

    #[test]
    fn test_grouped_format() {
        let formatter = GroupedFormatter::new().without_colors();
        let result = LintResult {
            diagnostics: vec![
                Diagnostic {
                    rule_id: "rule1".to_string(),
                    message: "Error 1".to_string(),
                    severity: Severity::Error,
                    location: Location::new(PathBuf::from("file1.wxs"), 10, 5),
                    source_line: Some("  <Component>".to_string()),
                    context_before: vec![],
                    context_after: vec![],
                    help: None,
                    fix: None,
                    notes: vec![],
                },
                Diagnostic {
                    rule_id: "rule2".to_string(),
                    message: "Error 2".to_string(),
                    severity: Severity::Warning,
                    location: Location::new(PathBuf::from("file1.wxs"), 20, 1),
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
        assert!(output.contains("file1.wxs"));
        assert!(output.contains("[E]"));
        assert!(output.contains("[W]"));
        assert!(output.contains("1 error(s), 1 warning(s)"));
    }
}
