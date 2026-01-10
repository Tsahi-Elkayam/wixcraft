//! Configuration system for linter engine
//!
//! Reads configuration from:
//! - `.linterrc.yaml` / `.linterrc.json` (project-level)
//! - `~/.linterrc.yaml` (user-level)
//! - SQLite database (for rules)

use crate::diagnostic::Severity;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use thiserror::Error;

/// Configuration error
#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("YAML parse error: {0}")]
    Yaml(#[from] serde_yaml::Error),

    #[error("JSON parse error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Invalid configuration: {0}")]
    Invalid(String),
}

/// Engine settings
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct EngineConfig {
    /// Enable parallel processing
    pub parallel: bool,

    /// Number of parallel jobs (0 = auto-detect)
    pub jobs: usize,

    /// Enable caching
    pub cache: bool,

    /// Cache directory
    pub cache_dir: Option<PathBuf>,
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            parallel: true,
            jobs: 0,
            cache: false,
            cache_dir: None,
        }
    }
}

/// Output settings
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct OutputConfig {
    /// Output format
    pub format: OutputFormat,

    /// Color mode
    pub color: ColorMode,

    /// Verbose output
    pub verbose: bool,

    /// Show statistics
    pub statistics: bool,
}

impl Default for OutputConfig {
    fn default() -> Self {
        Self {
            format: OutputFormat::Text,
            color: ColorMode::Auto,
            verbose: false,
            statistics: true,
        }
    }
}

/// Output format options
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OutputFormat {
    #[default]
    Text,
    Json,
    Sarif,
    Github,
    Compact,
    Grouped,
    Junit,
    Gitlab,
    Azure,
}

impl std::str::FromStr for OutputFormat {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "text" => Ok(OutputFormat::Text),
            "json" => Ok(OutputFormat::Json),
            "sarif" => Ok(OutputFormat::Sarif),
            "github" => Ok(OutputFormat::Github),
            "compact" => Ok(OutputFormat::Compact),
            "grouped" => Ok(OutputFormat::Grouped),
            "junit" => Ok(OutputFormat::Junit),
            "gitlab" => Ok(OutputFormat::Gitlab),
            "azure" => Ok(OutputFormat::Azure),
            _ => Err(format!("Unknown output format: {}", s)),
        }
    }
}

/// Color mode options
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ColorMode {
    #[default]
    Auto,
    Always,
    Never,
}

/// File handling settings
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct FilesConfig {
    /// Include patterns
    pub include: Vec<String>,

    /// Exclude patterns
    pub exclude: Vec<String>,
}

impl Default for FilesConfig {
    fn default() -> Self {
        Self {
            include: vec!["**/*.wxs".to_string(), "**/*.wxi".to_string()],
            exclude: vec![
                "**/generated/**".to_string(),
                "**/*.g.wxs".to_string(),
                "**/node_modules/**".to_string(),
                "**/target/**".to_string(),
            ],
        }
    }
}

/// Plugin-specific configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct PluginConfig {
    /// Whether the plugin is enabled
    pub enabled: bool,

    /// Custom rules directory
    pub rules_dir: Option<PathBuf>,

    /// Plugin-specific settings
    #[serde(flatten)]
    pub settings: HashMap<String, serde_yaml::Value>,
}

/// Rule configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct RulesConfig {
    /// Disabled rules
    pub disabled: Vec<String>,

    /// Enabled rules (empty = all)
    pub enabled: Vec<String>,

    /// Select rules by prefix (e.g., "WIX" selects all WIX* rules)
    pub extend: Vec<String>,

    /// Ignore rules by prefix (e.g., "WIX" ignores all WIX* rules)
    pub ignore: Vec<String>,

    /// Severity overrides (rule_id -> severity)
    pub severity: HashMap<String, Severity>,

    /// Per-file rule ignores (glob pattern -> rule IDs)
    pub per_file: HashMap<String, Vec<String>>,
}

/// Inline disable comment configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct InlineDisableConfig {
    /// Comment prefix
    pub prefix: String,

    /// Supported formats
    pub formats: Vec<String>,
}

impl Default for InlineDisableConfig {
    fn default() -> Self {
        Self {
            prefix: "winter-disable".to_string(),
            formats: vec![
                "winter-disable {rule}".to_string(),
                "winter-disable-next-line {rule}".to_string(),
                "winter-disable-file {rule}".to_string(),
            ],
        }
    }
}

