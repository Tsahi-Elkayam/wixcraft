//! Attribute value completions

use crate::loader::WixData;
use crate::types::{CompletionItem, CompletionKind};

/// Complete attribute values based on type and context
pub fn complete_values(
    data: &WixData,
    element: &str,
    attribute: &str,
    partial: &str,
    source: &str,
) -> Vec<CompletionItem> {
    let mut completions = Vec::new();

    // Get attribute definition for type info
    if let Some(attr_def) = data.get_attribute(element, attribute) {
        // Check for explicit values (enum type)
        if let Some(values) = &attr_def.values {
            completions.extend(values.iter().filter(|v| v.starts_with(partial)).map(|v| {
                CompletionItem::new(v, CompletionKind::Value)
                    .with_detail("Enum value")
                    .with_priority(10)
            }));
        }

        // Type-specific completions
        match attr_def.attr_type.as_str() {
            "yesno" => {
                completions.extend(complete_yesno(partial));
            }
            "guid" => {
                completions.extend(complete_guid(partial));
            }
            "identifier" => {
                // Check if this looks like a directory reference
                if attribute == "Directory" || attribute.ends_with("Directory") {
                    completions.extend(complete_directories(data, partial));
                }
                // Add IDs from document
                completions.extend(complete_ids_from_document(source, partial, element));
            }
            _ => {}
        }
    }

    // Check for standard directory completion by attribute name
    if attribute == "Id" && element == "Directory" {
        completions.extend(complete_directories(data, partial));
    }

    // Property value completions
    if attribute == "Id" && element == "Property" {
        completions.extend(complete_properties(data, partial));
    }

    completions
}

/// Complete yes/no values
fn complete_yesno(partial: &str) -> Vec<CompletionItem> {
    ["yes", "no"]
        .iter()
        .filter(|v| v.starts_with(partial))
        .map(|v| {
            CompletionItem::new(*v, CompletionKind::Value)
                .with_detail("Boolean")
                .with_priority(10)
        })
        .collect()
}

/// Complete GUID values
fn complete_guid(partial: &str) -> Vec<CompletionItem> {
    let mut completions = Vec::new();

    // Auto-generate marker
    if "*".starts_with(partial) || partial.is_empty() {
        completions.push(
            CompletionItem::new("*", CompletionKind::Value)
                .with_detail("Auto-generate GUID")
                .with_documentation("WiX will generate a unique GUID at build time")
                .with_priority(5),
        );
    }

    // GUID template
    if partial.is_empty() {
        completions.push(
            CompletionItem::new("PUT-GUID-HERE", CompletionKind::Value)
                .with_detail("GUID placeholder")
                .with_insert_text("${1:00000000-0000-0000-0000-000000000000}")
                .with_priority(20),
        );
    }

    completions
}

/// Complete standard directory IDs
fn complete_directories(data: &WixData, partial: &str) -> Vec<CompletionItem> {
    data.keywords
        .standard_directories
        .iter()
        .filter(|d| d.to_lowercase().starts_with(&partial.to_lowercase()))
        .map(|d| {
            CompletionItem::new(d, CompletionKind::Directory)
                .with_detail("Standard directory")
                .with_priority(15)
        })
        .collect()
}

/// Complete built-in property names
fn complete_properties(data: &WixData, partial: &str) -> Vec<CompletionItem> {
    data.keywords
        .builtin_properties
        .iter()
        .filter(|p| p.to_lowercase().starts_with(&partial.to_lowercase()))
        .map(|p| {
            CompletionItem::new(p, CompletionKind::Property)
                .with_detail("Built-in property")
                .with_priority(15)
        })
        .collect()
}

