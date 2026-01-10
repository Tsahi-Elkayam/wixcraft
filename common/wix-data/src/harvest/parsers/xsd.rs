//! XSD schema parser for WiX element extraction

use crate::models::{Attribute, AttributeType, Element};
use crate::{Result, WixDataError};
use roxmltree::{Document, Node};
use std::collections::HashMap;

const XS_NS: &str = "http://www.w3.org/2001/XMLSchema";

/// XSD Parser for extracting WiX elements and attributes
pub struct XsdParser {
    namespace: String,
}

impl XsdParser {
    pub fn new(namespace: &str) -> Self {
        Self {
            namespace: namespace.to_string(),
        }
    }

    /// Parse XSD content and extract elements
    pub fn parse(&self, content: &str) -> Result<ParsedSchema> {
        let doc = Document::parse(content)
            .map_err(|e| WixDataError::Parse(format!("XML parse error: {}", e)))?;

        let mut result = ParsedSchema::default();

        // First pass: collect all named types
        let mut simple_types: HashMap<String, Vec<String>> = HashMap::new();

        for node in doc.descendants() {
            if !is_xs_element(&node, "simpleType") {
                continue;
            }

            if let Some(name) = node.attribute("name") {
                let values = self.extract_enum_values(&node);
                if !values.is_empty() {
                    simple_types.insert(name.to_string(), values);
                }
            }
        }

        // Second pass: extract elements
        for node in doc.descendants() {
            if !is_xs_element(&node, "element") {
                continue;
            }

            // Only process top-level elements (direct children of schema)
            if let Some(parent) = node.parent() {
                if !is_xs_element(&parent, "schema") {
                    continue;
                }
            }

            if let Some(element) = self.parse_element(&node, &simple_types) {
                result.elements.push(element);
            }
        }

        Ok(result)
    }

    fn parse_element(&self, node: &Node, _simple_types: &HashMap<String, Vec<String>>) -> Option<Element> {
        let name = node.attribute("name")?;

        let description = self.get_documentation(node);
        let doc_url = format!(
            "https://wixtoolset.org/docs/schema/wxs/{}/",
            name.to_lowercase()
        );

        Some(Element {
            id: 0,
            name: name.to_string(),
            namespace: self.namespace.clone(),
            since_version: Some("v4".to_string()),
            deprecated_version: None,
            description,
            documentation_url: Some(doc_url),
            remarks: None,
            example: None,
        })
    }

    fn get_documentation(&self, node: &Node) -> Option<String> {
        for child in node.children() {
            if is_xs_element(&child, "annotation") {
                for doc in child.children() {
                    if is_xs_element(&doc, "documentation") {
                        if let Some(text) = doc.text() {
                            let cleaned: String = text
                                .split_whitespace()
                                .collect::<Vec<_>>()
                                .join(" ");
                            if !cleaned.is_empty() {
                                return Some(cleaned);
                            }
                        }
                    }
                }
            }
        }
        None
    }

    fn extract_enum_values(&self, node: &Node) -> Vec<String> {
        let mut values = Vec::new();

        for child in node.descendants() {
            if is_xs_element(&child, "enumeration") {
                if let Some(value) = child.attribute("value") {
                    values.push(value.to_string());
                }
            }
        }

        values
    }

    /// Extract attributes from complex type
    pub fn extract_attributes(&self, node: &Node, simple_types: &HashMap<String, Vec<String>>) -> Vec<Attribute> {
        let mut attributes = Vec::new();

        for child in node.descendants() {
            if !is_xs_element(&child, "attribute") {
                continue;
            }

            if let Some(attr) = self.parse_attribute(&child, simple_types) {
                attributes.push(attr);
            }
        }

        attributes
    }

    fn parse_attribute(&self, node: &Node, simple_types: &HashMap<String, Vec<String>>) -> Option<Attribute> {
        let name = node.attribute("name")?;
        let type_str = node.attribute("type").unwrap_or("string");
        let use_val = node.attribute("use").unwrap_or("optional");
        let default = node.attribute("default").map(|s| s.to_string());
        let description = self.get_documentation(node);

        let type_name = type_str.split(':').last().unwrap_or(type_str);
        let (attr_type, enum_values) = self.resolve_type(type_name, simple_types);

        Some(Attribute {
            id: 0,
            element_id: 0,
            name: name.to_string(),
            attr_type,
            required: use_val == "required",
            default_value: default,
            description,
            since_version: Some("v4".to_string()),
            deprecated_version: None,
            enum_values,
        })
    }

    fn resolve_type(&self, type_name: &str, simple_types: &HashMap<String, Vec<String>>) -> (AttributeType, Vec<String>) {
        // Check for enum type
        if let Some(values) = simple_types.get(type_name) {
            return (AttributeType::Enum, values.clone());
        }

        // Map XSD types to our types
        let attr_type = match type_name {
            "string" | "NMTOKEN" | "token" | "normalizedString" => AttributeType::String,
            "boolean" | "YesNoType" | "YesNoDefaultType" => AttributeType::YesNo,
            "integer" | "int" | "positiveInteger" | "nonNegativeInteger" | "long" | "short" => {
                AttributeType::Integer
            }
            "Guid" | "GuidType" | "ComponentGuid" | "AutogenGuid" => AttributeType::Guid,
            "VersionType" => AttributeType::Version,
            _ => AttributeType::String,
        };

        (attr_type, Vec::new())
    }
}

/// Check if a node is an xs: element with the given name
fn is_xs_element(node: &Node, name: &str) -> bool {
    node.tag_name().namespace() == Some(XS_NS) && node.tag_name().name() == name
}

/// Result of parsing an XSD schema
#[derive(Debug, Default)]
pub struct ParsedSchema {
    pub elements: Vec<Element>,
    pub attributes: Vec<Attribute>,
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_XSD: &str = r#"<?xml version="1.0" encoding="utf-8"?>
<xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema">
    <xs:simpleType name="YesNoType">
        <xs:restriction base="xs:NMTOKEN">
            <xs:enumeration value="yes"/>
            <xs:enumeration value="no"/>
        </xs:restriction>
    </xs:simpleType>

    <xs:element name="Package">
        <xs:annotation>
            <xs:documentation>The root element of a WiX installer package.</xs:documentation>
        </xs:annotation>
        <xs:complexType>
            <xs:attribute name="Id" type="xs:string" use="required"/>
            <xs:attribute name="Compressed" type="YesNoType"/>
        </xs:complexType>
    </xs:element>

    <xs:element name="Component">
        <xs:annotation>
            <xs:documentation>A component groups together files and resources.</xs:documentation>
        </xs:annotation>
    </xs:element>
</xs:schema>"#;

    #[test]
    fn test_parse_elements() {
        let parser = XsdParser::new("wix");
        let result = parser.parse(SAMPLE_XSD).unwrap();

        assert_eq!(result.elements.len(), 2);

        let package = result.elements.iter().find(|e| e.name == "Package").unwrap();
        assert_eq!(package.namespace, "wix");
        assert!(package.description.as_ref().unwrap().contains("root element"));

        let component = result.elements.iter().find(|e| e.name == "Component").unwrap();
        assert!(component.description.as_ref().unwrap().contains("component"));
    }

    #[test]
    fn test_enum_extraction() {
        let parser = XsdParser::new("wix");
        let doc = Document::parse(SAMPLE_XSD).unwrap();

        for node in doc.descendants() {
            if is_xs_element(&node, "simpleType") {
                if node.attribute("name") == Some("YesNoType") {
                    let values = parser.extract_enum_values(&node);
                    assert_eq!(values, vec!["yes", "no"]);
                }
            }
        }
    }
}
