//! Data models for WiX Data Layer

use serde::{Deserialize, Serialize};

/// WiX Element definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Element {
    pub id: i64,
    pub name: String,
    pub namespace: String,
    pub since_version: Option<String>,
    pub deprecated_version: Option<String>,
    pub description: Option<String>,
    pub documentation_url: Option<String>,
    pub remarks: Option<String>,
    pub example: Option<String>,
}

/// Element attribute
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attribute {
    pub id: i64,
    pub element_id: i64,
    pub name: String,
    pub attr_type: AttributeType,
    pub required: bool,
    pub default_value: Option<String>,
    pub description: Option<String>,
    pub since_version: Option<String>,
    pub deprecated_version: Option<String>,
    pub enum_values: Vec<String>,
}

/// Attribute types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum AttributeType {
    String,
    Guid,
    YesNo,
    Integer,
    Enum,
    Version,
    Identifier,
    Path,
}

impl From<&str> for AttributeType {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "guid" => AttributeType::Guid,
            "yesno" => AttributeType::YesNo,
            "integer" | "int" => AttributeType::Integer,
            "enum" => AttributeType::Enum,
            "version" => AttributeType::Version,
            "identifier" | "id" => AttributeType::Identifier,
            "path" => AttributeType::Path,
            _ => AttributeType::String,
        }
    }
}

impl std::fmt::Display for AttributeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AttributeType::String => write!(f, "string"),
            AttributeType::Guid => write!(f, "guid"),
            AttributeType::YesNo => write!(f, "yesno"),
            AttributeType::Integer => write!(f, "integer"),
            AttributeType::Enum => write!(f, "enum"),
            AttributeType::Version => write!(f, "version"),
            AttributeType::Identifier => write!(f, "identifier"),
            AttributeType::Path => write!(f, "path"),
        }
    }
}

/// Lint rule definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rule {
    pub id: i64,
    pub rule_id: String,
    pub category: String,
    pub severity: Severity,
    pub name: String,
    pub description: Option<String>,
    pub rationale: Option<String>,
    pub fix_suggestion: Option<String>,
    pub enabled: bool,
    pub auto_fixable: bool,
    pub conditions: Vec<RuleCondition>,
    /// Expression condition for Winter linter (e.g., "name == \"Package\" && !attributes.UpgradeCode")
    pub condition: Option<String>,
    /// Target node kind: element, comment, text
    pub target_kind: Option<String>,
    /// Target element name pattern (e.g., "Package", "Component*")
    pub target_name: Option<String>,
    /// Comma-separated tags
    pub tags: Option<String>,
}

/// Rule condition for matching
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleCondition {
    pub id: i64,
    pub condition_type: String,
    pub target: String,
    pub operator: Option<String>,
    pub value: Option<String>,
}

/// Severity levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Error,
    Warning,
    Info,
}

impl From<&str> for Severity {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "error" => Severity::Error,
            "warning" | "warn" => Severity::Warning,
            _ => Severity::Info,
        }
    }
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Severity::Error => write!(f, "error"),
            Severity::Warning => write!(f, "warning"),
            Severity::Info => write!(f, "info"),
        }
    }
}

/// WiX error/warning code
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WixError {
    pub id: i64,
    pub code: String,
    pub severity: Severity,
    pub message_template: String,
    pub description: Option<String>,
    pub resolution: Option<String>,
    pub documentation_url: Option<String>,
}

/// ICE validation rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IceRule {
    pub id: i64,
    pub code: String,
    pub severity: Severity,
    pub description: Option<String>,
    pub resolution: Option<String>,
    pub tables_affected: Vec<String>,
    pub documentation_url: Option<String>,
}

/// MSI database table definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MsiTable {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
    pub required: bool,
    pub columns: Vec<MsiColumn>,
    pub documentation_url: Option<String>,
}

/// MSI table column
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MsiColumn {
    pub name: String,
    pub data_type: String,
    pub nullable: bool,
    pub primary_key: bool,
    pub description: Option<String>,
}

/// Standard Windows directory
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StandardDirectory {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
    pub windows_path: Option<String>,
    pub example: Option<String>,
}

/// Built-in MSI property
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuiltinProperty {
    pub id: i64,
    pub name: String,
    pub property_type: Option<String>,
    pub description: Option<String>,
    pub default_value: Option<String>,
    pub readonly: bool,
}

/// Preprocessor directive
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreprocessorDirective {
    pub id: i64,
    pub name: String,
    pub syntax: Option<String>,
    pub description: Option<String>,
    pub example: Option<String>,
}

/// Code snippet
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snippet {
    pub id: i64,
    pub prefix: String,
    pub name: String,
    pub description: Option<String>,
    pub body: String,
    pub scope: String,
}

/// Version migration change
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Migration {
    pub id: i64,
    pub from_version: String,
    pub to_version: String,
    pub change_type: String,
    pub old_value: Option<String>,
    pub new_value: Option<String>,
    pub notes: Option<String>,
}

/// WiX extension
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Extension {
    pub id: i64,
    pub name: String,
    pub namespace: String,
    pub prefix: String,
    pub description: Option<String>,
    pub xsd_url: Option<String>,
}

/// Database statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbStats {
    pub elements: usize,
    pub attributes: usize,
    pub rules: usize,
    pub errors: usize,
    pub ice_rules: usize,
    pub msi_tables: usize,
    pub snippets: usize,
    pub keywords: usize,
    pub schema_version: String,
    pub last_updated: Option<String>,
}

/// Source tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Source {
    pub id: i64,
    pub name: String,
    pub url: Option<String>,
    pub source_type: String,
    pub last_harvested: Option<String>,
    pub content_hash: Option<String>,
    pub enabled: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_attribute_type_from_str() {
        assert_eq!(AttributeType::from("guid"), AttributeType::Guid);
        assert_eq!(AttributeType::from("GUID"), AttributeType::Guid);
        assert_eq!(AttributeType::from("yesno"), AttributeType::YesNo);
        assert_eq!(AttributeType::from("integer"), AttributeType::Integer);
        assert_eq!(AttributeType::from("unknown"), AttributeType::String);
    }

    #[test]
    fn test_severity_from_str() {
        assert_eq!(Severity::from("error"), Severity::Error);
        assert_eq!(Severity::from("warning"), Severity::Warning);
        assert_eq!(Severity::from("warn"), Severity::Warning);
        assert_eq!(Severity::from("info"), Severity::Info);
        assert_eq!(Severity::from("other"), Severity::Info);
    }

    #[test]
    fn test_attribute_type_display() {
        assert_eq!(format!("{}", AttributeType::Guid), "guid");
        assert_eq!(format!("{}", AttributeType::String), "string");
    }

    #[test]
    fn test_severity_display() {
        assert_eq!(format!("{}", Severity::Error), "error");
        assert_eq!(format!("{}", Severity::Warning), "warning");
    }
}
