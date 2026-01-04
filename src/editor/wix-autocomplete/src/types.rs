//! Core types for wix-autocomplete

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Where the cursor is in the XML document
#[derive(Debug, Clone, PartialEq)]
pub enum CursorContext {
    /// Inside element content: <Parent>|</Parent>
    ElementContent {
        parent: String,
        siblings: Vec<String>,
    },
    /// After '<': <|
    ElementStart {
        parent: Option<String>,
    },
    /// Inside element tag, after element name: <Element |
    AttributeName {
        element: String,
        existing: Vec<String>,
    },
    /// After '=' or inside quotes: <Element Attr="|
    AttributeValue {
        element: String,
        attribute: String,
        partial: String,
    },
    /// Unknown/unparseable position
    Unknown,
}

/// The kind of completion item
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CompletionKind {
    Element,
    Attribute,
    Value,
    Snippet,
    Directory,
    Property,
}

/// A completion suggestion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionItem {
    /// Display text
    pub label: String,
    /// Type of completion
    pub kind: CompletionKind,
    /// Short description or type info
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
    /// Full documentation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub documentation: Option<String>,
    /// Text to insert (may differ from label)
    pub insert_text: String,
    /// Lower = higher priority (for sorting)
    #[serde(skip_serializing_if = "is_zero")]
    pub sort_priority: u32,
}

fn is_zero(n: &u32) -> bool {
    *n == 0
}

impl CompletionItem {
    /// Create a new completion item
    pub fn new(label: impl Into<String>, kind: CompletionKind) -> Self {
        let label = label.into();
        Self {
            insert_text: label.clone(),
            label,
            kind,
            detail: None,
            documentation: None,
            sort_priority: 100,
        }
    }

    /// Set the detail (type info)
    pub fn with_detail(mut self, detail: impl Into<String>) -> Self {
        self.detail = Some(detail.into());
        self
    }

    /// Set the documentation
    pub fn with_documentation(mut self, doc: impl Into<String>) -> Self {
        self.documentation = Some(doc.into());
        self
    }

    /// Set custom insert text
    pub fn with_insert_text(mut self, text: impl Into<String>) -> Self {
        self.insert_text = text.into();
        self
    }

    /// Set sort priority (lower = higher priority)
    pub fn with_priority(mut self, priority: u32) -> Self {
        self.sort_priority = priority;
        self
    }
}

/// WiX element definition loaded from wix-data
#[derive(Debug, Clone, Deserialize)]
pub struct ElementDef {
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub documentation: Option<String>,
    #[serde(default)]
    pub parents: Vec<String>,
    #[serde(default)]
    pub children: Vec<String>,
    #[serde(default)]
    pub attributes: HashMap<String, AttributeDef>,
}

/// Attribute definition
#[derive(Debug, Clone, Deserialize)]
pub struct AttributeDef {
    #[serde(rename = "type", default = "default_type")]
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

/// Deserialize default value as string, handling integers and other types
fn deserialize_default_value<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::Visitor;

    struct DefaultValueVisitor;

    impl<'de> Visitor<'de> for DefaultValueVisitor {
        type Value = Option<String>;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("a string, number, or null")
        }

        fn visit_none<E>(self) -> Result<Self::Value, E> {
            Ok(None)
        }

        fn visit_unit<E>(self) -> Result<Self::Value, E> {
            Ok(None)
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E> {
            Ok(Some(v.to_string()))
        }

        fn visit_string<E>(self, v: String) -> Result<Self::Value, E> {
            Ok(Some(v))
        }

        fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E> {
            Ok(Some(v.to_string()))
        }

        fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E> {
            Ok(Some(v.to_string()))
        }

        fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E> {
            Ok(Some(v.to_string()))
        }

        fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E> {
            Ok(Some(if v { "yes" } else { "no" }.to_string()))
        }
    }

    deserializer.deserialize_any(DefaultValueVisitor)
}

fn default_type() -> String {
    "string".to_string()
}

/// Snippet definition loaded from wix-data
#[derive(Debug, Clone, Deserialize)]
pub struct Snippet {
    pub name: String,
    pub prefix: String,
    pub description: String,
    pub body: Vec<String>,
}

impl Snippet {
    /// Get the full body as a single string
    pub fn body_text(&self) -> String {
        self.body.join("\n")
    }
}

/// Keywords loaded from wix-data
#[derive(Debug, Clone, Default, Deserialize)]
pub struct Keywords {
    #[serde(default)]
    pub elements: Vec<String>,
    #[serde(rename = "standardDirectories", default)]
    pub standard_directories: Vec<String>,
    #[serde(rename = "builtinProperties", default)]
    pub builtin_properties: Vec<String>,
    #[serde(rename = "preprocessorDirectives", default)]
    pub preprocessor_directives: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_completion_item_builder() {
        let item = CompletionItem::new("Component", CompletionKind::Element)
            .with_detail("WiX Component")
            .with_documentation("Defines a component")
            .with_priority(10);

        assert_eq!(item.label, "Component");
        assert_eq!(item.kind, CompletionKind::Element);
        assert_eq!(item.detail, Some("WiX Component".to_string()));
        assert_eq!(item.sort_priority, 10);
    }

    #[test]
    fn test_completion_item_with_insert_text() {
        let item = CompletionItem::new("comp", CompletionKind::Snippet)
            .with_insert_text("<Component>$0</Component>");

        assert_eq!(item.label, "comp");
        assert_eq!(item.insert_text, "<Component>$0</Component>");
    }

    #[test]
    fn test_completion_item_serialization() {
        let item = CompletionItem::new("File", CompletionKind::Element)
            .with_detail("File element");

        let json = serde_json::to_string(&item).unwrap();
        assert!(json.contains("\"label\":\"File\""));
        assert!(json.contains("\"kind\":\"element\""));
    }

