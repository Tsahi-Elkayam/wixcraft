//! Configuration handling for wix-analyzer

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Configuration file name
pub const CONFIG_FILE_NAME: &str = ".wixanalyzer.json";

/// Main configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Default)]
pub struct Config {
    /// Which analyzers to run
    #[serde(default)]
    pub analyzers: AnalyzerConfig,

    /// Rule-specific configuration
    #[serde(default)]
    pub rules: RulesConfig,

    /// Fix-specific configuration
    #[serde(default)]
    pub fix: FixConfig,

    /// File patterns to exclude
    #[serde(default)]
    pub exclude: Vec<String>,

    /// Minimum severity to report
    #[serde(default)]
    pub min_severity: MinSeverity,
}

/// Analyzer enable/disable configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnalyzerConfig {
    #[serde(default = "default_true")]
    pub validation: bool,
    #[serde(default = "default_true")]
    pub best_practices: bool,
    #[serde(default = "default_true")]
    pub security: bool,
    #[serde(default = "default_true")]
    pub dead_code: bool,
}

fn default_true() -> bool {
    true
}

impl Default for AnalyzerConfig {
    fn default() -> Self {
        Self {
            validation: true,
            best_practices: true,
            security: true,
            dead_code: true,
        }
    }
}

/// Rule-specific configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RulesConfig {
    /// Rules to enable (supports wildcards like "SEC-*")
    #[serde(default)]
    pub enable: Vec<String>,

    /// Rules to disable (supports wildcards)
    #[serde(default)]
    pub disable: Vec<String>,

    /// Override severity for specific rules
    #[serde(default)]
    pub severity: HashMap<String, String>,
}

/// Fix-specific configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FixConfig {
    /// Rules to auto-apply without confirmation
    #[serde(default)]
    pub auto_apply: Vec<String>,

    /// Rules that require confirmation before fixing
    #[serde(default)]
    pub confirm: Vec<String>,
}

/// Minimum severity level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[derive(Default)]
pub enum MinSeverity {
    Error,
    Warning,
    #[default]
    Info,
}



impl Config {
    /// Load configuration from a file
    pub fn load(path: &Path) -> Result<Self, ConfigError> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| ConfigError::ReadError(path.to_path_buf(), e.to_string()))?;

        serde_json::from_str(&content)
            .map_err(|e| ConfigError::ParseError(path.to_path_buf(), e.to_string()))
    }

    /// Find and load configuration from the current directory or parents
    pub fn find_and_load(start_dir: &Path) -> Option<Self> {
        let mut current = start_dir.to_path_buf();

        loop {
            let config_path = current.join(CONFIG_FILE_NAME);
            if config_path.exists() {
                return Self::load(&config_path).ok();
            }

            if !current.pop() {
                break;
            }
        }

        None
    }

    /// Check if a rule should be enabled
    pub fn is_rule_enabled(&self, rule_id: &str) -> bool {
        // Check explicit disable first
        if self.matches_pattern(rule_id, &self.rules.disable) {
            return false;
        }

        // If enable list is empty, all rules are enabled by default
        if self.rules.enable.is_empty() {
            return true;
        }

        // Otherwise, check if rule matches enable patterns
        self.matches_pattern(rule_id, &self.rules.enable)
    }

    /// Check if a file should be excluded
    pub fn is_excluded(&self, path: &Path) -> bool {
        let path_str = path.to_string_lossy();
        for pattern in &self.exclude {
            if let Ok(glob) = glob::Pattern::new(pattern) {
                if glob.matches(&path_str) {
                    return true;
                }
            }
        }
        false
    }

    /// Get overridden severity for a rule
    pub fn get_severity_override(&self, rule_id: &str) -> Option<&str> {
        self.rules.severity.get(rule_id).map(|s| s.as_str())
    }

    /// Check if a rule should be auto-fixed
    pub fn should_auto_fix(&self, rule_id: &str) -> bool {
        self.matches_pattern(rule_id, &self.fix.auto_apply)
    }

    /// Check if a rule requires confirmation before fixing
    pub fn requires_fix_confirmation(&self, rule_id: &str) -> bool {
        self.matches_pattern(rule_id, &self.fix.confirm)
    }

    fn matches_pattern(&self, rule_id: &str, patterns: &[String]) -> bool {
        for pattern in patterns {
            if pattern.ends_with('*') {
                let prefix = &pattern[..pattern.len() - 1];
                if rule_id.starts_with(prefix) {
                    return true;
                }
            } else if pattern == rule_id {
                return true;
            }
        }
        false
    }
}

/// Configuration error
#[derive(Debug)]
pub enum ConfigError {
    ReadError(PathBuf, String),
    ParseError(PathBuf, String),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ReadError(path, msg) => {
                write!(
                    f,
                    "Failed to read config file '{}': {}",
                    path.display(),
                    msg
                )
            }
            Self::ParseError(path, msg) => {
                write!(
                    f,
                    "Failed to parse config file '{}': {}",
                    path.display(),
                    msg
                )
            }
        }
    }
}

