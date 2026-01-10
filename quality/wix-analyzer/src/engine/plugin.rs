//! Language plugin interface
//!
//! Plugins provide language-specific parsing and analysis capabilities.

use super::rule::{CodeRule, DataRule, RuleImpl};
use super::types::Document;
use std::path::Path;

/// Capabilities that a language plugin can provide
#[derive(Debug, Clone, Default)]
pub struct PluginCapabilities {
    /// Supports data-driven rules
    pub data_rules: bool,

    /// Supports code-based rules
    pub code_rules: bool,

    /// Supports cross-file analysis
    pub cross_file: bool,

    /// Supports incremental parsing
    pub incremental: bool,

    /// Supports auto-fix generation
    pub auto_fix: bool,
}

impl PluginCapabilities {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_data_rules(mut self) -> Self {
        self.data_rules = true;
        self
    }

    pub fn with_code_rules(mut self) -> Self {
        self.code_rules = true;
        self
    }

    pub fn with_cross_file(mut self) -> Self {
        self.cross_file = true;
        self
    }

    pub fn with_incremental(mut self) -> Self {
        self.incremental = true;
        self
    }

    pub fn with_auto_fix(mut self) -> Self {
        self.auto_fix = true;
        self
    }
}

/// Result of parsing a file
pub enum ParseResult {
    /// Successfully parsed
    Ok(Box<dyn Document>),

    /// Parse error with message and location
    Error {
        message: String,
        line: Option<usize>,
        column: Option<usize>,
    },
}

/// A language plugin that provides parsing and analysis for a specific language
pub trait LanguagePlugin: Send + Sync {
    /// Plugin identifier (e.g., "wix", "terraform", "dockerfile")
    fn id(&self) -> &str;

    /// Plugin display name (e.g., "WiX Toolset", "Terraform")
    fn name(&self) -> &str;

    /// Plugin version
    fn version(&self) -> &str;

    /// File extensions this plugin handles (e.g., [".wxs", ".wxi"])
    fn extensions(&self) -> &[&str];

    /// File patterns this plugin handles (e.g., ["Dockerfile*", "*.dockerfile"])
    fn patterns(&self) -> &[&str] {
        &[]
    }

    /// Check if this plugin can handle a file
    fn can_handle(&self, path: &Path) -> bool {
        // Check extensions
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            let ext_with_dot = format!(".{}", ext);
            if self.extensions().iter().any(|e| *e == ext_with_dot || *e == ext) {
                return true;
            }
        }

        // Check patterns
        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            for pattern in self.patterns() {
                if matches_glob(pattern, name) {
                    return true;
                }
            }
        }

        false
    }

    /// Get plugin capabilities
    fn capabilities(&self) -> PluginCapabilities;

    /// Parse a file into a document
    fn parse(&self, path: &Path, content: &str) -> ParseResult;

    /// Get all data-driven rules provided by this plugin
    fn data_rules(&self) -> Vec<DataRule> {
        Vec::new()
    }

    /// Get all code-based rules provided by this plugin
    fn code_rules(&self) -> Vec<Box<dyn CodeRule>> {
        Vec::new()
    }

    /// Get all rules (data + code)
    fn rules(&self) -> Vec<RuleImpl> {
        let mut rules: Vec<RuleImpl> = self
            .data_rules()
            .into_iter()
            .map(RuleImpl::Data)
            .collect();

        rules.extend(self.code_rules().into_iter().map(RuleImpl::Code));

        rules
    }

    /// Get rule by ID
    fn rule(&self, id: &str) -> Option<RuleImpl> {
        self.rules().into_iter().find(|r| r.id() == id)
    }

    /// Initialize the plugin (load resources, etc.)
    fn initialize(&mut self) -> Result<(), String> {
        Ok(())
    }

    /// Cleanup resources
    fn shutdown(&mut self) {}
}

