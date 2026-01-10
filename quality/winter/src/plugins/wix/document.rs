//! WiX document implementation

use super::parser::{parse_xml, WixNode};
use crate::plugin::{Document, Node, ParseError};
use regex::Regex;
use std::collections::HashSet;
use std::path::Path;
use std::sync::Arc;

/// A parsed WiX XML document
pub struct WixDocument {
    /// All nodes in the document
    nodes: Vec<Arc<WixNode>>,

    /// Source lines for display
    source_lines: Vec<String>,

    /// Lines with disable comments (rule_id -> set of lines)
    disabled_lines: std::collections::HashMap<String, HashSet<usize>>,

    /// Rules disabled for the entire file
    disabled_file_rules: HashSet<String>,
}

impl WixDocument {
    /// Parse WiX XML content into a document
    pub fn parse(content: &str, path: &Path) -> Result<Self, ParseError> {
        let nodes = parse_xml(content, path)?;
        let source_lines: Vec<String> = content.lines().map(String::from).collect();

        // Parse inline disable comments
        let (disabled_lines, disabled_file_rules) = Self::parse_disable_comments(&source_lines);

        Ok(Self {
            nodes,
            source_lines,
            disabled_lines,
            disabled_file_rules,
        })
    }

    /// Parse inline disable comments from source
    fn parse_disable_comments(
        lines: &[String],
    ) -> (std::collections::HashMap<String, HashSet<usize>>, HashSet<String>) {
        let mut disabled_lines: std::collections::HashMap<String, HashSet<usize>> =
            std::collections::HashMap::new();
        let mut disabled_file_rules: HashSet<String> = HashSet::new();

        // Patterns for disable comments
        let disable_re = Regex::new(r"<!--\s*winter-disable\s+(\S+)\s*-->").unwrap();
        let disable_next_re = Regex::new(r"<!--\s*winter-disable-next-line\s+(\S+)\s*-->").unwrap();
        let disable_file_re = Regex::new(r"<!--\s*winter-disable-file\s+(\S+)\s*-->").unwrap();

        for (i, line) in lines.iter().enumerate() {
            let line_num = i + 1;

            // Check for disable-file
            for cap in disable_file_re.captures_iter(line) {
                let rule_id = &cap[1];
                disabled_file_rules.insert(rule_id.to_string());
            }

            // Check for disable (applies to current line)
            for cap in disable_re.captures_iter(line) {
                let rule_id = &cap[1];
                disabled_lines
                    .entry(rule_id.to_string())
                    .or_default()
                    .insert(line_num);
            }

            // Check for disable-next-line
            for cap in disable_next_re.captures_iter(line) {
                let rule_id = &cap[1];
                disabled_lines
                    .entry(rule_id.to_string())
                    .or_default()
                    .insert(line_num + 1);
            }
        }

        (disabled_lines, disabled_file_rules)
    }
}

impl Document for WixDocument {
    fn root(&self) -> Option<&dyn Node> {
        self.nodes.first().map(|n| n.as_ref() as &dyn Node)
    }

    fn iter(&self) -> Box<dyn Iterator<Item = &dyn Node> + '_> {
        Box::new(self.nodes.iter().map(|n| n.as_ref() as &dyn Node))
    }

    fn get_source_line(&self, line: usize) -> Option<&str> {
        if line > 0 && line <= self.source_lines.len() {
            Some(&self.source_lines[line - 1])
        } else {
            None
        }
    }

    fn is_rule_disabled(&self, rule_id: &str, line: usize) -> bool {
        // Check "all" rules disabled
        if let Some(lines) = self.disabled_lines.get("all") {
            if lines.contains(&line) {
                return true;
            }
        }

        // Check specific rule
        if let Some(lines) = self.disabled_lines.get(rule_id) {
            if lines.contains(&line) {
                return true;
            }
        }

        false
    }

    fn is_rule_disabled_for_file(&self, rule_id: &str) -> bool {
        self.disabled_file_rules.contains("all") || self.disabled_file_rules.contains(rule_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_document() {
        let content = r#"<?xml version="1.0"?>
<Wix xmlns="http://wixtoolset.org/schemas/v4/wxs">
    <Package Name="Test"/>
</Wix>"#;

        let doc = WixDocument::parse(content, Path::new("test.wxs")).unwrap();
        assert!(doc.root().is_some());
    }

    #[test]
    fn test_iter_nodes() {
        let content = r#"<Wix><Package><Feature/></Package></Wix>"#;
        let doc = WixDocument::parse(content, Path::new("test.wxs")).unwrap();

        let names: Vec<&str> = doc.iter().map(|n| n.name()).collect();
        assert!(names.contains(&"Wix"));
        assert!(names.contains(&"Package"));
        assert!(names.contains(&"Feature"));
    }

    #[test]
    fn test_get_source_line() {
        let content = "line1\nline2\nline3";
        let doc = WixDocument::parse(content, Path::new("test.wxs")).unwrap();

        assert_eq!(doc.get_source_line(1), Some("line1"));
        assert_eq!(doc.get_source_line(2), Some("line2"));
        assert_eq!(doc.get_source_line(3), Some("line3"));
        assert_eq!(doc.get_source_line(0), None);
        assert_eq!(doc.get_source_line(4), None);
    }

    #[test]
    fn test_disable_comment() {
        let content = r#"<Wix>
    <!-- winter-disable test-rule -->
    <Package/>
</Wix>"#;

        let doc = WixDocument::parse(content, Path::new("test.wxs")).unwrap();
        assert!(doc.is_rule_disabled("test-rule", 2));
        assert!(!doc.is_rule_disabled("test-rule", 3));
        assert!(!doc.is_rule_disabled("other-rule", 2));
    }

    #[test]
    fn test_disable_next_line() {
        let content = r#"<Wix>
    <!-- winter-disable-next-line test-rule -->
    <Package/>
</Wix>"#;

        let doc = WixDocument::parse(content, Path::new("test.wxs")).unwrap();
        assert!(!doc.is_rule_disabled("test-rule", 2));
        assert!(doc.is_rule_disabled("test-rule", 3));
    }

    #[test]
    fn test_disable_file() {
        let content = r#"<!-- winter-disable-file test-rule -->
<Wix>
    <Package/>
</Wix>"#;

        let doc = WixDocument::parse(content, Path::new("test.wxs")).unwrap();
        assert!(doc.is_rule_disabled_for_file("test-rule"));
        assert!(!doc.is_rule_disabled_for_file("other-rule"));
    }

    #[test]
    fn test_disable_all() {
        let content = r#"<Wix>
    <!-- winter-disable all -->
    <Package/>
</Wix>"#;

        let doc = WixDocument::parse(content, Path::new("test.wxs")).unwrap();
        assert!(doc.is_rule_disabled("any-rule", 2));
    }
}
