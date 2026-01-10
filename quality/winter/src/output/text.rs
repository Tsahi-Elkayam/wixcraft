//! Human-readable text output formatter

use super::OutputFormatter;
use crate::diagnostic::{Diagnostic, Severity};
use crate::engine::LintResult;
use colored::*;

/// Text formatter with optional color support
pub struct TextFormatter {
    /// Enable colored output
    pub colored: bool,

    /// Show source context
    pub show_source: bool,

    /// Show help text
    pub show_help: bool,

    /// Show fix suggestions
    pub show_fixes: bool,

    /// Show statistics
    pub show_stats: bool,

    /// Show context lines before/after
    pub show_context: bool,
}

impl Default for TextFormatter {
    fn default() -> Self {
        Self {
            colored: true,
            show_source: true,
            show_help: true,
            show_fixes: true,
            show_stats: true,
            show_context: true,
        }
    }
}

impl TextFormatter {
    /// Create a new text formatter
    pub fn new() -> Self {
        Self::default()
    }

    /// Disable colors
    pub fn without_color(mut self) -> Self {
        self.colored = false;
        self
    }

    fn severity_str(&self, severity: Severity) -> ColoredString {
        let s = format!("{}", severity);
        if !self.colored {
            return s.normal();
        }
        match severity {
            Severity::Error => s.red().bold(),
            Severity::Warning => s.yellow().bold(),
            Severity::Info => s.blue(),
        }
    }

    fn format_location(&self, diag: &Diagnostic) -> String {
        format!(
            "{}:{}:{}",
            diag.location.file.display(),
            diag.location.line,
            diag.location.column
        )
    }
}

impl OutputFormatter for TextFormatter {
    fn format(&self, result: &LintResult) -> String {
        let mut output = String::new();

        // Group diagnostics by file
        let mut by_file: std::collections::HashMap<_, Vec<_>> = std::collections::HashMap::new();
        for diag in &result.diagnostics {
            by_file
                .entry(diag.location.file.clone())
                .or_default()
                .push(diag);
        }

        // Output diagnostics grouped by file
        for (file, diagnostics) in &by_file {
            if self.colored {
                output.push_str(&format!("{}\n", file.display().to_string().underline()));
            } else {
                output.push_str(&format!("{}\n", file.display()));
            }

            for diag in diagnostics {
                output.push_str(&self.format_diagnostic(diag));
                output.push('\n');
            }
            output.push('\n');
        }

        // Statistics
        if self.show_stats {
            output.push_str(&format!(
                "\n{} {} processed",
                result.files_processed,
                if result.files_processed == 1 {
                    "file"
                } else {
                    "files"
                }
            ));

            let mut counts = Vec::new();
            if result.error_count > 0 {
                let s = format!(
                    "{} {}",
                    result.error_count,
                    if result.error_count == 1 {
                        "error"
                    } else {
                        "errors"
                    }
                );
                counts.push(if self.colored {
                    s.red().to_string()
                } else {
                    s
                });
            }
            if result.warning_count > 0 {
                let s = format!(
                    "{} {}",
                    result.warning_count,
                    if result.warning_count == 1 {
                        "warning"
                    } else {
                        "warnings"
                    }
                );
                counts.push(if self.colored {
                    s.yellow().to_string()
                } else {
                    s
                });
            }
            if result.info_count > 0 {
                let s = format!(
                    "{} {}",
                    result.info_count,
                    if result.info_count == 1 { "info" } else { "infos" }
                );
                counts.push(if self.colored {
                    s.blue().to_string()
                } else {
                    s
                });
            }

            if !counts.is_empty() {
                output.push_str(&format!(": {}", counts.join(", ")));
            }
            output.push('\n');

            output.push_str(&format!(
                "Finished in {:.2}s\n",
                result.duration.as_secs_f64()
            ));
        }

        output
    }

