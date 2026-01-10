//! Language pack loader for code tooling framework
//!
//! Provides a unified API for loading language-specific schemas, rules, and snippets.
//!
//! # Example
//!
//! ```
//! use schema_loader::{SchemaLoader, Language};
//!
//! let loader = SchemaLoader::new();
//! if let Some(pack) = loader.get_pack(Language::Wix) {
//!     println!("Loaded {} elements", pack.elements().len());
//! }
//! ```

pub use code_detector::Language;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::RwLock;

/// Element definition in a language schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Element {
    /// Element name (e.g., "Component", "Feature")
    pub name: String,
    /// Human-readable description
    #[serde(default)]
    pub description: String,
    /// Documentation URL
    #[serde(default)]
    pub docs_url: Option<String>,
    /// Required attributes
    #[serde(default)]
    pub required_attributes: Vec<String>,
    /// Optional attributes
    #[serde(default)]
    pub optional_attributes: Vec<String>,
    /// Allowed child elements
    #[serde(default)]
    pub children: Vec<String>,
    /// Allowed parent elements
    #[serde(default)]
    pub parents: Vec<String>,
    /// Example usage
    #[serde(default)]
    pub example: Option<String>,
    /// Deprecation notice
    #[serde(default)]
    pub deprecated: Option<String>,
    /// Minimum version where this element is available
    #[serde(default)]
    pub since: Option<String>,
}

impl Element {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: String::new(),
            docs_url: None,
            required_attributes: Vec::new(),
            optional_attributes: Vec::new(),
            children: Vec::new(),
            parents: Vec::new(),
            example: None,
            deprecated: None,
            since: None,
        }
    }

    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = desc.into();
        self
    }
}

/// Attribute definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attribute {
    /// Attribute name
    pub name: String,
    /// Description
    #[serde(default)]
    pub description: String,
    /// Type (string, boolean, enum, etc.)
    #[serde(default = "default_type")]
    pub attr_type: String,
    /// Allowed values for enum types
    #[serde(default)]
    pub allowed_values: Vec<String>,
    /// Default value
    #[serde(default)]
    pub default: Option<String>,
    /// Is this attribute required?
    #[serde(default)]
    pub required: bool,
    /// Deprecation notice
    #[serde(default)]
    pub deprecated: Option<String>,
}

fn default_type() -> String {
    "string".to_string()
}

impl Attribute {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: String::new(),
            attr_type: "string".to_string(),
            allowed_values: Vec::new(),
            default: None,
            required: false,
            deprecated: None,
        }
    }
}

/// Lint/analysis rule definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rule {
    /// Rule ID (e.g., "SEC-001", "BP-002")
    pub id: String,
    /// Rule name
    pub name: String,
    /// Description of what the rule checks
    #[serde(default)]
    pub description: String,
    /// Category (security, best-practice, etc.)
    #[serde(default)]
    pub category: String,
    /// Severity (blocker, high, medium, low, info)
    #[serde(default = "default_severity")]
    pub severity: String,
    /// Effort to fix in minutes
    #[serde(default)]
    pub effort_minutes: u32,
    /// Tags for filtering
    #[serde(default)]
    pub tags: Vec<String>,
    /// Rationale explaining why this rule matters
    #[serde(default)]
    pub rationale: Option<String>,
    /// Bad code example
    #[serde(default)]
    pub bad_example: Option<String>,
    /// Good code example
    #[serde(default)]
    pub good_example: Option<String>,
    /// References (CWE, OWASP, etc.)
    #[serde(default)]
    pub references: Vec<String>,
    /// Is the rule enabled by default?
    #[serde(default = "default_true")]
    pub enabled: bool,
}

fn default_severity() -> String {
    "medium".to_string()
}

fn default_true() -> bool {
    true
}

impl Rule {
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: String::new(),
            category: String::new(),
            severity: "medium".to_string(),
            effort_minutes: 5,
            tags: Vec::new(),
            rationale: None,
            bad_example: None,
            good_example: None,
            references: Vec::new(),
            enabled: true,
        }
    }
}

/// Code snippet definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snippet {
    /// Snippet prefix (trigger)
    pub prefix: String,
    /// Display name
    pub name: String,
    /// Description shown in completion
    #[serde(default)]
    pub description: String,
    /// Snippet body (with $1, $2 placeholders)
    pub body: String,
    /// Scope/context where snippet applies
    #[serde(default)]
    pub scope: Option<String>,
}

