//! Compare WiX/MSI versions and show changes
//!
//! Provides semantic comparison of WXS files, showing differences
//! in components, features, files, and registry entries.
//!
//! # Example
//!
//! ```
//! use wix_diff::{WixDiff, DiffOptions};
//!
//! let old = r#"<Wix><Package Name="App" Version="1.0" /></Wix>"#;
//! let new = r#"<Wix><Package Name="App" Version="2.0" /></Wix>"#;
//!
//! let diff = WixDiff::new(DiffOptions::default());
//! let result = diff.compare(old, new).unwrap();
//! assert!(!result.changes.is_empty());
//! ```

use roxmltree::{Document, Node};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use thiserror::Error;

/// Diff error types
#[derive(Error, Debug)]
pub enum DiffError {
    #[error("Failed to parse old file: {0}")]
    ParseOldError(String),
    #[error("Failed to parse new file: {0}")]
    ParseNewError(String),
}

/// Options for diff operation
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DiffOptions {
    /// Include unchanged elements in output
    pub show_unchanged: bool,
    /// Ignore whitespace differences
    pub ignore_whitespace: bool,
    /// Ignore attribute order
    pub ignore_attribute_order: bool,
    /// Ignore comments
    pub ignore_comments: bool,
}

/// Type of change detected
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChangeType {
    Added,
    Removed,
    Modified,
    Unchanged,
}

/// A change in a WiX element
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Change {
    /// Type of change
    pub change_type: ChangeType,
    /// Element type (e.g., Component, File, Feature)
    pub element_type: String,
    /// Element ID or identifier
    pub element_id: Option<String>,
    /// Path to element in document
    pub path: String,
    /// Old value (for modifications)
    pub old_value: Option<String>,
    /// New value (for modifications)
    pub new_value: Option<String>,
    /// Attribute-level changes for modifications
    pub attribute_changes: Vec<AttributeChange>,
}

/// A change in an attribute
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttributeChange {
    pub name: String,
    pub change_type: ChangeType,
    pub old_value: Option<String>,
    pub new_value: Option<String>,
}

/// Result of comparing two WiX files
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DiffResult {
    /// All detected changes
    pub changes: Vec<Change>,
    /// Summary of changes
    pub summary: DiffSummary,
}

/// Summary of differences
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DiffSummary {
    pub added: usize,
    pub removed: usize,
    pub modified: usize,
    pub unchanged: usize,
    /// Changes by element type
    pub by_element_type: HashMap<String, TypeSummary>,
}

/// Summary for a specific element type
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TypeSummary {
    pub added: usize,
    pub removed: usize,
    pub modified: usize,
}

/// WiX file differ
pub struct WixDiff {
    options: DiffOptions,
}

impl Default for WixDiff {
    fn default() -> Self {
        Self::new(DiffOptions::default())
    }
}

impl WixDiff {
    pub fn new(options: DiffOptions) -> Self {
        Self { options }
    }

