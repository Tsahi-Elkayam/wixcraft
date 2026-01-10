//! Best practices analyzer - efficiency, idioms, performance, maintainability

use crate::core::{
    AnalysisResult, Category, Diagnostic, Fix, FixAction, InsertPosition, Location,
    SymbolIndex, WixDocument,
};
use regex::Regex;
use roxmltree::Node;
use std::collections::{HashMap, HashSet};
use std::sync::LazyLock;
use super::Analyzer;

/// GUID pattern
static GUID_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^\{?[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}\}?$").unwrap()
});

/// Windows absolute path pattern
static WINDOWS_ABSOLUTE_PATH: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^[A-Za-z]:\\").unwrap()
});

/// Best practices analyzer
pub struct BestPracticesAnalyzer {
    max_files_per_component: usize,
    max_directory_depth: usize,
}

impl BestPracticesAnalyzer {
    pub fn new() -> Self {
        Self {
            max_files_per_component: 1,
            max_directory_depth: 10,
        }
    }

    pub fn with_thresholds(max_files: usize, max_depth: usize) -> Self {
        Self {
            max_files_per_component: max_files,
            max_directory_depth: max_depth,
        }
    }
}

impl Default for BestPracticesAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl Analyzer for BestPracticesAnalyzer {
    fn analyze(&self, doc: &WixDocument, _index: &SymbolIndex) -> AnalysisResult {
        let mut result = AnalysisResult::new();

        // Efficiency checks
        self.check_duplicate_components(doc, &mut result);
        self.check_unused_components(doc, &mut result);
        self.check_duplicate_properties(doc, &mut result);

        // Idiom checks
        self.check_major_upgrade(doc, &mut result);
        self.check_hardcoded_guids(doc, &mut result);
        self.check_deprecated_elements(doc, &mut result);
        self.check_upgrade_code(doc, &mut result);

        // Performance checks
        self.check_multi_file_components(doc, &mut result);
        self.check_directory_depth(doc, &mut result);

        // Maintainability checks
        self.check_hardcoded_paths(doc, &mut result);
        self.check_naming_conventions(doc, &mut result);

        result
    }
}

impl BestPracticesAnalyzer {
    // === Efficiency Checks ===

    fn check_duplicate_components(&self, doc: &WixDocument, result: &mut AnalysisResult) {
        let mut components: HashMap<String, Vec<Location>> = HashMap::new();

        for node in doc.root().descendants() {
            if matches!(node.tag_name().name(), "Component" | "ComponentGroup") {
                if let Some(id) = node.attribute("Id") {
                    let range = doc.node_range(&node);
                    let location = Location::new(doc.file().to_path_buf(), range);
                    components.entry(id.to_string()).or_default().push(location);
                }
            }
        }

        for (id, locations) in &components {
            if locations.len() > 1 {
                for loc in locations.iter().skip(1) {
                    result.add(Diagnostic::warning(
                        "BP-EFF-001",
                        Category::BestPractice,
                        format!("Component Id '{}' is defined multiple times", id),
                        loc.clone(),
                    ));
                }
            }
        }
    }

    fn check_unused_components(&self, doc: &WixDocument, result: &mut AnalysisResult) {
        let mut components: HashMap<String, Location> = HashMap::new();
        let mut referenced: HashSet<String> = HashSet::new();

        for node in doc.root().descendants() {
            let tag_name = node.tag_name().name();

            if matches!(tag_name, "Component" | "ComponentGroup") {
                if let Some(id) = node.attribute("Id") {
                    let range = doc.node_range(&node);
                    let location = Location::new(doc.file().to_path_buf(), range);
                    components.insert(id.to_string(), location);
                }
            }

            if matches!(tag_name, "ComponentRef" | "ComponentGroupRef") {
                if let Some(id) = node.attribute("Id") {
                    referenced.insert(id.to_string());
                }
            }
        }

        for (id, location) in &components {
            if !referenced.contains(id) {
                result.add(
                    Diagnostic::warning(
                        "BP-EFF-002",
                        Category::BestPractice,
                        format!("Component '{}' is not referenced by any Feature", id),
                        location.clone(),
                    )
                    .with_help("Unreferenced components will not be installed"),
                );
            }
        }
    }

