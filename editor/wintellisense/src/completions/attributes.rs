//! Attribute name completions

use crate::loader::SchemaData;
use crate::types::{CompletionItem, CompletionKind, CursorContext};

/// Complete attribute names for the current element
pub fn complete_attributes(schema: &SchemaData, ctx: &CursorContext) -> Vec<CompletionItem> {
    let element_name = match &ctx.current_element {
        Some(name) => name,
        None => return Vec::new(),
    };

    let elem = match schema.get_element(element_name) {
        Some(e) => e,
        None => return Vec::new(),
    };

    elem.attributes
        .iter()
        .filter(|(name, _)| !ctx.existing_attributes.contains(name))
        .filter(|(name, _)| matches_prefix(name, &ctx.prefix))
        .map(|(name, def)| {
            let priority = if def.required { 10 } else { 50 };

            let mut detail_parts = vec![format!("Type: {}", def.attr_type)];
            if def.required {
                detail_parts.push("Required".to_string());
            }
            if let Some(default) = &def.default {
                detail_parts.push(format!("Default: {}", default));
            }
            if let Some(values) = &def.values {
                if !values.is_empty() {
                    detail_parts.push(format!("Values: {}", values.join(" | ")));
                }
            }

            CompletionItem::new(name, CompletionKind::Attribute)
                .with_insert_text(format!("{}=\"$1\"", name))
                .with_detail(detail_parts.join(" | "))
                .with_documentation(&def.description)
                .with_priority(priority)
                .with_required(def.required)
                .as_snippet()
        })
        .collect()
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
    use crate::types::{AttributeDef, ElementDef};
    use std::collections::HashMap;

    fn create_test_schema() -> SchemaData {
        let mut elements = HashMap::new();
        let mut attrs = HashMap::new();

        attrs.insert(
            "Id".to_string(),
            AttributeDef {
                attr_type: "identifier".to_string(),
                required: true,
                description: "Component ID".to_string(),
                ..Default::default()
            },
        );

        attrs.insert(
            "Guid".to_string(),
            AttributeDef {
                attr_type: "guid".to_string(),
                required: false,
                description: "Component GUID".to_string(),
                default: Some("*".to_string()),
                ..Default::default()
            },
        );

        elements.insert(
            "Component".to_string(),
            ElementDef {
                name: "Component".to_string(),
                attributes: attrs,
                ..Default::default()
            },
        );

        SchemaData {
            elements,
            ..Default::default()
        }
    }

    #[test]
    fn test_complete_attributes() {
        let schema = create_test_schema();
        let ctx = CursorContext {
            current_element: Some("Component".to_string()),
            in_opening_tag: true,
            ..Default::default()
        };

        let items = complete_attributes(&schema, &ctx);
        assert_eq!(items.len(), 2);

        // Required should come first (lower priority number)
        let first = &items.iter().find(|i| i.label == "Id").unwrap();
        assert!(first.required);
    }

    #[test]
    fn test_filter_existing() {
        let schema = create_test_schema();
        let ctx = CursorContext {
            current_element: Some("Component".to_string()),
            in_opening_tag: true,
            existing_attributes: vec!["Id".to_string()],
            ..Default::default()
        };

        let items = complete_attributes(&schema, &ctx);
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].label, "Guid");
    }
}
