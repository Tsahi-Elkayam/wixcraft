//! Completion providers

mod attributes;
mod elements;
mod snippets;
mod values;

pub use attributes::complete_attributes;
pub use elements::complete_elements;
pub use snippets::complete_snippets;
pub use values::complete_values;

use crate::loader::WixData;
use crate::types::{CompletionItem, CursorContext};

/// Get completions for a given context
pub fn get_completions(
    data: &WixData,
    context: &CursorContext,
    source: &str,
    limit: usize,
) -> Vec<CompletionItem> {
    let mut completions = match context {
        CursorContext::ElementStart { parent } => {
            complete_elements(data, parent.as_deref())
        }

        CursorContext::ElementContent { parent, .. } => {
            complete_elements(data, Some(parent.as_str()))
        }

        CursorContext::AttributeName { element, existing } => {
            complete_attributes(data, element, existing)
        }

        CursorContext::AttributeValue {
            element,
            attribute,
            partial,
        } => complete_values(data, element, attribute, partial, source),

        CursorContext::Unknown => {
            // Return top-level elements
            complete_elements(data, None)
        }
    };

    // Add snippet completions for element contexts
    if matches!(
        context,
        CursorContext::ElementStart { .. } | CursorContext::ElementContent { .. }
    ) {
        let prefix = extract_partial_text(source, context);
        completions.extend(complete_snippets(data, &prefix));
    }

    // Sort by priority
    completions.sort_by(|a, b| a.sort_priority.cmp(&b.sort_priority));

    // Limit results
    completions.truncate(limit);

    completions
}

/// Extract partial text at cursor for filtering
fn extract_partial_text(source: &str, _context: &CursorContext) -> String {
    // Simple: get the last word-like characters
    let chars: Vec<char> = source.chars().collect();
    let end = chars.len();
    let mut start = end;

    while start > 0 {
        let ch = chars[start - 1];
        if ch.is_alphanumeric() || ch == '_' || ch == '-' {
            start -= 1;
        } else {
            break;
        }
    }

    chars[start..end].iter().collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_data() -> (TempDir, WixData) {
        let temp = TempDir::new().unwrap();

        let elements_dir = temp.path().join("elements");
        fs::create_dir(&elements_dir).unwrap();

        let package = r#"{
            "name": "Package",
            "description": "Root package",
            "parents": ["Wix"],
            "children": ["Component", "Directory"],
            "attributes": {}
        }"#;
        fs::write(elements_dir.join("package.json"), package).unwrap();

        let component = r#"{
            "name": "Component",
            "description": "Component element",
            "parents": ["Package", "Directory"],
            "children": ["File"],
            "attributes": {
                "Guid": {"type": "guid", "required": true},
                "Id": {"type": "identifier"}
            }
        }"#;
        fs::write(elements_dir.join("component.json"), component).unwrap();

        let keywords_dir = temp.path().join("keywords");
        fs::create_dir(&keywords_dir).unwrap();
        let keywords = r#"{"standardDirectories": ["ProgramFilesFolder"], "builtinProperties": [], "elements": [], "preprocessorDirectives": []}"#;
        fs::write(keywords_dir.join("keywords.json"), keywords).unwrap();

        let snippets_dir = temp.path().join("snippets");
        fs::create_dir(&snippets_dir).unwrap();
        let snippets = r#"{"snippets": []}"#;
        fs::write(snippets_dir.join("snippets.json"), snippets).unwrap();

        let data = WixData::load(temp.path()).unwrap();
        (temp, data)
    }

    #[test]
    fn test_get_completions_element_start() {
        let (_temp, data) = create_test_data();
        let ctx = CursorContext::ElementStart {
            parent: Some("Package".to_string()),
        };
        let completions = get_completions(&data, &ctx, "<", 50);

        assert!(!completions.is_empty());
        assert!(completions.iter().any(|c| c.label == "Component"));
    }

    #[test]
    fn test_get_completions_attribute_name() {
        let (_temp, data) = create_test_data();
        let ctx = CursorContext::AttributeName {
            element: "Component".to_string(),
            existing: vec![],
        };
        let completions = get_completions(&data, &ctx, "", 50);

        assert!(!completions.is_empty());
        assert!(completions.iter().any(|c| c.label == "Guid"));
    }

    #[test]
    fn test_get_completions_element_content() {
        let (_temp, data) = create_test_data();
        let ctx = CursorContext::ElementContent {
            parent: "Package".to_string(),
            siblings: vec![],
        };
        let completions = get_completions(&data, &ctx, "<Package>\n  ", 50);

        assert!(!completions.is_empty());
        assert!(completions.iter().any(|c| c.label == "Component"));
    }

    #[test]
    fn test_get_completions_attribute_value() {
        let (_temp, data) = create_test_data();
        let ctx = CursorContext::AttributeValue {
            element: "Component".to_string(),
            attribute: "Guid".to_string(),
            partial: "".to_string(),
        };
        let completions = get_completions(&data, &ctx, "<Component Guid=\"", 50);

        assert!(!completions.is_empty());
        // Should have GUID suggestions
        assert!(completions.iter().any(|c| c.label == "*"));
    }

    #[test]
    fn test_get_completions_unknown_context() {
        let (_temp, data) = create_test_data();
        let ctx = CursorContext::Unknown;
        let completions = get_completions(&data, &ctx, "", 50);

        // Unknown context returns elements with no parent filter
        // May be empty if no elements match, which is valid behavior
        // Just ensure it doesn't panic
        let _ = completions;
    }

    #[test]
    fn test_completions_sorted_by_priority() {
        let (_temp, data) = create_test_data();
        let ctx = CursorContext::AttributeName {
            element: "Component".to_string(),
            existing: vec![],
        };
        let completions = get_completions(&data, &ctx, "", 50);

        // Verify sorting - lower priority should come first
        for i in 1..completions.len() {
            assert!(completions[i - 1].sort_priority <= completions[i].sort_priority);
        }
    }

    #[test]
    fn test_completions_limit() {
        let (_temp, data) = create_test_data();
        let ctx = CursorContext::ElementStart {
            parent: Some("Package".to_string()),
        };
        let completions = get_completions(&data, &ctx, "<", 1);

        assert!(completions.len() <= 1);
    }

    #[test]
    fn test_extract_partial_text_empty() {
        let partial = extract_partial_text("", &CursorContext::Unknown);
        assert_eq!(partial, "");
    }

    #[test]
    fn test_extract_partial_text_with_prefix() {
        let partial = extract_partial_text("<Package>\n  <Comp", &CursorContext::Unknown);
        assert_eq!(partial, "Comp");
    }

    #[test]
    fn test_extract_partial_text_no_word() {
        let partial = extract_partial_text("<Package>\n  <", &CursorContext::Unknown);
        assert_eq!(partial, "");
    }

    #[test]
    fn test_extract_partial_text_with_dash() {
        let partial = extract_partial_text("some-prefix", &CursorContext::Unknown);
        assert_eq!(partial, "some-prefix");
    }

    #[test]
    fn test_extract_partial_text_with_underscore() {
        let partial = extract_partial_text("some_prefix", &CursorContext::Unknown);
        assert_eq!(partial, "some_prefix");
    }
}
