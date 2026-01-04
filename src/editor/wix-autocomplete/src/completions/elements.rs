//! Element name completions

use crate::loader::WixData;
use crate::types::{CompletionItem, CompletionKind};

/// Complete element names based on parent context
pub fn complete_elements(data: &WixData, parent: Option<&str>) -> Vec<CompletionItem> {
    match parent {
        Some(parent_name) => {
            // Get valid children for this parent
            data.get_children(parent_name)
                .into_iter()
                .map(|elem| {
                    CompletionItem::new(&elem.name, CompletionKind::Element)
                        .with_detail(&elem.description)
                        .with_documentation(format_documentation(elem))
                        .with_insert_text(format_element_insert(&elem.name, &elem.attributes))
                        .with_priority(element_priority(&elem.name))
                })
                .collect()
        }
        None => {
            // No parent context - suggest top-level elements
            let top_level = ["Wix", "Include"];
            data.elements
                .values()
                .filter(|e| top_level.contains(&e.name.as_str()) || e.parents.is_empty())
                .map(|elem| {
                    CompletionItem::new(&elem.name, CompletionKind::Element)
                        .with_detail(&elem.description)
                        .with_documentation(format_documentation(elem))
                        .with_priority(element_priority(&elem.name))
                })
                .collect()
        }
    }
}

/// Format documentation for an element
fn format_documentation(elem: &crate::types::ElementDef) -> String {
    let mut doc = elem.description.clone();

    if !elem.children.is_empty() {
        doc.push_str("\n\nChildren: ");
        doc.push_str(&elem.children.join(", "));
    }

    if let Some(url) = &elem.documentation {
        doc.push_str("\n\n");
        doc.push_str(url);
    }

    doc
}

/// Format insert text for an element (with required attributes)
fn format_element_insert(
    name: &str,
    attrs: &std::collections::HashMap<String, crate::types::AttributeDef>,
) -> String {
    let required: Vec<_> = attrs
        .iter()
        .filter(|(_, def)| def.required)
        .map(|(name, _)| name.as_str())
        .collect();

    if required.is_empty() {
        format!("<{}>$0</{}>", name, name)
    } else {
        let attr_str: String = required
            .iter()
            .enumerate()
            .map(|(i, attr)| format!("{}=\"${{{}:}}\"", attr, i + 1))
            .collect::<Vec<_>>()
            .join(" ");
        format!("<{} {}>$0</{}>", name, attr_str, name)
    }
}

/// Determine sort priority for common elements
fn element_priority(name: &str) -> u32 {
    match name {
        // Most commonly used elements
        "Component" => 10,
        "File" => 11,
        "Directory" => 12,
        "Feature" => 13,
        "Package" => 14,
        "Property" => 15,
        "RegistryKey" => 20,
        "RegistryValue" => 21,
        "Shortcut" => 22,
        "CustomAction" => 25,
        // Less common
        _ => 50,
    }
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
            "description": "Root package element",
            "parents": ["Wix"],
            "children": ["Component", "Directory", "Feature"],
            "attributes": {}
        }"#;
        fs::write(elements_dir.join("package.json"), package).unwrap();

        let component = r#"{
            "name": "Component",
            "description": "Component element",
            "parents": ["Package", "Directory"],
            "children": ["File", "RegistryKey"],
            "attributes": {"Guid": {"type": "guid", "required": true}}
        }"#;
        fs::write(elements_dir.join("component.json"), component).unwrap();

        let file = r#"{
            "name": "File",
            "description": "File element",
            "parents": ["Component"],
            "children": [],
            "attributes": {"Source": {"type": "path", "required": true}}
        }"#;
        fs::write(elements_dir.join("file.json"), file).unwrap();

        let wix = r#"{
            "name": "Wix",
            "description": "Root Wix element",
            "parents": [],
            "children": ["Package", "Fragment"],
            "attributes": {}
        }"#;
        fs::write(elements_dir.join("wix.json"), wix).unwrap();

        // Create empty keywords and snippets
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
    fn test_complete_children() {
        let (_temp, data) = create_test_data();
        let completions = complete_elements(&data, Some("Package"));

        assert!(!completions.is_empty());
        assert!(completions.iter().any(|c| c.label == "Component"));
    }

    #[test]
    fn test_complete_top_level() {
        let (_temp, data) = create_test_data();
        let completions = complete_elements(&data, None);

        assert!(completions.iter().any(|c| c.label == "Wix"));
    }

    #[test]
    fn test_element_priority() {
        assert!(element_priority("Component") < element_priority("CustomAction"));
        assert!(element_priority("File") < element_priority("Unknown"));
    }

    #[test]
    fn test_format_element_insert() {
        let mut attrs = std::collections::HashMap::new();
        attrs.insert(
            "Guid".to_string(),
            crate::types::AttributeDef {
                attr_type: "guid".to_string(),
                required: true,
                description: String::new(),
                default: None,
                values: None,
            },
        );

        let insert = format_element_insert("Component", &attrs);
        assert!(insert.contains("Guid="));
        assert!(insert.starts_with("<Component"));
    }
}
