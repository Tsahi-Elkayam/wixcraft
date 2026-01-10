//! Plugin system for language-specific providers
//!
//! Like Winter linter, Wintellisense supports language plugins defined via YAML manifests.
//! This allows extending the engine with new languages without code changes.
//!
//! # Example Plugin Manifest
//!
//! ```yaml
//! plugin:
//!   id: wix
//!   name: WiX Toolset
//!   version: "1.0.0"
//!   extensions: ["wxs", "wxi"]
//!
//! completions:
//!   elements: true
//!   attributes: true
//!   snippets: true
//!   values: true
//!
//! definitions:
//!   - source: ComponentRef
//!     target: Component
//!     attribute: Id
//!   - source: DirectoryRef
//!     target: Directory
//!     attribute: Id
//!
//! hover:
//!   elements: true
//!   attributes: true
//!   keywords: true
//! ```

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Plugin manifest defining language support
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManifest {
    /// Plugin metadata
    pub plugin: PluginMeta,

    /// Completion configuration
    #[serde(default)]
    pub completions: CompletionConfig,

    /// Definition mappings
    #[serde(default)]
    pub definitions: Vec<DefinitionMapping>,

    /// Hover configuration
    #[serde(default)]
    pub hover: HoverConfig,
}

/// Plugin metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginMeta {
    /// Unique plugin ID
    pub id: String,

    /// Human-readable name
    #[serde(default)]
    pub name: String,

    /// Plugin version
    #[serde(default)]
    pub version: String,

    /// File extensions this plugin handles
    #[serde(default)]
    pub extensions: Vec<String>,

    /// Description
    #[serde(default)]
    pub description: String,
}

/// Completion configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CompletionConfig {
    /// Enable element completions
    #[serde(default = "default_true")]
    pub elements: bool,

    /// Enable attribute completions
    #[serde(default = "default_true")]
    pub attributes: bool,

    /// Enable snippet completions
    #[serde(default = "default_true")]
    pub snippets: bool,

    /// Enable value completions
    #[serde(default = "default_true")]
    pub values: bool,

    /// Enable word completions (All Autocomplete style)
    #[serde(default = "default_true")]
    pub words: bool,
}

fn default_true() -> bool {
    true
}

/// Mapping for go-to-definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefinitionMapping {
    /// Source element (e.g., ComponentRef)
    pub source: String,

    /// Target element (e.g., Component)
    pub target: String,

    /// Attribute containing the reference (e.g., Id)
    #[serde(default = "default_id")]
    pub attribute: String,
}

fn default_id() -> String {
    "Id".to_string()
}

/// Hover configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HoverConfig {
    /// Show element documentation on hover
    #[serde(default = "default_true")]
    pub elements: bool,

    /// Show attribute documentation on hover
    #[serde(default = "default_true")]
    pub attributes: bool,

    /// Show keyword documentation on hover
    #[serde(default = "default_true")]
    pub keywords: bool,
}

impl PluginManifest {
    /// Load manifest from file
    pub fn load(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;

        let manifest: PluginManifest = if path.extension().map_or(false, |e| e == "json") {
            serde_json::from_str(&content)?
        } else {
            serde_yaml::from_str(&content)?
        };

        Ok(manifest)
    }

    /// Check if this plugin handles a file extension
    pub fn handles_extension(&self, ext: &str) -> bool {
        self.plugin.extensions.iter().any(|e| e == ext)
    }
}

/// Default WiX plugin configuration
pub fn default_wix_plugin() -> PluginManifest {
    PluginManifest {
        plugin: PluginMeta {
            id: "wix".to_string(),
            name: "WiX Toolset".to_string(),
            version: "1.0.0".to_string(),
            extensions: vec!["wxs".to_string(), "wxi".to_string()],
            description: "WiX XML installer files".to_string(),
        },
        completions: CompletionConfig {
            elements: true,
            attributes: true,
            snippets: true,
            values: true,
            words: true,
        },
        definitions: vec![
            DefinitionMapping {
                source: "ComponentRef".to_string(),
                target: "Component".to_string(),
                attribute: "Id".to_string(),
            },
            DefinitionMapping {
                source: "ComponentGroupRef".to_string(),
                target: "ComponentGroup".to_string(),
                attribute: "Id".to_string(),
            },
            DefinitionMapping {
                source: "DirectoryRef".to_string(),
                target: "Directory".to_string(),
                attribute: "Id".to_string(),
            },
            DefinitionMapping {
                source: "FeatureRef".to_string(),
                target: "Feature".to_string(),
                attribute: "Id".to_string(),
            },
            DefinitionMapping {
                source: "FeatureGroupRef".to_string(),
                target: "FeatureGroup".to_string(),
                attribute: "Id".to_string(),
            },
            DefinitionMapping {
                source: "PropertyRef".to_string(),
                target: "Property".to_string(),
                attribute: "Id".to_string(),
            },
            DefinitionMapping {
                source: "CustomActionRef".to_string(),
                target: "CustomAction".to_string(),
                attribute: "Id".to_string(),
            },
        ],
        hover: HoverConfig {
            elements: true,
            attributes: true,
            keywords: true,
        },
    }
}

/// Plugin manager for loading and managing language plugins
#[derive(Debug, Default)]
pub struct PluginManager {
    plugins: Vec<PluginManifest>,
}

impl PluginManager {
    pub fn new() -> Self {
        Self::default()
    }

    /// Create with default WiX plugin
    pub fn with_defaults() -> Self {
        let mut manager = Self::new();
        manager.register(default_wix_plugin());
        manager
    }

    /// Register a plugin
    pub fn register(&mut self, plugin: PluginManifest) {
        self.plugins.push(plugin);
    }

    /// Load plugins from a directory
    pub fn load_from_directory(&mut self, dir: &Path) -> Result<usize> {
        let mut count = 0;

        if !dir.exists() {
            return Ok(0);
        }

        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().map_or(false, |e| e == "yaml" || e == "yml" || e == "json") {
                if let Ok(manifest) = PluginManifest::load(&path) {
                    self.register(manifest);
                    count += 1;
                }
            }
        }

        Ok(count)
    }

    /// Get plugin for a file extension
    pub fn get_plugin_for_extension(&self, ext: &str) -> Option<&PluginManifest> {
        self.plugins.iter().find(|p| p.handles_extension(ext))
    }

    /// Get all registered plugins
    pub fn plugins(&self) -> &[PluginManifest] {
        &self.plugins
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_wix_plugin() {
        let plugin = default_wix_plugin();
        assert_eq!(plugin.plugin.id, "wix");
        assert!(plugin.handles_extension("wxs"));
        assert!(plugin.handles_extension("wxi"));
        assert!(!plugin.handles_extension("xml"));
    }

    #[test]
    fn test_completion_config_defaults() {
        let config = CompletionConfig::default();
        // Defaults should be false without serde default attribute
        assert!(!config.elements);
    }

    #[test]
    fn test_plugin_manager() {
        let manager = PluginManager::with_defaults();
        assert_eq!(manager.plugins().len(), 1);

        let plugin = manager.get_plugin_for_extension("wxs");
        assert!(plugin.is_some());
        assert_eq!(plugin.unwrap().plugin.id, "wix");
    }

    #[test]
    fn test_definition_mappings() {
        let plugin = default_wix_plugin();

        let component_ref = plugin
            .definitions
            .iter()
            .find(|d| d.source == "ComponentRef");
        assert!(component_ref.is_some());
        assert_eq!(component_ref.unwrap().target, "Component");
    }
}
