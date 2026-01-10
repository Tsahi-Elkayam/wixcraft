//! Human-readable text output formatter

use crate::core::{AnalysisResult, Diagnostic, Severity};
use super::Formatter;

/// Text formatter with optional color support
pub struct TextFormatter {
    colored: bool,
}

impl TextFormatter {
    pub fn new(colored: bool) -> Self {
        Self { colored }
    }

    fn severity_prefix(&self, severity: Severity) -> &'static str {
        match severity {
            Severity::Blocker => "blocker",
            Severity::High => "error",
            Severity::Medium => "warning",
            Severity::Low => "info",
            Severity::Info => "hint",
        }
    }

    fn severity_color(&self, severity: Severity) -> &'static str {
        if !self.colored {
            return "";
        }
        match severity {
            Severity::Blocker => "\x1b[1;35m", // Bold magenta
            Severity::High => "\x1b[1;31m",    // Bold red
            Severity::Medium => "\x1b[1;33m",  // Bold yellow
            Severity::Low => "\x1b[1;36m",     // Bold cyan
            Severity::Info => "\x1b[2m",       // Dim
        }
    }

    fn reset(&self) -> &'static str {
        if self.colored { "\x1b[0m" } else { "" }
    }

    fn bold(&self) -> &'static str {
        if self.colored { "\x1b[1m" } else { "" }
    }

    fn dim(&self) -> &'static str {
        if self.colored { "\x1b[2m" } else { "" }
    }
}

impl Formatter for TextFormatter {
    fn format(&self, results: &[AnalysisResult]) -> String {
        let mut output = String::new();
        let mut total_blockers = 0;
        let mut total_errors = 0;
        let mut total_warnings = 0;
        let mut total_info = 0;

        for result in results {
            for diag in &result.diagnostics {
                output.push_str(&self.format_diagnostic(diag));
                output.push('\n');

                match diag.severity {
                    Severity::Blocker => total_blockers += 1,
                    Severity::High => total_errors += 1,
                    Severity::Medium => total_warnings += 1,
                    Severity::Low | Severity::Info => total_info += 1,
                }
            }
        }

        // Summary line
        if total_blockers > 0 || total_errors > 0 || total_warnings > 0 || total_info > 0 {
            output.push('\n');
            let mut parts = Vec::new();
            if total_blockers > 0 {
                parts.push(format!(
                    "{}{} blocker{}{}",
                    self.severity_color(Severity::Blocker),
                    total_blockers,
                    if total_blockers == 1 { "" } else { "s" },
                    self.reset()
                ));
            }
            if total_errors > 0 {
                parts.push(format!(
                    "{}{} error{}{}",
                    self.severity_color(Severity::High),
                    total_errors,
                    if total_errors == 1 { "" } else { "s" },
                    self.reset()
                ));
            }
            if total_warnings > 0 {
                parts.push(format!(
                    "{}{} warning{}{}",
                    self.severity_color(Severity::Medium),
                    total_warnings,
                    if total_warnings == 1 { "" } else { "s" },
                    self.reset()
                ));
            }
            if total_info > 0 {
                parts.push(format!(
                    "{}{} info{}",
                    self.severity_color(Severity::Low),
                    total_info,
                    self.reset()
                ));
            }
            output.push_str(&format!("Found {}\n", parts.join(", ")));
        }

        output
    }