/// Main configuration structure
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    /// Extend from other configuration files or presets
    /// Can be a single path/preset or a list
    #[serde(default)]
    pub extends: Vec<String>,

    /// Engine settings
    pub engine: EngineConfig,

    /// Output settings
    pub output: OutputConfig,

    /// File handling settings
    pub files: FilesConfig,

    /// Plugin configurations
    pub plugins: HashMap<String, PluginConfig>,

    /// Rule configuration
    pub rules: RulesConfig,

    /// Inline disable configuration
    pub inline_disable: InlineDisableConfig,

    /// Enable preview/experimental rules
    #[serde(default)]
    pub preview: bool,

    /// Rule categories to enable (empty = all stable)
    #[serde(default)]
    pub categories: Vec<String>,
}

impl Config {
    /// Create default configuration
    pub fn new() -> Self {
        Self::default()
    }

    /// Get a preset configuration by name
    pub fn preset(name: &str) -> Option<Self> {
        match name {
            "recommended" => Some(Self::preset_recommended()),
            "strict" => Some(Self::preset_strict()),
            "minimal" => Some(Self::preset_minimal()),
            _ => None,
        }
    }

    /// Recommended preset - balanced defaults
    fn preset_recommended() -> Self {
        Self {
            categories: vec![
                "correctness".to_string(),
                "suspicious".to_string(),
                "style".to_string(),
            ],
            ..Self::default()
        }
    }

    /// Strict preset - all rules enabled
    fn preset_strict() -> Self {
        Self {
            preview: true,
            categories: vec![
                "correctness".to_string(),
                "suspicious".to_string(),
                "style".to_string(),
                "perf".to_string(),
                "pedantic".to_string(),
            ],
            ..Self::default()
        }
    }

    /// Minimal preset - only critical rules
    fn preset_minimal() -> Self {
        Self {
            categories: vec!["correctness".to_string()],
            ..Self::default()
        }
    }

    /// Load configuration from a file
    pub fn load(path: &Path) -> Result<Self, ConfigError> {
        Self::load_with_depth(path, 0)
    }

    /// Load with recursion depth limit (to prevent infinite loops)
    fn load_with_depth(path: &Path, depth: usize) -> Result<Self, ConfigError> {
        const MAX_DEPTH: usize = 10;
        if depth >= MAX_DEPTH {
            return Err(ConfigError::Invalid(
                "Maximum config inheritance depth exceeded".to_string(),
            ));
        }

        let content = std::fs::read_to_string(path)?;

        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");

        let mut config: Self = match ext {
            "yaml" | "yml" => serde_yaml::from_str(&content)?,
            "json" => serde_json::from_str(&content)?,
            _ => {
                return Err(ConfigError::Invalid(format!(
                    "Unknown config file format: {}",
                    ext
                )))
            }
        };

        // Process extends
        if !config.extends.is_empty() {
            let base_dir = path.parent().unwrap_or(Path::new("."));
            let mut base_config = Self::default();

            for extend in &config.extends.clone() {
                let extended = if let Some(preset) = Self::preset(extend) {
                    // It's a preset name
                    preset
                } else {
                    // It's a file path
                    let extend_path = if Path::new(extend).is_absolute() {
                        PathBuf::from(extend)
                    } else {
                        base_dir.join(extend)
                    };
                    Self::load_with_depth(&extend_path, depth + 1)?
                };
                base_config.merge(extended);
            }

            // Merge current config on top of base
            base_config.merge(config);
            config = base_config;
        }

        Ok(config)
    }

    /// Merge another config into this one (other takes precedence)
    pub fn merge(&mut self, other: Self) {
        // Extends are not inherited
        // self.extends remains unchanged

        // Engine settings - other takes precedence if non-default
        if other.engine.jobs != 0 {
            self.engine.jobs = other.engine.jobs;
        }
        // parallel always inherits from other
        self.engine.parallel = other.engine.parallel;
        if other.engine.cache {
            self.engine.cache = other.engine.cache;
        }
        if other.engine.cache_dir.is_some() {
            self.engine.cache_dir = other.engine.cache_dir;
        }

        // Output settings
        if other.output.format != OutputFormat::Text {
            self.output.format = other.output.format;
        }
        if other.output.verbose {
            self.output.verbose = true;
        }
        if other.output.color != ColorMode::Auto {
            self.output.color = other.output.color;
        }

        // Files - extend lists
        self.files.include.extend(other.files.include);
        self.files.exclude.extend(other.files.exclude);

        // Plugins - merge maps
        for (id, plugin_config) in other.plugins {
            self.plugins.insert(id, plugin_config);
        }

        // Rules - merge
        self.rules.disabled.extend(other.rules.disabled);
        if !other.rules.enabled.is_empty() {
            self.rules.enabled = other.rules.enabled;
        }
        self.rules.extend.extend(other.rules.extend);
        self.rules.ignore.extend(other.rules.ignore);
        self.rules.severity.extend(other.rules.severity);
        for (pattern, rules) in other.rules.per_file {
            self.rules.per_file.entry(pattern).or_default().extend(rules);
        }

        // Preview and categories
        if other.preview {
            self.preview = true;
        }
        if !other.categories.is_empty() {
            self.categories = other.categories;
        }
    }

