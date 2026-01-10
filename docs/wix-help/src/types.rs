//! Core types for wix-help

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// An element definition from wix-data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElementDef {
    /// Element name
    pub name: String,
    /// Namespace (wix, bal, etc.)
    #[serde(default)]
    pub namespace: String,
    /// WiX version when introduced
    #[serde(default)]
    pub since: String,
    /// Element description
    #[serde(default)]
    pub description: String,
    /// Documentation URL
    #[serde(default)]
    pub documentation: String,
    /// Valid parent elements
    #[serde(default)]
    pub parents: Vec<String>,
    /// Valid child elements
    #[serde(default)]
    pub children: Vec<String>,
    /// Attributes
    #[serde(default)]
    pub attributes: HashMap<String, AttributeDef>,
    /// MSI tables this element writes to
    #[serde(default, rename = "msiTables")]
    pub msi_tables: Vec<String>,
    /// Related rules
    #[serde(default)]
    pub rules: Vec<String>,
    /// Examples
    #[serde(default)]
    pub examples: Vec<Example>,
}

impl ElementDef {
    /// Get required attributes
    pub fn required_attributes(&self) -> Vec<(&String, &AttributeDef)> {
        self.attributes
            .iter()
            .filter(|(_, attr)| attr.required.unwrap_or(false))
            .collect()
    }

    /// Get optional attributes
    pub fn optional_attributes(&self) -> Vec<(&String, &AttributeDef)> {
        self.attributes
            .iter()
            .filter(|(_, attr)| !attr.required.unwrap_or(false))
            .collect()
    }
}

/// An attribute definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttributeDef {
    /// Attribute type (string, identifier, guid, yesno, enum)
    #[serde(rename = "type")]
    pub attr_type: String,
    /// Whether required
    #[serde(default)]
    pub required: Option<bool>,
    /// Default value (can be string or number)
    #[serde(default, deserialize_with = "deserialize_default_value")]
    pub default: Option<String>,
    /// Description
    #[serde(default)]
    pub description: String,
    /// Enum values (for enum type)
    #[serde(default)]
    pub values: Vec<String>,
}

/// Deserialize default value which can be string, number, or null
fn deserialize_default_value<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::Error;

    let value: Option<serde_json::Value> = Option::deserialize(deserializer)?;

    match value {
        None => Ok(None),
        Some(serde_json::Value::String(s)) => Ok(Some(s)),
        Some(serde_json::Value::Number(n)) => Ok(Some(n.to_string())),
        Some(serde_json::Value::Bool(b)) => Ok(Some(b.to_string())),
        Some(serde_json::Value::Null) => Ok(None),
        Some(other) => Err(D::Error::custom(format!(
            "unexpected type for default: {:?}",
            other
        ))),
    }
}

/// An example
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Example {
    /// Example description
    #[serde(default)]
    pub description: String,
    /// Example code
    #[serde(default)]
    pub code: String,
}

/// A snippet definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snippet {
    /// Snippet name
    pub name: String,
    /// Snippet prefix (trigger)
    pub prefix: String,
    /// Description
    #[serde(default)]
    pub description: String,
    /// Body lines
    #[serde(default)]
    pub body: Vec<String>,
}

/// Snippets file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnippetsFile {
    pub snippets: Vec<Snippet>,
}

/// A WiX error definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WixError {
    /// Error code
    pub code: String,
    /// Severity
    pub severity: String,
    /// Error message template
    #[serde(default)]
    pub message: String,
    /// Description
    #[serde(default)]
    pub description: String,
    /// Resolution steps
    #[serde(default)]
    pub resolution: String,
}

/// An ICE error definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IceError {
    /// Error code
    pub code: String,
    /// Severity
    pub severity: String,
    /// Description
    #[serde(default)]
    pub description: String,
    /// Related tables
    #[serde(default)]
    pub tables: Vec<String>,
    /// Resolution steps
    #[serde(default)]
    pub resolution: String,
}

