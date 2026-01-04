//! Plugin system for wix-lint
//!
//! Plugins can add custom rules by providing JSON rule files in plugin directories.
//!
//! Plugin locations searched:
//! 1. ~/.wix-lint/plugins/
//! 2. .wix-lint-plugins/ in project directory
//! 3. Paths specified via --plugin-path or WIX_LINT_PLUGINS env var
//!
//! Each plugin is a directory containing:
//! - plugin.json: Plugin metadata (name, version, description)
//! - rules/*.json: Rule definition files (same format as wix-data rules)

use crate::rules::Rule;
use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PluginError {
    #[error("Failed to read plugin directory: {0}")]
    ReadDir(#[from] std::io::Error),
    #[error("Failed to parse plugin file {file}: {source}")]
    ParsePlugin {
        file: PathBuf,
        source: serde_json::Error,
    },
    #[error("Plugin directory not found: {0}")]
    NotFound(PathBuf),
}

/// Plugin metadata
#[derive(Debug, Deserialize)]
pub struct PluginMeta {
    pub name: String,
    pub version: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub author: String,
}

/// Plugin manager for loading and managing plugins
pub struct PluginManager {
    /// Directories to search for plugins
    search_paths: Vec<PathBuf>,
    /// Loaded plugins
    plugins: Vec<LoadedPlugin>,
}

/// A loaded plugin
#[derive(Debug)]
pub struct LoadedPlugin {
    pub meta: PluginMeta,
    pub path: PathBuf,
    pub rules: Vec<Rule>,
}

/// JSON structure for plugin rules (same as wix-data)
#[derive(Debug, Deserialize)]
struct RulesFile {
    rules: Vec<RuleJson>,
}

#[derive(Debug, Deserialize)]
struct RuleJson {
    id: String,
    name: String,
    description: String,
    severity: String,
    element: String,
    condition: String,
    message: String,
    #[serde(default)]
    fix: Option<FixJson>,
    #[serde(default)]
    since: Option<String>,
    #[serde(default)]
    deprecated: bool,
    #[serde(default, rename = "deprecatedMessage")]
    deprecated_message: Option<String>,
    #[serde(default, rename = "replacedBy")]
    replaced_by: Option<String>,
}

#[derive(Debug, Deserialize)]
struct FixJson {
    action: String,
    attribute: Option<String>,
    value: Option<String>,
}

impl PluginManager {
    /// Create a new plugin manager with default search paths
    pub fn new() -> Self {
        let mut search_paths = Vec::new();

        // User plugins directory
        if let Some(home) = dirs_next::home_dir() {
            search_paths.push(home.join(".wix-lint").join("plugins"));
        }

        // Project-local plugins
        if let Ok(cwd) = std::env::current_dir() {
            search_paths.push(cwd.join(".wix-lint-plugins"));
        }

        // Environment variable
        if let Ok(paths) = std::env::var("WIX_LINT_PLUGINS") {
            for path in paths.split(':') {
                search_paths.push(PathBuf::from(path));
            }
        }

        Self {
            search_paths,
            plugins: Vec::new(),
        }
    }

    /// Add a custom search path
    pub fn add_search_path(&mut self, path: PathBuf) {
        self.search_paths.push(path);
    }

    /// Discover and load all plugins
    pub fn load_all(&mut self) -> Result<(), PluginError> {
        for search_path in &self.search_paths.clone() {
            if search_path.exists() && search_path.is_dir() {
                self.load_plugins_from_dir(search_path)?;
            }
        }
        Ok(())
    }

    /// Load plugins from a directory
    fn load_plugins_from_dir(&mut self, dir: &Path) -> Result<(), PluginError> {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                if let Ok(plugin) = self.load_plugin(&path) {
                    self.plugins.push(plugin);
                }
            }
        }
        Ok(())
    }

    /// Load a single plugin
    fn load_plugin(&self, plugin_dir: &Path) -> Result<LoadedPlugin, PluginError> {
        // Read plugin.json
        let meta_path = plugin_dir.join("plugin.json");
        let meta_content = fs::read_to_string(&meta_path)?;
        let meta: PluginMeta = serde_json::from_str(&meta_content).map_err(|e| {
            PluginError::ParsePlugin {
                file: meta_path,
                source: e,
            }
        })?;

        // Load rules
        let mut rules = Vec::new();
        let rules_dir = plugin_dir.join("rules");
        if rules_dir.exists() {
            for entry in fs::read_dir(&rules_dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.extension().is_some_and(|ext| ext == "json") {
                    let content = fs::read_to_string(&path)?;
                    let rules_file: RulesFile =
                        serde_json::from_str(&content).map_err(|e| PluginError::ParsePlugin {
                            file: path,
                            source: e,
                        })?;

                    for r in rules_file.rules {
                        rules.push(Rule {
                            id: format!("{}/{}", meta.name, r.id),
                            name: r.name,
                            description: r.description,
                            severity: r.severity.parse().unwrap_or_default(),
                            element: r.element,
                            condition: r.condition,
                            message: r.message,
                            fix: r.fix.map(|f| crate::rules::FixTemplate {
                                action: f.action,
                                attribute: f.attribute,
                                value: f.value,
                            }),
                            since: r.since,
                            deprecated: r.deprecated,
                            deprecated_message: r.deprecated_message,
                            replaced_by: r.replaced_by,
                        });
                    }
                }
            }
        }

        Ok(LoadedPlugin {
            meta,
            path: plugin_dir.to_path_buf(),
            rules,
        })
    }

    /// Get all rules from all loaded plugins
    pub fn get_all_rules(&self) -> Vec<Rule> {
        self.plugins
            .iter()
            .flat_map(|p| p.rules.clone())
            .collect()
    }

    /// Get list of loaded plugins
    pub fn loaded_plugins(&self) -> &[LoadedPlugin] {
        &self.plugins
    }

    /// Get plugin count
    pub fn plugin_count(&self) -> usize {
        self.plugins.len()
    }

    /// Get total rule count from plugins
    pub fn rule_count(&self) -> usize {
        self.plugins.iter().map(|p| p.rules.len()).sum()
    }
}

