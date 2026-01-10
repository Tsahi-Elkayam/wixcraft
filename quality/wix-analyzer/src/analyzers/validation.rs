//! Validation analyzer - references, relationships, attributes

use crate::core::{
    AnalysisResult, Category, Diagnostic, Location, ReferenceKind, SymbolIndex,
    WixDocument,
};
use regex::Regex;
use roxmltree::Node;
use std::collections::{HashMap, HashSet};
use std::sync::LazyLock;
use super::Analyzer;

/// GUID regex pattern
static GUID_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^(\*|\{?[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}\}?)$").unwrap()
});

/// Validation analyzer for WiX files
pub struct ValidationAnalyzer {
    valid_parents: HashMap<&'static str, HashSet<&'static str>>,
}

impl ValidationAnalyzer {
    pub fn new() -> Self {
        Self {
            valid_parents: get_valid_parents(),
        }
    }
}

impl Default for ValidationAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl Analyzer for ValidationAnalyzer {
    fn analyze(&self, doc: &WixDocument, index: &SymbolIndex) -> AnalysisResult {
        let mut result = AnalysisResult::new();

        // Validate references
        self.validate_references(doc, index, &mut result);

        // Validate relationships
        self.validate_relationships(doc, &mut result);

        // Validate attributes
        self.validate_attributes(doc, &mut result);

        result
    }
}

impl ValidationAnalyzer {
    fn validate_references(&self, doc: &WixDocument, index: &SymbolIndex, result: &mut AnalysisResult) {
        for node in doc.root().descendants() {
            if let Some(kind) = ReferenceKind::from_element_name(node.tag_name().name()) {
                if let Some(id) = node.attribute("Id") {
                    let element_type = kind.definition_element();
                    if !index.has_definition(element_type, id) {
                        let range = doc.node_range(&node);
                        let location = Location::new(doc.file().to_path_buf(), range);
                        result.add(
                            Diagnostic::error(
                                "VAL-REF-001",
                                Category::Validation,
                                format!("No {} found with Id '{}'", element_type, id),
                                location,
                            )
                            .with_help(format!(
                                "Ensure a {} with Id='{}' is defined in this file or an included file",
                                element_type, id
                            )),
                        );
                    }
                }
            }
        }
    }

    fn validate_relationships(&self, doc: &WixDocument, result: &mut AnalysisResult) {
        self.check_node_relationships(doc.root(), None, doc, result);
    }

    fn check_node_relationships(
        &self,
        node: Node,
        parent_name: Option<&str>,
        doc: &WixDocument,
        result: &mut AnalysisResult,
    ) {
        if node.is_element() {
            let tag_name = node.tag_name().name();

            // Check if this element has parent restrictions
            if let Some(valid_parents) = self.valid_parents.get(tag_name) {
                if let Some(parent) = parent_name {
                    if !valid_parents.contains(parent) {
                        let range = doc.node_range(&node);
                        let location = Location::new(doc.file().to_path_buf(), range);
                        let valid_list: Vec<_> = valid_parents.iter().copied().collect();
                        result.add(
                            Diagnostic::error(
                                "VAL-REL-001",
                                Category::Validation,
                                format!(
                                    "{} cannot be a child of {}. Valid parents: {}",
                                    tag_name,
                                    parent,
                                    valid_list.join(", ")
                                ),
                                location,
                            ),
                        );
                    }
                }
            }

            // Recurse with current element as parent
            for child in node.children() {
                self.check_node_relationships(child, Some(tag_name), doc, result);
            }
        } else {
            // Non-element nodes pass through parent
            for child in node.children() {
                self.check_node_relationships(child, parent_name, doc, result);
            }
        }
    }

