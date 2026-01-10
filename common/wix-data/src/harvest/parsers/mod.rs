//! Parsers for various data source formats

pub mod html;
pub mod xsd;

use crate::{Result, WixDataError};

/// Parser trait for data sources
pub trait Parser {
    type Output;

    fn parse(&self, content: &str) -> Result<Self::Output>;
}

/// Supported parser types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParserType {
    Xsd,
    Json,
    Yaml,
    Html,
}

impl TryFrom<&str> for ParserType {
    type Error = WixDataError;

    fn try_from(value: &str) -> Result<Self> {
        match value.to_lowercase().as_str() {
            "xsd" | "xml" => Ok(ParserType::Xsd),
            "json" => Ok(ParserType::Json),
            "yaml" | "yml" => Ok(ParserType::Yaml),
            "html" => Ok(ParserType::Html),
            _ => Err(WixDataError::Config(format!("Unknown parser type: {}", value))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parser_type_from_xsd() {
        assert_eq!(ParserType::try_from("xsd").unwrap(), ParserType::Xsd);
        assert_eq!(ParserType::try_from("xml").unwrap(), ParserType::Xsd);
        assert_eq!(ParserType::try_from("XSD").unwrap(), ParserType::Xsd);
    }

    #[test]
    fn test_parser_type_from_json() {
        assert_eq!(ParserType::try_from("json").unwrap(), ParserType::Json);
        assert_eq!(ParserType::try_from("JSON").unwrap(), ParserType::Json);
    }

    #[test]
    fn test_parser_type_from_yaml() {
        assert_eq!(ParserType::try_from("yaml").unwrap(), ParserType::Yaml);
        assert_eq!(ParserType::try_from("yml").unwrap(), ParserType::Yaml);
    }

    #[test]
    fn test_parser_type_from_html() {
        assert_eq!(ParserType::try_from("html").unwrap(), ParserType::Html);
    }

    #[test]
    fn test_parser_type_unknown() {
        assert!(ParserType::try_from("unknown").is_err());
    }
}