impl Default for PluginManager {
    fn default() -> Self {
        Self::new()
    }
}

// Add dirs-next as optional dependency for home directory detection
// If not available, fall back to HOME env var
mod dirs_next {
    use std::path::PathBuf;

    pub fn home_dir() -> Option<PathBuf> {
        std::env::var_os("HOME")
            .or_else(|| std::env::var_os("USERPROFILE"))
            .map(PathBuf::from)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_plugin_manager_new() {
        let pm = PluginManager::new();
        assert!(!pm.search_paths.is_empty());
    }

    #[test]
    fn test_plugin_manager_default() {
        let pm = PluginManager::default();
        assert!(!pm.search_paths.is_empty());
        assert_eq!(pm.plugin_count(), 0);
        assert_eq!(pm.rule_count(), 0);
    }

    #[test]
    fn test_add_search_path() {
        let mut pm = PluginManager::new();
        let initial_count = pm.search_paths.len();
        pm.add_search_path(PathBuf::from("/custom/path"));
        assert_eq!(pm.search_paths.len(), initial_count + 1);
        assert!(pm.search_paths.contains(&PathBuf::from("/custom/path")));
    }

    #[test]
    fn test_load_all_empty() {
        let mut pm = PluginManager::new();
        // Clear search paths and add a non-existent one
        pm.search_paths.clear();
        pm.search_paths.push(PathBuf::from("/nonexistent/path"));

        // Should not error, just load nothing
        let result = pm.load_all();
        assert!(result.is_ok());
        assert_eq!(pm.plugin_count(), 0);
    }

    #[test]
    fn test_get_all_rules_empty() {
        let pm = PluginManager::new();
        assert!(pm.get_all_rules().is_empty());
    }

    #[test]
    fn test_loaded_plugins_empty() {
        let pm = PluginManager::new();
        assert!(pm.loaded_plugins().is_empty());
    }

    #[test]
    fn test_load_plugin_from_dir() {
        let temp_dir = TempDir::new().unwrap();
        let plugin_dir = temp_dir.path().join("my-plugin");
        fs::create_dir(&plugin_dir).unwrap();

        // Create plugin.json
        let plugin_meta = r#"{
            "name": "my-plugin",
            "version": "1.0.0",
            "description": "A test plugin",
            "author": "Test Author"
        }"#;
        fs::write(plugin_dir.join("plugin.json"), plugin_meta).unwrap();

        // Create rules directory
        let rules_dir = plugin_dir.join("rules");
        fs::create_dir(&rules_dir).unwrap();

        // Create a rules file
        let rules_content = r#"{
            "rules": [{
                "id": "custom-rule",
                "name": "Custom Rule",
                "description": "A custom rule from plugin",
                "severity": "warning",
                "element": "Component",
                "condition": "!attributes.Id",
                "message": "Component should have Id"
            }]
        }"#;
        fs::write(rules_dir.join("custom-rules.json"), rules_content).unwrap();

        let mut pm = PluginManager::new();
        pm.search_paths.clear();
        pm.search_paths.push(temp_dir.path().to_path_buf());

