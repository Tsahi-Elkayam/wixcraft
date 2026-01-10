//! Core types for wintellisense

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Position in a document (1-based)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Position {
    pub line: u32,
    pub column: u32,
}

impl Position {
    pub fn new(line: u32, column: u32) -> Self {
        Self { line, column }
    }
}

/// Location in a file
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Location {
    pub path: PathBuf,
    pub start: Position,
    pub end: Position,
}

impl Location {
    pub fn new(path: PathBuf, start: Position, end: Position) -> Self {
        Self { path, start, end }
    }

    pub fn point(path: PathBuf, line: u32, column: u32) -> Self {
        let pos = Position::new(line, column);
        Self {
            path,
            start: pos,
            end: pos,
        }
    }
}

/// Enum-based cursor context for type-safe context handling
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContextKind {
    /// Cursor is at element start position (after `<`)
    ElementStart {
        /// Parent element name
        parent: Option<String>,
    },

    /// Cursor is in element content (between tags)
    ElementContent {
        /// Parent element name
        parent: String,
        /// Sibling elements already present
        siblings: Vec<String>,
    },

    /// Cursor is in attribute name position
    AttributeName {
        /// Element the attribute belongs to
        element: String,
        /// Attributes already present
        existing: Vec<String>,
    },

    /// Cursor is in attribute value position
    AttributeValue {
        /// Element the attribute belongs to
        element: String,
        /// Attribute name
        attribute: String,
        /// Partial value already typed
        partial: String,
    },

    /// Unknown/unsupported context
    Unknown,
}

impl Default for ContextKind {
    fn default() -> Self {
        Self::Unknown
    }
}

/// Cursor context for completion/hover/definition
#[derive(Debug, Clone, Default)]
pub struct CursorContext {
    /// Parent element name
    pub parent_element: Option<String>,

    /// Current element name (if on/in an element tag)
    pub current_element: Option<String>,

    /// Current attribute name (if completing value)
    pub current_attribute: Option<String>,

    /// In opening tag (between < and >)
    pub in_opening_tag: bool,

    /// In attribute value (after =")
    pub in_attribute_value: bool,

    /// In element content (between tags)
    pub in_element_content: bool,

    /// Partial text already typed
    pub prefix: String,

    /// Line number (1-based)
    pub line: u32,

    /// Column number (1-based)
    pub column: u32,

    /// Current word under cursor (for hover/definition)
    pub word_at_cursor: Option<String>,

    /// Existing attributes on current element
    pub existing_attributes: Vec<String>,
}

impl CursorContext {
    pub fn should_suggest_elements(&self) -> bool {
        self.in_element_content
    }

    pub fn should_suggest_attributes(&self) -> bool {
        self.in_opening_tag && !self.in_attribute_value && self.current_element.is_some()
    }

    pub fn should_suggest_values(&self) -> bool {
        self.in_attribute_value && self.current_attribute.is_some()
    }

    /// Convert to enum-based context kind for pattern matching
    pub fn kind(&self) -> ContextKind {
        if self.in_attribute_value {
            if let (Some(element), Some(attribute)) =
                (&self.current_element, &self.current_attribute)
            {
                return ContextKind::AttributeValue {
                    element: element.clone(),
                    attribute: attribute.clone(),
                    partial: self.prefix.clone(),
                };
            }
        }

        if self.in_opening_tag {
            if let Some(element) = &self.current_element {
                return ContextKind::AttributeName {
                    element: element.clone(),
                    existing: self.existing_attributes.clone(),
                };
            }
        }

        if self.in_element_content {
            // Check if we're at element start (just typed <)
            if self.current_element.is_none() && !self.prefix.is_empty() {
                return ContextKind::ElementStart {
                    parent: self.parent_element.clone(),
                };
            }

            if let Some(parent) = &self.parent_element {
                return ContextKind::ElementContent {
                    parent: parent.clone(),
                    siblings: Vec::new(),
                };
            }

            return ContextKind::ElementStart {
                parent: self.parent_element.clone(),
            };
        }

        ContextKind::Unknown
    }

    /// Create context for element start
    pub fn element_start(parent: Option<String>) -> Self {
        Self {
            parent_element: parent,
            in_element_content: true,
            ..Default::default()
        }
    }

    /// Create context for element content
    pub fn element_content(parent: String) -> Self {
        Self {
            parent_element: Some(parent),
            in_element_content: true,
            ..Default::default()
        }
    }

    /// Create context for attribute name
    pub fn attribute_name(element: String, existing: Vec<String>) -> Self {
        Self {
            current_element: Some(element),
            in_opening_tag: true,
            existing_attributes: existing,
            ..Default::default()
        }
    }

    /// Create context for attribute value
    pub fn attribute_value(element: String, attribute: String, partial: String) -> Self {
        Self {
            current_element: Some(element),
            current_attribute: Some(attribute),
            in_attribute_value: true,
            prefix: partial,
            ..Default::default()
        }
    }
}

// ============================================================================
// Completion Types
// ============================================================================

/// Kind of completion item
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CompletionKind {
    Element,
    Attribute,
    Value,
    Snippet,
    Reference,
    Keyword,
    Word,
}

/// A completion item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionItem {
    /// Label shown in completion list
    pub label: String,

    /// Kind of completion
    pub kind: CompletionKind,

    /// Text to insert
    pub insert_text: String,

    /// Whether insert_text is a snippet
    #[serde(default)]
    pub is_snippet: bool,

    /// Short detail text
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,

    /// Full documentation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub documentation: Option<String>,

    /// Sort priority (lower = higher)
    #[serde(default = "default_priority")]
    pub sort_priority: u32,

    /// Is this a required item
    #[serde(default)]
    pub required: bool,
}

