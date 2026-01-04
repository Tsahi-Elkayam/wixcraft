//! Symbol resolution for go-to-definition and find-references

use crate::extractor::get_attribute_range;
use crate::index::SymbolIndex;
use crate::types::{
    DefinitionKind, DefinitionResult, Range, ReferenceKind, ReferencesResult, SymbolTarget,
};
use roxmltree::Document;

/// Detect what symbol is at the given cursor position
pub fn detect_symbol_at(source: &str, line: u32, column: u32) -> Option<SymbolTarget> {
    let offset = line_col_to_offset(source, line, column)?;

    let doc = Document::parse(source).ok()?;

    // Find the node at this offset
    for node in doc.descendants() {
        if !node.is_element() {
            continue;
        }

        let range = node.range();
        if offset < range.start || offset > range.end {
            continue;
        }

        let tag_name = node.tag_name().name();

        // Check if it's a reference element
        if let Some(kind) = ReferenceKind::from_element_name(tag_name) {
            if let Some(id) = node.attribute("Id") {
                if let Some(attr_range) = get_attribute_range(&node, "Id", source) {
                    // Check if cursor is on the Id attribute value
                    let attr_start = line_col_to_offset(
                        source,
                        attr_range.start.line,
                        attr_range.start.character,
                    )?;
                    let attr_end = line_col_to_offset(
                        source,
                        attr_range.end.line,
                        attr_range.end.character,
                    )?;

                    if offset >= attr_start && offset <= attr_end {
                        return Some(SymbolTarget::Reference {
                            kind,
                            id: id.to_string(),
                            range: attr_range,
                        });
                    }
                }

                // Cursor is somewhere on the element but not on Id value
                let node_range = Range::from_offsets(source, range.start, range.end);
                return Some(SymbolTarget::Reference {
                    kind,
                    id: id.to_string(),
                    range: node_range,
                });
            }
        }

        // Check if it's a definition element
        if let Some(kind) = DefinitionKind::from_element_name(tag_name) {
            let id_attr = match kind {
                DefinitionKind::Package | DefinitionKind::Bundle => {
                    node.attribute("Name").or_else(|| node.attribute("Id"))
                }
                _ => node.attribute("Id"),
            };

            if let Some(id) = id_attr {
                if let Some(attr_range) = get_attribute_range(&node, "Id", source)
                    .or_else(|| get_attribute_range(&node, "Name", source))
                {
                    let attr_start = line_col_to_offset(
                        source,
                        attr_range.start.line,
                        attr_range.start.character,
                    )?;
                    let attr_end = line_col_to_offset(
                        source,
                        attr_range.end.line,
                        attr_range.end.character,
                    )?;

                    if offset >= attr_start && offset <= attr_end {
                        return Some(SymbolTarget::Definition {
                            kind,
                            id: id.to_string(),
                            range: attr_range,
                        });
                    }
                }

                // Cursor is somewhere on the element
                let node_range = Range::from_offsets(source, range.start, range.end);
                return Some(SymbolTarget::Definition {
                    kind,
                    id: id.to_string(),
                    range: node_range,
                });
            }
        }
    }

    None
}

/// Go to definition from a position in source
pub fn go_to_definition(
    source: &str,
    line: u32,
    column: u32,
    index: &SymbolIndex,
) -> DefinitionResult {
    let Some(target) = detect_symbol_at(source, line, column) else {
        return DefinitionResult::no_symbol();
    };

    match target {
        SymbolTarget::Reference { kind, id, .. } => {
            if let Some(def) = index.get_definition_for_ref(kind, &id) {
                DefinitionResult::found(def.clone())
            } else {
                DefinitionResult::not_found(&id, kind.definition_element())
            }
        }
        SymbolTarget::Definition { kind, id, .. } => {
            // Already on a definition, return itself
            if let Some(def) = index.get_definition(kind.element_name(), &id) {
                DefinitionResult::found(def.clone())
            } else {
                DefinitionResult::not_found(&id, kind.element_name())
            }
        }
    }
}

/// Find all references to a symbol at a position
pub fn find_references(
    source: &str,
    line: u32,
    column: u32,
    index: &SymbolIndex,
    include_definition: bool,
) -> Option<ReferencesResult> {
    let target = detect_symbol_at(source, line, column)?;

    let (element_type, id) = match &target {
        SymbolTarget::Reference { kind, id, .. } => (kind.definition_element().to_string(), id.clone()),
        SymbolTarget::Definition { kind, id, .. } => (kind.canonical_type().to_string(), id.clone()),
    };

    let definition = if include_definition {
        index.get_definition(&element_type, &id).cloned()
    } else {
        None
    };

    let references = index
        .get_references(&element_type, &id)
        .into_iter()
        .cloned()
        .collect();

    Some(ReferencesResult::new(definition, references))
}

/// Convert line/column (1-based) to byte offset
fn line_col_to_offset(source: &str, line: u32, column: u32) -> Option<usize> {
    let mut current_line = 1u32;
    let mut current_col = 1u32;

    for (i, ch) in source.char_indices() {
        if current_line == line && current_col == column {
            return Some(i);
        }

        if ch == '\n' {
            if current_line == line {
                // Column is past end of line
                return None;
            }
            current_line += 1;
            current_col = 1;
        } else {
            current_col += 1;
        }
    }

    if current_line == line && current_col == column {
        return Some(source.len());
    }

    None
}

/// Find definition by id and type directly
pub fn find_definition_by_id(
    element_type: &str,
    id: &str,
    index: &SymbolIndex,
) -> DefinitionResult {
    if let Some(def) = index.get_definition(element_type, id) {
        DefinitionResult::found(def.clone())
    } else {
        DefinitionResult::not_found(id, element_type)
    }
}

