//! wix-syntax - Syntax highlighting definitions for WiX files
//!
//! Generates TextMate grammars and other syntax definitions.

use serde::{Deserialize, Serialize};

/// TextMate grammar for WiX files
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextMateGrammar {
    pub name: String,
    #[serde(rename = "scopeName")]
    pub scope_name: String,
    #[serde(rename = "fileTypes")]
    pub file_types: Vec<String>,
    pub patterns: Vec<Pattern>,
    pub repository: std::collections::HashMap<String, RepositoryItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pattern {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "match")]
    pub match_pattern: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub begin: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub patterns: Option<Vec<Pattern>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub captures: Option<std::collections::HashMap<String, Capture>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryItem {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "match")]
    pub match_pattern: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub begin: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub patterns: Option<Vec<Pattern>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Capture {
    pub name: String,
}

impl TextMateGrammar {
    /// Generate the WiX syntax grammar
    pub fn wix_grammar() -> Self {
        let mut repository = std::collections::HashMap::new();

        // WiX elements
        repository.insert(
            "wix-elements".to_string(),
            RepositoryItem {
                name: Some("entity.name.tag.wix".to_string()),
                match_pattern: Some(r"(?i)\b(Wix|Package|Product|Fragment|Component|ComponentGroup|Directory|DirectoryRef|Feature|FeatureRef|File|Property|Registry|RegistryKey|RegistryValue|Shortcut|ServiceInstall|ServiceControl|CustomAction|Binary|UI|UIRef|Media|MajorUpgrade|Upgrade|UpgradeVersion|StandardDirectory|Launch|Condition|SetProperty|ComponentRef|CreateFolder|RemoveFile|RemoveFolder|Environment|IniFile|PermissionEx|util:User|util:Group)\b".to_string()),
                begin: None,
                end: None,
                patterns: None,
            },
        );

        // WiX attributes
        repository.insert(
            "wix-attributes".to_string(),
            RepositoryItem {
                name: Some("entity.other.attribute-name.wix".to_string()),
                match_pattern: Some(r"\b(Id|Name|Value|Source|Directory|Guid|Type|Key|Root|Action|Execute|Return|Level|Title|Description|Manufacturer|Version|UpgradeCode|Language|Codepage|Compressed|Cabinet|EmbedCab|DiskId|Target|WorkingDirectory|Arguments|Advertise|Absent|AllowAdvertise|InstallDefault|TypicalDefault|ConfigurableDirectory)\b".to_string()),
                begin: None,
                end: None,
                patterns: None,
            },
        );

        // GUID pattern
        repository.insert(
            "guid".to_string(),
            RepositoryItem {
                name: Some("constant.other.guid.wix".to_string()),
                match_pattern: Some(r"\{[0-9A-Fa-f]{8}-[0-9A-Fa-f]{4}-[0-9A-Fa-f]{4}-[0-9A-Fa-f]{4}-[0-9A-Fa-f]{12}\}".to_string()),
                begin: None,
                end: None,
                patterns: None,
            },
        );

        // Preprocessor variables
        repository.insert(
            "preprocessor".to_string(),
            RepositoryItem {
                name: Some("variable.other.preprocessor.wix".to_string()),
                match_pattern: Some(r"\$\(var\.[A-Za-z_][A-Za-z0-9_]*\)".to_string()),
                begin: None,
                end: None,
                patterns: None,
            },
        );

        // Property references
        repository.insert(
            "property-reference".to_string(),
            RepositoryItem {
                name: Some("variable.other.property.wix".to_string()),
                match_pattern: Some(r"\[[A-Z_][A-Z0-9_]*\]".to_string()),
                begin: None,
                end: None,
                patterns: None,
            },
        );

        Self {
            name: "WiX".to_string(),
            scope_name: "text.xml.wix".to_string(),
            file_types: vec!["wxs".to_string(), "wxi".to_string(), "wxl".to_string()],
            patterns: vec![
                Pattern {
                    include: Some("#wix-elements".to_string()),
                    name: None,
                    match_pattern: None,
                    begin: None,
                    end: None,
                    patterns: None,
                    captures: None,
                },
                Pattern {
                    include: Some("#wix-attributes".to_string()),
                    name: None,
                    match_pattern: None,
                    begin: None,
                    end: None,
                    patterns: None,
                    captures: None,
                },
                Pattern {
                    include: Some("#guid".to_string()),
                    name: None,
                    match_pattern: None,
                    begin: None,
                    end: None,
                    patterns: None,
                    captures: None,
                },
                Pattern {
                    include: Some("#preprocessor".to_string()),
                    name: None,
                    match_pattern: None,
                    begin: None,
                    end: None,
                    patterns: None,
                    captures: None,
                },
                Pattern {
                    include: Some("#property-reference".to_string()),
                    name: None,
                    match_pattern: None,
                    begin: None,
                    end: None,
                    patterns: None,
                    captures: None,
                },
                Pattern {
                    include: Some("text.xml".to_string()),
                    name: None,
                    match_pattern: None,
                    begin: None,
                    end: None,
                    patterns: None,
                    captures: None,
                },
            ],
            repository,
        }
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap()
    }
}

/// Language configuration for VS Code
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageConfiguration {
    pub comments: Comments,
    pub brackets: Vec<(String, String)>,
    #[serde(rename = "autoClosingPairs")]
    pub auto_closing_pairs: Vec<AutoClosePair>,
    #[serde(rename = "surroundingPairs")]
    pub surrounding_pairs: Vec<(String, String)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Comments {
    #[serde(rename = "blockComment")]
    pub block_comment: (String, String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoClosePair {
    pub open: String,
    pub close: String,
}

impl LanguageConfiguration {
    pub fn wix_config() -> Self {
        Self {
            comments: Comments {
                block_comment: ("<!--".to_string(), "-->".to_string()),
            },
            brackets: vec![
                ("<".to_string(), ">".to_string()),
                ("{".to_string(), "}".to_string()),
                ("[".to_string(), "]".to_string()),
                ("(".to_string(), ")".to_string()),
            ],
            auto_closing_pairs: vec![
                AutoClosePair { open: "<".to_string(), close: ">".to_string() },
                AutoClosePair { open: "\"".to_string(), close: "\"".to_string() },
                AutoClosePair { open: "'".to_string(), close: "'".to_string() },
                AutoClosePair { open: "{".to_string(), close: "}".to_string() },
                AutoClosePair { open: "[".to_string(), close: "]".to_string() },
                AutoClosePair { open: "(".to_string(), close: ")".to_string() },
            ],
            surrounding_pairs: vec![
                ("\"".to_string(), "\"".to_string()),
                ("'".to_string(), "'".to_string()),
                ("<".to_string(), ">".to_string()),
            ],
        }
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grammar_generation() {
        let grammar = TextMateGrammar::wix_grammar();
        assert_eq!(grammar.name, "WiX");
        assert!(grammar.file_types.contains(&"wxs".to_string()));
    }

    #[test]
    fn test_grammar_json() {
        let grammar = TextMateGrammar::wix_grammar();
        let json = grammar.to_json();
        assert!(json.contains("scopeName"));
        assert!(json.contains("text.xml.wix"));
    }

    #[test]
    fn test_language_config() {
        let config = LanguageConfiguration::wix_config();
        let json = config.to_json();
        assert!(json.contains("blockComment"));
    }

    #[test]
    fn test_repository_items() {
        let grammar = TextMateGrammar::wix_grammar();
        assert!(grammar.repository.contains_key("wix-elements"));
        assert!(grammar.repository.contains_key("guid"));
    }
}
