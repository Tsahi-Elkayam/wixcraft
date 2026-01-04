//! WiX XML parser - parses .wxs files into a lintable AST

use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("Failed to read file: {0}")]
    ReadFile(#[from] std::io::Error),
    #[error("Failed to parse XML: {0}")]
    ParseXml(#[from] roxmltree::Error),
}

/// A parsed WiX document
#[derive(Debug)]
pub struct WixDocument {
    /// The source content (for error reporting)
    pub source: String,
    /// The root element
    pub root: WixElement,
    /// All elements in document order (for iteration)
    pub elements: Vec<WixElement>,
    /// Inline disable directives: line -> set of disabled rule IDs (empty = all)
    pub inline_disables: HashMap<usize, InlineDisable>,
}

/// Inline disable directive
#[derive(Debug, Clone)]
pub struct InlineDisable {
    /// If empty, all rules are disabled
    pub rules: HashSet<String>,
    /// Whether this affects the next line (vs current line)
    pub next_line: bool,
}

/// A WiX XML element
#[derive(Debug, Clone)]
pub struct WixElement {
    /// Element name (e.g., "Package", "Component")
    pub name: String,
    /// Element attributes
    pub attributes: HashMap<String, String>,
    /// Child element indices (into document's elements vec)
    pub children: Vec<usize>,
    /// Parent element index (None for root)
    pub parent: Option<usize>,
    /// Source location
    pub line: usize,
    pub column: usize,
    /// Byte offset in source for span calculation
    pub byte_offset: usize,
    /// Element's text content (if any)
    pub text: Option<String>,
}

impl WixDocument {
    /// Parse a WiX file
    pub fn parse_file(path: &Path) -> Result<Self, ParseError> {
        let source = fs::read_to_string(path)?;
        Self::parse_str(&source)
    }

    /// Parse WiX XML from a string
    pub fn parse_str(source: &str) -> Result<Self, ParseError> {
        let doc = roxmltree::Document::parse(source)?;
        let mut elements = Vec::new();

        // Parse inline disable comments
        let inline_disables = Self::parse_inline_disables(source);

        // Parse recursively
        fn parse_node(
            node: roxmltree::Node,
            elements: &mut Vec<WixElement>,
            parent_idx: Option<usize>,
        ) -> Option<usize> {
            if !node.is_element() {
                return None;
            }

            let idx = elements.len();

            // Get position info
            let pos = node.document().text_pos_at(node.range().start);

            // Collect attributes
            let attributes: HashMap<String, String> = node
                .attributes()
                .map(|a| (a.name().to_string(), a.value().to_string()))
                .collect();

            // Get text content
            let text = node
                .children()
                .find(|n| n.is_text())
                .map(|n| n.text().unwrap_or("").trim().to_string())
                .filter(|s| !s.is_empty());

            // Create element (children will be filled later)
            elements.push(WixElement {
                name: node.tag_name().name().to_string(),
                attributes,
                children: Vec::new(),
                parent: parent_idx,
                line: pos.row as usize,
                column: pos.col as usize,
                byte_offset: node.range().start,
                text,
            });

            // Parse children
            let child_indices: Vec<usize> = node
                .children()
                .filter_map(|child| parse_node(child, elements, Some(idx)))
                .collect();

            // Update children
            elements[idx].children = child_indices;

            Some(idx)
        }

        // Start parsing from root element
        if let Some(root_node) = doc.root_element().parent() {
            for child in root_node.children() {
                parse_node(child, &mut elements, None);
            }
        }

        let root = elements.first().cloned().unwrap_or(WixElement {
            name: String::new(),
            attributes: HashMap::new(),
            children: Vec::new(),
            parent: None,
            line: 1,
            column: 1,
            byte_offset: 0,
            text: None,
        });

        Ok(Self {
            source: source.to_string(),
            root,
            elements,
            inline_disables,
        })
    }

    /// Parse inline disable comments from source
    /// Supports:
    /// - <!-- wix-lint-disable --> - disable all rules for this line
    /// - <!-- wix-lint-disable rule-id --> - disable specific rule
    /// - <!-- wix-lint-disable rule1, rule2 --> - disable multiple rules
    /// - <!-- wix-lint-disable-next-line --> - disable for next line
    /// - <!-- wix-lint-disable-next-line rule-id --> - disable specific rule for next line
    fn parse_inline_disables(source: &str) -> HashMap<usize, InlineDisable> {
        let mut disables = HashMap::new();

        // Regex to match wix-lint-disable comments
        let re = Regex::new(
            r"<!--\s*wix-lint-disable(-next-line)?\s*([\w\-,\s]*)\s*-->"
        ).unwrap();

        for (line_num, line) in source.lines().enumerate() {
            let line_num = line_num + 1; // 1-based

            for cap in re.captures_iter(line) {
                let is_next_line = cap.get(1).is_some();
                let rules_str = cap.get(2).map(|m| m.as_str()).unwrap_or("");

                let rules: HashSet<String> = if rules_str.trim().is_empty() {
                    HashSet::new() // Empty means all rules
                } else {
                    rules_str
                        .split(',')
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty())
                        .collect()
                };

                let target_line = if is_next_line { line_num + 1 } else { line_num };

                disables.insert(target_line, InlineDisable {
                    rules,
                    next_line: is_next_line,
                });
            }
        }

        disables
    }

    /// Check if a rule is disabled at a specific line
    pub fn is_rule_disabled_at_line(&self, rule_id: &str, line: usize) -> bool {
        if let Some(disable) = self.inline_disables.get(&line) {
            // Empty rules set means all rules are disabled
            if disable.rules.is_empty() {
                return true;
            }
            return disable.rules.contains(rule_id);
        }
        false
    }

    /// Get element by index
    pub fn get(&self, idx: usize) -> Option<&WixElement> {
        self.elements.get(idx)
    }

    /// Get parent of an element
    pub fn parent(&self, element: &WixElement) -> Option<&WixElement> {
        element.parent.and_then(|idx| self.elements.get(idx))
    }

    /// Count children with a specific name
    pub fn count_children(&self, element: &WixElement, name: Option<&str>) -> usize {
        element
            .children
            .iter()
            .filter(|&idx| {
                if let Some(child) = self.elements.get(*idx) {
                    name.is_none_or(|n| child.name == n)
                } else {
                    false
                }
            })
            .count()
    }

    /// Check if element has a child with given name
    pub fn has_child(&self, element: &WixElement, name: &str) -> bool {
        element.children.iter().any(|&idx| {
            self.elements
                .get(idx)
                .is_some_and(|child| child.name == name)
        })
    }

    /// Get the source line for an element
    pub fn get_source_line(&self, line: usize) -> Option<String> {
        self.source.lines().nth(line.saturating_sub(1)).map(|s| s.to_string())
    }

    /// Iterate over all elements
    pub fn iter(&self) -> impl Iterator<Item = (usize, &WixElement)> {
        self.elements.iter().enumerate()
    }
}

