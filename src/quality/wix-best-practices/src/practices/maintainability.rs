//! Maintainability analyzers - hardcoded paths, naming conventions

use crate::types::{Impact, Location, PracticeCategory, Range, SuggestedFix, Suggestion};
use regex::Regex;
use roxmltree::{Document, Node};
use std::path::Path;
use std::sync::LazyLock;

/// Windows absolute path pattern
static WINDOWS_ABSOLUTE_PATH: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^[A-Za-z]:\\").unwrap()
});

/// Unix absolute path pattern
static UNIX_ABSOLUTE_PATH: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^/(?:usr|home|var|opt|etc|tmp)").unwrap()
});

/// Valid WiX identifier pattern
static VALID_IDENTIFIER: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^[A-Za-z_][A-Za-z0-9_\.]*$").unwrap()
});

/// Analyzer for maintainability issues
pub struct MaintainabilityAnalyzer;

impl MaintainabilityAnalyzer {
    pub fn new() -> Self {
        Self
    }

    /// Analyze a source file for maintainability issues
    pub fn analyze(&self, source: &str, file: &Path) -> Result<Vec<Suggestion>, String> {
        let doc = Document::parse(source).map_err(|e| format!("XML parse error: {}", e))?;
        let mut suggestions = Vec::new();

        // Check for hardcoded absolute paths
        self.check_hardcoded_paths(doc.root(), source, file, &mut suggestions);

        // Check for naming convention issues
        self.check_naming_conventions(doc.root(), source, file, &mut suggestions);

        // Check for magic numbers
        self.check_magic_numbers(doc.root(), source, file, &mut suggestions);

        Ok(suggestions)
    }

    fn check_hardcoded_paths(
        &self,
        node: Node,
        source: &str,
        file: &Path,
        suggestions: &mut Vec<Suggestion>,
    ) {
        if node.is_element() {
            // Check Source attribute on File elements
            if node.tag_name().name() == "File" {
                if let Some(src) = node.attribute("Source") {
                    if is_absolute_path(src) {
                        let range = get_node_range(&node, source);
                        let location = Location::new(file.to_path_buf(), range);

                        suggestions.push(
                            Suggestion::new(
                                "BP-MAINT-001",
                                "Hardcoded Absolute Path",
                                PracticeCategory::Maintainability,
                                Impact::Medium,
                                format!(
                                    "File Source uses absolute path '{}'. Use relative paths \
                                     or preprocessor variables like $(var.SourceDir) for portability.",
                                    src
                                ),
                                location,
                            )
                            .with_fix(SuggestedFix::new(
                                "Use relative path or $(var.SourceDir)\\filename",
                            )),
                        );
                    }
                }
            }

            // Check for hardcoded paths in other attributes
            for attr in node.attributes() {
                if matches!(attr.name(), "Directory" | "DefaultDir" | "FileSource")
                    && is_absolute_path(attr.value())
                {
                    let range = get_node_range(&node, source);
                    let location = Location::new(file.to_path_buf(), range);

                    suggestions.push(Suggestion::new(
                        "BP-MAINT-001",
                        "Hardcoded Absolute Path",
                        PracticeCategory::Maintainability,
                        Impact::Medium,
                        format!(
                            "Attribute '{}' uses absolute path '{}'. Consider using \
                             preprocessor variables for portability.",
                            attr.name(),
                            attr.value()
                        ),
                        location,
                    ));
                }
            }
        }

        for child in node.children() {
            self.check_hardcoded_paths(child, source, file, suggestions);
        }
    }

