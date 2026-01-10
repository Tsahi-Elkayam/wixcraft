//! Attribute value completions

use crate::index::ProjectIndex;
use crate::loader::SchemaData;
use crate::types::{CompletionItem, CompletionKind, CursorContext};
use uuid::Uuid;

/// Complete attribute values
pub fn complete_values(
    schema: &SchemaData,
    index: &ProjectIndex,
    ctx: &CursorContext,
) -> Vec<CompletionItem> {
    let mut items = Vec::new();

    let element_name = match &ctx.current_element {
        Some(name) => name,
        None => return items,
    };

    let attr_name = match &ctx.current_attribute {
        Some(name) => name,
        None => return items,
    };

    // Get attribute definition
    let elem = schema.get_element(element_name);
    let attr = elem.and_then(|e| e.attributes.get(attr_name));

    if let Some(attr_def) = attr {
        // Type-based completions
        match attr_def.attr_type.as_str() {
            "guid" => {
                items.extend(complete_guid(&ctx.prefix));
            }
            "yesno" => {
                items.extend(complete_yesno(&ctx.prefix));
            }
            _ => {}
        }

        // Enum values from schema
        if let Some(values) = &attr_def.values {
            for value in values {
                if matches_prefix(value, &ctx.prefix) {
                    items.push(CompletionItem::new(value, CompletionKind::Value));
                }
            }
        }

        // Default value
        if let Some(default) = &attr_def.default {
            if matches_prefix(default, &ctx.prefix) {
                items.push(
                    CompletionItem::new(default, CompletionKind::Value)
                        .with_detail("Default value")
                        .with_priority(5),
                );
            }
        }
    }

    // Attribute-specific completions
    items.extend(complete_by_attribute_name(schema, index, attr_name, &ctx.prefix));

    // Reference completions (e.g., ComponentRef Id -> Component Ids)
    items.extend(complete_reference_values(index, element_name, attr_name, &ctx.prefix));

    items
}

fn complete_guid(prefix: &str) -> Vec<CompletionItem> {
    let mut items = Vec::new();

    // Auto-generate
    if matches_prefix("*", prefix) {
        items.push(
            CompletionItem::new("*", CompletionKind::Value)
                .with_detail("Auto-generate GUID at build time")
                .with_priority(1),
        );
    }

    // New GUID
    if prefix.is_empty() {
        let guid = Uuid::new_v4().to_string().to_uppercase();
        items.push(
            CompletionItem::new(guid, CompletionKind::Value)
                .with_detail("New GUID")
                .with_priority(10),
        );
    }

    items
}

fn complete_yesno(prefix: &str) -> Vec<CompletionItem> {
    let mut items = Vec::new();

    for value in ["yes", "no"] {
        if matches_prefix(value, prefix) {
            items.push(CompletionItem::new(value, CompletionKind::Value));
        }
    }

    items
}

fn complete_by_attribute_name(
    schema: &SchemaData,
    index: &ProjectIndex,
    attr_name: &str,
    prefix: &str,
) -> Vec<CompletionItem> {
    let mut items = Vec::new();

    match attr_name {
        "Id" | "Directory" | "DirectoryRef" => {
            // Standard directories
            for dir in &schema.keywords.standard_directories {
                if matches_prefix(dir, prefix) {
                    items.push(
                        CompletionItem::new(dir, CompletionKind::Reference)
                            .with_detail("Standard directory"),
                    );
                }
            }

            // Project directories
            for sym in index.get_symbols_by_kind("Directory") {
                if matches_prefix(&sym.name, prefix) {
                    items.push(
                        CompletionItem::new(&sym.name, CompletionKind::Reference)
                            .with_detail("Project directory"),
                    );
                }
            }
        }

        "Root" => {
            // Registry roots
            for root in ["HKLM", "HKCU", "HKCR", "HKU", "HKMU"] {
                if matches_prefix(root, prefix) {
                    items.push(CompletionItem::new(root, CompletionKind::Value));
                }
            }
        }

        "Type" => {
            // Common types
            for t in ["string", "integer", "binary", "expandable", "multiString"] {
                if matches_prefix(t, prefix) {
                    items.push(CompletionItem::new(t, CompletionKind::Value));
                }
            }
        }

        "KeyPath" | "Compressed" | "Vital" | "EmbedCab" | "Win64" => {
            items.extend(complete_yesno(prefix));
        }

        "Start" => {
            // Service start types
            for t in ["auto", "demand", "disabled", "boot", "system"] {
                if matches_prefix(t, prefix) {
                    items.push(CompletionItem::new(t, CompletionKind::Value));
                }
            }
        }

        _ => {}
    }

    items
}

fn complete_reference_values(
    index: &ProjectIndex,
    element_name: &str,
    attr_name: &str,
    prefix: &str,
) -> Vec<CompletionItem> {
    let mut items = Vec::new();

    // Map element names to the kind of symbol they reference
    let ref_kind = match (element_name, attr_name) {
        ("ComponentRef", "Id") => Some("Component"),
        ("ComponentGroupRef", "Id") => Some("ComponentGroup"),
        ("DirectoryRef", "Id") => Some("Directory"),
        ("FeatureRef", "Id") => Some("Feature"),
        ("FeatureGroupRef", "Id") => Some("FeatureGroup"),
        ("PropertyRef", "Id") => Some("Property"),
        ("CustomActionRef", "Id") => Some("CustomAction"),
        _ => None,
    };

    if let Some(kind) = ref_kind {
        for sym in index.get_symbols_by_kind(kind) {
            if matches_prefix(&sym.name, prefix) {
                items.push(
                    CompletionItem::new(&sym.name, CompletionKind::Reference)
                        .with_detail(format!("{} in {}", kind, sym.location.path.display()))
                        .with_priority(5),
                );
            }
        }
    }

    items
}

fn matches_prefix(value: &str, prefix: &str) -> bool {
    if prefix.is_empty() {
        return true;
    }
    value.to_lowercase().starts_with(&prefix.to_lowercase())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_complete_guid() {
        let items = complete_guid("");
        assert!(items.iter().any(|i| i.label == "*"));
        assert!(items.len() >= 2); // * and generated GUID
    }

    #[test]
    fn test_complete_yesno() {
        let items = complete_yesno("");
        assert_eq!(items.len(), 2);
        assert!(items.iter().any(|i| i.label == "yes"));
        assert!(items.iter().any(|i| i.label == "no"));
    }

    #[test]
    fn test_complete_yesno_with_prefix() {
        let items = complete_yesno("y");
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].label, "yes");
    }
}
