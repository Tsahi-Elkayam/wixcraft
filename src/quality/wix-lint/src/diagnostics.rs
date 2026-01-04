//! Diagnostic types for lint results

use std::path::PathBuf;
use std::str::FromStr;

/// Severity level for diagnostics
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum Severity {
    /// Informational hint
    #[default]
    Info,
    /// Warning - potential issue
    Warning,
    /// Error - definite problem
    Error,
}

impl FromStr for Severity {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "error" => Ok(Severity::Error),
            "warning" | "warn" => Ok(Severity::Warning),
            "info" | "hint" => Ok(Severity::Info),
            _ => Ok(Severity::Info), // Default to Info for unknown
        }
    }
}

impl Severity {

    /// Get display name
    pub fn as_str(&self) -> &'static str {
        match self {
            Severity::Error => "error",
            Severity::Warning => "warning",
            Severity::Info => "info",
        }
    }

    /// Get colored display name for terminal output
    pub fn colored(&self) -> String {
        match self {
            Severity::Error => "\x1b[1;31merror\x1b[0m".to_string(),
            Severity::Warning => "\x1b[1;33mwarning\x1b[0m".to_string(),
            Severity::Info => "\x1b[1;36minfo\x1b[0m".to_string(),
        }
    }
}

/// Source location in a file
#[derive(Debug, Clone, Default)]
pub struct Location {
    /// File path
    pub file: PathBuf,
    /// Line number (1-based)
    pub line: usize,
    /// Column number (1-based)
    pub column: usize,
    /// Length of the span (in characters)
    pub length: usize,
}

/// A lint diagnostic
#[derive(Debug, Clone)]
pub struct Diagnostic {
    /// Rule ID that triggered this diagnostic
    pub rule_id: String,
    /// Severity level
    pub severity: Severity,
    /// Short message
    pub message: String,
    /// Detailed help text
    pub help: Option<String>,
    /// Source location
    pub location: Location,
    /// The source line content
    pub source_line: Option<String>,
    /// Suggested fix
    pub fix: Option<Fix>,
}

/// A suggested fix for a diagnostic
#[derive(Debug, Clone)]
pub struct Fix {
    /// Description of what the fix does
    pub description: String,
    /// The replacement text
    pub replacement: String,
}

impl Diagnostic {
    /// Create a new diagnostic
    pub fn new(
        rule_id: impl Into<String>,
        severity: Severity,
        message: impl Into<String>,
        location: Location,
    ) -> Self {
        Self {
            rule_id: rule_id.into(),
            severity,
            message: message.into(),
            help: None,
            location,
            source_line: None,
            fix: None,
        }
    }

    /// Add help text
    pub fn with_help(mut self, help: impl Into<String>) -> Self {
        self.help = Some(help.into());
        self
    }

    /// Add source line
    pub fn with_source_line(mut self, line: impl Into<String>) -> Self {
        self.source_line = Some(line.into());
        self
    }

    /// Add fix suggestion
    pub fn with_fix(mut self, description: impl Into<String>, replacement: impl Into<String>) -> Self {
        self.fix = Some(Fix {
            description: description.into(),
            replacement: replacement.into(),
        });
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_severity_from_str() {
        assert_eq!("error".parse::<Severity>().unwrap(), Severity::Error);
        assert_eq!("ERROR".parse::<Severity>().unwrap(), Severity::Error);
        assert_eq!("warning".parse::<Severity>().unwrap(), Severity::Warning);
        assert_eq!("warn".parse::<Severity>().unwrap(), Severity::Warning);
        assert_eq!("WARN".parse::<Severity>().unwrap(), Severity::Warning);
        assert_eq!("info".parse::<Severity>().unwrap(), Severity::Info);
        assert_eq!("INFO".parse::<Severity>().unwrap(), Severity::Info);
        assert_eq!("unknown".parse::<Severity>().unwrap(), Severity::Info);
        assert_eq!("".parse::<Severity>().unwrap(), Severity::Info);
    }

    #[test]
    fn test_severity_as_str() {
        assert_eq!(Severity::Error.as_str(), "error");
        assert_eq!(Severity::Warning.as_str(), "warning");
        assert_eq!(Severity::Info.as_str(), "info");
    }

    #[test]
    fn test_severity_colored() {
        assert!(Severity::Error.colored().contains("error"));
        assert!(Severity::Warning.colored().contains("warning"));
        assert!(Severity::Info.colored().contains("info"));
    }

    #[test]
    fn test_severity_ordering() {
        assert!(Severity::Error > Severity::Warning);
        assert!(Severity::Warning > Severity::Info);
        assert!(Severity::Error > Severity::Info);
    }

    #[test]
    fn test_severity_default() {
        assert_eq!(Severity::default(), Severity::Info);
    }

    #[test]
    fn test_location_default() {
        let loc = Location::default();
        assert_eq!(loc.file, PathBuf::new());
        assert_eq!(loc.line, 0);
        assert_eq!(loc.column, 0);
        assert_eq!(loc.length, 0);
    }

    #[test]
    fn test_diagnostic_new() {
        let loc = Location {
            file: PathBuf::from("test.wxs"),
            line: 10,
            column: 5,
            length: 7,
        };
        let diag = Diagnostic::new("test-rule", Severity::Error, "Test message", loc);

        assert_eq!(diag.rule_id, "test-rule");
        assert_eq!(diag.severity, Severity::Error);
        assert_eq!(diag.message, "Test message");
        assert_eq!(diag.location.line, 10);
        assert!(diag.help.is_none());
        assert!(diag.source_line.is_none());
        assert!(diag.fix.is_none());
    }

    #[test]
    fn test_diagnostic_with_help() {
        let loc = Location::default();
        let diag = Diagnostic::new("test", Severity::Warning, "msg", loc)
            .with_help("This is help text");

        assert_eq!(diag.help, Some("This is help text".to_string()));
    }

    #[test]
    fn test_diagnostic_with_source_line() {
        let loc = Location::default();
        let diag = Diagnostic::new("test", Severity::Info, "msg", loc)
            .with_source_line("<Package Name=\"Test\" />");

        assert_eq!(diag.source_line, Some("<Package Name=\"Test\" />".to_string()));
    }

    #[test]
    fn test_diagnostic_with_fix() {
        let loc = Location::default();
        let diag = Diagnostic::new("test", Severity::Error, "msg", loc)
            .with_fix("Add attribute", "Guid=\"*\"");

        assert!(diag.fix.is_some());
        let fix = diag.fix.unwrap();
        assert_eq!(fix.description, "Add attribute");
        assert_eq!(fix.replacement, "Guid=\"*\"");
    }

    #[test]
    fn test_diagnostic_builder_chain() {
        let loc = Location {
            file: PathBuf::from("test.wxs"),
            line: 1,
            column: 1,
            length: 10,
        };
        let diag = Diagnostic::new("rule-id", Severity::Error, "message", loc)
            .with_help("help text")
            .with_source_line("source line")
            .with_fix("fix desc", "fix replacement");

        assert_eq!(diag.rule_id, "rule-id");
        assert_eq!(diag.help, Some("help text".to_string()));
        assert_eq!(diag.source_line, Some("source line".to_string()));
        assert!(diag.fix.is_some());
    }
}
