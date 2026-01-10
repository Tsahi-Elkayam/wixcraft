//! Data harvesting from various sources

pub mod parsers;

use crate::config::{SourceDef, SourcesConfig};
use crate::db::Database;
use crate::models::*;
use crate::{Result, WixKbError};
use std::path::{Path, PathBuf};

/// Harvester for collecting data from sources
pub struct Harvester {
    config: SourcesConfig,
    base_path: PathBuf,
    cache_path: PathBuf,
}

impl Harvester {
    /// Create a new harvester with configuration
    pub fn new<P: AsRef<Path>>(config_path: P, base_path: P) -> Result<Self> {
        let config = SourcesConfig::load(&config_path)?;
        let cache_path = base_path.as_ref().join(&config.harvest.cache_dir);
        std::fs::create_dir_all(&cache_path)?;

        Ok(Self {
            config,
            base_path: base_path.as_ref().to_path_buf(),
            cache_path,
        })
    }

    /// Harvest all enabled sources
    pub fn harvest_all(&self, db: &Database) -> Result<HarvestReport> {
        let mut report = HarvestReport::default();

        for (category, sources) in &self.config.sources {
            for (name, source) in sources {
                match self.harvest_source(db, category, name, source) {
                    Ok(stats) => {
                        report.sources_processed += 1;
                        report.items_harvested += stats.items;
                        report.source_stats.push((name.clone(), stats));
                    }
                    Err(e) => {
                        report.errors.push(format!("{}/{}: {}", category, name, e));
                    }
                }
            }
        }

        db.set_last_updated()?;
        Ok(report)
    }

    /// Harvest a specific source
    pub fn harvest_source(
        &self,
        db: &Database,
        _category: &str,
        name: &str,
        source: &SourceDef,
    ) -> Result<SourceStats> {
        let content = self.fetch_source(source)?;
        let parser = self.config.get_parser(&source.parser)
            .ok_or_else(|| WixKbError::Config(format!("Unknown parser: {}", source.parser)))?;

        let mut stats = SourceStats::default();

        match parser.parser_type.as_str() {
            "xml" => {
                stats.items += self.parse_xsd(db, &content, source)?;
            }
            "json" => {
                stats.items += self.parse_json(db, &content, source, name)?;
            }
            "rules" => {
                // Infer category from source name (e.g., "component_rules" -> "component")
                let category = name.trim_end_matches("_rules").replace('_', "-");
                stats.items += self.parse_rules_with_category(db, &content, &category)?;
            }
            "migration" => {
                stats.items += self.parse_migration(db, &content)?;
            }
            "html" => {
                stats.items += self.parse_html(db, &content, source)?;
            }
            _ => {
                return Err(WixKbError::Config(format!(
                    "Unsupported parser type: {}",
                    parser.parser_type
                )));
            }
        }

        Ok(stats)
    }

    /// Fetch content from a source (URL or file)
    fn fetch_source(&self, source: &SourceDef) -> Result<String> {
        if let Some(url) = &source.url {
            self.fetch_url(url)
        } else if let Some(path) = &source.path {
            let full_path = self.base_path.join(path);
            std::fs::read_to_string(&full_path)
                .map_err(|e| WixKbError::Io(e))
        } else {
            Err(WixKbError::Config("Source has no url or path".into()))
        }
    }

    /// Fetch content from URL with caching
    fn fetch_url(&self, url: &str) -> Result<String> {
        use sha2::{Sha256, Digest};

        // Check cache
        let mut hasher = Sha256::new();
        hasher.update(url.as_bytes());
        let hash = hex::encode(hasher.finalize());
        let cache_file = self.cache_path.join(format!("{}.cache", hash));

        if cache_file.exists() {
            return std::fs::read_to_string(&cache_file)
                .map_err(|e| WixKbError::Io(e));
        }

        // Fetch from URL
        let response = reqwest::blocking::Client::new()
            .get(url)
            .header("User-Agent", &self.config.harvest.user_agent)
            .timeout(std::time::Duration::from_secs(self.config.harvest.timeout_seconds))
            .send()
            .map_err(|e| WixKbError::Network(e.to_string()))?;

        if !response.status().is_success() {
            return Err(WixKbError::Network(format!(
                "HTTP {}: {}",
                response.status(),
                url
            )));
        }

        let content = response.text()
            .map_err(|e| WixKbError::Network(e.to_string()))?;

        // Cache the content
        std::fs::write(&cache_file, &content)?;

        Ok(content)
    }

