//! Performance analyzers - multi-file components, deep nesting

use crate::types::{Impact, Location, PracticeCategory, Range, SuggestedFix, Suggestion};
use roxmltree::{Document, Node};
use std::path::Path;

/// Maximum recommended files per component
const MAX_FILES_PER_COMPONENT: usize = 1;

/// Maximum recommended directory nesting depth
const MAX_DIRECTORY_DEPTH: usize = 10;

/// Analyzer for performance issues
pub struct PerformanceAnalyzer {
    max_files_per_component: usize,
    max_directory_depth: usize,
}

impl PerformanceAnalyzer {
    pub fn new() -> Self {
        Self {
            max_files_per_component: MAX_FILES_PER_COMPONENT,
            max_directory_depth: MAX_DIRECTORY_DEPTH,
        }
    }

    /// Create with custom thresholds
    pub fn with_thresholds(max_files: usize, max_depth: usize) -> Self {
        Self {
            max_files_per_component: max_files,
            max_directory_depth: max_depth,
        }
    }

    /// Analyze a source file for performance issues
    pub fn analyze(&self, source: &str, file: &Path) -> Result<Vec<Suggestion>, String> {
        let doc = Document::parse(source).map_err(|e| format!("XML parse error: {}", e))?;
        let mut suggestions = Vec::new();

        // Check for multi-file components
        self.check_multi_file_components(doc.root(), source, file, &mut suggestions);

        // Check for deep directory nesting
        self.check_directory_depth(doc.root(), source, file, 0, &mut suggestions);

        // Check for large feature trees
        self.check_feature_complexity(doc.root(), source, file, &mut suggestions);

        Ok(suggestions)
    }

    fn check_multi_file_components(
        &self,
        node: Node,
        source: &str,
        file: &Path,
        suggestions: &mut Vec<Suggestion>,
    ) {
        if node.is_element() && node.tag_name().name() == "Component" {
            let file_count = node
                .children()
                .filter(|child| child.is_element() && child.tag_name().name() == "File")
                .count();

            if file_count > self.max_files_per_component {
                let range = get_node_range(&node, source);
                let location = Location::new(file.to_path_buf(), range);
                let id = node.attribute("Id").unwrap_or("unknown");

                suggestions.push(
                    Suggestion::new(
                        "BP-PERF-001",
                        "Multi-file Component",
                        PracticeCategory::Performance,
                        Impact::Medium,
                        format!(
                            "Component '{}' contains {} files. Consider using one file per component \
                             for better upgrade behavior and file tracking. Multiple files in a component \
                             share the same install state.",
                            id, file_count
                        ),
                        location,
                    )
                    .with_fix(SuggestedFix::new(
                        "Split into multiple components, one file each",
                    )),
                );
            }
        }

        for child in node.children() {
            self.check_multi_file_components(child, source, file, suggestions);
        }
    }

    fn check_directory_depth(
        &self,
        node: Node,
        source: &str,
        file: &Path,
        depth: usize,
        suggestions: &mut Vec<Suggestion>,
    ) {
        let new_depth = if node.is_element()
            && matches!(node.tag_name().name(), "Directory" | "StandardDirectory")
        {
            depth + 1
        } else {
            depth
        };

        if new_depth > self.max_directory_depth && node.is_element() && node.tag_name().name() == "Directory" {
            let range = get_node_range(&node, source);
            let location = Location::new(file.to_path_buf(), range);
            let id = node.attribute("Id").unwrap_or("unknown");

            suggestions.push(Suggestion::new(
                "BP-PERF-002",
                "Deep Directory Nesting",
                PracticeCategory::Performance,
                Impact::Low,
                format!(
                    "Directory '{}' is nested {} levels deep (max recommended: {}). \
                     Deep nesting can make the installer harder to maintain.",
                    id, new_depth, self.max_directory_depth
                ),
                location,
            ));
        }

        for child in node.children() {
            self.check_directory_depth(child, source, file, new_depth, suggestions);
        }
    }

