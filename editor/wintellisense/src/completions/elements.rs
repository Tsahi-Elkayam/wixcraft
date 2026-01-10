//! Element name completions

use crate::loader::SchemaData;
use crate::types::{CompletionItem, CompletionKind, CursorContext, ElementDef};

/// Complete element names based on parent context
pub fn complete_elements(schema: &SchemaData, ctx: &CursorContext) -> Vec<CompletionItem> {
    match &ctx.parent_element {
        Some(parent_name) => {
            // Get valid children for this parent
            schema
                .get_children(parent_name)
                .into_iter()
                .filter(|elem| matches_prefix(&elem.name, &ctx.prefix))
                .map(|elem| create_element_completion(elem))
                .collect()
        }
        None => {
            // No parent context - suggest top-level elements
            let top_level = ["Wix", "Include"];
            schema
                .elements
                .values()
                .filter(|e| top_level.contains(&e.name.as_str()) || e.parents.is_empty())
                .filter(|e| matches_prefix(&e.name, &ctx.prefix))
                .map(|elem| create_element_completion(elem))
                .collect()
        }
    }
}

fn create_element_completion(elem: &ElementDef) -> CompletionItem {
    let required_attrs: Vec<_> = elem
        .attributes
        .iter()
        .filter(|(_, def)| def.required)
        .map(|(name, _)| name.as_str())
        .collect();

    let insert_text = if required_attrs.is_empty() {
        format!("<{}>$0</{}>", elem.name, elem.name)
    } else {
        let attr_str: String = required_attrs
            .iter()
            .enumerate()
            .map(|(i, attr)| format!("{}=\"${{{}:}}\"", attr, i + 1))
            .collect::<Vec<_>>()
            .join(" ");
        format!("<{} {}>$0</{}>", elem.name, attr_str, elem.name)
    };

    let mut doc = elem.description.clone();
    if !elem.children.is_empty() {
        doc.push_str("\n\nChildren: ");
        doc.push_str(&elem.children.join(", "));
    }

    CompletionItem::new(&elem.name, CompletionKind::Element)
        .with_insert_text(insert_text)
        .with_detail(&elem.description)
        .with_documentation(doc)
        .with_priority(element_priority(&elem.name))
        .as_snippet()
}

fn element_priority(name: &str) -> u32 {
    match name {
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
        _ => 50,
    }
}

fn matches_prefix(name: &str, prefix: &str) -> bool {
    if prefix.is_empty() {
        return true;
    }
    name.to_lowercase().starts_with(&prefix.to_lowercase())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn create_test_schema() -> SchemaData {
        let mut elements = HashMap::new();

        elements.insert(
            "Package".to_string(),
            ElementDef {
                name: "Package".to_string(),
                description: "Package element".to_string(),
                children: vec!["Component".to_string(), "Directory".to_string()],
                ..Default::default()
            },
        );

        elements.insert(
            "Component".to_string(),
            ElementDef {
                name: "Component".to_string(),
                description: "Component".to_string(),
                parents: vec!["Package".to_string()],
                ..Default::default()
            },
        );

        elements.insert(
            "Directory".to_string(),
            ElementDef {
                name: "Directory".to_string(),
                description: "Directory".to_string(),
                parents: vec!["Package".to_string()],
                ..Default::default()
            },
        );

        SchemaData {
            elements,
            ..Default::default()
        }
    }

    #[test]
    fn test_complete_children() {
        let schema = create_test_schema();
        let ctx = CursorContext {
            parent_element: Some("Package".to_string()),
            in_element_content: true,
            ..Default::default()
        };

        let items = complete_elements(&schema, &ctx);
        let labels: Vec<_> = items.iter().map(|i| i.label.as_str()).collect();

        assert!(labels.contains(&"Component"));
        assert!(labels.contains(&"Directory"));
    }

    #[test]
    fn test_complete_with_prefix() {
        let schema = create_test_schema();
        let ctx = CursorContext {
            parent_element: Some("Package".to_string()),
            prefix: "Comp".to_string(),
            in_element_content: true,
            ..Default::default()
        };

        let items = complete_elements(&schema, &ctx);
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].label, "Component");
    }
}