/// Find all references by id and type directly
pub fn find_references_by_id(
    element_type: &str,
    id: &str,
    index: &SymbolIndex,
    include_definition: bool,
) -> ReferencesResult {
    let definition = if include_definition {
        index.get_definition(element_type, id).cloned()
    } else {
        None
    };

    let references = index
        .get_references(element_type, id)
        .into_iter()
        .cloned()
        .collect();

    ReferencesResult::new(definition, references)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::{Path, PathBuf};

    fn create_index_with(source: &str) -> SymbolIndex {
        let mut index = SymbolIndex::new();
        index
            .index_file(Path::new("test.wxs"), source)
            .unwrap();
        index
    }

    #[test]
    fn test_detect_on_component_ref() {
        let source = r#"<Wix><ComponentRef Id="MainComp" /></Wix>"#;
        let target = detect_symbol_at(source, 1, 20);

        assert!(target.is_some());
        if let Some(SymbolTarget::Reference { kind, id, .. }) = target {
            assert_eq!(kind, ReferenceKind::ComponentRef);
            assert_eq!(id, "MainComp");
        } else {
            panic!("Expected Reference");
        }
    }

    #[test]
    fn test_detect_on_component() {
        let source = r#"<Wix><Component Id="MainComp" Guid="*" /></Wix>"#;
        let target = detect_symbol_at(source, 1, 20);

        assert!(target.is_some());
        if let Some(SymbolTarget::Definition { kind, id, .. }) = target {
            assert_eq!(kind, DefinitionKind::Component);
            assert_eq!(id, "MainComp");
        } else {
            panic!("Expected Definition");
        }
    }

    #[test]
    fn test_go_to_definition_from_ref() {
        let source = r#"<Wix><Component Id="MainComp" /><ComponentRef Id="MainComp" /></Wix>"#;
        let index = create_index_with(source);

        // Position on ComponentRef Id value
        let result = go_to_definition(source, 1, 52, &index);

        assert!(result.definition.is_some());
        assert_eq!(result.definition.unwrap().id, "MainComp");
    }

    #[test]
    fn test_go_to_definition_missing() {
        let source = r#"<Wix><ComponentRef Id="MissingComp" /></Wix>"#;
        let index = create_index_with(source);

        let result = go_to_definition(source, 1, 25, &index);

        assert!(result.definition.is_none());
        assert!(result.error.is_some());
    }

    #[test]
    fn test_find_references_from_definition() {
        let source = r#"
<Wix>
    <Component Id="MainComp" />
    <Feature Id="F1"><ComponentRef Id="MainComp" /></Feature>
    <Feature Id="F2"><ComponentRef Id="MainComp" /></Feature>
</Wix>"#;
        let index = create_index_with(source);

        // Position on Component Id
        let result = find_references(source, 3, 20, &index, false).unwrap();

        assert_eq!(result.count, 2);
        assert!(result.definition.is_none());
    }

    #[test]
    fn test_find_references_include_definition() {
        let source = r#"
<Wix>
    <Component Id="MainComp" />
    <ComponentRef Id="MainComp" />
</Wix>"#;
        let index = create_index_with(source);

        let result = find_references(source, 3, 20, &index, true).unwrap();

        assert!(result.definition.is_some());
        assert_eq!(result.count, 1);
    }

    #[test]
    fn test_line_col_to_offset() {
        let source = "abc\ndef\nghi";
        assert_eq!(line_col_to_offset(source, 1, 1), Some(0));
        assert_eq!(line_col_to_offset(source, 1, 3), Some(2));
        assert_eq!(line_col_to_offset(source, 2, 1), Some(4));
        assert_eq!(line_col_to_offset(source, 3, 2), Some(9));
    }

    #[test]
    fn test_find_definition_by_id() {
        let source = r#"<Wix><Component Id="Test" /></Wix>"#;
        let index = create_index_with(source);

        let result = find_definition_by_id("Component", "Test", &index);
        assert!(result.definition.is_some());

        let result = find_definition_by_id("Component", "Missing", &index);
        assert!(result.definition.is_none());
    }

    #[test]
    fn test_find_references_by_id() {
        let source = r#"<Wix><Component Id="C1" /><ComponentRef Id="C1" /></Wix>"#;
        let index = create_index_with(source);

        let result = find_references_by_id("Component", "C1", &index, true);
        assert!(result.definition.is_some());
        assert_eq!(result.count, 1);
    }

    #[test]
    fn test_cross_file_definition() {
        let mut index = SymbolIndex::new();

        // Definition in one file
        index
            .index_file(
                Path::new("defs.wxs"),
                r#"<Wix><Component Id="SharedComp" /></Wix>"#,
            )
            .unwrap();

        // Reference in another
        let ref_source = r#"<Wix><ComponentRef Id="SharedComp" /></Wix>"#;
        index.index_file(Path::new("refs.wxs"), ref_source).unwrap();

        // Go to definition from refs.wxs should find defs.wxs
        let result = go_to_definition(ref_source, 1, 25, &index);

        assert!(result.definition.is_some());
        let def = result.definition.unwrap();
        assert_eq!(def.location.file, PathBuf::from("defs.wxs"));
    }

    #[test]
    fn test_no_symbol_at_whitespace() {
        let source = r#"<Wix>   </Wix>"#;
        let target = detect_symbol_at(source, 1, 7);
        assert!(target.is_none());
    }

    #[test]
    fn test_directory_ref_to_directory() {
        let source = r#"<Wix><Directory Id="TARGETDIR" /><DirectoryRef Id="TARGETDIR" /></Wix>"#;
        let index = create_index_with(source);

        let result = go_to_definition(source, 1, 50, &index);
        assert!(result.definition.is_some());
        assert_eq!(result.definition.unwrap().id, "TARGETDIR");
    }
}
