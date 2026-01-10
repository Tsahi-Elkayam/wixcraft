//! Data harvesting from various sources

pub mod parsers;

use crate::config::{SourceDef, SourcesConfig};
use crate::db::Database;
use crate::models::*;
use crate::{Result, WixDataError};
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

        // Process all categories except "enrichments" first
        for (category, sources) in &self.config.sources {
            if category == "enrichments" {
                continue; // Process enrichments last
            }
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

        // Process enrichments last - they update existing records
        if let Some(sources) = self.config.sources.get("enrichments") {
            for (name, source) in sources {
                match self.harvest_source(db, "enrichments", name, source) {
                    Ok(stats) => {
                        report.sources_processed += 1;
                        report.items_harvested += stats.items;
                        report.source_stats.push((name.clone(), stats));
                    }
                    Err(e) => {
                        report.errors.push(format!("enrichments/{}: {}", name, e));
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
            .ok_or_else(|| WixDataError::Config(format!("Unknown parser: {}", source.parser)))?;

        let mut stats = SourceStats::default();

        match parser.parser_type.as_str() {
            "xml" => {
                stats.items += self.parse_xsd(db, &content, source)?;
            }
            "json" => {
                stats.items += self.parse_json(db, &content, source, name)?;
            }
            "html" => {
                stats.items += self.parse_html(db, &content, source, name)?;
            }
            "wxs" => {
                stats.items += self.parse_wxs(db, &content, source)?;
            }
            "csharp" | "cpp" => {
                stats.items += self.parse_source_code(db, &content, source, name)?;
            }
            "markdown" => {
                stats.items += self.parse_markdown(db, &content, source, name)?;
            }
            "rules" => {
                let category = name.trim_end_matches("_rules").replace('_', "-");
                stats.items += self.parse_rules_with_category(db, &content, &category)?;
            }
            "migration" => {
                stats.items += self.parse_migration(db, &content)?;
            }
            _ => {
                return Err(WixDataError::Config(format!(
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
                .map_err(WixDataError::Io)
        } else {
            Err(WixDataError::Config("Source has no url or path".into()))
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
                .map_err(WixDataError::Io);
        }

        // Fetch from URL - reqwest handles gzip automatically when Accept-Encoding is set
        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(self.config.harvest.timeout_seconds))
            .build()
            .map_err(|e: reqwest::Error| WixDataError::Network(e.to_string()))?;

        let response = client
            .get(url)
            .header("User-Agent", &self.config.harvest.user_agent)
            .header("Accept", "application/json, text/xml, text/html, */*")
            .send()
            .map_err(|e: reqwest::Error| WixDataError::Network(e.to_string()))?;

        if !response.status().is_success() {
            return Err(WixDataError::Network(format!(
                "HTTP {}: {}",
                response.status(),
                url
            )));
        }

        // Get content - reqwest handles gzip decompression automatically
        let content = response.text()
            .map_err(|e: reqwest::Error| WixDataError::Network(e.to_string()))?;

        // Cache the content
        std::fs::write(&cache_file, &content)?;

        Ok(content)
    }

    /// Parse XSD schema - extracts elements, attributes, and relationships
    pub fn parse_xsd(&self, db: &Database, content: &str, source: &SourceDef) -> Result<usize> {
        use roxmltree::Document;

        let doc = Document::parse(content)
            .map_err(|e| WixDataError::Parse(format!("XML parse error: {}", e)))?;

        let mut count = 0;
        let xs_ns = "http://www.w3.org/2001/XMLSchema";
        let namespace = source.extension.clone().unwrap_or_else(|| "wix".to_string());

        // First pass: collect all elements
        let mut element_ids: std::collections::HashMap<String, i64> = std::collections::HashMap::new();

        for node in doc.descendants() {
            if node.tag_name().namespace() != Some(xs_ns) || node.tag_name().name() != "element" {
                continue;
            }

            let elem_name = match node.attribute("name") {
                Some(n) => n,
                None => continue,
            };

            let element = Element {
                id: 0,
                name: elem_name.to_string(),
                namespace: namespace.clone(),
                since_version: Some("v4".to_string()),
                deprecated_version: None,
                description: self.get_xsd_documentation(&node),
                documentation_url: Some(format!(
                    "https://wixtoolset.org/docs/schema/wxs/{}/",
                    elem_name.to_lowercase()
                )),
                remarks: None,
                example: None,
            };

            if let Ok(id) = db.insert_element(&element) {
                element_ids.insert(elem_name.to_string(), id);
                count += 1;
            }

            // Extract attributes from this element's complexType
            count += self.extract_element_attributes(db, &node, elem_name, xs_ns)?;
        }

        // Second pass: extract child element references from complexType/sequence
        for node in doc.descendants() {
            if node.tag_name().namespace() != Some(xs_ns) || node.tag_name().name() != "element" {
                continue;
            }

            let parent_name = match node.attribute("name") {
                Some(n) => n,
                None => continue,
            };

            let parent_id = match element_ids.get(parent_name) {
                Some(id) => *id,
                None => continue,
            };

            // Find child element references in sequence/choice/all
            self.extract_child_elements(db, &node, parent_id, &element_ids, xs_ns)?;
        }

        Ok(count)
    }

    /// Extract attributes from an element's complexType definition
    fn extract_element_attributes(&self, db: &Database, elem_node: &roxmltree::Node, elem_name: &str, xs_ns: &str) -> Result<usize> {
        let mut count = 0;

        // Get element ID
        let elem_id: i64 = db.conn().query_row(
            "SELECT id FROM elements WHERE name = ?1",
            rusqlite::params![elem_name],
            |row| row.get(0),
        ).unwrap_or(0);

        if elem_id == 0 {
            return Ok(0);
        }

        // Find attributes in complexType
        for node in elem_node.descendants() {
            if node.tag_name().namespace() != Some(xs_ns) || node.tag_name().name() != "attribute" {
                continue;
            }

            let attr_name = match node.attribute("name") {
                Some(n) => n,
                None => continue,
            };

            let attr_type = node.attribute("type").unwrap_or("xs:string");
            let required = node.attribute("use").map(|u| u == "required").unwrap_or(false);
            let default_val = node.attribute("default");
            let description = self.get_xsd_documentation(&node);

            // Map XSD types to our type system
            let mapped_type = match attr_type {
                t if t.contains("YesNoType") || t.contains("yesno") => "yesno",
                t if t.contains("Guid") || t.contains("guid") => "guid",
                t if t.contains("Integer") || t.contains("int") || t.contains("Long") => "integer",
                t if t.contains("Version") => "version",
                t if t.contains("Identifier") || t.ends_with("Id") => "identifier",
                _ => "string",
            };

            db.conn().execute(
                "INSERT OR IGNORE INTO attributes (element_id, name, attr_type, required, default_value, description)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                rusqlite::params![elem_id, attr_name, mapped_type, required as i32, default_val, description],
            )?;
            count += 1;

            // Extract enum values if this is a simpleType restriction
            self.extract_enum_values(db, &node, xs_ns)?;
        }

        Ok(count)
    }

    /// Extract child element references from sequence/choice/all
    fn extract_child_elements(
        &self,
        db: &Database,
        elem_node: &roxmltree::Node,
        parent_id: i64,
        element_ids: &std::collections::HashMap<String, i64>,
        xs_ns: &str,
    ) -> Result<()> {
        for node in elem_node.descendants() {
            // Look for element references
            if node.tag_name().namespace() == Some(xs_ns) && node.tag_name().name() == "element" {
                // Check for ref attribute (reference to another element)
                if let Some(ref_name) = node.attribute("ref") {
                    // Strip namespace prefix if present
                    let child_name = ref_name.split(':').last().unwrap_or(ref_name);
                    if let Some(&child_id) = element_ids.get(child_name) {
                        let min_occurs: i32 = node.attribute("minOccurs")
                            .and_then(|s| s.parse().ok())
                            .unwrap_or(1);
                        let max_occurs: Option<i32> = node.attribute("maxOccurs")
                            .and_then(|s| if s == "unbounded" { None } else { s.parse().ok() });

                        db.conn().execute(
                            "INSERT OR IGNORE INTO element_children (element_id, child_id, min_occurs, max_occurs)
                             VALUES (?1, ?2, ?3, ?4)",
                            rusqlite::params![parent_id, child_id, min_occurs, max_occurs],
                        )?;

                        db.conn().execute(
                            "INSERT OR IGNORE INTO element_parents (element_id, parent_id)
                             VALUES (?1, ?2)",
                            rusqlite::params![child_id, parent_id],
                        )?;
                    }
                }
            }
        }
        Ok(())
    }

    /// Extract enum values from simpleType restrictions
    fn extract_enum_values(&self, db: &Database, attr_node: &roxmltree::Node, xs_ns: &str) -> Result<()> {
        // Get the attribute ID
        let attr_name = match attr_node.attribute("name") {
            Some(n) => n,
            None => return Ok(()),
        };

        let attr_id: i64 = db.conn().query_row(
            "SELECT id FROM attributes WHERE name = ?1 ORDER BY id DESC LIMIT 1",
            rusqlite::params![attr_name],
            |row| row.get(0),
        ).unwrap_or(0);

        if attr_id == 0 {
            return Ok(());
        }

        // Look for enumeration values in simpleType/restriction
        for node in attr_node.descendants() {
            if node.tag_name().namespace() == Some(xs_ns) && node.tag_name().name() == "enumeration" {
                if let Some(value) = node.attribute("value") {
                    let description = self.get_xsd_documentation(&node);
                    db.conn().execute(
                        "INSERT OR IGNORE INTO attribute_enum_values (attribute_id, value, description)
                         VALUES (?1, ?2, ?3)",
                        rusqlite::params![attr_id, value, description],
                    )?;
                }
            }
        }

        Ok(())
    }

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
            .map_err(|e| WixDataError::Parse(format!("JSON parse error: {}", e)))?;

        let mut count = 0;

        for target in &source.targets {
            count += match target.as_str() {
                "keywords" => self.import_keywords(db, &value)?,
                "snippets" => self.import_snippets(db, &value)?,
                "rules" => {
                    let category = source_name.trim_end_matches("_rules").replace('_', "-");
                    self.parse_rules_with_category(db, content, &category)?
                }
                "ice_rules" => self.import_ice_rules(db, &value)?,
                "directories" => self.import_standard_directories(db, &value)?,
                "properties" => self.import_builtin_properties(db, &value)?,
                "msi_tables" => self.import_msi_tables(db, &value)?,
                "errors" => self.import_wix_errors(db, &value)?,
                "standard_actions" => self.import_standard_actions(db, &value)?,
                "preprocessor" => self.import_preprocessor(db, &value)?,
                "extensions" => self.import_extensions(db, &value)?,
                "extension_elements" => self.import_extension_elements(db, &value)?,
                "prerequisites" => self.import_prerequisites(db, &value)?,
                "additional_rules" => {
                    let category = source_name.trim_end_matches("_rules").replace('_', "-");
                    self.parse_rules_with_category(db, content, &category)?
                }
                "extension_snippets" => self.import_snippets(db, &value)?,
                "ui_elements" => self.import_ui_elements(db, &value)?,
                "rule_conditions" => self.import_rule_conditions(db, &value)?,
                "element_descriptions_patch" => self.import_element_descriptions_patch(db, &value)?,
                "attribute_descriptions_patch" => self.import_attribute_descriptions_patch(db, &value)?,
                "ui_elements_patch" => self.import_ui_elements_patch(db, &value)?,
                "documentation" => self.import_documentation(db, &value)?,
                "custom_action_types" => self.import_custom_action_types(db, &value)?,
                "condition_operators" => self.import_condition_operators(db, &value)?,
                "burn_variables" => self.import_burn_variables(db, &value)?,
                "launch_condition_patterns" => self.import_launch_condition_patterns(db, &value)?,
                "cli_commands" => self.import_cli_commands(db, &value)?,
                "localization_cultures" => self.import_localization_cultures(db, &value)?,
                "patching_reference" => self.import_reference_docs(db, &value, "patching")?,
                "transform_reference" => self.import_reference_docs(db, &value, "transforms")?,
                "windows_builds" => self.import_windows_builds(db, &value)?,
                "ui_dialogs" => self.import_wixui_dialogs(db, &value)?,
                "msiexec" => self.import_msiexec_reference(db, &value)?,
                "attribute_enums" => self.import_attribute_enums(db, &value)?,
                "migrations" => self.import_migrations(db, &value)?,
                "service_reference" => self.import_reference_docs(db, &value, "service")?,
                "msi_table_descriptions" => self.import_msi_table_descriptions(db, &value)?,
                "error_resolutions" => self.import_error_resolutions(db, &value)?,
                "ice_resolutions" => self.import_ice_resolutions(db, &value)?,
                // Enriched data targets
                "element_enrichments" => self.import_element_enrichments(db, &value)?,
                "rule_enrichments" => self.import_rule_enrichments(db, &value)?,
                "enum_descriptions" => self.import_enum_descriptions(db, &value)?,
                "msi_table_definitions" => self.import_msi_table_definitions(db, &value)?,
                "element_parents" => self.import_element_parents_json(db, &value)?,
                "sources" => self.import_sources(db, &value)?,
                "standard_directories_enriched" => self.import_standard_directories_enriched(db, &value)?,
                "migration_notes" => self.import_migration_notes(db, &value)?,
                "cli_commands_enriched" => self.import_cli_commands_enriched(db, &value)?,
                _ => 0,
            };
        }

        Ok(count)
    }

    /// Import element enrichments (examples and remarks)
    pub fn import_element_enrichments(&self, db: &Database, value: &serde_json::Value) -> Result<usize> {
        let enrichments = value.get("element_enrichments")
            .and_then(|v| v.as_array())
            .ok_or_else(|| WixDataError::Parse("Missing element_enrichments array".into()))?;

        let mut count = 0;
        for item in enrichments {
            let name = item.get("name").and_then(|v| v.as_str());
            let example = item.get("example").and_then(|v| v.as_str());
            let remarks = item.get("remarks").and_then(|v| v.as_str());

            if let Some(name) = name {
                if example.is_some() || remarks.is_some() {
                    db.conn().execute(
                        "UPDATE elements SET example = COALESCE(?1, example), remarks = COALESCE(?2, remarks) WHERE name = ?3",
                        rusqlite::params![example, remarks, name],
                    )?;
                    count += 1;
                }
            }
        }
        Ok(count)
    }

    /// Import rule enrichments (rationales, fix suggestions)
    pub fn import_rule_enrichments(&self, db: &Database, value: &serde_json::Value) -> Result<usize> {
        let enrichments = value.get("rule_enrichments")
            .and_then(|v| v.as_array())
            .ok_or_else(|| WixDataError::Parse("Missing rule_enrichments array".into()))?;

        let mut count = 0;
        for item in enrichments {
            let rule_id = item.get("rule_id").and_then(|v| v.as_str());
            let rationale = item.get("rationale").and_then(|v| v.as_str());
            let fix_suggestion = item.get("fix_suggestion").and_then(|v| v.as_str());
            let auto_fixable = item.get("auto_fixable").and_then(|v| v.as_bool()).unwrap_or(false);

            if let Some(rule_id) = rule_id {
                db.conn().execute(
                    "UPDATE rules SET rationale = ?1, fix_suggestion = ?2, auto_fixable = ?3 WHERE rule_id = ?4",
                    rusqlite::params![rationale, fix_suggestion, auto_fixable as i32, rule_id],
                )?;
                count += 1;
            }
        }
        Ok(count)
    }

    /// Import enum value descriptions
    pub fn import_enum_descriptions(&self, db: &Database, value: &serde_json::Value) -> Result<usize> {
        let descriptions = value.get("enum_descriptions")
            .and_then(|v| v.as_array())
            .ok_or_else(|| WixDataError::Parse("Missing enum_descriptions array".into()))?;

        let mut count = 0;
        for item in descriptions {
            let element = item.get("element").and_then(|v| v.as_str());
            let attribute = item.get("attribute").and_then(|v| v.as_str());
            let enum_value = item.get("value").and_then(|v| v.as_str());
            let description = item.get("description").and_then(|v| v.as_str());

            if let (Some(element), Some(attribute), Some(enum_value), Some(description)) = (element, attribute, enum_value, description) {
                db.conn().execute(
                    "UPDATE attribute_enum_values SET description = ?1
                     WHERE id IN (
                         SELECT ev.id FROM attribute_enum_values ev
                         JOIN attributes a ON ev.attribute_id = a.id
                         JOIN elements e ON a.element_id = e.id
                         WHERE e.name = ?2 AND a.name = ?3 AND ev.value = ?4
                     )",
                    rusqlite::params![description, element, attribute, enum_value],
                )?;
                count += 1;
            }
        }
        Ok(count)
    }

    /// Import MSI table column definitions
    pub fn import_msi_table_definitions(&self, db: &Database, value: &serde_json::Value) -> Result<usize> {
        let tables = value.get("msi_tables")
            .and_then(|v| v.as_array())
            .ok_or_else(|| WixDataError::Parse("Missing msi_tables array".into()))?;

        let mut count = 0;
        for table in tables {
            let name = table.get("name").and_then(|v| v.as_str());
            let description = table.get("description").and_then(|v| v.as_str());
            let required = table.get("required").and_then(|v| v.as_bool()).unwrap_or(false);
            let columns = table.get("columns");
            let doc_url = table.get("documentation_url").and_then(|v| v.as_str());

            if let Some(name) = name {
                let columns_json = columns.map(|c| c.to_string());
                db.conn().execute(
                    "INSERT OR REPLACE INTO msi_tables (name, description, required, columns, documentation_url)
                     VALUES (?1, ?2, ?3, ?4, ?5)",
                    rusqlite::params![name, description, required as i32, columns_json, doc_url],
                )?;
                count += 1;
            }
        }
        Ok(count)
    }

    /// Import element parent relationships from JSON
    pub fn import_element_parents_json(&self, db: &Database, value: &serde_json::Value) -> Result<usize> {
        let parents = value.get("element_parents")
            .and_then(|v| v.as_array())
            .ok_or_else(|| WixDataError::Parse("Missing element_parents array".into()))?;

        let mut count = 0;
        for item in parents {
            let element = item.get("element").and_then(|v| v.as_str());
            let parent_list = item.get("parents").and_then(|v| v.as_array());

            if let (Some(element), Some(parent_list)) = (element, parent_list) {
                for parent in parent_list {
                    if let Some(parent_name) = parent.as_str() {
                        db.conn().execute(
                            "INSERT OR IGNORE INTO element_parents (element_id, parent_id)
                             SELECT e.id, p.id FROM elements e, elements p
                             WHERE e.name = ?1 AND p.name = ?2",
                            rusqlite::params![element, parent_name],
                        )?;
                        count += 1;
                    }
                }
            }
        }
        Ok(count)
    }

    /// Import data sources
    pub fn import_sources(&self, db: &Database, value: &serde_json::Value) -> Result<usize> {
        let sources = value.get("sources")
            .and_then(|v| v.as_array())
            .ok_or_else(|| WixDataError::Parse("Missing sources array".into()))?;

        let mut count = 0;
        for source in sources {
            let name = source.get("name").and_then(|v| v.as_str());
            let url = source.get("url").and_then(|v| v.as_str());
            let source_type = source.get("source_type").and_then(|v| v.as_str());
            let enabled = source.get("enabled").and_then(|v| v.as_bool()).unwrap_or(true);

            if let (Some(name), Some(source_type)) = (name, source_type) {
                db.conn().execute(
                    "INSERT OR REPLACE INTO sources (name, url, source_type, enabled)
                     VALUES (?1, ?2, ?3, ?4)",
                    rusqlite::params![name, url, source_type, enabled as i32],
                )?;
                count += 1;
            }
        }
        Ok(count)
    }

    /// Import enriched standard directories
    pub fn import_standard_directories_enriched(&self, db: &Database, value: &serde_json::Value) -> Result<usize> {
        let directories = value.get("standard_directories")
            .and_then(|v| v.as_array())
            .ok_or_else(|| WixDataError::Parse("Missing standard_directories array".into()))?;

        let mut count = 0;
        for dir in directories {
            let name = dir.get("name").and_then(|v| v.as_str());
            let description = dir.get("description").and_then(|v| v.as_str());
            let windows_path = dir.get("windows_path").and_then(|v| v.as_str());
            let example = dir.get("example").and_then(|v| v.as_str());

            if let Some(name) = name {
                db.conn().execute(
                    "INSERT OR REPLACE INTO standard_directories (name, description, windows_path, example)
                     VALUES (?1, ?2, ?3, ?4)",
                    rusqlite::params![name, description, windows_path, example],
                )?;
                count += 1;
            }
        }
        Ok(count)
    }

    /// Import migration notes
    pub fn import_migration_notes(&self, db: &Database, value: &serde_json::Value) -> Result<usize> {
        let migrations = value.get("migrations")
            .and_then(|v| v.as_array())
            .ok_or_else(|| WixDataError::Parse("Missing migrations array".into()))?;

        let mut count = 0;
        for migration in migrations {
            let from_version = migration.get("from_version").and_then(|v| v.as_str());
            let to_version = migration.get("to_version").and_then(|v| v.as_str());
            let change_type = migration.get("change_type").and_then(|v| v.as_str());
            let old_value = migration.get("old_value").and_then(|v| v.as_str());
            let _new_value = migration.get("new_value").and_then(|v| v.as_str());
            let notes = migration.get("notes").and_then(|v| v.as_str());

            if let (Some(from_version), Some(to_version), Some(change_type)) = (from_version, to_version, change_type) {
                db.conn().execute(
                    "UPDATE migrations SET notes = ?1
                     WHERE from_version = ?2 AND to_version = ?3 AND change_type = ?4
                       AND (old_value = ?5 OR (old_value IS NULL AND ?5 IS NULL))",
                    rusqlite::params![notes, from_version, to_version, change_type, old_value],
                )?;
                count += 1;
            }
        }
        Ok(count)
    }

    /// Import enriched CLI commands
    pub fn import_cli_commands_enriched(&self, db: &Database, value: &serde_json::Value) -> Result<usize> {
        let commands = value.get("cli_commands")
            .and_then(|v| v.as_array())
            .ok_or_else(|| WixDataError::Parse("Missing cli_commands array".into()))?;

        let mut count = 0;
        for cmd in commands {
            let name = cmd.get("name").and_then(|v| v.as_str());
            let description = cmd.get("description").and_then(|v| v.as_str());
            let syntax = cmd.get("syntax").and_then(|v| v.as_str());
            let category = cmd.get("category").and_then(|v| v.as_str());

            if let Some(name) = name {
                db.conn().execute(
                    "INSERT OR REPLACE INTO cli_commands (name, description, syntax, category)
                     VALUES (?1, ?2, ?3, ?4)",
                    rusqlite::params![name, description, syntax, category],
                )?;
                count += 1;
            }
        }
        Ok(count)
    }

    /// Import custom action types
    pub fn import_custom_action_types(&self, db: &Database, value: &serde_json::Value) -> Result<usize> {
        let mut count = 0;

        // Import custom action types
        if let Some(types) = value.get("custom_action_types").and_then(|v| v.as_array()) {
            for ca_type in types {
                let type_num = ca_type.get("type").and_then(|v| v.as_i64());
                let source_type = ca_type.get("source").and_then(|v| v.as_str());
                let target_type = ca_type.get("target").and_then(|v| v.as_str());
                let description = ca_type.get("description").and_then(|v| v.as_str());
                let execution = ca_type.get("execution").and_then(|v| v.as_str());
                let example = ca_type.get("example").and_then(|v| v.as_str());

                if let (Some(type_num), Some(source_type), Some(target_type)) = (type_num, source_type, target_type) {
                    db.conn().execute(
                        "INSERT OR REPLACE INTO custom_action_types (type_num, source_type, target_type, description, execution, example)
                         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                        rusqlite::params![type_num, source_type, target_type, description, execution, example],
                    )?;
                    count += 1;
                }
            }
        }

        // Import execution options
        if let Some(options) = value.get("execution_options").and_then(|v| v.as_array()) {
            for opt in options {
                let flag = opt.get("value").and_then(|v| v.as_i64());
                let name = opt.get("name").and_then(|v| v.as_str());
                let description = opt.get("description").and_then(|v| v.as_str());

                if let (Some(flag), Some(name)) = (flag, name) {
                    db.conn().execute(
                        "INSERT OR REPLACE INTO custom_action_options (flag, name, description)
                         VALUES (?1, ?2, ?3)",
                        rusqlite::params![flag, name, description],
                    )?;
                    count += 1;
                }
            }
        }

        Ok(count)
    }

    /// Import condition operators
    pub fn import_condition_operators(&self, db: &Database, value: &serde_json::Value) -> Result<usize> {
        let mut count = 0;

        let categories = [
            ("logical_operators", "logical"),
            ("comparison_operators", "comparison"),
            ("substring_operators", "substring"),
            ("bitwise_operators", "bitwise"),
        ];

        for (key, category) in categories {
            if let Some(ops) = value.get(key).and_then(|v| v.as_array()) {
                for op in ops {
                    let operator = op.get("operator").and_then(|v| v.as_str());
                    let description = op.get("description").and_then(|v| v.as_str());
                    let example = op.get("example").and_then(|v| v.as_str());
                    let precedence = op.get("precedence").and_then(|v| v.as_i64());
                    let notes = op.get("notes").and_then(|v| v.as_str());

                    if let Some(operator) = operator {
                        db.conn().execute(
                            "INSERT OR REPLACE INTO condition_operators (operator, category, description, example, precedence, notes)
                             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                            rusqlite::params![operator, category, description, example, precedence, notes],
                        )?;
                        count += 1;
                    }
                }
            }
        }

        // Import feature/component operators
        if let Some(ops) = value.get("feature_component_operators").and_then(|v| v.as_array()) {
            for op in ops {
                let operator = op.get("operator").and_then(|v| v.as_str());
                let description = op.get("description").and_then(|v| v.as_str());
                let example = op.get("example").and_then(|v| v.as_str());
                let notes = op.get("notes").and_then(|v| v.as_str());

                if let Some(operator) = operator {
                    db.conn().execute(
                        "INSERT OR REPLACE INTO condition_operators (operator, category, description, example, precedence, notes)
                         VALUES (?1, 'feature_component', ?2, ?3, NULL, ?4)",
                        rusqlite::params![operator, description, example, notes],
                    )?;
                    count += 1;
                }
            }
        }

        Ok(count)
    }

    /// Import Burn variables
    pub fn import_burn_variables(&self, db: &Database, value: &serde_json::Value) -> Result<usize> {
        let mut count = 0;

        // Import builtin variables
        if let Some(vars) = value.get("builtin_variables").and_then(|v| v.as_array()) {
            for var in vars {
                let name = var.get("name").and_then(|v| v.as_str());
                let var_type = var.get("type").and_then(|v| v.as_str()).unwrap_or("string");
                let readonly = var.get("readonly").and_then(|v| v.as_bool()).unwrap_or(false);
                let description = var.get("description").and_then(|v| v.as_str());

                if let Some(name) = name {
                    db.conn().execute(
                        "INSERT OR REPLACE INTO burn_variables (name, var_type, readonly, description, category)
                         VALUES (?1, ?2, ?3, ?4, 'builtin')",
                        rusqlite::params![name, var_type, readonly as i32, description],
                    )?;
                    count += 1;
                }
            }
        }

        // Import system variables
        if let Some(vars) = value.get("system_variables").and_then(|v| v.as_array()) {
            for var in vars {
                let name = var.get("name").and_then(|v| v.as_str());
                let var_type = var.get("type").and_then(|v| v.as_str()).unwrap_or("string");
                let description = var.get("description").and_then(|v| v.as_str());

                if let Some(name) = name {
                    db.conn().execute(
                        "INSERT OR REPLACE INTO burn_variables (name, var_type, readonly, description, category)
                         VALUES (?1, ?2, 1, ?3, 'system')",
                        rusqlite::params![name, var_type, description],
                    )?;
                    count += 1;
                }
            }
        }

        Ok(count)
    }

    /// Import launch condition patterns
    pub fn import_launch_condition_patterns(&self, db: &Database, value: &serde_json::Value) -> Result<usize> {
        let mut count = 0;

        let categories = [
            ("os_version_conditions", "os_version"),
            ("architecture_conditions", "architecture"),
            ("privilege_conditions", "privilege"),
            ("product_type_conditions", "product_type"),
            ("prerequisite_conditions", "prerequisite"),
            ("memory_disk_conditions", "memory_disk"),
            ("custom_property_conditions", "custom"),
            ("installation_state_conditions", "installation_state"),
        ];

        for (key, category) in categories {
            if let Some(patterns) = value.get(key).and_then(|v| v.as_array()) {
                for pattern in patterns {
                    let name = pattern.get("name").and_then(|v| v.as_str());
                    let condition = pattern.get("condition").and_then(|v| v.as_str());
                    let message = pattern.get("message").and_then(|v| v.as_str());
                    let notes = pattern.get("notes").and_then(|v| v.as_str());

                    if let (Some(name), Some(condition)) = (name, condition) {
                        db.conn().execute(
                            "INSERT OR REPLACE INTO launch_condition_patterns (name, category, condition, message, notes)
                             VALUES (?1, ?2, ?3, ?4, ?5)",
                            rusqlite::params![name, category, condition, message, notes],
                        )?;
                        count += 1;
                    }
                }
            }
        }

        // Import VersionNT values
        if let Some(versions) = value.get("versionNT_values").and_then(|v| v.as_array()) {
            for ver in versions {
                let version_nt = ver.get("value").and_then(|v| v.as_i64());
                let os_name = ver.get("os").and_then(|v| v.as_str());

                if let (Some(version_nt), Some(os_name)) = (version_nt, os_name) {
                    db.conn().execute(
                        "INSERT OR REPLACE INTO version_nt_values (version_nt, os_name)
                         VALUES (?1, ?2)",
                        rusqlite::params![version_nt, os_name],
                    )?;
                    count += 1;
                }
            }
        }

        Ok(count)
    }

    /// Import user documentation entries
    pub fn import_documentation(&self, db: &Database, value: &serde_json::Value) -> Result<usize> {
        // Try "entries" key first (new format), then "documentation" (old format)
        let docs_key = if value.get("entries").is_some() { "entries" } else { "documentation" };
        let docs = value.get(docs_key)
            .and_then(|v| v.as_array())
            .ok_or_else(|| WixDataError::Parse("Missing documentation/entries array".into()))?;

        let mut count = 0;
        for doc in docs {
            // Support both old format (category/title) and new format (source/topic)
            let source = doc.get("source")
                .or_else(|| doc.get("category"))
                .and_then(|v| v.as_str());
            let topic = doc.get("topic")
                .or_else(|| doc.get("title"))
                .and_then(|v| v.as_str());
            let content = doc.get("content").and_then(|v| v.as_str());

            if let (Some(source), Some(topic), Some(content)) = (source, topic, content) {
                db.conn().execute(
                    "INSERT OR REPLACE INTO documentation (source, topic, content)
                     VALUES (?1, ?2, ?3)",
                    rusqlite::params![source, topic, content],
                )?;
                count += 1;
            }
        }
        Ok(count)
    }

    /// Import CLI commands
    pub fn import_cli_commands(&self, db: &Database, value: &serde_json::Value) -> Result<usize> {
        let mut count = 0;

        if let Some(commands) = value.get("commands").and_then(|v| v.as_array()) {
            for cmd in commands {
                let name = cmd.get("name").and_then(|v| v.as_str());
                let description = cmd.get("description").and_then(|v| v.as_str());
                let syntax = cmd.get("syntax").and_then(|v| v.as_str());

                if let Some(name) = name {
                    db.conn().execute(
                        "INSERT OR REPLACE INTO cli_commands (name, description, syntax, category)
                         VALUES (?1, ?2, ?3, 'wix')",
                        rusqlite::params![name, description, syntax],
                    )?;
                    count += 1;

                    // Get command ID for options
                    let cmd_id: i64 = db.conn().query_row(
                        "SELECT id FROM cli_commands WHERE name = ?1",
                        rusqlite::params![name],
                        |row| row.get(0),
                    ).unwrap_or(0);

                    // Import options for this command
                    if cmd_id > 0 {
                        if let Some(options) = cmd.get("options").and_then(|v| v.as_array()) {
                            for opt in options {
                                let opt_name = opt.get("name").and_then(|v| v.as_str());
                                let opt_alias = opt.get("alias").and_then(|v| v.as_str());
                                let opt_desc = opt.get("description").and_then(|v| v.as_str());
                                let opt_default = opt.get("default").and_then(|v| v.as_str());

                                if let Some(opt_name) = opt_name {
                                    db.conn().execute(
                                        "INSERT OR REPLACE INTO cli_command_options (command_id, name, alias, description, default_value)
                                         VALUES (?1, ?2, ?3, ?4, ?5)",
                                        rusqlite::params![cmd_id, opt_name, opt_alias, opt_desc, opt_default],
                                    )?;
                                    count += 1;
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(count)
    }

    /// Import localization cultures
    pub fn import_localization_cultures(&self, db: &Database, value: &serde_json::Value) -> Result<usize> {
        let mut count = 0;

        // Try "cultures" key first (new format), then "common_cultures" (old format)
        let cultures_key = if value.get("cultures").is_some() { "cultures" } else { "common_cultures" };
        if let Some(cultures) = value.get(cultures_key).and_then(|v| v.as_array()) {
            for culture in cultures {
                let code = culture.get("code").and_then(|v| v.as_str());
                let language = culture.get("language").and_then(|v| v.as_str());
                let lcid = culture.get("lcid").and_then(|v| v.as_i64());
                let codepage = culture.get("codepage").and_then(|v| v.as_i64());

                if let (Some(code), Some(language)) = (code, language) {
                    db.conn().execute(
                        "INSERT OR REPLACE INTO localization_cultures (code, language, lcid, codepage)
                         VALUES (?1, ?2, ?3, ?4)",
                        rusqlite::params![code, language, lcid, codepage],
                    )?;
                    count += 1;
                }
            }
        }

        Ok(count)
    }

    /// Import Windows build numbers for version targeting
    pub fn import_windows_builds(&self, db: &Database, value: &serde_json::Value) -> Result<usize> {
        let mut count = 0;

        // Import Windows 10/11 builds
        if let Some(builds) = value.get("windows_builds").and_then(|v| v.as_array()) {
            for build in builds {
                let build_num = build.get("build").and_then(|v| v.as_i64());
                let version = build.get("version").and_then(|v| v.as_str());
                let name = build.get("name").and_then(|v| v.as_str());
                let release_date = build.get("release_date").and_then(|v| v.as_str());
                let support_ended = build.get("support_ended").and_then(|v| v.as_bool()).unwrap_or(false);
                let os = build.get("os").and_then(|v| v.as_str()).unwrap_or("Windows 10");

                if let (Some(build_num), Some(version), Some(name)) = (build_num, version, name) {
                    db.conn().execute(
                        "INSERT OR REPLACE INTO windows_builds (build_number, version, name, os, release_date, support_ended)
                         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                        rusqlite::params![build_num, version, name, os, release_date, support_ended as i32],
                    )?;
                    count += 1;
                }
            }
        }

        // Import Windows Server builds
        if let Some(builds) = value.get("windows_server_builds").and_then(|v| v.as_array()) {
            for build in builds {
                let build_num = build.get("build").and_then(|v| v.as_i64());
                let version = build.get("version").and_then(|v| v.as_str());
                let name = build.get("name").and_then(|v| v.as_str());
                let release_date = build.get("release_date").and_then(|v| v.as_str());
                let support_ended = build.get("support_ended").and_then(|v| v.as_bool()).unwrap_or(false);

                if let (Some(build_num), Some(version), Some(name)) = (build_num, version, name) {
                    db.conn().execute(
                        "INSERT OR REPLACE INTO windows_builds (build_number, version, name, os, release_date, support_ended)
                         VALUES (?1, ?2, ?3, 'Windows Server', ?4, ?5)",
                        rusqlite::params![build_num, version, name, release_date, support_ended as i32],
                    )?;
                    count += 1;
                }
            }
        }

        // Import condition examples
        if let Some(examples) = value.get("condition_examples").and_then(|v| v.as_array()) {
            for ex in examples {
                let name = ex.get("name").and_then(|v| v.as_str());
                let condition = ex.get("condition").and_then(|v| v.as_str());
                let message = ex.get("message").and_then(|v| v.as_str());

                if let (Some(name), Some(condition)) = (name, condition) {
                    db.conn().execute(
                        "INSERT OR REPLACE INTO launch_condition_patterns (name, category, condition, message, notes)
                         VALUES (?1, 'windows_version', ?2, ?3, NULL)",
                        rusqlite::params![name, condition, message],
                    )?;
                    count += 1;
                }
            }
        }

        Ok(count)
    }

    /// Import WixUI dialog sets and dialogs
    pub fn import_wixui_dialogs(&self, db: &Database, value: &serde_json::Value) -> Result<usize> {
        let mut count = 0;

        // Import dialog sets
        if let Some(dialog_sets) = value.get("dialog_sets").and_then(|v| v.as_array()) {
            for ds in dialog_sets {
                let id = ds.get("id").and_then(|v| v.as_str());
                let name = ds.get("name").and_then(|v| v.as_str());
                let description = ds.get("description").and_then(|v| v.as_str());
                let use_case = ds.get("use_case").and_then(|v| v.as_str());
                let required_property = ds.get("required_property").and_then(|v| v.as_str());

                if let (Some(id), Some(name)) = (id, name) {
                    let full_desc = match (use_case, required_property) {
                        (Some(uc), Some(rp)) => format!("{}. Use case: {}. Required property: {}", description.unwrap_or(""), uc, rp),
                        (Some(uc), None) => format!("{}. Use case: {}", description.unwrap_or(""), uc),
                        (None, Some(rp)) => format!("{}. Required property: {}", description.unwrap_or(""), rp),
                        (None, None) => description.unwrap_or("").to_string(),
                    };
                    db.conn().execute(
                        "INSERT OR REPLACE INTO ui_elements (element_type, element_id, description)
                         VALUES ('dialog_set', ?1, ?2)",
                        rusqlite::params![id, format!("{}: {}", name, full_desc)],
                    )?;
                    count += 1;
                }
            }
        }

        // Import standard dialogs
        if let Some(dialogs) = value.get("standard_dialogs").and_then(|v| v.as_array()) {
            for dialog in dialogs {
                let id = dialog.get("id").and_then(|v| v.as_str());
                let name = dialog.get("name").and_then(|v| v.as_str());
                let description = dialog.get("description").and_then(|v| v.as_str());
                let notes = dialog.get("notes").and_then(|v| v.as_str());

                if let Some(id) = id {
                    let full_desc = match notes {
                        Some(n) => format!("{} Note: {}", description.unwrap_or(""), n),
                        None => description.unwrap_or("").to_string(),
                    };
                    db.conn().execute(
                        "INSERT OR REPLACE INTO ui_elements (element_type, element_id, description)
                         VALUES ('dialog', ?1, ?2)",
                        rusqlite::params![id, format!("{}: {}", name.unwrap_or(id), full_desc)],
                    )?;
                    count += 1;
                }
            }
        }

        // Import customization examples as documentation
        if let Some(examples) = value.get("customization_examples").and_then(|v| v.as_array()) {
            for ex in examples {
                let name = ex.get("name").and_then(|v| v.as_str());
                let wix = ex.get("wix").and_then(|v| v.as_str());
                let notes = ex.get("notes").and_then(|v| v.as_str());

                if let (Some(name), Some(wix)) = (name, wix) {
                    let content = match notes {
                        Some(n) => format!("{}\n\n{}", wix, n),
                        None => wix.to_string(),
                    };
                    db.conn().execute(
                        "INSERT OR REPLACE INTO documentation (source, topic, content)
                         VALUES ('wixui_customization', ?1, ?2)",
                        rusqlite::params![name, content],
                    )?;
                    count += 1;
                }
            }
        }

        Ok(count)
    }

    /// Import msiexec command-line reference
    pub fn import_msiexec_reference(&self, db: &Database, value: &serde_json::Value) -> Result<usize> {
        let mut count = 0;

        // Helper to import option arrays
        let import_options = |options: &[serde_json::Value], category: &str| -> Result<usize> {
            let mut c = 0;
            for opt in options {
                let option = opt.get("option").and_then(|v| v.as_str());
                let syntax = opt.get("syntax").and_then(|v| v.as_str());
                let description = opt.get("description").and_then(|v| v.as_str());

                if let Some(option) = option {
                    db.conn().execute(
                        "INSERT OR REPLACE INTO cli_commands (name, description, syntax, category)
                         VALUES (?1, ?2, ?3, ?4)",
                        rusqlite::params![option, description, syntax, category],
                    )?;
                    c += 1;
                }
            }
            Ok(c)
        };

        // Import install options
        if let Some(opts) = value.get("install_options").and_then(|v| v.as_array()) {
            count += import_options(opts, "msiexec_install")?;
        }

        // Import display options
        if let Some(opts) = value.get("display_options").and_then(|v| v.as_array()) {
            count += import_options(opts, "msiexec_display")?;
        }

        // Import logging options
        if let Some(opts) = value.get("logging_options").and_then(|v| v.as_array()) {
            count += import_options(opts, "msiexec_logging")?;
        }

        // Import restart options
        if let Some(opts) = value.get("restart_options").and_then(|v| v.as_array()) {
            count += import_options(opts, "msiexec_restart")?;
        }

        // Import common properties
        if let Some(props) = value.get("common_properties").and_then(|v| v.as_array()) {
            for prop in props {
                let name = prop.get("name").and_then(|v| v.as_str());
                let description = prop.get("description").and_then(|v| v.as_str());

                if let Some(name) = name {
                    db.conn().execute(
                        "INSERT OR IGNORE INTO builtin_properties (name, property_type, description, default_value, readonly)
                         VALUES (?1, 'string', ?2, '', 0)",
                        rusqlite::params![name, description],
                    )?;
                    count += 1;
                }
            }
        }

        // Import common scenarios as documentation
        if let Some(scenarios) = value.get("common_scenarios").and_then(|v| v.as_array()) {
            for scenario in scenarios {
                let name = scenario.get("name").and_then(|v| v.as_str());
                let command = scenario.get("command").and_then(|v| v.as_str());
                let description = scenario.get("description").and_then(|v| v.as_str());

                if let (Some(name), Some(command)) = (name, command) {
                    let content = format!("{}\n\nCommand: {}", description.unwrap_or(""), command);
                    db.conn().execute(
                        "INSERT OR REPLACE INTO documentation (source, topic, content)
                         VALUES ('msiexec_scenarios', ?1, ?2)",
                        rusqlite::params![name, content],
                    )?;
                    count += 1;
                }
            }
        }

        // Import exit codes
        if let Some(codes) = value.get("exit_codes").and_then(|v| v.as_array()) {
            for code in codes {
                let code_num = code.get("code").and_then(|v| v.as_i64());
                let name = code.get("name").and_then(|v| v.as_str());
                let description = code.get("description").and_then(|v| v.as_str());

                if let (Some(code_num), Some(name)) = (code_num, name) {
                    db.conn().execute(
                        "INSERT OR REPLACE INTO errors (code, severity, message_template, description, resolution)
                         VALUES (?1, 'exit_code', ?2, ?3, '')",
                        rusqlite::params![format!("MSIEXEC_{}", code_num), name, description],
                    )?;
                    count += 1;
                }
            }
        }

        // Import troubleshooting tips as documentation
        if let Some(tips) = value.get("troubleshooting_tips").and_then(|v| v.as_array()) {
            for tip in tips {
                let problem = tip.get("problem").and_then(|v| v.as_str());
                let solutions = tip.get("solutions").and_then(|v| v.as_array());

                if let (Some(problem), Some(solutions)) = (problem, solutions) {
                    let content = solutions.iter()
                        .filter_map(|s| s.as_str())
                        .collect::<Vec<_>>()
                        .join("\n- ");
                    db.conn().execute(
                        "INSERT OR REPLACE INTO documentation (source, topic, content)
                         VALUES ('msiexec_troubleshooting', ?1, ?2)",
                        rusqlite::params![problem, format!("- {}", content)],
                    )?;
                    count += 1;
                }
            }
        }

        Ok(count)
    }

    /// Import reference documentation (patching, transforms, etc.)
    pub fn import_reference_docs(&self, db: &Database, value: &serde_json::Value, source: &str) -> Result<usize> {
        let mut count = 0;

        // Store the description
        if let Some(desc) = value.get("description").and_then(|v| v.as_str()) {
            db.conn().execute(
                "INSERT OR REPLACE INTO documentation (source, topic, content)
                 VALUES (?1, 'overview', ?2)",
                rusqlite::params![source, desc],
            )?;
            count += 1;
        }

        // Store best practices
        if let Some(practices) = value.get("best_practices").and_then(|v| v.as_array()) {
            let content: Vec<&str> = practices.iter()
                .filter_map(|p| p.as_str())
                .collect();
            if !content.is_empty() {
                db.conn().execute(
                    "INSERT OR REPLACE INTO documentation (source, topic, content)
                     VALUES (?1, 'best_practices', ?2)",
                    rusqlite::params![source, content.join("\n")],
                )?;
                count += 1;
            }
        }

        // Store troubleshooting/common issues
        if let Some(issues) = value.get("troubleshooting").or_else(|| value.get("common_issues")).and_then(|v| v.as_array()) {
            for issue in issues {
                let issue_name = issue.get("issue").and_then(|v| v.as_str());
                let solution = issue.get("solution").and_then(|v| v.as_str());

                if let (Some(issue_name), Some(solution)) = (issue_name, solution) {
                    let topic = format!("troubleshooting:{}", issue_name);
                    db.conn().execute(
                        "INSERT OR REPLACE INTO documentation (source, topic, content)
                         VALUES (?1, ?2, ?3)",
                        rusqlite::params![source, topic, solution],
                    )?;
                    count += 1;
                }
            }
        }

        Ok(count)
    }

    pub fn import_keywords(&self, db: &Database, value: &serde_json::Value) -> Result<usize> {
        let mut count = 0;

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

        // Handle additional-keywords.json format
        // Extension prefixes
        if let Some(prefixes) = value.get("extension_prefixes").and_then(|v| v.as_array()) {
            for prefix in prefixes {
                if let Some(name) = prefix.as_str() {
                    db.conn().execute(
                        "INSERT OR IGNORE INTO keywords (word, category) VALUES (?1, 'extension')",
                        rusqlite::params![name],
                    )?;
                    count += 1;
                }
            }
        }

        // Standard directories
        if let Some(dirs) = value.get("standard_directories").and_then(|v| v.as_array()) {
            for dir in dirs {
                if let Some(name) = dir.as_str() {
                    db.conn().execute(
                        "INSERT OR IGNORE INTO keywords (word, category) VALUES (?1, 'directory')",
                        rusqlite::params![name],
                    )?;
                    count += 1;
                }
            }
        }

        // Built-in properties
        if let Some(props) = value.get("builtin_properties").and_then(|v| v.as_array()) {
            for prop in props {
                if let Some(name) = prop.as_str() {
                    db.conn().execute(
                        "INSERT OR IGNORE INTO keywords (word, category) VALUES (?1, 'property')",
                        rusqlite::params![name],
                    )?;
                    count += 1;
                }
            }
        }

        // Attribute types
        if let Some(types) = value.get("attribute_types").and_then(|v| v.as_array()) {
            for typ in types {
                if let Some(name) = typ.as_str() {
                    db.conn().execute(
                        "INSERT OR IGNORE INTO keywords (word, category) VALUES (?1, 'type')",
                        rusqlite::params![name],
                    )?;
                    count += 1;
                }
            }
        }

        // WixUI sets
        if let Some(sets) = value.get("wixui_sets").and_then(|v| v.as_array()) {
            for set in sets {
                if let Some(name) = set.as_str() {
                    db.conn().execute(
                        "INSERT OR IGNORE INTO keywords (word, category) VALUES (?1, 'wixui')",
                        rusqlite::params![name],
                    )?;
                    count += 1;
                }
            }
        }

        // Standard actions
        if let Some(actions) = value.get("standard_actions").and_then(|v| v.as_array()) {
            for action in actions {
                if let Some(name) = action.as_str() {
                    db.conn().execute(
                        "INSERT OR IGNORE INTO keywords (word, category) VALUES (?1, 'action')",
                        rusqlite::params![name],
                    )?;
                    count += 1;
                }
            }
        }

        // Burn actions
        if let Some(actions) = value.get("burn_actions").and_then(|v| v.as_array()) {
            for action in actions {
                if let Some(name) = action.as_str() {
                    db.conn().execute(
                        "INSERT OR IGNORE INTO keywords (word, category) VALUES (?1, 'burn_action')",
                        rusqlite::params![name],
                    )?;
                    count += 1;
                }
            }
        }

        // Control types
        if let Some(controls) = value.get("control_types").and_then(|v| v.as_array()) {
            for control in controls {
                if let Some(name) = control.as_str() {
                    db.conn().execute(
                        "INSERT OR IGNORE INTO keywords (word, category) VALUES (?1, 'control')",
                        rusqlite::params![name],
                    )?;
                    count += 1;
                }
            }
        }

        Ok(count)
    }

    pub fn import_snippets(&self, db: &Database, value: &serde_json::Value) -> Result<usize> {
        let snippets = value.get("snippets")
            .and_then(|v| v.as_array())
            .ok_or_else(|| WixDataError::Parse("Missing snippets array".into()))?;

        let mut count = 0;
        for snippet in snippets {
            let prefix = snippet.get("prefix").and_then(|v| v.as_str());
            let name = snippet.get("name").and_then(|v| v.as_str());
            let description = snippet.get("description").and_then(|v| v.as_str());
            let scope = snippet.get("scope").and_then(|v| v.as_str()).unwrap_or("wxs");
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
                    "INSERT OR REPLACE INTO snippets (prefix, name, description, body, scope)
                     VALUES (?1, ?2, ?3, ?4, ?5)",
                    rusqlite::params![prefix, name, description, body, scope],
                )?;
                count += 1;
            }
        }
        Ok(count)
    }

    /// Parse rules JSON with a default category
    pub fn parse_rules_with_category(&self, db: &Database, content: &str, default_category: &str) -> Result<usize> {
        let value: serde_json::Value = serde_json::from_str(content)
            .map_err(|e| WixDataError::Parse(format!("JSON parse error: {}", e)))?;

        let rules = value.get("rules")
            .and_then(|v| v.as_array())
            .ok_or_else(|| WixDataError::Parse("Missing rules array".into()))?;

        let mut count = 0;
        for rule_value in rules {
            let rule_id = rule_value.get("id").and_then(|v| v.as_str());
            let category = rule_value.get("category")
                .and_then(|v| v.as_str())
                .unwrap_or(default_category);
            let severity = rule_value.get("severity").and_then(|v| v.as_str()).unwrap_or("warning");
            let name = rule_value.get("name").and_then(|v| v.as_str());
            let description = rule_value.get("description").and_then(|v| v.as_str());

            if let (Some(rule_id), Some(name)) = (rule_id, name) {
                let rule = Rule {
                    id: 0,
                    rule_id: rule_id.to_string(),
                    category: category.to_string(),
                    severity: Severity::from(severity),
                    name: name.to_string(),
                    description: description.map(|s| s.to_string()),
                    rationale: None,
                    fix_suggestion: None,
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

    /// Import ICE rules
    pub fn import_ice_rules(&self, db: &Database, value: &serde_json::Value) -> Result<usize> {
        let rules = value.get("ice_rules")
            .and_then(|v| v.as_array())
            .ok_or_else(|| WixDataError::Parse("Missing ice_rules array".into()))?;

        let mut count = 0;
        for rule in rules {
            let code = rule.get("code").and_then(|v| v.as_str());
            let severity = rule.get("severity").and_then(|v| v.as_str()).unwrap_or("warning");
            let description = rule.get("description").and_then(|v| v.as_str());
            let resolution = rule.get("resolution").and_then(|v| v.as_str());
            let tables = rule.get("tables")
                .and_then(|v| v.as_array())
                .map(|arr| arr.iter()
                    .filter_map(|t| t.as_str())
                    .collect::<Vec<_>>()
                    .join(","));

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

    /// Import standard directories
    pub fn import_standard_directories(&self, db: &Database, value: &serde_json::Value) -> Result<usize> {
        let dirs = value.get("directories")
            .and_then(|v| v.as_array())
            .ok_or_else(|| WixDataError::Parse("Missing directories array".into()))?;

        let mut count = 0;
        for dir in dirs {
            let name = dir.get("name").and_then(|v| v.as_str());
            let description = dir.get("description").and_then(|v| v.as_str());
            let windows_path = dir.get("windows_path").and_then(|v| v.as_str());
            let example = dir.get("example").and_then(|v| v.as_str());

            if let Some(name) = name {
                db.conn().execute(
                    "INSERT OR IGNORE INTO standard_directories (name, description, windows_path, example)
                     VALUES (?1, ?2, ?3, ?4)",
                    rusqlite::params![name, description, windows_path, example],
                )?;
                count += 1;
            }
        }
        Ok(count)
    }

    /// Import builtin properties
    pub fn import_builtin_properties(&self, db: &Database, value: &serde_json::Value) -> Result<usize> {
        let props = value.get("properties")
            .and_then(|v| v.as_array())
            .ok_or_else(|| WixDataError::Parse("Missing properties array".into()))?;

        let mut count = 0;
        for prop in props {
            let name = prop.get("name").and_then(|v| v.as_str());
            let property_type = prop.get("type").and_then(|v| v.as_str());
            let description = prop.get("description").and_then(|v| v.as_str());
            let default_value = prop.get("default").and_then(|v| v.as_str());
            let readonly = prop.get("readonly").and_then(|v| v.as_bool()).unwrap_or(false);

            if let Some(name) = name {
                db.conn().execute(
                    "INSERT OR IGNORE INTO builtin_properties (name, property_type, description, default_value, readonly)
                     VALUES (?1, ?2, ?3, ?4, ?5)",
                    rusqlite::params![name, property_type, description, default_value, readonly as i32],
                )?;
                count += 1;
            }
        }
        Ok(count)
    }

    /// Import MSI tables
    pub fn import_msi_tables(&self, db: &Database, value: &serde_json::Value) -> Result<usize> {
        let tables = value.get("tables")
            .and_then(|v| v.as_array())
            .ok_or_else(|| WixDataError::Parse("Missing tables array".into()))?;

        let mut count = 0;
        for table in tables {
            let name = table.get("name").and_then(|v| v.as_str());
            let description = table.get("description").and_then(|v| v.as_str());
            let required = table.get("required").and_then(|v| v.as_bool()).unwrap_or(false);
            let columns = table.get("columns").map(|v| v.to_string());

            if let Some(name) = name {
                db.conn().execute(
                    "INSERT OR IGNORE INTO msi_tables (name, description, required, columns)
                     VALUES (?1, ?2, ?3, ?4)",
                    rusqlite::params![name, description, required as i32, columns],
                )?;
                count += 1;
            }
        }
        Ok(count)
    }

    /// Import WiX errors
    pub fn import_wix_errors(&self, db: &Database, value: &serde_json::Value) -> Result<usize> {
        let errors = value.get("errors")
            .and_then(|v| v.as_array())
            .ok_or_else(|| WixDataError::Parse("Missing errors array".into()))?;

        let mut count = 0;
        for error in errors {
            let code = error.get("code").and_then(|v| v.as_str());
            let severity = error.get("severity").and_then(|v| v.as_str()).unwrap_or("error");
            // Support both "message" and "message_template" keys
            let message = error.get("message_template")
                .or_else(|| error.get("message"))
                .and_then(|v| v.as_str());
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

    /// Import standard MSI actions
    pub fn import_standard_actions(&self, db: &Database, value: &serde_json::Value) -> Result<usize> {
        let actions = value.get("actions")
            .and_then(|v| v.as_array())
            .ok_or_else(|| WixDataError::Parse("Missing actions array".into()))?;

        let mut count = 0;
        for action in actions {
            let name = action.get("name").and_then(|v| v.as_str());
            let sequence = action.get("sequence").and_then(|v| v.as_i64());
            let description = action.get("description").and_then(|v| v.as_str());

            if let Some(name) = name {
                db.conn().execute(
                    "INSERT OR IGNORE INTO standard_actions (name, sequence, description)
                     VALUES (?1, ?2, ?3)",
                    rusqlite::params![name, sequence, description],
                )?;
                count += 1;
            }
        }
        Ok(count)
    }

    /// Import preprocessor directives and functions
    pub fn import_preprocessor(&self, db: &Database, value: &serde_json::Value) -> Result<usize> {
        let mut count = 0;

        // Import directives
        if let Some(directives) = value.get("directives").and_then(|v| v.as_array()) {
            for directive in directives {
                let name = directive.get("name").and_then(|v| v.as_str());
                let syntax = directive.get("syntax").and_then(|v| v.as_str());
                let description = directive.get("description").and_then(|v| v.as_str());
                let example = directive.get("example").and_then(|v| v.as_str());

                if let Some(name) = name {
                    db.conn().execute(
                        "INSERT OR IGNORE INTO preprocessor_directives (name, syntax, description, example)
                         VALUES (?1, ?2, ?3, ?4)",
                        rusqlite::params![name, syntax, description, example],
                    )?;
                    count += 1;
                }
            }
        }

        // Import functions as keywords
        if let Some(functions) = value.get("functions").and_then(|v| v.as_array()) {
            for func in functions {
                let name = func.get("name").and_then(|v| v.as_str());
                if let Some(name) = name {
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

    /// Import extension metadata
    pub fn import_extensions(&self, db: &Database, value: &serde_json::Value) -> Result<usize> {
        let extensions = value.get("extensions")
            .and_then(|v| v.as_array())
            .ok_or_else(|| WixDataError::Parse("Missing extensions array".into()))?;

        let mut count = 0;
        for ext in extensions {
            let name = ext.get("name").and_then(|v| v.as_str());
            let namespace = ext.get("namespace").and_then(|v| v.as_str());
            let prefix = ext.get("prefix").and_then(|v| v.as_str());
            let description = ext.get("description").and_then(|v| v.as_str());
            let xsd_url = ext.get("xsd_url").and_then(|v| v.as_str());

            if let (Some(name), Some(namespace), Some(prefix)) = (name, namespace, prefix) {
                db.conn().execute(
                    "INSERT OR IGNORE INTO extensions (name, namespace, prefix, description, xsd_url)
                     VALUES (?1, ?2, ?3, ?4, ?5)",
                    rusqlite::params![name, namespace, prefix, description, xsd_url],
                )?;
                count += 1;
            }
        }
        Ok(count)
    }

    /// Import extension elements mapping
    pub fn import_extension_elements(&self, db: &Database, value: &serde_json::Value) -> Result<usize> {
        let ext_elements = value.get("extension_elements")
            .and_then(|v| v.as_array())
            .ok_or_else(|| WixDataError::Parse("Missing extension_elements array".into()))?;

        let mut count = 0;
        for ext in ext_elements {
            let extension_name = ext.get("extension").and_then(|v| v.as_str());
            let elements = ext.get("elements").and_then(|v| v.as_array());

            if let (Some(ext_name), Some(elems)) = (extension_name, elements) {
                // Get extension ID
                let ext_id: i64 = db.conn().query_row(
                    "SELECT id FROM extensions WHERE name = ?1",
                    rusqlite::params![ext_name],
                    |row| row.get(0),
                ).unwrap_or(0);

                if ext_id == 0 {
                    continue;
                }

                for elem in elems {
                    let elem_name = elem.get("name").and_then(|v| v.as_str());
                    let description = elem.get("description").and_then(|v| v.as_str());

                    if let Some(name) = elem_name {
                        db.conn().execute(
                            "INSERT OR IGNORE INTO extension_elements (extension_id, name, description)
                             VALUES (?1, ?2, ?3)",
                            rusqlite::params![ext_id, name, description],
                        )?;
                        count += 1;
                    }
                }
            }
        }
        Ok(count)
    }

    /// Import prerequisites detection data
    pub fn import_prerequisites(&self, db: &Database, value: &serde_json::Value) -> Result<usize> {
        let prereqs = value.get("prerequisites")
            .and_then(|v| v.as_array())
            .ok_or_else(|| WixDataError::Parse("Missing prerequisites array".into()))?;

        let mut count = 0;
        for prereq in prereqs {
            let name = prereq.get("name").and_then(|v| v.as_str());
            let display_name = prereq.get("display_name").and_then(|v| v.as_str());
            let version = prereq.get("version").and_then(|v| v.as_str());
            let download_url = prereq.get("download_url").and_then(|v| v.as_str());
            let detection_method = prereq.get("detection_method").and_then(|v| v.as_str());
            let detection_value = prereq.get("detection_key")
                .or_else(|| prereq.get("detection_path"))
                .and_then(|v| v.as_str());

            if let Some(name) = name {
                db.conn().execute(
                    "INSERT OR IGNORE INTO prerequisites (name, display_name, version, download_url, detection_method, detection_value)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                    rusqlite::params![name, display_name, version, download_url, detection_method, detection_value],
                )?;
                count += 1;
            }
        }
        Ok(count)
    }

    /// Import UI elements (controls, dialog sets, dialogs)
    pub fn import_ui_elements(&self, db: &Database, value: &serde_json::Value) -> Result<usize> {
        let mut count = 0;

        // Import MSI control types
        if let Some(controls) = value.get("controls").and_then(|v| v.as_array()) {
            for control in controls {
                let name = control.get("name").and_then(|v| v.as_str());
                let description = control.get("description").and_then(|v| v.as_str());
                let category = control.get("category").and_then(|v| v.as_str());

                if let Some(name) = name {
                    db.conn().execute(
                        "INSERT OR IGNORE INTO ui_elements (element_type, element_id, description)
                         VALUES (?1, ?2, ?3)",
                        rusqlite::params![category.unwrap_or("control"), name, description],
                    )?;
                    count += 1;
                }
            }
        }

        // Import WixUI dialog sets
        if let Some(dialog_sets) = value.get("dialog_sets").and_then(|v| v.as_array()) {
            for ds in dialog_sets {
                let name = ds.get("name").and_then(|v| v.as_str());
                let description = ds.get("description").and_then(|v| v.as_str());
                let use_case = ds.get("use_case").and_then(|v| v.as_str());

                if let Some(name) = name {
                    let full_desc = match use_case {
                        Some(uc) => format!("{}. Use case: {}", description.unwrap_or(""), uc),
                        None => description.unwrap_or("").to_string(),
                    };
                    db.conn().execute(
                        "INSERT OR IGNORE INTO ui_elements (element_type, element_id, description)
                         VALUES ('dialog_set', ?1, ?2)",
                        rusqlite::params![name, full_desc],
                    )?;
                    count += 1;
                }
            }
        }

        // Import dialog definitions
        if let Some(dialogs) = value.get("dialogs").and_then(|v| v.as_array()) {
            for dialog in dialogs {
                let name = dialog.get("name").and_then(|v| v.as_str());
                let description = dialog.get("description").and_then(|v| v.as_str());

                if let Some(name) = name {
                    db.conn().execute(
                        "INSERT OR IGNORE INTO ui_elements (element_type, element_id, description)
                         VALUES ('dialog', ?1, ?2)",
                        rusqlite::params![name, description],
                    )?;
                    count += 1;
                }
            }
        }

        // Import control events
        if let Some(events) = value.get("control_events").and_then(|v| v.as_array()) {
            for event in events {
                let name = event.get("name").and_then(|v| v.as_str());
                let description = event.get("description").and_then(|v| v.as_str());

                if let Some(name) = name {
                    db.conn().execute(
                        "INSERT OR IGNORE INTO ui_elements (element_type, element_id, description)
                         VALUES ('event', ?1, ?2)",
                        rusqlite::params![name, description],
                    )?;
                    count += 1;
                }
            }
        }

        // Import control conditions
        if let Some(conditions) = value.get("control_conditions").and_then(|v| v.as_array()) {
            for cond in conditions {
                let name = cond.get("name").and_then(|v| v.as_str());
                let description = cond.get("description").and_then(|v| v.as_str());

                if let Some(name) = name {
                    db.conn().execute(
                        "INSERT OR IGNORE INTO ui_elements (element_type, element_id, description)
                         VALUES ('condition', ?1, ?2)",
                        rusqlite::params![name, description],
                    )?;
                    count += 1;
                }
            }
        }

        Ok(count)
    }

    /// Import rule conditions for linter evaluation
    pub fn import_rule_conditions(&self, db: &Database, value: &serde_json::Value) -> Result<usize> {
        let rule_conditions = value.get("rule_conditions")
            .and_then(|v| v.as_array())
            .ok_or_else(|| WixDataError::Parse("Missing rule_conditions array".into()))?;

        let mut count = 0;
        for rc in rule_conditions {
            let rule_id_str = rc.get("rule_id").and_then(|v| v.as_str());
            let conditions = rc.get("conditions").and_then(|v| v.as_array());

            if let (Some(rule_id_str), Some(conds)) = (rule_id_str, conditions) {
                // Get rule ID from database
                let rule_db_id: i64 = db.conn().query_row(
                    "SELECT id FROM rules WHERE rule_id = ?1",
                    rusqlite::params![rule_id_str],
                    |row| row.get(0),
                ).unwrap_or(0);

                if rule_db_id == 0 {
                    // Rule not found, skip
                    continue;
                }

                for cond in conds {
                    let condition_type = cond.get("condition_type").and_then(|v| v.as_str());
                    let target = cond.get("target").and_then(|v| v.as_str());
                    let operator = cond.get("operator").and_then(|v| v.as_str());
                    let value_str = cond.get("value").and_then(|v| v.as_str());

                    if let (Some(cond_type), Some(target)) = (condition_type, target) {
                        db.conn().execute(
                            "INSERT OR IGNORE INTO rule_conditions (rule_id, condition_type, target, operator, value)
                             VALUES (?1, ?2, ?3, ?4, ?5)",
                            rusqlite::params![rule_db_id, cond_type, target, operator, value_str],
                        )?;
                        count += 1;
                    }
                }
            }
        }
        Ok(count)
    }

    /// Import element description patches (fill in missing descriptions)
    pub fn import_element_descriptions_patch(&self, db: &Database, value: &serde_json::Value) -> Result<usize> {
        let patches = value.get("patches")
            .and_then(|v| v.as_array())
            .ok_or_else(|| WixDataError::Parse("Missing patches array".into()))?;

        let mut count = 0;
        for patch in patches {
            let name = patch.get("name").and_then(|v| v.as_str());
            let namespace = patch.get("namespace").and_then(|v| v.as_str());
            let description = patch.get("description").and_then(|v| v.as_str());

            if let (Some(name), Some(namespace), Some(description)) = (name, namespace, description) {
                let rows_updated = db.conn().execute(
                    "UPDATE elements SET description = ?1 WHERE name = ?2 AND namespace = ?3 AND (description IS NULL OR description = '')",
                    rusqlite::params![description, name, namespace],
                )?;
                if rows_updated > 0 {
                    count += 1;
                }
            }
        }
        Ok(count)
    }

    /// Import attribute description patches (fill in missing descriptions)
    pub fn import_attribute_descriptions_patch(&self, db: &Database, value: &serde_json::Value) -> Result<usize> {
        let patches = value.get("patches")
            .and_then(|v| v.as_array())
            .ok_or_else(|| WixDataError::Parse("Missing patches array".into()))?;

        let mut count = 0;
        for patch in patches {
            let element = patch.get("element").and_then(|v| v.as_str());
            let attribute = patch.get("attribute").and_then(|v| v.as_str());
            let namespace = patch.get("namespace").and_then(|v| v.as_str());
            let description = patch.get("description").and_then(|v| v.as_str());

            if let (Some(element), Some(attribute), Some(namespace), Some(description)) = (element, attribute, namespace, description) {
                // Get element_id first
                let element_id: Option<i64> = db.conn().query_row(
                    "SELECT id FROM elements WHERE name = ?1 AND namespace = ?2",
                    rusqlite::params![element, namespace],
                    |row| row.get(0),
                ).ok();

                if let Some(elem_id) = element_id {
                    let rows_updated = db.conn().execute(
                        "UPDATE attributes SET description = ?1 WHERE element_id = ?2 AND name = ?3 AND (description IS NULL OR description = '')",
                        rusqlite::params![description, elem_id, attribute],
                    )?;
                    if rows_updated > 0 {
                        count += 1;
                    }
                }
            }
        }
        Ok(count)
    }

    /// Import UI element description patches
    pub fn import_ui_elements_patch(&self, db: &Database, value: &serde_json::Value) -> Result<usize> {
        let patches = value.get("patches")
            .and_then(|v| v.as_array())
            .ok_or_else(|| WixDataError::Parse("Missing patches array".into()))?;

        let mut count = 0;
        for patch in patches {
            let element_type = patch.get("element_type").and_then(|v| v.as_str());
            let element_id = patch.get("element_id").and_then(|v| v.as_str());
            let description = patch.get("description").and_then(|v| v.as_str());

            if let (Some(etype), Some(eid), Some(desc)) = (element_type, element_id, description) {
                let rows_updated = db.conn().execute(
                    "UPDATE ui_elements SET description = ?1 WHERE element_type = ?2 AND element_id = ?3 AND (description IS NULL OR description = '')",
                    rusqlite::params![desc, etype, eid],
                )?;
                if rows_updated > 0 {
                    count += 1;
                }
            }
        }
        Ok(count)
    }

    /// Parse migration JSON
    pub fn parse_migration(&self, db: &Database, content: &str) -> Result<usize> {
        let value: serde_json::Value = serde_json::from_str(content)
            .map_err(|e| WixDataError::Parse(format!("JSON parse error: {}", e)))?;

        let from = value.get("from").and_then(|v| v.as_str()).unwrap_or("v3");
        let to = value.get("to").and_then(|v| v.as_str()).unwrap_or("v4");

        let changes = value.get("changes")
            .and_then(|v| v.as_array())
            .ok_or_else(|| WixDataError::Parse("Missing changes array".into()))?;

        let mut count = 0;
        for change in changes {
            let change_type = change.get("type").and_then(|v| v.as_str());
            let old_value = change.get("old").and_then(|v| v.as_str());
            let new_value = change.get("new").and_then(|v| v.as_str());

            if let Some(change_type) = change_type {
                db.conn().execute(
                    "INSERT INTO migrations (from_version, to_version, change_type, old_value, new_value)
                     VALUES (?1, ?2, ?3, ?4, ?5)",
                    rusqlite::params![from, to, change_type, old_value, new_value],
                )?;
                count += 1;
            }
        }
        Ok(count)
    }

    /// Parse HTML documentation page
    pub fn parse_html(&self, db: &Database, content: &str, source: &SourceDef, source_name: &str) -> Result<usize> {
        use scraper::{Html, Selector};

        let document = Html::parse_document(content);
        let mut count = 0;

        // Extract documentation content based on targets
        for target in &source.targets {
            match target.as_str() {
                "elements" => {
                    // Try to find element definitions in documentation
                    if let Ok(selector) = Selector::parse("h2, h3, .element-name") {
                        for element in document.select(&selector) {
                            let name = element.text().collect::<String>().trim().to_string();
                            if !name.is_empty() && name.chars().next().map(|c| c.is_uppercase()).unwrap_or(false) {
                                // Looks like an element name, store as documentation entry
                                db.conn().execute(
                                    "INSERT OR IGNORE INTO documentation (source, topic, content)
                                     VALUES (?1, ?2, ?3)",
                                    rusqlite::params![source_name, name, ""],
                                )?;
                                count += 1;
                            }
                        }
                    }
                }
                "tutorials" | "docs" | "tools" | "commands" | "extensions" | "burn" | "preprocessor" => {
                    // Extract main content
                    let selectors = ["article", "main", ".content", "#main-content", ".documentation"];
                    for sel_str in selectors {
                        if let Ok(selector) = Selector::parse(sel_str) {
                            for elem in document.select(&selector) {
                                let text = elem.text().collect::<String>();
                                if !text.trim().is_empty() {
                                    db.conn().execute(
                                        "INSERT OR IGNORE INTO documentation (source, topic, content)
                                         VALUES (?1, ?2, ?3)",
                                        rusqlite::params![source_name, target, text.trim()],
                                    )?;
                                    count += 1;
                                    break; // Only get first match
                                }
                            }
                        }
                    }
                }
                "msi_tables" => {
                    // Extract table information from MSI documentation
                    if let Ok(selector) = Selector::parse("table tr td a") {
                        for elem in document.select(&selector) {
                            let name = elem.text().collect::<String>().trim().to_string();
                            if !name.is_empty() {
                                db.conn().execute(
                                    "INSERT OR IGNORE INTO msi_tables (name, description, required, columns)
                                     VALUES (?1, '', 0, '')",
                                    rusqlite::params![name],
                                )?;
                                count += 1;
                            }
                        }
                    }
                }
                "properties" => {
                    // Extract property names from documentation tables
                    if let Ok(selector) = Selector::parse("table tr") {
                        for row in document.select(&selector) {
                            if let Ok(td_sel) = Selector::parse("td") {
                                let cells: Vec<_> = row.select(&td_sel).collect();
                                if !cells.is_empty() {
                                    let name = cells[0].text().collect::<String>().trim().to_string();
                                    let desc = cells.get(1)
                                        .map(|c| c.text().collect::<String>().trim().to_string())
                                        .unwrap_or_default();
                                    if !name.is_empty() && name.chars().all(|c| c.is_ascii_uppercase() || c == '_') {
                                        db.conn().execute(
                                            "INSERT OR IGNORE INTO builtin_properties (name, property_type, description, default_value, readonly)
                                             VALUES (?1, 'string', ?2, '', 0)",
                                            rusqlite::params![name, desc],
                                        )?;
                                        count += 1;
                                    }
                                }
                            }
                        }
                    }
                }
                _ => {
                    // Generic documentation extraction
                    if let Ok(selector) = Selector::parse("p, li") {
                        for para in document.select(&selector).take(50) {
                            let text = para.text().collect::<String>().trim().to_string();
                            if text.len() > 50 {
                                db.conn().execute(
                                    "INSERT OR IGNORE INTO documentation (source, topic, content)
                                     VALUES (?1, ?2, ?3)",
                                    rusqlite::params![source_name, target, text],
                                )?;
                                count += 1;
                            }
                        }
                    }
                }
            }
        }

        Ok(count)
    }

    /// Parse WXS (WiX source) files for UI elements and patterns
    pub fn parse_wxs(&self, db: &Database, content: &str, source: &SourceDef) -> Result<usize> {
        use roxmltree::Document;

        let doc = Document::parse(content)
            .map_err(|e| WixDataError::Parse(format!("WXS parse error: {}", e)))?;

        let mut count = 0;

        for target in &source.targets {
            match target.as_str() {
                "ui_elements" | "dialogs" => {
                    // Extract Dialog elements
                    for node in doc.descendants() {
                        if node.tag_name().name() == "Dialog" {
                            if let Some(id) = node.attribute("Id") {
                                db.conn().execute(
                                    "INSERT OR IGNORE INTO ui_elements (element_type, element_id, description)
                                     VALUES ('Dialog', ?1, '')",
                                    rusqlite::params![id],
                                )?;
                                count += 1;
                            }
                        }
                        if node.tag_name().name() == "Control" {
                            if let Some(id) = node.attribute("Id") {
                                let ctrl_type = node.attribute("Type").unwrap_or("unknown");
                                db.conn().execute(
                                    "INSERT OR IGNORE INTO ui_elements (element_type, element_id, description)
                                     VALUES (?1, ?2, '')",
                                    rusqlite::params![ctrl_type, id],
                                )?;
                                count += 1;
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        Ok(count)
    }

    /// Parse C# or C++ source code for error definitions
    pub fn parse_source_code(&self, db: &Database, content: &str, source: &SourceDef, source_name: &str) -> Result<usize> {
        use regex::Regex;

        let mut count = 0;

        // C# error message patterns
        if source.parser == "csharp" {
            // Pattern: public static Message ErrorName(...)
            let method_re = Regex::new(r"public static Message (\w+)\(")
                .map_err(|e| WixDataError::Parse(format!("Regex error: {}", e)))?;

            // Pattern: Ids.ErrorCode
            let id_re = Regex::new(r"Ids\.(\w+)")
                .map_err(|e| WixDataError::Parse(format!("Regex error: {}", e)))?;

            // Pattern: "Message format string"
            let msg_re = Regex::new(r#""([^"\\]*(?:\\.[^"\\]*)*)""#)
                .map_err(|e| WixDataError::Parse(format!("Regex error: {}", e)))?;

            for cap in method_re.captures_iter(content) {
                let name = &cap[1];
                // Find the corresponding message
                let start = cap.get(0).map(|m| m.end()).unwrap_or(0);
                let snippet = &content[start..std::cmp::min(start + 500, content.len())];

                let code = id_re.captures(snippet)
                    .and_then(|c| c.get(1))
                    .map(|m| m.as_str().to_string());

                let message = msg_re.captures(snippet)
                    .and_then(|c| c.get(1))
                    .map(|m| m.as_str().to_string());

                if let (Some(code), Some(msg)) = (code, message) {
                    db.conn().execute(
                        "INSERT OR IGNORE INTO errors (code, severity, message_template, description, resolution)
                         VALUES (?1, 'error', ?2, ?3, '')",
                        rusqlite::params![code, msg, name],
                    )?;
                    count += 1;
                }
            }
        }

        // C++ patterns (for burn engine)
        if source.parser == "cpp" {
            // Look for logging patterns
            let log_re = Regex::new(r#"Log\w+\([^,]+,\s*"([^"]+)""#)
                .map_err(|e| WixDataError::Parse(format!("Regex error: {}", e)))?;

            for cap in log_re.captures_iter(content) {
                let message = &cap[1];
                if message.len() > 10 {
                    db.conn().execute(
                        "INSERT OR IGNORE INTO documentation (source, topic, content)
                         VALUES (?1, 'burn_log', ?2)",
                        rusqlite::params![source_name, message],
                    )?;
                    count += 1;
                }
            }
        }

        Ok(count)
    }

    /// Parse Markdown documentation
    pub fn parse_markdown(&self, db: &Database, content: &str, source: &SourceDef, source_name: &str) -> Result<usize> {
        let mut count = 0;

        for _target in &source.targets {
            // Extract headings and content sections
            let mut current_heading = String::new();
            let mut current_content = String::new();

            for line in content.lines() {
                if line.starts_with('#') {
                    // Save previous section
                    if !current_heading.is_empty() && !current_content.is_empty() {
                        db.conn().execute(
                            "INSERT OR IGNORE INTO documentation (source, topic, content)
                             VALUES (?1, ?2, ?3)",
                            rusqlite::params![source_name, current_heading, current_content.trim()],
                        )?;
                        count += 1;
                    }
                    current_heading = line.trim_start_matches('#').trim().to_string();
                    current_content.clear();
                } else {
                    current_content.push_str(line);
                    current_content.push('\n');
                }
            }

            // Save last section
            if !current_heading.is_empty() && !current_content.is_empty() {
                db.conn().execute(
                    "INSERT OR IGNORE INTO documentation (source, topic, content)
                     VALUES (?1, ?2, ?3)",
                    rusqlite::params![source_name, current_heading, current_content.trim()],
                )?;
                count += 1;
            }
        }

        Ok(count)
    }

    /// Import attribute enum values
    pub fn import_attribute_enums(&self, db: &Database, value: &serde_json::Value) -> Result<usize> {
        let mut count = 0;

        if let Some(enums) = value.get("enums").and_then(|v| v.as_object()) {
            for (element_name, attrs) in enums {
                if let Some(attrs_obj) = attrs.as_object() {
                    for (attr_name, values) in attrs_obj {
                        if let Some(values_arr) = values.as_array() {
                            // First, find the attribute_id
                            let attr_id: Option<i64> = db.conn().query_row(
                                "SELECT a.id FROM attributes a
                                 JOIN elements e ON a.element_id = e.id
                                 WHERE e.name = ?1 AND a.name = ?2",
                                rusqlite::params![element_name, attr_name],
                                |row| row.get(0),
                            ).ok();

                            if let Some(attr_id) = attr_id {
                                for val in values_arr {
                                    let value_str = val.get("value").and_then(|v| v.as_str());
                                    let description = val.get("description").and_then(|v| v.as_str());

                                    if let Some(value_str) = value_str {
                                        db.conn().execute(
                                            "INSERT OR REPLACE INTO attribute_enum_values (attribute_id, value, description)
                                             VALUES (?1, ?2, ?3)",
                                            rusqlite::params![attr_id, value_str, description],
                                        )?;
                                        count += 1;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(count)
    }

    /// Import migration mappings
    pub fn import_migrations(&self, db: &Database, value: &serde_json::Value) -> Result<usize> {
        let mut count = 0;

        // Import v3 to v4 migrations
        if let Some(v3_v4) = value.get("v3_to_v4").and_then(|v| v.as_object()) {
            // Namespace changes
            if let Some(ns) = v3_v4.get("namespace_changes").and_then(|v| v.as_array()) {
                for change in ns {
                    let from = change.get("from").and_then(|v| v.as_str());
                    let to = change.get("to").and_then(|v| v.as_str());
                    let notes = change.get("notes").and_then(|v| v.as_str());

                    if let (Some(from), Some(to)) = (from, to) {
                        db.conn().execute(
                            "INSERT OR REPLACE INTO migrations (from_version, to_version, change_type, old_value, new_value, notes)
                             VALUES ('v3', 'v4', 'namespace', ?1, ?2, ?3)",
                            rusqlite::params![from, to, notes],
                        )?;
                        count += 1;
                    }
                }
            }

            // Element renames
            if let Some(renames) = v3_v4.get("element_renames").and_then(|v| v.as_array()) {
                for rename in renames {
                    let from = rename.get("from").and_then(|v| v.as_str());
                    let to = rename.get("to").and_then(|v| v.as_str());
                    let notes = rename.get("notes").and_then(|v| v.as_str());

                    if let (Some(from), Some(to)) = (from, to) {
                        db.conn().execute(
                            "INSERT OR REPLACE INTO migrations (from_version, to_version, change_type, old_value, new_value, notes)
                             VALUES ('v3', 'v4', 'element_renamed', ?1, ?2, ?3)",
                            rusqlite::params![from, to, notes],
                        )?;
                        count += 1;
                    }
                }
            }

            // Attribute changes
            if let Some(changes) = v3_v4.get("attribute_changes").and_then(|v| v.as_array()) {
                for change in changes {
                    let element = change.get("element").and_then(|v| v.as_str()).unwrap_or("");
                    let from = change.get("from").and_then(|v| v.as_str());
                    let to = change.get("to").and_then(|v| v.as_str());
                    let notes = change.get("notes").and_then(|v| v.as_str());

                    if let (Some(from), Some(to)) = (from, to) {
                        let old_val = format!("{}/@{}", element, from);
                        let new_val = format!("{}/@{}", element, to);
                        db.conn().execute(
                            "INSERT OR REPLACE INTO migrations (from_version, to_version, change_type, old_value, new_value, notes)
                             VALUES ('v3', 'v4', 'attribute_changed', ?1, ?2, ?3)",
                            rusqlite::params![old_val, new_val, notes],
                        )?;
                        count += 1;
                    }
                }
            }

            // Tool changes
            if let Some(tools) = v3_v4.get("tool_changes").and_then(|v| v.as_array()) {
                for tool in tools {
                    let from = tool.get("from").and_then(|v| v.as_str());
                    let to = tool.get("to").and_then(|v| v.as_str());
                    let notes = tool.get("notes").and_then(|v| v.as_str());

                    if let (Some(from), Some(to)) = (from, to) {
                        db.conn().execute(
                            "INSERT OR REPLACE INTO migrations (from_version, to_version, change_type, old_value, new_value, notes)
                             VALUES ('v3', 'v4', 'tool_renamed', ?1, ?2, ?3)",
                            rusqlite::params![from, to, notes],
                        )?;
                        count += 1;
                    }
                }
            }
        }

        // Import v4 to v5 migrations
        if let Some(v4_v5) = value.get("v4_to_v5").and_then(|v| v.as_object()) {
            if let Some(additions) = v4_v5.get("element_additions").and_then(|v| v.as_array()) {
                for add in additions {
                    let element = add.get("element").and_then(|v| v.as_str());
                    let notes = add.get("notes").and_then(|v| v.as_str());

                    if let Some(element) = element {
                        db.conn().execute(
                            "INSERT OR REPLACE INTO migrations (from_version, to_version, change_type, old_value, new_value, notes)
                             VALUES ('v4', 'v5', 'added', NULL, ?1, ?2)",
                            rusqlite::params![element, notes],
                        )?;
                        count += 1;
                    }
                }
            }

            if let Some(removals) = v4_v5.get("element_removals").and_then(|v| v.as_array()) {
                for rem in removals {
                    let element = rem.get("element").and_then(|v| v.as_str());
                    let notes = rem.get("notes").and_then(|v| v.as_str());

                    if let Some(element) = element {
                        db.conn().execute(
                            "INSERT OR REPLACE INTO migrations (from_version, to_version, change_type, old_value, new_value, notes)
                             VALUES ('v4', 'v5', 'removed', ?1, NULL, ?2)",
                            rusqlite::params![element, notes],
                        )?;
                        count += 1;
                    }
                }
            }
        }

        // Import common migration issues as documentation
        if let Some(issues) = value.get("common_migration_issues").and_then(|v| v.as_array()) {
            for issue in issues {
                let issue_name = issue.get("issue").and_then(|v| v.as_str());
                let symptom = issue.get("symptom").and_then(|v| v.as_str()).unwrap_or("");
                let solution = issue.get("solution").and_then(|v| v.as_str()).unwrap_or("");

                if let Some(issue_name) = issue_name {
                    let content = format!("Symptom: {}\nSolution: {}", symptom, solution);
                    db.conn().execute(
                        "INSERT OR REPLACE INTO documentation (source, topic, content)
                         VALUES ('migration', ?1, ?2)",
                        rusqlite::params![issue_name, content],
                    )?;
                    count += 1;
                }
            }
        }

        Ok(count)
    }

    /// Import error resolutions (patches existing errors with resolutions)
    pub fn import_error_resolutions(&self, db: &Database, value: &serde_json::Value) -> Result<usize> {
        let resolutions = value.get("resolutions")
            .and_then(|v| v.as_array())
            .ok_or_else(|| WixDataError::Parse("Missing resolutions array".into()))?;

        let mut count = 0;
        for res in resolutions {
            let code = res.get("code").and_then(|v| v.as_str());
            let resolution = res.get("resolution").and_then(|v| v.as_str());

            if let (Some(code), Some(resolution)) = (code, resolution) {
                let updated = db.conn().execute(
                    "UPDATE errors SET resolution = ?2 WHERE code = ?1 AND (resolution IS NULL OR resolution = '')",
                    rusqlite::params![code, resolution],
                )?;
                if updated > 0 {
                    count += 1;
                }
            }
        }
        Ok(count)
    }

    /// Import ICE rule resolutions (patches existing ICE rules with resolutions)
    pub fn import_ice_resolutions(&self, db: &Database, value: &serde_json::Value) -> Result<usize> {
        let resolutions = value.get("resolutions")
            .and_then(|v| v.as_array())
            .ok_or_else(|| WixDataError::Parse("Missing resolutions array".into()))?;

        let mut count = 0;
        for res in resolutions {
            let code = res.get("code").and_then(|v| v.as_str());
            let resolution = res.get("resolution").and_then(|v| v.as_str());

            if let (Some(code), Some(resolution)) = (code, resolution) {
                let updated = db.conn().execute(
                    "UPDATE ice_rules SET resolution = ?2 WHERE code = ?1 AND (resolution IS NULL OR resolution = '')",
                    rusqlite::params![code, resolution],
                )?;
                if updated > 0 {
                    count += 1;
                }
            }
        }
        Ok(count)
    }

    /// Import MSI table descriptions (patches existing tables with descriptions)
    pub fn import_msi_table_descriptions(&self, db: &Database, value: &serde_json::Value) -> Result<usize> {
        let mut count = 0;

        // Import table descriptions
        if let Some(tables) = value.get("tables").and_then(|v| v.as_array()) {
            for table in tables {
                let name = table.get("name").and_then(|v| v.as_str());
                let description = table.get("description").and_then(|v| v.as_str());

                if let (Some(name), Some(description)) = (name, description) {
                    // Update existing table or insert if not exists
                    let updated = db.conn().execute(
                        "UPDATE msi_tables SET description = ?2 WHERE name = ?1 AND (description IS NULL OR description = '')",
                        rusqlite::params![name, description],
                    )?;

                    if updated == 0 {
                        // Table doesn't exist, insert it
                        db.conn().execute(
                            "INSERT OR IGNORE INTO msi_tables (name, description) VALUES (?1, ?2)",
                            rusqlite::params![name, description],
                        )?;
                    }
                    count += 1;
                }
            }
        }

        // Import data type descriptions into documentation
        if let Some(data_types) = value.get("data_types").and_then(|v| v.as_array()) {
            for dt in data_types {
                let name = dt.get("name").and_then(|v| v.as_str());
                let description = dt.get("description").and_then(|v| v.as_str());

                if let (Some(name), Some(description)) = (name, description) {
                    db.conn().execute(
                        "INSERT OR REPLACE INTO documentation (source, topic, content)
                         VALUES ('msi-data-types', ?1, ?2)",
                        rusqlite::params![name, description],
                    )?;
                    count += 1;
                }
            }
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
