//! Diagnostic types

use serde::Serialize;
use std::path::PathBuf;

/// Diagnostic severity (matches LSP DiagnosticSeverity)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum DiagnosticSeverity {
    /// Error: a definite problem
    Error = 1,
    /// Warning: a potential issue
    Warning = 2,
    /// Information: informational message
    Information = 3,
    /// Hint: suggestion for improvement
    Hint = 4,
}

impl DiagnosticSeverity {
    pub fn as_str(&self) -> &'static str {
        match self {
            DiagnosticSeverity::Error => "error",
            DiagnosticSeverity::Warning => "warning",
            DiagnosticSeverity::Information => "info",
            DiagnosticSeverity::Hint => "hint",
        }
    }
}

/// Position in source (1-based)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct Position {
    pub line: u32,
    pub character: u32,
}

impl Position {
    pub fn new(line: u32, character: u32) -> Self {
        Self { line, character }
    }
}

/// Range in source
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct Range {
    pub start: Position,
    pub end: Position,
}

impl Range {
    pub fn new(start_line: u32, start_col: u32, end_line: u32, end_col: u32) -> Self {
        Self {
            start: Position::new(start_line, start_col),
            end: Position::new(end_line, end_col),
        }
    }

    /// Create from byte offsets
    pub fn from_offsets(source: &str, start_offset: usize, end_offset: usize) -> Self {
        let start = offset_to_position(source, start_offset);
        let end = offset_to_position(source, end_offset);
        Self { start, end }
    }
}

/// Convert byte offset to position
fn offset_to_position(source: &str, offset: usize) -> Position {
    let mut line = 1u32;
    let mut col = 1u32;

    for (i, ch) in source.char_indices() {
        if i >= offset {
            break;
        }
        if ch == '\n' {
            line += 1;
            col = 1;
        } else {
            col += 1;
        }
    }

    Position::new(line, col)
}

/// A diagnostic message
#[derive(Debug, Clone, Serialize)]
pub struct Diagnostic {
    /// Severity level
    pub severity: DiagnosticSeverity,
    /// Range in the document
    pub range: Range,
    /// Short message
    pub message: String,
    /// Diagnostic code/rule id
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
    /// Source (e.g., "wix-diagnostics")
    pub source: String,
    /// Related information
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub related_information: Vec<RelatedInformation>,
}

impl Diagnostic {
    /// Create a new diagnostic
    pub fn new(severity: DiagnosticSeverity, range: Range, message: String) -> Self {
        Self {
            severity,
            range,
            message,
            code: None,
            source: "wix-diagnostics".to_string(),
            related_information: Vec::new(),
        }
    }

    /// Create an error diagnostic
    pub fn error(range: Range, message: impl Into<String>) -> Self {
        Self::new(DiagnosticSeverity::Error, range, message.into())
    }

    /// Create a warning diagnostic
    pub fn warning(range: Range, message: impl Into<String>) -> Self {
        Self::new(DiagnosticSeverity::Warning, range, message.into())
    }

    /// Create an info diagnostic
    pub fn info(range: Range, message: impl Into<String>) -> Self {
        Self::new(DiagnosticSeverity::Information, range, message.into())
    }

    /// Create a hint diagnostic
    pub fn hint(range: Range, message: impl Into<String>) -> Self {
        Self::new(DiagnosticSeverity::Hint, range, message.into())
    }

    /// Add a code to the diagnostic
    pub fn with_code(mut self, code: impl Into<String>) -> Self {
        self.code = Some(code.into());
        self
    }

    /// Add related information
    pub fn with_related(mut self, related: RelatedInformation) -> Self {
        self.related_information.push(related);
        self
    }
}

/// Related diagnostic information
#[derive(Debug, Clone, Serialize)]
pub struct RelatedInformation {
    pub location: Location,
    pub message: String,
}

impl RelatedInformation {
    pub fn new(location: Location, message: impl Into<String>) -> Self {
        Self {
            location,
            message: message.into(),
        }
    }
}

/// File location
#[derive(Debug, Clone, Serialize)]
pub struct Location {
    pub file: PathBuf,
    pub range: Range,
}

impl Location {
    pub fn new(file: PathBuf, range: Range) -> Self {
        Self { file, range }
    }
}

/// Diagnostic result for a file
#[derive(Debug, Clone, Serialize)]
pub struct DiagnosticsResult {
    pub file: PathBuf,
    pub diagnostics: Vec<Diagnostic>,
}

impl DiagnosticsResult {
    pub fn new(file: PathBuf) -> Self {
        Self {
            file,
            diagnostics: Vec::new(),
        }
    }

    pub fn add(&mut self, diagnostic: Diagnostic) {
        self.diagnostics.push(diagnostic);
    }

    pub fn extend(&mut self, diagnostics: impl IntoIterator<Item = Diagnostic>) {
        self.diagnostics.extend(diagnostics);
    }

    pub fn error_count(&self) -> usize {
        self.diagnostics
            .iter()
            .filter(|d| d.severity == DiagnosticSeverity::Error)
            .count()
    }

    pub fn warning_count(&self) -> usize {
        self.diagnostics
            .iter()
            .filter(|d| d.severity == DiagnosticSeverity::Warning)
            .count()
    }

    pub fn is_empty(&self) -> bool {
        self.diagnostics.is_empty()
    }

    pub fn len(&self) -> usize {
        self.diagnostics.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diagnostic_severity_ordering() {
        assert!(DiagnosticSeverity::Error < DiagnosticSeverity::Warning);
        assert!(DiagnosticSeverity::Warning < DiagnosticSeverity::Information);
    }

    #[test]
    fn test_diagnostic_creation() {
        let range = Range::new(1, 1, 1, 10);
        let diag = Diagnostic::error(range, "Test error");

        assert_eq!(diag.severity, DiagnosticSeverity::Error);
        assert_eq!(diag.message, "Test error");
    }

    #[test]
    fn test_diagnostic_with_code() {
        let range = Range::new(1, 1, 1, 10);
        let diag = Diagnostic::warning(range, "Test").with_code("W001");

        assert_eq!(diag.code, Some("W001".to_string()));
    }

    #[test]
    fn test_diagnostics_result() {
        let mut result = DiagnosticsResult::new(PathBuf::from("test.wxs"));
        result.add(Diagnostic::error(Range::new(1, 1, 1, 10), "Error 1"));
        result.add(Diagnostic::warning(Range::new(2, 1, 2, 10), "Warning 1"));

        assert_eq!(result.error_count(), 1);
        assert_eq!(result.warning_count(), 1);
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_range_from_offsets() {
        let source = "abc\ndef\nghi";
        let range = Range::from_offsets(source, 4, 7);

        assert_eq!(range.start.line, 2);
        assert_eq!(range.start.character, 1);
        assert_eq!(range.end.line, 2);
        assert_eq!(range.end.character, 4);
    }
}
