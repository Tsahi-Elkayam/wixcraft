//! Attribute validation - GUID format, required attributes, enum values

use crate::types::{Diagnostic, DiagnosticSeverity, Range};
use regex::Regex;
use roxmltree::{Document, Node};
use std::collections::HashMap;
use std::path::Path;
use std::sync::LazyLock;

/// GUID regex pattern
static GUID_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^(\*|\{?[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}\}?)$").unwrap()
});

/// Identifier regex pattern (valid WiX identifier)
static IDENTIFIER_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^[a-zA-Z_][a-zA-Z0-9_\.]*$").unwrap()
});

/// Attribute requirements
struct AttributeReq {
    required: bool,
    attr_type: AttributeType,
}

#[derive(Clone)]
#[allow(dead_code)]
enum AttributeType {
    String,
    Identifier,
    Guid,
    YesNo,
    Integer,
    Enum(Vec<&'static str>),
}

/// Get attribute requirements for elements
fn get_attribute_requirements() -> HashMap<&'static str, HashMap<&'static str, AttributeReq>> {
    let mut map = HashMap::new();

    // Component
    let mut component = HashMap::new();
    component.insert(
        "Guid",
        AttributeReq {
            required: false, // Can be auto-generated in WiX v4
            attr_type: AttributeType::Guid,
        },
    );
    component.insert(
        "Id",
        AttributeReq {
            required: false,
            attr_type: AttributeType::Identifier,
        },
    );
    component.insert(
        "Transitive",
        AttributeReq {
            required: false,
            attr_type: AttributeType::YesNo,
        },
    );
    map.insert("Component", component);

    // Directory
    let mut directory = HashMap::new();
    directory.insert(
        "Id",
        AttributeReq {
            required: true,
            attr_type: AttributeType::Identifier,
        },
    );
    map.insert("Directory", directory);

    // Feature
    let mut feature = HashMap::new();
    feature.insert(
        "Id",
        AttributeReq {
            required: true,
            attr_type: AttributeType::Identifier,
        },
    );
    feature.insert(
        "Display",
        AttributeReq {
            required: false,
            attr_type: AttributeType::Enum(vec!["expand", "collapse", "hidden"]),
        },
    );
    feature.insert(
        "Level",
        AttributeReq {
            required: false,
            attr_type: AttributeType::Integer,
        },
    );
    feature.insert(
        "Absent",
        AttributeReq {
            required: false,
            attr_type: AttributeType::Enum(vec!["allow", "disallow"]),
        },
    );
    map.insert("Feature", feature);

    // File
    let mut file = HashMap::new();
    file.insert(
        "Id",
        AttributeReq {
            required: false,
            attr_type: AttributeType::Identifier,
        },
    );
    file.insert(
        "Vital",
        AttributeReq {
            required: false,
            attr_type: AttributeType::YesNo,
        },
    );
    file.insert(
        "ReadOnly",
        AttributeReq {
            required: false,
            attr_type: AttributeType::YesNo,
        },
    );
    file.insert(
        "Hidden",
        AttributeReq {
            required: false,
            attr_type: AttributeType::YesNo,
        },
    );
    map.insert("File", file);

    // Property
    let mut property = HashMap::new();
    property.insert(
        "Id",
        AttributeReq {
            required: true,
            attr_type: AttributeType::Identifier,
        },
    );
    property.insert(
        "Secure",
        AttributeReq {
            required: false,
            attr_type: AttributeType::YesNo,
        },
    );
    map.insert("Property", property);

    // CustomAction
    let mut custom_action = HashMap::new();
    custom_action.insert(
        "Id",
        AttributeReq {
            required: true,
            attr_type: AttributeType::Identifier,
        },
    );
    custom_action.insert(
        "Execute",
        AttributeReq {
            required: false,
            attr_type: AttributeType::Enum(vec![
                "immediate",
                "deferred",
                "rollback",
                "commit",
                "oncePerProcess",
                "firstSequence",
                "secondSequence",
            ]),
        },
    );
    custom_action.insert(
        "Return",
        AttributeReq {
            required: false,
            attr_type: AttributeType::Enum(vec!["check", "ignore", "asyncNoWait", "asyncWait"]),
        },
    );
    custom_action.insert(
        "Impersonate",
        AttributeReq {
            required: false,
            attr_type: AttributeType::YesNo,
        },
    );
    map.insert("CustomAction", custom_action);

    // RegistryKey
    let mut registry_key = HashMap::new();
    registry_key.insert(
        "Root",
        AttributeReq {
            required: true,
            attr_type: AttributeType::Enum(vec![
                "HKMU", "HKCR", "HKCU", "HKLM", "HKU",
            ]),
        },
    );
    map.insert("RegistryKey", registry_key);

    // RegistryValue
    let mut registry_value = HashMap::new();
    registry_value.insert(
        "Type",
        AttributeReq {
            required: false,
            attr_type: AttributeType::Enum(vec![
                "string", "integer", "binary", "expandable", "multiString",
            ]),
        },
    );
    map.insert("RegistryValue", registry_value);

    map
}

/// Validator for attributes
pub struct AttributeValidator {
    requirements: HashMap<&'static str, HashMap<&'static str, AttributeReq>>,
}

impl AttributeValidator {
    pub fn new() -> Self {
        Self {
            requirements: get_attribute_requirements(),
        }
    }