impl Snippet {
    pub fn new(prefix: impl Into<String>, name: impl Into<String>, body: impl Into<String>) -> Self {
        Self {
            prefix: prefix.into(),
            name: name.into(),
            description: String::new(),
            body: body.into(),
            scope: None,
        }
    }
}

/// Keyword for autocomplete
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Keyword {
    /// The keyword text
    pub text: String,
    /// Description
    #[serde(default)]
    pub description: String,
    /// Category (e.g., "builtin", "type", "function")
    #[serde(default)]
    pub category: String,
    /// Documentation URL
    #[serde(default)]
    pub docs_url: Option<String>,
}

impl Keyword {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            description: String::new(),
            category: String::new(),
            docs_url: None,
        }
    }
}

/// Formatting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormatConfig {
    /// Indentation string (spaces or tab)
    #[serde(default = "default_indent")]
    pub indent: String,
    /// Number of spaces for indentation
    #[serde(default = "default_indent_size")]
    pub indent_size: usize,
    /// Maximum line length
    #[serde(default = "default_line_length")]
    pub max_line_length: usize,
    /// Insert final newline
    #[serde(default = "default_true")]
    pub final_newline: bool,
    /// Trim trailing whitespace
    #[serde(default = "default_true")]
    pub trim_trailing_whitespace: bool,
    /// Element ordering rules (for XML-like languages)
    #[serde(default)]
    pub element_order: Vec<String>,
    /// Attribute ordering rules
    #[serde(default)]
    pub attribute_order: Vec<String>,
}

impl Default for FormatConfig {
    fn default() -> Self {
        Self {
            indent: "  ".to_string(),
            indent_size: 2,
            max_line_length: 120,
            final_newline: true,
            trim_trailing_whitespace: true,
            element_order: Vec::new(),
            attribute_order: Vec::new(),
        }
    }
}

fn default_indent() -> String {
    "  ".to_string()
}

fn default_indent_size() -> usize {
    2
}

fn default_line_length() -> usize {
    120
}

/// A language pack containing all schema data for a language
#[derive(Debug, Clone, Default)]
pub struct LanguagePack {
    /// Language this pack is for
    pub language: Language,
    /// Display name
    pub name: String,
    /// Version of the pack
    pub version: String,
    /// Element definitions
    elements: Vec<Element>,
    /// Attribute definitions (element -> attributes)
    attributes: HashMap<String, Vec<Attribute>>,
    /// Lint rules
    rules: Vec<Rule>,
    /// Code snippets
    snippets: Vec<Snippet>,
    /// Keywords for autocomplete
    keywords: Vec<Keyword>,
    /// Formatting config
    format_config: FormatConfig,
}

impl LanguagePack {
    /// Create a new empty language pack
    pub fn new(language: Language) -> Self {
        Self {
            language,
            name: language.display_name().to_string(),
            version: "0.1.0".to_string(),
            elements: Vec::new(),
            attributes: HashMap::new(),
            rules: Vec::new(),
            snippets: Vec::new(),
            keywords: Vec::new(),
            format_config: FormatConfig::default(),
        }
    }

    // === Accessors ===

    pub fn elements(&self) -> &[Element] {
        &self.elements
    }

    pub fn get_element(&self, name: &str) -> Option<&Element> {
        self.elements.iter().find(|e| e.name == name)
    }

    pub fn attributes(&self, element: &str) -> &[Attribute] {
        self.attributes.get(element).map(|v| v.as_slice()).unwrap_or(&[])
    }

    pub fn get_attribute(&self, element: &str, name: &str) -> Option<&Attribute> {
        self.attributes.get(element)
            .and_then(|attrs| attrs.iter().find(|a| a.name == name))
    }

    pub fn rules(&self) -> &[Rule] {
        &self.rules
    }

    pub fn get_rule(&self, id: &str) -> Option<&Rule> {
        self.rules.iter().find(|r| r.id == id)
    }

    pub fn snippets(&self) -> &[Snippet] {
        &self.snippets
    }

    pub fn keywords(&self) -> &[Keyword] {
        &self.keywords
    }

    pub fn format_config(&self) -> &FormatConfig {
        &self.format_config
    }

    // === Builders ===

    pub fn with_element(mut self, element: Element) -> Self {
        self.elements.push(element);
        self
    }

    pub fn with_attribute(mut self, element: impl Into<String>, attr: Attribute) -> Self {
        self.attributes
            .entry(element.into())
            .or_default()
            .push(attr);
        self
    }

    pub fn with_rule(mut self, rule: Rule) -> Self {
        self.rules.push(rule);
        self
    }

    pub fn with_snippet(mut self, snippet: Snippet) -> Self {
        self.snippets.push(snippet);
        self
    }

