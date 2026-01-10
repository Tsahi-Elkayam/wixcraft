//! XML plugin for general XML validation

mod document;
mod rules;

pub use document::XmlDocument;

use crate::plugin::{Document, ParseError, Plugin, RuleLoadError};
use crate::rule::Rule;
use std::path::Path;

/// XML plugin for general XML file linting
pub struct XmlPlugin {
    /// Plugin version
    version: String,

    /// Loaded rules
    rules: Vec<Rule>,
}

impl Default for XmlPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl XmlPlugin {
    /// Create a new XML plugin
    pub fn new() -> Self {
        Self {
            version: env!("CARGO_PKG_VERSION").to_string(),
            rules: rules::builtin_rules(),
        }
    }
}

impl Plugin for XmlPlugin {
    fn id(&self) -> &str {
        "xml"
    }

    fn version(&self) -> &str {
        &self.version
    }

    fn description(&self) -> &str {
        "General XML linter plugin"
    }

    fn extensions(&self) -> &[&str] {
        &["xml"]
    }

    fn parse(&self, content: &str, path: &Path) -> Result<Box<dyn Document>, ParseError> {
        let doc = XmlDocument::parse(content, path)?;
        Ok(Box::new(doc))
    }

    fn rules(&self) -> &[Rule] {
        &self.rules
    }

    fn load_rules(&mut self, dir: &Path) -> Result<usize, RuleLoadError> {
        let mut count = 0;

        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
            if ext != "yaml" && ext != "yml" && ext != "json" {
                continue;
            }

            let content = std::fs::read_to_string(&path)?;
            let rule_file: crate::rule::RuleFile = match ext {
                "yaml" | "yml" => serde_yaml::from_str(&content).map_err(|e| {
                    RuleLoadError::Parse {
                        file: path.display().to_string(),
                        message: e.to_string(),
                    }
                })?,
                "json" => serde_json::from_str(&content).map_err(|e| RuleLoadError::Parse {
                    file: path.display().to_string(),
                    message: e.to_string(),
                })?,
                _ => continue,
            };

            if let Some(plugin) = &rule_file.plugin {
                if plugin != "xml" {
                    continue;
                }
            }

            for rule in rule_file.rules {
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
        let plugin = XmlPlugin::new();
        assert_eq!(plugin.id(), "xml");
    }

    #[test]
    fn test_plugin_extensions() {
        let plugin = XmlPlugin::new();
        assert!(plugin.extensions().contains(&"xml"));
    }

    #[test]
    fn test_default_rules() {
        let plugin = XmlPlugin::new();
        assert!(!plugin.rules().is_empty());
    }
}
