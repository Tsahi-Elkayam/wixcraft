//! Hover types

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// What the cursor is hovering over
#[derive(Debug, Clone, PartialEq)]
pub enum HoverTarget {
    /// Hovering over element name: <Component ...>
    Element { name: String, range: Range },
    /// Hovering over attribute name: <File Source="...">
    AttributeName {
        element: String,
        attribute: String,
        range: Range,
    },
    /// Hovering over attribute value: <Directory Id="ProgramFilesFolder">
    AttributeValue {
        element: String,
        attribute: String,
        value: String,
        range: Range,
    },
    /// Nothing to show hover for
    None,
}

/// Source range for highlighting
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Range {
    pub start_line: u32,
    pub start_col: u32,
    pub end_line: u32,
    pub end_col: u32,
}

/// Hover information result
#[derive(Debug, Clone, Serialize)]
pub struct HoverInfo {
    /// Markdown-formatted content
    pub contents: String,
    /// Range in source that this hover applies to
    #[serde(skip_serializing_if = "Option::is_none")]
    pub range: Option<Range>,
}

impl HoverInfo {
    pub fn new(contents: String) -> Self {
        Self {
            contents,
            range: None,
        }
    }

    pub fn with_range(mut self, range: Range) -> Self {
        self.range = Some(range);
        self
    }
}

/// Element definition from wix-data
#[derive(Debug, Clone, Deserialize)]
pub struct ElementDef {
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub documentation: Option<String>,
    #[serde(default)]
    pub since: Option<String>,
    #[serde(default)]
    pub parents: Vec<String>,
    #[serde(default)]
    pub children: Vec<String>,
    #[serde(default)]
    pub attributes: HashMap<String, AttributeDef>,
}

/// Attribute definition from wix-data
#[derive(Debug, Clone, Deserialize)]
pub struct AttributeDef {
    #[serde(rename = "type", default)]
    pub attr_type: String,
    #[serde(default)]
    pub required: bool,
    #[serde(default)]
    pub description: String,
    #[serde(default, deserialize_with = "deserialize_default_value")]
    pub default: Option<String>,
    #[serde(default)]
    pub values: Option<Vec<String>>,
}

/// Keywords from wix-data
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Keywords {
    #[serde(default)]
    pub elements: Vec<String>,
    #[serde(default)]
    pub standard_directories: Vec<String>,
    #[serde(default)]
    pub builtin_properties: Vec<String>,
    #[serde(default)]
    pub preprocessor_directives: Vec<String>,
}

/// Custom deserializer that converts various JSON types to String
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
        assert_eq!(attr.values, Some(vec!["expand".to_string(), "collapse".to_string(), "hidden".to_string()]));
    }
}