    fn check_duplicate_properties(&self, doc: &WixDocument, result: &mut AnalysisResult) {
        let mut properties: HashMap<String, Vec<Location>> = HashMap::new();

        for node in doc.root().descendants() {
            if node.tag_name().name() == "Property" {
                if let Some(id) = node.attribute("Id") {
                    let range = doc.node_range(&node);
                    let location = Location::new(doc.file().to_path_buf(), range);
                    properties.entry(id.to_string()).or_default().push(location);
                }
            }
        }

        for (id, locations) in &properties {
            if locations.len() > 1 {
                for loc in locations.iter().skip(1) {
                    result.add(Diagnostic::warning(
                        "BP-EFF-003",
                        Category::BestPractice,
                        format!("Property '{}' is defined multiple times", id),
                        loc.clone(),
                    ));
                }
            }
        }
    }

    // === Idiom Checks ===

    fn check_major_upgrade(&self, doc: &WixDocument, result: &mut AnalysisResult) {
        for node in doc.root().descendants() {
            if node.tag_name().name() == "Package" {
                let has_major_upgrade = node.children().any(|child| {
                    child.is_element() && child.tag_name().name() == "MajorUpgrade"
                });

                if !has_major_upgrade {
                    let range = doc.node_range(&node);
                    let location = Location::new(doc.file().to_path_buf(), range);
                    result.add(
                        Diagnostic::error(
                            "BP-IDIOM-001",
                            Category::BestPractice,
                            "Package should include a MajorUpgrade element for proper upgrade handling",
                            location.clone(),
                        )
                        .with_help("Without MajorUpgrade, users cannot upgrade without first uninstalling")
                        .with_fix(Fix::new(
                            "Add MajorUpgrade element",
                            FixAction::AddElement {
                                parent_range: range,
                                element: r#"<MajorUpgrade DowngradeErrorMessage="A newer version is already installed." />"#.to_string(),
                                position: InsertPosition::First,
                            },
                        )),
                    );
                }
            }
        }
    }

    fn check_hardcoded_guids(&self, doc: &WixDocument, result: &mut AnalysisResult) {
        for node in doc.root().descendants() {
            if node.tag_name().name() == "Component" {
                if let Some(guid) = node.attribute("Guid") {
                    if guid != "*" && GUID_REGEX.is_match(guid) {
                        let range = doc.node_range(&node);
                        let location = Location::new(doc.file().to_path_buf(), range);
                        result.add(
                            Diagnostic::warning(
                                "BP-IDIOM-002",
                                Category::BestPractice,
                                format!("Component uses hardcoded GUID '{}'. Consider using Guid=\"*\"", guid),
                                location.clone(),
                            )
                            .with_fix(Fix::new(
                                "Use auto-generated GUID",
                                FixAction::ReplaceAttribute {
                                    range,
                                    name: "Guid".to_string(),
                                    new_value: "*".to_string(),
                                },
                            )),
                        );
                    }
                }
            }
        }
    }

    fn check_deprecated_elements(&self, doc: &WixDocument, result: &mut AnalysisResult) {
        for node in doc.root().descendants() {
            if node.tag_name().name() == "Product" {
                let range = doc.node_range(&node);
                let location = Location::new(doc.file().to_path_buf(), range);
                result.add(
                    Diagnostic::warning(
                        "BP-IDIOM-003",
                        Category::BestPractice,
                        "Product element is deprecated in WiX v4. Use Package instead",
                        location,
                    ),
                );
            }
        }
    }

