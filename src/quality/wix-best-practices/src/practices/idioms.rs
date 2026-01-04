//! WiX idioms analyzers - patterns and best practices

use crate::types::{Impact, Location, PracticeCategory, Range, SuggestedFix, Suggestion};
use regex::Regex;
use roxmltree::{Document, Node};
use std::path::Path;
use std::sync::LazyLock;

/// GUID pattern (full format with braces)
static GUID_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^\{?[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}\}?$").unwrap()
});

/// Analyzer for WiX idioms and patterns
pub struct IdiomsAnalyzer;

impl IdiomsAnalyzer {
    pub fn new() -> Self {
        Self
    }

    /// Analyze a source file for idiom issues
    pub fn analyze(&self, source: &str, file: &Path) -> Result<Vec<Suggestion>, String> {
        let doc = Document::parse(source).map_err(|e| format!("XML parse error: {}", e))?;
        let mut suggestions = Vec::new();

        // Check for Package without MajorUpgrade
        self.check_major_upgrade(doc.root(), source, file, &mut suggestions);

        // Check for hardcoded GUIDs (should use "*")
        self.check_hardcoded_guids(doc.root(), source, file, &mut suggestions);

        // Check for deprecated elements
        self.check_deprecated_elements(doc.root(), source, file, &mut suggestions);

        // Check for missing UpgradeCode
        self.check_upgrade_code(doc.root(), source, file, &mut suggestions);

        Ok(suggestions)
    }

    fn check_major_upgrade(
        &self,
        node: Node,
        source: &str,
        file: &Path,
        suggestions: &mut Vec<Suggestion>,
    ) {
        // Find Package elements
        if node.is_element() && node.tag_name().name() == "Package" {
            // Check if there's a MajorUpgrade child
            let has_major_upgrade = node.children().any(|child| {
                child.is_element() && child.tag_name().name() == "MajorUpgrade"
            });

            if !has_major_upgrade {
                let range = get_node_range(&node, source);
                let location = Location::new(file.to_path_buf(), range);
                suggestions.push(
                    Suggestion::new(
                        "BP-IDIOM-001",
                        "Missing MajorUpgrade",
                        PracticeCategory::Idiom,
                        Impact::High,
                        "Package should include a MajorUpgrade element to handle upgrades properly. \
                         Without it, users cannot upgrade the product without first uninstalling.",
                        location,
                    )
                    .with_fix(SuggestedFix::new(
                        "Add <MajorUpgrade DowngradeErrorMessage=\"A newer version is already installed.\" />",
                    )),
                );
            }
        }

        for child in node.children() {
            self.check_major_upgrade(child, source, file, suggestions);
        }
    }

    fn check_hardcoded_guids(
        &self,
        node: Node,
        source: &str,
        file: &Path,
        suggestions: &mut Vec<Suggestion>,
    ) {
        if node.is_element() && node.tag_name().name() == "Component" {
            if let Some(guid) = node.attribute("Guid") {
                // Check if it's a hardcoded GUID (not "*")
                if guid != "*" && GUID_REGEX.is_match(guid) {
                    let range = get_node_range(&node, source);
                    let location = Location::new(file.to_path_buf(), range);
                    suggestions.push(
                        Suggestion::new(
                            "BP-IDIOM-002",
                            "Hardcoded Component GUID",
                            PracticeCategory::Idiom,
                            Impact::Medium,
                            format!(
                                "Component uses hardcoded GUID '{}'. Consider using Guid=\"*\" \
                                 for auto-generated GUIDs, which are more maintainable.",
                                guid
                            ),
                            location,
                        )
                        .with_fix(
                            SuggestedFix::new("Use Guid=\"*\" for auto-generation")
                                .with_replacement("*"),
                        ),
                    );
                }
            }
        }

        for child in node.children() {
            self.check_hardcoded_guids(child, source, file, suggestions);
        }
    }

    fn check_deprecated_elements(
        &self,
        node: Node,
        source: &str,
        file: &Path,
        suggestions: &mut Vec<Suggestion>,
    ) {
        if node.is_element() {
            let tag_name = node.tag_name().name();

            // WiX v4 deprecations
            let deprecation = match tag_name {
                "Product" => Some((
                    "Product element is deprecated in WiX v4",
                    "Use Package element instead",
                )),
                "Fragment" if node.children().any(|c| c.is_element() && c.tag_name().name() == "Product") => {
                    Some((
                        "Fragment containing Product is deprecated",
                        "Use Package element directly",
                    ))
                }
                _ => None,
            };

            if let Some((message, fix_desc)) = deprecation {
                let range = get_node_range(&node, source);
                let location = Location::new(file.to_path_buf(), range);
                suggestions.push(
                    Suggestion::new(
                        "BP-IDIOM-003",
                        "Deprecated Element",
                        PracticeCategory::Idiom,
                        Impact::Medium,
                        message,
                        location,
                    )
                    .with_fix(SuggestedFix::new(fix_desc)),
                );
            }
        }

        for child in node.children() {
            self.check_deprecated_elements(child, source, file, suggestions);
        }
    }

