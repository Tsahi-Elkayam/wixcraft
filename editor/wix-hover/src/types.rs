//! Core types for hover functionality.
//!
//! This module contains the data structures used throughout wix-hover:
//! - [`HoverTarget`] - What the cursor is hovering over
//! - [`HoverInfo`] - The hover result with formatted content
//! - [`Range`] - Source location for highlighting
//! - Schema types ([`ElementDef`], [`AttributeDef`], [`Keywords`])

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// What the cursor is hovering over in the source.
///
/// Used by [`detect_hover_target`](crate::detect_hover_target) to identify
/// the semantic element under the cursor.
#[derive(Debug, Clone, PartialEq)]
pub enum HoverTarget {
    /// Hovering over element name: `<Component ...>`
    Element {
        /// Element name (e.g., "Component")
        name: String,
        /// Source range of the element name
        range: Range,
    },

    /// Hovering over attribute name: `<File Source="...">`
    AttributeName {
        /// Parent element name
        element: String,
        /// Attribute name (e.g., "Source")
        attribute: String,
        /// Source range of the attribute name
        range: Range,
    },

    /// Hovering over attribute value: `<Directory Id="ProgramFilesFolder">`
    AttributeValue {
        /// Parent element name
        element: String,
        /// Attribute name
        attribute: String,
        /// The value being hovered
        value: String,
        /// Source range of the value
        range: Range,
    },

    /// Nothing to show hover for
    None,
}

/// Source range for highlighting.
///
/// All positions are 1-based (line 1, column 1 is the start).
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Range {
    pub start_line: u32,
    pub start_col: u32,
    pub end_line: u32,
    pub end_col: u32,
}

/// Hover information result.
///
/// Contains markdown-formatted content to display in a tooltip,
/// and optionally the source range to highlight.
#[derive(Debug, Clone, Serialize)]
pub struct HoverInfo {
    /// Markdown-formatted content
    pub contents: String,

    /// Range in source that this hover applies to
    #[serde(skip_serializing_if = "Option::is_none")]
    pub range: Option<Range>,
}

impl HoverInfo {
    /// Create new hover info with content only.
    pub fn new(contents: String) -> Self {
        Self {
            contents,
            range: None,
        }
    }

    /// Add a source range to the hover info.
    pub fn with_range(mut self, range: Range) -> Self {
        self.range = Some(range);
        self
    }
}

/// Element definition from wix-data schema.
#[derive(Debug, Clone, Deserialize)]
pub struct ElementDef {
    /// Element name (e.g., "Component")
    pub name: String,

    /// Short description
    #[serde(default)]
    pub description: String,

    /// URL to documentation
    #[serde(default)]
    pub documentation: Option<String>,

    /// Version when element was introduced (e.g., "v3")
    #[serde(default)]
    pub since: Option<String>,

    /// Valid parent elements
    #[serde(default)]
    pub parents: Vec<String>,

    /// Valid child elements
    #[serde(default)]
    pub children: Vec<String>,

    /// Attribute definitions
    #[serde(default)]
    pub attributes: HashMap<String, AttributeDef>,
}

/// Attribute definition from wix-data schema.
#[derive(Debug, Clone, Deserialize)]
pub struct AttributeDef {
    /// Attribute type (identifier, guid, yesno, enum, etc.)
    #[serde(rename = "type", default)]
    pub attr_type: String,

    /// Whether the attribute is required
    #[serde(default)]
    pub required: bool,

    /// Description of the attribute
    #[serde(default)]
    pub description: String,

    /// Default value (handles bool/int/string in JSON)
    #[serde(default, deserialize_with = "deserialize_default_value")]
    pub default: Option<String>,

    /// Allowed values for enum types
    #[serde(default)]
    pub values: Option<Vec<String>>,
}