    /// Parse XSD schema
    pub fn parse_xsd(&self, db: &Database, content: &str, source: &SourceDef) -> Result<usize> {
        use roxmltree::Document;

        let doc = Document::parse(content)
            .map_err(|e| WixKbError::Parse(format!("XML parse error: {}", e)))?;

        let mut count = 0;
        let xs_ns = "http://www.w3.org/2001/XMLSchema";
        let namespace = source.extension.clone().unwrap_or_else(|| "wix".to_string());

        // First pass: collect all simple types for enum resolution
        let mut simple_types: std::collections::HashMap<String, Vec<String>> = std::collections::HashMap::new();
        for node in doc.descendants() {
            if node.tag_name().namespace() == Some(xs_ns) && node.tag_name().name() == "simpleType" {
                if let Some(type_name) = node.attribute("name") {
                    let enum_values = self.extract_xsd_enum_values(&node, xs_ns);
                    if !enum_values.is_empty() {
                        simple_types.insert(type_name.to_string(), enum_values);
                    }
                }
            }
        }

        // Second pass: parse elements with attributes and child refs
        for node in doc.descendants() {
            if node.tag_name().namespace() != Some(xs_ns) || node.tag_name().name() != "element" {
                continue;
            }

            // Only process named elements (top-level definitions)
            let elem_name = match node.attribute("name") {
                Some(n) => n,
                None => continue,
            };

            let description = self.get_xsd_documentation(&node);

            let element = Element {
                id: 0,
                name: elem_name.to_string(),
                namespace: namespace.clone(),
                since_version: Some("v4".to_string()),
                deprecated_version: None,
                description,
                documentation_url: Some(format!(
                    "https://wixtoolset.org/docs/schema/wxs/{}/",
                    elem_name.to_lowercase()
                )),
                remarks: None,
                example: None,
            };

            let element_id = db.insert_element(&element)?;
            count += 1;

            // Extract attributes from complexType
            let attributes = self.extract_xsd_attributes(&node, xs_ns, &simple_types);
            for attr in attributes {
                let attribute = Attribute {
                    id: 0,
                    element_id,
                    name: attr.0,
                    attr_type: attr.1,
                    required: attr.2,
                    default_value: attr.3,
                    description: attr.4,
                    since_version: Some("v4".to_string()),
                    deprecated_version: None,
                    enum_values: attr.5,
                };
                if let Err(e) = db.insert_attribute(&attribute) {
                    // Log but continue on duplicate attribute errors
                    eprintln!("Warning: Failed to insert attribute {}.{}: {}", elem_name, attribute.name, e);
                }
                count += 1;
            }

            // Extract child element references
            let children = self.extract_xsd_children(&node, xs_ns);
            for child_name in children {
                db.add_element_child(elem_name, &child_name)?;
            }
        }

        Ok(count)
    }

    /// Extract enum values from a simpleType
    fn extract_xsd_enum_values(&self, node: &roxmltree::Node, xs_ns: &str) -> Vec<String> {
        let mut values = Vec::new();
        for child in node.descendants() {
            if child.tag_name().namespace() == Some(xs_ns) && child.tag_name().name() == "enumeration" {
                if let Some(v) = child.attribute("value") {
                    values.push(v.to_string());
                }
            }
        }
        values
    }

