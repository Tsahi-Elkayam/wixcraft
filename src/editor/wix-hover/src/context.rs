//! Cursor context detection for hover

use crate::types::{HoverTarget, Range};

/// Detect what the cursor is hovering over
pub fn detect_hover_target(source: &str, line: u32, column: u32) -> HoverTarget {
    let offset = line_col_to_offset(source, line, column);
    let chars: Vec<char> = source.chars().collect();

    if offset >= chars.len() {
        return HoverTarget::None;
    }

    // Find if we're inside a tag
    if let Some((tag_start, tag_end)) = find_enclosing_tag(&chars, offset) {
        return parse_hover_in_tag(&chars, tag_start, tag_end, offset, line, column, source);
    }

    // Check if we're on a closing tag element name
    if let Some(target) = check_closing_tag(&chars, offset, line, source) {
        return target;
    }

    HoverTarget::None
}

/// Convert line/column (1-based) to byte offset
fn line_col_to_offset(source: &str, line: u32, column: u32) -> usize {
    let mut current_line = 1u32;
    let mut current_col = 1u32;

    for (i, ch) in source.char_indices() {
        if current_line == line && current_col == column {
            return i;
        }

        if ch == '\n' {
            current_line += 1;
            current_col = 1;
        } else {
            current_col += 1;
        }
    }

    source.len()
}

/// Convert byte offset to line/column (1-based)
fn offset_to_line_col(source: &str, target_offset: usize) -> (u32, u32) {
    let mut line = 1u32;
    let mut col = 1u32;

    for (i, ch) in source.char_indices() {
        if i == target_offset {
            return (line, col);
        }

        if ch == '\n' {
            line += 1;
            col = 1;
        } else {
            col += 1;
        }
    }

    (line, col)
}

/// Find the enclosing tag boundaries (start '<' to end '>')
fn find_enclosing_tag(chars: &[char], offset: usize) -> Option<(usize, usize)> {
    // Search backwards for '<'
    let mut start = offset;
    while start > 0 {
        if chars[start] == '<' {
            break;
        }
        if chars[start] == '>' {
            return None; // We're outside a tag
        }
        start -= 1;
    }

    if chars[start] != '<' {
        return None;
    }

    // Search forwards for '>'
    let mut end = offset;
    while end < chars.len() {
        if chars[end] == '>' {
            return Some((start, end));
        }
        end += 1;
    }

    // Unclosed tag
    Some((start, chars.len() - 1))
}

/// Parse hover target when inside a tag
fn parse_hover_in_tag(
    chars: &[char],
    tag_start: usize,
    tag_end: usize,
    offset: usize,
    line: u32,
    _column: u32,
    source: &str,
) -> HoverTarget {
    // Handle closing tags: </ElementName>
    if tag_start + 1 < chars.len() && chars[tag_start + 1] == '/' {
        let name_start = tag_start + 2;
        let mut name_end = name_start;
        while name_end < chars.len() && chars[name_end] != '>' && !chars[name_end].is_whitespace() {
            name_end += 1;
        }

        if offset >= name_start && offset < name_end {
            let element_name: String = chars[name_start..name_end].iter().collect();
            let (start_line, start_col) = offset_to_line_col(source, name_start);
            let (end_line, end_col) = offset_to_line_col(source, name_end);

            return HoverTarget::Element {
                name: element_name,
                range: Range {
                    start_line,
                    start_col,
                    end_line,
                    end_col,
                },
            };
        }
        return HoverTarget::None;
    }

    let tag_content: String = chars[tag_start + 1..=tag_end].iter().collect();

    // Find element name
    let element_end = tag_content
        .find(|c: char| c.is_whitespace() || c == '>' || c == '/')
        .unwrap_or(tag_content.len());
    let element_name = &tag_content[..element_end];

    if element_name.is_empty() {
        return HoverTarget::None;
    }

    // Check if cursor is on element name
    let element_name_end_offset = tag_start + 1 + element_end;
    if offset >= tag_start + 1 && offset < element_name_end_offset {
        let (start_line, start_col) = offset_to_line_col(source, tag_start + 1);
        let (end_line, end_col) = offset_to_line_col(source, element_name_end_offset);

        return HoverTarget::Element {
            name: element_name.to_string(),
            range: Range {
                start_line,
                start_col,
                end_line,
                end_col,
            },
        };
    }

    // Parse attributes to find what we're hovering over
    let attrs_str = &tag_content[element_end..];
    if let Some(target) = find_attribute_hover(
        attrs_str,
        element_name,
        offset - (tag_start + 1 + element_end),
        tag_start + 1 + element_end,
        line,
        source,
    ) {
        return target;
    }

    HoverTarget::None
}