    /// Load configuration from default locations
    pub fn load_default() -> Result<Self, ConfigError> {
        let config_names = [
            ".linterrc.yaml",
            ".linterrc.yml",
            ".linterrc.json",
            "linter.yaml",
            "linter.yml",
            "linter.json",
        ];

        // Check current directory
        for name in &config_names {
            let path = PathBuf::from(name);
            if path.exists() {
                return Self::load(&path);
            }
        }

        // Check home directory
        if let Some(home) = dirs::home_dir() {
            for name in &config_names {
                let path = home.join(name);
                if path.exists() {
                    return Self::load(&path);
                }
            }
        }

        // Return default config
        Ok(Self::default())
    }

    /// Merge CLI arguments into configuration
    pub fn merge_cli(
        &mut self,
        format: Option<OutputFormat>,
        verbose: Option<bool>,
        jobs: Option<usize>,
        disabled_rules: Option<Vec<String>>,
        enabled_rules: Option<Vec<String>>,
    ) {
        if let Some(f) = format {
            self.output.format = f;
        }
        if let Some(v) = verbose {
            self.output.verbose = v;
        }
        if let Some(j) = jobs {
            self.engine.jobs = j;
        }
        if let Some(disabled) = disabled_rules {
            self.rules.disabled.extend(disabled);
        }
        if let Some(enabled) = enabled_rules {
            self.rules.enabled = enabled;
        }
    }

    /// Add prefixes to extend (select rules by prefix)
    pub fn add_extend_prefixes(&mut self, prefixes: Vec<String>) {
        self.rules.extend.extend(prefixes);
    }

    /// Add prefixes to ignore
    pub fn add_ignore_prefixes(&mut self, prefixes: Vec<String>) {
        self.rules.ignore.extend(prefixes);
    }

    /// Check if a rule is enabled
    pub fn is_rule_enabled(&self, rule_id: &str) -> bool {
        // If explicitly disabled, return false
        if self.rules.disabled.contains(&rule_id.to_string()) {
            return false;
        }

        // Check if rule matches any ignore prefix (case-insensitive)
        let rule_upper = rule_id.to_uppercase();
        for prefix in &self.rules.ignore {
            if rule_upper.starts_with(&prefix.to_uppercase()) {
                return false;
            }
        }

        // If enabled list is not empty, rule must be in it
        if !self.rules.enabled.is_empty() {
            return self.rules.enabled.contains(&rule_id.to_string());
        }

        // If extend list is not empty, rule must match one of the prefixes
        if !self.rules.extend.is_empty() {
            for prefix in &self.rules.extend {
                if rule_upper.starts_with(&prefix.to_uppercase()) {
                    return true;
                }
            }
            return false;
        }

        true
    }

    /// Check if a rule matches any prefix in the extend list
    pub fn matches_extend_prefix(&self, rule_id: &str) -> bool {
        if self.rules.extend.is_empty() {
            return true; // No prefix filter = all match
        }
        let rule_upper = rule_id.to_uppercase();
        for prefix in &self.rules.extend {
            if rule_upper.starts_with(&prefix.to_uppercase()) {
                return true;
            }
        }
        false
    }

    /// Check if a rule matches any prefix in the ignore list
    pub fn matches_ignore_prefix(&self, rule_id: &str) -> bool {
        let rule_upper = rule_id.to_uppercase();
        for prefix in &self.rules.ignore {
            if rule_upper.starts_with(&prefix.to_uppercase()) {
                return true;
            }
        }
        false
    }

    /// Get severity override for a rule
    pub fn get_severity_override(&self, rule_id: &str) -> Option<Severity> {
        self.rules.severity.get(rule_id).copied()
    }

    /// Check if a rule should be ignored for a file
    pub fn should_ignore_rule_for_file(&self, rule_id: &str, file_path: &Path) -> bool {
        let file_str = file_path.to_string_lossy();

        for (pattern, rules) in &self.rules.per_file {
            if let Ok(glob) = globset::Glob::new(pattern) {
                let matcher = glob.compile_matcher();
                if matcher.is_match(file_str.as_ref())
                    && (rules.contains(&"all".to_string())
                        || rules.contains(&rule_id.to_string()))
                {
                    return true;
                }
            }
        }

        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::new();
        assert!(config.engine.parallel);
        assert_eq!(config.engine.jobs, 0);
        assert_eq!(config.output.format, OutputFormat::Text);
        assert!(!config.files.include.is_empty());
    }

