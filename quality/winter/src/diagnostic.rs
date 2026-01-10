//! Diagnostic types for linting results

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Severity level for diagnostics
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Serialize, Deserialize,
)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    /// Informational message
    Info,
    /// Warning - potential issue
    #[default]
    Warning,
    /// Error - definite problem
    Error,
}

/// Fix safety classification (like Ruff's safe/unsafe fixes)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FixSafety {
    /// Safe fix - preserves code meaning, can be applied automatically
    #[default]
    Safe,
    /// Unsafe fix - may change runtime behavior or remove comments
    Unsafe,
    /// Display only - shown to user but not auto-applied
    Display,
}

impl std::fmt::Display for FixSafety {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FixSafety::Safe => write!(f, "safe"),
            FixSafety::Unsafe => write!(f, "unsafe"),
            FixSafety::Display => write!(f, "display"),
        }
    }
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Severity::Info => write!(f, "info"),
            Severity::Warning => write!(f, "warning"),
            Severity::Error => write!(f, "error"),
        }
    }
}

impl std::str::FromStr for Severity {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "info" | "hint" | "note" => Ok(Severity::Info),
            "warning" | "warn" => Ok(Severity::Warning),
            "error" | "err" => Ok(Severity::Error),
            _ => Err(()),
        }
    }
}

/// Source code location
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Location {
    /// File path
    pub file: PathBuf,
    /// Line number (1-based)
    pub line: usize,
    /// Column number (1-based)
    pub column: usize,
    /// Length of the highlighted region
    pub length: usize,
}

impl Location {
    pub fn new(file: PathBuf, line: usize, column: usize) -> Self {
        Self {
            file,
            line,
            column,
            length: 0,
        }
    }

    pub fn with_length(mut self, length: usize) -> Self {
        self.length = length;
        self
    }
}

/// A suggested fix for the diagnostic
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Fix {
    /// Description of the fix
    pub description: String,
    /// The replacement text
    pub replacement: String,
    /// Start offset in the file (optional, for auto-fix)
    pub start_offset: Option<usize>,
    /// End offset in the file (optional, for auto-fix)
    pub end_offset: Option<usize>,
    /// Safety classification of this fix
    #[serde(default)]
    pub safety: FixSafety,
}

impl Fix {
    /// Create a new safe fix
    pub fn safe(description: &str, replacement: &str) -> Self {
        Self {
            description: description.to_string(),
            replacement: replacement.to_string(),
            start_offset: None,
            end_offset: None,
            safety: FixSafety::Safe,
        }
    }

    /// Create a new unsafe fix
    pub fn unsafe_fix(description: &str, replacement: &str) -> Self {
        Self {
            description: description.to_string(),
            replacement: replacement.to_string(),
            start_offset: None,
            end_offset: None,
            safety: FixSafety::Unsafe,
        }
    }

    /// Check if this fix is safe to apply automatically
    pub fn is_safe(&self) -> bool {
        self.safety == FixSafety::Safe
    }

    /// Check if this fix is unsafe
    pub fn is_unsafe(&self) -> bool {
        self.safety == FixSafety::Unsafe
    }
}

/// A lint diagnostic (warning, error, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Diagnostic {
    /// Rule ID that triggered this diagnostic
    pub rule_id: String,
    /// Severity level
    pub severity: Severity,
    /// Human-readable message
    pub message: String,
    /// Source location
    pub location: Location,
    /// The source line (for display)
    pub source_line: Option<String>,
    /// Context lines before the error line
    #[serde(default)]
    pub context_before: Vec<(usize, String)>,
    /// Context lines after the error line
    #[serde(default)]
    pub context_after: Vec<(usize, String)>,
    /// Help text (usually rule description)
    pub help: Option<String>,
    /// Suggested fix
    pub fix: Option<Fix>,
    /// Additional notes
    pub notes: Vec<String>,
}

impl Diagnostic {
    /// Create a new diagnostic
    pub fn new(rule_id: &str, severity: Severity, message: &str, location: Location) -> Self {
        Self {
            rule_id: rule_id.to_string(),
            severity,
            message: message.to_string(),
            location,
            source_line: None,
            context_before: Vec::new(),
            context_after: Vec::new(),
            help: None,
            fix: None,
            notes: Vec::new(),
        }
    }

    /// Add source line for display
    pub fn with_source_line(mut self, line: &str) -> Self {
        self.source_line = Some(line.to_string());
        self
    }

