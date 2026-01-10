//! Dead code analyzer - identifies unused elements

use crate::core::{
    AnalysisResult, Category, Diagnostic, Fix, FixAction, Location, SymbolIndex,
    WixDocument,
};
use std::collections::{HashMap, HashSet};
use super::Analyzer;

/// Dead code analyzer
pub struct DeadCodeAnalyzer;

impl DeadCodeAnalyzer {
    pub fn new() -> Self {
        Self
    }
}

impl Default for DeadCodeAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl Analyzer for DeadCodeAnalyzer {
    fn analyze(&self, doc: &WixDocument, index: &SymbolIndex) -> AnalysisResult {
        let mut result = AnalysisResult::new();

        // Check for properties never referenced
        self.check_unused_properties(doc, &mut result);

        // Check for CustomActions never scheduled
        self.check_unused_custom_actions(doc, &mut result);

        // Check for components not in any feature (cross-file aware)
        self.check_orphan_components(doc, index, &mut result);

        // Check for directories with no components
        self.check_empty_directories(doc, &mut result);

        // Check for features with no components
        self.check_empty_features(doc, &mut result);

        result
    }
}

impl DeadCodeAnalyzer {
    fn check_unused_properties(&self, doc: &WixDocument, result: &mut AnalysisResult) {
        let mut defined: HashMap<String, Location> = HashMap::new();
        let mut referenced: HashSet<String> = HashSet::new();

        // Collect property definitions
        for node in doc.root().descendants() {
            if node.tag_name().name() == "Property" {
                if let Some(id) = node.attribute("Id") {
                    let range = doc.node_range(&node);
                    let location = Location::new(doc.file().to_path_buf(), range);
                    defined.insert(id.to_string(), location);
                }
            }
        }

        // Scan for property references in attributes and conditions
        let _source = doc.source();
        for node in doc.root().descendants() {
            // Check all attributes for property references [PropertyName]
            for attr in node.attributes() {
                let value = attr.value();
                // Find [PropertyName] patterns
                let mut i = 0;
                while let Some(start) = value[i..].find('[') {
                    let start_idx = i + start;
                    if let Some(end) = value[start_idx..].find(']') {
                        let prop_name = &value[start_idx + 1..start_idx + end];
                        // Skip special prefixes like [#FileId], [!ComponentId], etc.
                        if !prop_name.starts_with('#')
                            && !prop_name.starts_with('!')
                            && !prop_name.starts_with('$')
                            && !prop_name.starts_with('%')
                            && !prop_name.is_empty()
                        {
                            referenced.insert(prop_name.to_string());
                        }
                        i = start_idx + end + 1;
                    } else {
                        break;
                    }
                }
            }

            // Check PropertyRef elements
            if node.tag_name().name() == "PropertyRef" {
                if let Some(id) = node.attribute("Id") {
                    referenced.insert(id.to_string());
                }
            }

            // Check SetProperty targets
            if node.tag_name().name() == "SetProperty" {
                if let Some(id) = node.attribute("Id") {
                    referenced.insert(id.to_string());
                }
            }
        }

        // Report unused properties
        for (id, location) in &defined {
            // Skip public properties (all uppercase) - they may be set by installer
            if id.chars().all(|c| c.is_uppercase() || c == '_') {
                continue;
            }

            // Skip common internal properties
            if id.starts_with("_") || id.starts_with("Wix") {
                continue;
            }

            if !referenced.contains(id) {
                result.add(
                    Diagnostic::warning(
                        "DEAD-001",
                        Category::DeadCode,
                        format!("Property '{}' is defined but never referenced", id),
                        location.clone(),
                    )
                    .with_fix(Fix::new(
                        "Remove unused property",
                        FixAction::RemoveElement { range: location.range },
                    )),
                );
            }
        }
    }