    /// Extract attributes from an element's complexType
    fn extract_xsd_attributes(
        &self,
        node: &roxmltree::Node,
        xs_ns: &str,
        simple_types: &std::collections::HashMap<String, Vec<String>>,
    ) -> Vec<(String, AttributeType, bool, Option<String>, Option<String>, Vec<String>)> {
        let mut attrs = Vec::new();

        for child in node.descendants() {
            if child.tag_name().namespace() != Some(xs_ns) || child.tag_name().name() != "attribute" {
                continue;
            }

            let name = match child.attribute("name") {
                Some(n) => n.to_string(),
                None => continue,
            };

            let type_str = child.attribute("type").unwrap_or("xs:string");
            let use_val = child.attribute("use").unwrap_or("optional");
            let default = child.attribute("default").map(|s| s.to_string());
            let description = self.get_xsd_documentation(&child);

            // Resolve type and enum values
            let type_name = type_str.split(':').last().unwrap_or(type_str);
            let (attr_type, enum_values) = self.resolve_xsd_type(type_name, simple_types);

            attrs.push((
                name,
                attr_type,
                use_val == "required",
                default,
                description,
                enum_values,
            ));
        }

        attrs
    }

    /// Resolve XSD type to AttributeType
    fn resolve_xsd_type(
        &self,
        type_name: &str,
        simple_types: &std::collections::HashMap<String, Vec<String>>,
    ) -> (AttributeType, Vec<String>) {
        // Check if it's an enum type
        if let Some(values) = simple_types.get(type_name) {
            return (AttributeType::Enum, values.clone());
        }

        // Map common XSD types
        let attr_type = match type_name.to_lowercase().as_str() {
            "string" | "xs:string" => AttributeType::String,
            "guid" | "uuid" => AttributeType::Guid,
            "yesnotype" | "yesno" => AttributeType::YesNo,
            "integer" | "int" | "long" | "short" | "unsignedint" => AttributeType::Integer,
            "version" | "versiontype" => AttributeType::Version,
            "id" | "idtype" | "identifier" | "componentid" | "featureid" => AttributeType::Identifier,
            _ => AttributeType::String,
        };

        (attr_type, Vec::new())
    }

    /// Extract child element references from complexType
    fn extract_xsd_children(&self, node: &roxmltree::Node, xs_ns: &str) -> Vec<String> {
        let mut children = Vec::new();

        for child in node.descendants() {
            if child.tag_name().namespace() != Some(xs_ns) || child.tag_name().name() != "element" {
                continue;
            }

            // Look for ref attributes (references to other elements)
            if let Some(ref_name) = child.attribute("ref") {
                // Remove namespace prefix if present
                let name = ref_name.split(':').last().unwrap_or(ref_name);
                if !children.contains(&name.to_string()) {
                    children.push(name.to_string());
                }
            }
        }

        children
    }

    /// Get documentation from XSD annotation
    fn get_xsd_documentation(&self, node: &roxmltree::Node) -> Option<String> {
        for child in node.children() {
            if child.tag_name().name() == "annotation" {
                for doc in child.children() {
                    if doc.tag_name().name() == "documentation" {
                        return doc.text().map(|s| s.trim().to_string());
                    }
                }
            }
        }
        None
    }

    /// Parse JSON data file
    fn parse_json(&self, db: &Database, content: &str, source: &SourceDef, source_name: &str) -> Result<usize> {
        let value: serde_json::Value = serde_json::from_str(content)
            .map_err(|e| WixKbError::Parse(format!("JSON parse error: {}", e)))?;

        let mut count = 0;

        for target in &source.targets {
            count += match target.as_str() {
                "standard_directories" => self.import_standard_directories(db, &value)?,
                "builtin_properties" => self.import_builtin_properties(db, &value)?,
                "keywords" => self.import_keywords(db, &value)?,
                "snippets" => self.import_snippets(db, &value)?,
                "errors" => self.import_errors(db, &value)?,
                "ice_rules" => self.import_ice_rules(db, &value)?,
                "rules" => {
                    // Infer category from source name (e.g., "component_rules" -> "component")
                    let category = source_name.trim_end_matches("_rules").replace('_', "-");
                    self.parse_rules_with_category(db, content, &category)?
                }
                "msi_tables" => self.import_msi_tables(db, &value)?,
                "preprocessor_directives" => self.import_preprocessor_directives(db, &value)?,
                "standard_actions" => self.import_standard_actions(db, &value)?,
                "migrations" => self.parse_migration(db, content)?,
                _ => 0,
            };
        }

        Ok(count)
    }

