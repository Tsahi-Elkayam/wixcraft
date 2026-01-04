//! Core types for best practices analysis

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Category of best practice
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PracticeCategory {
    /// Code efficiency - duplicates, unused elements
    Efficiency,
    /// WiX patterns and idioms
    Idiom,
    /// Performance considerations
    Performance,
    /// Code maintainability
    Maintainability,
}

impl PracticeCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            PracticeCategory::Efficiency => "efficiency",
            PracticeCategory::Idiom => "idiom",
            PracticeCategory::Performance => "performance",
            PracticeCategory::Maintainability => "maintainability",
        }
    }
}

/// Impact level of a suggestion
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Impact {
    /// Low impact - nice to have
    Low = 1,
    /// Medium impact - should address
    Medium = 2,
    /// High impact - important to fix
    High = 3,
}

impl Impact {
    pub fn as_str(&self) -> &'static str {
        match self {
            Impact::Low => "low",
            Impact::Medium => "medium",
            Impact::High => "high",
        }
    }
}

/// Position in a file (1-based for LSP compatibility)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Position {
    /// Line number (1-based)
    pub line: usize,
    /// Character offset (1-based)
    pub character: usize,
}

impl Position {
    pub fn new(line: usize, character: usize) -> Self {
        Self { line, character }
    }
}

/// Range in a file
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Range {
    pub start: Position,
    pub end: Position,
}

impl Range {
    pub fn new(start: Position, end: Position) -> Self {
        Self { start, end }
    }

    /// Create range from byte offsets in source
    pub fn from_offsets(source: &str, start: usize, end: usize) -> Self {
        let start_pos = offset_to_position(source, start);
        let end_pos = offset_to_position(source, end);
        Self::new(start_pos, end_pos)
    }
}

/// Convert byte offset to Position
fn offset_to_position(source: &str, offset: usize) -> Position {
    let mut line = 1;
    let mut character = 1;

    for (i, ch) in source.char_indices() {
        if i >= offset {
            break;
        }
        if ch == '\n' {
            line += 1;
            character = 1;
        } else {
            character += 1;
        }
    }

    Position::new(line, character)
}

/// Location in a file
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Location {
    pub file: PathBuf,
    pub range: Range,
}

impl Location {
    pub fn new(file: PathBuf, range: Range) -> Self {
        Self { file, range }
    }
}

/// A suggested fix for a best practice violation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SuggestedFix {
    /// Description of the fix
    pub description: String,
    /// The replacement text (if applicable)
    pub replacement: Option<String>,
}

impl SuggestedFix {
    pub fn new(description: impl Into<String>) -> Self {
        Self {
            description: description.into(),
            replacement: None,
        }
    }

    pub fn with_replacement(mut self, replacement: impl Into<String>) -> Self {
        self.replacement = Some(replacement.into());
        self
    }
}

/// A best practice suggestion
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Suggestion {
    /// Unique rule identifier
    pub rule_id: String,
    /// Short title of the suggestion
    pub title: String,
    /// Category of the best practice
    pub category: PracticeCategory,
    /// Impact level
    pub impact: Impact,
    /// Detailed message
    pub message: String,
    /// Location in file
    pub location: Location,
    /// Optional suggested fix
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fix: Option<SuggestedFix>,
}

impl Suggestion {
    pub fn new(
        rule_id: impl Into<String>,
        title: impl Into<String>,
        category: PracticeCategory,
        impact: Impact,
        message: impl Into<String>,
        location: Location,
    ) -> Self {
        Self {
            rule_id: rule_id.into(),
            title: title.into(),
            category,
            impact,
            message: message.into(),
            location,
            fix: None,
        }
    }

    pub fn with_fix(mut self, fix: SuggestedFix) -> Self {
        self.fix = Some(fix);
        self
    }
}

/// Result of best practices analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisResult {
    /// Files analyzed
    pub files: Vec<PathBuf>,
    /// Suggestions found
    pub suggestions: Vec<Suggestion>,
}

impl AnalysisResult {
    pub fn new() -> Self {
        Self {
            files: Vec::new(),
            suggestions: Vec::new(),
        }
    }

    pub fn add_file(&mut self, file: PathBuf) {
        self.files.push(file);
    }

    pub fn add_suggestion(&mut self, suggestion: Suggestion) {
        self.suggestions.push(suggestion);
    }

