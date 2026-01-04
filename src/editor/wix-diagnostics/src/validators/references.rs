//! Reference validation - check that references point to existing definitions

use crate::types::{Diagnostic, Range};
use roxmltree::{Document, Node};
use std::collections::{HashMap, HashSet};
use std::path::Path;

/// Reference element to definition element mapping
const REFERENCE_MAPPINGS: &[(&str, &str)] = &[
    ("ComponentRef", "Component"),
    ("ComponentGroupRef", "ComponentGroup"),
    ("DirectoryRef", "Directory"),
    ("FeatureRef", "Feature"),
    ("FeatureGroupRef", "FeatureGroup"),
    ("PropertyRef", "Property"),
    ("CustomActionRef", "CustomAction"),
    ("BinaryRef", "Binary"),
];

/// Validator for references
pub struct ReferenceValidator {
    /// Known definitions: type -> set of ids
    definitions: HashMap<String, HashSet<String>>,
}

impl ReferenceValidator {
    /// Create a new reference validator
    pub fn new() -> Self {
        Self {
            definitions: HashMap::new(),
        }
    }

    /// Index definitions from a source file
    pub fn index_file(&mut self, source: &str) -> Result<(), String> {
        let doc = Document::parse(source).map_err(|e| format!("XML parse error: {}", e))?;
        self.index_node(doc.root());
        Ok(())
    }

    fn index_node(&mut self, node: Node) {
        if node.is_element() {
            let tag_name = node.tag_name().name();

            // Check if this is a definition element
            if is_definition_element(tag_name) {
                let id_attr = match tag_name {
                    "Package" | "Bundle" => node.attribute("Name").or_else(|| node.attribute("Id")),
                    _ => node.attribute("Id"),
                };

                if let Some(id) = id_attr {
                    // Use canonical type for storage
                    let canonical = canonical_type(tag_name);
                    self.definitions
                        .entry(canonical.to_string())
                        .or_default()
                        .insert(id.to_string());
                }
            }
        }

        for child in node.children() {
            self.index_node(child);
        }
    }

    /// Add a known definition
    pub fn add_definition(&mut self, element_type: &str, id: &str) {
        let canonical = canonical_type(element_type);
        self.definitions
            .entry(canonical.to_string())
            .or_default()
            .insert(id.to_string());
    }

    /// Check if a definition exists
    pub fn has_definition(&self, element_type: &str, id: &str) -> bool {
        let canonical = canonical_type(element_type);
        self.definitions
            .get(canonical)
            .map(|ids| ids.contains(id))
            .unwrap_or(false)
    }

    /// Validate a source file and return diagnostics
    pub fn validate(&self, source: &str, _file: &Path) -> Result<Vec<Diagnostic>, String> {
        let doc = Document::parse(source).map_err(|e| format!("XML parse error: {}", e))?;
        let mut diagnostics = Vec::new();
        self.validate_node(doc.root(), source, &mut diagnostics);
        Ok(diagnostics)
    }

    fn validate_node(&self, node: Node, source: &str, diagnostics: &mut Vec<Diagnostic>) {
        if node.is_element() {
            let tag_name = node.tag_name().name();

            // Check if this is a reference element
            if let Some(def_type) = get_definition_type(tag_name) {
                if let Some(id) = node.attribute("Id") {
                    if !self.has_definition(def_type, id) {
                        let range = get_node_range(&node, source);
                        diagnostics.push(
                            Diagnostic::error(
                                range,
                                format!("No {} found with Id '{}'", def_type, id),
                            )
                            .with_code("invalid-reference"),
                        );
                    }
                }
            }
        }

        for child in node.children() {
            self.validate_node(child, source, diagnostics);
        }
    }
}

impl Default for ReferenceValidator {
    fn default() -> Self {
        Self::new()
    }
}

/// Check if element is a definition element
fn is_definition_element(tag_name: &str) -> bool {
    matches!(
        tag_name,
        "Component"
            | "ComponentGroup"
            | "Directory"
            | "StandardDirectory"
            | "Feature"
            | "FeatureGroup"
            | "Property"
            | "CustomAction"
            | "Binary"
            | "Fragment"
            | "Package"
            | "Module"
            | "Bundle"
    )
}

/// Get the definition type for a reference element
fn get_definition_type(tag_name: &str) -> Option<&'static str> {
    for (ref_name, def_name) in REFERENCE_MAPPINGS {
        if tag_name == *ref_name {
            return Some(def_name);
        }
    }
    None
}

/// Get canonical type for grouping
fn canonical_type(element_type: &str) -> &str {
    match element_type {
        "Component" | "ComponentGroup" => "Component",
        "Directory" | "StandardDirectory" => "Directory",
        "Feature" | "FeatureGroup" => "Feature",
        other => other,
    }
}

/// Get range of a node
fn get_node_range(node: &Node, source: &str) -> Range {
    let r = node.range();
    Range::from_offsets(source, r.start, r.end)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_reference() {
        let source = r#"<Wix><Component Id="C1" /><ComponentRef Id="C1" /></Wix>"#;

        let mut validator = ReferenceValidator::new();
        validator.index_file(source).unwrap();

        let diagnostics = validator.validate(source, Path::new("test.wxs")).unwrap();
        assert!(diagnostics.is_empty());
    }

    #[test]
    fn test_invalid_reference() {
        let source = r#"<Wix><ComponentRef Id="Missing" /></Wix>"#;

        let validator = ReferenceValidator::new();
        let diagnostics = validator.validate(source, Path::new("test.wxs")).unwrap();

        assert_eq!(diagnostics.len(), 1);
        assert!(diagnostics[0].message.contains("No Component"));
    }

    #[test]
    fn test_directory_ref() {
        let source = r#"<Wix><Directory Id="D1" /><DirectoryRef Id="D1" /></Wix>"#;

        let mut validator = ReferenceValidator::new();
        validator.index_file(source).unwrap();

        let diagnostics = validator.validate(source, Path::new("test.wxs")).unwrap();
        assert!(diagnostics.is_empty());
    }

    #[test]
    fn test_feature_ref() {
        let source = r#"<Wix><Feature Id="F1" /><FeatureRef Id="F1" /></Wix>"#;

        let mut validator = ReferenceValidator::new();
        validator.index_file(source).unwrap();

        let diagnostics = validator.validate(source, Path::new("test.wxs")).unwrap();
        assert!(diagnostics.is_empty());
    }

    #[test]
    fn test_component_group_ref() {
        let source = r#"<Wix><ComponentGroup Id="G1" /><ComponentGroupRef Id="G1" /></Wix>"#;

        let mut validator = ReferenceValidator::new();
        validator.index_file(source).unwrap();

        let diagnostics = validator.validate(source, Path::new("test.wxs")).unwrap();
        assert!(diagnostics.is_empty());
    }

    #[test]
    fn test_multiple_invalid_references() {
        let source = r#"<Wix>
            <ComponentRef Id="Missing1" />
            <DirectoryRef Id="Missing2" />
        </Wix>"#;

        let validator = ReferenceValidator::new();
        let diagnostics = validator.validate(source, Path::new("test.wxs")).unwrap();

        assert_eq!(diagnostics.len(), 2);
    }

    #[test]
    fn test_add_definition_manually() {
        let mut validator = ReferenceValidator::new();
        validator.add_definition("Component", "External");

        let source = r#"<Wix><ComponentRef Id="External" /></Wix>"#;
        let diagnostics = validator.validate(source, Path::new("test.wxs")).unwrap();

        assert!(diagnostics.is_empty());
    }
}
