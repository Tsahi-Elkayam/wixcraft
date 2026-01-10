//! Plugin system for format-specific parsing and rule evaluation

use crate::diagnostic::Location;
use crate::rule::Rule;
use std::collections::HashMap;
use std::path::Path;
use thiserror::Error;

/// Error during parsing
#[derive(Debug, Error)]
pub enum ParseError {
    #[error("XML parse error at line {line}: {message}")]
    Xml { line: usize, message: String },

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Invalid document: {0}")]
    Invalid(String),
}

/// A node in the parsed document tree
pub trait Node: Send + Sync {
    /// Node type/kind (e.g., "element", "text", "comment")
    fn kind(&self) -> &str;

    /// Node name (e.g., "Package", "Component")
    fn name(&self) -> &str;

    /// Get attribute/property value
    fn get(&self, key: &str) -> Option<&str>;

    /// Get all attributes/properties
    fn attributes(&self) -> &HashMap<String, String>;

    /// Get child nodes
    fn children(&self) -> Vec<&dyn Node>;

    /// Get parent node (if any)
    fn parent(&self) -> Option<&dyn Node>;

    /// Source location
    fn location(&self) -> Location;

    /// Get text content (for text nodes)
    fn text(&self) -> Option<&str> {
        None
    }
}

/// A parsed document
pub trait Document: Send + Sync {
    /// Get the root node
    fn root(&self) -> Option<&dyn Node>;

    /// Iterate over all nodes in the document (depth-first)
    fn iter(&self) -> Box<dyn Iterator<Item = &dyn Node> + '_>;

    /// Get source line at line number (1-based)
    fn get_source_line(&self, line: usize) -> Option<&str>;

    /// Check if a rule is disabled at a specific line (inline comments)
    fn is_rule_disabled(&self, rule_id: &str, line: usize) -> bool;

    /// Check if a rule is disabled for the entire file
    fn is_rule_disabled_for_file(&self, rule_id: &str) -> bool;
}

/// Plugin trait for format-specific parsing and linting
pub trait Plugin: Send + Sync {
    /// Plugin identifier (e.g., "wix", "json", "yaml")
    fn id(&self) -> &str;

    /// Plugin version
    fn version(&self) -> &str;

    /// Human-readable description
    fn description(&self) -> &str;

    /// File extensions this plugin handles (without dot, e.g., "wxs", "wxi")
    fn extensions(&self) -> &[&str];

    /// MIME types this plugin handles (optional)
    fn mime_types(&self) -> &[&str] {
        &[]
    }

    /// Parse file content into a document tree
    fn parse(&self, content: &str, path: &Path) -> Result<Box<dyn Document>, ParseError>;

    /// Get all rules provided by this plugin
    fn rules(&self) -> &[Rule];

    /// Load additional rules from a directory
    fn load_rules(&mut self, dir: &Path) -> Result<usize, RuleLoadError>;
}

/// Error loading rules
#[derive(Debug, Error)]
pub enum RuleLoadError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Parse error in {file}: {message}")]
    Parse { file: String, message: String },

    #[error("Invalid rule: {0}")]
    Invalid(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_error_display() {
        let err = ParseError::Xml {
            line: 10,
            message: "unexpected token".to_string(),
        };
        assert_eq!(
            format!("{}", err),
            "XML parse error at line 10: unexpected token"
        );
    }

    #[test]
    fn test_rule_load_error_display() {
        let err = RuleLoadError::Parse {
            file: "rules.yaml".to_string(),
            message: "invalid syntax".to_string(),
        };
        assert_eq!(
            format!("{}", err),
            "Parse error in rules.yaml: invalid syntax"
        );
    }
}