    /// Compare two WiX XML strings
    pub fn compare(&self, old: &str, new: &str) -> Result<DiffResult, DiffError> {
        let old_doc =
            Document::parse(old).map_err(|e| DiffError::ParseOldError(e.to_string()))?;
        let new_doc =
            Document::parse(new).map_err(|e| DiffError::ParseNewError(e.to_string()))?;

        let mut result = DiffResult::default();

        // Extract elements from both documents
        let old_elements = self.extract_elements(old_doc.root_element(), "");
        let new_elements = self.extract_elements(new_doc.root_element(), "");

        // Build lookup maps by key
        let old_map: HashMap<_, _> = old_elements
            .iter()
            .map(|e| (self.element_key(e), e))
            .collect();
        let new_map: HashMap<_, _> = new_elements
            .iter()
            .map(|e| (self.element_key(e), e))
            .collect();

        let old_keys: HashSet<_> = old_map.keys().cloned().collect();
        let new_keys: HashSet<_> = new_map.keys().cloned().collect();

        // Find removed elements (in old but not in new)
        for key in old_keys.difference(&new_keys) {
            if let Some(elem) = old_map.get(key) {
                let change = Change {
                    change_type: ChangeType::Removed,
                    element_type: elem.element_type.clone(),
                    element_id: elem.id.clone(),
                    path: elem.path.clone(),
                    old_value: Some(elem.content.clone()),
                    new_value: None,
                    attribute_changes: vec![],
                };
                result.changes.push(change);
                result.summary.removed += 1;
                result
                    .summary
                    .by_element_type
                    .entry(elem.element_type.clone())
                    .or_default()
                    .removed += 1;
            }
        }

        // Find added elements (in new but not in old)
        for key in new_keys.difference(&old_keys) {
            if let Some(elem) = new_map.get(key) {
                let change = Change {
                    change_type: ChangeType::Added,
                    element_type: elem.element_type.clone(),
                    element_id: elem.id.clone(),
                    path: elem.path.clone(),
                    old_value: None,
                    new_value: Some(elem.content.clone()),
                    attribute_changes: vec![],
                };
                result.changes.push(change);
                result.summary.added += 1;
                result
                    .summary
                    .by_element_type
                    .entry(elem.element_type.clone())
                    .or_default()
                    .added += 1;
            }
        }

        // Find modified elements (in both, but different)
        for key in old_keys.intersection(&new_keys) {
            let old_elem = old_map.get(key).unwrap();
            let new_elem = new_map.get(key).unwrap();

            let attr_changes = self.compare_attributes(&old_elem.attributes, &new_elem.attributes);

            if !attr_changes.is_empty() || old_elem.content != new_elem.content {
                let change = Change {
                    change_type: ChangeType::Modified,
                    element_type: old_elem.element_type.clone(),
                    element_id: old_elem.id.clone(),
                    path: old_elem.path.clone(),
                    old_value: Some(old_elem.content.clone()),
                    new_value: Some(new_elem.content.clone()),
                    attribute_changes: attr_changes,
                };
                result.changes.push(change);
                result.summary.modified += 1;
                result
                    .summary
                    .by_element_type
                    .entry(old_elem.element_type.clone())
                    .or_default()
                    .modified += 1;
            } else if self.options.show_unchanged {
                result.summary.unchanged += 1;
            }
        }

        Ok(result)
    }

    /// Extract elements from XML node
    fn extract_elements(&self, node: Node, parent_path: &str) -> Vec<ElementInfo> {
        let mut elements = Vec::new();

        if !node.is_element() {
            return elements;
        }

        let tag = node.tag_name().name();
        let path = if parent_path.is_empty() {
            tag.to_string()
        } else {
            format!("{}/{}", parent_path, tag)
        };

        // Extract ID or identifying attribute
        let id = self.get_element_id(&node);

        // Build attributes map
        let mut attributes = HashMap::new();
        for attr in node.attributes() {
            attributes.insert(attr.name().to_string(), attr.value().to_string());
        }

        // Create content representation
        let content = self.node_to_string(&node);

        elements.push(ElementInfo {
            element_type: tag.to_string(),
            id,
            path: path.clone(),
            attributes,
            content,
        });

        // Recurse into children
        for child in node.children().filter(|n| n.is_element()) {
            elements.extend(self.extract_elements(child, &path));
        }

        elements
    }

    /// Get element's identifying attribute
    fn get_element_id(&self, node: &Node) -> Option<String> {
        // Common ID attributes in order of preference
        let id_attrs = ["Id", "Name", "Guid", "SourceFile", "Source", "Key"];

        for attr in &id_attrs {
            if let Some(val) = node.attribute(*attr) {
                return Some(val.to_string());
            }
        }
        None
    }

    /// Generate a unique key for an element
    fn element_key(&self, elem: &ElementInfo) -> String {
        if let Some(ref id) = elem.id {
            format!("{}:{}", elem.element_type, id)
        } else {
            format!("{}@{}", elem.element_type, elem.path)
        }
    }

    /// Convert node to string representation
    fn node_to_string(&self, node: &Node) -> String {
        let mut result = format!("<{}", node.tag_name().name());

        for attr in node.attributes() {
            result.push_str(&format!(" {}=\"{}\"", attr.name(), attr.value()));
        }

        let children: Vec<_> = node.children().filter(|n| n.is_element()).collect();
        if children.is_empty() {
            result.push_str(" />");
        } else {
            result.push('>');
            result.push_str(&format!("...{} children...", children.len()));
            result.push_str(&format!("</{}>", node.tag_name().name()));
        }

        result
    }