    fn check_unused_custom_actions(&self, doc: &WixDocument, result: &mut AnalysisResult) {
        let mut defined: HashMap<String, Location> = HashMap::new();
        let mut scheduled: HashSet<String> = HashSet::new();

        // Collect CustomAction definitions
        for node in doc.root().descendants() {
            if node.tag_name().name() == "CustomAction" {
                if let Some(id) = node.attribute("Id") {
                    let range = doc.node_range(&node);
                    let location = Location::new(doc.file().to_path_buf(), range);
                    defined.insert(id.to_string(), location);
                }
            }
        }

        // Find scheduled actions
        for node in doc.root().descendants() {
            let tag_name = node.tag_name().name();

            // Check Custom elements in sequences
            if tag_name == "Custom" {
                if let Some(action) = node.attribute("Action") {
                    scheduled.insert(action.to_string());
                }
            }

            // Check CustomActionRef
            if tag_name == "CustomActionRef" {
                if let Some(id) = node.attribute("Id") {
                    scheduled.insert(id.to_string());
                }
            }

            // Check Publish elements with action
            if tag_name == "Publish" {
                if let Some(event) = node.attribute("Event") {
                    if event == "DoAction" {
                        if let Some(value) = node.attribute("Value") {
                            scheduled.insert(value.to_string());
                        }
                    }
                }
            }
        }

        // Report unused CustomActions
        for (id, location) in &defined {
            if !scheduled.contains(id) {
                result.add(
                    Diagnostic::warning(
                        "DEAD-002",
                        Category::DeadCode,
                        format!("CustomAction '{}' is defined but never scheduled", id),
                        location.clone(),
                    )
                    .with_help("CustomActions must be scheduled in an InstallExecuteSequence or InstallUISequence"),
                );
            }
        }
    }

    fn check_orphan_components(&self, doc: &WixDocument, index: &SymbolIndex, result: &mut AnalysisResult) {
        // Get all component definitions from this file
        let mut components: HashMap<String, Location> = HashMap::new();

        for node in doc.root().descendants() {
            if matches!(node.tag_name().name(), "Component" | "ComponentGroup") {
                if let Some(id) = node.attribute("Id") {
                    let range = doc.node_range(&node);
                    let location = Location::new(doc.file().to_path_buf(), range);
                    components.insert(id.to_string(), location);
                }
            }
        }

        // Check if each component is referenced (using the cross-file index)
        for (id, location) in &components {
            // Check if there's a reference in the index
            let has_reference = index.all_references().iter().any(|r| r.id == *id);

            // Also check local references in this file
            let has_local_ref = doc.root().descendants().any(|node| {
                matches!(node.tag_name().name(), "ComponentRef" | "ComponentGroupRef")
                    && node.attribute("Id") == Some(id.as_str())
            });

            if !has_reference && !has_local_ref {
                result.add(
                    Diagnostic::error(
                        "DEAD-003",
                        Category::DeadCode,
                        format!("Component '{}' is not included in any Feature", id),
                        location.clone(),
                    )
                    .with_help("Components must be referenced by a Feature to be installed"),
                );
            }
        }
    }

    fn check_empty_directories(&self, doc: &WixDocument, result: &mut AnalysisResult) {
        for node in doc.root().descendants() {
            if node.tag_name().name() == "Directory" {
                if let Some(id) = node.attribute("Id") {
                    // Skip standard directories
                    if id == "TARGETDIR" || id.starts_with("INSTALL") || id.starts_with("Program") {
                        continue;
                    }

                    // Check if directory has any meaningful children
                    let has_component = node.children().any(|n| {
                        n.is_element()
                            && matches!(
                                n.tag_name().name(),
                                "Component" | "File" | "Directory"
                            )
                    });

                    if !has_component {
                        // Check if directory is referenced elsewhere
                        let is_referenced = doc.root().descendants().any(|n| {
                            n.tag_name().name() == "DirectoryRef"
                                && n.attribute("Id") == Some(id)
                        });

                        if !is_referenced {
                            let range = doc.node_range(&node);
                            let location = Location::new(doc.file().to_path_buf(), range);
                            result.add(Diagnostic::warning(
                                "DEAD-004",
                                Category::DeadCode,
                                format!("Directory '{}' is empty and not referenced", id),
                                location,
                            ));
                        }
                    }
                }
            }
        }
    }