    fn validate_attributes(&self, doc: &WixDocument, result: &mut AnalysisResult) {
        for node in doc.root().descendants() {
            let tag_name = node.tag_name().name();

            // Check required attributes and validate types
            match tag_name {
                "Directory" => {
                    if node.attribute("Id").is_none() {
                        let range = doc.node_range(&node);
                        let location = Location::new(doc.file().to_path_buf(), range);
                        result.add(Diagnostic::error(
                            "VAL-ATTR-001",
                            Category::Validation,
                            "Directory requires 'Id' attribute",
                            location,
                        ));
                    }
                }
                "Feature" => {
                    if node.attribute("Id").is_none() {
                        let range = doc.node_range(&node);
                        let location = Location::new(doc.file().to_path_buf(), range);
                        result.add(Diagnostic::error(
                            "VAL-ATTR-001",
                            Category::Validation,
                            "Feature requires 'Id' attribute",
                            location,
                        ));
                    }
                    // Validate Display enum
                    if let Some(display) = node.attribute("Display") {
                        if !matches!(display, "expand" | "collapse" | "hidden") {
                            let range = doc.node_range(&node);
                            let location = Location::new(doc.file().to_path_buf(), range);
                            result.add(Diagnostic::error(
                                "VAL-ATTR-002",
                                Category::Validation,
                                format!(
                                    "Invalid value '{}' for Feature.Display. Valid values: expand, collapse, hidden",
                                    display
                                ),
                                location,
                            ));
                        }
                    }
                }
                "Property" => {
                    if node.attribute("Id").is_none() {
                        let range = doc.node_range(&node);
                        let location = Location::new(doc.file().to_path_buf(), range);
                        result.add(Diagnostic::error(
                            "VAL-ATTR-001",
                            Category::Validation,
                            "Property requires 'Id' attribute",
                            location,
                        ));
                    }
                }
                "CustomAction" => {
                    if node.attribute("Id").is_none() {
                        let range = doc.node_range(&node);
                        let location = Location::new(doc.file().to_path_buf(), range);
                        result.add(Diagnostic::error(
                            "VAL-ATTR-001",
                            Category::Validation,
                            "CustomAction requires 'Id' attribute",
                            location,
                        ));
                    }
                    // Validate Execute enum
                    if let Some(execute) = node.attribute("Execute") {
                        if !matches!(
                            execute,
                            "immediate" | "deferred" | "rollback" | "commit" | "oncePerProcess" | "firstSequence" | "secondSequence"
                        ) {
                            let range = doc.node_range(&node);
                            let location = Location::new(doc.file().to_path_buf(), range);
                            result.add(Diagnostic::error(
                                "VAL-ATTR-002",
                                Category::Validation,
                                format!(
                                    "Invalid value '{}' for CustomAction.Execute",
                                    execute
                                ),
                                location,
                            ));
                        }
                    }
                }
                "Component" => {
                    // Validate GUID if present
                    if let Some(guid) = node.attribute("Guid") {
                        if !GUID_REGEX.is_match(guid) {
                            let range = doc.node_range(&node);
                            let location = Location::new(doc.file().to_path_buf(), range);
                            result.add(
                                Diagnostic::error(
                                    "VAL-ATTR-003",
                                    Category::Validation,
                                    "Invalid GUID format. Use '*' for auto-generation or a valid GUID",
                                    location,
                                ),
                            );
                        }
                    }
                }
                "RegistryKey" => {
                    if let Some(root) = node.attribute("Root") {
                        if !matches!(root, "HKMU" | "HKCR" | "HKCU" | "HKLM" | "HKU") {
                            let range = doc.node_range(&node);
                            let location = Location::new(doc.file().to_path_buf(), range);
                            result.add(Diagnostic::error(
                                "VAL-ATTR-002",
                                Category::Validation,
                                format!(
                                    "Invalid value '{}' for RegistryKey.Root. Valid values: HKMU, HKCR, HKCU, HKLM, HKU",
                                    root
                                ),
                                location,
                            ));
                        }
                    }
                }
                _ => {}
            }

            // Validate YesNo attributes
            for attr in node.attributes() {
                if matches!(
                    attr.name(),
                    "Vital" | "ReadOnly" | "Hidden" | "Secure" | "Transitive" | "Impersonate"
                ) {
                    if !matches!(attr.value(), "yes" | "no" | "true" | "false" | "1" | "0") {
                        let range = doc.node_range(&node);
                        let location = Location::new(doc.file().to_path_buf(), range);
                        result.add(Diagnostic::error(
                            "VAL-ATTR-004",
                            Category::Validation,
                            format!(
                                "Invalid yes/no value '{}' for {}.{}. Use 'yes' or 'no'",
                                attr.value(),
                                tag_name,
                                attr.name()
                            ),
                            location,
                        ));
                    }
                }
            }
        }
    }
}