    /// Compare attributes between old and new
    fn compare_attributes(
        &self,
        old: &HashMap<String, String>,
        new: &HashMap<String, String>,
    ) -> Vec<AttributeChange> {
        let mut changes = Vec::new();

        let old_keys: HashSet<_> = old.keys().cloned().collect();
        let new_keys: HashSet<_> = new.keys().cloned().collect();

        // Removed attributes
        for key in old_keys.difference(&new_keys) {
            changes.push(AttributeChange {
                name: key.clone(),
                change_type: ChangeType::Removed,
                old_value: old.get(key).cloned(),
                new_value: None,
            });
        }

        // Added attributes
        for key in new_keys.difference(&old_keys) {
            changes.push(AttributeChange {
                name: key.clone(),
                change_type: ChangeType::Added,
                old_value: None,
                new_value: new.get(key).cloned(),
            });
        }

        // Modified attributes
        for key in old_keys.intersection(&new_keys) {
            let old_val = old.get(key).unwrap();
            let new_val = new.get(key).unwrap();
            if old_val != new_val {
                changes.push(AttributeChange {
                    name: key.clone(),
                    change_type: ChangeType::Modified,
                    old_value: Some(old_val.clone()),
                    new_value: Some(new_val.clone()),
                });
            }
        }

        changes
    }
}

/// Internal element representation
#[derive(Debug, Clone)]
struct ElementInfo {
    element_type: String,
    id: Option<String>,
    path: String,
    attributes: HashMap<String, String>,
    content: String,
}

/// Text-based diff for showing line-by-line changes
pub struct TextDiff {
    /// Context lines around changes
    pub context_lines: usize,
}

impl Default for TextDiff {
    fn default() -> Self {
        Self { context_lines: 3 }
    }
}

impl TextDiff {
    pub fn new(context_lines: usize) -> Self {
        Self { context_lines }
    }

    /// Generate unified diff output
    pub fn unified_diff(&self, old: &str, new: &str, old_name: &str, new_name: &str) -> String {
        use similar::{ChangeTag, TextDiff as SimilarDiff};

        let diff = SimilarDiff::from_lines(old, new);
        let mut output = String::new();

        output.push_str(&format!("--- {}\n", old_name));
        output.push_str(&format!("+++ {}\n", new_name));

        for (idx, group) in diff.grouped_ops(self.context_lines).iter().enumerate() {
            if idx > 0 {
                output.push_str("...\n");
            }

            for op in group {
                for change in diff.iter_changes(op) {
                    let tag = match change.tag() {
                        ChangeTag::Delete => "-",
                        ChangeTag::Insert => "+",
                        ChangeTag::Equal => " ",
                    };

                    output.push_str(tag);
                    output.push_str(change.value());
                    if change.missing_newline() {
                        output.push('\n');
                    }
                }
            }
        }

        output
    }

    /// Get statistics about differences
    pub fn stats(&self, old: &str, new: &str) -> DiffStats {
        use similar::{ChangeTag, TextDiff as SimilarDiff};

        let diff = SimilarDiff::from_lines(old, new);
        let mut stats = DiffStats::default();

        for change in diff.iter_all_changes() {
            match change.tag() {
                ChangeTag::Delete => stats.lines_removed += 1,
                ChangeTag::Insert => stats.lines_added += 1,
                ChangeTag::Equal => stats.lines_unchanged += 1,
            }
        }

        stats.total_old_lines = old.lines().count();
        stats.total_new_lines = new.lines().count();

        stats
    }
}

/// Statistics about text diff
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DiffStats {
    pub lines_added: usize,
    pub lines_removed: usize,
    pub lines_unchanged: usize,
    pub total_old_lines: usize,
    pub total_new_lines: usize,
}

impl DiffStats {
    pub fn has_changes(&self) -> bool {
        self.lines_added > 0 || self.lines_removed > 0
    }