    fn check_feature_complexity(
        &self,
        node: Node,
        source: &str,
        file: &Path,
        suggestions: &mut Vec<Suggestion>,
    ) {
        if node.is_element() && node.tag_name().name() == "Feature" {
            // Count direct ComponentRef children
            let component_count = node
                .children()
                .filter(|child| {
                    child.is_element()
                        && matches!(child.tag_name().name(), "ComponentRef" | "ComponentGroupRef")
                })
                .count();

            // Warn if feature has too many direct component references
            if component_count > 50 {
                let range = get_node_range(&node, source);
                let location = Location::new(file.to_path_buf(), range);
                let id = node.attribute("Id").unwrap_or("unknown");

                suggestions.push(
                    Suggestion::new(
                        "BP-PERF-003",
                        "Large Feature",
                        PracticeCategory::Performance,
                        Impact::Low,
                        format!(
                            "Feature '{}' has {} component references. Consider using ComponentGroup \
                             to organize components into logical groups for better maintainability.",
                            id, component_count
                        ),
                        location,
                    )
                    .with_fix(SuggestedFix::new(
                        "Group related components using ComponentGroup",
                    )),
                );
            }
        }

        for child in node.children() {
            self.check_feature_complexity(child, source, file, suggestions);
        }
    }
}

impl Default for PerformanceAnalyzer {
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
    fn test_multi_file_component() {
        let source = r#"<Wix>
            <Component Id="MultiFile">
                <File Id="F1" Source="a.dll" />
                <File Id="F2" Source="b.dll" />
            </Component>
        </Wix>"#;

        let analyzer = PerformanceAnalyzer::new();
        let suggestions = analyzer.analyze(source, Path::new("test.wxs")).unwrap();

        assert!(suggestions.iter().any(|s| s.rule_id == "BP-PERF-001"));
    }

    #[test]
    fn test_single_file_component() {
        let source = r#"<Wix>
            <Component Id="SingleFile">
                <File Id="F1" Source="a.dll" />
            </Component>
        </Wix>"#;

        let analyzer = PerformanceAnalyzer::new();
        let suggestions = analyzer.analyze(source, Path::new("test.wxs")).unwrap();

        assert!(suggestions.iter().all(|s| s.rule_id != "BP-PERF-001"));
    }

    #[test]
    fn test_deep_directory_nesting() {
        // Create deeply nested directories
        let source = r#"<Wix>
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
        </Wix>"#;

        let analyzer = PerformanceAnalyzer::new();
        let suggestions = analyzer.analyze(source, Path::new("test.wxs")).unwrap();

        assert!(suggestions.iter().any(|s| s.rule_id == "BP-PERF-002"));
    }

    #[test]
    fn test_acceptable_depth() {
        let source = r#"<Wix>
            <Directory Id="D1">
                <Directory Id="D2">
                    <Directory Id="D3" />
                </Directory>
            </Directory>
        </Wix>"#;

        let analyzer = PerformanceAnalyzer::new();
        let suggestions = analyzer.analyze(source, Path::new("test.wxs")).unwrap();

        assert!(suggestions.iter().all(|s| s.rule_id != "BP-PERF-002"));
    }

    #[test]
    fn test_custom_thresholds() {
        let source = r#"<Wix>
            <Component Id="C1">
                <File Id="F1" Source="a.dll" />
                <File Id="F2" Source="b.dll" />
                <File Id="F3" Source="c.dll" />
            </Component>
        </Wix>"#;

        // Allow 5 files per component
        let analyzer = PerformanceAnalyzer::with_thresholds(5, 10);
        let suggestions = analyzer.analyze(source, Path::new("test.wxs")).unwrap();

        // Should not warn with higher threshold
        assert!(suggestions.iter().all(|s| s.rule_id != "BP-PERF-001"));
    }
}
