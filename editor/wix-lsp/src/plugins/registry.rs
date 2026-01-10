//! Plugin registry
//!
//! Manages plugin registration and routing requests to appropriate plugins
//! based on file extensions.

use super::traits::{
    Completion, CompletionProvider, Diagnostic, DiagnosticProvider, FormatProvider, HoverInfo,
    HoverProvider, LanguagePlugin, Symbol, SymbolProvider,
};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

/// Registry of language plugins
pub struct PluginRegistry {
    /// Registered plugins
    plugins: Vec<Arc<dyn FullPluginDyn>>,
    /// Map from file extension to plugin index
    extension_map: HashMap<String, usize>,
}

/// Object-safe version of FullPlugin for dynamic dispatch
pub trait FullPluginDyn: Send + Sync {
    fn as_language(&self) -> &dyn LanguagePlugin;
    fn as_completion(&self) -> &dyn CompletionProvider;
    fn as_hover(&self) -> &dyn HoverProvider;
    fn as_symbol(&self) -> &dyn SymbolProvider;
    fn as_diagnostic(&self) -> &dyn DiagnosticProvider;
    fn as_format(&self) -> &dyn FormatProvider;
}

impl PluginRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            plugins: Vec::new(),
            extension_map: HashMap::new(),
        }
    }

    /// Register a plugin
    pub fn register<P: FullPluginDyn + 'static>(&mut self, plugin: P) {
        let idx = self.plugins.len();
        let plugin = Arc::new(plugin);

        // Map extensions to this plugin
        for ext in plugin.as_language().file_extensions() {
            let ext = ext.trim_start_matches('.');
            self.extension_map.insert(ext.to_lowercase(), idx);
        }

        self.plugins.push(plugin);
    }

    /// Get plugin for a file path
    pub fn plugin_for_path(&self, path: &Path) -> Option<&Arc<dyn FullPluginDyn>> {
        let ext = path.extension()?.to_str()?.to_lowercase();
        let idx = self.extension_map.get(&ext)?;
        self.plugins.get(*idx)
    }

    /// Get plugin for a URI string
    pub fn plugin_for_uri(&self, uri: &str) -> Option<&Arc<dyn FullPluginDyn>> {
        // Extract extension from URI
        let path = uri.rsplit('/').next()?;
        let ext = path.rsplit('.').next()?;
        let idx = self.extension_map.get(&ext.to_lowercase())?;
        self.plugins.get(*idx)
    }

    /// Get all registered plugins
    pub fn plugins(&self) -> &[Arc<dyn FullPluginDyn>] {
        &self.plugins
    }

    /// Get combined trigger characters from all plugins
    pub fn all_trigger_characters(&self) -> Vec<String> {
        let mut chars: Vec<char> = self
            .plugins
            .iter()
            .flat_map(|p| p.as_language().trigger_characters().to_vec())
            .collect();
        chars.sort();
        chars.dedup();
        chars.into_iter().map(|c| c.to_string()).collect()
    }

    /// Get all supported extensions
    pub fn all_extensions(&self) -> Vec<String> {
        self.extension_map.keys().cloned().collect()
    }

    /// Check if any plugin handles the given extension
    pub fn supports_extension(&self, ext: &str) -> bool {
        self.extension_map
            .contains_key(&ext.trim_start_matches('.').to_lowercase())
    }

    /// Initialize all plugins with the given data path
    pub fn initialize_all(&mut self, _data_path: &Path) -> Vec<String> {
        let mut errors = Vec::new();

        for plugin in &self.plugins {
            // We need interior mutability here - in practice this would use RwLock
            // For now, plugins should initialize in their constructor
            if !plugin.as_language().is_initialized() {
                errors.push(format!(
                    "Plugin {} not initialized",
                    plugin.as_language().name()
                ));
            }
        }

        errors
    }
}

impl Default for PluginRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Wrapper to provide completion for a specific file
pub struct PluginCompletionContext<'a> {
    registry: &'a PluginRegistry,
    uri: &'a str,
}

impl<'a> PluginCompletionContext<'a> {
    pub fn new(registry: &'a PluginRegistry, uri: &'a str) -> Self {
        Self { registry, uri }
    }

    pub fn complete(&self, source: &str, line: u32, column: u32) -> Vec<Completion> {
        if let Some(plugin) = self.registry.plugin_for_uri(self.uri) {
            plugin.as_completion().complete(source, line, column)
        } else {
            Vec::new()
        }
    }
}

/// Wrapper to provide hover for a specific file
pub struct PluginHoverContext<'a> {
    registry: &'a PluginRegistry,
    uri: &'a str,
}

impl<'a> PluginHoverContext<'a> {
    pub fn new(registry: &'a PluginRegistry, uri: &'a str) -> Self {
        Self { registry, uri }
    }

    pub fn hover(&self, source: &str, line: u32, column: u32) -> Option<HoverInfo> {
        let plugin = self.registry.plugin_for_uri(self.uri)?;
        plugin.as_hover().hover(source, line, column)
    }
}

/// Wrapper to provide symbols for a specific file
pub struct PluginSymbolContext<'a> {
    registry: &'a PluginRegistry,
    uri: &'a str,
}

impl<'a> PluginSymbolContext<'a> {
    pub fn new(registry: &'a PluginRegistry, uri: &'a str) -> Self {
        Self { registry, uri }
    }

    pub fn symbols(&self, source: &str) -> Result<Vec<Symbol>, String> {
        if let Some(plugin) = self.registry.plugin_for_uri(self.uri) {
            plugin.as_symbol().symbols(source)
        } else {
            Ok(Vec::new())
        }
    }
}

/// Wrapper to provide diagnostics for a specific file
pub struct PluginDiagnosticContext<'a> {
    registry: &'a PluginRegistry,
    uri: &'a str,
}

impl<'a> PluginDiagnosticContext<'a> {
    pub fn new(registry: &'a PluginRegistry, uri: &'a str) -> Self {
        Self { registry, uri }
    }

    pub fn diagnose(&self, source: &str, path: &Path) -> Vec<Diagnostic> {
        if let Some(plugin) = self.registry.plugin_for_uri(self.uri) {
            plugin.as_diagnostic().diagnose(source, path)
        } else {
            Vec::new()
        }
    }
}

/// Wrapper to provide formatting for a specific file
pub struct PluginFormatContext<'a> {
    registry: &'a PluginRegistry,
    uri: &'a str,
}

impl<'a> PluginFormatContext<'a> {
    pub fn new(registry: &'a PluginRegistry, uri: &'a str) -> Self {
        Self { registry, uri }
    }

    pub fn format(&self, source: &str) -> Result<String, String> {
        if let Some(plugin) = self.registry.plugin_for_uri(self.uri) {
            plugin.as_format().format(source)
        } else {
            Err("No plugin for file type".to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_new() {
        let registry = PluginRegistry::new();
        assert!(registry.plugins().is_empty());
    }

    #[test]
    fn test_supports_extension() {
        let registry = PluginRegistry::new();
        assert!(!registry.supports_extension("wxs"));
    }

    #[test]
    fn test_all_trigger_characters_empty() {
        let registry = PluginRegistry::new();
        assert!(registry.all_trigger_characters().is_empty());
    }
}
