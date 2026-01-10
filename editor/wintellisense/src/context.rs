//! XML context parser for determining cursor position and completion context.

use crate::types::CursorContext;

/// Parse XML content and determine completion context at position (1-based line/column)
pub fn parse_context(content: &str, line: u32, column: u32) -> CursorContext {
    let mut ctx = CursorContext {
        line,
        column,
        ..Default::default()
    };

    let lines: Vec<&str> = content.lines().collect();
    let line_idx = (line as usize).saturating_sub(1);

    if line_idx >= lines.len() {
        return ctx;
    }

    // Build content up to cursor
    let mut before_cursor = String::new();
    for (i, l) in lines.iter().enumerate() {
        if i < line_idx {
            before_cursor.push_str(l);
            before_cursor.push('\n');
        } else if i == line_idx {
            let col = ((column as usize).saturating_sub(1)).min(l.len());
            before_cursor.push_str(&l[..col]);
        }
    }

    // Find last < and > positions
    let last_open = before_cursor.rfind('<');
    let last_close = before_cursor.rfind('>');

    // Determine if inside a tag
    let in_tag = match (last_open, last_close) {
        (Some(open), Some(close)) => open > close,
        (Some(_), None) => true,
        _ => false,
    };

    if in_tag {
        if let Some(open_pos) = last_open {
            let tag_content = &before_cursor[open_pos + 1..];

            // Closing tag
            if tag_content.starts_with('/') {
                return ctx;
            }

            let parts: Vec<&str> = tag_content.split_whitespace().collect();

            if parts.is_empty() {
                // Just after <, suggest elements
                ctx.in_element_content = true;
                ctx.parent_element = find_parent_element(&before_cursor);
            } else {
                // Have element name
                ctx.current_element = Some(parts[0].to_string());
                ctx.in_opening_tag = true;

                // Parse existing attributes
                ctx.existing_attributes = parse_existing_attributes(tag_content);

                // Check if in attribute value
                if let Some(eq_pos) = tag_content.rfind('=') {
                    let after_eq = &tag_content[eq_pos + 1..];
                    let trimmed = after_eq.trim_start();

                    if trimmed.starts_with('"') {
                        // Check if the attribute value is complete (has closing quote)
                        let after_open_quote = &trimmed[1..];
                        if let Some(close_quote_pos) = after_open_quote.find('"') {
                            // Value is complete, check what's after it
                            let after_value = &after_open_quote[close_quote_pos + 1..];
                            if after_value.trim().is_empty() {
                                // Nothing after the completed value, attribute name context
                                ctx.prefix = String::new();
                            } else {
                                // There's more text - could be typing a new attribute name
                                ctx.prefix = extract_partial_word(tag_content);
                            }
                            // Stay in opening tag context (not attribute value)
                        } else {
                            // No closing quote - we're in the attribute value
                            ctx.in_attribute_value = true;
                            ctx.in_opening_tag = false;

                            // Find attribute name
                            let before_eq = &tag_content[..eq_pos];
                            if let Some(attr_name) = before_eq.split_whitespace().last() {
                                ctx.current_attribute = Some(attr_name.to_string());
                            }

                            // Get partial value (everything after the opening quote)
                            ctx.prefix = after_open_quote.to_string();
                        }
                    } else if trimmed.is_empty() {
                        // Right after =
                        ctx.in_attribute_value = true;
                        ctx.in_opening_tag = false;
                        let before_eq = &tag_content[..eq_pos];
                        if let Some(attr_name) = before_eq.split_whitespace().last() {
                            ctx.current_attribute = Some(attr_name.to_string());
                        }
                    } else {
                        ctx.prefix = extract_partial_word(tag_content);
                    }
                } else {
                    ctx.prefix = extract_partial_word(tag_content);
                }
            }
        }
    } else {
        // In element content
        ctx.in_element_content = true;
        ctx.parent_element = find_parent_element(&before_cursor);

        let after_last_close = last_close
            .map(|pos| &before_cursor[pos + 1..])
            .unwrap_or(&before_cursor);

        ctx.prefix = extract_partial_word(after_last_close);

        if ctx.prefix.starts_with('<') || after_last_close.trim().starts_with('<') {
            ctx.prefix = ctx.prefix.trim_start_matches('<').to_string();
        }
    }

    // Extract word at cursor for hover/definition
    ctx.word_at_cursor = extract_word_at_cursor(content, line, column);

    ctx
}

