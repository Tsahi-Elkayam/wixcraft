//! WiX XML plugin for winter linter

mod document;
mod parser;
mod rules;

pub use document::WixDocument;
pub use parser::WixNode;

use crate::plugin::{Document, ParseError, Plugin, RuleLoadError};
use crate::rule::{Rule, RuleFile};
use std::path::Path;

/// WiX plugin for linting WiX XML files
pub struct WixPlugin {
    /// Plugin version
    version: String,

    /// Loaded rules
    rules: Vec<Rule>,

    /// WiX version (3 or 4)
    wix_version: u8,

    /// Whether rules were loaded from database
    rules_from_db: bool,
}

impl Default for WixPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl WixPlugin {
    /// Create a new WiX plugin
    ///
    /// Tries to load rules from wixkb database first.
    /// Falls back to built-in rules if database is unavailable.
    pub fn new() -> Self {
        // Try to load rules from database
        let (rules, from_db) = Self::load_rules_from_db()
            .map(|r| (r, true))
            .unwrap_or_else(|| (rules::builtin_rules(), false));

        Self {
            version: env!("CARGO_PKG_VERSION").to_string(),
            rules,
            wix_version: 4,
            rules_from_db: from_db,
        }
    }

    /// Create a plugin with only built-in rules (no database)
    pub fn with_builtin_rules() -> Self {
        Self {
            version: env!("CARGO_PKG_VERSION").to_string(),
            rules: rules::builtin_rules(),
            wix_version: 4,
            rules_from_db: false,
        }
    }

    /// Create a plugin for a specific WiX version
    pub fn with_version(mut self, version: u8) -> Self {
        self.wix_version = version;
        self
    }

    /// Check if rules were loaded from database
    pub fn is_using_db_rules(&self) -> bool {
        self.rules_from_db
    }

    /// Try to load rules from the wix-data database
    fn load_rules_from_db() -> Option<Vec<Rule>> {
        // Try to open the database
        let db = wix_data::WixData::open_default().ok()?;

        // Get all enabled rules with conditions (Winter rules)
        let db_rules = db.get_enabled_rules().ok()?;

        // Filter to only rules with Winter conditions
        let winter_rules: Vec<_> = db_rules
            .into_iter()
            .filter(|r| r.condition.is_some())
            .collect();

        if winter_rules.is_empty() {
            return None;
        }

        // Convert wix_data rules to Winter rules
        let rules: Vec<Rule> = winter_rules
            .into_iter()
            .filter_map(Self::convert_db_rule)
            .collect();

        if rules.is_empty() {
            None
        } else {
            Some(rules)
        }
    }

    /// Convert a wix_data Rule to a Winter Rule
    fn convert_db_rule(db_rule: wix_data::models::Rule) -> Option<Rule> {
        let condition = db_rule.condition?;

        let severity = match db_rule.severity {
            wix_data::models::Severity::Error => crate::diagnostic::Severity::Error,
            wix_data::models::Severity::Warning => crate::diagnostic::Severity::Warning,
            wix_data::models::Severity::Info => crate::diagnostic::Severity::Info,
        };

        let mut rule = Rule::new(&db_rule.rule_id, &condition, &db_rule.name)
            .with_severity(severity)
            .with_target(
                db_rule.target_kind.as_deref(),
                db_rule.target_name.as_deref(),
            );

        // Add description if available
        if let Some(desc) = db_rule.description {
            rule = rule.with_description(&desc);
        }

        // Add tags if available
        if let Some(tags) = db_rule.tags {
            for tag in tags.split(',') {
                let tag = tag.trim();
                if !tag.is_empty() {
                    rule = rule.with_tag(tag);
                }
            }
        }

        Some(rule)
    }
}

impl Plugin for WixPlugin {
    fn id(&self) -> &str {
        "wix"
    }

    fn version(&self) -> &str {
        &self.version
    }

    fn description(&self) -> &str {
        "WiX XML linter plugin"
    }

    fn extensions(&self) -> &[&str] {
        &["wxs", "wxi", "wxl"]
    }

    fn parse(&self, content: &str, path: &Path) -> Result<Box<dyn Document>, ParseError> {
        let doc = WixDocument::parse(content, path)?;
        Ok(Box::new(doc))
    }

    fn rules(&self) -> &[Rule] {
        &self.rules
    }

    fn load_rules(&mut self, dir: &Path) -> Result<usize, RuleLoadError> {
        let mut count = 0;

        // Look for YAML and JSON rule files
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
            if ext != "yaml" && ext != "yml" && ext != "json" {
                continue;
            }

            let content = std::fs::read_to_string(&path)?;
            let rule_file: RuleFile = match ext {
                "yaml" | "yml" => {
                    serde_yaml::from_str(&content).map_err(|e| RuleLoadError::Parse {
                        file: path.display().to_string(),
                        message: e.to_string(),
                    })?
                }
                "json" => serde_json::from_str(&content).map_err(|e| RuleLoadError::Parse {
                    file: path.display().to_string(),
                    message: e.to_string(),
                })?,
                _ => continue,
            };

            // Only load rules for this plugin
            if let Some(plugin) = &rule_file.plugin {
                if plugin != "wix" {
                    continue;
                }
            }

            // Filter by WiX version if specified
            for rule in rule_file.rules {
                if let Some(v) = rule.wix_version {
                    if v != self.wix_version {
                        continue;
                    }
                }
                self.rules.push(rule);
                count += 1;
            }
        }

        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_id() {
        let plugin = WixPlugin::new();
        assert_eq!(plugin.id(), "wix");
    }

    #[test]
    fn test_plugin_extensions() {
        let plugin = WixPlugin::new();
        let exts = plugin.extensions();
        assert!(exts.contains(&"wxs"));
        assert!(exts.contains(&"wxi"));
        assert!(exts.contains(&"wxl"));
    }

    #[test]
    fn test_default_rules() {
        let plugin = WixPlugin::new();
        let rules = plugin.rules();
        assert!(!rules.is_empty());

        // Check that key rules exist
        let rule_ids: Vec<&str> = rules.iter().map(|r| r.id.as_str()).collect();
        assert!(rule_ids.contains(&"package-requires-upgradecode"));
        assert!(rule_ids.contains(&"component-requires-guid"));
    }

    #[test]
    fn test_parse_simple() {
        let plugin = WixPlugin::new();
        let content = r#"<?xml version="1.0"?>
<Wix xmlns="http://wixtoolset.org/schemas/v4/wxs">
    <Package Name="Test" Version="1.0.0" Manufacturer="Test" UpgradeCode="12345678-1234-1234-1234-123456789012">
    </Package>
</Wix>"#;

        let result = plugin.parse(content, Path::new("test.wxs"));
        assert!(result.is_ok());
    }
}