impl std::error::Error for ConfigError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert!(config.analyzers.validation);
        assert!(config.analyzers.best_practices);
        assert!(config.analyzers.security);
        assert!(config.analyzers.dead_code);
    }

    #[test]
    fn test_rule_enabled_default() {
        let config = Config::default();
        assert!(config.is_rule_enabled("SEC-001"));
        assert!(config.is_rule_enabled("BP-IDIOM-001"));
    }

    #[test]
    fn test_rule_disabled() {
        let mut config = Config::default();
        config.rules.disable.push("SEC-001".to_string());
        assert!(!config.is_rule_enabled("SEC-001"));
        assert!(config.is_rule_enabled("SEC-002"));
    }

    #[test]
    fn test_rule_wildcard_disable() {
        let mut config = Config::default();
        config.rules.disable.push("SEC-*".to_string());
        assert!(!config.is_rule_enabled("SEC-001"));
        assert!(!config.is_rule_enabled("SEC-007"));
        assert!(config.is_rule_enabled("BP-001"));
    }

    #[test]
    fn test_rule_wildcard_enable() {
        let mut config = Config::default();
        config.rules.enable.push("SEC-*".to_string());
        assert!(config.is_rule_enabled("SEC-001"));
        assert!(!config.is_rule_enabled("BP-001"));
    }

    #[test]
    fn test_parse_config() {
        let json = r#"{
            "analyzers": {
                "validation": true,
                "security": false
            },
            "rules": {
                "disable": ["BP-MAINT-002"]
            },
            "minSeverity": "warning"
        }"#;

        let config: Config = serde_json::from_str(json).unwrap();
        assert!(config.analyzers.validation);
        assert!(!config.analyzers.security);
        assert!(!config.is_rule_enabled("BP-MAINT-002"));
        assert_eq!(config.min_severity, MinSeverity::Warning);
    }

    #[test]
    fn test_is_excluded() {
        let mut config = Config::default();
        config.exclude.push("**/generated/**".to_string());
        config.exclude.push("*.test.wxs".to_string());

        assert!(config.is_excluded(Path::new("src/generated/foo.wxs")));
        assert!(config.is_excluded(Path::new("foo.test.wxs")));
        assert!(!config.is_excluded(Path::new("src/main.wxs")));
    }

    #[test]
    fn test_severity_override() {
        let mut config = Config::default();
        config
            .rules
            .severity
            .insert("SEC-001".to_string(), "error".to_string());

        assert_eq!(config.get_severity_override("SEC-001"), Some("error"));
        assert_eq!(config.get_severity_override("SEC-002"), None);
    }

    #[test]
    fn test_auto_fix() {
        let mut config = Config::default();
        config.fix.auto_apply.push("BP-*".to_string());
        config.fix.confirm.push("DEAD-001".to_string());

        assert!(config.should_auto_fix("BP-IDIOM-001"));
        assert!(!config.should_auto_fix("SEC-001"));
        assert!(config.requires_fix_confirmation("DEAD-001"));
        assert!(!config.requires_fix_confirmation("DEAD-002"));
    }

    #[test]
    fn test_config_error_display() {
        let read_err = ConfigError::ReadError(PathBuf::from("test.json"), "not found".to_string());
        assert!(read_err.to_string().contains("Failed to read"));
        assert!(read_err.to_string().contains("test.json"));

        let parse_err = ConfigError::ParseError(PathBuf::from("bad.json"), "invalid".to_string());
        assert!(parse_err.to_string().contains("Failed to parse"));
        assert!(parse_err.to_string().contains("bad.json"));
    }

    #[test]
    fn test_load_nonexistent_config() {
        let result = Config::load(Path::new("/nonexistent/config.json"));
        assert!(result.is_err());
    }

    #[test]
    fn test_find_and_load_found() {
        use std::fs::File;
        use std::io::Write;
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join(".wixanalyzer.json");
        {
            let mut f = File::create(&config_path).unwrap();
            writeln!(f, r#"{{ "analyzers": {{ "validation": false }} }}"#).unwrap();
        }

        let found = Config::find_and_load(temp_dir.path());
        assert!(found.is_some());
        let config = found.unwrap();
        assert!(!config.analyzers.validation);
    }

    #[test]
    fn test_find_and_load_not_found() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        // No config file in this directory
        let found = Config::find_and_load(temp_dir.path());
        assert!(found.is_none());
    }

    #[test]
    fn test_find_and_load_in_parent() {
        use std::fs::{self, File};
        use std::io::Write;
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join(".wixanalyzer.json");
        {
            let mut f = File::create(&config_path).unwrap();
            writeln!(f, r#"{{ "analyzers": {{ "security": false }} }}"#).unwrap();
        }

        // Create a subdirectory
        let sub_dir = temp_dir.path().join("subdir");
        fs::create_dir(&sub_dir).unwrap();

        // Search from subdirectory should find parent's config
        let found = Config::find_and_load(&sub_dir);
        assert!(found.is_some());
        let config = found.unwrap();
        assert!(!config.analyzers.security);
    }

    #[test]
    fn test_min_severity_default() {
        assert_eq!(MinSeverity::default(), MinSeverity::Info);
    }

    #[test]
    fn test_analyzer_config_default() {
        let config = AnalyzerConfig::default();
        assert!(config.validation);
        assert!(config.best_practices);
        assert!(config.security);
        assert!(config.dead_code);
    }
}
