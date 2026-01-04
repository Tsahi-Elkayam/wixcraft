//! Extract definitions and references from WiX XML files

use crate::types::{
    DefinitionKind, Location, Range, ReferenceKind, SymbolDefinition, SymbolReference,
};
use roxmltree::{Document, Node};
use std::path::Path;

/// Extraction result for a single file
#[derive(Debug, Default)]
pub struct ExtractionResult {
    pub definitions: Vec<SymbolDefinition>,
    pub references: Vec<SymbolReference>,
}

/// Extract all definitions and references from source
pub fn extract_from_source(source: &str, file: &Path) -> Result<ExtractionResult, String> {
    let doc = Document::parse(source).map_err(|e| format!("XML parse error: {}", e))?;

    let mut result = ExtractionResult::default();
    extract_from_node(doc.root(), source, file, &mut result);

    Ok(result)
}

/// Recursively extract from a node
fn extract_from_node(node: Node, source: &str, file: &Path, result: &mut ExtractionResult) {
    if node.is_element() {
        let tag_name = node.tag_name().name();

        // Check if it's a definition element
        if let Some(kind) = DefinitionKind::from_element_name(tag_name) {
            if let Some(def) = create_definition(&node, kind, source, file) {
                result.definitions.push(def);
            }
        }

        // Check if it's a reference element
        if let Some(kind) = ReferenceKind::from_element_name(tag_name) {
            if let Some(reference) = create_reference(&node, kind, source, file) {
                result.references.push(reference);
            }
        }
    }

    // Recurse into children
    for child in node.children() {
        extract_from_node(child, source, file, result);
    }
}

/// Create a definition from a node
fn create_definition(
    node: &Node,
    kind: DefinitionKind,
    source: &str,
    file: &Path,
) -> Option<SymbolDefinition> {
    // Get the Id attribute (or Name for Package/Bundle)
    let id = match kind {
        DefinitionKind::Package | DefinitionKind::Bundle => {
            node.attribute("Name").or_else(|| node.attribute("Id"))
        }
        _ => node.attribute("Id"),
    }?;

    let range = get_node_range(node, source);
    let location = Location::new(file.to_path_buf(), range);

    let mut def = SymbolDefinition::new(id.to_string(), kind, location);

    // Add detail based on element type
    let detail = match kind {
        DefinitionKind::Directory | DefinitionKind::StandardDirectory => {
            node.attribute("Name").map(|s| s.to_string())
        }
        DefinitionKind::Feature => node.attribute("Title").map(|s| s.to_string()),
        DefinitionKind::Package | DefinitionKind::Bundle => {
            node.attribute("Version").map(|s| s.to_string())
        }
        _ => None,
    };

    if let Some(d) = detail {
        def = def.with_detail(d);
    }

    Some(def)
}

/// Create a reference from a node
fn create_reference(
    node: &Node,
    kind: ReferenceKind,
    source: &str,
    file: &Path,
) -> Option<SymbolReference> {
    let id = node.attribute("Id")?;
    let range = get_node_range(node, source);
    let location = Location::new(file.to_path_buf(), range);

    Some(SymbolReference::new(id.to_string(), kind, location))
}

/// Get the range of a node in source
fn get_node_range(node: &Node, source: &str) -> Range {
    let node_range = node.range();
    Range::from_offsets(source, node_range.start, node_range.end)
}