    fn check_upgrade_code(&self, doc: &WixDocument, result: &mut AnalysisResult) {
        for node in doc.root().descendants() {
            if node.tag_name().name() == "Package" {
                if node.attribute("UpgradeCode").is_none() {
                    let range = doc.node_range(&node);
                    let location = Location::new(doc.file().to_path_buf(), range);
                    result.add(
                        Diagnostic::error(
                            "BP-IDIOM-004",
                            Category::BestPractice,
                            "Package should have an UpgradeCode attribute for upgrade support",
                            location,
                        )
                        .with_help("Generate a GUID and use it consistently across versions"),
                    );
                }
            }
        }
    }

    // === Performance Checks ===

    fn check_multi_file_components(&self, doc: &WixDocument, result: &mut AnalysisResult) {
        for node in doc.root().descendants() {
            if node.tag_name().name() == "Component" {
                let file_count = node
                    .children()
                    .filter(|child| child.is_element() && child.tag_name().name() == "File")
                    .count();

                if file_count > self.max_files_per_component {
                    let id = node.attribute("Id").unwrap_or("unknown");
                    let range = doc.node_range(&node);
                    let location = Location::new(doc.file().to_path_buf(), range);
                    result.add(
                        Diagnostic::warning(
                            "BP-PERF-001",
                            Category::BestPractice,
                            format!(
                                "Component '{}' contains {} files. Consider one file per component",
                                id, file_count
                            ),
                            location,
                        )
                        .with_help("Multiple files in a component share the same install state"),
                    );
                }
            }
        }
    }

    fn check_directory_depth(&self, doc: &WixDocument, result: &mut AnalysisResult) {
        self.check_depth_recursive(doc.root(), doc, 0, result);
    }

    fn check_depth_recursive(&self, node: Node, doc: &WixDocument, depth: usize, result: &mut AnalysisResult) {
        let new_depth = if node.is_element()
            && matches!(node.tag_name().name(), "Directory" | "StandardDirectory")
        {
            depth + 1
        } else {
            depth
        };

        if new_depth > self.max_directory_depth
            && node.is_element()
            && node.tag_name().name() == "Directory"
        {
            let id = node.attribute("Id").unwrap_or("unknown");
            let range = doc.node_range(&node);
            let location = Location::new(doc.file().to_path_buf(), range);
            result.add(Diagnostic::info(
                "BP-PERF-002",
                Category::BestPractice,
                format!(
                    "Directory '{}' is nested {} levels deep (max recommended: {})",
                    id, new_depth, self.max_directory_depth
                ),
                location,
            ));
        }

        for child in node.children() {
            self.check_depth_recursive(child, doc, new_depth, result);
        }
    }

    // === Maintainability Checks ===

    fn check_hardcoded_paths(&self, doc: &WixDocument, result: &mut AnalysisResult) {
        for node in doc.root().descendants() {
            if node.tag_name().name() == "File" {
                if let Some(source) = node.attribute("Source") {
                    if WINDOWS_ABSOLUTE_PATH.is_match(source) {
                        let range = doc.node_range(&node);
                        let location = Location::new(doc.file().to_path_buf(), range);
                        result.add(
                            Diagnostic::warning(
                                "BP-MAINT-001",
                                Category::BestPractice,
                                format!("File Source uses absolute path '{}'. Use relative paths or $(var.SourceDir)", source),
                                location,
                            ),
                        );
                    }
                }
            }
        }
    }