    pub fn extend(&mut self, suggestions: impl IntoIterator<Item = Suggestion>) {
        self.suggestions.extend(suggestions);
    }

    pub fn is_empty(&self) -> bool {
        self.suggestions.is_empty()
    }

    pub fn len(&self) -> usize {
        self.suggestions.len()
    }

    pub fn count_by_impact(&self, impact: Impact) -> usize {
        self.suggestions.iter().filter(|s| s.impact == impact).count()
    }

    pub fn count_by_category(&self, category: PracticeCategory) -> usize {
        self.suggestions.iter().filter(|s| s.category == category).count()
    }

    /// Filter suggestions by minimum impact
    pub fn filter_by_impact(&mut self, min_impact: Impact) {
        self.suggestions.retain(|s| s.impact >= min_impact);
    }

    /// Filter suggestions by categories
    pub fn filter_by_categories(&mut self, categories: &[PracticeCategory]) {
        self.suggestions.retain(|s| categories.contains(&s.category));
    }
}

impl Default for AnalysisResult {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_impact_ordering() {
        assert!(Impact::Low < Impact::Medium);
        assert!(Impact::Medium < Impact::High);
    }

    #[test]
    fn test_suggestion_creation() {
        let location = Location::new(
            PathBuf::from("test.wxs"),
            Range::new(Position::new(1, 1), Position::new(1, 10)),
        );

        let suggestion = Suggestion::new(
            "BP001",
            "Missing MajorUpgrade",
            PracticeCategory::Idiom,
            Impact::High,
            "Add MajorUpgrade element for proper upgrade behavior",
            location,
        );

        assert_eq!(suggestion.rule_id, "BP001");
        assert_eq!(suggestion.category, PracticeCategory::Idiom);
        assert_eq!(suggestion.impact, Impact::High);
    }

    #[test]
    fn test_suggestion_with_fix() {
        let location = Location::new(
            PathBuf::from("test.wxs"),
            Range::new(Position::new(1, 1), Position::new(1, 10)),
        );

        let fix = SuggestedFix::new("Use auto-generated GUID")
            .with_replacement("*");

        let suggestion = Suggestion::new(
            "BP002",
            "Hardcoded GUID",
            PracticeCategory::Idiom,
            Impact::Medium,
            "Use Guid=\"*\" for auto-generated GUIDs",
            location,
        )
        .with_fix(fix);

        assert!(suggestion.fix.is_some());
    }

    #[test]
    fn test_analysis_result() {
        let mut result = AnalysisResult::new();

        let location = Location::new(
            PathBuf::from("test.wxs"),
            Range::new(Position::new(1, 1), Position::new(1, 10)),
        );

        result.add_suggestion(Suggestion::new(
            "BP001",
            "Test",
            PracticeCategory::Idiom,
            Impact::High,
            "Test message",
            location.clone(),
        ));

        result.add_suggestion(Suggestion::new(
            "BP002",
            "Test 2",
            PracticeCategory::Efficiency,
            Impact::Low,
            "Test message 2",
            location,
        ));

        assert_eq!(result.len(), 2);
        assert_eq!(result.count_by_impact(Impact::High), 1);
        assert_eq!(result.count_by_category(PracticeCategory::Idiom), 1);
    }

    #[test]
    fn test_range_from_offsets() {
        let source = "line1\nline2\nline3";
        let range = Range::from_offsets(source, 6, 11); // "line2"

        assert_eq!(range.start.line, 2);
        assert_eq!(range.start.character, 1);
    }

    #[test]
    fn test_filter_by_impact() {
        let mut result = AnalysisResult::new();
        let location = Location::new(
            PathBuf::from("test.wxs"),
            Range::new(Position::new(1, 1), Position::new(1, 10)),
        );

        result.add_suggestion(Suggestion::new(
            "BP001", "High", PracticeCategory::Idiom, Impact::High, "msg", location.clone(),
        ));
        result.add_suggestion(Suggestion::new(
            "BP002", "Medium", PracticeCategory::Idiom, Impact::Medium, "msg", location.clone(),
        ));
        result.add_suggestion(Suggestion::new(
            "BP003", "Low", PracticeCategory::Idiom, Impact::Low, "msg", location,
        ));

        result.filter_by_impact(Impact::Medium);
        assert_eq!(result.len(), 2); // High and Medium
    }
}