/// Simple glob pattern matching (supports * and ?)
fn matches_glob(pattern: &str, text: &str) -> bool {
    let mut pattern_chars = pattern.chars().peekable();
    let mut text_chars = text.chars().peekable();

    while let Some(p) = pattern_chars.next() {
        match p {
            '*' => {
                // Match zero or more characters
                if pattern_chars.peek().is_none() {
                    return true; // * at end matches everything
                }

                // Try matching remaining pattern at each position
                let remaining_pattern: String = pattern_chars.collect();
                let mut remaining_text: String = text_chars.collect();

                while !remaining_text.is_empty() {
                    if matches_glob(&remaining_pattern, &remaining_text) {
                        return true;
                    }
                    remaining_text = remaining_text.chars().skip(1).collect();
                }

                return matches_glob(&remaining_pattern, "");
            }
            '?' => {
                // Match exactly one character
                if text_chars.next().is_none() {
                    return false;
                }
            }
            c => {
                // Match literal character
                if text_chars.next() != Some(c) {
                    return false;
                }
            }
        }
    }

    text_chars.peek().is_none()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_glob_matching() {
        // Exact match
        assert!(matches_glob("Dockerfile", "Dockerfile"));
        assert!(!matches_glob("Dockerfile", "dockerfile"));

        // Wildcard at end
        assert!(matches_glob("Dockerfile*", "Dockerfile"));
        assert!(matches_glob("Dockerfile*", "Dockerfile.dev"));
        assert!(matches_glob("Dockerfile*", "Dockerfile.prod"));
        assert!(!matches_glob("Dockerfile*", "notDockerfile"));

        // Wildcard at start
        assert!(matches_glob("*.dockerfile", "app.dockerfile"));
        assert!(matches_glob("*.dockerfile", "test.dockerfile"));
        assert!(!matches_glob("*.dockerfile", "dockerfile"));

        // Wildcard in middle
        assert!(matches_glob("docker*.yaml", "docker-compose.yaml"));
        assert!(matches_glob("docker*.yaml", "docker.yaml"));

        // Question mark
        assert!(matches_glob("file?.txt", "file1.txt"));
        assert!(matches_glob("file?.txt", "fileA.txt"));
        assert!(!matches_glob("file?.txt", "file12.txt"));

        // Complex patterns
        assert!(matches_glob("*.wix.*", "app.wix.xml"));
        assert!(matches_glob("test_*.rs", "test_main.rs"));
    }

    #[test]
    fn test_capabilities_builder() {
        let caps = PluginCapabilities::new()
            .with_data_rules()
            .with_code_rules()
            .with_auto_fix();

        assert!(caps.data_rules);
        assert!(caps.code_rules);
        assert!(caps.auto_fix);
        assert!(!caps.cross_file);
        assert!(!caps.incremental);
    }

    // Mock plugin for testing
    struct MockPlugin;

    impl LanguagePlugin for MockPlugin {
        fn id(&self) -> &str {
            "mock"
        }

        fn name(&self) -> &str {
            "Mock Language"
        }

        fn version(&self) -> &str {
            "1.0.0"
        }

        fn extensions(&self) -> &[&str] {
            &[".mock", ".mk"]
        }

        fn patterns(&self) -> &[&str] {
            &["Mockfile*"]
        }

        fn capabilities(&self) -> PluginCapabilities {
            PluginCapabilities::new().with_data_rules()
        }

        fn parse(&self, _path: &Path, _content: &str) -> ParseResult {
            ParseResult::Error {
                message: "Not implemented".to_string(),
                line: None,
                column: None,
            }
        }
    }

    #[test]
    fn test_plugin_can_handle() {
        let plugin = MockPlugin;

        // By extension
        assert!(plugin.can_handle(Path::new("file.mock")));
        assert!(plugin.can_handle(Path::new("file.mk")));
        assert!(!plugin.can_handle(Path::new("file.txt")));

        // By pattern
        assert!(plugin.can_handle(Path::new("Mockfile")));
        assert!(plugin.can_handle(Path::new("Mockfile.dev")));
        assert!(!plugin.can_handle(Path::new("notMockfile")));
    }

    #[test]
    fn test_plugin_interface() {
        let plugin = MockPlugin;

        assert_eq!(plugin.id(), "mock");
        assert_eq!(plugin.name(), "Mock Language");
        assert_eq!(plugin.version(), "1.0.0");
        assert!(plugin.capabilities().data_rules);
        assert!(!plugin.capabilities().code_rules);
    }
}