impl WixElement {
    /// Get an attribute value
    pub fn attr(&self, name: &str) -> Option<&str> {
        self.attributes.get(name).map(|s| s.as_str())
    }

    /// Check if element has an attribute
    pub fn has_attr(&self, name: &str) -> bool {
        self.attributes.contains_key(name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<Wix xmlns="http://wixtoolset.org/schemas/v4/wxs">
  <Package Name="Test" Version="1.0.0" Manufacturer="Acme" UpgradeCode="12345678-1234-1234-1234-123456789012">
    <Component Guid="*">
      <File Source="test.exe" />
    </Component>
  </Package>
</Wix>"#;

        let doc = WixDocument::parse_str(xml).unwrap();
        assert!(!doc.elements.is_empty());
        assert_eq!(doc.root.name, "Wix");
    }

    #[test]
    fn test_count_children() {
        let xml = r#"<?xml version="1.0"?>
<Wix xmlns="http://wixtoolset.org/schemas/v4/wxs">
  <Package Name="Test" Version="1.0.0" Manufacturer="Acme" UpgradeCode="12345678-1234-1234-1234-123456789012">
    <Component Guid="*">
      <File Source="a.exe" />
      <File Source="b.exe" />
      <File Source="c.exe" />
    </Component>
  </Package>
</Wix>"#;

        let doc = WixDocument::parse_str(xml).unwrap();

        // Find the Component element
        let component = doc.elements.iter().find(|e| e.name == "Component").unwrap();
        assert_eq!(doc.count_children(component, Some("File")), 3);
    }

    #[test]
    fn test_count_all_children() {
        let xml = r#"<?xml version="1.0"?>
<Wix xmlns="http://wixtoolset.org/schemas/v4/wxs">
  <Package Name="Test" Version="1.0.0" Manufacturer="Acme" UpgradeCode="12345678-1234-1234-1234-123456789012">
    <Component Guid="*">
      <File Source="a.exe" />
      <File Source="b.exe" />
      <RegistryValue Root="HKCU" Key="Software" Name="Test" Value="1" />
    </Component>
  </Package>
</Wix>"#;

        let doc = WixDocument::parse_str(xml).unwrap();
        let component = doc.elements.iter().find(|e| e.name == "Component").unwrap();

        // Count all children (None = no filter)
        assert_eq!(doc.count_children(component, None), 3);
        // Count only File children
        assert_eq!(doc.count_children(component, Some("File")), 2);
        // Count only RegistryValue children
        assert_eq!(doc.count_children(component, Some("RegistryValue")), 1);
    }

    #[test]
    fn test_has_child() {
        let xml = r#"<?xml version="1.0"?>
<Wix xmlns="http://wixtoolset.org/schemas/v4/wxs">
  <Package Name="Test" Version="1.0.0" Manufacturer="Acme" UpgradeCode="12345678-1234-1234-1234-123456789012">
    <MajorUpgrade DowngradeErrorMessage="Cannot downgrade" />
    <Component Guid="*">
      <File Source="a.exe" />
    </Component>
  </Package>
</Wix>"#;

        let doc = WixDocument::parse_str(xml).unwrap();
        let package = doc.elements.iter().find(|e| e.name == "Package").unwrap();

        assert!(doc.has_child(package, "MajorUpgrade"));
        assert!(doc.has_child(package, "Component"));
        assert!(!doc.has_child(package, "Feature"));
    }

    #[test]
    fn test_element_attributes() {
        let xml = r#"<?xml version="1.0"?>
<Wix xmlns="http://wixtoolset.org/schemas/v4/wxs">
  <Package Name="TestApp" Version="2.0.0" Manufacturer="TestCo" UpgradeCode="12345678-1234-1234-1234-123456789012" />
</Wix>"#;

        let doc = WixDocument::parse_str(xml).unwrap();
        let package = doc.elements.iter().find(|e| e.name == "Package").unwrap();

        assert_eq!(package.attr("Name"), Some("TestApp"));
        assert_eq!(package.attr("Version"), Some("2.0.0"));
        assert_eq!(package.attr("Manufacturer"), Some("TestCo"));
        assert!(package.has_attr("Name"));
        assert!(!package.has_attr("Description"));
        assert_eq!(package.attr("NonExistent"), None);
    }

    #[test]
    fn test_parent_relationship() {
        let xml = r#"<?xml version="1.0"?>
<Wix xmlns="http://wixtoolset.org/schemas/v4/wxs">
  <Package Name="Test" Version="1.0.0" Manufacturer="Acme" UpgradeCode="12345678-1234-1234-1234-123456789012">
    <Component Guid="*">
      <File Source="test.exe" />
    </Component>
  </Package>
</Wix>"#;

        let doc = WixDocument::parse_str(xml).unwrap();

        // Find File element
        let (_file_idx, file) = doc.iter().find(|(_, e)| e.name == "File").unwrap();
        assert!(file.parent.is_some());

        // Get parent
        let parent = doc.parent(file).unwrap();
        assert_eq!(parent.name, "Component");

        // Wix root has no parent
        let wix = doc.elements.iter().find(|e| e.name == "Wix").unwrap();
        assert!(wix.parent.is_none());
        assert!(doc.parent(wix).is_none());
    }

    #[test]
    fn test_get_element() {
        let xml = r#"<?xml version="1.0"?>
<Wix xmlns="http://wixtoolset.org/schemas/v4/wxs">
  <Package Name="Test" Version="1.0.0" Manufacturer="Acme" UpgradeCode="12345678-1234-1234-1234-123456789012" />
</Wix>"#;

        let doc = WixDocument::parse_str(xml).unwrap();

        assert!(doc.get(0).is_some());
        assert_eq!(doc.get(0).unwrap().name, "Wix");
        assert!(doc.get(1).is_some());
        assert_eq!(doc.get(1).unwrap().name, "Package");
        assert!(doc.get(100).is_none());
    }

    #[test]
    fn test_get_source_line() {
        let xml = "<?xml version=\"1.0\"?>\n<Wix>\n  <Package />\n</Wix>";

        let doc = WixDocument::parse_str(xml).unwrap();

        assert_eq!(doc.get_source_line(1), Some("<?xml version=\"1.0\"?>".to_string()));
        assert_eq!(doc.get_source_line(2), Some("<Wix>".to_string()));
        assert_eq!(doc.get_source_line(3), Some("  <Package />".to_string()));
        assert_eq!(doc.get_source_line(4), Some("</Wix>".to_string()));
        assert_eq!(doc.get_source_line(100), None); // Line 100 doesn't exist
    }

    #[test]
    fn test_element_text_content() {
        let xml = r#"<?xml version="1.0"?>
<Wix xmlns="http://wixtoolset.org/schemas/v4/wxs">
  <CustomAction Id="MyAction" Execute="immediate">
    Some text content here
  </CustomAction>
</Wix>"#;

        let doc = WixDocument::parse_str(xml).unwrap();
        let custom_action = doc.elements.iter().find(|e| e.name == "CustomAction").unwrap();

        assert!(custom_action.text.is_some());
        assert_eq!(custom_action.text.as_deref(), Some("Some text content here"));
    }

    #[test]
    fn test_element_no_text() {
        let xml = r#"<?xml version="1.0"?>
<Wix xmlns="http://wixtoolset.org/schemas/v4/wxs">
  <Package Name="Test" Version="1.0.0" />
</Wix>"#;

        let doc = WixDocument::parse_str(xml).unwrap();
        let package = doc.elements.iter().find(|e| e.name == "Package").unwrap();

        assert!(package.text.is_none());
    }

    #[test]
    fn test_line_column_info() {
        let xml = r#"<?xml version="1.0"?>
<Wix>
  <Package />
</Wix>"#;

        let doc = WixDocument::parse_str(xml).unwrap();
        let package = doc.elements.iter().find(|e| e.name == "Package").unwrap();

        assert_eq!(package.line, 3);
        assert!(package.column > 0);
    }

    #[test]
    fn test_inline_disable_all() {
        let xml = r#"<?xml version="1.0"?>
<Wix xmlns="http://wixtoolset.org/schemas/v4/wxs">
  <!-- wix-lint-disable -->
  <Package Name="Test" Version="1.0.0" />
</Wix>"#;

        let doc = WixDocument::parse_str(xml).unwrap();

        // Line 3 has the disable comment, so line 3 should have all rules disabled
        assert!(doc.is_rule_disabled_at_line("any-rule", 3));
        assert!(doc.is_rule_disabled_at_line("another-rule", 3));

        // Line 4 should not be disabled
        assert!(!doc.is_rule_disabled_at_line("any-rule", 4));
    }

    #[test]
    fn test_inline_disable_specific_rule() {
        let xml = r#"<?xml version="1.0"?>
<Wix xmlns="http://wixtoolset.org/schemas/v4/wxs">
  <!-- wix-lint-disable package-requires-upgradecode -->
  <Package Name="Test" Version="1.0.0" />
</Wix>"#;

        let doc = WixDocument::parse_str(xml).unwrap();

        // Only the specific rule should be disabled
        assert!(doc.is_rule_disabled_at_line("package-requires-upgradecode", 3));
        assert!(!doc.is_rule_disabled_at_line("other-rule", 3));
    }

    #[test]
    fn test_inline_disable_multiple_rules() {
        let xml = r#"<?xml version="1.0"?>
<Wix xmlns="http://wixtoolset.org/schemas/v4/wxs">
  <!-- wix-lint-disable rule-a, rule-b, rule-c -->
  <Package Name="Test" Version="1.0.0" />
</Wix>"#;

        let doc = WixDocument::parse_str(xml).unwrap();

        assert!(doc.is_rule_disabled_at_line("rule-a", 3));
        assert!(doc.is_rule_disabled_at_line("rule-b", 3));
        assert!(doc.is_rule_disabled_at_line("rule-c", 3));
        assert!(!doc.is_rule_disabled_at_line("rule-d", 3));
    }

    #[test]
    fn test_inline_disable_next_line() {
        let xml = r#"<?xml version="1.0"?>
<Wix xmlns="http://wixtoolset.org/schemas/v4/wxs">
  <!-- wix-lint-disable-next-line package-requires-upgradecode -->
  <Package Name="Test" Version="1.0.0" />
</Wix>"#;

        let doc = WixDocument::parse_str(xml).unwrap();

        // The comment is on line 3, so line 4 should be disabled
        assert!(!doc.is_rule_disabled_at_line("package-requires-upgradecode", 3));
        assert!(doc.is_rule_disabled_at_line("package-requires-upgradecode", 4));
    }

    #[test]
    fn test_inline_disable_next_line_all() {
        let xml = r#"<?xml version="1.0"?>
<Wix xmlns="http://wixtoolset.org/schemas/v4/wxs">
  <!-- wix-lint-disable-next-line -->
  <Package Name="Test" Version="1.0.0" />
</Wix>"#;

        let doc = WixDocument::parse_str(xml).unwrap();

        // Line 4 should have all rules disabled
        assert!(doc.is_rule_disabled_at_line("any-rule", 4));
        assert!(doc.is_rule_disabled_at_line("another-rule", 4));

        // Line 3 should not be affected
        assert!(!doc.is_rule_disabled_at_line("any-rule", 3));
    }

    #[test]
    fn test_no_inline_disables() {
        let xml = r#"<?xml version="1.0"?>
<Wix xmlns="http://wixtoolset.org/schemas/v4/wxs">
  <Package Name="Test" Version="1.0.0" />
</Wix>"#;

        let doc = WixDocument::parse_str(xml).unwrap();

        // No line should have disabled rules
        assert!(!doc.is_rule_disabled_at_line("any-rule", 1));
        assert!(!doc.is_rule_disabled_at_line("any-rule", 2));
        assert!(!doc.is_rule_disabled_at_line("any-rule", 3));
    }

    #[test]
    fn test_iter_elements() {
        let xml = r#"<?xml version="1.0"?>
<Wix xmlns="http://wixtoolset.org/schemas/v4/wxs">
  <Package Name="Test" Version="1.0.0" Manufacturer="Acme" UpgradeCode="12345678-1234-1234-1234-123456789012">
    <Feature Id="Main" />
  </Package>
</Wix>"#;

        let doc = WixDocument::parse_str(xml).unwrap();

        let element_names: Vec<&str> = doc.iter().map(|(_, e)| e.name.as_str()).collect();

        assert!(element_names.contains(&"Wix"));
        assert!(element_names.contains(&"Package"));
        assert!(element_names.contains(&"Feature"));
        assert_eq!(element_names.len(), 3);
    }

    #[test]
    fn test_parse_invalid_xml() {
        let xml = "<Wix><Package></Wix>";

        let result = WixDocument::parse_str(xml);
        assert!(result.is_err());
    }

    #[test]
    fn test_empty_xml() {
        let xml = "<?xml version=\"1.0\"?><Root/>";

        let doc = WixDocument::parse_str(xml).unwrap();
        assert_eq!(doc.root.name, "Root");
    }

    #[test]
    fn test_deeply_nested() {
        let xml = r#"<?xml version="1.0"?>
<Wix>
  <Package Name="Test" Version="1.0.0" Manufacturer="Acme" UpgradeCode="12345678-1234-1234-1234-123456789012">
    <Directory Id="TARGETDIR">
      <Directory Id="ProgramFilesFolder">
        <Directory Id="INSTALLDIR">
          <Component Guid="*">
            <File Source="test.exe" />
          </Component>
        </Directory>
      </Directory>
    </Directory>
  </Package>
</Wix>"#;

        let doc = WixDocument::parse_str(xml).unwrap();

        // Verify all elements are parsed
        let names: Vec<&str> = doc.iter().map(|(_, e)| e.name.as_str()).collect();
        assert!(names.contains(&"Wix"));
        assert!(names.contains(&"Package"));
        assert!(names.contains(&"Directory"));
        assert!(names.contains(&"Component"));
        assert!(names.contains(&"File"));

        // Verify parent-child relationships
        let file = doc.elements.iter().find(|e| e.name == "File").unwrap();
        let component = doc.parent(file).unwrap();
        assert_eq!(component.name, "Component");

        let inner_dir = doc.parent(component).unwrap();
        assert_eq!(inner_dir.name, "Directory");
        assert_eq!(inner_dir.attr("Id"), Some("INSTALLDIR"));
    }
}