    pub fn import_standard_directories(&self, db: &Database, value: &serde_json::Value) -> Result<usize> {
        let dirs = value.get("standardDirectories")
            .and_then(|v| v.as_array())
            .ok_or_else(|| WixKbError::Parse("Missing standardDirectories array".into()))?;

        let mut count = 0;
        for item in dirs {
            // Handle both string and object formats
            let name = if let Some(name_str) = item.as_str() {
                name_str.to_string()
            } else if let Some(obj) = item.as_object() {
                if let Some(id) = obj.get("id").and_then(|v| v.as_str()) {
                    id.to_string()
                } else {
                    continue;
                }
            } else {
                continue;
            };

            let description = item.get("description").and_then(|v| v.as_str());

            db.conn().execute(
                "INSERT OR IGNORE INTO standard_directories (name, description) VALUES (?1, ?2)",
                rusqlite::params![name, description],
            )?;
            count += 1;
        }
        Ok(count)
    }

    pub fn import_builtin_properties(&self, db: &Database, value: &serde_json::Value) -> Result<usize> {
        let props = value.get("builtInProperties")
            .and_then(|v| v.as_array())
            .ok_or_else(|| WixKbError::Parse("Missing builtInProperties array".into()))?;

        let mut count = 0;
        for item in props {
            // Handle both string and object formats
            let name = if let Some(name_str) = item.as_str() {
                name_str.to_string()
            } else if let Some(obj) = item.as_object() {
                if let Some(n) = obj.get("name").and_then(|v| v.as_str()) {
                    n.to_string()
                } else {
                    continue;
                }
            } else {
                continue;
            };

            let description = item.get("description").and_then(|v| v.as_str());

            db.conn().execute(
                "INSERT OR IGNORE INTO builtin_properties (name, description) VALUES (?1, ?2)",
                rusqlite::params![name, description],
            )?;
            count += 1;
        }
        Ok(count)
    }

    pub fn import_keywords(&self, db: &Database, value: &serde_json::Value) -> Result<usize> {
        let mut count = 0;

        // Elements
        if let Some(elements) = value.get("elements").and_then(|v| v.as_array()) {
            for elem in elements {
                if let Some(name) = elem.as_str() {
                    db.conn().execute(
                        "INSERT OR IGNORE INTO keywords (word, category) VALUES (?1, 'element')",
                        rusqlite::params![name],
                    )?;
                    count += 1;
                }
            }
        }

        // Preprocessor
        if let Some(directives) = value.get("preprocessor").and_then(|v| v.as_array()) {
            for dir in directives {
                if let Some(name) = dir.as_str() {
                    db.conn().execute(
                        "INSERT OR IGNORE INTO keywords (word, category) VALUES (?1, 'preprocessor')",
                        rusqlite::params![name],
                    )?;
                    count += 1;
                }
            }
        }

        Ok(count)
    }

    pub fn import_standard_actions(&self, db: &Database, value: &serde_json::Value) -> Result<usize> {
        let actions = value.get("standardActions")
            .and_then(|v| v.as_array())
            .ok_or_else(|| WixKbError::Parse("Missing standardActions array".into()))?;

        let mut count = 0;
        for action in actions {
            let name = action.get("name").and_then(|v| v.as_str());
            let sequence = action.get("sequence").and_then(|v| v.as_i64());
            let description = action.get("description").and_then(|v| v.as_str());

            if let Some(name) = name {
                db.conn().execute(
                    "INSERT OR IGNORE INTO standard_actions (name, sequence, description) VALUES (?1, ?2, ?3)",
                    rusqlite::params![name, sequence, description],
                )?;
                count += 1;
            }
        }
        Ok(count)
    }

