//! XML document implementation

use crate::diagnostic::Location;
use crate::plugin::{Document, Node, ParseError};
use quick_xml::events::Event;
use quick_xml::Reader;
use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Type alias for disable comment parsing result
type DisableParseResult = (
    HashMap<String, HashSet<usize>>,  // disabled_lines
    HashSet<String>,                   // disabled_file_rules
    HashMap<String, HashMap<usize, String>>,  // disable_reasons
);

/// A node in the XML document
#[derive(Debug, Clone)]
pub struct XmlNode {
    pub kind: String,
    pub name: String,
    pub attrs: HashMap<String, String>,
    pub children: Vec<Arc<XmlNode>>,
    pub parent_idx: Option<usize>,
    pub file: PathBuf,
    pub line: usize,
    pub column: usize,
    pub length: usize,
    pub text_content: Option<String>,
}

impl XmlNode {
    pub fn element(name: &str, file: PathBuf, line: usize, column: usize) -> Self {
        Self {
            kind: "element".to_string(),
            name: name.to_string(),
            attrs: HashMap::new(),
            children: Vec::new(),
            parent_idx: None,
            file,
            line,
            column,
            length: name.len() + 2,
            text_content: None,
        }
    }

    pub fn text(content: &str, file: PathBuf, line: usize, column: usize) -> Self {
        Self {
            kind: "text".to_string(),
            name: "#text".to_string(),
            attrs: HashMap::new(),
            children: Vec::new(),
            parent_idx: None,
            file,
            line,
            column,
            length: content.len(),
            text_content: Some(content.to_string()),
        }
    }

    pub fn comment(content: &str, file: PathBuf, line: usize, column: usize) -> Self {
        let mut attrs = HashMap::new();
        attrs.insert("text".to_string(), content.to_string());
        Self {
            kind: "comment".to_string(),
            name: "#comment".to_string(),
            attrs,
            children: Vec::new(),
            parent_idx: None,
            file,
            line,
            column,
            length: content.len() + 7, // <!-- -->
            text_content: Some(content.to_string()),
        }
    }
}

impl Node for XmlNode {
    fn kind(&self) -> &str {
        &self.kind
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn get(&self, key: &str) -> Option<&str> {
        self.attrs.get(key).map(|s| s.as_str())
    }

    fn attributes(&self) -> &HashMap<String, String> {
        &self.attrs
    }

    fn children(&self) -> Vec<&dyn Node> {
        self.children.iter().map(|c| c.as_ref() as &dyn Node).collect()
    }

    fn parent(&self) -> Option<&dyn Node> {
        None
    }

    fn location(&self) -> Location {
        Location {
            file: self.file.clone(),
            line: self.line,
            column: self.column,
            length: self.length,
        }
    }

    fn text(&self) -> Option<&str> {
        self.text_content.as_deref()
    }
}

/// Information about a disable comment
#[derive(Debug, Clone)]
pub struct DisableInfo {
    pub line: usize,
    pub rule_id: String,
    pub reason: Option<String>,
}

/// A parsed XML document
pub struct XmlDocument {
    nodes: Vec<Arc<XmlNode>>,
    source_lines: Vec<String>,
    disabled_lines: HashMap<String, HashSet<usize>>,
    disabled_file_rules: HashSet<String>,
    /// Reasons for disable comments (rule_id -> line -> reason)
    disable_reasons: HashMap<String, HashMap<usize, String>>,
}

impl XmlDocument {
    pub fn parse(content: &str, path: &Path) -> Result<Self, ParseError> {
        let nodes = parse_xml(content, path)?;
        let source_lines: Vec<String> = content.lines().map(String::from).collect();
        let (disabled_lines, disabled_file_rules, disable_reasons) =
            Self::parse_disable_comments(&source_lines);

        Ok(Self {
            nodes,
            source_lines,
            disabled_lines,
            disabled_file_rules,
            disable_reasons,
        })
    }

