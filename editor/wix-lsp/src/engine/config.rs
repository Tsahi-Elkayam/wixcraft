//! Configuration loading for the LSP engine
//!
//! Loads settings from YAML configuration files.

use serde::Deserialize;
use std::path::{Path, PathBuf};

/// Main engine configuration
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
#[derive(Default)]
pub struct EngineConfig {
    /// Engine settings
    pub engine: EngineSettings,
    /// Plugin settings
    pub plugins: PluginSettings,
}

/// Core engine settings
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct EngineSettings {
    /// Server name
    pub name: String,
    /// Log level (trace, debug, info, warn, error)
    pub log_level: String,
    /// Enable workspace discovery
    pub workspace_discovery: bool,
}

/// Plugin configuration
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct PluginSettings {
    /// Paths to search for plugin data
    pub data_search_paths: Vec<String>,
    /// Required subdirectories in data path
    pub required_subdirs: Vec<String>,
    /// Enabled plugins (empty = all)
    pub enabled: Vec<String>,
}

impl Default for EngineSettings {
    fn default() -> Self {
        Self {
            name: "wix-lsp".to_string(),
            log_level: "info".to_string(),
            workspace_discovery: true,
        }
    }
}

impl Default for PluginSettings {
    fn default() -> Self {
        Self {
            data_search_paths: vec![
                "wix-data".to_string(),
                ".wix-data".to_string(),
                "src/core/wix-data".to_string(),
                "../wix-data".to_string(),
                "../../wix-data".to_string(),
            ],
            required_subdirs: vec!["elements".to_string()],
            enabled: vec![],
        }
    }
}

impl EngineConfig {
    /// Load configuration from a YAML file
    pub fn load(path: &Path) -> Result<Self, String> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read config file: {}", e))?;

        serde_yaml::from_str(&content).map_err(|e| format!("Failed to parse config: {}", e))
    }

    /// Load configuration from default locations
    pub fn load_default(workspace_root: &Path) -> Self {
        let candidates = [
            workspace_root.join(".wix-lsp.yaml"),
            workspace_root.join(".wix-lsp.yml"),
            workspace_root.join("wix-lsp.yaml"),
            workspace_root.join("wix-lsp.yml"),
        ];

        for candidate in &candidates {
            if candidate.exists() {
                if let Ok(config) = Self::load(candidate) {
                    return config;
                }
            }
        }

        Self::default()
    }

    /// Find data path in workspace
    pub fn find_data_path(&self, workspace_root: &Path) -> Option<PathBuf> {
        for search_path in &self.plugins.data_search_paths {
            let candidate = workspace_root.join(search_path);

            if candidate.exists() {
                // Check required subdirs
                let has_all_subdirs = self
                    .plugins
                    .required_subdirs
                    .iter()
                    .all(|subdir| candidate.join(subdir).exists());

                if has_all_subdirs {
                    return Some(candidate);
                }
            }
        }

        None
    }
}

/// Plugin-specific configuration loaded from plugin.yaml
#[derive(Debug, Clone, Deserialize)]
pub struct PluginManifest {
    /// Plugin name
    pub name: String,
    /// Plugin version
    pub version: String,
    /// Description
    pub description: Option<String>,
    /// Language configuration
    pub language: LanguageConfig,
    /// Capabilities
    #[serde(default)]
    pub capabilities: CapabilityConfig,
}

/// Language-specific configuration
#[derive(Debug, Clone, Deserialize)]
pub struct LanguageConfig {
    /// Language identifier
    pub id: String,
    /// File extensions
    pub extensions: Vec<String>,
    /// Trigger characters for completion
    #[serde(default)]
    pub trigger_characters: Vec<String>,
}

/// Capability flags
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct CapabilityConfig {
    pub completion: bool,
    pub hover: bool,
    pub document_symbols: bool,
    pub diagnostics: bool,
    pub formatting: bool,
    pub definition: bool,
    pub references: bool,
}

impl Default for CapabilityConfig {
    fn default() -> Self {
        Self {
            completion: true,
            hover: true,
            document_symbols: true,
            diagnostics: true,
            formatting: true,
            definition: false,
            references: false,
        }
    }
}

impl PluginManifest {
    /// Load plugin manifest from YAML
    pub fn load(path: &Path) -> Result<Self, String> {
        let content =
            std::fs::read_to_string(path).map_err(|e| format!("Failed to read manifest: {}", e))?;

        serde_yaml::from_str(&content).map_err(|e| format!("Failed to parse manifest: {}", e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_default_config() {
        let config = EngineConfig::default();
        assert_eq!(config.engine.name, "wix-lsp");
        assert_eq!(config.engine.log_level, "info");
        assert!(config.engine.workspace_discovery);
    }

    #[test]
    fn test_default_plugin_settings() {
        let settings = PluginSettings::default();
        assert!(!settings.data_search_paths.is_empty());
        assert!(settings.data_search_paths.contains(&"wix-data".to_string()));
    }

    #[test]
    fn test_find_data_path() {
        let temp_dir = TempDir::new().unwrap();
        let data_dir = temp_dir.path().join("wix-data");
        fs::create_dir(&data_dir).unwrap();
        fs::create_dir(data_dir.join("elements")).unwrap();

        let config = EngineConfig::default();
        let found = config.find_data_path(temp_dir.path());

        assert!(found.is_some());
        assert!(found.unwrap().ends_with("wix-data"));
    }

    #[test]
    fn test_find_data_path_missing_subdir() {
        let temp_dir = TempDir::new().unwrap();
        let data_dir = temp_dir.path().join("wix-data");
        fs::create_dir(&data_dir).unwrap();
        // Don't create elements subdir

        let config = EngineConfig::default();
        let found = config.find_data_path(temp_dir.path());

        assert!(found.is_none());
    }

    #[test]
    fn test_load_config_from_yaml() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("settings.yaml");

        let yaml = r#"
engine:
  name: test-lsp
  log_level: debug
  workspace_discovery: false

plugins:
  data_search_paths:
    - custom-data
  required_subdirs:
    - elements
    - rules
"#;
        fs::write(&config_path, yaml).unwrap();

        let config = EngineConfig::load(&config_path).unwrap();
        assert_eq!(config.engine.name, "test-lsp");
        assert_eq!(config.engine.log_level, "debug");
        assert!(!config.engine.workspace_discovery);
        assert_eq!(config.plugins.data_search_paths, vec!["custom-data"]);
    }

    #[test]
    fn test_load_default_config() {
        let temp_dir = TempDir::new().unwrap();

        // No config file - should return defaults
        let config = EngineConfig::load_default(temp_dir.path());
        assert_eq!(config.engine.name, "wix-lsp");
    }

    #[test]
    fn test_load_default_config_with_file() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join(".wix-lsp.yaml");

        let yaml = r#"
engine:
  name: custom-name
"#;
        fs::write(&config_path, yaml).unwrap();

        let config = EngineConfig::load_default(temp_dir.path());
        assert_eq!(config.engine.name, "custom-name");
    }

    #[test]
    fn test_capability_config_default() {
        let caps = CapabilityConfig::default();
        assert!(caps.completion);
        assert!(caps.hover);
        assert!(caps.document_symbols);
        assert!(caps.diagnostics);
        assert!(caps.formatting);
        assert!(!caps.definition);
        assert!(!caps.references);
    }
}
