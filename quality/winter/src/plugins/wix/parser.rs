//! WiX XML parser using quick-xml

use crate::diagnostic::Location;
use crate::plugin::{Node, ParseError};
use quick_xml::events::Event;
use quick_xml::Reader;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

/// A node in the WiX XML document
#[derive(Debug, Clone)]
pub struct WixNode {
    /// Node kind ("element", "text", "comment")
    pub kind: String,

    /// Element name (e.g., "Package", "Component")
    pub name: String,

    /// Attributes
    pub attrs: HashMap<String, String>,

    /// Child nodes
    pub children: Vec<Arc<WixNode>>,

    /// Parent node (weak reference via index)
    pub parent_idx: Option<usize>,

    /// Source location
    pub file: PathBuf,
    pub line: usize,
    pub column: usize,
    pub length: usize,

    /// Text content (for text nodes)
    pub text_content: Option<String>,
}

impl WixNode {
    /// Create a new element node
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
            length: name.len() + 2, // <Name>
            text_content: None,
        }
    }

    /// Create a text node
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

    /// Add an attribute
    pub fn with_attr(mut self, name: &str, value: &str) -> Self {
        self.attrs.insert(name.to_string(), value.to_string());
        self
    }
}

/// Parse WiX XML content into a tree of nodes
pub fn parse_xml(content: &str, file_path: &std::path::Path) -> Result<Vec<Arc<WixNode>>, ParseError> {
    let mut reader = Reader::from_str(content);
    reader.config_mut().trim_text(true);

    let mut nodes: Vec<Arc<WixNode>> = Vec::new();
    let mut stack: Vec<usize> = Vec::new(); // Stack of indices into nodes
    let mut buf = Vec::new();

    // Pre-calculate line positions for fast lookup
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
                let mut node = WixNode::element(&name, file_path.to_path_buf(), line, col);

                // Parse attributes
                for attr in e.attributes().flatten() {
                    let key = String::from_utf8_lossy(attr.key.as_ref()).to_string();
                    let value = String::from_utf8_lossy(&attr.value).to_string();
                    node.attrs.insert(key, value);
                }

                // Set parent
                if let Some(&parent_idx) = stack.last() {
                    node.parent_idx = Some(parent_idx);
                }

                let node_arc = Arc::new(node);
                let idx = nodes.len();

                // Add as child to parent first (clone before push to avoid borrow issues)
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
                let mut node = WixNode::element(&name, file_path.to_path_buf(), line, col);

                // Parse attributes
                for attr in e.attributes().flatten() {
                    let key = String::from_utf8_lossy(attr.key.as_ref()).to_string();
                    let value = String::from_utf8_lossy(&attr.value).to_string();
                    node.attrs.insert(key, value);
                }

                // Set parent
                if let Some(&parent_idx) = stack.last() {
                    node.parent_idx = Some(parent_idx);
                }

                let node_arc = Arc::new(node);

                // Add as child to parent (clone before push)
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
                    let mut node = WixNode::text(trimmed, file_path.to_path_buf(), line, col);

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

            Ok(Event::Eof) => break,

            Err(e) => {
                let (line, _) = pos_to_line_col(reader.buffer_position());
                return Err(ParseError::Xml {
                    line,
                    message: e.to_string(),
                });
            }

            _ => {} // Skip comments, declarations, etc.
        }

        buf.clear();
    }

    Ok(nodes)
}

// Node implementation for WixNode
impl Node for WixNode {
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
        // Cannot return parent without all_nodes context
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_parse_simple_element() {
        let xml = r#"<Wix><Package/></Wix>"#;
        let nodes = parse_xml(xml, Path::new("test.wxs")).unwrap();

        assert!(!nodes.is_empty());
        assert_eq!(nodes[0].name, "Wix");
    }

    #[test]
    fn test_parse_attributes() {
        let xml = r#"<Package Id="MyProduct" Name="Test"/>"#;
        let nodes = parse_xml(xml, Path::new("test.wxs")).unwrap();

        assert_eq!(nodes[0].attrs.get("Id"), Some(&"MyProduct".to_string()));
        assert_eq!(nodes[0].attrs.get("Name"), Some(&"Test".to_string()));
    }

    #[test]
    fn test_parse_nested() {
        let xml = r#"<Wix><Package><Feature Id="Main"/></Package></Wix>"#;
        let nodes = parse_xml(xml, Path::new("test.wxs")).unwrap();

        assert_eq!(nodes[0].name, "Wix");
        assert_eq!(nodes[0].children.len(), 1);
        assert_eq!(nodes[0].children[0].name, "Package");
    }

    #[test]
    fn test_node_trait() {
        let xml = r#"<Package Id="Test" Version="1.0"/>"#;
        let nodes = parse_xml(xml, Path::new("test.wxs")).unwrap();

        let node: &dyn Node = nodes[0].as_ref();
        assert_eq!(node.kind(), "element");
        assert_eq!(node.name(), "Package");
        assert_eq!(node.get("Id"), Some("Test"));
        assert_eq!(node.get("Version"), Some("1.0"));
        assert_eq!(node.get("Missing"), None);
    }

    #[test]
    fn test_parse_error() {
        let xml = r#"<Unclosed"#;
        let result = parse_xml(xml, Path::new("test.wxs"));
        assert!(result.is_err());
    }
}
