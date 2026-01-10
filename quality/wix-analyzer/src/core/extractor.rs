//! Symbol extraction from WiX documents

use roxmltree::Node;
use std::path::Path;

use super::document::WixDocument;
use super::types::{
    DefinitionKind, Location, Range, ReferenceKind, SymbolDefinition, SymbolReference,
};

/// Result of extracting symbols from a document
#[derive(Debug, Default)]
pub struct ExtractionResult {
    pub definitions: Vec<SymbolDefinition>,
    pub references: Vec<SymbolReference>,
}

impl ExtractionResult {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn merge(&mut self, other: ExtractionResult) {
        self.definitions.extend(other.definitions);
        self.references.extend(other.references);
    }
}

/// Extract symbols from a WiX document
pub fn extract_symbols(doc: &WixDocument) -> ExtractionResult {
    let mut result = ExtractionResult::new();
    extract_from_node(doc.root(), doc, &mut result);
    result
}

/// Extract symbols from source string
pub fn extract_from_source(source: &str, file: &Path) -> Result<ExtractionResult, String> {
    let doc = WixDocument::parse(source, file)?;
    Ok(extract_symbols(&doc))
}

fn extract_from_node(node: Node, doc: &WixDocument, result: &mut ExtractionResult) {
    if node.is_element() {
        let tag_name = node.tag_name().name();

        // Check if it's a definition
        if let Some(kind) = DefinitionKind::from_element_name(tag_name) {
            let id_attr = kind.id_attribute();
            if let Some(id) = node.attribute(id_attr) {
                let range = doc.node_range(&node);
                let location = Location::new(doc.file().to_path_buf(), range);

                let mut def = SymbolDefinition::new(id, kind, location);

                // Add detail for some types
                match kind {
                    DefinitionKind::Component => {
                        if let Some(guid) = node.attribute("Guid") {
                            def = def.with_detail(format!("Guid: {}", guid));
                        }
                    }
                    DefinitionKind::Package | DefinitionKind::Bundle => {
                        if let Some(version) = node.attribute("Version") {
                            def = def.with_detail(format!("Version: {}", version));
                        }
                    }
                    _ => {}
                }

                result.definitions.push(def);
            }
        }

        // Check if it's a reference
        if let Some(kind) = ReferenceKind::from_element_name(tag_name) {
            if let Some(id) = node.attribute("Id") {
                let range = doc.node_range(&node);
                let location = Location::new(doc.file().to_path_buf(), range);
                result
                    .references
                    .push(SymbolReference::new(id, kind, location));
            }
        }
    }

    // Recurse into children
    for child in node.children() {
        extract_from_node(child, doc, result);
    }
}

/// Detect symbol at a specific position in a document
pub fn symbol_at_position(
    doc: &WixDocument,
    line: usize,
    column: usize,
) -> Option<SymbolAtPosition> {
    let node = doc.element_at(line, column)?;
    let tag_name = node.tag_name().name();

    // Check if cursor is on a reference element
    if let Some(kind) = ReferenceKind::from_element_name(tag_name) {
        if let Some(id) = node.attribute("Id") {
            return Some(SymbolAtPosition::Reference {
                id: id.to_string(),
                kind,
                range: doc.node_range(&node),
            });
        }
    }

    // Check if cursor is on a definition element
    if let Some(kind) = DefinitionKind::from_element_name(tag_name) {
        let id_attr = kind.id_attribute();
        if let Some(id) = node.attribute(id_attr) {
            return Some(SymbolAtPosition::Definition {
                id: id.to_string(),
                kind,
                range: doc.node_range(&node),
            });
        }
    }

    None
}

/// Symbol found at a position
#[derive(Debug, Clone)]
pub enum SymbolAtPosition {
    Reference {
        id: String,
        kind: ReferenceKind,
        range: Range,
    },
    Definition {
        id: String,
        kind: DefinitionKind,
        range: Range,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_definitions() {
        let source = r#"<Wix>
            <Component Id="C1" Guid="*" />
            <Directory Id="D1" />
            <Feature Id="F1" />
        </Wix>"#;

        let result = extract_from_source(source, Path::new("test.wxs")).unwrap();

        assert_eq!(result.definitions.len(), 3);
        assert!(result.definitions.iter().any(|d| d.id == "C1"));
        assert!(result.definitions.iter().any(|d| d.id == "D1"));
        assert!(result.definitions.iter().any(|d| d.id == "F1"));
    }

    #[test]
    fn test_extract_references() {
        let source = r#"<Wix>
            <ComponentRef Id="C1" />
            <DirectoryRef Id="D1" />
            <FeatureRef Id="F1" />
        </Wix>"#;

        let result = extract_from_source(source, Path::new("test.wxs")).unwrap();

        assert_eq!(result.references.len(), 3);
        assert!(result.references.iter().any(|r| r.id == "C1"));
        assert!(result.references.iter().any(|r| r.id == "D1"));
        assert!(result.references.iter().any(|r| r.id == "F1"));
    }

    #[test]
    fn test_extract_package() {
        let source = r#"<Wix><Package Name="TestApp" Version="1.0" /></Wix>"#;

        let result = extract_from_source(source, Path::new("test.wxs")).unwrap();

        assert_eq!(result.definitions.len(), 1);
        assert_eq!(result.definitions[0].id, "TestApp");
        assert_eq!(result.definitions[0].kind, DefinitionKind::Package);
    }

    #[test]
    fn test_symbol_at_position() {
        let source = "<Wix>\n  <Component Id=\"C1\" />\n</Wix>";
        let doc = WixDocument::parse(source, Path::new("test.wxs")).unwrap();

        let symbol = symbol_at_position(&doc, 2, 5).unwrap();
        match symbol {
            SymbolAtPosition::Definition { id, kind, .. } => {
                assert_eq!(id, "C1");
                assert_eq!(kind, DefinitionKind::Component);
            }
            _ => panic!("Expected definition"),
        }
    }

    #[test]
    fn test_component_with_guid_detail() {
        let source = r#"<Wix><Component Id="C1" Guid="*" /></Wix>"#;

        let result = extract_from_source(source, Path::new("test.wxs")).unwrap();

        assert_eq!(result.definitions.len(), 1);
        assert!(result.definitions[0]
            .detail
            .as_ref()
            .unwrap()
            .contains("Guid: *"));
    }

    #[test]
    fn test_extraction_result_merge() {
        let source1 = r#"<Wix><Component Id="C1" /></Wix>"#;
        let source2 = r#"<Wix><Component Id="C2" /></Wix>"#;

        let mut result1 = extract_from_source(source1, Path::new("file1.wxs")).unwrap();
        let result2 = extract_from_source(source2, Path::new("file2.wxs")).unwrap();

        result1.merge(result2);

        assert_eq!(result1.definitions.len(), 2);
        assert!(result1.definitions.iter().any(|d| d.id == "C1"));
        assert!(result1.definitions.iter().any(|d| d.id == "C2"));
    }

    #[test]
    fn test_extraction_result_new() {
        let result = ExtractionResult::new();
        assert!(result.definitions.is_empty());
        assert!(result.references.is_empty());
    }
}