    fn parse_disable_comments(lines: &[String]) -> DisableParseResult {
        let mut disabled_lines: HashMap<String, HashSet<usize>> = HashMap::new();
        let mut disabled_file_rules: HashSet<String> = HashSet::new();
        let mut disable_reasons: HashMap<String, HashMap<usize, String>> = HashMap::new();

        // Support formats:
        // <!-- winter-disable rule-id -->
        // <!-- winter-disable rule-id: reason here -->
        // <!-- winter-disable rule-id -- reason here -->
        let disable_re =
            Regex::new(r"<!--\s*winter-disable\s+(\S+?)(?:\s*:\s*(.+?)|(?:\s+--\s+(.+?)))?\s*-->")
                .unwrap();
        let disable_next_re = Regex::new(
            r"<!--\s*winter-disable-next-line\s+(\S+?)(?:\s*:\s*(.+?)|(?:\s+--\s+(.+?)))?\s*-->",
        )
        .unwrap();
        let disable_file_re = Regex::new(
            r"<!--\s*winter-disable-file\s+(\S+?)(?:\s*:\s*(.+?)|(?:\s+--\s+(.+?)))?\s*-->",
        )
        .unwrap();

        for (i, line) in lines.iter().enumerate() {
            let line_num = i + 1;

            for cap in disable_file_re.captures_iter(line) {
                let rule_id = cap[1].to_string();
                disabled_file_rules.insert(rule_id.clone());

                // Extract reason from either capture group 2 (colon style) or 3 (double-dash style)
                let reason = cap
                    .get(2)
                    .or_else(|| cap.get(3))
                    .map(|m| m.as_str().trim().to_string());
                if let Some(r) = reason {
                    disable_reasons
                        .entry(rule_id)
                        .or_default()
                        .insert(0, r); // 0 means file-level
                }
            }

            for cap in disable_re.captures_iter(line) {
                let rule_id = cap[1].to_string();
                disabled_lines
                    .entry(rule_id.clone())
                    .or_default()
                    .insert(line_num);

                let reason = cap
                    .get(2)
                    .or_else(|| cap.get(3))
                    .map(|m| m.as_str().trim().to_string());
                if let Some(r) = reason {
                    disable_reasons
                        .entry(rule_id)
                        .or_default()
                        .insert(line_num, r);
                }
            }

            for cap in disable_next_re.captures_iter(line) {
                let rule_id = cap[1].to_string();
                disabled_lines
                    .entry(rule_id.clone())
                    .or_default()
                    .insert(line_num + 1);

                let reason = cap
                    .get(2)
                    .or_else(|| cap.get(3))
                    .map(|m| m.as_str().trim().to_string());
                if let Some(r) = reason {
                    disable_reasons
                        .entry(rule_id)
                        .or_default()
                        .insert(line_num + 1, r);
                }
            }
        }

        (disabled_lines, disabled_file_rules, disable_reasons)
    }

    /// Get the reason why a rule was disabled at a specific line
    pub fn get_disable_reason(&self, rule_id: &str, line: usize) -> Option<&str> {
        // Check for specific rule
        if let Some(lines) = self.disable_reasons.get(rule_id) {
            if let Some(reason) = lines.get(&line) {
                return Some(reason);
            }
            // Check file-level (line 0)
            if let Some(reason) = lines.get(&0) {
                return Some(reason);
            }
        }
        // Check for "all" rule
        if let Some(lines) = self.disable_reasons.get("all") {
            if let Some(reason) = lines.get(&line) {
                return Some(reason);
            }
            if let Some(reason) = lines.get(&0) {
                return Some(reason);
            }
        }
        None
    }