    fn check_upgrade_code(
        &self,
        node: Node,
        source: &str,
        file: &Path,
        suggestions: &mut Vec<Suggestion>,
    ) {
        if node.is_element() && node.tag_name().name() == "Package" {
            if node.attribute("UpgradeCode").is_none() {
                let range = get_node_range(&node, source);
                let location = Location::new(file.to_path_buf(), range);
                suggestions.push(Suggestion::new(
                    "BP-IDIOM-004",
                    "Missing UpgradeCode",
                    PracticeCategory::Idiom,
                    Impact::High,
                    "Package should have an UpgradeCode attribute to support upgrades. \
                     Generate a GUID and use it consistently across versions.",
                    location,
                ));
            }
        }

        for child in node.children() {
            self.check_upgrade_code(child, source, file, suggestions);
        }
    }
}

impl Default for IdiomsAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

fn get_node_range(node: &Node, source: &str) -> Range {
    let r = node.range();
    Range::from_offsets(source, r.start, r.end)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_missing_major_upgrade() {
        let source = r#"<Wix><Package Name="Test" Version="1.0" /></Wix>"#;

        let analyzer = IdiomsAnalyzer::new();
        let suggestions = analyzer.analyze(source, Path::new("test.wxs")).unwrap();

        assert!(suggestions.iter().any(|s| s.rule_id == "BP-IDIOM-001"));
    }

    #[test]
    fn test_has_major_upgrade() {
        let source = r#"<Wix><Package Name="Test" Version="1.0"><MajorUpgrade /></Package></Wix>"#;

        let analyzer = IdiomsAnalyzer::new();
        let suggestions = analyzer.analyze(source, Path::new("test.wxs")).unwrap();

        // Should not have missing MajorUpgrade warning
        assert!(suggestions.iter().all(|s| s.rule_id != "BP-IDIOM-001"));
    }

    #[test]
    fn test_hardcoded_guid() {
        let source = r#"<Wix><Component Id="C1" Guid="{12345678-1234-1234-1234-123456789ABC}" /></Wix>"#;

        let analyzer = IdiomsAnalyzer::new();
        let suggestions = analyzer.analyze(source, Path::new("test.wxs")).unwrap();

        assert!(suggestions.iter().any(|s| s.rule_id == "BP-IDIOM-002"));
    }

    #[test]
    fn test_auto_guid() {
        let source = r#"<Wix><Component Id="C1" Guid="*" /></Wix>"#;

        let analyzer = IdiomsAnalyzer::new();
        let suggestions = analyzer.analyze(source, Path::new("test.wxs")).unwrap();

        // Should not have hardcoded GUID warning
        assert!(suggestions.iter().all(|s| s.rule_id != "BP-IDIOM-002"));
    }

    #[test]
    fn test_deprecated_product() {
        let source = r#"<Wix><Product Id="*" Name="Test" /></Wix>"#;

        let analyzer = IdiomsAnalyzer::new();
        let suggestions = analyzer.analyze(source, Path::new("test.wxs")).unwrap();

        assert!(suggestions.iter().any(|s| s.rule_id == "BP-IDIOM-003"));
    }

    #[test]
    fn test_missing_upgrade_code() {
        let source = r#"<Wix><Package Name="Test" Version="1.0" /></Wix>"#;

        let analyzer = IdiomsAnalyzer::new();
        let suggestions = analyzer.analyze(source, Path::new("test.wxs")).unwrap();

        assert!(suggestions.iter().any(|s| s.rule_id == "BP-IDIOM-004"));
    }

    #[test]
    fn test_has_upgrade_code() {
        let source = r#"<Wix><Package Name="Test" Version="1.0" UpgradeCode="{12345678-1234-1234-1234-123456789ABC}" /></Wix>"#;

        let analyzer = IdiomsAnalyzer::new();
        let suggestions = analyzer.analyze(source, Path::new("test.wxs")).unwrap();

        assert!(suggestions.iter().all(|s| s.rule_id != "BP-IDIOM-004"));
    }
}