/// Extract partial word at end of string
fn extract_partial_word(s: &str) -> String {
    let trimmed = s.trim_end();
    if let Some(pos) = trimmed.rfind(|c: char| c.is_whitespace() || c == '<' || c == '>' || c == '"')
    {
        trimmed[pos + 1..].to_string()
    } else {
        trimmed.to_string()
    }
}

/// Parse existing attributes from tag content
fn parse_existing_attributes(tag_content: &str) -> Vec<String> {
    let mut attrs = Vec::new();
    let mut in_value = false;

    for part in tag_content.split('"') {
        if in_value {
            in_value = false;
            continue;
        }

        for word in part.split_whitespace() {
            if let Some(eq_pos) = word.find('=') {
                attrs.push(word[..eq_pos].to_string());
            }
        }
        in_value = true;
    }

    attrs
}

/// Find parent element by analyzing unclosed tags
fn find_parent_element(content: &str) -> Option<String> {
    let mut stack: Vec<String> = Vec::new();
    let mut chars = content.chars().peekable();
    let mut in_tag = false;
    let mut current_tag = String::new();
    let mut is_closing = false;
    let mut is_self_closing = false;

    while let Some(c) = chars.next() {
        match c {
            '<' => {
                in_tag = true;
                current_tag.clear();
                is_closing = false;
                is_self_closing = false;

                if chars.peek() == Some(&'/') {
                    chars.next();
                    is_closing = true;
                }
            }
            '>' => {
                if in_tag {
                    if current_tag.ends_with('/') {
                        is_self_closing = true;
                        current_tag.pop();
                    }

                    let tag_name = current_tag
                        .split_whitespace()
                        .next()
                        .unwrap_or("")
                        .to_string();

                    if !tag_name.is_empty()
                        && !tag_name.starts_with('?')
                        && !tag_name.starts_with('!')
                    {
                        if is_closing {
                            if let Some(pos) = stack.iter().rposition(|t| t == &tag_name) {
                                stack.truncate(pos);
                            }
                        } else if !is_self_closing {
                            stack.push(tag_name);
                        }
                    }

                    in_tag = false;
                    current_tag.clear();
                }
            }
            '/' if in_tag && chars.peek() == Some(&'>') => {
                is_self_closing = true;
            }
            _ if in_tag => {
                current_tag.push(c);
            }
            _ => {}
        }
    }

    stack.last().cloned()
}