    fn format_diagnostic(&self, diag: &Diagnostic) -> String {
        let mut output = String::new();

        // Location
        output.push_str(&format!(
            "{}{}:{}:{}:{} ",
            self.bold(),
            diag.location.file.display(),
            diag.location.range.start.line,
            diag.location.range.start.character,
            self.reset()
        ));

        // Severity and rule ID
        output.push_str(&format!(
            "{}{}{}[{}]: ",
            self.severity_color(diag.severity),
            self.severity_prefix(diag.severity),
            self.reset(),
            diag.rule_id
        ));

        // Message
        output.push_str(&diag.message);

        // Help text
        if let Some(help) = &diag.help {
            output.push_str(&format!("\n  {}help: {}{}", self.dim(), help, self.reset()));
        }

        // Fix suggestion
        if let Some(fix) = &diag.fix {
            output.push_str(&format!(
                "\n  {}fix: {}{}",
                self.dim(),
                fix.description,
                self.reset()
            ));
        }

        // Related info
        for related in &diag.related {
            output.push_str(&format!(
                "\n  {}note: {} ({}:{}:{}){}",
                self.dim(),
                related.message,
                related.location.file.display(),
                related.location.range.start.line,
                related.location.range.start.character,
                self.reset()
            ));
        }

        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{Category, Fix, FixAction, Location, Position, Range, RelatedInfo};
    use std::path::PathBuf;

    fn make_location() -> Location {
        Location::new(
            PathBuf::from("test.wxs"),
            Range::new(Position::new(10, 5), Position::new(10, 20)),
        )
    }

    #[test]
    fn test_format_error() {
        let formatter = TextFormatter::new(false);
        let diag = Diagnostic::error(
            "TEST-001",
            Category::Validation,
            "Test error message",
            make_location(),
        );

        let output = formatter.format_diagnostic(&diag);
        assert!(output.contains("test.wxs:10:5"));
        assert!(output.contains("error"));
        assert!(output.contains("TEST-001"));
        assert!(output.contains("Test error message"));
    }

    #[test]
    fn test_format_info() {
        let formatter = TextFormatter::new(false);
        let diag = Diagnostic::info(
            "INFO-001",
            Category::BestPractice,
            "Info message",
            make_location(),
        );

        let output = formatter.format_diagnostic(&diag);
        assert!(output.contains("hint")); // Info maps to hint
        assert!(output.contains("INFO-001"));
    }

    #[test]
    fn test_format_with_help() {
        let formatter = TextFormatter::new(false);
        let diag = Diagnostic::warning(
            "TEST-002",
            Category::BestPractice,
            "Test warning",
            make_location(),
        )
        .with_help("This is help text");

        let output = formatter.format_diagnostic(&diag);
        assert!(output.contains("help: This is help text"));
    }

    #[test]
    fn test_format_with_fix() {
        let formatter = TextFormatter::new(false);
        let fix = Fix::new("Add missing attribute", FixAction::AddAttribute {
            range: Range::new(Position::new(1, 1), Position::new(1, 10)),
            name: "Id".to_string(),
            value: "MyId".to_string(),
        });
        let diag = Diagnostic::error("E1", Category::Validation, "Error", make_location())
            .with_fix(fix);

        let output = formatter.format_diagnostic(&diag);
        assert!(output.contains("fix: Add missing attribute"));
    }

    #[test]
    fn test_format_with_related() {
        let formatter = TextFormatter::new(false);
        let related_loc = Location::new(
            PathBuf::from("other.wxs"),
            Range::new(Position::new(5, 1), Position::new(5, 10)),
        );
        let diag = Diagnostic::error("E1", Category::Validation, "Error", make_location())
            .with_related(RelatedInfo::new(related_loc, "See definition here"));

        let output = formatter.format_diagnostic(&diag);
        assert!(output.contains("note: See definition here"));
        assert!(output.contains("other.wxs:5:1"));
    }

    #[test]
    fn test_summary() {
        let formatter = TextFormatter::new(false);
        let results = vec![AnalysisResult {
            files: Vec::new(),
            diagnostics: vec![
                Diagnostic::error("E1", Category::Validation, "Error 1", make_location()),
                Diagnostic::warning("W1", Category::BestPractice, "Warning 1", make_location()),
            ],
        }];

        let output = formatter.format(&results);
        assert!(output.contains("1 error"));
        assert!(output.contains("1 warning"));
    }

    #[test]
    fn test_summary_info() {
        let formatter = TextFormatter::new(false);
        let results = vec![AnalysisResult {
            files: Vec::new(),
            diagnostics: vec![
                Diagnostic::info("I1", Category::BestPractice, "Info 1", make_location()),
            ],
        }];

        let output = formatter.format(&results);
        assert!(output.contains("1 info"));
    }

    #[test]
    fn test_summary_plural() {
        let formatter = TextFormatter::new(false);
        let results = vec![AnalysisResult {
            files: Vec::new(),
            diagnostics: vec![
                Diagnostic::error("E1", Category::Validation, "Error 1", make_location()),
                Diagnostic::error("E2", Category::Validation, "Error 2", make_location()),
            ],
        }];

        let output = formatter.format(&results);
        assert!(output.contains("2 errors")); // Plural
    }

    #[test]
    fn test_colored_output() {
        let formatter = TextFormatter::new(true);
        let diag = Diagnostic::error("E1", Category::Validation, "Error", make_location());

        let output = formatter.format_diagnostic(&diag);
        // Should contain ANSI color codes
        assert!(output.contains("\x1b[1;31m")); // Bold red for High (error)
        assert!(output.contains("\x1b[0m")); // Reset
    }

    #[test]
    fn test_colored_warning() {
        let formatter = TextFormatter::new(true);
        let diag = Diagnostic::warning("W1", Category::BestPractice, "Warning", make_location());

        let output = formatter.format_diagnostic(&diag);
        // Should contain ANSI color codes
        assert!(output.contains("\x1b[1;33m")); // Bold yellow for Medium (warning)
    }

    #[test]
    fn test_colored_info() {
        let formatter = TextFormatter::new(true);
        let diag = Diagnostic::info("I1", Category::BestPractice, "Info", make_location());

        let output = formatter.format_diagnostic(&diag);
        // Should contain ANSI color codes (dim for Info/hint)
        assert!(output.contains("\x1b[2m")); // Dim for Info
    }

    #[test]
    fn test_no_diagnostics_no_summary() {
        let formatter = TextFormatter::new(false);
        let results: Vec<AnalysisResult> = vec![];

        let output = formatter.format(&results);
        assert!(output.is_empty());
    }

    #[test]
    fn test_colored_summary() {
        let formatter = TextFormatter::new(true);
        let results = vec![AnalysisResult {
            files: Vec::new(),
            diagnostics: vec![
                Diagnostic::error("E1", Category::Validation, "Error 1", make_location()),
                Diagnostic::warning("W1", Category::BestPractice, "Warning 1", make_location()),
                Diagnostic::info("I1", Category::BestPractice, "Info 1", make_location()),
            ],
        }];

        let output = formatter.format(&results);
        assert!(output.contains("\x1b[1;31m1 error")); // Red for High
        assert!(output.contains("\x1b[1;33m1 warning")); // Yellow for Medium
        assert!(output.contains("\x1b[1;36m1 info")); // Cyan for Low/Info
    }
}
