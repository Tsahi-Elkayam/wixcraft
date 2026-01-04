//! Parent/child relationship validation

use crate::types::{Diagnostic, Range};
use roxmltree::{Document, Node};
use std::collections::{HashMap, HashSet};
use std::path::Path;

/// Known parent-child relationships
/// Maps child element -> set of valid parent elements
fn get_valid_parents() -> HashMap<&'static str, HashSet<&'static str>> {
    let mut map = HashMap::new();

    // File must be in Component
    map.insert(
        "File",
        ["Component"].iter().copied().collect(),
    );

    // RegistryKey/RegistryValue can be in Component or RegistryKey
    map.insert(
        "RegistryKey",
        ["Component", "RegistryKey"].iter().copied().collect(),
    );
    map.insert(
        "RegistryValue",
        ["Component", "RegistryKey", "RegistryValue"].iter().copied().collect(),
    );

    // Component can be in Directory, DirectoryRef, StandardDirectory, Fragment, ComponentGroup
    map.insert(
        "Component",
        ["Directory", "DirectoryRef", "StandardDirectory", "Fragment", "ComponentGroup", "Wix"]
            .iter()
            .copied()
            .collect(),
    );

    // ComponentRef can be in Feature, FeatureGroup, Fragment
    map.insert(
        "ComponentRef",
        ["Feature", "FeatureGroup", "FeatureRef", "Fragment", "Wix"]
            .iter()
            .copied()
            .collect(),
    );

    // Feature can be in Package, Feature, Bundle, Fragment
    map.insert(
        "Feature",
        ["Package", "Feature", "Fragment", "Module", "Wix"]
            .iter()
            .copied()
            .collect(),
    );

    // Directory can be in Directory, DirectoryRef, StandardDirectory, Fragment, Package
    map.insert(
        "Directory",
        ["Directory", "DirectoryRef", "StandardDirectory", "Fragment", "Package", "Module", "Wix"]
            .iter()
            .copied()
            .collect(),
    );

    // ServiceInstall/ServiceControl in Component
    map.insert(
        "ServiceInstall",
        ["Component"].iter().copied().collect(),
    );
    map.insert(
        "ServiceControl",
        ["Component"].iter().copied().collect(),
    );

    // Shortcut in Component or File
    map.insert(
        "Shortcut",
        ["Component", "File"].iter().copied().collect(),
    );

    map
}

/// Validator for parent-child relationships
pub struct RelationshipValidator {
    valid_parents: HashMap<&'static str, HashSet<&'static str>>,
}

impl RelationshipValidator {
    pub fn new() -> Self {
        Self {
            valid_parents: get_valid_parents(),
        }
    }

    /// Validate a source file
    pub fn validate(&self, source: &str, _file: &Path) -> Result<Vec<Diagnostic>, String> {
        let doc = Document::parse(source).map_err(|e| format!("XML parse error: {}", e))?;
        let mut diagnostics = Vec::new();
        self.validate_node(doc.root(), None, source, &mut diagnostics);
        Ok(diagnostics)
    }

    fn validate_node(
        &self,
        node: Node,
        parent_name: Option<&str>,
        source: &str,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        if node.is_element() {
            let tag_name = node.tag_name().name();

            // Check if this element has parent restrictions
            if let Some(valid_parents) = self.valid_parents.get(tag_name) {
                if let Some(parent) = parent_name {
                    if !valid_parents.contains(parent) {
                        let range = get_node_range(&node, source);
                        let valid_list: Vec<_> = valid_parents.iter().copied().collect();
                        diagnostics.push(
                            Diagnostic::error(
                                range,
                                format!(
                                    "{} cannot be a child of {}. Valid parents: {}",
                                    tag_name,
                                    parent,
                                    valid_list.join(", ")
                                ),
                            )
                            .with_code("invalid-parent"),
                        );
                    }
                }
            }

            // Recurse with current element as parent
            for child in node.children() {
                self.validate_node(child, Some(tag_name), source, diagnostics);
            }
        } else {
            // Non-element nodes pass through parent
            for child in node.children() {
                self.validate_node(child, parent_name, source, diagnostics);
            }
        }
    }
}

impl Default for RelationshipValidator {
    fn default() -> Self {
        Self::new()
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
    fn test_file_in_component_valid() {
        let source = r#"<Wix><Component Id="C1"><File Id="F1" /></Component></Wix>"#;
        let validator = RelationshipValidator::new();
        let diagnostics = validator.validate(source, Path::new("test.wxs")).unwrap();
        assert!(diagnostics.is_empty());
    }

    #[test]
    fn test_file_not_in_component() {
        let source = r#"<Wix><Directory Id="D1"><File Id="F1" /></Directory></Wix>"#;
        let validator = RelationshipValidator::new();
        let diagnostics = validator.validate(source, Path::new("test.wxs")).unwrap();

        assert_eq!(diagnostics.len(), 1);
        assert!(diagnostics[0].message.contains("cannot be a child of"));
    }

    #[test]
    fn test_component_in_directory_valid() {
        let source = r#"<Wix><Directory Id="D1"><Component Id="C1" /></Directory></Wix>"#;
        let validator = RelationshipValidator::new();
        let diagnostics = validator.validate(source, Path::new("test.wxs")).unwrap();
        assert!(diagnostics.is_empty());
    }

    #[test]
    fn test_component_in_feature_invalid() {
        let source = r#"<Wix><Feature Id="F1"><Component Id="C1" /></Feature></Wix>"#;
        let validator = RelationshipValidator::new();
        let diagnostics = validator.validate(source, Path::new("test.wxs")).unwrap();

        assert_eq!(diagnostics.len(), 1);
    }

    #[test]
    fn test_component_ref_in_feature_valid() {
        let source = r#"<Wix><Feature Id="F1"><ComponentRef Id="C1" /></Feature></Wix>"#;
        let validator = RelationshipValidator::new();
        let diagnostics = validator.validate(source, Path::new("test.wxs")).unwrap();
        assert!(diagnostics.is_empty());
    }

    #[test]
    fn test_registry_key_in_component() {
        let source = r#"<Wix><Component Id="C1"><RegistryKey Root="HKLM" Key="Test" /></Component></Wix>"#;
        let validator = RelationshipValidator::new();
        let diagnostics = validator.validate(source, Path::new("test.wxs")).unwrap();
        assert!(diagnostics.is_empty());
    }

    #[test]
    fn test_nested_registry_key() {
        let source = r#"<Wix><Component Id="C1"><RegistryKey Root="HKLM" Key="Parent"><RegistryKey Key="Child" /></RegistryKey></Component></Wix>"#;
        let validator = RelationshipValidator::new();
        let diagnostics = validator.validate(source, Path::new("test.wxs")).unwrap();
        assert!(diagnostics.is_empty());
    }

    #[test]
    fn test_service_install_in_component() {
        let source = r#"<Wix><Component Id="C1"><ServiceInstall Name="Svc" /></Component></Wix>"#;
        let validator = RelationshipValidator::new();
        let diagnostics = validator.validate(source, Path::new("test.wxs")).unwrap();
        assert!(diagnostics.is_empty());
    }
}