    #[test]
    fn test_output_format_from_str() {
        assert_eq!("text".parse::<OutputFormat>().unwrap(), OutputFormat::Text);
        assert_eq!("json".parse::<OutputFormat>().unwrap(), OutputFormat::Json);
        assert_eq!("sarif".parse::<OutputFormat>().unwrap(), OutputFormat::Sarif);
        assert!("invalid".parse::<OutputFormat>().is_err());
    }

    #[test]
    fn test_config_merge_cli() {
        let mut config = Config::new();
        config.merge_cli(
            Some(OutputFormat::Json),
            Some(true),
            Some(4),
            Some(vec!["rule1".to_string()]),
            None,
        );

        assert_eq!(config.output.format, OutputFormat::Json);
        assert!(config.output.verbose);
        assert_eq!(config.engine.jobs, 4);
        assert!(config.rules.disabled.contains(&"rule1".to_string()));
    }

    #[test]
    fn test_rule_enabled() {
        let mut config = Config::new();

        // All rules enabled by default
        assert!(config.is_rule_enabled("any-rule"));

        // Disable a rule
        config.rules.disabled.push("disabled-rule".to_string());
        assert!(!config.is_rule_enabled("disabled-rule"));
        assert!(config.is_rule_enabled("other-rule"));

        // Set enabled list
        config.rules.enabled = vec!["only-this".to_string()];
        assert!(!config.is_rule_enabled("disabled-rule"));
        assert!(!config.is_rule_enabled("other-rule"));
        assert!(config.is_rule_enabled("only-this"));
    }

    #[test]
    fn test_severity_override() {
        let mut config = Config::new();
        config.rules.severity.insert("rule1".to_string(), Severity::Error);

        assert_eq!(config.get_severity_override("rule1"), Some(Severity::Error));
        assert_eq!(config.get_severity_override("rule2"), None);
    }

    #[test]
    fn test_yaml_deserialize() {
        let yaml = r#"
engine:
  parallel: false
  jobs: 4
output:
  format: json
  verbose: true
rules:
  disabled:
    - rule1
    - rule2
"#;

        let config: Config = serde_yaml::from_str(yaml).unwrap();
        assert!(!config.engine.parallel);
        assert_eq!(config.engine.jobs, 4);
        assert_eq!(config.output.format, OutputFormat::Json);
        assert!(config.output.verbose);
        assert_eq!(config.rules.disabled.len(), 2);
    }

    #[test]
    fn test_extend_prefix() {
        let mut config = Config::new();

        // Without extend, all rules are enabled
        assert!(config.is_rule_enabled("WIX001"));
        assert!(config.is_rule_enabled("XML001"));

        // With extend prefix, only matching rules are enabled
        config.add_extend_prefixes(vec!["WIX".to_string()]);
        assert!(config.is_rule_enabled("WIX001"));
        assert!(config.is_rule_enabled("wix002")); // Case insensitive
        assert!(!config.is_rule_enabled("XML001"));
        assert!(!config.is_rule_enabled("other-rule"));
    }

    #[test]
    fn test_ignore_prefix() {
        let mut config = Config::new();

        // Without ignore, all rules are enabled
        assert!(config.is_rule_enabled("WIX001"));
        assert!(config.is_rule_enabled("XML001"));

        // With ignore prefix, matching rules are disabled
        config.add_ignore_prefixes(vec!["WIX".to_string()]);
        assert!(!config.is_rule_enabled("WIX001"));
        assert!(!config.is_rule_enabled("wix002")); // Case insensitive
        assert!(config.is_rule_enabled("XML001"));
        assert!(config.is_rule_enabled("other-rule"));
    }

    #[test]
    fn test_extend_and_ignore_combined() {
        let mut config = Config::new();

        // Extend WIX, ignore WIX-deprec
        config.add_extend_prefixes(vec!["WIX".to_string()]);
        config.add_ignore_prefixes(vec!["WIX-deprec".to_string()]);

        assert!(config.is_rule_enabled("WIX001"));
        assert!(config.is_rule_enabled("WIX-style001"));
        assert!(!config.is_rule_enabled("WIX-deprecated001")); // Ignored by prefix
        assert!(!config.is_rule_enabled("XML001")); // Not in extend list
    }

    #[test]
    fn test_multiple_prefixes() {
        let mut config = Config::new();

        config.add_extend_prefixes(vec!["WIX".to_string(), "XML".to_string()]);
        assert!(config.is_rule_enabled("WIX001"));
        assert!(config.is_rule_enabled("XML001"));
        assert!(!config.is_rule_enabled("OTHER001"));
    }
}