    fn check_naming_conventions(
        &self,
        node: Node,
        source: &str,
        file: &Path,
        suggestions: &mut Vec<Suggestion>,
    ) {
        if node.is_element() {
            // Check Id attributes for naming conventions
            if let Some(id) = node.attribute("Id") {
                let tag_name = node.tag_name().name();

                // Skip auto-generated IDs
                if id == "*" || id.starts_with("!(") {
                    return;
                }

                // Check valid identifier format
                if !VALID_IDENTIFIER.is_match(id) && !id.contains('.') {
                    let range = get_node_range(&node, source);
                    let location = Location::new(file.to_path_buf(), range);

                    suggestions.push(Suggestion::new(
                        "BP-MAINT-002",
                        "Invalid Identifier Format",
                        PracticeCategory::Maintainability,
                        Impact::Low,
                        format!(
                            "{} Id '{}' contains invalid characters. \
                             Use only letters, numbers, underscores, and periods.",
                            tag_name, id
                        ),
                        location,
                    ));
                }

                // Check naming conventions by element type
                let warning = match tag_name {
                    "Component" if !id.starts_with("C_") && !id.starts_with("cmp") => {
                        Some("Consider prefixing Component IDs with 'C_' or 'cmp' for clarity")
                    }
                    "Directory" if !id.starts_with("D_") && !id.starts_with("dir") && id != "TARGETDIR" && !id.starts_with("INSTALL") => {
                        Some("Consider prefixing Directory IDs with 'D_' or 'dir' for clarity")
                    }
                    "Feature" if !id.starts_with("F_") && !id.starts_with("feat") => {
                        Some("Consider prefixing Feature IDs with 'F_' or 'feat' for clarity")
                    }
                    "Property" if id.chars().any(|c| c.is_lowercase()) && !id.starts_with("_") => {
                        Some("Public properties should be ALL_UPPERCASE. Use lowercase for private properties.")
                    }
                    _ => None,
                };

                if let Some(msg) = warning {
                    let range = get_node_range(&node, source);
                    let location = Location::new(file.to_path_buf(), range);

                    suggestions.push(Suggestion::new(
                        "BP-MAINT-003",
                        "Non-standard Naming",
                        PracticeCategory::Maintainability,
                        Impact::Low,
                        format!("{} Id '{}': {}", tag_name, id, msg),
                        location,
                    ));
                }
            }
        }

        for child in node.children() {
            self.check_naming_conventions(child, source, file, suggestions);
        }
    }

    fn check_magic_numbers(
        &self,
        node: Node,
        source: &str,
        file: &Path,
        suggestions: &mut Vec<Suggestion>,
    ) {
        if node.is_element() {
            // Check for magic numbers in certain contexts
            if node.tag_name().name() == "CustomAction" {
                if let Some(return_val) = node.attribute("Return") {
                    if return_val.parse::<i32>().is_ok() && return_val != "0" && return_val != "1" {
                        let range = get_node_range(&node, source);
                        let location = Location::new(file.to_path_buf(), range);

                        suggestions.push(Suggestion::new(
                            "BP-MAINT-004",
                            "Magic Number in CustomAction",
                            PracticeCategory::Maintainability,
                            Impact::Low,
                            format!(
                                "CustomAction Return '{}' is a magic number. \
                                 Use named values like 'check', 'ignore', 'asyncWait', 'asyncNoWait'.",
                                return_val
                            ),
                            location,
                        ));
                    }
                }
            }

            // Check Sequence attributes for magic numbers
            if let Some(seq) = node.attribute("Sequence") {
                if let Ok(num) = seq.parse::<i32>() {
                    if num != 0 && num != 1 {
                        let range = get_node_range(&node, source);
                        let location = Location::new(file.to_path_buf(), range);

                        suggestions.push(Suggestion::new(
                            "BP-MAINT-005",
                            "Hardcoded Sequence Number",
                            PracticeCategory::Maintainability,
                            Impact::Low,
                            format!(
                                "Sequence '{}' is hardcoded. Consider using Before/After \
                                 attributes for relative positioning.",
                                num
                            ),
                            location,
                        ));
                    }
                }
            }
        }

        for child in node.children() {
            self.check_magic_numbers(child, source, file, suggestions);
        }
    }
}

impl Default for MaintainabilityAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

fn is_absolute_path(path: &str) -> bool {
    WINDOWS_ABSOLUTE_PATH.is_match(path) || UNIX_ABSOLUTE_PATH.is_match(path)
}