    pub fn import_snippets(&self, db: &Database, value: &serde_json::Value) -> Result<usize> {
        let snippets = value.get("snippets")
            .and_then(|v| v.as_array())
            .ok_or_else(|| WixKbError::Parse("Missing snippets array".into()))?;

        let mut count = 0;
        for snippet in snippets {
            let prefix = snippet.get("prefix").and_then(|v| v.as_str());
            let name = snippet.get("name").and_then(|v| v.as_str());
            let description = snippet.get("description").and_then(|v| v.as_str());

            // Handle body as either string or array of strings
            let body = snippet.get("body").and_then(|v| {
                if let Some(s) = v.as_str() {
                    Some(s.to_string())
                } else if let Some(arr) = v.as_array() {
                    Some(arr.iter()
                        .filter_map(|line| line.as_str())
                        .collect::<Vec<_>>()
                        .join("\n"))
                } else {
                    None
                }
            });

            if let (Some(prefix), Some(name), Some(body)) = (prefix, name, body) {
                db.conn().execute(
                    "INSERT OR IGNORE INTO snippets (prefix, name, description, body, scope)
                     VALUES (?1, ?2, ?3, ?4, 'wxs')",
                    rusqlite::params![prefix, name, description, body],
                )?;
                count += 1;
            }
        }
        Ok(count)
    }

    pub fn import_errors(&self, db: &Database, value: &serde_json::Value) -> Result<usize> {
        let errors = value.get("errors")
            .and_then(|v| v.as_array())
            .ok_or_else(|| WixKbError::Parse("Missing errors array".into()))?;

        let mut count = 0;
        for error in errors {
            let code = error.get("code").and_then(|v| v.as_str());
            let severity = error.get("severity").and_then(|v| v.as_str()).unwrap_or("error");
            let message = error.get("message").and_then(|v| v.as_str());
            let description = error.get("description").and_then(|v| v.as_str());
            let resolution = error.get("resolution").and_then(|v| v.as_str());

            if let (Some(code), Some(message)) = (code, message) {
                db.conn().execute(
                    "INSERT OR IGNORE INTO errors (code, severity, message_template, description, resolution)
                     VALUES (?1, ?2, ?3, ?4, ?5)",
                    rusqlite::params![code, severity, message, description, resolution],
                )?;
                count += 1;
            }
        }
        Ok(count)
    }

    pub fn import_msi_tables(&self, db: &Database, value: &serde_json::Value) -> Result<usize> {
        let tables = value.get("msiTables")
            .and_then(|v| v.as_array())
            .ok_or_else(|| WixKbError::Parse("Missing msiTables array".into()))?;

        let mut count = 0;
        for table in tables {
            let name = table.get("name").and_then(|v| v.as_str());
            let description = table.get("description").and_then(|v| v.as_str());
            let required = table.get("required").and_then(|v| v.as_bool()).unwrap_or(false);

            if let Some(name) = name {
                db.conn().execute(
                    "INSERT OR IGNORE INTO msi_tables (name, description, required)
                     VALUES (?1, ?2, ?3)",
                    rusqlite::params![name, description, required as i32],
                )?;
                count += 1;
            }
        }
        Ok(count)
    }

    pub fn import_preprocessor_directives(&self, db: &Database, value: &serde_json::Value) -> Result<usize> {
        let directives = value.get("preprocessor")
            .and_then(|v| v.as_array())
            .ok_or_else(|| WixKbError::Parse("Missing preprocessor array".into()))?;

        let mut count = 0;
        for dir in directives {
            let directive = dir.get("directive").and_then(|v| v.as_str());
            let syntax = dir.get("syntax").and_then(|v| v.as_str());
            let description = dir.get("description").and_then(|v| v.as_str());

            if let Some(name) = directive {
                db.conn().execute(
                    "INSERT OR IGNORE INTO preprocessor_directives (name, syntax, description)
                     VALUES (?1, ?2, ?3)",
                    rusqlite::params![name, syntax, description],
                )?;
                count += 1;
            }
        }
        Ok(count)
    }