fn default_priority() -> u32 {
    50
}

impl CompletionItem {
    pub fn new(label: impl Into<String>, kind: CompletionKind) -> Self {
        let label = label.into();
        Self {
            insert_text: label.clone(),
            label,
            kind,
            is_snippet: false,
            detail: None,
            documentation: None,
            sort_priority: 50,
            required: false,
        }
    }

    pub fn with_insert_text(mut self, text: impl Into<String>) -> Self {
        self.insert_text = text.into();
        self
    }

    pub fn with_detail(mut self, detail: impl Into<String>) -> Self {
        self.detail = Some(detail.into());
        self
    }

    pub fn with_documentation(mut self, doc: impl Into<String>) -> Self {
        self.documentation = Some(doc.into());
        self
    }

    pub fn with_priority(mut self, priority: u32) -> Self {
        self.sort_priority = priority;
        self
    }

    pub fn with_required(mut self, required: bool) -> Self {
        self.required = required;
        if required {
            self.sort_priority = 10;
        }
        self
    }

    pub fn as_snippet(mut self) -> Self {
        self.is_snippet = true;
        self
    }
}

/// Result of completion request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionResult {
    pub items: Vec<CompletionItem>,
    #[serde(default)]
    pub is_incomplete: bool,
}

impl CompletionResult {
    pub fn new(items: Vec<CompletionItem>) -> Self {
        Self {
            items,
            is_incomplete: false,
        }
    }

    pub fn empty() -> Self {
        Self {
            items: Vec::new(),
            is_incomplete: false,
        }
    }
}

// ============================================================================
// Definition Types
// ============================================================================

/// A definition location
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Definition {
    /// Location of the definition
    pub location: Location,

    /// Name of the symbol
    pub name: String,

    /// Kind of symbol (Component, Directory, etc.)
    pub kind: String,

    /// Preview/context text
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preview: Option<String>,
}

/// Result of go-to-definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefinitionResult {
    pub definitions: Vec<Definition>,
}

impl DefinitionResult {
    pub fn new(definitions: Vec<Definition>) -> Self {
        Self { definitions }
    }

    pub fn empty() -> Self {
        Self {
            definitions: Vec::new(),
        }
    }

    pub fn single(def: Definition) -> Self {
        Self {
            definitions: vec![def],
        }
    }
}

// ============================================================================
// Hover Types
// ============================================================================

/// Hover information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HoverInfo {
    /// Main content (markdown)
    pub content: String,

    /// Optional range this hover applies to
    #[serde(skip_serializing_if = "Option::is_none")]
    pub range: Option<(Position, Position)>,
}

impl HoverInfo {
    pub fn new(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            range: None,
        }
    }

    pub fn with_range(mut self, start: Position, end: Position) -> Self {
        self.range = Some((start, end));
        self
    }
}

/// Result of hover request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HoverResult {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub info: Option<HoverInfo>,
}

impl HoverResult {
    pub fn some(info: HoverInfo) -> Self {
        Self { info: Some(info) }
    }

    pub fn none() -> Self {
        Self { info: None }
    }
}

// ============================================================================
// Schema Types (from loader)
// ============================================================================

/// Element definition
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ElementDef {
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub documentation: Option<String>,
    #[serde(default)]
    pub parents: Vec<String>,
    #[serde(default)]
    pub children: Vec<String>,
    #[serde(default)]
    pub attributes: std::collections::HashMap<String, AttributeDef>,
}

/// Attribute definition
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AttributeDef {
    #[serde(rename = "type", default)]
    pub attr_type: String,
    #[serde(default)]
    pub required: bool,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub default: Option<String>,
    #[serde(default)]
    pub values: Option<Vec<String>>,
}

/// Snippet definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snippet {
    pub name: String,
    pub prefix: String,
    pub description: String,
    pub body: Vec<String>,
}

impl Snippet {
    pub fn body_text(&self) -> String {
        self.body.join("\n")
    }
}

/// Keywords (standard directories, properties, etc.)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Keywords {
    #[serde(default, rename = "standardDirectories")]
    pub standard_directories: Vec<String>,
    #[serde(default, rename = "builtinProperties")]
    pub builtin_properties: Vec<String>,
    #[serde(default)]
    pub elements: Vec<String>,
    #[serde(default, rename = "preprocessorDirectives")]
    pub preprocessor_directives: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_completion_item() {
        let item = CompletionItem::new("Component", CompletionKind::Element)
            .with_detail("WiX Component")
            .with_priority(10)
            .as_snippet();

        assert_eq!(item.label, "Component");
        assert!(item.is_snippet);
        assert_eq!(item.sort_priority, 10);
    }

    #[test]
    fn test_cursor_context_suggestions() {
        let mut ctx = CursorContext::default();

        ctx.in_element_content = true;
        assert!(ctx.should_suggest_elements());

        ctx.in_element_content = false;
        ctx.in_opening_tag = true;
        ctx.current_element = Some("Component".to_string());
        assert!(ctx.should_suggest_attributes());

        ctx.in_attribute_value = true;
        ctx.current_attribute = Some("Guid".to_string());
        assert!(ctx.should_suggest_values());
    }

    #[test]
    fn test_position() {
        let pos = Position::new(10, 5);
        assert_eq!(pos.line, 10);
        assert_eq!(pos.column, 5);
    }

    #[test]
    fn test_location() {
        let loc = Location::point(PathBuf::from("test.wxs"), 10, 5);
        assert_eq!(loc.start, loc.end);
    }
}