    /// Validate a source file
    pub fn validate(&self, source: &str, _file: &Path) -> Result<Vec<Diagnostic>, String> {
        let doc = Document::parse(source).map_err(|e| format!("XML parse error: {}", e))?;
        let mut diagnostics = Vec::new();
        self.validate_node(doc.root(), source, &mut diagnostics);
        Ok(diagnostics)
    }

    fn validate_node(&self, node: Node, source: &str, diagnostics: &mut Vec<Diagnostic>) {
        if node.is_element() {
            let tag_name = node.tag_name().name();

            if let Some(attr_reqs) = self.requirements.get(tag_name) {
                // Check required attributes
                for (attr_name, req) in attr_reqs {
                    if req.required && node.attribute(*attr_name).is_none() {
                        let range = get_node_range(&node, source);
                        diagnostics.push(
                            Diagnostic::error(
                                range,
                                format!("{} requires '{}' attribute", tag_name, attr_name),
                            )
                            .with_code("missing-required-attribute"),
                        );
                    }
                }

                // Validate attribute values
                for attr in node.attributes() {
                    if let Some(req) = attr_reqs.get(attr.name()) {
                        if let Some(diag) =
                            self.validate_attribute_value(tag_name, attr.name(), attr.value(), &req.attr_type, source, &node)
                        {
                            diagnostics.push(diag);
                        }
                    }
                }
            }
        }

        for child in node.children() {
            self.validate_node(child, source, diagnostics);
        }
    }