/// Get the range of a specific attribute value in a node
pub fn get_attribute_range(node: &Node, attr_name: &str, source: &str) -> Option<Range> {
    let node_start = node.range().start;
    let node_text = &source[node.range()];

    // Search for attr="value" or attr='value'
    let patterns = [
        format!("{}=\"", attr_name),
        format!("{}='", attr_name),
        format!("{} =\"", attr_name),
        format!("{} ='", attr_name),
    ];

    for pattern in &patterns {
        if let Some(attr_pos) = node_text.find(pattern.as_str()) {
            let quote_char = if pattern.ends_with('"') { '"' } else { '\'' };
            let value_start = attr_pos + pattern.len();

            if let Some(value_end) = node_text[value_start..].find(quote_char) {
                let abs_start = node_start + value_start;
                let abs_end = node_start + value_start + value_end;
                return Some(Range::from_offsets(source, abs_start, abs_end));
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn test_file() -> PathBuf {
        PathBuf::from("test.wxs")
    }

    #[test]
    fn test_extract_component_definition() {
        let source = r#"<Wix><Component Id="MainComp" Guid="*" /></Wix>"#;
        let result = extract_from_source(source, &test_file()).unwrap();

        assert_eq!(result.definitions.len(), 1);
        assert_eq!(result.definitions[0].id, "MainComp");
        assert_eq!(result.definitions[0].kind, DefinitionKind::Component);
    }

    #[test]
    fn test_extract_directory_with_name() {
        let source = r#"<Wix><Directory Id="INSTALLFOLDER" Name="MyApp" /></Wix>"#;
        let result = extract_from_source(source, &test_file()).unwrap();

        assert_eq!(result.definitions.len(), 1);
        assert_eq!(result.definitions[0].id, "INSTALLFOLDER");
        assert_eq!(result.definitions[0].detail, Some("MyApp".to_string()));
    }

    #[test]
    fn test_extract_feature_with_title() {
        let source = r#"<Wix><Feature Id="MainFeature" Title="Main Application" /></Wix>"#;
        let result = extract_from_source(source, &test_file()).unwrap();

        assert_eq!(result.definitions.len(), 1);
        assert_eq!(result.definitions[0].id, "MainFeature");
        assert_eq!(
            result.definitions[0].detail,
            Some("Main Application".to_string())
        );
    }

    #[test]
    fn test_extract_component_ref() {
        let source = r#"<Wix><ComponentRef Id="MainComp" /></Wix>"#;
        let result = extract_from_source(source, &test_file()).unwrap();

        assert_eq!(result.references.len(), 1);
        assert_eq!(result.references[0].id, "MainComp");
        assert_eq!(result.references[0].kind, ReferenceKind::ComponentRef);
    }

    #[test]
    fn test_extract_directory_ref() {
        let source = r#"<Wix><DirectoryRef Id="INSTALLFOLDER" /></Wix>"#;
        let result = extract_from_source(source, &test_file()).unwrap();

        assert_eq!(result.references.len(), 1);
        assert_eq!(result.references[0].id, "INSTALLFOLDER");
        assert_eq!(result.references[0].kind, ReferenceKind::DirectoryRef);
    }

    #[test]
    fn test_extract_both_definitions_and_references() {
        let source = r#"
<Wix>
    <Component Id="Comp1" />
    <Feature Id="Feature1">
        <ComponentRef Id="Comp1" />
    </Feature>
</Wix>"#;
        let result = extract_from_source(source, &test_file()).unwrap();

        assert_eq!(result.definitions.len(), 2); // Component + Feature
        assert_eq!(result.references.len(), 1); // ComponentRef
    }

    #[test]
    fn test_extract_nested_elements() {
        let source = r#"
<Wix>
    <Directory Id="TARGETDIR">
        <Directory Id="ProgramFilesFolder">
            <Directory Id="INSTALLFOLDER" Name="MyApp">
                <Component Id="MainComp">
                </Component>
            </Directory>
        </Directory>
    </Directory>
</Wix>"#;
        let result = extract_from_source(source, &test_file()).unwrap();

        assert_eq!(result.definitions.len(), 4); // 3 Directories + 1 Component
    }

    #[test]
    fn test_extract_package() {
        let source = r#"<Wix><Package Name="MyApp" Version="1.0.0" /></Wix>"#;
        let result = extract_from_source(source, &test_file()).unwrap();

        assert_eq!(result.definitions.len(), 1);
        assert_eq!(result.definitions[0].id, "MyApp");
        assert_eq!(result.definitions[0].detail, Some("1.0.0".to_string()));
    }

    #[test]
    fn test_extract_fragment() {
        let source = r#"<Wix><Fragment Id="UIFragment" /></Wix>"#;
        let result = extract_from_source(source, &test_file()).unwrap();

        assert_eq!(result.definitions.len(), 1);
        assert_eq!(result.definitions[0].id, "UIFragment");
        assert_eq!(result.definitions[0].kind, DefinitionKind::Fragment);
    }

    #[test]
    fn test_extract_component_group() {
        let source = r#"<Wix><ComponentGroup Id="MainGroup" /></Wix>"#;
        let result = extract_from_source(source, &test_file()).unwrap();

        assert_eq!(result.definitions.len(), 1);
        assert_eq!(result.definitions[0].kind, DefinitionKind::ComponentGroup);
    }

    #[test]
    fn test_extract_feature_group() {
        let source = r#"<Wix><FeatureGroup Id="MainFeatures" /></Wix>"#;
        let result = extract_from_source(source, &test_file()).unwrap();

        assert_eq!(result.definitions.len(), 1);
        assert_eq!(result.definitions[0].kind, DefinitionKind::FeatureGroup);
    }

    #[test]
    fn test_extract_custom_action() {
        let source = r#"<Wix><CustomAction Id="SetInstallDir" /></Wix>"#;
        let result = extract_from_source(source, &test_file()).unwrap();

        assert_eq!(result.definitions.len(), 1);
        assert_eq!(result.definitions[0].kind, DefinitionKind::CustomAction);
    }

    #[test]
    fn test_extract_property() {
        let source = r#"<Wix><Property Id="INSTALLDIR" /></Wix>"#;
        let result = extract_from_source(source, &test_file()).unwrap();

        assert_eq!(result.definitions.len(), 1);
        assert_eq!(result.definitions[0].kind, DefinitionKind::Property);
    }

    #[test]
    fn test_extract_standard_directory() {
        let source = r#"<Wix><StandardDirectory Id="ProgramFilesFolder" /></Wix>"#;
        let result = extract_from_source(source, &test_file()).unwrap();

        assert_eq!(result.definitions.len(), 1);
        assert_eq!(result.definitions[0].kind, DefinitionKind::StandardDirectory);
    }

    #[test]
    fn test_element_without_id_skipped() {
        let source = r#"<Wix><Component Guid="*" /></Wix>"#;
        let result = extract_from_source(source, &test_file()).unwrap();

        assert_eq!(result.definitions.len(), 0);
    }

    #[test]
    fn test_invalid_xml() {
        let source = "<Wix><Invalid";
        let result = extract_from_source(source, &test_file());
        assert!(result.is_err());
    }

    #[test]
    fn test_location_has_correct_file() {
        let source = r#"<Wix><Component Id="Test" /></Wix>"#;
        let file = PathBuf::from("/path/to/test.wxs");
        let result = extract_from_source(source, &file).unwrap();

        assert_eq!(result.definitions[0].location.file, file);
    }

    #[test]
    fn test_get_attribute_range() {
        let source = r#"<Component Id="MainComp" Guid="*" />"#;
        let doc = Document::parse(source).unwrap();
        let node = doc.root().first_child().unwrap();

        let range = get_attribute_range(&node, "Id", source).unwrap();
        assert_eq!(range.start.character, 16); // After Id="
        assert_eq!(range.end.character, 24); // Before "
    }
}
