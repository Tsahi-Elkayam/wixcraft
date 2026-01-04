//! Parse cursor context from XML source

use crate::types::CursorContext;

/// Parse the cursor context from XML source at a given position
pub fn parse_context(source: &str, line: u32, column: u32) -> CursorContext {
    let offset = line_col_to_offset(source, line, column);

    // Use the entire source if offset is at or past end
    let before = if offset >= source.len() {
        source
    } else {
        &source[..offset]
    };

    // Find what context we're in by scanning backwards
    let chars: Vec<char> = before.chars().collect();

    // Check if we're inside a tag
    if let Some(tag_context) = parse_tag_context(&chars, source, offset) {
        return tag_context;
    }

    // We're in element content - find the parent element
    parse_element_content_context(&chars, source)
}

/// Convert line/column (1-based) to byte offset
fn line_col_to_offset(source: &str, line: u32, column: u32) -> usize {
    let mut current_line = 1u32;
    let mut current_col = 1u32;
    let mut offset = 0;

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
        offset = i + ch.len_utf8();
    }

    // If we reached the end, return the final position
    if current_line == line && current_col == column {
        return offset;
    }

    offset
}

/// Check if cursor is inside an XML tag and determine context
fn parse_tag_context(chars: &[char], _source: &str, _offset: usize) -> Option<CursorContext> {
    // Find the last '<' that isn't closed
    let mut depth = 0;
    let mut last_open = None;

    for (i, &ch) in chars.iter().enumerate().rev() {
        match ch {
            '>' => depth += 1,
            '<' => {
                if depth == 0 {
                    last_open = Some(i);
                    break;
                }
                depth -= 1;
            }
            _ => {}
        }
    }

    let tag_start = last_open?;

    // Check if this is a closing tag
    if chars.get(tag_start + 1) == Some(&'/') {
        return None; // Inside closing tag, treat as element content
    }

    // Extract the content after '<'
    let tag_content_raw: String = chars[tag_start + 1..].iter().collect();
    let tag_content = tag_content_raw.trim();

    // Check if we just typed '<' with no element name yet
    if tag_content.is_empty() {
        let parent = find_parent_element(chars, tag_start);
        return Some(CursorContext::ElementStart { parent });
    }

    // Parse the tag content, passing original to detect trailing whitespace
    parse_inside_tag(tag_content, &tag_content_raw, chars, tag_start)
}

/// Parse context when inside an element tag
fn parse_inside_tag(
    tag_content: &str,
    tag_content_raw: &str,
    chars: &[char],
    tag_start: usize,
) -> Option<CursorContext> {
    // Split into element name and rest
    let mut parts = tag_content.splitn(2, |c: char| c.is_whitespace());
    let element_name = parts.next()?.to_string();
    let rest = parts.next().unwrap_or("");

    // If no whitespace after element name, we're still typing element name
    // Use raw content to check for trailing whitespace (indicates we finished element name)
    if rest.is_empty() && !tag_content_raw.ends_with(char::is_whitespace) {
        let parent = find_parent_element(chars, tag_start);
        return Some(CursorContext::ElementStart { parent });
    }

    // We have an element name, now check for attributes
    let existing_attrs = extract_existing_attributes(rest);

    // Check if we're in an attribute value
    if let Some(attr_value_ctx) = check_attribute_value_context(rest, &element_name) {
        return Some(attr_value_ctx);
    }

    // We're in attribute name position
    Some(CursorContext::AttributeName {
        element: element_name,
        existing: existing_attrs,
    })
}

/// Check if cursor is inside an attribute value
fn check_attribute_value_context(rest: &str, element: &str) -> Option<CursorContext> {
    // Look for pattern: AttrName="partial (with unclosed quote)
    let bytes = rest.as_bytes();
    let len = bytes.len();

    if len == 0 {
        return None;
    }

    // Scan from start to find unclosed quotes
    let mut in_quotes = false;
    let mut quote_char = b'"';
    let mut quote_start = 0;

    for (i, &ch) in bytes.iter().enumerate() {
        if !in_quotes && (ch == b'"' || ch == b'\'') {
            in_quotes = true;
            quote_char = ch;
            quote_start = i;
        } else if in_quotes && ch == quote_char {
            in_quotes = false;
        }
    }

    // If we're inside an unclosed quote, we're in attribute value context
    if in_quotes {
        let before_quote = &rest[..quote_start];
        if let Some(eq_pos) = before_quote.rfind('=') {
            let before_eq = before_quote[..eq_pos].trim();
            let attr_name = before_eq.split_whitespace().last()?;
            let partial = &rest[quote_start + 1..];

            return Some(CursorContext::AttributeValue {
                element: element.to_string(),
                attribute: attr_name.to_string(),
                partial: partial.to_string(),
            });
        }
    }

    // Check if cursor is right after '='
    if rest.trim_end().ends_with('=') {
        let before_eq = rest.trim_end().trim_end_matches('=').trim();
        let attr_name = before_eq.split_whitespace().last()?;
        return Some(CursorContext::AttributeValue {
            element: element.to_string(),
            attribute: attr_name.to_string(),
            partial: String::new(),
        });
    }

    None
}

