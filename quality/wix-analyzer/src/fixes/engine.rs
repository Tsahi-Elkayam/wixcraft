//! Fix application engine

use crate::core::{Diagnostic, Fix, FixAction, Range};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Error during fix application
#[derive(Debug)]
pub struct FixError {
    pub message: String,
    pub file: PathBuf,
}

impl std::fmt::Display for FixError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.file.display(), self.message)
    }
}

impl std::error::Error for FixError {}

/// Preview of what a fix would change
#[derive(Debug, Clone)]
pub struct FixPreview {
    pub file: PathBuf,
    pub rule_id: String,
    pub description: String,
    pub line: usize,
    pub before: String,
    pub after: String,
}

/// Result of applying fixes to a file
#[derive(Debug)]
pub struct FixResult {
    pub file: PathBuf,
    pub fixes_applied: usize,
    pub new_content: String,
}

/// Engine for collecting and applying fixes
pub struct FixEngine {
    fixes: HashMap<PathBuf, Vec<(Diagnostic, Fix)>>,
}

impl FixEngine {
    pub fn new() -> Self {
        Self {
            fixes: HashMap::new(),
        }
    }

    /// Collect fixes from diagnostics
    pub fn collect_fixes(&mut self, diagnostics: &[Diagnostic]) {
        for diag in diagnostics {
            if let Some(fix) = &diag.fix {
                self.fixes
                    .entry(diag.location.file.clone())
                    .or_default()
                    .push((diag.clone(), fix.clone()));
            }
        }
    }

    /// Get the number of fixes available
    pub fn fix_count(&self) -> usize {
        self.fixes.values().map(|v| v.len()).sum()
    }

    /// Preview fixes for a file
    pub fn preview(&self, file: &Path, source: &str) -> Vec<FixPreview> {
        let mut previews = Vec::new();

        if let Some(fixes) = self.fixes.get(file) {
            for (diag, fix) in fixes {
                if let Some(preview) = self.create_preview(source, diag, fix) {
                    previews.push(preview);
                }
            }
        }

        previews
    }

    fn create_preview(&self, source: &str, diag: &Diagnostic, fix: &Fix) -> Option<FixPreview> {
        let lines: Vec<&str> = source.lines().collect();
        let line_idx = diag.location.range.start.line.saturating_sub(1);

        if line_idx >= lines.len() {
            return None;
        }

        let before = lines[line_idx].to_string();
        let after = self.apply_action_to_line(&before, &diag.location.range, &fix.action)?;

        Some(FixPreview {
            file: diag.location.file.clone(),
            rule_id: diag.rule_id.clone(),
            description: fix.description.clone(),
            line: diag.location.range.start.line,
            before,
            after,
        })
    }

    /// Apply all fixes to a file
    pub fn apply(&self, file: &Path, source: &str) -> Result<FixResult, FixError> {
        let fixes = match self.fixes.get(file) {
            Some(f) => f,
            None => {
                return Ok(FixResult {
                    file: file.to_path_buf(),
                    fixes_applied: 0,
                    new_content: source.to_string(),
                })
            }
        };

        // Sort fixes by position (reverse order to apply from bottom to top)
        let mut sorted_fixes: Vec<_> = fixes.iter().collect();
        sorted_fixes.sort_by(|a, b| {
            b.0.location.range.start.line.cmp(&a.0.location.range.start.line)
                .then(b.0.location.range.start.character.cmp(&a.0.location.range.start.character))
        });

        let mut content = source.to_string();
        let mut applied = 0;

        for (diag, fix) in sorted_fixes {
            if let Some(new_content) = self.apply_fix(&content, &diag.location.range, &fix.action) {
                content = new_content;
                applied += 1;
            }
        }

        Ok(FixResult {
            file: file.to_path_buf(),
            fixes_applied: applied,
            new_content: content,
        })
    }

    fn apply_fix(&self, source: &str, range: &Range, action: &FixAction) -> Option<String> {
        match action {
            FixAction::ReplaceAttribute { name, new_value, .. } => {
                self.replace_attribute(source, range, name, new_value)
            }
            FixAction::AddAttribute { name, value, .. } => {
                self.add_attribute(source, range, name, value)
            }
            FixAction::RemoveElement { range } => {
                self.remove_range(source, range)
            }
            FixAction::AddElement { element, .. } => {
                self.add_child_element(source, range, element)
            }
            FixAction::ReplaceText { new_text, .. } => {
                self.replace_range(source, range, new_text)
            }
            FixAction::RemoveAttribute { name, .. } => {
                self.remove_attribute(source, range, name)
            }
        }
    }

