//! Formatting configuration

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// Indent style
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum IndentStyle {
    #[default]
    Space,
    Tab,
}

/// Formatting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FormatConfig {
    /// Indent style (space or tab)
    #[serde(default)]
    pub indent_style: IndentStyle,

    /// Number of spaces/tabs per indent level
    #[serde(default = "default_indent_size")]
    pub indent_size: usize,

    /// Maximum line width before wrapping attributes
    #[serde(default = "default_max_line_width")]
    pub max_line_width: usize,

    /// Number of attributes before switching to multiline
    #[serde(default = "default_attr_threshold")]
    pub attr_threshold: usize,

    /// Sort child elements by canonical wix-data order
    #[serde(default)]
    pub sort_elements: bool,

    /// Sort attributes (Id first, then required, then alphabetical)
    #[serde(default)]
    pub sort_attributes: bool,

    /// Remove trailing whitespace from lines
    #[serde(default = "default_true")]
    pub trim_trailing_whitespace: bool,

    /// Ensure file ends with newline
    #[serde(default = "default_true")]
    pub insert_final_newline: bool,
}

fn default_indent_size() -> usize {
    2
}

fn default_max_line_width() -> usize {
    120
}

fn default_attr_threshold() -> usize {
    3
}

fn default_true() -> bool {
    true
}

impl Default for FormatConfig {
    fn default() -> Self {
        Self {
            indent_style: IndentStyle::Space,
            indent_size: 2,
            max_line_width: 120,
            attr_threshold: 3,
            sort_elements: false,
            sort_attributes: false,
            trim_trailing_whitespace: true,
            insert_final_newline: true,
        }
    }
}

impl FormatConfig {
    /// Load configuration from a JSON file
    pub fn load(path: &Path) -> Result<Self, ConfigError> {
        let content = fs::read_to_string(path).map_err(|e| ConfigError::Read {
            path: path.to_path_buf(),
            source: e,
        })?;

        serde_json::from_str(&content).map_err(|e| ConfigError::Parse {
            path: path.to_path_buf(),
            source: e,
        })
    }

    /// Find and load config from standard locations
    pub fn find_and_load(start_dir: &Path) -> Option<Self> {
        let config_names = [".wixfmtrc.json", ".wixfmtrc", "wixfmt.json"];

        let mut current = Some(start_dir);
        while let Some(dir) = current {
            for name in &config_names {
                let config_path = dir.join(name);
                if config_path.exists() {
                    if let Ok(config) = Self::load(&config_path) {
                        return Some(config);
                    }
                }
            }
            current = dir.parent();
        }

        None
    }

    /// Get the indent string for one level
    pub fn indent_str(&self) -> String {
        match self.indent_style {
            IndentStyle::Space => " ".repeat(self.indent_size),
            IndentStyle::Tab => "\t".repeat(self.indent_size),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Failed to read config file {path}: {source}")]
    Read {
        path: std::path::PathBuf,
        source: std::io::Error,
    },
    #[error("Failed to parse config file {path}: {source}")]
    Parse {
        path: std::path::PathBuf,
        source: serde_json::Error,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_default_config() {
        let config = FormatConfig::default();
        assert_eq!(config.indent_style, IndentStyle::Space);
        assert_eq!(config.indent_size, 2);
        assert_eq!(config.max_line_width, 120);
        assert_eq!(config.attr_threshold, 3);
        assert!(!config.sort_elements);
        assert!(!config.sort_attributes);
        assert!(config.trim_trailing_whitespace);
        assert!(config.insert_final_newline);
    }

    #[test]
    fn test_indent_str_spaces() {
        let config = FormatConfig {
            indent_style: IndentStyle::Space,
            indent_size: 4,
            ..Default::default()
        };
        assert_eq!(config.indent_str(), "    ");
    }

    #[test]
    fn test_indent_str_tabs() {
        let config = FormatConfig {
            indent_style: IndentStyle::Tab,
            indent_size: 1,
            ..Default::default()
        };
        assert_eq!(config.indent_str(), "\t");
    }

    #[test]
    fn test_load_config() {
        let temp = TempDir::new().unwrap();
        let config_path = temp.path().join(".wixfmtrc.json");

        let config_json = r#"{
            "indentStyle": "tab",
            "indentSize": 1,
            "maxLineWidth": 80,
            "attrThreshold": 2,
            "sortElements": true,
            "sortAttributes": true
        }"#;
        fs::write(&config_path, config_json).unwrap();

        let config = FormatConfig::load(&config_path).unwrap();
        assert_eq!(config.indent_style, IndentStyle::Tab);
        assert_eq!(config.indent_size, 1);
        assert_eq!(config.max_line_width, 80);
        assert_eq!(config.attr_threshold, 2);
        assert!(config.sort_elements);
        assert!(config.sort_attributes);
    }

    #[test]
    fn test_load_partial_config() {
        let temp = TempDir::new().unwrap();
        let config_path = temp.path().join(".wixfmtrc.json");

        // Only specify some options
        let config_json = r#"{"indentSize": 4}"#;
        fs::write(&config_path, config_json).unwrap();

        let config = FormatConfig::load(&config_path).unwrap();
        assert_eq!(config.indent_size, 4);
        // Defaults for unspecified
        assert_eq!(config.indent_style, IndentStyle::Space);
        assert_eq!(config.max_line_width, 120);
    }

    #[test]
    fn test_find_and_load_in_current() {
        let temp = TempDir::new().unwrap();
        let config_path = temp.path().join(".wixfmtrc.json");
        fs::write(&config_path, r#"{"indentSize": 8}"#).unwrap();

        let config = FormatConfig::find_and_load(temp.path()).unwrap();
        assert_eq!(config.indent_size, 8);
    }

    #[test]
    fn test_find_and_load_in_parent() {
        let temp = TempDir::new().unwrap();
        let sub_dir = temp.path().join("subdir");
        fs::create_dir(&sub_dir).unwrap();

        let config_path = temp.path().join(".wixfmtrc.json");
        fs::write(&config_path, r#"{"indentSize": 6}"#).unwrap();

        let config = FormatConfig::find_and_load(&sub_dir).unwrap();
        assert_eq!(config.indent_size, 6);
    }

    #[test]
    fn test_find_and_load_not_found() {
        let temp = TempDir::new().unwrap();
        let config = FormatConfig::find_and_load(temp.path());
        assert!(config.is_none());
    }

    #[test]
    fn test_load_invalid_json() {
        let temp = TempDir::new().unwrap();
        let config_path = temp.path().join(".wixfmtrc.json");
        fs::write(&config_path, "not valid json").unwrap();

        let result = FormatConfig::load(&config_path);
        assert!(matches!(result, Err(ConfigError::Parse { .. })));
    }

    #[test]
    fn test_load_missing_file() {
        let result = FormatConfig::load(Path::new("/nonexistent/.wixfmtrc.json"));
        assert!(matches!(result, Err(ConfigError::Read { .. })));
    }

    #[test]
    fn test_config_error_display() {
        let err = ConfigError::Read {
            path: "/test/path".into(),
            source: std::io::Error::new(std::io::ErrorKind::NotFound, "not found"),
        };
        assert!(err.to_string().contains("/test/path"));
    }
}