    #[test]
    fn test_cursor_context_equality() {
        let ctx1 = CursorContext::ElementStart { parent: Some("Package".to_string()) };
        let ctx2 = CursorContext::ElementStart { parent: Some("Package".to_string()) };
        assert_eq!(ctx1, ctx2);
    }

    #[test]
    fn test_cursor_context_variants() {
        let ctx = CursorContext::ElementContent {
            parent: "Package".to_string(),
            siblings: vec!["Directory".to_string()],
        };
        assert!(matches!(ctx, CursorContext::ElementContent { .. }));

        let ctx = CursorContext::AttributeName {
            element: "Component".to_string(),
            existing: vec!["Guid".to_string()],
        };
        assert!(matches!(ctx, CursorContext::AttributeName { .. }));

        let ctx = CursorContext::AttributeValue {
            element: "Feature".to_string(),
            attribute: "Display".to_string(),
            partial: "exp".to_string(),
        };
        assert!(matches!(ctx, CursorContext::AttributeValue { .. }));

        let ctx = CursorContext::Unknown;
        assert!(matches!(ctx, CursorContext::Unknown));
    }

    #[test]
    fn test_attribute_def_deserialize_string_default() {
        let json = r#"{"type": "string", "default": "hello"}"#;
        let attr: AttributeDef = serde_json::from_str(json).unwrap();
        assert_eq!(attr.default, Some("hello".to_string()));
    }

    #[test]
    fn test_attribute_def_deserialize_integer_default() {
        let json = r#"{"type": "integer", "default": 42}"#;
        let attr: AttributeDef = serde_json::from_str(json).unwrap();
        assert_eq!(attr.default, Some("42".to_string()));
    }

    #[test]
    fn test_attribute_def_deserialize_float_default() {
        let json = r#"{"type": "number", "default": 3.14}"#;
        let attr: AttributeDef = serde_json::from_str(json).unwrap();
        assert_eq!(attr.default, Some("3.14".to_string()));
    }

    #[test]
    fn test_attribute_def_deserialize_bool_default() {
        let json = r#"{"type": "yesno", "default": true}"#;
        let attr: AttributeDef = serde_json::from_str(json).unwrap();
        assert_eq!(attr.default, Some("yes".to_string()));

        let json = r#"{"type": "yesno", "default": false}"#;
        let attr: AttributeDef = serde_json::from_str(json).unwrap();
        assert_eq!(attr.default, Some("no".to_string()));
    }

    #[test]
    fn test_attribute_def_deserialize_null_default() {
        let json = r#"{"type": "string", "default": null}"#;
        let attr: AttributeDef = serde_json::from_str(json).unwrap();
        assert_eq!(attr.default, None);
    }

    #[test]
    fn test_attribute_def_deserialize_no_default() {
        let json = r#"{"type": "string"}"#;
        let attr: AttributeDef = serde_json::from_str(json).unwrap();
        assert_eq!(attr.default, None);
    }

    #[test]
    fn test_attribute_def_with_values() {
        let json = r#"{"type": "enum", "values": ["a", "b", "c"]}"#;
        let attr: AttributeDef = serde_json::from_str(json).unwrap();
        assert_eq!(attr.values, Some(vec!["a".to_string(), "b".to_string(), "c".to_string()]));
    }

    #[test]
    fn test_element_def_deserialization() {
        let json = r#"{
            "name": "Component",
            "description": "A component",
            "parents": ["Package"],
            "children": ["File"],
            "attributes": {
                "Guid": {"type": "guid", "required": true}
            }
        }"#;
        let elem: ElementDef = serde_json::from_str(json).unwrap();
        assert_eq!(elem.name, "Component");
        assert_eq!(elem.parents, vec!["Package"]);
        assert_eq!(elem.children, vec!["File"]);
        assert!(elem.attributes.contains_key("Guid"));
    }

    #[test]
    fn test_snippet_body_text() {
        let snippet = Snippet {
            name: "Test".to_string(),
            prefix: "test".to_string(),
            description: "A test snippet".to_string(),
            body: vec!["line1".to_string(), "line2".to_string(), "line3".to_string()],
        };
        assert_eq!(snippet.body_text(), "line1\nline2\nline3");
    }

    #[test]
    fn test_keywords_deserialization() {
        let json = r#"{
            "elements": ["Package", "Component"],
            "standardDirectories": ["ProgramFilesFolder"],
            "builtinProperties": ["ProductName"],
            "preprocessorDirectives": ["if", "endif"]
        }"#;
        let keywords: Keywords = serde_json::from_str(json).unwrap();
        assert_eq!(keywords.elements, vec!["Package", "Component"]);
        assert_eq!(keywords.standard_directories, vec!["ProgramFilesFolder"]);
        assert_eq!(keywords.builtin_properties, vec!["ProductName"]);
        assert_eq!(keywords.preprocessor_directives, vec!["if", "endif"]);
    }

    #[test]
    fn test_keywords_default() {
        let keywords = Keywords::default();
        assert!(keywords.elements.is_empty());
        assert!(keywords.standard_directories.is_empty());
    }

    #[test]
    fn test_completion_kind_serialization() {
        let kinds = [
            (CompletionKind::Element, "element"),
            (CompletionKind::Attribute, "attribute"),
            (CompletionKind::Value, "value"),
            (CompletionKind::Snippet, "snippet"),
            (CompletionKind::Directory, "directory"),
            (CompletionKind::Property, "property"),
        ];

        for (kind, expected) in kinds {
            let item = CompletionItem::new("test", kind);
            let json = serde_json::to_string(&item).unwrap();
            assert!(json.contains(&format!("\"kind\":\"{}\"", expected)));
        }
    }
}