    fn check_naming_conventions(&self, doc: &WixDocument, result: &mut AnalysisResult) {
        for node in doc.root().descendants() {
            let tag_name = node.tag_name().name();

            if let Some(id) = node.attribute("Id") {
                if id == "*" || id.starts_with("!(") {
                    continue;
                }

                let warning = match tag_name {
                    "Component" if !id.starts_with("C_") && !id.starts_with("cmp") => {
                        Some("Consider prefixing Component IDs with 'C_' or 'cmp'")
                    }
                    "Directory" if !id.starts_with("D_") && !id.starts_with("dir") && id != "TARGETDIR" && !id.starts_with("INSTALL") => {
                        Some("Consider prefixing Directory IDs with 'D_' or 'dir'")
                    }
                    "Feature" if !id.starts_with("F_") && !id.starts_with("feat") => {
                        Some("Consider prefixing Feature IDs with 'F_' or 'feat'")
                    }
                    "Property" if id.chars().any(|c| c.is_lowercase()) && !id.starts_with("_") => {
                        Some("Public properties should be ALL_UPPERCASE")
                    }
                    _ => None,
                };

                if let Some(msg) = warning {
                    let range = doc.node_range(&node);
                    let location = Location::new(doc.file().to_path_buf(), range);
                    result.add(Diagnostic::info(
                        "BP-MAINT-002",
                        Category::BestPractice,
                        format!("{} Id '{}': {}", tag_name, id, msg),
                        location,
                    ));
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
        let index = SymbolIndex::new();
        let analyzer = BestPracticesAnalyzer::new();
        analyzer.analyze(&doc, &index)
    }

    fn analyze_with_thresholds(source: &str, max_files: usize, max_depth: usize) -> AnalysisResult {
        let doc = WixDocument::parse(source, Path::new("test.wxs")).unwrap();
        let index = SymbolIndex::new();
        let analyzer = BestPracticesAnalyzer::with_thresholds(max_files, max_depth);
        analyzer.analyze(&doc, &index)
    }

    #[test]
    fn test_default_impl() {
        let analyzer = BestPracticesAnalyzer::default();
        let doc = WixDocument::parse("<Wix />", Path::new("test.wxs")).unwrap();
        let index = SymbolIndex::new();
        let result = analyzer.analyze(&doc, &index);
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_with_thresholds() {
        // 3 files per component allowed, no warning
        let result = analyze_with_thresholds(
            r#"<Wix><Component Id="C1"><File Id="F1" /><File Id="F2" /><File Id="F3" /></Component></Wix>"#,
            3, 10
        );
        assert!(result.diagnostics.iter().all(|d| d.rule_id != "BP-PERF-001"));
    }

    // === Efficiency Tests ===

    #[test]
    fn test_duplicate_component() {
        let result = analyze(r#"<Wix>
            <Component Id="C1" />
            <Component Id="C1" />
        </Wix>"#);
        assert!(result.diagnostics.iter().any(|d| d.rule_id == "BP-EFF-001"));
    }

    #[test]
    fn test_duplicate_component_group() {
        let result = analyze(r#"<Wix>
            <ComponentGroup Id="CG1" />
            <ComponentGroup Id="CG1" />
        </Wix>"#);
        assert!(result.diagnostics.iter().any(|d| d.rule_id == "BP-EFF-001"));
    }

    #[test]
    fn test_no_duplicate_component() {
        let result = analyze(r#"<Wix>
            <Component Id="C1" />
            <Component Id="C2" />
        </Wix>"#);
        assert!(result.diagnostics.iter().all(|d| d.rule_id != "BP-EFF-001"));
    }

    #[test]
    fn test_unused_component() {
        let result = analyze(r#"<Wix><Component Id="UnusedComp" /></Wix>"#);
        assert!(result.diagnostics.iter().any(|d| d.rule_id == "BP-EFF-002"));
    }

    #[test]
    fn test_used_component() {
        let result = analyze(r#"<Wix>
            <Component Id="C1" />
            <Feature Id="F1"><ComponentRef Id="C1" /></Feature>
        </Wix>"#);
        assert!(result.diagnostics.iter().all(|d| d.rule_id != "BP-EFF-002" || !d.message.contains("C1")));
    }

    #[test]
    fn test_used_component_group() {
        let result = analyze(r#"<Wix>
            <ComponentGroup Id="CG1" />
            <Feature Id="F1"><ComponentGroupRef Id="CG1" /></Feature>
        </Wix>"#);
        assert!(result.diagnostics.iter().all(|d| d.rule_id != "BP-EFF-002" || !d.message.contains("CG1")));
    }

    #[test]
    fn test_duplicate_property() {
        let result = analyze(r#"<Wix>
            <Property Id="P1" Value="a" />
            <Property Id="P1" Value="b" />
        </Wix>"#);
        assert!(result.diagnostics.iter().any(|d| d.rule_id == "BP-EFF-003"));
    }

    #[test]
    fn test_no_duplicate_property() {
        let result = analyze(r#"<Wix>
            <Property Id="P1" Value="a" />
            <Property Id="P2" Value="b" />
        </Wix>"#);
        assert!(result.diagnostics.iter().all(|d| d.rule_id != "BP-EFF-003"));
    }

    // === Idiom Tests ===

    #[test]
    fn test_missing_major_upgrade() {
        let result = analyze(r#"<Wix><Package Name="Test" Version="1.0" /></Wix>"#);
        assert!(result.diagnostics.iter().any(|d| d.rule_id == "BP-IDIOM-001"));
    }

    #[test]
    fn test_has_major_upgrade() {
        let result = analyze(r#"<Wix><Package Name="Test"><MajorUpgrade /></Package></Wix>"#);
        assert!(result.diagnostics.iter().all(|d| d.rule_id != "BP-IDIOM-001"));
    }

    #[test]
    fn test_hardcoded_guid() {
        let result = analyze(r#"<Wix><Component Id="C1" Guid="{12345678-1234-1234-1234-123456789ABC}" /></Wix>"#);
        assert!(result.diagnostics.iter().any(|d| d.rule_id == "BP-IDIOM-002"));
    }

    #[test]
    fn test_auto_guid_ok() {
        let result = analyze(r#"<Wix><Component Id="C1" Guid="*" /></Wix>"#);
        assert!(result.diagnostics.iter().all(|d| d.rule_id != "BP-IDIOM-002"));
    }

    #[test]
    fn test_guid_without_braces() {
        let result = analyze(r#"<Wix><Component Id="C1" Guid="12345678-1234-1234-1234-123456789ABC" /></Wix>"#);
        assert!(result.diagnostics.iter().any(|d| d.rule_id == "BP-IDIOM-002"));
    }

    #[test]
    fn test_deprecated_product() {
        let result = analyze(r#"<Wix><Product Id="*" Name="Test" /></Wix>"#);
        assert!(result.diagnostics.iter().any(|d| d.rule_id == "BP-IDIOM-003"));
    }

    #[test]
    fn test_missing_upgrade_code() {
        let result = analyze(r#"<Wix><Package Name="Test" /></Wix>"#);
        assert!(result.diagnostics.iter().any(|d| d.rule_id == "BP-IDIOM-004"));
    }

    #[test]
    fn test_has_upgrade_code() {
        let result = analyze(r#"<Wix><Package Name="Test" UpgradeCode="{12345678-1234-1234-1234-123456789ABC}" /></Wix>"#);
        assert!(result.diagnostics.iter().all(|d| d.rule_id != "BP-IDIOM-004"));
    }

    // === Performance Tests ===

    #[test]
    fn test_multi_file_component() {
        let result = analyze(r#"<Wix><Component Id="C1"><File Id="F1" /><File Id="F2" /></Component></Wix>"#);
        assert!(result.diagnostics.iter().any(|d| d.rule_id == "BP-PERF-001"));
    }

    #[test]
    fn test_single_file_component_ok() {
        let result = analyze(r#"<Wix><Component Id="C1"><File Id="F1" /></Component></Wix>"#);
        assert!(result.diagnostics.iter().all(|d| d.rule_id != "BP-PERF-001"));
    }

    #[test]
    fn test_deep_directory_nesting() {
        // Create deeply nested directories (11 levels)
        let result = analyze_with_thresholds(
            r#"<Wix>
                <Directory Id="D1">
                    <Directory Id="D2">
                        <Directory Id="D3">
                            <Directory Id="D4">
                                <Directory Id="D5">
                                    <Directory Id="D6">
                                        <Directory Id="D7">
                                            <Directory Id="D8">
                                                <Directory Id="D9">
                                                    <Directory Id="D10">
                                                        <Directory Id="D11" />
                                                    </Directory>
                                                </Directory>
                                            </Directory>
                                        </Directory>
                                    </Directory>
                                </Directory>
                            </Directory>
                        </Directory>
                    </Directory>
                </Directory>
            </Wix>"#,
            1, 10
        );
        assert!(result.diagnostics.iter().any(|d| d.rule_id == "BP-PERF-002"));
    }

    #[test]
    fn test_standard_directory_counts_for_depth() {
        // StandardDirectory should count toward depth
        let result = analyze_with_thresholds(
            r#"<Wix>
                <StandardDirectory Id="ProgramFilesFolder">
                    <Directory Id="D1">
                        <Directory Id="D2" />
                    </Directory>
                </StandardDirectory>
            </Wix>"#,
            1, 2
        );
        assert!(result.diagnostics.iter().any(|d| d.rule_id == "BP-PERF-002"));
    }

    #[test]
    fn test_acceptable_depth_ok() {
        let result = analyze_with_thresholds(
            r#"<Wix>
                <Directory Id="D1">
                    <Directory Id="D2" />
                </Directory>
            </Wix>"#,
            1, 10
        );
        assert!(result.diagnostics.iter().all(|d| d.rule_id != "BP-PERF-002"));
    }

    // === Maintainability Tests ===

    #[test]
    fn test_hardcoded_path() {
        let result = analyze(r#"<Wix><File Id="F1" Source="C:\Build\app.exe" /></Wix>"#);
        assert!(result.diagnostics.iter().any(|d| d.rule_id == "BP-MAINT-001"));
    }

    #[test]
    fn test_relative_path_ok() {
        let result = analyze(r#"<Wix><File Id="F1" Source="$(var.SourceDir)\app.exe" /></Wix>"#);
        assert!(result.diagnostics.iter().all(|d| d.rule_id != "BP-MAINT-001"));
    }

    #[test]
    fn test_lowercase_drive_path() {
        let result = analyze(r#"<Wix><File Id="F1" Source="d:\projects\app.exe" /></Wix>"#);
        assert!(result.diagnostics.iter().any(|d| d.rule_id == "BP-MAINT-001"));
    }

    #[test]
    fn test_component_naming_convention() {
        let result = analyze(r#"<Wix><Component Id="BadName" /></Wix>"#);
        assert!(result.diagnostics.iter().any(|d| d.rule_id == "BP-MAINT-002" && d.message.contains("Component")));
    }

    #[test]
    fn test_component_cmp_prefix_ok() {
        let result = analyze(r#"<Wix><Component Id="cmpMyComponent" /></Wix>"#);
        assert!(result.diagnostics.iter().all(|d| d.rule_id != "BP-MAINT-002" || !d.message.contains("Component")));
    }

    #[test]
    fn test_component_c_prefix_ok() {
        let result = analyze(r#"<Wix><Component Id="C_MyComponent" /></Wix>"#);
        assert!(result.diagnostics.iter().all(|d| d.rule_id != "BP-MAINT-002" || !d.message.contains("Component")));
    }

    #[test]
    fn test_directory_naming_convention() {
        let result = analyze(r#"<Wix><Directory Id="BadDirName" /></Wix>"#);
        assert!(result.diagnostics.iter().any(|d| d.rule_id == "BP-MAINT-002" && d.message.contains("Directory")));
    }

    #[test]
    fn test_directory_dir_prefix_ok() {
        let result = analyze(r#"<Wix><Directory Id="dirMyDir" /></Wix>"#);
        assert!(result.diagnostics.iter().all(|d| d.rule_id != "BP-MAINT-002" || !d.message.contains("Directory")));
    }

    #[test]
    fn test_directory_targetdir_ok() {
        let result = analyze(r#"<Wix><Directory Id="TARGETDIR" /></Wix>"#);
        assert!(result.diagnostics.iter().all(|d| d.rule_id != "BP-MAINT-002" || !d.message.contains("Directory")));
    }

    #[test]
    fn test_directory_installdir_ok() {
        let result = analyze(r#"<Wix><Directory Id="INSTALLDIR" /></Wix>"#);
        assert!(result.diagnostics.iter().all(|d| d.rule_id != "BP-MAINT-002" || !d.message.contains("Directory")));
    }

    #[test]
    fn test_feature_naming_convention() {
        let result = analyze(r#"<Wix><Feature Id="BadFeatureName" /></Wix>"#);
        assert!(result.diagnostics.iter().any(|d| d.rule_id == "BP-MAINT-002" && d.message.contains("Feature")));
    }

    #[test]
    fn test_feature_feat_prefix_ok() {
        let result = analyze(r#"<Wix><Feature Id="featMain" /></Wix>"#);
        assert!(result.diagnostics.iter().all(|d| d.rule_id != "BP-MAINT-002" || !d.message.contains("Feature")));
    }

    #[test]
    fn test_feature_f_prefix_ok() {
        let result = analyze(r#"<Wix><Feature Id="F_Main" /></Wix>"#);
        assert!(result.diagnostics.iter().all(|d| d.rule_id != "BP-MAINT-002" || !d.message.contains("Feature")));
    }

    #[test]
    fn test_property_lowercase_warning() {
        let result = analyze(r#"<Wix><Property Id="myProperty" Value="test" /></Wix>"#);
        assert!(result.diagnostics.iter().any(|d| d.rule_id == "BP-MAINT-002" && d.message.contains("Property")));
    }

    #[test]
    fn test_property_all_uppercase_ok() {
        let result = analyze(r#"<Wix><Property Id="MY_PROPERTY" Value="test" /></Wix>"#);
        assert!(result.diagnostics.iter().all(|d| d.rule_id != "BP-MAINT-002" || !d.message.contains("Property")));
    }

    #[test]
    fn test_property_underscore_prefix_ok() {
        let result = analyze(r#"<Wix><Property Id="_internalProp" Value="test" /></Wix>"#);
        assert!(result.diagnostics.iter().all(|d| d.rule_id != "BP-MAINT-002" || !d.message.contains("Property")));
    }

    #[test]
    fn test_id_auto_generated_skipped() {
        // Id="*" should not trigger naming convention warnings
        let result = analyze(r#"<Wix><Component Id="*" /></Wix>"#);
        assert!(result.diagnostics.iter().all(|d| d.rule_id != "BP-MAINT-002"));
    }

    #[test]
    fn test_id_binding_expression_skipped() {
        // Id="!(bind.xxx)" should not trigger naming convention warnings
        let result = analyze(r#"<Wix><Component Id="!(bind.ComponentId)" /></Wix>"#);
        assert!(result.diagnostics.iter().all(|d| d.rule_id != "BP-MAINT-002"));
    }

    #[test]
    fn test_component_no_id() {
        // Component without Id should not crash
        let result = analyze(r#"<Wix><Component Guid="*" /></Wix>"#);
        // Should not cause any issues
        assert!(result.diagnostics.iter().all(|d| d.rule_id != "BP-EFF-001"));
    }

    #[test]
    fn test_property_no_id() {
        // Property without Id should not crash
        let result = analyze(r#"<Wix><Property Value="test" /></Wix>"#);
        assert!(result.diagnostics.iter().all(|d| d.rule_id != "BP-EFF-003"));
    }

    #[test]
    fn test_component_without_guid() {
        // Component without Guid should not crash hardcoded guid check
        let result = analyze(r#"<Wix><Component Id="C1" /></Wix>"#);
        assert!(result.diagnostics.iter().all(|d| d.rule_id != "BP-IDIOM-002"));
    }

    #[test]
    fn test_file_without_source() {
        // File without Source should not crash
        let result = analyze(r#"<Wix><File Id="F1" /></Wix>"#);
        assert!(result.diagnostics.iter().all(|d| d.rule_id != "BP-MAINT-001"));
    }
}