    /// Get all disable information for this document
    pub fn get_all_disables(&self) -> Vec<DisableInfo> {
        let mut disables = Vec::new();

        // Collect file-level disables
        for rule_id in &self.disabled_file_rules {
            let reason = self
                .disable_reasons
                .get(rule_id)
                .and_then(|m| m.get(&0))
                .cloned();
            disables.push(DisableInfo {
                line: 0,
                rule_id: rule_id.clone(),
                reason,
            });
        }

        // Collect line-level disables
        for (rule_id, lines) in &self.disabled_lines {
            for &line in lines {
                let reason = self
                    .disable_reasons
                    .get(rule_id)
                    .and_then(|m| m.get(&line))
                    .cloned();
                disables.push(DisableInfo {
                    line,
                    rule_id: rule_id.clone(),
                    reason,
                });
            }
        }

        disables.sort_by_key(|d| (d.line, d.rule_id.clone()));
        disables
    }
}

impl Document for XmlDocument {
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
        if let Some(lines) = self.disabled_lines.get("all") {
            if lines.contains(&line) {
                return true;
            }
        }
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

fn parse_xml(content: &str, file_path: &Path) -> Result<Vec<Arc<XmlNode>>, ParseError> {
    let mut reader = Reader::from_str(content);
    reader.config_mut().trim_text(false); // Keep text as-is for XML plugin

    let mut nodes: Vec<Arc<XmlNode>> = Vec::new();
    let mut stack: Vec<usize> = Vec::new();
    let mut buf = Vec::new();

    let line_starts: Vec<usize> = std::iter::once(0)
        .chain(content.match_indices('\n').map(|(i, _)| i + 1))
        .collect();

    let pos_to_line_col = |pos: u64| -> (usize, usize) {
        let pos = pos as usize;
        let line = line_starts.partition_point(|&start| start <= pos);
        let col = pos - line_starts.get(line.saturating_sub(1)).unwrap_or(&0) + 1;
        (line, col)
    };

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) => {
                let (line, col) = pos_to_line_col(reader.buffer_position());
                let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                let mut node = XmlNode::element(&name, file_path.to_path_buf(), line, col);

                for attr in e.attributes().flatten() {
                    let key = String::from_utf8_lossy(attr.key.as_ref()).to_string();
                    let value = String::from_utf8_lossy(&attr.value).to_string();
                    node.attrs.insert(key, value);
                }

                if let Some(&parent_idx) = stack.last() {
                    node.parent_idx = Some(parent_idx);
                }

                let node_arc = Arc::new(node);
                let idx = nodes.len();

                if let Some(&parent_idx) = stack.last() {
                    let child_clone = Arc::clone(&node_arc);
                    nodes.push(node_arc);
                    let parent = Arc::make_mut(&mut nodes[parent_idx]);
                    parent.children.push(child_clone);
                } else {
                    nodes.push(node_arc);
                }

                stack.push(idx);
            }

            Ok(Event::Empty(e)) => {
                let (line, col) = pos_to_line_col(reader.buffer_position());
                let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                let mut node = XmlNode::element(&name, file_path.to_path_buf(), line, col);

                for attr in e.attributes().flatten() {
                    let key = String::from_utf8_lossy(attr.key.as_ref()).to_string();
                    let value = String::from_utf8_lossy(&attr.value).to_string();
                    node.attrs.insert(key, value);
                }

                if let Some(&parent_idx) = stack.last() {
                    node.parent_idx = Some(parent_idx);
                }

                let node_arc = Arc::new(node);

                if let Some(&parent_idx) = stack.last() {
                    let child_clone = Arc::clone(&node_arc);
                    nodes.push(node_arc);
                    let parent = Arc::make_mut(&mut nodes[parent_idx]);
                    parent.children.push(child_clone);
                } else {
                    nodes.push(node_arc);
                }
            }

            Ok(Event::End(_)) => {
                stack.pop();
            }

            Ok(Event::Text(e)) => {
                let text = e.unescape().map_err(|err| ParseError::Xml {
                    line: 0,
                    message: err.to_string(),
                })?;
                let trimmed = text.trim();
                if !trimmed.is_empty() {
                    let (line, col) = pos_to_line_col(reader.buffer_position());
                    let mut node = XmlNode::text(trimmed, file_path.to_path_buf(), line, col);

                    if let Some(&parent_idx) = stack.last() {
                        node.parent_idx = Some(parent_idx);
                    }

                    let node_arc = Arc::new(node);

                    if let Some(&parent_idx) = stack.last() {
                        let child_clone = Arc::clone(&node_arc);
                        nodes.push(node_arc);
                        let parent = Arc::make_mut(&mut nodes[parent_idx]);
                        parent.children.push(child_clone);
                    } else {
                        nodes.push(node_arc);
                    }
                }
            }

            Ok(Event::Comment(e)) => {
                let (line, col) = pos_to_line_col(reader.buffer_position());
                let text = String::from_utf8_lossy(&e).to_string();
                let mut node = XmlNode::comment(&text, file_path.to_path_buf(), line, col);

                if let Some(&parent_idx) = stack.last() {
                    node.parent_idx = Some(parent_idx);
                }

                let node_arc = Arc::new(node);

                if let Some(&parent_idx) = stack.last() {
                    let child_clone = Arc::clone(&node_arc);
                    nodes.push(node_arc);
                    let parent = Arc::make_mut(&mut nodes[parent_idx]);
                    parent.children.push(child_clone);
                } else {
                    nodes.push(node_arc);
                }
            }