/// Find if hovering over an attribute name or value
fn find_attribute_hover(
    attrs_str: &str,
    element_name: &str,
    relative_offset: usize,
    base_offset: usize,
    _line: u32,
    source: &str,
) -> Option<HoverTarget> {
    let mut pos = 0usize;
    let chars: Vec<char> = attrs_str.chars().collect();

    while pos < chars.len() {
        // Skip whitespace
        while pos < chars.len() && chars[pos].is_whitespace() {
            pos += 1;
        }

        if pos >= chars.len() || chars[pos] == '>' || chars[pos] == '/' {
            break;
        }

        // Parse attribute name
        let attr_start = pos;
        while pos < chars.len() && chars[pos] != '=' && !chars[pos].is_whitespace() {
            pos += 1;
        }
        let attr_end = pos;
        let attr_name: String = chars[attr_start..attr_end].iter().collect();

        if attr_name.is_empty() {
            break;
        }

        // Check if cursor is on attribute name
        if relative_offset >= attr_start && relative_offset < attr_end {
            let (start_line, start_col) = offset_to_line_col(source, base_offset + attr_start);
            let (end_line, end_col) = offset_to_line_col(source, base_offset + attr_end);

            return Some(HoverTarget::AttributeName {
                element: element_name.to_string(),
                attribute: attr_name,
                range: Range {
                    start_line,
                    start_col,
                    end_line,
                    end_col,
                },
            });
        }

        // Skip to '='
        while pos < chars.len() && chars[pos].is_whitespace() {
            pos += 1;
        }

        if pos >= chars.len() || chars[pos] != '=' {
            continue;
        }
        pos += 1; // Skip '='

        // Skip whitespace after '='
        while pos < chars.len() && chars[pos].is_whitespace() {
            pos += 1;
        }

        // Parse attribute value
        if pos >= chars.len() {
            break;
        }

        let quote_char = chars[pos];
        if quote_char != '"' && quote_char != '\'' {
            continue;
        }
        pos += 1; // Skip opening quote

        let value_start = pos;
        while pos < chars.len() && chars[pos] != quote_char {
            pos += 1;
        }
        let value_end = pos;
        let attr_value: String = chars[value_start..value_end].iter().collect();

        // Check if cursor is on attribute value
        if relative_offset >= value_start && relative_offset <= value_end {
            let (start_line, start_col) = offset_to_line_col(source, base_offset + value_start);
            let (end_line, end_col) = offset_to_line_col(source, base_offset + value_end);

            return Some(HoverTarget::AttributeValue {
                element: element_name.to_string(),
                attribute: attr_name,
                value: attr_value,
                range: Range {
                    start_line,
                    start_col,
                    end_line,
                    end_col,
                },
            });
        }

        if pos < chars.len() {
            pos += 1; // Skip closing quote
        }
    }

    None
}