    /// Add context lines from source content
    pub fn with_context(mut self, source_lines: &[&str], context_count: usize) -> Self {
        if context_count == 0 || self.location.line == 0 {
            return self;
        }

        let line_num = self.location.line;

        // Lines before
        let start = line_num.saturating_sub(context_count + 1);
        let end = line_num.saturating_sub(1);
        for (i, line) in source_lines
            .iter()
            .enumerate()
            .skip(start)
            .take(end.saturating_sub(start))
        {
            self.context_before.push((i + 1, line.to_string()));
        }

        // Lines after
        let end = (line_num + context_count).min(source_lines.len());
        for (i, line) in source_lines
            .iter()
            .enumerate()
            .skip(line_num)
            .take(end.saturating_sub(line_num))
        {
            self.context_after.push((i + 1, line.to_string()));
        }

        self
    }

    /// Add help text
    pub fn with_help(mut self, help: &str) -> Self {
        self.help = Some(help.to_string());
        self
    }

    /// Add a suggested safe fix
    pub fn with_fix(mut self, description: &str, replacement: &str) -> Self {
        self.fix = Some(Fix::safe(description, replacement));
        self
    }

    /// Add a suggested unsafe fix
    pub fn with_unsafe_fix(mut self, description: &str, replacement: &str) -> Self {
        self.fix = Some(Fix::unsafe_fix(description, replacement));
        self
    }

    /// Check if this diagnostic has a fix
    pub fn has_fix(&self) -> bool {
        self.fix.is_some()
    }

    /// Check if this diagnostic has a safe fix
    pub fn has_safe_fix(&self) -> bool {
        self.fix.as_ref().is_some_and(|f| f.is_safe())
    }

    /// Check if this diagnostic has an unsafe fix
    pub fn has_unsafe_fix(&self) -> bool {
        self.fix.as_ref().is_some_and(|f| f.is_unsafe())
    }

    /// Add a note
    pub fn with_note(mut self, note: &str) -> Self {
        self.notes.push(note.to_string());
        self
    }

    /// Check if this is an error
    pub fn is_error(&self) -> bool {
        self.severity == Severity::Error
    }

    /// Check if this is a warning
    pub fn is_warning(&self) -> bool {
        self.severity == Severity::Warning
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_severity_ordering() {
        assert!(Severity::Error > Severity::Warning);
        assert!(Severity::Warning > Severity::Info);
    }

    #[test]
    fn test_severity_from_str() {
        assert_eq!("error".parse::<Severity>(), Ok(Severity::Error));
        assert_eq!("warning".parse::<Severity>(), Ok(Severity::Warning));
        assert_eq!("info".parse::<Severity>(), Ok(Severity::Info));
        assert_eq!("warn".parse::<Severity>(), Ok(Severity::Warning));
        assert_eq!("hint".parse::<Severity>(), Ok(Severity::Info));
    }

    #[test]
    fn test_severity_display() {
        assert_eq!(format!("{}", Severity::Error), "error");
        assert_eq!(format!("{}", Severity::Warning), "warning");
        assert_eq!(format!("{}", Severity::Info), "info");
    }

    #[test]
    fn test_diagnostic_creation() {
        let loc = Location::new(PathBuf::from("test.wxs"), 10, 5);
        let diag = Diagnostic::new("test-rule", Severity::Error, "Test message", loc);

        assert_eq!(diag.rule_id, "test-rule");
        assert_eq!(diag.severity, Severity::Error);
        assert_eq!(diag.message, "Test message");
        assert!(diag.is_error());
        assert!(!diag.is_warning());
    }

    #[test]
    fn test_diagnostic_with_extras() {
        let loc = Location::new(PathBuf::from("test.wxs"), 10, 5);
        let diag = Diagnostic::new("test-rule", Severity::Warning, "Test", loc)
            .with_source_line("  <Package>")
            .with_help("Add the required attribute")
            .with_fix("Add Id attribute", "Id=\"MyId\"")
            .with_note("See documentation");

        assert!(diag.source_line.is_some());
        assert!(diag.help.is_some());
        assert!(diag.fix.is_some());
        assert_eq!(diag.notes.len(), 1);
    }

    #[test]
    fn test_location_with_length() {
        let loc = Location::new(PathBuf::from("test.wxs"), 1, 1).with_length(10);
        assert_eq!(loc.length, 10);
    }
}