/// Errors file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorsFile {
    #[serde(default)]
    pub errors: Vec<WixError>,
    #[serde(default, rename = "iceErrors")]
    pub ice_errors: Vec<IceError>,
}

/// A lint rule definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LintRule {
    /// Rule ID
    pub id: String,
    /// Rule name
    pub name: String,
    /// Description
    #[serde(default)]
    pub description: String,
    /// Severity
    #[serde(default)]
    pub severity: String,
    /// Element this rule applies to
    #[serde(default)]
    pub element: String,
    /// Error message
    #[serde(default)]
    pub message: String,
}

/// Rules file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RulesFile {
    pub rules: Vec<LintRule>,
}

/// Help topic type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HelpTopic {
    Element,
    Error,
    Snippet,
    Rule,
}

/// Output format
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OutputFormat {
    #[default]
    Text,
    Json,
    Markdown,
}

impl std::str::FromStr for OutputFormat {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "text" => Ok(Self::Text),
            "json" => Ok(Self::Json),
            "markdown" | "md" => Ok(Self::Markdown),
            _ => Err(format!("Unknown format: {}", s)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_element_def_required_attributes() {
        let mut attrs = HashMap::new();
        attrs.insert(
            "Id".to_string(),
            AttributeDef {
                attr_type: "identifier".to_string(),
                required: Some(true),
                default: None,
                description: "Identifier".to_string(),
                values: vec![],
            },
        );
        attrs.insert(
            "Name".to_string(),
            AttributeDef {
                attr_type: "string".to_string(),
                required: Some(false),
                default: None,
                description: "Name".to_string(),
                values: vec![],
            },
        );

        let elem = ElementDef {
            name: "Test".to_string(),
            namespace: "wix".to_string(),
            since: "v4".to_string(),
            description: "Test element".to_string(),
            documentation: String::new(),
            parents: vec![],
            children: vec![],
            attributes: attrs,
            msi_tables: vec![],
            rules: vec![],
            examples: vec![],
        };

        let required = elem.required_attributes();
        assert_eq!(required.len(), 1);
        assert_eq!(required[0].0, "Id");

        let optional = elem.optional_attributes();
        assert_eq!(optional.len(), 1);
        assert_eq!(optional[0].0, "Name");
    }

    #[test]
    fn test_output_format_from_str() {
        assert_eq!("text".parse::<OutputFormat>().unwrap(), OutputFormat::Text);
        assert_eq!("json".parse::<OutputFormat>().unwrap(), OutputFormat::Json);
        assert_eq!(
            "markdown".parse::<OutputFormat>().unwrap(),
            OutputFormat::Markdown
        );
        assert_eq!("md".parse::<OutputFormat>().unwrap(), OutputFormat::Markdown);
        assert!("unknown".parse::<OutputFormat>().is_err());
    }

    #[test]
    fn test_attribute_def() {
        let attr = AttributeDef {
            attr_type: "enum".to_string(),
            required: Some(true),
            default: Some("auto".to_string()),
            description: "Start type".to_string(),
            values: vec!["auto".to_string(), "demand".to_string()],
        };
        assert!(attr.required.unwrap());
        assert_eq!(attr.default.unwrap(), "auto");
        assert_eq!(attr.values.len(), 2);
    }

    #[test]
    fn test_example() {
        let ex = Example {
            description: "Basic example".to_string(),
            code: "<Component />".to_string(),
        };
        assert!(!ex.description.is_empty());
        assert!(!ex.code.is_empty());
    }

    #[test]
    fn test_snippet() {
        let snippet = Snippet {
            name: "component".to_string(),
            prefix: "comp".to_string(),
            description: "Create a component".to_string(),
            body: vec!["<Component>".to_string(), "</Component>".to_string()],
        };
        assert_eq!(snippet.prefix, "comp");
        assert_eq!(snippet.body.len(), 2);
    }

    #[test]
    fn test_wix_error() {
        let err = WixError {
            code: "WIX0001".to_string(),
            severity: "error".to_string(),
            message: "Error message".to_string(),
            description: "Description".to_string(),
            resolution: "Fix it".to_string(),
        };
        assert!(err.code.starts_with("WIX"));
    }

    #[test]
    fn test_ice_error() {
        let err = IceError {
            code: "ICE03".to_string(),
            severity: "error".to_string(),
            description: "Schema validation".to_string(),
            tables: vec!["_Validation".to_string()],
            resolution: "Fix schema".to_string(),
        };
        assert!(err.code.starts_with("ICE"));
        assert!(!err.tables.is_empty());
    }

    #[test]
    fn test_lint_rule() {
        let rule = LintRule {
            id: "component-requires-guid".to_string(),
            name: "Component requires GUID".to_string(),
            description: "Every component must have a GUID".to_string(),
            severity: "error".to_string(),
            element: "Component".to_string(),
            message: "Missing GUID".to_string(),
        };
        assert!(!rule.id.is_empty());
    }

    #[test]
    fn test_help_topic() {
        assert_eq!(HelpTopic::Element, HelpTopic::Element);
        assert_ne!(HelpTopic::Element, HelpTopic::Error);
    }

    #[test]
    fn test_snippets_file_deserialize() {
        let json = r#"{"snippets": [{"name": "test", "prefix": "t", "description": "Test", "body": ["<Test />"]}]}"#;
        let file: SnippetsFile = serde_json::from_str(json).unwrap();
        assert_eq!(file.snippets.len(), 1);
    }

    #[test]
    fn test_errors_file_deserialize() {
        let json = r#"{"errors": [{"code": "WIX0001", "severity": "error"}], "iceErrors": []}"#;
        let file: ErrorsFile = serde_json::from_str(json).unwrap();
        assert_eq!(file.errors.len(), 1);
    }

    #[test]
    fn test_rules_file_deserialize() {
        let json = r#"{"rules": [{"id": "test", "name": "Test Rule"}]}"#;
        let file: RulesFile = serde_json::from_str(json).unwrap();
        assert_eq!(file.rules.len(), 1);
    }

    #[test]
    fn test_element_def_default_values() {
        let json = r#"{"name": "Test"}"#;
        let elem: ElementDef = serde_json::from_str(json).unwrap();
        assert_eq!(elem.name, "Test");
        assert!(elem.namespace.is_empty());
        assert!(elem.parents.is_empty());
    }

    #[test]
    fn test_output_format_default() {
        let format = OutputFormat::default();
        assert_eq!(format, OutputFormat::Text);
    }

    #[test]
    fn test_attribute_def_with_integer_default() {
        let json = r#"{"type": "integer", "default": 500, "description": "Version"}"#;
        let attr: AttributeDef = serde_json::from_str(json).unwrap();
        assert_eq!(attr.default, Some("500".to_string()));
    }

    #[test]
    fn test_attribute_def_with_string_default() {
        let json = r#"{"type": "string", "default": "yes", "description": "Compressed"}"#;
        let attr: AttributeDef = serde_json::from_str(json).unwrap();
        assert_eq!(attr.default, Some("yes".to_string()));
    }

    #[test]
    fn test_attribute_def_with_bool_default() {
        let json = r#"{"type": "yesno", "default": true, "description": "Enabled"}"#;
        let attr: AttributeDef = serde_json::from_str(json).unwrap();
        assert_eq!(attr.default, Some("true".to_string()));
    }

    #[test]
    fn test_attribute_def_with_null_default() {
        let json = r#"{"type": "string", "default": null, "description": "Optional"}"#;
        let attr: AttributeDef = serde_json::from_str(json).unwrap();
        assert!(attr.default.is_none());
    }

    #[test]
    fn test_attribute_def_without_default() {
        let json = r#"{"type": "string", "description": "Optional"}"#;
        let attr: AttributeDef = serde_json::from_str(json).unwrap();
        assert!(attr.default.is_none());
    }

    #[test]
    fn test_attribute_def_with_array_default_fails() {
        let json = r#"{"type": "string", "default": ["a", "b"], "description": "Invalid"}"#;
        let result: Result<AttributeDef, _> = serde_json::from_str(json);
        assert!(result.is_err());
    }
}