    pub fn with_keyword(mut self, keyword: Keyword) -> Self {
        self.keywords.push(keyword);
        self
    }

    pub fn with_format_config(mut self, config: FormatConfig) -> Self {
        self.format_config = config;
        self
    }

    // === Mutation ===

    pub fn add_element(&mut self, element: Element) {
        self.elements.push(element);
    }

    pub fn add_rule(&mut self, rule: Rule) {
        self.rules.push(rule);
    }

    pub fn add_snippet(&mut self, snippet: Snippet) {
        self.snippets.push(snippet);
    }

    pub fn add_keyword(&mut self, keyword: Keyword) {
        self.keywords.push(keyword);
    }
}

/// Schema loader that manages language packs
pub struct SchemaLoader {
    /// Loaded packs by language
    packs: RwLock<HashMap<Language, LanguagePack>>,
    /// Search paths for pack data
    search_paths: Vec<PathBuf>,
}

impl Default for SchemaLoader {
    fn default() -> Self {
        Self::new()
    }
}

impl SchemaLoader {
    /// Create a new schema loader
    pub fn new() -> Self {
        Self {
            packs: RwLock::new(HashMap::new()),
            search_paths: Vec::new(),
        }
    }

    /// Add a search path for language pack data
    pub fn add_search_path(&mut self, path: impl AsRef<Path>) {
        self.search_paths.push(path.as_ref().to_path_buf());
    }

    /// Register a language pack
    pub fn register_pack(&self, pack: LanguagePack) {
        let mut packs = self.packs.write().unwrap();
        packs.insert(pack.language, pack);
    }

    /// Get a language pack (if loaded)
    pub fn get_pack(&self, language: Language) -> Option<LanguagePack> {
        let packs = self.packs.read().unwrap();
        packs.get(&language).cloned()
    }

    /// Check if a pack is loaded
    pub fn has_pack(&self, language: Language) -> bool {
        let packs = self.packs.read().unwrap();
        packs.contains_key(&language)
    }

    /// Get list of loaded languages
    pub fn loaded_languages(&self) -> Vec<Language> {
        let packs = self.packs.read().unwrap();
        packs.keys().copied().collect()
    }

    /// Load pack from a directory
    pub fn load_from_directory(&self, language: Language, dir: &Path) -> Result<(), String> {
        let mut pack = LanguagePack::new(language);

        // Load elements
        let elements_dir = dir.join("elements");
        if elements_dir.exists() {
            for entry in std::fs::read_dir(&elements_dir)
                .map_err(|e| format!("Failed to read elements directory: {}", e))?
            {
                let entry = entry.map_err(|e| e.to_string())?;
                let path = entry.path();
                if path.extension().and_then(|e| e.to_str()) == Some("json") {
                    let content = std::fs::read_to_string(&path)
                        .map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;
                    let element: Element = serde_json::from_str(&content)
                        .map_err(|e| format!("Failed to parse {}: {}", path.display(), e))?;
                    pack.add_element(element);
                }
            }
        }

        // Load rules
        let rules_dir = dir.join("rules");
        if rules_dir.exists() {
            for entry in std::fs::read_dir(&rules_dir)
                .map_err(|e| format!("Failed to read rules directory: {}", e))?
            {
                let entry = entry.map_err(|e| e.to_string())?;
                let path = entry.path();
                if path.extension().and_then(|e| e.to_str()) == Some("json") {
                    let content = std::fs::read_to_string(&path)
                        .map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;

                    // Try as array first, then single rule
                    if let Ok(rules) = serde_json::from_str::<Vec<Rule>>(&content) {
                        for rule in rules {
                            pack.add_rule(rule);
                        }
                    } else if let Ok(rule) = serde_json::from_str::<Rule>(&content) {
                        pack.add_rule(rule);
                    }
                }
            }
        }

        // Load snippets
        let snippets_path = dir.join("snippets").join("snippets.json");
        if snippets_path.exists() {
            let content = std::fs::read_to_string(&snippets_path)
                .map_err(|e| format!("Failed to read snippets: {}", e))?;
            let snippets: Vec<Snippet> = serde_json::from_str(&content)
                .map_err(|e| format!("Failed to parse snippets: {}", e))?;
            for snippet in snippets {
                pack.add_snippet(snippet);
            }
        }

        // Load keywords
        let keywords_path = dir.join("keywords").join("keywords.json");
        if keywords_path.exists() {
            let content = std::fs::read_to_string(&keywords_path)
                .map_err(|e| format!("Failed to read keywords: {}", e))?;
            let keywords: Vec<Keyword> = serde_json::from_str(&content)
                .map_err(|e| format!("Failed to parse keywords: {}", e))?;
            for keyword in keywords {
                pack.add_keyword(keyword);
            }
        }

        self.register_pack(pack);
        Ok(())
    }