            Ok(Event::Eof) => break,

            Err(e) => {
                let (line, _) = pos_to_line_col(reader.buffer_position());
                return Err(ParseError::Xml {
                    line,
                    message: e.to_string(),
                });
            }

            _ => {}
        }

        buf.clear();
    }

    Ok(nodes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_xml() {
        let content = r#"<?xml version="1.0"?><root><child/></root>"#;
        let doc = XmlDocument::parse(content, Path::new("test.xml")).unwrap();
        assert!(doc.root().is_some());
    }

    #[test]
    fn test_parse_comments() {
        let content = r#"<root><!-- This is a comment --></root>"#;
        let doc = XmlDocument::parse(content, Path::new("test.xml")).unwrap();
        let nodes: Vec<_> = doc.iter().collect();
        assert!(nodes.iter().any(|n| n.kind() == "comment"));
    }

    #[test]
    fn test_disable_with_colon_reason() {
        let content = r#"<root>
<!-- winter-disable my-rule: This is intentional -->
<element/>
</root>"#;
        let doc = XmlDocument::parse(content, Path::new("test.xml")).unwrap();
        assert!(doc.is_rule_disabled("my-rule", 2));
        assert_eq!(
            doc.get_disable_reason("my-rule", 2),
            Some("This is intentional")
        );
    }

    #[test]
    fn test_disable_with_doubledash_reason() {
        let content = r#"<root>
<!-- winter-disable my-rule -- Intentional for testing -->
<element/>
</root>"#;
        let doc = XmlDocument::parse(content, Path::new("test.xml")).unwrap();
        assert!(doc.is_rule_disabled("my-rule", 2));
        assert_eq!(
            doc.get_disable_reason("my-rule", 2),
            Some("Intentional for testing")
        );
    }

    #[test]
    fn test_disable_next_line_with_reason() {
        let content = r#"<root>
<!-- winter-disable-next-line my-rule: Legacy code -->
<element/>
</root>"#;
        let doc = XmlDocument::parse(content, Path::new("test.xml")).unwrap();
        assert!(doc.is_rule_disabled("my-rule", 3));
        assert_eq!(doc.get_disable_reason("my-rule", 3), Some("Legacy code"));
    }

    #[test]
    fn test_disable_file_with_reason() {
        let content = r#"<!-- winter-disable-file my-rule: Generated code -->
<root>
<element/>
</root>"#;
        let doc = XmlDocument::parse(content, Path::new("test.xml")).unwrap();
        assert!(doc.is_rule_disabled_for_file("my-rule"));
        // File-level reasons are stored with line 0
        assert_eq!(doc.get_disable_reason("my-rule", 0), Some("Generated code"));
    }

    #[test]
    fn test_get_all_disables() {
        let content = r#"<!-- winter-disable-file rule1: File reason -->
<root>
<!-- winter-disable rule2: Line reason -->
<element/>
</root>"#;
        let doc = XmlDocument::parse(content, Path::new("test.xml")).unwrap();
        let disables = doc.get_all_disables();

        assert_eq!(disables.len(), 2);

        let file_disable = disables.iter().find(|d| d.rule_id == "rule1").unwrap();
        assert_eq!(file_disable.line, 0);
        assert_eq!(file_disable.reason, Some("File reason".to_string()));

        let line_disable = disables.iter().find(|d| d.rule_id == "rule2").unwrap();
        assert_eq!(line_disable.line, 3);
        assert_eq!(line_disable.reason, Some("Line reason".to_string()));
    }

    #[test]
    fn test_disable_without_reason() {
        let content = r#"<root>
<!-- winter-disable my-rule -->
<element/>
</root>"#;
        let doc = XmlDocument::parse(content, Path::new("test.xml")).unwrap();
        assert!(doc.is_rule_disabled("my-rule", 2));
        assert_eq!(doc.get_disable_reason("my-rule", 2), None);
    }
}
