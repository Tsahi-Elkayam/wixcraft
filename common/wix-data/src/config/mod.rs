//! Configuration management for WiX Data Layer

use crate::{Result, WixDataError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// Sources configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourcesConfig {
    pub version: String,
    pub sources: HashMap<String, HashMap<String, SourceDef>>,
    pub parsers: HashMap<String, ParserConfig>,
    pub harvest: HarvestSettings,
}

/// Single source definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceDef {
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub path: Option<String>,
    pub parser: String,
    pub targets: Vec<String>,
    #[serde(default)]
    pub extension: Option<String>,
}

/// Parser configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParserConfig {
    #[serde(rename = "type")]
    pub parser_type: String,
    #[serde(default)]
    pub extract: Option<Vec<HashMap<String, String>>>,
    #[serde(default)]
    pub selectors: Option<HashMap<String, String>>,
    #[serde(default)]
    pub validate_schema: Option<String>,
    #[serde(default)]
    pub map: Option<HashMap<String, HashMap<String, String>>>,
}

/// Harvest settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HarvestSettings {
    pub cache_dir: String,
    pub timeout_seconds: u64,
    pub retry_count: u32,
    pub user_agent: String,
    pub rate_limit: RateLimitConfig,
}

/// Rate limiting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    pub requests_per_second: u32,
    pub burst: u32,
}

impl SourcesConfig {
    /// Load configuration from YAML file
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: SourcesConfig = serde_yaml::from_str(&content)
            .map_err(|e| WixDataError::Config(format!("Failed to parse YAML: {}", e)))?;
        Ok(config)
    }

    /// Save configuration to YAML file
    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let content = serde_yaml::to_string(self)
            .map_err(|e| WixDataError::Config(format!("Failed to serialize YAML: {}", e)))?;
        std::fs::write(path, content)?;
        Ok(())
    }

    /// Get all sources of a specific category
    pub fn get_sources(&self, category: &str) -> Option<&HashMap<String, SourceDef>> {
        self.sources.get(category)
    }

    /// Get a specific source
    pub fn get_source(&self, category: &str, name: &str) -> Option<&SourceDef> {
        self.sources.get(category)?.get(name)
    }

    /// Get parser configuration
    pub fn get_parser(&self, name: &str) -> Option<&ParserConfig> {
        self.parsers.get(name)
    }

    /// List all categories
    pub fn categories(&self) -> Vec<&String> {
        self.sources.keys().collect()
    }

    /// List all sources in a category
    pub fn list_sources(&self, category: &str) -> Vec<&String> {
        self.sources
            .get(category)
            .map(|s| s.keys().collect())
            .unwrap_or_default()
    }

    /// Count total sources
    pub fn total_sources(&self) -> usize {
        self.sources.values().map(|s| s.len()).sum()
    }
}

/// Lint configuration (for user overrides)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LintConfig {
    #[serde(default)]
    pub enabled_rules: Vec<String>,
    #[serde(default)]
    pub disabled_rules: Vec<String>,
    #[serde(default)]
    pub severity_overrides: HashMap<String, String>,
}

impl LintConfig {
    /// Load from JSON file
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: LintConfig = serde_json::from_str(&content)
            .map_err(|e| WixDataError::Config(format!("Failed to parse JSON: {}", e)))?;
        Ok(config)
    }

    /// Check if a rule is enabled
    pub fn is_rule_enabled(&self, rule_id: &str) -> bool {
        if self.disabled_rules.contains(&rule_id.to_string()) {
            return false;
        }
        if self.enabled_rules.is_empty() {
            return true;
        }
        self.enabled_rules.contains(&rule_id.to_string())
    }

    /// Get severity override for a rule
    pub fn get_severity(&self, rule_id: &str) -> Option<&String> {
        self.severity_overrides.get(rule_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn create_test_sources_config() -> SourcesConfig {
        let mut sources = HashMap::new();
        let mut elements = HashMap::new();
        elements.insert("test-source".to_string(), SourceDef {
            url: Some("https://example.com".to_string()),
            path: None,
            parser: "json".to_string(),
            targets: vec!["keywords".to_string()],
            extension: None,
        });
        sources.insert("elements".to_string(), elements);

        let mut parsers = HashMap::new();
        parsers.insert("json".to_string(), ParserConfig {
            parser_type: "json".to_string(),
            extract: None,
            selectors: None,
            validate_schema: None,
            map: None,
        });

        SourcesConfig {
            version: "1.0".to_string(),
            sources,
            parsers,
            harvest: HarvestSettings {
                cache_dir: ".cache".to_string(),
                timeout_seconds: 30,
                retry_count: 3,
                user_agent: "test".to_string(),
                rate_limit: RateLimitConfig {
                    requests_per_second: 2,
                    burst: 5,
                },
            },
        }
    }

    #[test]
    fn test_lint_config_default() {
        let config = LintConfig::default();
        assert!(config.is_rule_enabled("any_rule"));
    }

    #[test]
    fn test_lint_config_disabled() {
        let config = LintConfig {
            disabled_rules: vec!["RULE001".to_string()],
            ..Default::default()
        };
        assert!(!config.is_rule_enabled("RULE001"));
        assert!(config.is_rule_enabled("RULE002"));
    }

    #[test]
    fn test_lint_config_enabled_only() {
        let config = LintConfig {
            enabled_rules: vec!["RULE001".to_string()],
            ..Default::default()
        };
        assert!(config.is_rule_enabled("RULE001"));
        assert!(!config.is_rule_enabled("RULE002"));
    }

    #[test]
    fn test_lint_config_severity_override() {
        let mut overrides = HashMap::new();
        overrides.insert("RULE001".to_string(), "error".to_string());

        let config = LintConfig {
            severity_overrides: overrides,
            ..Default::default()
        };

        assert_eq!(config.get_severity("RULE001"), Some(&"error".to_string()));
        assert_eq!(config.get_severity("RULE002"), None);
    }

    #[test]
    fn test_sources_config_save_and_load() {
        let dir = tempdir().unwrap();
        let config_path = dir.path().join("sources.yaml");

        let config = create_test_sources_config();
        config.save(&config_path).unwrap();

        let loaded = SourcesConfig::load(&config_path).unwrap();
        assert_eq!(loaded.version, "1.0");
        assert_eq!(loaded.total_sources(), 1);
    }

    #[test]
    fn test_sources_config_get_source() {
        let config = create_test_sources_config();

        let source = config.get_source("elements", "test-source");
        assert!(source.is_some());
        assert_eq!(source.unwrap().parser, "json");

        let missing = config.get_source("elements", "nonexistent");
        assert!(missing.is_none());

        let missing_cat = config.get_source("nonexistent", "test");
        assert!(missing_cat.is_none());
    }

    #[test]
    fn test_sources_config_get_parser() {
        let config = create_test_sources_config();

        let parser = config.get_parser("json");
        assert!(parser.is_some());
        assert_eq!(parser.unwrap().parser_type, "json");

        let missing = config.get_parser("nonexistent");
        assert!(missing.is_none());
    }

    #[test]
    fn test_sources_config_categories() {
        let config = create_test_sources_config();
        let cats = config.categories();
        assert_eq!(cats.len(), 1);
        assert!(cats.contains(&&"elements".to_string()));
    }

    #[test]
    fn test_sources_config_list_sources() {
        let config = create_test_sources_config();

        let sources = config.list_sources("elements");
        assert_eq!(sources.len(), 1);

        let empty = config.list_sources("nonexistent");
        assert!(empty.is_empty());
    }
}
