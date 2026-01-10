//! Generic AST types for the analysis engine
//!
//! These traits abstract over language-specific AST representations,
//! allowing the engine to work with any language.

use std::path::Path;

/// A node in the abstract syntax tree
pub trait Node: Send + Sync {
    /// Get the node kind/type (e.g., "element", "attribute", "Component")
    fn kind(&self) -> &str;

    /// Get the node's text content
    fn text(&self) -> &str;

    /// Get the source range (start_line, start_col, end_line, end_col)
    /// Lines and columns are 1-based
    fn range(&self) -> (usize, usize, usize, usize);

    /// Get the parent node, if any
    fn parent(&self) -> Option<&dyn Node>;

    /// Get child nodes
    fn children(&self) -> Vec<&dyn Node>;

    /// Get an attribute value by name
    fn attribute(&self, name: &str) -> Option<&str>;

    /// Get all attributes
    fn attributes(&self) -> Vec<Attribute>;

    /// Check if this node has a specific attribute
    fn has_attribute(&self, name: &str) -> bool {
        self.attribute(name).is_some()
    }

    /// Get the number of children with a specific kind
    fn count_children(&self, kind: &str) -> usize {
        self.children().iter().filter(|c| c.kind() == kind).count()
    }

    /// Check if node has a child with specific kind
    fn has_child(&self, kind: &str) -> bool {
        self.children().iter().any(|c| c.kind() == kind)
    }

    /// Get all descendants (recursive)
    fn descendants(&self) -> Vec<&dyn Node> {
        let mut result = Vec::new();
        for child in self.children() {
            result.push(child);
            result.extend(child.descendants());
        }
        result
    }
}

/// A parsed document
pub trait Document: Send + Sync {
    /// Get the source text
    fn source(&self) -> &str;

    /// Get the file path
    fn path(&self) -> &Path;

    /// Get the root node
    fn root(&self) -> &dyn Node;

    /// Get all nodes of a specific kind
    fn nodes_of_kind(&self, kind: &str) -> Vec<&dyn Node> {
        let mut result = Vec::new();
        collect_nodes_of_kind(self.root(), kind, &mut result);
        result
    }

    /// Find node at position (line, column are 1-based)
    fn node_at(&self, line: usize, column: usize) -> Option<&dyn Node>;
}

fn collect_nodes_of_kind<'a>(node: &'a dyn Node, kind: &str, result: &mut Vec<&'a dyn Node>) {
    if node.kind() == kind {
        result.push(node);
    }
    for child in node.children() {
        collect_nodes_of_kind(child, kind, result);
    }
}

/// Node kind classification
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeKind {
    /// Root document node
    Root,
    /// An element (XML element, YAML mapping, etc.)
    Element,
    /// An attribute or property
    Attribute,
    /// Text content
    Text,
    /// Comment
    Comment,
    /// Other/unknown
    Other,
}

/// An attribute on a node
#[derive(Debug, Clone)]
pub struct Attribute {
    pub name: String,
    pub value: String,
}

impl Attribute {
    pub fn new(name: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            value: value.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Mock node for testing
    struct MockNode {
        kind: String,
        text: String,
        attributes: Vec<Attribute>,
        children: Vec<MockNode>,
    }

    impl MockNode {
        fn new(kind: &str) -> Self {
            Self {
                kind: kind.to_string(),
                text: String::new(),
                attributes: Vec::new(),
                children: Vec::new(),
            }
        }

        fn with_attr(mut self, name: &str, value: &str) -> Self {
            self.attributes.push(Attribute::new(name, value));
            self
        }

        fn with_child(mut self, child: MockNode) -> Self {
            self.children.push(child);
            self
        }
    }

    impl Node for MockNode {
        fn kind(&self) -> &str {
            &self.kind
        }

        fn text(&self) -> &str {
            &self.text
        }

        fn range(&self) -> (usize, usize, usize, usize) {
            (1, 1, 1, 1)
        }

        fn parent(&self) -> Option<&dyn Node> {
            None
        }

        fn children(&self) -> Vec<&dyn Node> {
            self.children.iter().map(|c| c as &dyn Node).collect()
        }

        fn attribute(&self, name: &str) -> Option<&str> {
            self.attributes
                .iter()
                .find(|a| a.name == name)
                .map(|a| a.value.as_str())
        }

        fn attributes(&self) -> Vec<Attribute> {
            self.attributes.clone()
        }
    }

    #[test]
    fn test_node_attribute() {
        let node = MockNode::new("Component")
            .with_attr("Id", "C1")
            .with_attr("Guid", "*");

        assert_eq!(node.attribute("Id"), Some("C1"));
        assert_eq!(node.attribute("Guid"), Some("*"));
        assert_eq!(node.attribute("Missing"), None);
    }

    #[test]
    fn test_node_has_attribute() {
        let node = MockNode::new("Component").with_attr("Id", "C1");

        assert!(node.has_attribute("Id"));
        assert!(!node.has_attribute("Guid"));
    }

    #[test]
    fn test_node_children() {
        let node = MockNode::new("Component")
            .with_child(MockNode::new("File"))
            .with_child(MockNode::new("File"))
            .with_child(MockNode::new("RegistryKey"));

        assert_eq!(node.children().len(), 3);
        assert_eq!(node.count_children("File"), 2);
        assert_eq!(node.count_children("RegistryKey"), 1);
        assert!(node.has_child("File"));
        assert!(!node.has_child("Shortcut"));
    }

    #[test]
    fn test_node_descendants() {
        let node = MockNode::new("Package")
            .with_child(
                MockNode::new("Directory")
                    .with_child(MockNode::new("Component").with_child(MockNode::new("File"))),
            )
            .with_child(MockNode::new("Feature"));

        let descendants = node.descendants();
        assert_eq!(descendants.len(), 4); // Directory, Component, File, Feature
    }
}