    fn check_empty_features(&self, doc: &WixDocument, result: &mut AnalysisResult) {
        for node in doc.root().descendants() {
            if node.tag_name().name() == "Feature" {
                if let Some(id) = node.attribute("Id") {
                    // Check if feature has any component references or nested features
                    // Skip the feature node itself in descendants check
                    let has_content = node.children().any(|n| {
                        n.is_element()
                            && matches!(
                                n.tag_name().name(),
                                "ComponentRef" | "ComponentGroupRef" | "Feature" | "MergeRef"
                            )
                    });

                    if !has_content {
                        let range = doc.node_range(&node);
                        let location = Location::new(doc.file().to_path_buf(), range);
                        result.add(Diagnostic::warning(
                            "DEAD-005",
                            Category::DeadCode,
                            format!("Feature '{}' has no components", id),
                            location,
                        ));
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    fn analyze(source: &str) -> AnalysisResult {
        let doc = WixDocument::parse(source, Path::new("test.wxs")).unwrap();
        let mut index = SymbolIndex::new();
        index.index_source(source, Path::new("test.wxs")).unwrap();
        let analyzer = DeadCodeAnalyzer::new();
        analyzer.analyze(&doc, &index)
    }

    #[test]
    fn test_default_impl() {
        let analyzer = DeadCodeAnalyzer::default();
        let doc = WixDocument::parse("<Wix />", Path::new("test.wxs")).unwrap();
        let index = SymbolIndex::new();
        let result = analyzer.analyze(&doc, &index);
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_unused_property() {
        let result = analyze(r#"<Wix><Property Id="myUnusedProp" Value="test" /></Wix>"#);
        assert!(result.diagnostics.iter().any(|d| d.rule_id == "DEAD-001"));
    }

    #[test]
    fn test_used_property() {
        let result = analyze(r#"<Wix>
            <Property Id="myProp" Value="test" />
            <Component Id="C1" Condition="[myProp]" />
        </Wix>"#);
        assert!(result.diagnostics.iter().all(|d| d.rule_id != "DEAD-001" || !d.message.contains("myProp")));
    }

    #[test]
    fn test_property_ref_used() {
        let result = analyze(r#"<Wix>
            <Property Id="myProp" Value="test" />
            <PropertyRef Id="myProp" />
        </Wix>"#);
        assert!(result.diagnostics.iter().all(|d| d.rule_id != "DEAD-001" || !d.message.contains("myProp")));
    }

    #[test]
    fn test_set_property_used() {
        let result = analyze(r#"<Wix>
            <Property Id="myProp" Value="test" />
            <SetProperty Id="myProp" Value="new" />
        </Wix>"#);
        assert!(result.diagnostics.iter().all(|d| d.rule_id != "DEAD-001" || !d.message.contains("myProp")));
    }

    #[test]
    fn test_property_with_underscore_prefix_ok() {
        // Properties starting with _ are internal
        let result = analyze(r#"<Wix><Property Id="_internalProp" Value="test" /></Wix>"#);
        assert!(result.diagnostics.iter().all(|d| d.rule_id != "DEAD-001"));
    }

    #[test]
    fn test_property_with_wix_prefix_ok() {
        // Properties starting with Wix are internal
        let result = analyze(r#"<Wix><Property Id="WixUIBannerBmp" Value="test" /></Wix>"#);
        assert!(result.diagnostics.iter().all(|d| d.rule_id != "DEAD-001"));
    }

    #[test]
    fn test_public_property_not_flagged() {
        let result = analyze(r#"<Wix><Property Id="MY_PUBLIC_PROP" Value="test" /></Wix>"#);
        assert!(result.diagnostics.iter().all(|d| d.rule_id != "DEAD-001"));
    }

    #[test]
    fn test_property_multiple_refs_in_value() {
        let result = analyze(r#"<Wix>
            <Property Id="propA" Value="a" />
            <Property Id="propB" Value="b" />
            <Property Id="combined" Value="[propA][propB]" />
        </Wix>"#);
        // propA and propB are referenced, combined is not
        assert!(result.diagnostics.iter().all(|d| d.rule_id != "DEAD-001" || !d.message.contains("propA")));
        assert!(result.diagnostics.iter().all(|d| d.rule_id != "DEAD-001" || !d.message.contains("propB")));
    }

    #[test]
    fn test_property_special_prefixes_ignored() {
        // [#FileId], [!ComponentId], [$ComponentId], [%EnvVar] should not be treated as property refs
        let result = analyze(r#"<Wix>
            <Property Id="fileRef" Value="test" />
            <Property Id="path" Value="[#FileId]" />
        </Wix>"#);
        // fileRef should still be flagged as unused
        assert!(result.diagnostics.iter().any(|d| d.rule_id == "DEAD-001" && d.message.contains("fileRef")));
    }

    #[test]
    fn test_unused_custom_action() {
        let result = analyze(r#"<Wix><CustomAction Id="CA1" Script="vbscript" /></Wix>"#);
        assert!(result.diagnostics.iter().any(|d| d.rule_id == "DEAD-002"));
    }

    #[test]
    fn test_scheduled_custom_action() {
        let result = analyze(r#"<Wix>
            <CustomAction Id="CA1" Script="vbscript" />
            <InstallExecuteSequence>
                <Custom Action="CA1" After="InstallFiles" />
            </InstallExecuteSequence>
        </Wix>"#);
        assert!(result.diagnostics.iter().all(|d| d.rule_id != "DEAD-002"));
    }

    #[test]
    fn test_custom_action_ref_scheduled() {
        let result = analyze(r#"<Wix>
            <CustomAction Id="CA1" Script="vbscript" />
            <CustomActionRef Id="CA1" />
        </Wix>"#);
        assert!(result.diagnostics.iter().all(|d| d.rule_id != "DEAD-002"));
    }

    #[test]
    fn test_custom_action_publish_doaction() {
        let result = analyze(r#"<Wix>
            <CustomAction Id="CA1" Script="vbscript" />
            <Publish Event="DoAction" Value="CA1" />
        </Wix>"#);
        assert!(result.diagnostics.iter().all(|d| d.rule_id != "DEAD-002"));
    }

    #[test]
    fn test_orphan_component() {
        let result = analyze(r#"<Wix><Component Id="OrphanComp" /></Wix>"#);
        assert!(result.diagnostics.iter().any(|d| d.rule_id == "DEAD-003"));
    }

    #[test]
    fn test_orphan_component_group() {
        let result = analyze(r#"<Wix><ComponentGroup Id="OrphanGroup" /></Wix>"#);
        assert!(result.diagnostics.iter().any(|d| d.rule_id == "DEAD-003"));
    }

    #[test]
    fn test_referenced_component() {
        let result = analyze(r#"<Wix>
            <Component Id="C1" />
            <Feature Id="F1"><ComponentRef Id="C1" /></Feature>
        </Wix>"#);
        assert!(result.diagnostics.iter().all(|d| d.rule_id != "DEAD-003" || !d.message.contains("C1")));
    }

    #[test]
    fn test_referenced_component_group() {
        let result = analyze(r#"<Wix>
            <ComponentGroup Id="CG1" />
            <Feature Id="F1"><ComponentGroupRef Id="CG1" /></Feature>
        </Wix>"#);
        assert!(result.diagnostics.iter().all(|d| d.rule_id != "DEAD-003" || !d.message.contains("CG1")));
    }

    #[test]
    fn test_empty_directory() {
        let result = analyze(r#"<Wix><Directory Id="MyEmptyDir" /></Wix>"#);
        assert!(result.diagnostics.iter().any(|d| d.rule_id == "DEAD-004"));
    }

    #[test]
    fn test_directory_with_component() {
        let result = analyze(r#"<Wix>
            <Directory Id="MyDir">
                <Component Id="C1" />
            </Directory>
        </Wix>"#);
        assert!(result.diagnostics.iter().all(|d| d.rule_id != "DEAD-004"));
    }

    #[test]
    fn test_directory_with_file() {
        let result = analyze(r#"<Wix>
            <Directory Id="MyDir">
                <File Id="F1" Source="test.txt" />
            </Directory>
        </Wix>"#);
        assert!(result.diagnostics.iter().all(|d| d.rule_id != "DEAD-004"));
    }

    #[test]
    fn test_directory_with_subdirectory() {
        let result = analyze(r#"<Wix>
            <Directory Id="MyDir">
                <Directory Id="SubDir" />
            </Directory>
        </Wix>"#);
        // MyDir is not empty because it has SubDir
        assert!(result.diagnostics.iter().all(|d| d.rule_id != "DEAD-004" || !d.message.contains("MyDir")));
    }

    #[test]
    fn test_empty_directory_referenced() {
        let result = analyze(r#"<Wix>
            <Directory Id="MyDir" />
            <DirectoryRef Id="MyDir" />
        </Wix>"#);
        assert!(result.diagnostics.iter().all(|d| d.rule_id != "DEAD-004"));
    }

    #[test]
    fn test_standard_directory_targetdir_ok() {
        let result = analyze(r#"<Wix><Directory Id="TARGETDIR" /></Wix>"#);
        assert!(result.diagnostics.iter().all(|d| d.rule_id != "DEAD-004"));
    }

    #[test]
    fn test_standard_directory_installdir_ok() {
        let result = analyze(r#"<Wix><Directory Id="INSTALLDIR" /></Wix>"#);
        assert!(result.diagnostics.iter().all(|d| d.rule_id != "DEAD-004"));
    }

    #[test]
    fn test_standard_directory_program_ok() {
        let result = analyze(r#"<Wix><Directory Id="ProgramFilesFolder" /></Wix>"#);
        assert!(result.diagnostics.iter().all(|d| d.rule_id != "DEAD-004"));
    }

    #[test]
    fn test_empty_feature() {
        let result = analyze(r#"<Wix><Feature Id="EmptyFeature" /></Wix>"#);
        assert!(result.diagnostics.iter().any(|d| d.rule_id == "DEAD-005"));
    }

    #[test]
    fn test_feature_with_components() {
        let result = analyze(r#"<Wix>
            <Feature Id="F1"><ComponentRef Id="C1" /></Feature>
        </Wix>"#);
        assert!(result.diagnostics.iter().all(|d| d.rule_id != "DEAD-005"));
    }

    #[test]
    fn test_feature_with_component_group_ref() {
        let result = analyze(r#"<Wix>
            <Feature Id="F1"><ComponentGroupRef Id="CG1" /></Feature>
        </Wix>"#);
        assert!(result.diagnostics.iter().all(|d| d.rule_id != "DEAD-005"));
    }

    #[test]
    fn test_feature_with_nested_feature() {
        let result = analyze(r#"<Wix>
            <Feature Id="F1">
                <Feature Id="F2" />
            </Feature>
        </Wix>"#);
        // F1 is not empty because it has F2; F2 is empty
        assert!(result.diagnostics.iter().all(|d| d.rule_id != "DEAD-005" || !d.message.contains("F1")));
        assert!(result.diagnostics.iter().any(|d| d.rule_id == "DEAD-005" && d.message.contains("F2")));
    }

    #[test]
    fn test_feature_with_merge_ref() {
        let result = analyze(r#"<Wix>
            <Feature Id="F1"><MergeRef Id="M1" /></Feature>
        </Wix>"#);
        assert!(result.diagnostics.iter().all(|d| d.rule_id != "DEAD-005"));
    }

    #[test]
    fn test_property_no_id() {
        // Property without Id should not crash
        let result = analyze(r#"<Wix><Property Value="test" /></Wix>"#);
        assert!(result.diagnostics.iter().all(|d| d.rule_id != "DEAD-001"));
    }

    #[test]
    fn test_custom_action_no_id() {
        // CustomAction without Id should not crash
        let result = analyze(r#"<Wix><CustomAction Script="vbscript" /></Wix>"#);
        assert!(result.diagnostics.iter().all(|d| d.rule_id != "DEAD-002"));
    }

    #[test]
    fn test_unclosed_bracket_in_attribute() {
        // Edge case: unclosed bracket
        let result = analyze(r#"<Wix>
            <Property Id="myProp" Value="test" />
            <Component Id="C1" Condition="[myProp" />
        </Wix>"#);
        // Should not crash, unclosed bracket is not a valid reference
        assert!(result.diagnostics.iter().any(|d| d.rule_id == "DEAD-001"));
    }
}