    pub fn import_ice_rules(&self, db: &Database, value: &serde_json::Value) -> Result<usize> {
        let rules = value.get("iceErrors")
            .and_then(|v| v.as_array())
            .ok_or_else(|| WixKbError::Parse("Missing iceErrors array".into()))?;

        let mut count = 0;
        for rule in rules {
            let code = rule.get("code").and_then(|v| v.as_str());
            let severity = rule.get("severity").and_then(|v| v.as_str()).unwrap_or("error");
            let description = rule.get("description").and_then(|v| v.as_str());
            let resolution = rule.get("resolution").and_then(|v| v.as_str());
            let tables = rule.get("tables")
                .and_then(|v| v.as_array())
                .map(|arr| arr.iter()
                    .filter_map(|v| v.as_str())
                    .collect::<Vec<_>>()
                    .join(","))
                .unwrap_or_default();

            if let Some(code) = code {
                db.conn().execute(
                    "INSERT OR IGNORE INTO ice_rules (code, severity, description, resolution, tables_affected)
                     VALUES (?1, ?2, ?3, ?4, ?5)",
                    rusqlite::params![code, severity, description, resolution, tables],
                )?;
                count += 1;
            }
        }
        Ok(count)
    }

    /// Parse rules JSON
    pub fn parse_rules(&self, db: &Database, content: &str) -> Result<usize> {
        self.parse_rules_with_category(db, content, "general")
    }

    /// Parse rules JSON with a default category
    pub fn parse_rules_with_category(&self, db: &Database, content: &str, default_category: &str) -> Result<usize> {
        let value: serde_json::Value = serde_json::from_str(content)
            .map_err(|e| WixKbError::Parse(format!("JSON parse error: {}", e)))?;

        let rules = value.get("rules")
            .and_then(|v| v.as_array())
            .ok_or_else(|| WixKbError::Parse("Missing rules array".into()))?;

        let mut count = 0;
        for rule_value in rules {
            let rule_id = rule_value.get("id").and_then(|v| v.as_str());
            let category = rule_value.get("category")
                .and_then(|v| v.as_str())
                .unwrap_or(default_category);
            let severity = rule_value.get("severity").and_then(|v| v.as_str()).unwrap_or("warning");
            let name = rule_value.get("name").and_then(|v| v.as_str());
            let description = rule_value.get("description").and_then(|v| v.as_str());
            let rationale = rule_value.get("rationale").and_then(|v| v.as_str());
            let fix = rule_value.get("fix").and_then(|v| {
                // Handle fix as either string or object
                if let Some(s) = v.as_str() {
                    Some(s.to_string())
                } else if v.is_object() {
                    // Serialize the fix object as JSON
                    serde_json::to_string(v).ok()
                } else {
                    None
                }
            });

            if let (Some(rule_id), Some(name)) = (rule_id, name) {
                let rule = Rule {
                    id: 0,
                    rule_id: rule_id.to_string(),
                    category: category.to_string(),
                    severity: Severity::from(severity),
                    name: name.to_string(),
                    description: description.map(|s| s.to_string()),
                    rationale: rationale.map(|s| s.to_string()),
                    fix_suggestion: fix,
                    enabled: true,
                    auto_fixable: false,
                    conditions: Vec::new(),
                    condition: None,
                    target_kind: None,
                    target_name: None,
                    tags: None,
                };
                db.insert_rule(&rule)?;
                count += 1;
            }
        }
        Ok(count)
    }

    /// Parse migration JSON
    pub fn parse_migration(&self, db: &Database, content: &str) -> Result<usize> {
        let value: serde_json::Value = serde_json::from_str(content)
            .map_err(|e| WixKbError::Parse(format!("JSON parse error: {}", e)))?;

        let from = value.get("from").and_then(|v| v.as_str()).unwrap_or("v3");
        let to = value.get("to").and_then(|v| v.as_str()).unwrap_or("v4");

        let changes = value.get("changes")
            .and_then(|v| v.as_array())
            .ok_or_else(|| WixKbError::Parse("Missing changes array".into()))?;

        let mut count = 0;
        for change in changes {
            let change_type = change.get("type").and_then(|v| v.as_str());
            let old_value = change.get("old").or_else(|| change.get("element")).and_then(|v| v.as_str());
            let new_value = change.get("new").and_then(|v| v.as_str());
            let notes = change.get("notes").or_else(|| change.get("migration")).and_then(|v| v.as_str());

            if let Some(change_type) = change_type {
                db.conn().execute(
                    "INSERT INTO migrations (from_version, to_version, change_type, old_value, new_value, notes)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                    rusqlite::params![from, to, change_type, old_value, new_value, notes],
                )?;
                count += 1;
            }
        }
        Ok(count)
    }