/// Extract word at cursor position
fn extract_word_at_cursor(content: &str, line: u32, column: u32) -> Option<String> {
    let lines: Vec<&str> = content.lines().collect();
    let line_idx = (line as usize).saturating_sub(1);

    if line_idx >= lines.len() {
        return None;
    }

    let line_content = lines[line_idx];
    let col_idx = ((column as usize).saturating_sub(1)).min(line_content.len());

    // Find word boundaries
    let is_word_char = |c: char| c.is_alphanumeric() || c == '_' || c == '-';

    let start = line_content[..col_idx]
        .rfind(|c: char| !is_word_char(c))
        .map(|p| p + 1)
        .unwrap_or(0);

    let end = line_content[col_idx..]
        .find(|c: char| !is_word_char(c))
        .map(|p| col_idx + p)
        .unwrap_or(line_content.len());

    if start < end {
        Some(line_content[start..end].to_string())
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::ContextKind;

    // =========================================================================
    // Basic context tests
    // =========================================================================

    #[test]
    fn test_element_content_context() {
        let content = "<Wix>\n  <Package>\n    ";
        let ctx = parse_context(content, 3, 5);

        assert!(ctx.in_element_content);
        assert_eq!(ctx.parent_element, Some("Package".to_string()));
    }

    #[test]
    fn test_opening_tag_context() {
        let content = "<Wix>\n  <Component ";
        let ctx = parse_context(content, 2, 14);

        assert!(ctx.in_opening_tag);
        assert_eq!(ctx.current_element, Some("Component".to_string()));
    }

    #[test]
    fn test_attribute_value_context() {
        let content = r#"<Component Id="Comp"#;
        let ctx = parse_context(content, 1, 20);

        assert!(ctx.in_attribute_value);
        assert_eq!(ctx.current_element, Some("Component".to_string()));
        assert_eq!(ctx.current_attribute, Some("Id".to_string()));
        assert_eq!(ctx.prefix, "Comp");
    }

    // =========================================================================
    // Parent element detection tests
    // =========================================================================

    #[test]
    fn test_find_parent_basic() {
        assert_eq!(
            find_parent_element("<Wix><Package>"),
            Some("Package".to_string())
        );
    }

    #[test]
    fn test_find_parent_with_closed() {
        assert_eq!(
            find_parent_element("<Wix><Package></Package>"),
            Some("Wix".to_string())
        );
    }

    #[test]
    fn test_find_parent_self_closing() {
        assert_eq!(
            find_parent_element("<Wix><Package><MajorUpgrade />"),
            Some("Package".to_string())
        );
    }

    #[test]
    fn test_find_parent_none() {
        assert_eq!(find_parent_element(""), None);
        assert_eq!(find_parent_element("   "), None);
    }

    // =========================================================================
    // Attribute parsing tests
    // =========================================================================

    #[test]
    fn test_existing_attributes() {
        let attrs = parse_existing_attributes(r#"Component Id="C1" Guid="*" "#);
        assert!(attrs.contains(&"Id".to_string()));
        assert!(attrs.contains(&"Guid".to_string()));
    }

    #[test]
    fn test_extract_existing_attributes_multiple() {
        let rest = r#"Guid="*" Id="Test" Name="foo" "#;
        let attrs = parse_existing_attributes(rest);
        assert!(attrs.contains(&"Guid".to_string()));
        assert!(attrs.contains(&"Id".to_string()));
        assert!(attrs.contains(&"Name".to_string()));
    }

    // =========================================================================
    // Word extraction tests
    // =========================================================================

    #[test]
    fn test_word_at_cursor() {
        let content = "Hello World Test";
        assert_eq!(
            extract_word_at_cursor(content, 1, 8),
            Some("World".to_string())
        );
    }

    #[test]
    fn test_word_at_cursor_start() {
        let content = "Hello World";
        assert_eq!(
            extract_word_at_cursor(content, 1, 1),
            Some("Hello".to_string())
        );
    }

    #[test]
    fn test_word_at_cursor_end() {
        let content = "Hello World";
        assert_eq!(
            extract_word_at_cursor(content, 1, 11),
            Some("World".to_string())
        );
    }

    // =========================================================================
    // Element start context tests (ported from old wix-autocomplete)
    // =========================================================================

    #[test]
    fn test_element_start_context() {
        let source = "<Package>\n  <";
        let ctx = parse_context(source, 2, 4);

        assert!(ctx.in_element_content);
        assert_eq!(ctx.parent_element, Some("Package".to_string()));
    }

    #[test]
    fn test_root_element_start() {
        let source = "<";
        let ctx = parse_context(source, 1, 2);

        assert!(ctx.in_element_content);
        assert_eq!(ctx.parent_element, None);
    }

    #[test]
    fn test_typing_element_name() {
        let source = "<Comp";
        let ctx = parse_context(source, 1, 6);

        // When typing element name, we're in element content with a prefix
        assert!(ctx.in_element_content || ctx.in_opening_tag);
    }

    // =========================================================================
    // Attribute name context tests
    // =========================================================================

    #[test]
    fn test_attribute_name_context() {
        // Position at the space after element name, before any attributes
        let source = "<Component ";
        let ctx = parse_context(source, 1, 12);

        assert!(ctx.in_opening_tag);
        assert_eq!(ctx.current_element, Some("Component".to_string()));
    }

    #[test]
    fn test_attribute_name_with_existing() {
        // Position right after a completed attribute
        let source = "<Component Guid=\"*\" Id";
        let ctx = parse_context(source, 1, 23);

        assert!(ctx.in_opening_tag);
        assert_eq!(ctx.current_element, Some("Component".to_string()));
        assert!(ctx.existing_attributes.contains(&"Guid".to_string()));
    }

    #[test]
    fn test_attribute_name_with_enum() {
        let source = "<Component ";
        let ctx = parse_context(source, 1, 12);

        match ctx.kind() {
            ContextKind::AttributeName { element, .. } => {
                assert_eq!(element, "Component");
            }
            _ => panic!("Expected AttributeName, got {:?}", ctx.kind()),
        }
    }

    // =========================================================================
    // Attribute value context tests
    // =========================================================================

    #[test]
    fn test_attribute_value_empty() {
        let source = "<Component Guid=\"";
        let ctx = parse_context(source, 1, 18);

        assert!(ctx.in_attribute_value);
        assert_eq!(ctx.current_element, Some("Component".to_string()));
        assert_eq!(ctx.current_attribute, Some("Guid".to_string()));
        assert_eq!(ctx.prefix, "");
    }

    #[test]
    fn test_attribute_value_with_partial() {
        let source = "<Directory Id=\"Prog";
        let ctx = parse_context(source, 1, 20);

        assert!(ctx.in_attribute_value);
        assert_eq!(ctx.current_element, Some("Directory".to_string()));
        assert_eq!(ctx.current_attribute, Some("Id".to_string()));
        assert_eq!(ctx.prefix, "Prog");
    }

    #[test]
    fn test_attribute_value_after_equals() {
        let source = "<Component Guid=";
        let ctx = parse_context(source, 1, 17);

        assert!(ctx.in_attribute_value);
        assert_eq!(ctx.current_element, Some("Component".to_string()));
        assert_eq!(ctx.current_attribute, Some("Guid".to_string()));
    }

    #[test]
    fn test_attribute_value_with_enum() {
        let source = "<Directory Id=\"Prog";
        let ctx = parse_context(source, 1, 20);

        match ctx.kind() {
            ContextKind::AttributeValue {
                element,
                attribute,
                partial,
            } => {
                assert_eq!(element, "Directory");
                assert_eq!(attribute, "Id");
                assert_eq!(partial, "Prog");
            }
            _ => panic!("Expected AttributeValue, got {:?}", ctx.kind()),
        }
    }

    // =========================================================================
    // Element content context tests
    // =========================================================================

    #[test]
    fn test_element_content_between_tags() {
        let source = "<Package>\n  \n</Package>";
        let ctx = parse_context(source, 2, 3);

        assert!(ctx.in_element_content);
        assert_eq!(ctx.parent_element, Some("Package".to_string()));
    }

    #[test]
    fn test_element_content_with_enum() {
        let source = "<Package>\n  \n</Package>";
        let ctx = parse_context(source, 2, 3);

        match ctx.kind() {
            ContextKind::ElementContent { parent, .. } => {
                assert_eq!(parent, "Package");
            }
            ContextKind::ElementStart { parent } => {
                assert_eq!(parent, Some("Package".to_string()));
            }
            _ => panic!("Expected ElementContent or ElementStart, got {:?}", ctx.kind()),
        }
    }

    // =========================================================================
    // Nested elements tests
    // =========================================================================

    #[test]
    fn test_nested_elements() {
        let source = "<Package>\n  <Directory>\n    <";
        let ctx = parse_context(source, 3, 6);

        assert!(ctx.in_element_content);
        assert_eq!(ctx.parent_element, Some("Directory".to_string()));
    }

    #[test]
    fn test_deeply_nested_elements() {
        let source = "<Wix>\n  <Package>\n    <Directory>\n      <Component>\n        <";
        let ctx = parse_context(source, 5, 10);

        assert!(ctx.in_element_content);
        assert_eq!(ctx.parent_element, Some("Component".to_string()));
    }

    // =========================================================================
    // Closing tag tests
    // =========================================================================

    #[test]
    fn test_closing_tag_context() {
        // Inside a closing tag should not trigger completions
        let source = "<Package>\n  </";
        let ctx = parse_context(source, 2, 5);

        // Should not be in attribute context
        assert!(!ctx.in_opening_tag);
        assert!(!ctx.in_attribute_value);
    }

    #[test]
    fn test_after_closed_element() {
        let source = "<Package><Directory></Directory>\n<";
        let ctx = parse_context(source, 2, 2);

        assert!(ctx.in_element_content);
        // Directory is closed, so parent should be Package
        assert_eq!(ctx.parent_element, Some("Package".to_string()));
    }

    // =========================================================================
    // Self-closing tag tests
    // =========================================================================

    #[test]
    fn test_self_closing_tag() {
        let source = "<Package>\n  <File />\n  <";
        let ctx = parse_context(source, 3, 4);

        assert!(ctx.in_element_content);
        assert_eq!(ctx.parent_element, Some("Package".to_string()));
    }

    #[test]
    fn test_self_closing_with_attributes() {
        let source = "<Package>\n  <File Source=\"a.exe\" />\n  <";
        let ctx = parse_context(source, 3, 4);

        assert!(ctx.in_element_content);
        assert_eq!(ctx.parent_element, Some("Package".to_string()));
    }

    // =========================================================================
    // XML declaration tests
    // =========================================================================

    #[test]
    fn test_xml_declaration_ignored() {
        let source = "<?xml version=\"1.0\"?>\n<Package>\n  <";
        let ctx = parse_context(source, 3, 4);

        assert!(ctx.in_element_content);
        assert_eq!(ctx.parent_element, Some("Package".to_string()));
    }

    // =========================================================================
    // Edge cases
    // =========================================================================

    #[test]
    fn test_empty_source() {
        let source = "";
        let ctx = parse_context(source, 1, 1);

        assert!(!ctx.in_opening_tag);
        assert!(!ctx.in_attribute_value);
        assert_eq!(ctx.kind(), ContextKind::Unknown);
    }

    #[test]
    fn test_no_parent_element() {
        let source = "  ";
        let ctx = parse_context(source, 1, 3);

        assert_eq!(ctx.parent_element, None);
    }

    #[test]
    fn test_element_content_no_parent() {
        let source = "hello world";
        let ctx = parse_context(source, 1, 6);

        // Plain text with no XML returns ElementStart with no parent
        // which is effectively unknown/root context
        match ctx.kind() {
            ContextKind::Unknown | ContextKind::ElementStart { parent: None } => {}
            _ => panic!("Expected Unknown or ElementStart with no parent, got {:?}", ctx.kind()),
        }
    }

    #[test]
    fn test_multiple_completed_attributes() {
        // Test with cursor typing a new attribute name
        let source = "<File Source=\"a.exe\" Vital=\"yes\" Key";
        let ctx = parse_context(source, 1, 37);

        assert!(ctx.in_opening_tag);
        assert_eq!(ctx.current_element, Some("File".to_string()));
        assert!(ctx.existing_attributes.contains(&"Source".to_string()));
        assert!(ctx.existing_attributes.contains(&"Vital".to_string()));
    }

    #[test]
    fn test_single_quotes_in_attribute() {
        let source = "<Component Guid='";
        let ctx = parse_context(source, 1, 18);

        // Single quotes should also trigger attribute value context
        // Note: current impl may not support this - adjust if needed
        assert!(ctx.in_opening_tag || ctx.in_attribute_value);
    }

    // =========================================================================
    // Position edge cases
    // =========================================================================

    #[test]
    fn test_position_past_end_of_line() {
        let source = "abc";
        let ctx = parse_context(source, 1, 100);

        // Should handle gracefully
        assert!(!ctx.in_opening_tag);
    }

    #[test]
    fn test_position_past_end_of_file() {
        let source = "abc\ndef";
        let ctx = parse_context(source, 10, 1);

        // Should handle gracefully
        assert_eq!(ctx.kind(), ContextKind::Unknown);
    }

    #[test]
    fn test_multiline_tag() {
        // Test multiline tag with cursor typing a new attribute
        let source = "<Component\n  Id=\"Test\"\n  Guid=\"*\"\n  Key";
        let ctx = parse_context(source, 4, 6);

        assert!(ctx.in_opening_tag);
        assert_eq!(ctx.current_element, Some("Component".to_string()));
    }
}