/// Extract IDs from the current document for reference completion
fn complete_ids_from_document(
    source: &str,
    partial: &str,
    _current_element: &str,
) -> Vec<CompletionItem> {
    let mut ids = Vec::new();

    // Simple regex-like extraction of Id attributes
    // This is a simplified version - a real implementation would use proper parsing
    for line in source.lines() {
        if let Some(start) = line.find("Id=\"") {
            let rest = &line[start + 4..];
            if let Some(end) = rest.find('"') {
                let id = &rest[..end];
                if id.to_lowercase().starts_with(&partial.to_lowercase()) && !id.is_empty() {
                    ids.push(
                        CompletionItem::new(id, CompletionKind::Value)
                            .with_detail("Reference")
                            .with_priority(30),
                    );
                }
            }
        }
    }

    // Deduplicate
    ids.sort_by(|a, b| a.label.cmp(&b.label));
    ids.dedup_by(|a, b| a.label == b.label);

    ids
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
            "description": "Component",
            "parents": [],
            "children": [],
            "attributes": {
                "Guid": {"type": "guid", "required": true},
                "Transitive": {"type": "yesno", "required": false},
                "Directory": {"type": "identifier", "required": false}
            }
        }"#;
        fs::write(elements_dir.join("component.json"), component).unwrap();

        let feature = r#"{
            "name": "Feature",
            "description": "Feature",
            "parents": [],
            "children": [],
            "attributes": {
                "Display": {"type": "enum", "values": ["expand", "collapse", "hidden"]}
            }
        }"#;
        fs::write(elements_dir.join("feature.json"), feature).unwrap();

        let directory = r#"{
            "name": "Directory",
            "description": "Directory",
            "parents": [],
            "children": [],
            "attributes": {
                "Id": {"type": "identifier"}
            }
        }"#;
        fs::write(elements_dir.join("directory.json"), directory).unwrap();

        let keywords_dir = temp.path().join("keywords");
        fs::create_dir(&keywords_dir).unwrap();
        let keywords = r#"{
            "standardDirectories": ["ProgramFilesFolder", "SystemFolder", "WindowsFolder"],
            "builtinProperties": ["ProductName", "ProductVersion"],
            "elements": [],
            "preprocessorDirectives": []
        }"#;
        fs::write(keywords_dir.join("keywords.json"), keywords).unwrap();

        let snippets_dir = temp.path().join("snippets");
        fs::create_dir(&snippets_dir).unwrap();
        fs::write(snippets_dir.join("snippets.json"), r#"{"snippets":[]}"#).unwrap();

        let data = WixData::load(temp.path()).unwrap();
        (temp, data)
    }

    #[test]
    fn test_complete_yesno() {
        let completions = complete_yesno("");
        assert_eq!(completions.len(), 2);
        assert!(completions.iter().any(|c| c.label == "yes"));
        assert!(completions.iter().any(|c| c.label == "no"));
    }

    #[test]
    fn test_complete_yesno_partial() {
        let completions = complete_yesno("y");
        assert_eq!(completions.len(), 1);
        assert_eq!(completions[0].label, "yes");
    }

    #[test]
    fn test_complete_guid() {
        let completions = complete_guid("");
        assert!(completions.iter().any(|c| c.label == "*"));
    }

    #[test]
    fn test_complete_directories() {
        let (_temp, data) = create_test_data();
        let completions = complete_directories(&data, "Program");

        assert!(!completions.is_empty());
        assert!(completions.iter().any(|c| c.label == "ProgramFilesFolder"));
    }

    #[test]
    fn test_complete_enum_values() {
        let (_temp, data) = create_test_data();
        let completions = complete_values(&data, "Feature", "Display", "", "");

        assert_eq!(completions.len(), 3);
        assert!(completions.iter().any(|c| c.label == "expand"));
        assert!(completions.iter().any(|c| c.label == "collapse"));
        assert!(completions.iter().any(|c| c.label == "hidden"));
    }

    #[test]
    fn test_complete_ids_from_document() {
        let source = r#"
            <Component Id="MyComponent" Guid="*">
            <Directory Id="INSTALLFOLDER">
        "#;
        let completions = complete_ids_from_document(source, "", "ComponentRef");

        assert!(completions.iter().any(|c| c.label == "MyComponent"));
        assert!(completions.iter().any(|c| c.label == "INSTALLFOLDER"));
    }

    #[test]
    fn test_directory_id_completion() {
        let (_temp, data) = create_test_data();
        let completions = complete_values(&data, "Directory", "Id", "Program", "");

        assert!(completions.iter().any(|c| c.label == "ProgramFilesFolder"));
    }

    #[test]
    fn test_complete_yesno_type() {
        let (_temp, data) = create_test_data();
        // Component.Transitive is yesno type
        let completions = complete_values(&data, "Component", "Transitive", "", "");

        assert!(completions.iter().any(|c| c.label == "yes"));
        assert!(completions.iter().any(|c| c.label == "no"));
    }

    #[test]
    fn test_complete_guid_type() {
        let (_temp, data) = create_test_data();
        let completions = complete_values(&data, "Component", "Guid", "", "");

        assert!(completions.iter().any(|c| c.label == "*"));
    }

    #[test]
    fn test_complete_guid_partial() {
        let completions = complete_guid("*");
        assert!(completions.iter().any(|c| c.label == "*"));
    }

    #[test]
    fn test_complete_guid_with_text() {
        let completions = complete_guid("abc");
        // No completions when partial doesn't match
        assert!(completions.is_empty() || completions.iter().all(|c| c.label != "*"));
    }

    #[test]
    fn test_complete_properties() {
        let (_temp, data) = create_test_data();
        let completions = complete_properties(&data, "Product");

        assert!(completions.iter().any(|c| c.label == "ProductName"));
        assert!(completions.iter().any(|c| c.label == "ProductVersion"));
    }

    #[test]
    fn test_complete_property_id() {
        let (_temp, data) = create_test_data();
        // Property.Id should get builtin properties
        let completions = complete_values(&data, "Property", "Id", "Product", "");

        assert!(completions.iter().any(|c| c.label == "ProductName"));
    }

    #[test]
    fn test_complete_identifier_type_with_directory() {
        let (_temp, data) = create_test_data();
        // Test identifier type with Directory suffix attribute name
        // Component.Directory attribute is identifier type that ends with "Directory"
        let completions = complete_values(&data, "Component", "Directory", "Program", "");

        // Should suggest standard directories because attribute name ends with "Directory"
        assert!(completions.iter().any(|c| c.label == "ProgramFilesFolder"));
    }

    #[test]
    fn test_complete_ids_partial_match() {
        let source = r#"<Component Id="MyComp" /><Component Id="OtherComp" />"#;
        let completions = complete_ids_from_document(source, "My", "ComponentRef");

        assert_eq!(completions.len(), 1);
        assert_eq!(completions[0].label, "MyComp");
    }

    #[test]
    fn test_complete_ids_empty_source() {
        let completions = complete_ids_from_document("", "", "ComponentRef");
        assert!(completions.is_empty());
    }

    #[test]
    fn test_complete_ids_no_ids() {
        let source = "<Package><Directory></Directory></Package>";
        let completions = complete_ids_from_document(source, "", "ComponentRef");
        assert!(completions.is_empty());
    }

    #[test]
    fn test_complete_values_unknown_element() {
        let (_temp, data) = create_test_data();
        let completions = complete_values(&data, "UnknownElement", "Attr", "", "");

        // Should return empty for unknown element
        assert!(completions.is_empty());
    }

    #[test]
    fn test_complete_enum_partial() {
        let (_temp, data) = create_test_data();
        let completions = complete_values(&data, "Feature", "Display", "exp", "");

        assert_eq!(completions.len(), 1);
        assert_eq!(completions[0].label, "expand");
    }

    #[test]
    fn test_complete_directories_case_insensitive() {
        let (_temp, data) = create_test_data();
        let completions = complete_directories(&data, "program");

        assert!(completions.iter().any(|c| c.label == "ProgramFilesFolder"));
    }

    #[test]
    fn test_complete_properties_case_insensitive() {
        let (_temp, data) = create_test_data();
        let completions = complete_properties(&data, "product");

        assert!(completions.iter().any(|c| c.label == "ProductName"));
    }
}