    fn validate_attribute_value(
        &self,
        element: &str,
        attr_name: &str,
        value: &str,
        attr_type: &AttributeType,
        source: &str,
        node: &Node,
    ) -> Option<Diagnostic> {
        let range = get_node_range(node, source);

        match attr_type {
            AttributeType::Guid => {
                if !GUID_REGEX.is_match(value) {
                    return Some(
                        Diagnostic::error(
                            range,
                            format!(
                                "Invalid GUID format for {}.{}. Use '*' for auto-generation or a valid GUID",
                                element, attr_name
                            ),
                        )
                        .with_code("invalid-guid"),
                    );
                }
            }
            AttributeType::Identifier => {
                if !IDENTIFIER_REGEX.is_match(value) && !value.starts_with("!(") {
                    return Some(
                        Diagnostic::warning(
                            range,
                            format!(
                                "Invalid identifier format for {}.{}. Should start with letter/underscore",
                                element, attr_name
                            ),
                        )
                        .with_code("invalid-identifier"),
                    );
                }
            }
            AttributeType::YesNo => {
                if !matches!(value, "yes" | "no" | "true" | "false" | "1" | "0") {
                    return Some(
                        Diagnostic::error(
                            range,
                            format!(
                                "Invalid yes/no value for {}.{}. Use 'yes' or 'no'",
                                element, attr_name
                            ),
                        )
                        .with_code("invalid-yesno"),
                    );
                }
            }
            AttributeType::Integer => {
                if value.parse::<i64>().is_err() {
                    return Some(
                        Diagnostic::error(
                            range,
                            format!("Invalid integer value for {}.{}", element, attr_name),
                        )
                        .with_code("invalid-integer"),
                    );
                }
            }
            AttributeType::Enum(valid_values) => {
                if !valid_values.contains(&value) {
                    return Some(
                        Diagnostic::new(
                            DiagnosticSeverity::Error,
                            range,
                            format!(
                                "Invalid value '{}' for {}.{}. Valid values: {}",
                                value,
                                element,
                                attr_name,
                                valid_values.join(", ")
                            ),
                        )
                        .with_code("invalid-enum-value"),
                    );
                }
            }
            AttributeType::String => {
                // No validation for strings
            }
        }

        None
    }
}

impl Default for AttributeValidator {
    fn default() -> Self {
        Self::new()
    }
}

/// Get range of a node
fn get_node_range(node: &Node, source: &str) -> Range {
    let r = node.range();
    Range::from_offsets(source, r.start, r.end)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_guid() {
        let source = r#"<Wix><Component Guid="{12345678-1234-1234-1234-123456789ABC}" /></Wix>"#;
        let validator = AttributeValidator::new();
        let diagnostics = validator.validate(source, Path::new("test.wxs")).unwrap();
        assert!(diagnostics.is_empty());
    }

    #[test]
    fn test_auto_guid() {
        let source = r#"<Wix><Component Guid="*" /></Wix>"#;
        let validator = AttributeValidator::new();
        let diagnostics = validator.validate(source, Path::new("test.wxs")).unwrap();
        assert!(diagnostics.is_empty());
    }

    #[test]
    fn test_invalid_guid() {
        let source = r#"<Wix><Component Guid="not-a-guid" /></Wix>"#;
        let validator = AttributeValidator::new();
        let diagnostics = validator.validate(source, Path::new("test.wxs")).unwrap();

        assert_eq!(diagnostics.len(), 1);
        assert!(diagnostics[0].message.contains("Invalid GUID"));
    }

    #[test]
    fn test_missing_required_attribute() {
        let source = r#"<Wix><Directory Name="Test" /></Wix>"#;
        let validator = AttributeValidator::new();
        let diagnostics = validator.validate(source, Path::new("test.wxs")).unwrap();

        assert_eq!(diagnostics.len(), 1);
        assert!(diagnostics[0].message.contains("requires 'Id'"));
    }

    #[test]
    fn test_valid_enum_value() {
        let source = r#"<Wix><Feature Id="F1" Display="expand" /></Wix>"#;
        let validator = AttributeValidator::new();
        let diagnostics = validator.validate(source, Path::new("test.wxs")).unwrap();
        assert!(diagnostics.is_empty());
    }

    #[test]
    fn test_invalid_enum_value() {
        let source = r#"<Wix><Feature Id="F1" Display="invalid" /></Wix>"#;
        let validator = AttributeValidator::new();
        let diagnostics = validator.validate(source, Path::new("test.wxs")).unwrap();

        assert_eq!(diagnostics.len(), 1);
        assert!(diagnostics[0].message.contains("Invalid value"));
    }

    #[test]
    fn test_valid_yesno() {
        let source = r#"<Wix><File Id="F1" Vital="yes" /></Wix>"#;
        let validator = AttributeValidator::new();
        let diagnostics = validator.validate(source, Path::new("test.wxs")).unwrap();
        assert!(diagnostics.is_empty());
    }

    #[test]
    fn test_invalid_yesno() {
        let source = r#"<Wix><File Id="F1" Vital="maybe" /></Wix>"#;
        let validator = AttributeValidator::new();
        let diagnostics = validator.validate(source, Path::new("test.wxs")).unwrap();

        assert_eq!(diagnostics.len(), 1);
        assert!(diagnostics[0].message.contains("yes/no"));
    }

    #[test]
    fn test_valid_identifier() {
        let source = r#"<Wix><Component Id="Valid_Id123" /></Wix>"#;
        let validator = AttributeValidator::new();
        let diagnostics = validator.validate(source, Path::new("test.wxs")).unwrap();
        assert!(diagnostics.is_empty());
    }

    #[test]
    fn test_registry_root_enum() {
        let source = r#"<Wix><RegistryKey Root="HKLM" Key="Test" /></Wix>"#;
        let validator = AttributeValidator::new();
        let diagnostics = validator.validate(source, Path::new("test.wxs")).unwrap();
        assert!(diagnostics.is_empty());
    }

    #[test]
    fn test_invalid_registry_root() {
        let source = r#"<Wix><RegistryKey Root="INVALID" Key="Test" /></Wix>"#;
        let validator = AttributeValidator::new();
        let diagnostics = validator.validate(source, Path::new("test.wxs")).unwrap();

        assert_eq!(diagnostics.len(), 1);
    }

    #[test]
    fn test_custom_action_execute() {
        let source = r#"<Wix><CustomAction Id="CA1" Execute="deferred" /></Wix>"#;
        let validator = AttributeValidator::new();
        let diagnostics = validator.validate(source, Path::new("test.wxs")).unwrap();
        assert!(diagnostics.is_empty());
    }
}