    fn apply_action_to_line(&self, line: &str, _range: &Range, action: &FixAction) -> Option<String> {
        match action {
            FixAction::ReplaceAttribute { name, new_value, .. } => {
                // Simple regex replacement for preview
                let pattern = format!(r#"{}="[^"]*""#, regex::escape(name));
                let re = regex::Regex::new(&pattern).ok()?;
                let replacement = format!(r#"{}="{}""#, name, new_value);
                Some(re.replace(line, replacement.as_str()).to_string())
            }
            FixAction::RemoveElement { .. } => {
                Some(String::new()) // Line would be removed
            }
            FixAction::AddElement { element, .. } => {
                // Show element being added
                Some(format!("{}\n    {}", line, element))
            }
            _ => Some(line.to_string()),
        }
    }

    fn replace_attribute(&self, source: &str, range: &Range, name: &str, new_value: &str) -> Option<String> {
        // Find the line containing the attribute
        let lines: Vec<&str> = source.lines().collect();
        let line_idx = range.start.line.saturating_sub(1);

        if line_idx >= lines.len() {
            return None;
        }

        let line = lines[line_idx];
        let pattern = format!(r#"{}="[^"]*""#, regex::escape(name));
        let re = regex::Regex::new(&pattern).ok()?;

        if !re.is_match(line) {
            return None;
        }

        let replacement = format!(r#"{}="{}""#, name, new_value);
        let new_line = re.replace(line, replacement.as_str());

        let result: Vec<&str> = lines.clone();
        let new_line_str = new_line.to_string();

        let mut output = String::new();
        for (i, l) in result.iter().enumerate() {
            if i == line_idx {
                output.push_str(&new_line_str);
            } else {
                output.push_str(l);
            }
            if i < lines.len() - 1 {
                output.push('\n');
            }
        }

        Some(output)
    }

    fn add_attribute(&self, source: &str, range: &Range, name: &str, value: &str) -> Option<String> {
        let lines: Vec<&str> = source.lines().collect();
        let line_idx = range.start.line.saturating_sub(1);

        if line_idx >= lines.len() {
            return None;
        }

        let line = lines[line_idx];

        // Find position before > or />
        let insert_pos = line.rfind("/>").or_else(|| line.rfind('>'))?;
        let new_line = format!(
            "{} {}=\"{}\"{}",
            &line[..insert_pos].trim_end(),
            name,
            value,
            &line[insert_pos..]
        );

        let mut output = String::new();
        for (i, l) in lines.iter().enumerate() {
            if i == line_idx {
                output.push_str(&new_line);
            } else {
                output.push_str(l);
            }
            if i < lines.len() - 1 {
                output.push('\n');
            }
        }

        Some(output)
    }

    fn remove_range(&self, source: &str, range: &Range) -> Option<String> {
        let lines: Vec<&str> = source.lines().collect();
        let start_line = range.start.line.saturating_sub(1);
        let end_line = range.end.line.saturating_sub(1);

        let mut output = String::new();
        for (i, line) in lines.iter().enumerate() {
            if i < start_line || i > end_line {
                output.push_str(line);
                if i < lines.len() - 1 {
                    output.push('\n');
                }
            }
        }

        Some(output)
    }

    fn add_child_element(&self, source: &str, range: &Range, element: &str) -> Option<String> {
        let lines: Vec<&str> = source.lines().collect();
        let line_idx = range.start.line.saturating_sub(1);

        if line_idx >= lines.len() {
            return None;
        }

        let line = lines[line_idx];

        // Detect indentation
        let indent = line.len() - line.trim_start().len();
        let child_indent = " ".repeat(indent + 4);

        // If self-closing, need to convert to open/close
        if line.trim().ends_with("/>") {
            // Convert <Element ... /> to <Element ...>...</Element>
            let close_pos = line.rfind("/>").unwrap();
            let tag_name = extract_tag_name(line);
            let new_line = format!(
                "{}>\n{}{}\n{}</{}>",
                &line[..close_pos].trim_end(),
                child_indent,
                element,
                " ".repeat(indent),
                tag_name
            );

            let mut output = String::new();
            for (i, l) in lines.iter().enumerate() {
                if i == line_idx {
                    output.push_str(&new_line);
                } else {
                    output.push_str(l);
                }
                if i < lines.len() - 1 {
                    output.push('\n');
                }
            }
            return Some(output);
        }

        // Insert after opening tag
        let mut output = String::new();
        for (i, l) in lines.iter().enumerate() {
            output.push_str(l);
            if i == line_idx {
                output.push('\n');
                output.push_str(&child_indent);
                output.push_str(element);
            }
            if i < lines.len() - 1 {
                output.push('\n');
            }
        }

        Some(output)
    }

    fn replace_range(&self, source: &str, range: &Range, new_text: &str) -> Option<String> {
        // Simple line replacement for now
        let lines: Vec<&str> = source.lines().collect();
        let line_idx = range.start.line.saturating_sub(1);

        if line_idx >= lines.len() {
            return None;
        }

        let mut output = String::new();
        for (i, line) in lines.iter().enumerate() {
            if i == line_idx {
                output.push_str(new_text);
            } else {
                output.push_str(line);
            }
            if i < lines.len() - 1 {
                output.push('\n');
            }
        }

        Some(output)
    }

    fn remove_attribute(&self, source: &str, range: &Range, name: &str) -> Option<String> {
        let lines: Vec<&str> = source.lines().collect();
        let line_idx = range.start.line.saturating_sub(1);

        if line_idx >= lines.len() {
            return None;
        }

        let line = lines[line_idx];
        let pattern = format!(r#"\s*{}="[^"]*""#, regex::escape(name));
        let re = regex::Regex::new(&pattern).ok()?;
        let new_line = re.replace(line, "");

        let mut output = String::new();
        for (i, l) in lines.iter().enumerate() {
            if i == line_idx {
                output.push_str(&new_line);
            } else {
                output.push_str(l);
            }
            if i < lines.len() - 1 {
                output.push('\n');
            }
        }

        Some(output)
    }
}

impl Default for FixEngine {
    fn default() -> Self {
        Self::new()
    }
}

fn extract_tag_name(line: &str) -> &str {
    let trimmed = line.trim();
    if let Some(start) = trimmed.find('<') {
        let rest = &trimmed[start + 1..];
        if let Some(end) = rest.find(|c: char| c.is_whitespace() || c == '>' || c == '/') {
            return &rest[..end];
        }
    }
    ""
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{Category, Location, Position};

    fn make_diagnostic(line: usize, rule_id: &str, fix: Fix) -> Diagnostic {
        Diagnostic::warning(
            rule_id,
            Category::BestPractice,
            "Test",
            Location::new(
                PathBuf::from("test.wxs"),
                Range::new(Position::new(line, 1), Position::new(line, 50)),
            ),
        )
        .with_fix(fix)
    }

    #[test]
    fn test_replace_attribute() {
        let source = r#"<Component Id="C1" Guid="{12345678-1234-1234-1234-123456789ABC}" />"#;
        let diag = make_diagnostic(
            1,
            "BP-IDIOM-002",
            Fix::new(
                "Use auto GUID",
                FixAction::ReplaceAttribute {
                    range: Range::new(Position::new(1, 1), Position::new(1, 70)),
                    name: "Guid".to_string(),
                    new_value: "*".to_string(),
                },
            ),
        );

        let mut engine = FixEngine::new();
        engine.collect_fixes(&[diag]);

        let result = engine.apply(Path::new("test.wxs"), source).unwrap();
        assert!(result.new_content.contains(r#"Guid="*""#));
        assert_eq!(result.fixes_applied, 1);
    }

    #[test]
    fn test_preview() {
        let source = r#"<Component Id="C1" Guid="{12345678-1234-1234-1234-123456789ABC}" />"#;
        let diag = make_diagnostic(
            1,
            "BP-IDIOM-002",
            Fix::new(
                "Use auto GUID",
                FixAction::ReplaceAttribute {
                    range: Range::new(Position::new(1, 1), Position::new(1, 70)),
                    name: "Guid".to_string(),
                    new_value: "*".to_string(),
                },
            ),
        );

        let mut engine = FixEngine::new();
        engine.collect_fixes(&[diag]);

        let previews = engine.preview(Path::new("test.wxs"), source);
        assert_eq!(previews.len(), 1);
        assert!(previews[0].after.contains(r#"Guid="*""#));
    }

    #[test]
    fn test_fix_count() {
        let diag = make_diagnostic(
            1,
            "TEST",
            Fix::new("Test", FixAction::RemoveElement {
                range: Range::new(Position::new(1, 1), Position::new(1, 10)),
            }),
        );

        let mut engine = FixEngine::new();
        assert_eq!(engine.fix_count(), 0);

        engine.collect_fixes(&[diag]);
        assert_eq!(engine.fix_count(), 1);
    }

    #[test]
    fn test_add_attribute() {
        let source = r#"<Component Id="C1" />"#;
        let diag = make_diagnostic(
            1,
            "TEST",
            Fix::new(
                "Add Guid",
                FixAction::AddAttribute {
                    range: Range::new(Position::new(1, 1), Position::new(1, 21)),
                    name: "Guid".to_string(),
                    value: "*".to_string(),
                },
            ),
        );

        let mut engine = FixEngine::new();
        engine.collect_fixes(&[diag]);

        let result = engine.apply(Path::new("test.wxs"), source).unwrap();
        assert!(result.new_content.contains(r#"Guid="*""#));
    }

    #[test]
    fn test_remove_element() {
        let source = "line1\nremove this\nline3";
        let diag = make_diagnostic(
            2,
            "TEST",
            Fix::new(
                "Remove",
                FixAction::RemoveElement {
                    range: Range::new(Position::new(2, 1), Position::new(2, 12)),
                },
            ),
        );

        let mut engine = FixEngine::new();
        engine.collect_fixes(&[diag]);

        let result = engine.apply(Path::new("test.wxs"), source).unwrap();
        assert!(!result.new_content.contains("remove this"));
    }

    #[test]
    fn test_remove_attribute() {
        let source = r#"<Component Id="C1" Guid="{OLD}" />"#;
        let diag = make_diagnostic(
            1,
            "TEST",
            Fix::new(
                "Remove Guid",
                FixAction::RemoveAttribute {
                    range: Range::new(Position::new(1, 1), Position::new(1, 35)),
                    name: "Guid".to_string(),
                },
            ),
        );

        let mut engine = FixEngine::new();
        engine.collect_fixes(&[diag]);

        let result = engine.apply(Path::new("test.wxs"), source).unwrap();
        assert!(!result.new_content.contains("Guid="));
    }

    #[test]
    fn test_replace_text() {
        let source = "old line\nkeep this";
        let diag = make_diagnostic(
            1,
            "TEST",
            Fix::new(
                "Replace",
                FixAction::ReplaceText {
                    range: Range::new(Position::new(1, 1), Position::new(1, 9)),
                    new_text: "new line".to_string(),
                },
            ),
        );

        let mut engine = FixEngine::new();
        engine.collect_fixes(&[diag]);

        let result = engine.apply(Path::new("test.wxs"), source).unwrap();
        assert!(result.new_content.contains("new line"));
    }

    #[test]
    fn test_add_child_element_self_closing() {
        let source = r#"<Package Name="Test" />"#;
        let diag = make_diagnostic(
            1,
            "TEST",
            Fix::new(
                "Add MajorUpgrade",
                FixAction::AddElement {
                    parent_range: Range::new(Position::new(1, 1), Position::new(1, 24)),
                    element: r#"<MajorUpgrade />"#.to_string(),
                    position: crate::core::InsertPosition::First,
                },
            ),
        );

        let mut engine = FixEngine::new();
        engine.collect_fixes(&[diag]);

        let result = engine.apply(Path::new("test.wxs"), source).unwrap();
        assert!(result.new_content.contains("MajorUpgrade"));
    }

    #[test]
    fn test_apply_no_fixes() {
        let engine = FixEngine::new();
        let source = "test content";
        let result = engine.apply(Path::new("other.wxs"), source).unwrap();
        assert_eq!(result.fixes_applied, 0);
        assert_eq!(result.new_content, source);
    }

    #[test]
    fn test_fix_error_display() {
        let err = FixError {
            message: "test error".to_string(),
            file: PathBuf::from("test.wxs"),
        };
        assert!(err.to_string().contains("test.wxs"));
        assert!(err.to_string().contains("test error"));
    }

    #[test]
    fn test_default_engine() {
        let engine = FixEngine::default();
        assert_eq!(engine.fix_count(), 0);
    }

    #[test]
    fn test_extract_tag_name() {
        assert_eq!(extract_tag_name("<Package Name=\"Test\">"), "Package");
        assert_eq!(extract_tag_name("<Component Id=\"C1\" />"), "Component");
        assert_eq!(extract_tag_name("  <Directory>"), "Directory");
    }

    #[test]
    fn test_preview_nonexistent_file() {
        let engine = FixEngine::new();
        let previews = engine.preview(Path::new("nonexistent.wxs"), "content");
        assert!(previews.is_empty());
    }

    #[test]
    fn test_multiple_fixes_same_file() {
        let source = r#"<A Id="1" Guid="{OLD1}" />
<B Id="2" Guid="{OLD2}" />"#;

        let diag1 = make_diagnostic(
            1,
            "TEST",
            Fix::new(
                "Fix 1",
                FixAction::ReplaceAttribute {
                    range: Range::new(Position::new(1, 1), Position::new(1, 25)),
                    name: "Guid".to_string(),
                    new_value: "*".to_string(),
                },
            ),
        );

        let diag2 = make_diagnostic(
            2,
            "TEST",
            Fix::new(
                "Fix 2",
                FixAction::ReplaceAttribute {
                    range: Range::new(Position::new(2, 1), Position::new(2, 25)),
                    name: "Guid".to_string(),
                    new_value: "*".to_string(),
                },
            ),
        );

        let mut engine = FixEngine::new();
        engine.collect_fixes(&[diag1, diag2]);

        assert_eq!(engine.fix_count(), 2);
        let result = engine.apply(Path::new("test.wxs"), source).unwrap();
        assert_eq!(result.fixes_applied, 2);
    }

    #[test]
    fn test_collect_fixes_without_fix() {
        // Diagnostics without fixes should be skipped
        let diag = Diagnostic::warning(
            "TEST",
            Category::BestPractice,
            "Test",
            Location::new(
                PathBuf::from("test.wxs"),
                Range::new(Position::new(1, 1), Position::new(1, 10)),
            ),
        );

        let mut engine = FixEngine::new();
        engine.collect_fixes(&[diag]);
        assert_eq!(engine.fix_count(), 0);
    }

    #[test]
    fn test_preview_out_of_range() {
        let source = "single line";
        let diag = make_diagnostic(
            100, // Line 100 doesn't exist
            "TEST",
            Fix::new(
                "Test",
                FixAction::ReplaceAttribute {
                    range: Range::new(Position::new(100, 1), Position::new(100, 10)),
                    name: "Attr".to_string(),
                    new_value: "value".to_string(),
                },
            ),
        );

        let mut engine = FixEngine::new();
        engine.collect_fixes(&[diag]);

        let previews = engine.preview(Path::new("test.wxs"), source);
        assert!(previews.is_empty());
    }

    #[test]
    fn test_replace_attribute_no_match() {
        let source = r#"<Component Id="C1" />"#;
        let diag = make_diagnostic(
            1,
            "TEST",
            Fix::new(
                "Replace nonexistent",
                FixAction::ReplaceAttribute {
                    range: Range::new(Position::new(1, 1), Position::new(1, 22)),
                    name: "NonExistent".to_string(),
                    new_value: "value".to_string(),
                },
            ),
        );

        let mut engine = FixEngine::new();
        engine.collect_fixes(&[diag]);

        let result = engine.apply(Path::new("test.wxs"), source).unwrap();
        // Fix fails silently - content unchanged
        assert_eq!(result.fixes_applied, 0);
    }

    #[test]
    fn test_add_attribute_with_closing_tag() {
        let source = r#"<Component Id="C1"></Component>"#;
        let diag = make_diagnostic(
            1,
            "TEST",
            Fix::new(
                "Add Guid",
                FixAction::AddAttribute {
                    range: Range::new(Position::new(1, 1), Position::new(1, 32)),
                    name: "Guid".to_string(),
                    value: "*".to_string(),
                },
            ),
        );

        let mut engine = FixEngine::new();
        engine.collect_fixes(&[diag]);

        let result = engine.apply(Path::new("test.wxs"), source).unwrap();
        assert!(result.new_content.contains(r#"Guid="*""#));
    }

    #[test]
    fn test_add_child_element_open_tag() {
        let source = "<Package Name=\"Test\">\n</Package>";
        let diag = make_diagnostic(
            1,
            "TEST",
            Fix::new(
                "Add Child",
                FixAction::AddElement {
                    parent_range: Range::new(Position::new(1, 1), Position::new(1, 22)),
                    element: "<Child />".to_string(),
                    position: crate::core::InsertPosition::First,
                },
            ),
        );

        let mut engine = FixEngine::new();
        engine.collect_fixes(&[diag]);

        let result = engine.apply(Path::new("test.wxs"), source).unwrap();
        assert!(result.new_content.contains("<Child />"));
    }

    #[test]
    fn test_preview_remove_element() {
        let source = "<Element to remove />";
        let diag = make_diagnostic(
            1,
            "TEST",
            Fix::new(
                "Remove",
                FixAction::RemoveElement {
                    range: Range::new(Position::new(1, 1), Position::new(1, 22)),
                },
            ),
        );

        let mut engine = FixEngine::new();
        engine.collect_fixes(&[diag]);

        let previews = engine.preview(Path::new("test.wxs"), source);
        assert_eq!(previews.len(), 1);
        assert!(previews[0].after.is_empty());
    }

    #[test]
    fn test_preview_add_element() {
        let source = "<Package />";
        let diag = make_diagnostic(
            1,
            "TEST",
            Fix::new(
                "Add Child",
                FixAction::AddElement {
                    parent_range: Range::new(Position::new(1, 1), Position::new(1, 12)),
                    element: "<Child />".to_string(),
                    position: crate::core::InsertPosition::First,
                },
            ),
        );

        let mut engine = FixEngine::new();
        engine.collect_fixes(&[diag]);

        let previews = engine.preview(Path::new("test.wxs"), source);
        assert_eq!(previews.len(), 1);
        assert!(previews[0].after.contains("<Child />"));
    }

    #[test]
    fn test_preview_add_attribute_fallback() {
        let source = "<Component Id=\"C1\" />";
        let diag = make_diagnostic(
            1,
            "TEST",
            Fix::new(
                "Add",
                FixAction::AddAttribute {
                    range: Range::new(Position::new(1, 1), Position::new(1, 22)),
                    name: "Guid".to_string(),
                    value: "*".to_string(),
                },
            ),
        );

        let mut engine = FixEngine::new();
        engine.collect_fixes(&[diag]);

        let previews = engine.preview(Path::new("test.wxs"), source);
        assert_eq!(previews.len(), 1);
        // AddAttribute in preview just returns unchanged line
        assert_eq!(previews[0].after, source);
    }

    #[test]
    fn test_extract_tag_name_edge_cases() {
        assert_eq!(extract_tag_name("<Tag/>"), "Tag");
        assert_eq!(extract_tag_name("   <  >"), "");
        assert_eq!(extract_tag_name("no tag"), "");
        assert_eq!(extract_tag_name("<SomeElement>"), "SomeElement");
    }

    #[test]
    fn test_apply_out_of_range() {
        let source = "single line";
        let diag = make_diagnostic(
            100, // Line 100 doesn't exist
            "TEST",
            Fix::new(
                "Replace",
                FixAction::ReplaceAttribute {
                    range: Range::new(Position::new(100, 1), Position::new(100, 10)),
                    name: "Attr".to_string(),
                    new_value: "value".to_string(),
                },
            ),
        );

        let mut engine = FixEngine::new();
        engine.collect_fixes(&[diag]);

        let result = engine.apply(Path::new("test.wxs"), source).unwrap();
        // Fix fails due to out of range - content unchanged
        assert_eq!(result.fixes_applied, 0);
        assert_eq!(result.new_content, source);
    }

    #[test]
    fn test_remove_range_multi_line() {
        let source = "line1\nline2 to remove\nline3 to remove\nline4";
        let diag = make_diagnostic(
            2,
            "TEST",
            Fix::new(
                "Remove lines",
                FixAction::RemoveElement {
                    range: Range::new(Position::new(2, 1), Position::new(3, 20)),
                },
            ),
        );

        let mut engine = FixEngine::new();
        engine.collect_fixes(&[diag]);

        let result = engine.apply(Path::new("test.wxs"), source).unwrap();
        assert!(!result.new_content.contains("line2"));
        assert!(!result.new_content.contains("line3"));
        assert!(result.new_content.contains("line1"));
        assert!(result.new_content.contains("line4"));
    }

    #[test]
    fn test_add_attribute_no_closing_bracket() {
        let source = r#"<Component Id="C1""#; // Malformed - no closing bracket
        let diag = make_diagnostic(
            1,
            "TEST",
            Fix::new(
                "Add Guid",
                FixAction::AddAttribute {
                    range: Range::new(Position::new(1, 1), Position::new(1, 19)),
                    name: "Guid".to_string(),
                    value: "*".to_string(),
                },
            ),
        );

        let mut engine = FixEngine::new();
        engine.collect_fixes(&[diag]);

        let result = engine.apply(Path::new("test.wxs"), source).unwrap();
        // Can't add attribute without closing bracket
        assert_eq!(result.fixes_applied, 0);
    }

    #[test]
    fn test_fix_result_fields() {
        let result = FixResult {
            file: PathBuf::from("test.wxs"),
            fixes_applied: 5,
            new_content: "content".to_string(),
        };

        assert_eq!(result.file, PathBuf::from("test.wxs"));
        assert_eq!(result.fixes_applied, 5);
        assert_eq!(result.new_content, "content");
    }

    #[test]
    fn test_fix_preview_fields() {
        let preview = FixPreview {
            file: PathBuf::from("test.wxs"),
            rule_id: "RULE-001".to_string(),
            description: "Fix it".to_string(),
            line: 10,
            before: "old".to_string(),
            after: "new".to_string(),
        };

        assert_eq!(preview.file, PathBuf::from("test.wxs"));
        assert_eq!(preview.rule_id, "RULE-001");
        assert_eq!(preview.line, 10);
    }

    #[test]
    fn test_fix_error_is_error_trait() {
        let err = FixError {
            message: "test".to_string(),
            file: PathBuf::from("test.wxs"),
        };
        // Verify Error trait is implemented
        let _: &dyn std::error::Error = &err;
    }

    #[test]
    fn test_add_attribute_out_of_range() {
        let source = "single line";
        let diag = make_diagnostic(
            100, // Line doesn't exist
            "TEST",
            Fix::new(
                "Add attr",
                FixAction::AddAttribute {
                    range: Range::new(Position::new(100, 1), Position::new(100, 10)),
                    name: "Attr".to_string(),
                    value: "value".to_string(),
                },
            ),
        );

        let mut engine = FixEngine::new();
        engine.collect_fixes(&[diag]);

        let result = engine.apply(Path::new("test.wxs"), source).unwrap();
        assert_eq!(result.fixes_applied, 0);
    }

    #[test]
    fn test_add_child_element_out_of_range() {
        let source = "single line";
        let diag = make_diagnostic(
            100, // Line doesn't exist
            "TEST",
            Fix::new(
                "Add element",
                FixAction::AddElement {
                    parent_range: Range::new(Position::new(100, 1), Position::new(100, 10)),
                    element: "<Child />".to_string(),
                    position: crate::core::InsertPosition::First,
                },
            ),
        );

        let mut engine = FixEngine::new();
        engine.collect_fixes(&[diag]);

        let result = engine.apply(Path::new("test.wxs"), source).unwrap();
        assert_eq!(result.fixes_applied, 0);
    }

    #[test]
    fn test_replace_text_out_of_range() {
        let source = "single line";
        let diag = make_diagnostic(
            100, // Line doesn't exist
            "TEST",
            Fix::new(
                "Replace",
                FixAction::ReplaceText {
                    range: Range::new(Position::new(100, 1), Position::new(100, 10)),
                    new_text: "new".to_string(),
                },
            ),
        );

        let mut engine = FixEngine::new();
        engine.collect_fixes(&[diag]);

        let result = engine.apply(Path::new("test.wxs"), source).unwrap();
        assert_eq!(result.fixes_applied, 0);
    }

    #[test]
    fn test_remove_attribute_out_of_range() {
        let source = "single line";
        let diag = make_diagnostic(
            100, // Line doesn't exist
            "TEST",
            Fix::new(
                "Remove attr",
                FixAction::RemoveAttribute {
                    range: Range::new(Position::new(100, 1), Position::new(100, 10)),
                    name: "Attr".to_string(),
                },
            ),
        );

        let mut engine = FixEngine::new();
        engine.collect_fixes(&[diag]);

        let result = engine.apply(Path::new("test.wxs"), source).unwrap();
        assert_eq!(result.fixes_applied, 0);
    }

    #[test]
    fn test_add_attribute_multiline() {
        // Multi-line source to cover the else branch in the loop
        let source = "line1\n<Component Id=\"C1\" />\nline3";
        let diag = make_diagnostic(
            2, // Target line 2
            "TEST",
            Fix::new(
                "Add Guid",
                FixAction::AddAttribute {
                    range: Range::new(Position::new(2, 1), Position::new(2, 22)),
                    name: "Guid".to_string(),
                    value: "*".to_string(),
                },
            ),
        );

        let mut engine = FixEngine::new();
        engine.collect_fixes(&[diag]);

        let result = engine.apply(Path::new("test.wxs"), source).unwrap();
        assert!(result.new_content.contains(r#"Guid="*""#));
        assert!(result.new_content.contains("line1"));
        assert!(result.new_content.contains("line3"));
    }

    #[test]
    fn test_add_child_element_self_closing_multiline() {
        // Multi-line source with self-closing element to cover the else branch
        let source = "line1\n<Package Name=\"Test\" />\nline3";
        let diag = make_diagnostic(
            2, // Target line 2
            "TEST",
            Fix::new(
                "Add MajorUpgrade",
                FixAction::AddElement {
                    parent_range: Range::new(Position::new(2, 1), Position::new(2, 24)),
                    element: r#"<MajorUpgrade />"#.to_string(),
                    position: crate::core::InsertPosition::First,
                },
            ),
        );

        let mut engine = FixEngine::new();
        engine.collect_fixes(&[diag]);

        let result = engine.apply(Path::new("test.wxs"), source).unwrap();
        assert!(result.new_content.contains("MajorUpgrade"));
        assert!(result.new_content.contains("line1"));
        assert!(result.new_content.contains("line3"));
    }

    #[test]
    fn test_remove_attribute_multiline() {
        // Multi-line source to cover the else branch in the loop
        let source = "line1\n<Component Id=\"C1\" Guid=\"{OLD}\" />\nline3";
        let diag = make_diagnostic(
            2, // Target line 2
            "TEST",
            Fix::new(
                "Remove Guid",
                FixAction::RemoveAttribute {
                    range: Range::new(Position::new(2, 1), Position::new(2, 40)),
                    name: "Guid".to_string(),
                },
            ),
        );

        let mut engine = FixEngine::new();
        engine.collect_fixes(&[diag]);

        let result = engine.apply(Path::new("test.wxs"), source).unwrap();
        assert!(!result.new_content.contains("Guid="));
        assert!(result.new_content.contains("line1"));
        assert!(result.new_content.contains("line3"));
    }

    #[test]
    fn test_replace_range_multiline() {
        // Multi-line source to cover the else branch
        let source = "line1\nold line\nline3";
        let diag = make_diagnostic(
            2, // Target line 2
            "TEST",
            Fix::new(
                "Replace",
                FixAction::ReplaceText {
                    range: Range::new(Position::new(2, 1), Position::new(2, 9)),
                    new_text: "new line".to_string(),
                },
            ),
        );

        let mut engine = FixEngine::new();
        engine.collect_fixes(&[diag]);

        let result = engine.apply(Path::new("test.wxs"), source).unwrap();
        assert!(result.new_content.contains("new line"));
        assert!(result.new_content.contains("line1"));
        assert!(result.new_content.contains("line3"));
    }

    #[test]
    fn test_fixes_sorted_by_line_and_character() {
        // Test that fixes on same line are sorted by character (reverse order)
        let source = r#"<Component Id="C1" Guid="{OLD1}" Other="{OLD2}" />"#;

        // Create a diagnostic with a specific position
        fn make_diagnostic_at(line: usize, col: usize, rule_id: &str, fix: Fix) -> Diagnostic {
            Diagnostic::warning(
                rule_id,
                Category::BestPractice,
                "Test",
                Location::new(
                    PathBuf::from("test.wxs"),
                    Range::new(Position::new(line, col), Position::new(line, col + 10)),
                ),
            )
            .with_fix(fix)
        }

        // Two fixes on the same line at different character positions
        let diag1 = make_diagnostic_at(
            1,
            20, // Earlier in line
            "TEST1",
            Fix::new(
                "Fix 1",
                FixAction::ReplaceAttribute {
                    range: Range::new(Position::new(1, 20), Position::new(1, 30)),
                    name: "Guid".to_string(),
                    new_value: "*".to_string(),
                },
            ),
        );

        let diag2 = make_diagnostic_at(
            1,
            35, // Later in line
            "TEST2",
            Fix::new(
                "Fix 2",
                FixAction::ReplaceAttribute {
                    range: Range::new(Position::new(1, 35), Position::new(1, 45)),
                    name: "Other".to_string(),
                    new_value: "NEW".to_string(),
                },
            ),
        );

        let mut engine = FixEngine::new();
        engine.collect_fixes(&[diag1, diag2]);

        // Both fixes should be applied (sorted in reverse order to apply from end to start)
        let result = engine.apply(Path::new("test.wxs"), source).unwrap();
        // Check that at least one fix was applied
        assert!(result.fixes_applied >= 1);
    }
}