    /// Parse HTML documentation page
    fn parse_html(&self, db: &Database, content: &str, source: &SourceDef) -> Result<usize> {
        let mut count = 0;

        for target in &source.targets {
            count += match target.as_str() {
                "ice_rules" => {
                    let rules = parsers::html::parse_ice_rules(content)?;
                    let count = rules.len();
                    for rule in rules {
                        db.conn().execute(
                            "INSERT OR IGNORE INTO ice_rules (code, severity, description)
                             VALUES (?1, ?2, ?3)",
                            rusqlite::params![rule.code, rule.severity, rule.description],
                        )?;
                    }
                    count
                }
                "msi_tables" => {
                    let tables = parsers::html::parse_msi_tables(content)?;
                    for table in &tables {
                        db.conn().execute(
                            "INSERT OR IGNORE INTO msi_tables (name, description, required)
                             VALUES (?1, ?2, ?3)",
                            rusqlite::params![table.name, table.description, table.required as i32],
                        )?;
                    }
                    tables.len()
                }
                "standard_directories" => {
                    let dirs = parsers::html::parse_standard_directories(content)?;
                    for dir in &dirs {
                        db.conn().execute(
                            "INSERT OR IGNORE INTO standard_directories (name, description)
                             VALUES (?1, ?2)",
                            rusqlite::params![dir.name, dir.description],
                        )?;
                    }
                    dirs.len()
                }
                "builtin_properties" => {
                    let props = parsers::html::parse_builtin_properties(content)?;
                    for prop in &props {
                        db.conn().execute(
                            "INSERT OR IGNORE INTO builtin_properties (name, description)
                             VALUES (?1, ?2)",
                            rusqlite::params![prop.name, prop.description],
                        )?;
                    }
                    props.len()
                }
                _ => 0,
            };
        }

        Ok(count)
    }
}

/// Harvest report
#[derive(Debug, Default)]
pub struct HarvestReport {
    pub sources_processed: usize,
    pub items_harvested: usize,
    pub source_stats: Vec<(String, SourceStats)>,
    pub errors: Vec<String>,
}

/// Statistics for a single source
#[derive(Debug, Default)]
pub struct SourceStats {
    pub items: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_harvester_parse_json_keywords() {
        let db = Database::open_memory().unwrap();

        let json = r#"{
            "elements": ["Package", "Component"],
            "preprocessor": ["define", "if"]
        }"#;

        let source = SourceDef {
            url: None,
            path: None,
            parser: "json".to_string(),
            targets: vec!["keywords".to_string()],
            extension: None,
        };

        // Direct test of import function
        let value: serde_json::Value = serde_json::from_str(json).unwrap();

        let dir = tempdir().unwrap();
        let config_content = r#"
version: "1.0"
sources: {}
parsers:
  json:
    type: json
harvest:
  cache_dir: ".cache"
  timeout_seconds: 30
  retry_count: 3
  user_agent: "test"
  rate_limit:
    requests_per_second: 2
    burst: 5
"#;
        let config_path = dir.path().join("sources.yaml");
        std::fs::write(&config_path, config_content).unwrap();
        let base_path = dir.path().to_path_buf();

        let harvester = Harvester::new(&config_path, &base_path).unwrap();
        let count = harvester.import_keywords(&db, &value).unwrap();

        assert_eq!(count, 4);

        let element_keywords = db.get_keywords("element").unwrap();
        assert!(element_keywords.contains(&"Package".to_string()));
        assert!(element_keywords.contains(&"Component".to_string()));
    }
}