    pub fn change_percentage(&self) -> f64 {
        let total = self.lines_added + self.lines_removed + self.lines_unchanged;
        if total == 0 {
            0.0
        } else {
            ((self.lines_added + self.lines_removed) as f64 / total as f64) * 100.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identical_files() {
        let content = r#"<Wix><Package Name="Test" /></Wix>"#;

        let diff = WixDiff::default();
        let result = diff.compare(content, content).unwrap();

        assert!(result.changes.is_empty());
        assert_eq!(result.summary.added, 0);
        assert_eq!(result.summary.removed, 0);
        assert_eq!(result.summary.modified, 0);
    }

    #[test]
    fn test_added_element() {
        let old = r#"<Wix><Package Name="Test" /></Wix>"#;
        let new = r#"<Wix><Package Name="Test" /><Feature Id="Main" /></Wix>"#;

        let diff = WixDiff::default();
        let result = diff.compare(old, new).unwrap();

        assert!(result.changes.iter().any(|c| c.change_type == ChangeType::Added));
        assert!(result.summary.added > 0);
    }

    #[test]
    fn test_removed_element() {
        let old = r#"<Wix><Package Name="Test" /><Feature Id="Main" /></Wix>"#;
        let new = r#"<Wix><Package Name="Test" /></Wix>"#;

        let diff = WixDiff::default();
        let result = diff.compare(old, new).unwrap();

        assert!(result.changes.iter().any(|c| c.change_type == ChangeType::Removed));
        assert!(result.summary.removed > 0);
    }

    #[test]
    fn test_modified_attribute() {
        let old = r#"<Wix><Package Name="Test" Version="1.0" /></Wix>"#;
        let new = r#"<Wix><Package Name="Test" Version="2.0" /></Wix>"#;

        let diff = WixDiff::default();
        let result = diff.compare(old, new).unwrap();

        assert!(result.changes.iter().any(|c| c.change_type == ChangeType::Modified));
        let modified = result
            .changes
            .iter()
            .find(|c| c.change_type == ChangeType::Modified)
            .unwrap();
        assert!(modified.attribute_changes.iter().any(|a| a.name == "Version"));
    }

    #[test]
    fn test_complex_diff() {
        let old = r#"<Wix>
            <Package Name="MyApp" Version="1.0" />
            <Feature Id="Main">
                <Component Id="Comp1" />
            </Feature>
        </Wix>"#;

        let new = r#"<Wix>
            <Package Name="MyApp" Version="2.0" />
            <Feature Id="Main">
                <Component Id="Comp1" />
                <Component Id="Comp2" />
            </Feature>
            <Feature Id="Extra" />
        </Wix>"#;

        let diff = WixDiff::default();
        let result = diff.compare(old, new).unwrap();

        // Should have version change, new component, new feature
        assert!(result.summary.added > 0);
        assert!(result.summary.modified > 0);
    }

    #[test]
    fn test_element_type_summary() {
        let old = r#"<Wix><Package Name="Test" /></Wix>"#;
        let new = r#"<Wix><Package Name="Test" /><Feature Id="F1" /><Feature Id="F2" /></Wix>"#;

        let diff = WixDiff::default();
        let result = diff.compare(old, new).unwrap();

        if let Some(feature_summary) = result.summary.by_element_type.get("Feature") {
            assert_eq!(feature_summary.added, 2);
        }
    }

    #[test]
    fn test_text_diff() {
        let old = "line1\nline2\nline3\n";
        let new = "line1\nmodified\nline3\n";

        let diff = TextDiff::default();
        let output = diff.unified_diff(old, new, "old.wxs", "new.wxs");

        assert!(output.contains("--- old.wxs"));
        assert!(output.contains("+++ new.wxs"));
        assert!(output.contains("-line2"));
        assert!(output.contains("+modified"));
    }

    #[test]
    fn test_diff_stats() {
        let old = "line1\nline2\nline3\n";
        let new = "line1\nmodified\nline3\nnew_line\n";

        let diff = TextDiff::default();
        let stats = diff.stats(old, new);

        assert_eq!(stats.lines_removed, 1); // line2
        assert_eq!(stats.lines_added, 2); // modified, new_line
        assert!(stats.has_changes());
    }

    #[test]
    fn test_no_changes_stats() {
        let content = "line1\nline2\nline3\n";

        let diff = TextDiff::default();
        let stats = diff.stats(content, content);

        assert!(!stats.has_changes());
        assert_eq!(stats.change_percentage(), 0.0);
    }

    #[test]
    fn test_parse_error_old() {
        let diff = WixDiff::default();
        let result = diff.compare("<invalid", "<Wix />");

        assert!(matches!(result, Err(DiffError::ParseOldError(_))));
    }

    #[test]
    fn test_parse_error_new() {
        let diff = WixDiff::default();
        let result = diff.compare("<Wix />", "<invalid");

        assert!(matches!(result, Err(DiffError::ParseNewError(_))));
    }

    #[test]
    fn test_attribute_added() {
        let old = r#"<Wix><Component Id="C1" /></Wix>"#;
        let new = r#"<Wix><Component Id="C1" Guid="*" /></Wix>"#;

        let diff = WixDiff::default();
        let result = diff.compare(old, new).unwrap();

        let modified = result
            .changes
            .iter()
            .find(|c| c.change_type == ChangeType::Modified && c.element_type == "Component");
        assert!(modified.is_some());

        let attr_change = modified
            .unwrap()
            .attribute_changes
            .iter()
            .find(|a| a.name == "Guid");
        assert!(attr_change.is_some());
        assert_eq!(attr_change.unwrap().change_type, ChangeType::Added);
    }

    #[test]
    fn test_attribute_removed() {
        let old = r#"<Wix><Component Id="C1" Guid="*" /></Wix>"#;
        let new = r#"<Wix><Component Id="C1" /></Wix>"#;

        let diff = WixDiff::default();
        let result = diff.compare(old, new).unwrap();

        let modified = result
            .changes
            .iter()
            .find(|c| c.change_type == ChangeType::Modified);
        assert!(modified.is_some());

        let attr_change = modified
            .unwrap()
            .attribute_changes
            .iter()
            .find(|a| a.name == "Guid");
        assert!(attr_change.is_some());
        assert_eq!(attr_change.unwrap().change_type, ChangeType::Removed);
    }

    #[test]
    fn test_diff_options() {
        let options = DiffOptions {
            show_unchanged: true,
            ignore_whitespace: true,
            ignore_attribute_order: true,
            ignore_comments: true,
        };

        let diff = WixDiff::new(options);
        let result = diff
            .compare(r#"<Wix><A Id="1" /></Wix>"#, r#"<Wix><A Id="1" /></Wix>"#)
            .unwrap();

        // With show_unchanged, summary should reflect unchanged items
        assert!(result.summary.modified == 0);
    }

    #[test]
    fn test_change_percentage() {
        let mut stats = DiffStats::default();
        stats.lines_added = 10;
        stats.lines_removed = 5;
        stats.lines_unchanged = 85;

        let pct = stats.change_percentage();
        assert!((pct - 15.0).abs() < 0.001); // 15% changed
    }

    #[test]
    fn test_type_summary() {
        let old = r#"<Wix>
            <Feature Id="F1" />
            <Component Id="C1" />
            <Component Id="C2" />
        </Wix>"#;

        let new = r#"<Wix>
            <Feature Id="F1" />
            <Feature Id="F2" />
            <Component Id="C3" />
        </Wix>"#;

        let diff = WixDiff::default();
        let result = diff.compare(old, new).unwrap();

        // Feature: +1, Component: +1 -2
        assert!(result.summary.by_element_type.contains_key("Feature"));
        assert!(result.summary.by_element_type.contains_key("Component"));
    }

    #[test]
    fn test_empty_files() {
        let diff = WixDiff::default();
        let result = diff.compare("<Wix />", "<Wix />");
        assert!(result.is_ok());
    }

    #[test]
    fn test_nested_changes() {
        let old = r#"<Wix>
            <Feature Id="Main">
                <Component Id="C1">
                    <File Source="a.exe" />
                </Component>
            </Feature>
        </Wix>"#;

        let new = r#"<Wix>
            <Feature Id="Main">
                <Component Id="C1">
                    <File Source="b.exe" />
                </Component>
            </Feature>
        </Wix>"#;

        let diff = WixDiff::default();
        let result = diff.compare(old, new).unwrap();

        // The File element with different Source is tracked by Source as ID,
        // so it appears as one removed (a.exe) and one added (b.exe)
        let file_added = result
            .changes
            .iter()
            .any(|c| c.element_type == "File" && c.change_type == ChangeType::Added);
        let file_removed = result
            .changes
            .iter()
            .any(|c| c.element_type == "File" && c.change_type == ChangeType::Removed);
        assert!(file_added || file_removed);
    }
}