/// Known parent-child relationships
fn get_valid_parents() -> HashMap<&'static str, HashSet<&'static str>> {
    let mut map = HashMap::new();

    map.insert(
        "File",
        ["Component"].iter().copied().collect(),
    );

    map.insert(
        "RegistryKey",
        ["Component", "RegistryKey"].iter().copied().collect(),
    );

    map.insert(
        "RegistryValue",
        ["Component", "RegistryKey", "RegistryValue"].iter().copied().collect(),
    );

    map.insert(
        "Component",
        ["Directory", "DirectoryRef", "StandardDirectory", "Fragment", "ComponentGroup", "Wix"]
            .iter()
            .copied()
            .collect(),
    );

    map.insert(
        "ComponentRef",
        ["Feature", "FeatureGroup", "FeatureRef", "Fragment", "Wix"]
            .iter()
            .copied()
            .collect(),
    );

    map.insert(
        "Feature",
        ["Package", "Feature", "Fragment", "Module", "Wix"]
            .iter()
            .copied()
            .collect(),
    );

    map.insert(
        "Directory",
        ["Directory", "DirectoryRef", "StandardDirectory", "Fragment", "Package", "Module", "Wix"]
            .iter()
            .copied()
            .collect(),
    );

    map.insert(
        "ServiceInstall",
        ["Component"].iter().copied().collect(),
    );

    map.insert(
        "ServiceControl",
        ["Component"].iter().copied().collect(),
    );

    map.insert(
        "Shortcut",
        ["Component", "File"].iter().copied().collect(),
    );

    map
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    fn analyze(source: &str) -> AnalysisResult {
        let doc = WixDocument::parse(source, Path::new("test.wxs")).unwrap();
        let mut index = SymbolIndex::new();
        index.index_source(source, Path::new("test.wxs")).unwrap();
        let analyzer = ValidationAnalyzer::new();
        analyzer.analyze(&doc, &index)
    }

    #[test]
    fn test_validation_analyzer_default() {
        let analyzer = ValidationAnalyzer::default();
        let doc = WixDocument::parse("<Wix />", Path::new("test.wxs")).unwrap();
        let index = SymbolIndex::new();
        let result = analyzer.analyze(&doc, &index);
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_invalid_reference() {
        let result = analyze(r#"<Wix><ComponentRef Id="Missing" /></Wix>"#);
        assert_eq!(result.error_count(), 1);
        assert!(result.diagnostics[0].message.contains("No Component"));
    }

    #[test]
    fn test_valid_reference() {
        let result = analyze(r#"<Wix>
            <Component Id="C1" />
            <Feature Id="F1"><ComponentRef Id="C1" /></Feature>
        </Wix>"#);

        // Should only have relationship warning (Component not in valid parent in test)
        let ref_errors: Vec<_> = result.diagnostics.iter()
            .filter(|d| d.rule_id == "VAL-REF-001")
            .collect();
        assert!(ref_errors.is_empty());
    }

    #[test]
    fn test_file_not_in_component() {
        let result = analyze(r#"<Wix><Directory Id="D1"><File Id="F1" /></Directory></Wix>"#);
        assert!(result.diagnostics.iter().any(|d| d.message.contains("cannot be a child of")));
    }

    #[test]
    fn test_invalid_guid() {
        let result = analyze(r#"<Wix><Component Guid="not-a-guid" /></Wix>"#);
        assert!(result.diagnostics.iter().any(|d| d.message.contains("Invalid GUID")));
    }

    #[test]
    fn test_valid_guid() {
        let result = analyze(r#"<Wix><Component Guid="*" /></Wix>"#);
        assert!(result.diagnostics.iter().all(|d| !d.message.contains("Invalid GUID")));
    }

    #[test]
    fn test_missing_required_id() {
        let result = analyze(r#"<Wix><Directory Name="Test" /></Wix>"#);
        assert!(result.diagnostics.iter().any(|d| d.message.contains("requires 'Id'")));
    }

    #[test]
    fn test_invalid_enum_value() {
        let result = analyze(r#"<Wix><Feature Id="F1" Display="invalid" /></Wix>"#);
        assert!(result.diagnostics.iter().any(|d| d.message.contains("Invalid value")));
    }

    #[test]
    fn test_feature_missing_id() {
        let result = analyze(r#"<Wix><Feature Title="Test" /></Wix>"#);
        assert!(result.diagnostics.iter().any(|d|
            d.message.contains("Feature requires 'Id'")
        ));
    }

    #[test]
    fn test_property_missing_id() {
        let result = analyze(r#"<Wix><Property Value="test" /></Wix>"#);
        assert!(result.diagnostics.iter().any(|d|
            d.message.contains("Property requires 'Id'")
        ));
    }

    #[test]
    fn test_custom_action_missing_id() {
        let result = analyze(r#"<Wix><CustomAction Script="vbscript" /></Wix>"#);
        assert!(result.diagnostics.iter().any(|d|
            d.message.contains("CustomAction requires 'Id'")
        ));
    }

    #[test]
    fn test_custom_action_invalid_execute() {
        let result = analyze(r#"<Wix><CustomAction Id="CA1" Execute="invalid" /></Wix>"#);
        assert!(result.diagnostics.iter().any(|d|
            d.message.contains("Invalid value 'invalid' for CustomAction.Execute")
        ));
    }

    #[test]
    fn test_custom_action_valid_execute() {
        let result = analyze(r#"<Wix><CustomAction Id="CA1" Execute="deferred" /></Wix>"#);
        assert!(result.diagnostics.iter().all(|d|
            !d.message.contains("CustomAction.Execute")
        ));
    }

    #[test]
    fn test_registry_key_invalid_root() {
        let result = analyze(r#"<Wix><Component Id="C1"><RegistryKey Root="INVALID" /></Component></Wix>"#);
        assert!(result.diagnostics.iter().any(|d|
            d.message.contains("Invalid value 'INVALID' for RegistryKey.Root")
        ));
    }

    #[test]
    fn test_registry_key_valid_root() {
        let result = analyze(r#"<Wix><Component Id="C1"><RegistryKey Root="HKLM" /></Component></Wix>"#);
        assert!(result.diagnostics.iter().all(|d|
            !d.message.contains("RegistryKey.Root")
        ));
    }

    #[test]
    fn test_invalid_yesno_attribute() {
        let result = analyze(r#"<Wix><File Id="F1" Vital="maybe" /></Wix>"#);
        assert!(result.diagnostics.iter().any(|d|
            d.message.contains("Invalid yes/no value 'maybe'")
        ));
    }

    #[test]
    fn test_valid_yesno_attribute() {
        let result = analyze(r#"<Wix><Component Id="C1"><File Id="F1" Vital="yes" /></Component></Wix>"#);
        assert!(result.diagnostics.iter().all(|d|
            !d.message.contains("Invalid yes/no")
        ));
    }
}