/// Extract already-present attribute names
fn extract_existing_attributes(rest: &str) -> Vec<String> {
    let mut attrs = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;
    let mut quote_char = '"';

    for ch in rest.chars() {
        if !in_quotes && (ch == '"' || ch == '\'') {
            in_quotes = true;
            quote_char = ch;
        } else if in_quotes && ch == quote_char {
            in_quotes = false;
        } else if !in_quotes && ch == '=' {
            let attr = current.trim().to_string();
            if !attr.is_empty() {
                attrs.push(attr);
            }
            current.clear();
        } else if !in_quotes {
            current.push(ch);
        }
    }

    attrs
}

/// Find the parent element by scanning backwards
fn find_parent_element(chars: &[char], before_pos: usize) -> Option<String> {
    let mut depth = 0;
    let mut i = before_pos;

    while i > 0 {
        i -= 1;

        // Skip if we're inside a tag
        if chars[i] == '>' {
            // Find matching '<'
            let mut j = i;
            while j > 0 {
                j -= 1;
                if chars[j] == '<' {
                    // Check if it's a closing tag
                    if j + 1 < chars.len() && chars[j + 1] == '/' {
                        depth += 1;
                    } else if j + 1 < chars.len() && chars[j + 1] != '?' && chars[j + 1] != '!' {
                        // It's an opening tag
                        if depth == 0 {
                            // This is our parent - extract element name
                            let tag: String = chars[j + 1..i].iter().collect();
                            let name = tag.split_whitespace().next()?;
                            // Skip self-closing tags
                            if !tag.trim_end().ends_with('/') {
                                return Some(name.to_string());
                            }
                        } else {
                            depth -= 1;
                        }
                    }
                    i = j;
                    break;
                }
            }
        }
    }

    None
}

