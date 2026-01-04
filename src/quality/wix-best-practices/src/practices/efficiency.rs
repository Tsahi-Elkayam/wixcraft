//! Efficiency analyzers - duplicates, unused elements

use crate::types::{Impact, Location, PracticeCategory, Range, Suggestion};
use roxmltree::{Document, Node};
use std::collections::{HashMap, HashSet};
use std::path::Path;

/// Analyzer for code efficiency issues
pub struct EfficiencyAnalyzer;

impl EfficiencyAnalyzer {
    pub fn new() -> Self {
        Self
    }

    /// Analyze a source file for efficiency issues
    pub fn analyze(&self, source: &str, file: &Path) -> Result<Vec<Suggestion>, String> {
        let doc = Document::parse(source).map_err(|e| format!("XML parse error: {}", e))?;
        let mut suggestions = Vec::new();

        // Collect component definitions and references
        let mut components: HashMap<String, Vec<(Node, Range)>> = HashMap::new();
        let mut component_refs: HashSet<String> = HashSet::new();
        let mut feature_refs: HashSet<String> = HashSet::new();

        self.collect_elements(doc.root(), source, &mut components, &mut component_refs, &mut feature_refs);

        // Check for duplicate component IDs
        for (id, locations) in &components {
            if locations.len() > 1 {
                for (node, range) in locations.iter().skip(1) {
                    let location = Location::new(file.to_path_buf(), *range);
                    suggestions.push(Suggestion::new(
                        "BP-EFF-001",
                        "Duplicate Component ID",
                        PracticeCategory::Efficiency,
                        Impact::High,
                        format!(
                            "Component Id '{}' is defined multiple times. Each Component should have a unique Id.",
                            id
                        ),
                        location,
                    ));
                    // Suppress unused warning
                    let _ = node;
                }
            }
        }

        // Check for unused components (not referenced by any Feature)
        for (id, locations) in &components {
            if !component_refs.contains(id) {
                if let Some((_, range)) = locations.first() {
                    let location = Location::new(file.to_path_buf(), *range);
                    suggestions.push(Suggestion::new(
                        "BP-EFF-002",
                        "Unused Component",
                        PracticeCategory::Efficiency,
                        Impact::Medium,
                        format!(
                            "Component '{}' is not referenced by any Feature. It will not be installed.",
                            id
                        ),
                        location,
                    ));
                }
            }
        }

        // Check for duplicate property definitions
        let mut properties: HashMap<String, Vec<Range>> = HashMap::new();
        self.collect_properties(doc.root(), source, &mut properties);

        for (id, ranges) in &properties {
            if ranges.len() > 1 {
                for range in ranges.iter().skip(1) {
                    let location = Location::new(file.to_path_buf(), *range);
                    suggestions.push(Suggestion::new(
                        "BP-EFF-003",
                        "Duplicate Property",
                        PracticeCategory::Efficiency,
                        Impact::Medium,
                        format!(
                            "Property '{}' is defined multiple times. Later definitions will override earlier ones.",
                            id
                        ),
                        location,
                    ));
                }
            }
        }

        Ok(suggestions)
    }

    fn collect_elements<'a>(
        &self,
        node: Node<'a, 'a>,
        source: &str,
        components: &mut HashMap<String, Vec<(Node<'a, 'a>, Range)>>,
        component_refs: &mut HashSet<String>,
        _feature_refs: &mut HashSet<String>,
    ) {
        if node.is_element() {
            let tag_name = node.tag_name().name();

            match tag_name {
                "Component" | "ComponentGroup" => {
                    if let Some(id) = node.attribute("Id") {
                        let range = get_node_range(&node, source);
                        components.entry(id.to_string()).or_default().push((node, range));
                    }
                }
                "ComponentRef" | "ComponentGroupRef" => {
                    if let Some(id) = node.attribute("Id") {
                        component_refs.insert(id.to_string());
                    }
                }
                _ => {}
            }
        }

        for child in node.children() {
            self.collect_elements(child, source, components, component_refs, _feature_refs);
        }
    }

    fn collect_properties(&self, node: Node, source: &str, properties: &mut HashMap<String, Vec<Range>>) {
        if node.is_element() && node.tag_name().name() == "Property" {
            if let Some(id) = node.attribute("Id") {
                let range = get_node_range(&node, source);
                properties.entry(id.to_string()).or_default().push(range);
            }
        }

        for child in node.children() {
            self.collect_properties(child, source, properties);
        }
    }
}

impl Default for EfficiencyAnalyzer {
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
    fn test_unused_component() {
        let source = r#"<Wix>
            <Component Id="UnusedComp" />
        </Wix>"#;

        let analyzer = EfficiencyAnalyzer::new();
        let suggestions = analyzer.analyze(source, Path::new("test.wxs")).unwrap();

        assert_eq!(suggestions.len(), 1);
        assert_eq!(suggestions[0].rule_id, "BP-EFF-002");
        assert!(suggestions[0].message.contains("UnusedComp"));
    }

    #[test]
    fn test_used_component() {
        let source = r#"<Wix>
            <Component Id="UsedComp" />
            <Feature Id="Main">
                <ComponentRef Id="UsedComp" />
            </Feature>
        </Wix>"#;

        let analyzer = EfficiencyAnalyzer::new();
        let suggestions = analyzer.analyze(source, Path::new("test.wxs")).unwrap();

        // No unused component warning
        assert!(suggestions.iter().all(|s| s.rule_id != "BP-EFF-002"));
    }

    #[test]
    fn test_duplicate_component_id() {
        let source = r#"<Wix>
            <Component Id="DupComp" />
            <Component Id="DupComp" />
        </Wix>"#;

        let analyzer = EfficiencyAnalyzer::new();
        let suggestions = analyzer.analyze(source, Path::new("test.wxs")).unwrap();

        assert!(suggestions.iter().any(|s| s.rule_id == "BP-EFF-001"));
    }

    #[test]
    fn test_duplicate_property() {
        let source = r#"<Wix>
            <Property Id="MYPROP" Value="1" />
            <Property Id="MYPROP" Value="2" />
        </Wix>"#;

        let analyzer = EfficiencyAnalyzer::new();
        let suggestions = analyzer.analyze(source, Path::new("test.wxs")).unwrap();

        assert!(suggestions.iter().any(|s| s.rule_id == "BP-EFF-003"));
    }

    #[test]
    fn test_component_group_ref_counts() {
        let source = r#"<Wix>
            <ComponentGroup Id="MyGroup" />
            <Feature Id="Main">
                <ComponentGroupRef Id="MyGroup" />
            </Feature>
        </Wix>"#;

        let analyzer = EfficiencyAnalyzer::new();
        let suggestions = analyzer.analyze(source, Path::new("test.wxs")).unwrap();

        // ComponentGroup referenced via ComponentGroupRef should not be flagged
        assert!(suggestions.iter().all(|s| !s.message.contains("MyGroup")));
    }
}