/// Check if cursor is on a closing tag element name
fn check_closing_tag(chars: &[char], offset: usize, _line: u32, source: &str) -> Option<HoverTarget> {
    // Find start of potential closing tag
    let mut start = offset;
    while start > 0 && chars[start] != '<' {
        if chars[start] == '>' {
            // Check if we just passed a closing tag
            break;
        }
        start -= 1;
    }

    // Check for </
    if start + 1 < chars.len() && chars[start] == '<' && chars[start + 1] == '/' {
        let name_start = start + 2;
        let mut name_end = name_start;

        while name_end < chars.len() && chars[name_end] != '>' && !chars[name_end].is_whitespace() {
            name_end += 1;
        }

        if offset >= name_start && offset < name_end {
            let element_name: String = chars[name_start..name_end].iter().collect();

            let (start_line, start_col) = offset_to_line_col(source, name_start);
            let (end_line, end_col) = offset_to_line_col(source, name_end);

            return Some(HoverTarget::Element {
                name: element_name,
                range: Range {
                    start_line,
                    start_col,
                    end_line,
                    end_col,
                },
            });
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hover_element_name() {
        let source = "<Component Guid=\"*\" />";
        let target = detect_hover_target(source, 1, 3); // On "Component"

        match target {
            HoverTarget::Element { name, .. } => {
                assert_eq!(name, "Component");
            }
            _ => panic!("Expected Element, got {:?}", target),
        }
    }

    #[test]
    fn test_hover_attribute_name() {
        let source = "<Component Guid=\"*\" />";
        let target = detect_hover_target(source, 1, 13); // On "Guid"

        match target {
            HoverTarget::AttributeName {
                element, attribute, ..
            } => {
                assert_eq!(element, "Component");
                assert_eq!(attribute, "Guid");
            }
            _ => panic!("Expected AttributeName, got {:?}", target),
        }
    }

    #[test]
    fn test_hover_attribute_value() {
        let source = "<Directory Id=\"ProgramFilesFolder\" />";
        let target = detect_hover_target(source, 1, 20); // On "ProgramFilesFolder"

        match target {
            HoverTarget::AttributeValue {
                element,
                attribute,
                value,
                ..
            } => {
                assert_eq!(element, "Directory");
                assert_eq!(attribute, "Id");
                assert_eq!(value, "ProgramFilesFolder");
            }
            _ => panic!("Expected AttributeValue, got {:?}", target),
        }
    }

    #[test]
    fn test_hover_closing_tag() {
        let source = "<Package></Package>";
        let target = detect_hover_target(source, 1, 13); // On closing "Package"

        match target {
            HoverTarget::Element { name, .. } => {
                assert_eq!(name, "Package");
            }
            _ => panic!("Expected Element, got {:?}", target),
        }
    }

    #[test]
    fn test_hover_outside_tag() {
        let source = "<Package>  </Package>";
        let target = detect_hover_target(source, 1, 11); // On whitespace

        assert!(matches!(target, HoverTarget::None));
    }

    #[test]
    fn test_hover_empty_source() {
        let target = detect_hover_target("", 1, 1);
        assert!(matches!(target, HoverTarget::None));
    }

    #[test]
    fn test_hover_past_end() {
        let source = "<P />";
        let target = detect_hover_target(source, 1, 100);
        assert!(matches!(target, HoverTarget::None));
    }

    #[test]
    fn test_hover_multiline() {
        let source = "<Component\n  Guid=\"*\" />";
        let target = detect_hover_target(source, 2, 4); // On "Guid" on line 2

        match target {
            HoverTarget::AttributeName { attribute, .. } => {
                assert_eq!(attribute, "Guid");
            }
            _ => panic!("Expected AttributeName, got {:?}", target),
        }
    }

    #[test]
    fn test_line_col_to_offset() {
        let source = "abc\ndef\nghi";
        assert_eq!(line_col_to_offset(source, 1, 1), 0);
        assert_eq!(line_col_to_offset(source, 1, 3), 2);
        assert_eq!(line_col_to_offset(source, 2, 1), 4);
        assert_eq!(line_col_to_offset(source, 3, 2), 9);
    }

    #[test]
    fn test_offset_to_line_col() {
        let source = "abc\ndef\nghi";
        assert_eq!(offset_to_line_col(source, 0), (1, 1));
        assert_eq!(offset_to_line_col(source, 2), (1, 3));
        assert_eq!(offset_to_line_col(source, 4), (2, 1));
        assert_eq!(offset_to_line_col(source, 9), (3, 2));
    }

    #[test]
    fn test_hover_element_with_range() {
        let source = "<Component />";
        let target = detect_hover_target(source, 1, 3);

        if let HoverTarget::Element { range, .. } = target {
            assert_eq!(range.start_line, 1);
            assert_eq!(range.start_col, 2);
            assert_eq!(range.end_col, 11);
        } else {
            panic!("Expected Element");
        }
    }

    #[test]
    fn test_hover_single_quotes() {
        let source = "<File Source='test.exe' />";
        let target = detect_hover_target(source, 1, 17); // On "test.exe"

        match target {
            HoverTarget::AttributeValue { value, .. } => {
                assert_eq!(value, "test.exe");
            }
            _ => panic!("Expected AttributeValue, got {:?}", target),
        }
    }

    #[test]
    fn test_hover_multiple_attributes() {
        let source = "<File Id=\"F1\" Source=\"app.exe\" />";
        let target = detect_hover_target(source, 1, 17); // On "Source"

        match target {
            HoverTarget::AttributeName { attribute, .. } => {
                assert_eq!(attribute, "Source");
            }
            _ => panic!("Expected AttributeName, got {:?}", target),
        }
    }
}