    fn format_diagnostic(&self, diag: &Diagnostic) -> String {
        let mut output = String::new();

        // Main diagnostic line
        output.push_str(&format!(
            "{}: {}[{}]: {}\n",
            self.format_location(diag),
            self.severity_str(diag.severity),
            if self.colored {
                diag.rule_id.cyan().to_string()
            } else {
                diag.rule_id.clone()
            },
            diag.message
        ));

        // Source line with context
        if self.show_source {
            output.push_str(&format!(
                "   {}\n",
                if self.colored {
                    "|".blue().to_string()
                } else {
                    "|".to_string()
                }
            ));

            // Context lines before
            if self.show_context {
                for (line_num, line) in &diag.context_before {
                    let num_str = format!("{:>4}", line_num);
                    output.push_str(&format!(
                        "{} {} {}\n",
                        if self.colored {
                            num_str.dimmed().to_string()
                        } else {
                            num_str
                        },
                        if self.colored {
                            "|".blue().to_string()
                        } else {
                            "|".to_string()
                        },
                        if self.colored {
                            line.dimmed().to_string()
                        } else {
                            line.clone()
                        }
                    ));
                }
            }

            // Error line
            if let Some(source) = &diag.source_line {
                let line_num = format!("{:>4}", diag.location.line);
                output.push_str(&format!(
                    "{} {} {}\n",
                    if self.colored {
                        line_num.blue().to_string()
                    } else {
                        line_num
                    },
                    if self.colored {
                        "|".blue().to_string()
                    } else {
                        "|".to_string()
                    },
                    source
                ));

                // Underline the problematic part
                if diag.location.column > 0 {
                    let padding = " ".repeat(diag.location.column - 1);
                    let underline = "^".repeat(diag.location.length.max(1));
                    output.push_str(&format!(
                        "   {} {}{}\n",
                        if self.colored {
                            "|".blue().to_string()
                        } else {
                            "|".to_string()
                        },
                        padding,
                        if self.colored {
                            underline.red().to_string()
                        } else {
                            underline
                        }
                    ));
                }
            }

            // Context lines after
            if self.show_context {
                for (line_num, line) in &diag.context_after {
                    let num_str = format!("{:>4}", line_num);
                    output.push_str(&format!(
                        "{} {} {}\n",
                        if self.colored {
                            num_str.dimmed().to_string()
                        } else {
                            num_str
                        },
                        if self.colored {
                            "|".blue().to_string()
                        } else {
                            "|".to_string()
                        },
                        if self.colored {
                            line.dimmed().to_string()
                        } else {
                            line.clone()
                        }
                    ));
                }
            }
        }

        // Help text
        if self.show_help {
            if let Some(help) = &diag.help {
                output.push_str(&format!(
                    "   {} help: {}\n",
                    if self.colored {
                        "=".blue().to_string()
                    } else {
                        "=".to_string()
                    },
                    help
                ));
            }
        }

        // Fix suggestion
        if self.show_fixes {
            if let Some(fix) = &diag.fix {
                output.push_str(&format!(
                    "   {} fix: {} -> {}\n",
                    if self.colored {
                        "=".green().to_string()
                    } else {
                        "=".to_string()
                    },
                    fix.description,
                    if self.colored {
                        fix.replacement.green().to_string()
                    } else {
                        fix.replacement.clone()
                    }
                ));
            }
        }

        // Notes
        for note in &diag.notes {
            output.push_str(&format!(
                "   {} note: {}\n",
                if self.colored {
                    "=".blue().to_string()
                } else {
                    "=".to_string()
                },
                note
            ));
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
    fn test_format_diagnostic() {
        let formatter = TextFormatter::new().without_color();
        let diag = Diagnostic::new(
            "test-rule",
            Severity::Error,
            "Test message",
            Location::new(PathBuf::from("test.wxs"), 10, 5),
        )
        .with_source_line("    <Package>")
        .with_help("Add the required attribute");

        let output = formatter.format_diagnostic(&diag);
        assert!(output.contains("test.wxs:10:5"));
        assert!(output.contains("error"));
        assert!(output.contains("test-rule"));
        assert!(output.contains("Test message"));
        assert!(output.contains("<Package>"));
        assert!(output.contains("help:"));
    }

    #[test]
    fn test_format_result() {
        let formatter = TextFormatter::new().without_color();
        let result = LintResult {
            diagnostics: vec![Diagnostic::new(
                "test",
                Severity::Warning,
                "Test",
                Location::new(PathBuf::from("test.wxs"), 1, 1),
            )],
            files_processed: 1,
            warning_count: 1,
            ..Default::default()
        };

        let output = formatter.format(&result);
        assert!(output.contains("1 file processed"));
        assert!(output.contains("1 warning"));
    }
}