        let result = pm.load_all();
        assert!(result.is_ok());
        assert_eq!(pm.plugin_count(), 1);
        assert_eq!(pm.rule_count(), 1);

        let plugins = pm.loaded_plugins();
        assert_eq!(plugins[0].meta.name, "my-plugin");
        assert_eq!(plugins[0].meta.version, "1.0.0");
        assert_eq!(plugins[0].meta.description, "A test plugin");
        assert_eq!(plugins[0].meta.author, "Test Author");

        let rules = pm.get_all_rules();
        assert_eq!(rules.len(), 1);
        assert_eq!(rules[0].id, "my-plugin/custom-rule");
        assert_eq!(rules[0].element, "Component");
    }

    #[test]
    fn test_load_plugin_multiple_rules() {
        let temp_dir = TempDir::new().unwrap();
        let plugin_dir = temp_dir.path().join("multi-rule-plugin");
        fs::create_dir(&plugin_dir).unwrap();

        let plugin_meta = r#"{
            "name": "multi-rule-plugin",
            "version": "2.0.0"
        }"#;
        fs::write(plugin_dir.join("plugin.json"), plugin_meta).unwrap();

        let rules_dir = plugin_dir.join("rules");
        fs::create_dir(&rules_dir).unwrap();

        // File 1 with 2 rules
        let rules1 = r#"{
            "rules": [
                {
                    "id": "rule-a",
                    "name": "Rule A",
                    "description": "First rule",
                    "severity": "error",
                    "element": "Package",
                    "condition": "true",
                    "message": "Message A"
                },
                {
                    "id": "rule-b",
                    "name": "Rule B",
                    "description": "Second rule",
                    "severity": "warning",
                    "element": "File",
                    "condition": "true",
                    "message": "Message B"
                }
            ]
        }"#;
        fs::write(rules_dir.join("set1-rules.json"), rules1).unwrap();

        // File 2 with 1 rule
        let rules2 = r#"{
            "rules": [{
                "id": "rule-c",
                "name": "Rule C",
                "description": "Third rule",
                "severity": "info",
                "element": "Directory",
                "condition": "true",
                "message": "Message C"
            }]
        }"#;
        fs::write(rules_dir.join("set2-rules.json"), rules2).unwrap();

        let mut pm = PluginManager::new();
        pm.search_paths.clear();
        pm.search_paths.push(temp_dir.path().to_path_buf());

        pm.load_all().unwrap();
        assert_eq!(pm.plugin_count(), 1);
        assert_eq!(pm.rule_count(), 3);

        let rules = pm.get_all_rules();
        let rule_ids: Vec<&str> = rules.iter().map(|r| r.id.as_str()).collect();
        assert!(rule_ids.contains(&"multi-rule-plugin/rule-a"));
        assert!(rule_ids.contains(&"multi-rule-plugin/rule-b"));
        assert!(rule_ids.contains(&"multi-rule-plugin/rule-c"));
    }

    #[test]
    fn test_load_plugin_with_fix() {
        let temp_dir = TempDir::new().unwrap();
        let plugin_dir = temp_dir.path().join("fix-plugin");
        fs::create_dir(&plugin_dir).unwrap();

        let plugin_meta = r#"{"name": "fix-plugin", "version": "1.0.0"}"#;
        fs::write(plugin_dir.join("plugin.json"), plugin_meta).unwrap();

        let rules_dir = plugin_dir.join("rules");
        fs::create_dir(&rules_dir).unwrap();

        let rules = r#"{
            "rules": [{
                "id": "fixable-rule",
                "name": "Fixable Rule",
                "description": "A rule with a fix",
                "severity": "error",
                "element": "Component",
                "condition": "!attributes.Guid",
                "message": "Missing Guid",
                "fix": {
                    "action": "addAttribute",
                    "attribute": "Guid",
                    "value": "*"
                }
            }]
        }"#;
        fs::write(rules_dir.join("fix-rules.json"), rules).unwrap();

        let mut pm = PluginManager::new();
        pm.search_paths.clear();
        pm.search_paths.push(temp_dir.path().to_path_buf());

        pm.load_all().unwrap();
        let rules = pm.get_all_rules();
        assert!(rules[0].fix.is_some());
        let fix = rules[0].fix.as_ref().unwrap();
        assert_eq!(fix.action, "addAttribute");
        assert_eq!(fix.attribute, Some("Guid".to_string()));
        assert_eq!(fix.value, Some("*".to_string()));
    }

    #[test]
    fn test_load_plugin_no_rules_dir() {
        let temp_dir = TempDir::new().unwrap();
        let plugin_dir = temp_dir.path().join("no-rules-plugin");
        fs::create_dir(&plugin_dir).unwrap();

        let plugin_meta = r#"{"name": "no-rules-plugin", "version": "1.0.0"}"#;
        fs::write(plugin_dir.join("plugin.json"), plugin_meta).unwrap();

        // No rules directory created

        let mut pm = PluginManager::new();
        pm.search_paths.clear();
        pm.search_paths.push(temp_dir.path().to_path_buf());

        pm.load_all().unwrap();
        assert_eq!(pm.plugin_count(), 1);
        assert_eq!(pm.rule_count(), 0);
    }

    #[test]
    fn test_load_multiple_plugins() {
        let temp_dir = TempDir::new().unwrap();

        // Plugin 1
        let plugin1_dir = temp_dir.path().join("plugin1");
        fs::create_dir(&plugin1_dir).unwrap();
        fs::write(plugin1_dir.join("plugin.json"), r#"{"name": "plugin1", "version": "1.0"}"#).unwrap();
        let rules1_dir = plugin1_dir.join("rules");
        fs::create_dir(&rules1_dir).unwrap();
        fs::write(rules1_dir.join("rules.json"), r#"{"rules": [{"id": "r1", "name": "R1", "description": "D1", "severity": "error", "element": "A", "condition": "true", "message": "M1"}]}"#).unwrap();

        // Plugin 2
        let plugin2_dir = temp_dir.path().join("plugin2");
        fs::create_dir(&plugin2_dir).unwrap();
        fs::write(plugin2_dir.join("plugin.json"), r#"{"name": "plugin2", "version": "2.0"}"#).unwrap();
        let rules2_dir = plugin2_dir.join("rules");
        fs::create_dir(&rules2_dir).unwrap();
        fs::write(rules2_dir.join("rules.json"), r#"{"rules": [{"id": "r2", "name": "R2", "description": "D2", "severity": "warning", "element": "B", "condition": "true", "message": "M2"}]}"#).unwrap();

        let mut pm = PluginManager::new();
        pm.search_paths.clear();
        pm.search_paths.push(temp_dir.path().to_path_buf());

        pm.load_all().unwrap();
        assert_eq!(pm.plugin_count(), 2);
        assert_eq!(pm.rule_count(), 2);
    }

    #[test]
    fn test_home_dir_fallback() {
        // Test the dirs_next fallback module
        let home = dirs_next::home_dir();
        // Should return Some on any platform with HOME or USERPROFILE set
        // We just verify it doesn't panic
        assert!(home.is_some() || std::env::var_os("HOME").is_none() && std::env::var_os("USERPROFILE").is_none());
    }

    #[test]
    fn test_skip_non_json_files() {
        let temp_dir = TempDir::new().unwrap();
        let plugin_dir = temp_dir.path().join("skip-plugin");
        fs::create_dir(&plugin_dir).unwrap();

        fs::write(plugin_dir.join("plugin.json"), r#"{"name": "skip-plugin", "version": "1.0"}"#).unwrap();

        let rules_dir = plugin_dir.join("rules");
        fs::create_dir(&rules_dir).unwrap();

        // Valid JSON rules file
        fs::write(rules_dir.join("valid-rules.json"), r#"{"rules": [{"id": "r1", "name": "R1", "description": "D1", "severity": "error", "element": "A", "condition": "true", "message": "M1"}]}"#).unwrap();

        // Non-JSON files that should be skipped
        fs::write(rules_dir.join("readme.txt"), "This is not JSON").unwrap();
        fs::write(rules_dir.join("config.yaml"), "name: test").unwrap();

        let mut pm = PluginManager::new();
        pm.search_paths.clear();
        pm.search_paths.push(temp_dir.path().to_path_buf());

        pm.load_all().unwrap();
        assert_eq!(pm.rule_count(), 1);
    }

    #[test]
    fn test_skip_invalid_plugin_dir() {
        let temp_dir = TempDir::new().unwrap();

        // Create a file instead of directory
        fs::write(temp_dir.path().join("not-a-plugin"), "just a file").unwrap();

        // Create a valid plugin
        let plugin_dir = temp_dir.path().join("valid-plugin");
        fs::create_dir(&plugin_dir).unwrap();
        fs::write(plugin_dir.join("plugin.json"), r#"{"name": "valid", "version": "1.0"}"#).unwrap();

        let mut pm = PluginManager::new();
        pm.search_paths.clear();
        pm.search_paths.push(temp_dir.path().to_path_buf());

        pm.load_all().unwrap();
        // Only the valid plugin should be loaded
        assert_eq!(pm.plugin_count(), 1);
    }
}