fn get_node_range(node: &Node, source: &str) -> Range {
    let r = node.range();
    Range::from_offsets(source, r.start, r.end)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hardcoded_windows_path() {
        let source = r#"<Wix>
            <File Id="F1" Source="C:\Build\output\app.exe" />
        </Wix>"#;

        let analyzer = MaintainabilityAnalyzer::new();
        let suggestions = analyzer.analyze(source, Path::new("test.wxs")).unwrap();

        assert!(suggestions.iter().any(|s| s.rule_id == "BP-MAINT-001"));
    }

    #[test]
    fn test_relative_path() {
        let source = r#"<Wix>
            <File Id="F1" Source="$(var.SourceDir)\app.exe" />
        </Wix>"#;

        let analyzer = MaintainabilityAnalyzer::new();
        let suggestions = analyzer.analyze(source, Path::new("test.wxs")).unwrap();

        assert!(suggestions.iter().all(|s| s.rule_id != "BP-MAINT-001"));
    }

    #[test]
    fn test_invalid_identifier() {
        let source = r#"<Wix>
            <Component Id="My Component!" />
        </Wix>"#;

        let analyzer = MaintainabilityAnalyzer::new();
        let suggestions = analyzer.analyze(source, Path::new("test.wxs")).unwrap();

        assert!(suggestions.iter().any(|s| s.rule_id == "BP-MAINT-002"));
    }

    #[test]
    fn test_valid_identifier() {
        let source = r#"<Wix>
            <Component Id="MyComponent_123" />
        </Wix>"#;

        let analyzer = MaintainabilityAnalyzer::new();
        let suggestions = analyzer.analyze(source, Path::new("test.wxs")).unwrap();

        assert!(suggestions.iter().all(|s| s.rule_id != "BP-MAINT-002"));
    }

    #[test]
    fn test_non_standard_naming() {
        let source = r#"<Wix>
            <Component Id="AppComponent" />
        </Wix>"#;

        let analyzer = MaintainabilityAnalyzer::new();
        let suggestions = analyzer.analyze(source, Path::new("test.wxs")).unwrap();

        assert!(suggestions.iter().any(|s| s.rule_id == "BP-MAINT-003"));
    }

    #[test]
    fn test_standard_naming() {
        let source = r#"<Wix>
            <Component Id="C_AppComponent" />
        </Wix>"#;

        let analyzer = MaintainabilityAnalyzer::new();
        let suggestions = analyzer.analyze(source, Path::new("test.wxs")).unwrap();

        assert!(suggestions.iter().all(|s| s.rule_id != "BP-MAINT-003"));
    }

    #[test]
    fn test_lowercase_public_property() {
        let source = r#"<Wix>
            <Property Id="myProperty" Value="test" />
        </Wix>"#;

        let analyzer = MaintainabilityAnalyzer::new();
        let suggestions = analyzer.analyze(source, Path::new("test.wxs")).unwrap();

        assert!(suggestions.iter().any(|s| s.rule_id == "BP-MAINT-003"));
    }

    #[test]
    fn test_uppercase_public_property() {
        let source = r#"<Wix>
            <Property Id="MY_PROPERTY" Value="test" />
        </Wix>"#;

        let analyzer = MaintainabilityAnalyzer::new();
        let suggestions = analyzer.analyze(source, Path::new("test.wxs")).unwrap();

        // Should not warn about uppercase properties
        let property_warnings: Vec<_> = suggestions
            .iter()
            .filter(|s| s.rule_id == "BP-MAINT-003" && s.message.contains("Property"))
            .collect();
        assert!(property_warnings.is_empty());
    }

    #[test]
    fn test_magic_sequence_number() {
        let source = r#"<Wix>
            <Custom Action="CA1" Sequence="1500" />
        </Wix>"#;

        let analyzer = MaintainabilityAnalyzer::new();
        let suggestions = analyzer.analyze(source, Path::new("test.wxs")).unwrap();

        assert!(suggestions.iter().any(|s| s.rule_id == "BP-MAINT-005"));
    }
}
