//! Go-to-definition provider

use crate::index::ProjectIndex;
use crate::loader::SchemaData;
use crate::types::{CursorContext, Definition, DefinitionResult};

/// Find definition for symbol at cursor
pub fn find_definition(
    schema: &SchemaData,
    index: &ProjectIndex,
    ctx: &CursorContext,
    _source: &str,
) -> DefinitionResult {
    // Get the word at cursor
    let word = match &ctx.word_at_cursor {
        Some(w) => w,
        None => return DefinitionResult::empty(),
    };

    // Check if we're in a reference element (e.g., ComponentRef)
    if let Some(ref element) = ctx.current_element {
        if element.ends_with("Ref") {
            // This is a reference element - find the definition
            if let Some(symbol) = index.find_definition_for_ref(element, word) {
                return DefinitionResult::single(Definition {
                    location: symbol.location.clone(),
                    name: symbol.name.clone(),
                    kind: symbol.kind.clone(),
                    preview: symbol.preview.clone(),
                });
            }
        }
    }

    // Check if we're on an Id attribute value that references something
    if ctx.in_attribute_value {
        if let Some(ref attr) = ctx.current_attribute {
            if let Some(ref element) = ctx.current_element {
                // Handle reference attributes
                let definitions = find_reference_definitions(index, element, attr, word);
                if !definitions.is_empty() {
                    return DefinitionResult::new(definitions);
                }
            }
        }
    }

    // Try to find any symbol with this name
    let symbols = index.find_symbol(word);
    if !symbols.is_empty() {
        let definitions = symbols
            .into_iter()
            .map(|s| Definition {
                location: s.location.clone(),
                name: s.name.clone(),
                kind: s.kind.clone(),
                preview: s.preview.clone(),
            })
            .collect();
        return DefinitionResult::new(definitions);
    }

    // Check if this is an element name - go to schema documentation
    if let Some(elem) = schema.get_element(word) {
        // For elements, we could provide a "virtual" definition pointing to docs
        // For now, return empty as we don't have file locations for schema elements
        let _ = elem; // Suppress unused warning
    }

    DefinitionResult::empty()
}

/// Find definitions for reference attributes
fn find_reference_definitions(
    index: &ProjectIndex,
    element: &str,
    attribute: &str,
    value: &str,
) -> Vec<Definition> {
    // Map element+attribute to target kind
    let target_kind = match (element, attribute) {
        ("ComponentRef", "Id") => Some("Component"),
        ("ComponentGroupRef", "Id") => Some("ComponentGroup"),
        ("DirectoryRef", "Id") => Some("Directory"),
        ("FeatureRef", "Id") => Some("Feature"),
        ("FeatureGroupRef", "Id") => Some("FeatureGroup"),
        ("PropertyRef", "Id") => Some("Property"),
        ("CustomActionRef", "Id") => Some("CustomAction"),
        // Also handle reference attributes on non-Ref elements
        (_, "Directory") => Some("Directory"),
        (_, "Feature") => Some("Feature"),
        (_, "Component") => Some("Component"),
        _ => None,
    };

    if let Some(kind) = target_kind {
        return index
            .find_symbol_by_kind(kind, value)
            .into_iter()
            .map(|s| Definition {
                location: s.location.clone(),
                name: s.name.clone(),
                kind: s.kind.clone(),
                preview: s.preview.clone(),
            })
            .collect();
    }

    Vec::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_definition_empty() {
        let schema = SchemaData::default();
        let index = ProjectIndex::new();
        let ctx = CursorContext::default();

        let result = find_definition(&schema, &index, &ctx, "");
        assert!(result.definitions.is_empty());
    }

    #[test]
    fn test_find_definition_with_word() {
        let schema = SchemaData::default();
        let index = ProjectIndex::new();
        let ctx = CursorContext {
            word_at_cursor: Some("MyComponent".to_string()),
            ..Default::default()
        };

        // Should return empty since index is empty
        let result = find_definition(&schema, &index, &ctx, "");
        assert!(result.definitions.is_empty());
    }

    #[test]
    fn test_find_reference_definitions_mapping() {
        let index = ProjectIndex::new();

        // Just test the mapping function returns empty when no symbols
        let defs = find_reference_definitions(&index, "ComponentRef", "Id", "Test");
        assert!(defs.is_empty());

        // Test various reference types are mapped correctly
        let defs = find_reference_definitions(&index, "DirectoryRef", "Id", "Test");
        assert!(defs.is_empty());

        let defs = find_reference_definitions(&index, "FeatureRef", "Id", "Test");
        assert!(defs.is_empty());
    }

    #[test]
    fn test_find_definition_for_ref_element() {
        let schema = SchemaData::default();
        let index = ProjectIndex::new();
        let ctx = CursorContext {
            word_at_cursor: Some("MyComp".to_string()),
            current_element: Some("ComponentRef".to_string()),
            in_attribute_value: true,
            current_attribute: Some("Id".to_string()),
            ..Default::default()
        };

        // Should return empty since index is empty
        let result = find_definition(&schema, &index, &ctx, "");
        assert!(result.definitions.is_empty());
    }
}