    /// Detect language and get appropriate pack
    pub fn get_pack_for_file(&self, path: &str, content: &str) -> Option<LanguagePack> {
        let language = code_detector::detect(path, content);
        self.get_pack(language)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_element_builder() {
        let elem = Element::new("Component")
            .with_description("A component groups files");

        assert_eq!(elem.name, "Component");
        assert_eq!(elem.description, "A component groups files");
    }

    #[test]
    fn test_rule_defaults() {
        let rule = Rule::new("SEC-001", "Security check");

        assert_eq!(rule.severity, "medium");
        assert_eq!(rule.effort_minutes, 5);
        assert!(rule.enabled);
    }

    #[test]
    fn test_language_pack_builder() {
        let pack = LanguagePack::new(Language::Wix)
            .with_element(Element::new("Wix"))
            .with_element(Element::new("Package"))
            .with_rule(Rule::new("SEC-001", "Security"))
            .with_snippet(Snippet::new("comp", "Component", "<Component Id=\"$1\" />"));

        assert_eq!(pack.elements().len(), 2);
        assert_eq!(pack.rules().len(), 1);
        assert_eq!(pack.snippets().len(), 1);
    }

    #[test]
    fn test_get_element() {
        let pack = LanguagePack::new(Language::Wix)
            .with_element(Element::new("Component").with_description("Groups files"));

        let elem = pack.get_element("Component");
        assert!(elem.is_some());
        assert_eq!(elem.unwrap().description, "Groups files");

        let missing = pack.get_element("NotFound");
        assert!(missing.is_none());
    }

    #[test]
    fn test_attributes() {
        let pack = LanguagePack::new(Language::Wix)
            .with_attribute("Component", Attribute::new("Id"))
            .with_attribute("Component", Attribute::new("Guid"));

        let attrs = pack.attributes("Component");
        assert_eq!(attrs.len(), 2);

        let id = pack.get_attribute("Component", "Id");
        assert!(id.is_some());

        let empty = pack.attributes("NotFound");
        assert!(empty.is_empty());
    }

    #[test]
    fn test_schema_loader() {
        let loader = SchemaLoader::new();

        let pack = LanguagePack::new(Language::Wix)
            .with_element(Element::new("Wix"));

        loader.register_pack(pack);

        assert!(loader.has_pack(Language::Wix));
        assert!(!loader.has_pack(Language::Rust));

        let retrieved = loader.get_pack(Language::Wix);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().elements().len(), 1);
    }

    #[test]
    fn test_loaded_languages() {
        let loader = SchemaLoader::new();

        loader.register_pack(LanguagePack::new(Language::Wix));
        loader.register_pack(LanguagePack::new(Language::Ansible));

        let loaded = loader.loaded_languages();
        assert_eq!(loaded.len(), 2);
        assert!(loaded.contains(&Language::Wix));
        assert!(loaded.contains(&Language::Ansible));
    }

    #[test]
    fn test_format_config_defaults() {
        let config = FormatConfig::default();

        assert_eq!(config.indent, "  ");
        assert_eq!(config.indent_size, 2);
        assert_eq!(config.max_line_length, 120);
        assert!(config.final_newline);
        assert!(config.trim_trailing_whitespace);
    }

    #[test]
    fn test_snippet_creation() {
        let snippet = Snippet::new("feat", "Feature", "<Feature Id=\"$1\">\n  $0\n</Feature>");

        assert_eq!(snippet.prefix, "feat");
        assert_eq!(snippet.name, "Feature");
        assert!(snippet.body.contains("$1"));
    }

    #[test]
    fn test_keyword_creation() {
        let keyword = Keyword::new("async");

        assert_eq!(keyword.text, "async");
        assert!(keyword.description.is_empty());
    }

    #[test]
    fn test_get_pack_for_file() {
        let loader = SchemaLoader::new();
        loader.register_pack(LanguagePack::new(Language::Wix));

        let pack = loader.get_pack_for_file("product.wxs", "<Wix>");
        assert!(pack.is_some());
        assert_eq!(pack.unwrap().language, Language::Wix);

        let no_pack = loader.get_pack_for_file("main.rs", "fn main() {}");
        assert!(no_pack.is_none()); // Rust pack not loaded
    }
}