/// Parse element content context (cursor is between tags)
fn parse_element_content_context(chars: &[char], _source: &str) -> CursorContext {
    // Find current parent and siblings
    let parent = find_parent_element(chars, chars.len());

    // TODO: Extract siblings for more intelligent filtering

    match parent {
        Some(p) => CursorContext::ElementContent {
            parent: p,
            siblings: Vec::new(),
        },
        None => CursorContext::Unknown,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_line_col_to_offset() {
        let source = "abc\ndef\nghi";
        assert_eq!(line_col_to_offset(source, 1, 1), 0);
        assert_eq!(line_col_to_offset(source, 1, 3), 2);
        assert_eq!(line_col_to_offset(source, 2, 1), 4);
        assert_eq!(line_col_to_offset(source, 3, 2), 9);
    }

    #[test]
    fn test_element_start_context() {
        let source = "<Package>\n  <";
        let ctx = parse_context(source, 2, 4);
        match ctx {
            CursorContext::ElementStart { parent } => {
                assert_eq!(parent, Some("Package".to_string()));
            }
            _ => panic!("Expected ElementStart, got {:?}", ctx),
        }
    }

    #[test]
    fn test_attribute_name_context() {
        let source = "<Component Guid=\"*\" ";
        let ctx = parse_context(source, 1, 21);
        match ctx {
            CursorContext::AttributeName { element, existing } => {
                assert_eq!(element, "Component");
                assert!(existing.contains(&"Guid".to_string()));
            }
            _ => panic!("Expected AttributeName, got {:?}", ctx),
        }
    }

    #[test]
    fn test_attribute_value_context() {
        let source = "<Component Guid=\"";
        let ctx = parse_context(source, 1, 18);
        match ctx {
            CursorContext::AttributeValue { element, attribute, partial } => {
                assert_eq!(element, "Component");
                assert_eq!(attribute, "Guid");
                assert_eq!(partial, "");
            }
            _ => panic!("Expected AttributeValue, got {:?}", ctx),
        }
    }

    #[test]
    fn test_attribute_value_with_partial() {
        let source = "<Directory Id=\"Prog";
        let ctx = parse_context(source, 1, 20);
        match ctx {
            CursorContext::AttributeValue { element, attribute, partial } => {
                assert_eq!(element, "Directory");
                assert_eq!(attribute, "Id");
                assert_eq!(partial, "Prog");
            }
            _ => panic!("Expected AttributeValue, got {:?}", ctx),
        }
    }

    #[test]
    fn test_element_content_context() {
        let source = "<Package>\n  \n</Package>";
        let ctx = parse_context(source, 2, 3);
        match ctx {
            CursorContext::ElementContent { parent, .. } => {
                assert_eq!(parent, "Package");
            }
            _ => panic!("Expected ElementContent, got {:?}", ctx),
        }
    }

    #[test]
    fn test_nested_elements() {
        let source = "<Package>\n  <Directory>\n    <";
        let ctx = parse_context(source, 3, 6);
        match ctx {
            CursorContext::ElementStart { parent } => {
                assert_eq!(parent, Some("Directory".to_string()));
            }
            _ => panic!("Expected ElementStart, got {:?}", ctx),
        }
    }

    #[test]
    fn test_extract_existing_attributes() {
        let rest = "Guid=\"*\" Id=\"Test\" ";
        let attrs = extract_existing_attributes(rest);
        assert!(attrs.contains(&"Guid".to_string()));
        assert!(attrs.contains(&"Id".to_string()));
    }

    #[test]
    fn test_closing_tag_context() {
        // Inside a closing tag should be treated as element content
        let source = "<Package>\n  </";
        let ctx = parse_context(source, 2, 5);
        // Should be ElementContent, not inside the closing tag
        assert!(matches!(ctx, CursorContext::ElementContent { .. } | CursorContext::Unknown));
    }

    #[test]
    fn test_self_closing_tag() {
        let source = "<Package>\n  <File />\n  <";
        let ctx = parse_context(source, 3, 4);
        match ctx {
            CursorContext::ElementStart { parent } => {
                assert_eq!(parent, Some("Package".to_string()));
            }
            _ => panic!("Expected ElementStart, got {:?}", ctx),
        }
    }

    #[test]
    fn test_empty_source() {
        let source = "";
        let ctx = parse_context(source, 1, 1);
        assert!(matches!(ctx, CursorContext::Unknown));
    }

    #[test]
    fn test_no_parent_element() {
        let source = "  ";
        let ctx = parse_context(source, 1, 3);
        assert!(matches!(ctx, CursorContext::Unknown));
    }

    #[test]
    fn test_root_element_start() {
        let source = "<";
        let ctx = parse_context(source, 1, 2);
        match ctx {
            CursorContext::ElementStart { parent } => {
                assert_eq!(parent, None);
            }
            _ => panic!("Expected ElementStart with no parent, got {:?}", ctx),
        }
    }

    #[test]
    fn test_attribute_value_after_equals() {
        let source = "<Component Guid=";
        let ctx = parse_context(source, 1, 17);
        match ctx {
            CursorContext::AttributeValue { element, attribute, partial } => {
                assert_eq!(element, "Component");
                assert_eq!(attribute, "Guid");
                assert_eq!(partial, "");
            }
            _ => panic!("Expected AttributeValue, got {:?}", ctx),
        }
    }

    #[test]
    fn test_typing_element_name() {
        let source = "<Comp";
        let ctx = parse_context(source, 1, 6);
        match ctx {
            CursorContext::ElementStart { parent } => {
                assert_eq!(parent, None);
            }
            _ => panic!("Expected ElementStart, got {:?}", ctx),
        }
    }

    #[test]
    fn test_attribute_with_single_quotes() {
        let source = "<Component Guid='";
        let ctx = parse_context(source, 1, 18);
        match ctx {
            CursorContext::AttributeValue { element, attribute, .. } => {
                assert_eq!(element, "Component");
                assert_eq!(attribute, "Guid");
            }
            _ => panic!("Expected AttributeValue, got {:?}", ctx),
        }
    }

    #[test]
    fn test_multiple_completed_attributes() {
        let source = "<File Source=\"a.exe\" Vital=\"yes\" ";
        let ctx = parse_context(source, 1, 34);
        match ctx {
            CursorContext::AttributeName { element, existing } => {
                assert_eq!(element, "File");
                assert!(existing.contains(&"Source".to_string()));
                assert!(existing.contains(&"Vital".to_string()));
            }
            _ => panic!("Expected AttributeName, got {:?}", ctx),
        }
    }

    #[test]
    fn test_deeply_nested_elements() {
        let source = "<Wix>\n  <Package>\n    <Directory>\n      <Component>\n        <";
        let ctx = parse_context(source, 5, 10);
        match ctx {
            CursorContext::ElementStart { parent } => {
                assert_eq!(parent, Some("Component".to_string()));
            }
            _ => panic!("Expected ElementStart, got {:?}", ctx),
        }
    }

    #[test]
    fn test_xml_declaration_ignored() {
        let source = "<?xml version=\"1.0\"?>\n<Package>\n  <";
        let ctx = parse_context(source, 3, 4);
        match ctx {
            CursorContext::ElementStart { parent } => {
                assert_eq!(parent, Some("Package".to_string()));
            }
            _ => panic!("Expected ElementStart, got {:?}", ctx),
        }
    }

    #[test]
    fn test_line_col_to_offset_end_of_file() {
        let source = "abc";
        // Position after last char
        assert_eq!(line_col_to_offset(source, 1, 4), 3);
    }

    #[test]
    fn test_line_col_to_offset_past_end() {
        let source = "abc";
        // Position way past end
        assert_eq!(line_col_to_offset(source, 1, 100), 3);
    }

    #[test]
    fn test_element_content_no_parent() {
        // Just text with no XML
        let source = "hello world";
        let ctx = parse_context(source, 1, 6);
        assert!(matches!(ctx, CursorContext::Unknown));
    }

    #[test]
    fn test_after_closed_element() {
        let source = "<Package><Directory></Directory>\n<";
        let ctx = parse_context(source, 2, 2);
        match ctx {
            CursorContext::ElementStart { parent } => {
                // Directory is closed, so parent should be Package
                assert_eq!(parent, Some("Package".to_string()));
            }
            _ => panic!("Expected ElementStart, got {:?}", ctx),
        }
    }
}