/// Keywords from wix-data (directories, properties, etc.)
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Keywords {
    /// WiX element names
    #[serde(default)]
    pub elements: Vec<String>,

    /// Standard MSI directories (ProgramFilesFolder, etc.)
    #[serde(default)]
    pub standard_directories: Vec<String>,

    /// Built-in MSI properties (ProductName, etc.)
    #[serde(default)]
    pub builtin_properties: Vec<String>,

    /// Preprocessor directives (if, endif, etc.)
    #[serde(default)]
    pub preprocessor_directives: Vec<String>,
}

/// Custom deserializer that converts various JSON types to String.
fn deserialize_default_value<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::Error;

    let value: Option<serde_json::Value> = Option::deserialize(deserializer)?;

    match value {
        None => Ok(None),
        Some(v) => match v {
            serde_json::Value::String(s) => Ok(Some(s)),
            serde_json::Value::Number(n) => Ok(Some(n.to_string())),
            serde_json::Value::Bool(b) => Ok(Some(if b { "yes" } else { "no" }.to_string())),
            serde_json::Value::Null => Ok(None),
            _ => Err(D::Error::custom("unexpected type for default value")),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hover_info_new() {
        let info = HoverInfo::new("test".to_string());
        assert_eq!(info.contents, "test");
        assert!(info.range.is_none());
    }

    #[test]
    fn test_hover_info_with_range() {
        let range = Range {
            start_line: 1,
            start_col: 1,
            end_line: 1,
            end_col: 5,
        };
        let info = HoverInfo::new("test".to_string()).with_range(range.clone());
        assert_eq!(info.range, Some(range));
    }

    #[test]
    fn test_hover_target_variants() {
        let range = Range {
            start_line: 1,
            start_col: 1,
            end_line: 1,
            end_col: 10,
        };

        let element = HoverTarget::Element {
            name: "Component".to_string(),
            range: range.clone(),
        };
        assert!(matches!(element, HoverTarget::Element { .. }));

        let attr = HoverTarget::AttributeName {
            element: "File".to_string(),
            attribute: "Source".to_string(),
            range: range.clone(),
        };
        assert!(matches!(attr, HoverTarget::AttributeName { .. }));

        let value = HoverTarget::AttributeValue {
            element: "Directory".to_string(),
            attribute: "Id".to_string(),
            value: "ProgramFilesFolder".to_string(),
            range,
        };
        assert!(matches!(value, HoverTarget::AttributeValue { .. }));
    }

    #[test]
    fn test_deserialize_element_def() {
        let json = r#"{
            "name": "Test",
            "description": "Test element",
            "documentation": "https://example.com",
            "since": "v3",
            "parents": ["Parent"],
            "children": ["Child"],
            "attributes": {}
        }"#;

        let elem: ElementDef = serde_json::from_str(json).unwrap();
        assert_eq!(elem.name, "Test");
        assert_eq!(elem.since, Some("v3".to_string()));
    }

    #[test]
    fn test_deserialize_attribute_def() {
        let json = r#"{
            "type": "string",
            "required": true,
            "description": "Test attr"
        }"#;

        let attr: AttributeDef = serde_json::from_str(json).unwrap();
        assert_eq!(attr.attr_type, "string");
        assert!(attr.required);
    }

    #[test]
    fn test_deserialize_default_int() {
        let json = r#"{"type": "int", "default": 42}"#;
        let attr: AttributeDef = serde_json::from_str(json).unwrap();
        assert_eq!(attr.default, Some("42".to_string()));
    }

    #[test]
    fn test_deserialize_default_bool() {
        let json = r#"{"type": "bool", "default": true}"#;
        let attr: AttributeDef = serde_json::from_str(json).unwrap();
        assert_eq!(attr.default, Some("yes".to_string()));
    }

    #[test]
    fn test_deserialize_enum_values() {
        let json = r#"{
            "type": "enum",
            "values": ["expand", "collapse", "hidden"]
        }"#;

        let attr: AttributeDef = serde_json::from_str(json).unwrap();
        assert_eq!(
            attr.values,
            Some(vec![
                "expand".to_string(),
                "collapse".to_string(),
                "hidden".to_string()
            ])
        );
    }
}
