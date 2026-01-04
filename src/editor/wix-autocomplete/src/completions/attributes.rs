//! Attribute name completions

use crate::loader::WixData;
use crate::types::{CompletionItem, CompletionKind};

/// Complete attribute names for an element
pub fn complete_attributes(
    data: &WixData,
    element: &str,
    existing: &[String],
) -> Vec<CompletionItem> {
    let Some(elem_def) = data.get_element(element) else {
        return Vec::new();
    };

    elem_def
        .attributes
        .iter()
        .filter(|(name, _)| !existing.contains(name))
        .map(|(name, def)| {
            let priority = if def.required { 10 } else { 50 };
            let detail = format_attribute_detail(def);

            CompletionItem::new(name, CompletionKind::Attribute)
                .with_detail(detail)
                .with_documentation(&def.description)
                .with_insert_text(format!("{}=\"$1\"", name))
                .with_priority(priority)
        })
        .collect()
}

/// Format attribute detail (type and required status)
fn format_attribute_detail(def: &crate::types::AttributeDef) -> String {
    let mut parts = vec![format!("Type: {}", def.attr_type)];

    if def.required {
        parts.push("Required".to_string());
    }

    if let Some(default) = &def.default {
        parts.push(format!("Default: {}", default));
    }

    if let Some(values) = &def.values {
        if !values.is_empty() {
            parts.push(format!("Values: {}", values.join(" | ")));
        }
    }

    parts.join(" | ")
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

        let component = r#"{
            "name": "Component",
            "description": "Component element",
            "parents": ["Package"],
            "children": ["File"],
            "attributes": {
                "Guid": {"type": "guid", "required": true, "description": "Component GUID"},
                "Id": {"type": "identifier", "required": false, "description": "Component ID"},
                "Directory": {"type": "identifier", "required": false, "description": "Target directory"}
            }
        }"#;
        fs::write(elements_dir.join("component.json"), component).unwrap();

        let keywords_dir = temp.path().join("keywords");
        fs::create_dir(&keywords_dir).unwrap();
        fs::write(keywords_dir.join("keywords.json"), r#"{"standardDirectories":[],"builtinProperties":[],"elements":[],"preprocessorDirectives":[]}"#).unwrap();

        let snippets_dir = temp.path().join("snippets");
        fs::create_dir(&snippets_dir).unwrap();
        fs::write(snippets_dir.join("snippets.json"), r#"{"snippets":[]}"#).unwrap();

        let data = WixData::load(temp.path()).unwrap();
        (temp, data)
    }

    #[test]
    fn test_complete_attributes() {
        let (_temp, data) = create_test_data();
        let completions = complete_attributes(&data, "Component", &[]);

        assert_eq!(completions.len(), 3);
        assert!(completions.iter().any(|c| c.label == "Guid"));
        assert!(completions.iter().any(|c| c.label == "Id"));
    }

    #[test]
    fn test_filter_existing_attributes() {
        let (_temp, data) = create_test_data();
        let existing = vec!["Guid".to_string()];
        let completions = complete_attributes(&data, "Component", &existing);

        assert_eq!(completions.len(), 2);
        assert!(!completions.iter().any(|c| c.label == "Guid"));
    }

    #[test]
    fn test_required_priority() {
        let (_temp, data) = create_test_data();
        let completions = complete_attributes(&data, "Component", &[]);

        let guid = completions.iter().find(|c| c.label == "Guid").unwrap();
        let id = completions.iter().find(|c| c.label == "Id").unwrap();

        assert!(guid.sort_priority < id.sort_priority);
    }

    #[test]
    fn test_unknown_element() {
        let (_temp, data) = create_test_data();
        let completions = complete_attributes(&data, "Unknown", &[]);

        assert!(completions.is_empty());
    }

    #[test]
    fn test_attribute_detail() {
        use crate::types::AttributeDef;

        let def = AttributeDef {
            attr_type: "guid".to_string(),
            required: true,
            description: String::new(),
            default: None,
            values: None,
        };

        let detail = format_attribute_detail(&def);
        assert!(detail.contains("guid"));
        assert!(detail.contains("Required"));
    }
}
