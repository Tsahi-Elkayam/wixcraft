//! Configuration handling for wix-lint

use crate::Severity;
use globset::{Glob, GlobSet, GlobSetBuilder};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Failed to read config file: {0}")]
    ReadFile(#[from] std::io::Error),
    #[error("Failed to parse JSON config: {0}")]
    ParseJson(#[from] serde_json::Error),
    #[error("Failed to parse YAML config: {0}")]
    ParseYaml(#[from] serde_yaml::Error),
    #[error("Invalid glob pattern: {0}")]
    InvalidGlob(#[from] globset::Error),
}

/// Runtime lint configuration
#[derive(Debug, Clone)]
pub struct Config {
    /// Only run these rules (if Some)
    pub enabled_rules: Option<Vec<String>>,
    /// Skip these rules
    pub disabled_rules: Vec<String>,
    /// Minimum severity to report
    pub min_severity: Severity,
    /// Verbose output
    pub verbose: bool,
    /// Show statistics at the end
    pub statistics: bool,
    /// File patterns to exclude
    pub exclude_patterns: GlobSet,
    /// Filename patterns to include (if set, only lint matching files)
    pub filename_patterns: Option<GlobSet>,
    /// Per-file rule overrides (file pattern -> disabled rules)
    pub per_file_ignores: HashMap<String, Vec<String>>,
    /// Severity overrides per rule
    pub severity_overrides: HashMap<String, Severity>,
    /// Max errors before stopping (0 = unlimited)
    pub max_errors: usize,
    /// Number of parallel jobs (0 = auto)
    pub jobs: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            enabled_rules: None,
            disabled_rules: Vec::new(),
            min_severity: Severity::Info,
            verbose: false,
            statistics: false,
            exclude_patterns: GlobSet::empty(),
            filename_patterns: None,
            per_file_ignores: HashMap::new(),
            severity_overrides: HashMap::new(),
            max_errors: 0,
            jobs: 0,
        }
    }
}

/// CLI options to merge into config
#[derive(Debug, Default)]
pub struct CliOptions {
    /// Rules to enable (replaces config if set)
    pub enabled_rules: Option<Vec<String>>,
    /// Rules to disable (adds to config)
    pub disabled_rules: Vec<String>,
    /// Additional rules to ignore
    pub extend_ignore: Vec<String>,
    /// Minimum severity level
    pub min_severity: Option<Severity>,
    /// Verbose output
    pub verbose: bool,
    /// Show statistics
    pub statistics: bool,
    /// Maximum errors before stopping
    pub max_errors: Option<usize>,
    /// Number of parallel jobs
    pub jobs: Option<usize>,
}

/// Configuration file format (.wixlintrc.json or .wixlintrc.yaml)
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ConfigFile {
    /// Rules to enable (if specified, only these run)
    #[serde(default)]
    pub select: Vec<String>,

    /// Rules to ignore/disable
    #[serde(default)]
    pub ignore: Vec<String>,

    /// Additional rules to ignore (added to ignore, not replacing)
    #[serde(default)]
    pub extend_ignore: Vec<String>,

    /// Minimum severity: "error", "warning", or "info"
    #[serde(default)]
    pub min_severity: Option<String>,

    /// File/folder patterns to exclude
    #[serde(default)]
    pub exclude: Vec<String>,

    /// Filename patterns to include (only lint matching files)
    #[serde(default)]
    pub filename: Vec<String>,

    /// Per-file rule ignores: {"*.generated.wxs": ["rule1", "rule2"]}
    #[serde(default)]
    pub per_file_ignores: HashMap<String, Vec<String>>,

    /// Severity overrides: {"rule-id": "warning"}
    #[serde(default)]
    pub severity: HashMap<String, String>,

    /// Max errors before stopping (0 = unlimited)
    #[serde(default)]
    pub max_errors: usize,

    /// Number of parallel jobs (0 = auto)
    #[serde(default)]
    pub jobs: usize,
}

impl Config {
    /// Load configuration from a file
    pub fn from_file(path: &Path) -> Result<Self, ConfigError> {
        let content = fs::read_to_string(path)?;
        let config_file: ConfigFile = if path.extension().is_some_and(|e| e == "yaml" || e == "yml")
        {
            serde_yaml::from_str(&content)?
        } else {
            serde_json::from_str(&content)?
        };

        Self::from_config_file(config_file)
    }

    /// Try to find and load config from standard locations
    pub fn find_and_load(start_dir: &Path) -> Result<Option<(PathBuf, Self)>, ConfigError> {
        let config_names = [
            ".wixlintrc.json",
            ".wixlintrc.yaml",
            ".wixlintrc.yml",
            ".wixlintrc",
            "wixlint.json",
            "wixlint.yaml",
        ];

        let mut current = start_dir.to_path_buf();
        loop {
            for name in &config_names {
                let config_path = current.join(name);
                if config_path.exists() {
                    let config = Self::from_file(&config_path)?;
                    return Ok(Some((config_path, config)));
                }
            }

            // Also check pyproject.toml style: look for [tool.wixlint] section
            // (not implemented for now, but could be added)

            if !current.pop() {
                break;
            }
        }

        Ok(None)
    }

    /// Build config from a ConfigFile
    fn from_config_file(file: ConfigFile) -> Result<Self, ConfigError> {
        // Build exclude glob set
        let mut exclude_builder = GlobSetBuilder::new();
        for pattern in &file.exclude {
            exclude_builder.add(Glob::new(pattern)?);
        }
        let exclude_patterns = exclude_builder.build()?;

        // Build filename glob set (if specified)
        let filename_patterns = if file.filename.is_empty() {
            None
        } else {
            let mut filename_builder = GlobSetBuilder::new();
            for pattern in &file.filename {
                filename_builder.add(Glob::new(pattern)?);
            }
            Some(filename_builder.build()?)
        };

        // Parse severity overrides
        let mut severity_overrides = HashMap::new();
        for (rule, sev) in &file.severity {
            severity_overrides.insert(rule.clone(), sev.parse().unwrap_or_default());
        }

        // Parse min severity
        let min_severity = file
            .min_severity
            .as_ref()
            .and_then(|s| s.parse().ok())
            .unwrap_or(Severity::Info);

        // Combine ignore and extend_ignore
        let mut disabled_rules = file.ignore;
        disabled_rules.extend(file.extend_ignore);

        Ok(Self {
            enabled_rules: if file.select.is_empty() {
                None
            } else {
                Some(file.select)
            },
            disabled_rules,
            min_severity,
            verbose: false,
            statistics: false,
            exclude_patterns,
            filename_patterns,
            per_file_ignores: file.per_file_ignores,
            severity_overrides,
            max_errors: file.max_errors,
            jobs: file.jobs,
        })
    }

    /// Merge CLI options into this config (CLI takes precedence)
    pub fn merge_cli(&mut self, opts: CliOptions) {
        // CLI enabled rules override config
        if opts.enabled_rules.is_some() {
            self.enabled_rules = opts.enabled_rules;
        }

        // CLI disabled rules add to config
        self.disabled_rules.extend(opts.disabled_rules);

        // Extend ignore adds to disabled rules
        self.disabled_rules.extend(opts.extend_ignore);

        // CLI severity overrides config
        if let Some(sev) = opts.min_severity {
            self.min_severity = sev;
        }

        self.verbose = opts.verbose;
        self.statistics = opts.statistics;

        if let Some(max) = opts.max_errors {
            self.max_errors = max;
        }

        if let Some(j) = opts.jobs {
            self.jobs = j;
        }
    }

    /// Check if a file matches the filename pattern filter
    pub fn matches_filename_pattern(&self, file_path: &Path) -> bool {
        match &self.filename_patterns {
            Some(patterns) => patterns.is_match(file_path),
            None => true, // No filter = all files match
        }
    }

    /// Check if a rule is enabled
    pub fn is_rule_enabled(&self, rule_id: &str) -> bool {
        // Check if explicitly disabled
        if self.disabled_rules.iter().any(|r| r == rule_id) {
            return false;
        }

        // If we have an explicit enable list, check it
        if let Some(ref enabled) = self.enabled_rules {
            return enabled.iter().any(|r| r == rule_id);
        }

        // Default: enabled
        true
    }

    /// Check if a rule is enabled for a specific file
    pub fn is_rule_enabled_for_file(&self, rule_id: &str, file_path: &Path) -> bool {
        // First check global enable/disable
        if !self.is_rule_enabled(rule_id) {
            return false;
        }

        // Check per-file ignores
        let file_str = file_path.to_string_lossy();
        for (pattern, ignored_rules) in &self.per_file_ignores {
            if let Ok(glob) = Glob::new(pattern) {
                if glob.compile_matcher().is_match(file_str.as_ref())
                    && ignored_rules.iter().any(|r| r == rule_id) {
                        return false;
                    }
            }
        }

        true
    }

    /// Check if a file should be excluded
    pub fn is_file_excluded(&self, file_path: &Path) -> bool {
        self.exclude_patterns.is_match(file_path)
    }

    /// Get effective severity for a rule (considering overrides)
    pub fn get_severity(&self, rule_id: &str, default: Severity) -> Severity {
        self.severity_overrides
            .get(rule_id)
            .copied()
            .unwrap_or(default)
    }

    /// Check if a severity should be reported
    pub fn should_report(&self, severity: Severity) -> bool {
        severity >= self.min_severity
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_json_config() {
        let json = r#"{
            "select": ["package-requires-upgradecode"],
            "ignore": ["component-one-file-recommended"],
            "minSeverity": "warning",
            "exclude": ["**/generated/**", "*.g.wxs"],
            "perFileIgnores": {
                "tests/*.wxs": ["package-requires-manufacturer"]
            },
            "severity": {
                "component-requires-guid": "error"
            }
        }"#;

        let config_file: ConfigFile = serde_json::from_str(json).unwrap();
        let config = Config::from_config_file(config_file).unwrap();

        assert_eq!(config.enabled_rules, Some(vec!["package-requires-upgradecode".to_string()]));
        assert_eq!(config.disabled_rules, vec!["component-one-file-recommended".to_string()]);
        assert_eq!(config.min_severity, Severity::Warning);
        assert!(config.severity_overrides.contains_key("component-requires-guid"));
    }

    #[test]
    fn test_rule_enabled() {
        let mut config = Config::default();
        config.disabled_rules = vec!["disabled-rule".to_string()];

        assert!(config.is_rule_enabled("some-rule"));
        assert!(!config.is_rule_enabled("disabled-rule"));
    }

    #[test]
    fn test_config_default() {
        let config = Config::default();
        assert!(config.enabled_rules.is_none());
        assert!(config.disabled_rules.is_empty());
        assert_eq!(config.min_severity, Severity::Info);
        assert!(!config.verbose);
        assert!(!config.statistics);
        assert_eq!(config.max_errors, 0);
        assert_eq!(config.jobs, 0);
    }

    #[test]
    fn test_rule_enabled_with_select() {
        let mut config = Config::default();
        config.enabled_rules = Some(vec!["allowed-rule".to_string()]);

        assert!(config.is_rule_enabled("allowed-rule"));
        assert!(!config.is_rule_enabled("other-rule"));
    }

    #[test]
    fn test_rule_enabled_for_file() {
        let mut config = Config::default();
        config.per_file_ignores.insert(
            "tests/*.wxs".to_string(),
            vec!["test-rule".to_string()],
        );

        // Rule should be enabled for non-matching files
        assert!(config.is_rule_enabled_for_file("test-rule", Path::new("src/main.wxs")));

        // Rule should be disabled for matching files
        assert!(!config.is_rule_enabled_for_file("test-rule", Path::new("tests/foo.wxs")));

        // Other rules should still be enabled
        assert!(config.is_rule_enabled_for_file("other-rule", Path::new("tests/foo.wxs")));
    }

    #[test]
    fn test_is_file_excluded() {
        let json = r#"{
            "exclude": ["**/generated/**", "*.g.wxs"]
        }"#;
        let config_file: ConfigFile = serde_json::from_str(json).unwrap();
        let config = Config::from_config_file(config_file).unwrap();

        assert!(config.is_file_excluded(Path::new("src/generated/file.wxs")));
        assert!(config.is_file_excluded(Path::new("Product.g.wxs")));
        assert!(!config.is_file_excluded(Path::new("Product.wxs")));
    }

    #[test]
    fn test_matches_filename_pattern() {
        let json = r#"{
            "filename": ["*.wxs", "*.wxi"]
        }"#;
        let config_file: ConfigFile = serde_json::from_str(json).unwrap();
        let config = Config::from_config_file(config_file).unwrap();

        assert!(config.matches_filename_pattern(Path::new("Product.wxs")));
        assert!(config.matches_filename_pattern(Path::new("Include.wxi")));
        assert!(!config.matches_filename_pattern(Path::new("README.md")));
    }

    #[test]
    fn test_matches_filename_pattern_default() {
        let config = Config::default();
        // Without filename filter, all files should match
        assert!(config.matches_filename_pattern(Path::new("anything.txt")));
    }

    #[test]
    fn test_get_severity_with_override() {
        let mut config = Config::default();
        config.severity_overrides.insert(
            "test-rule".to_string(),
            Severity::Error,
        );

        assert_eq!(config.get_severity("test-rule", Severity::Warning), Severity::Error);
        assert_eq!(config.get_severity("other-rule", Severity::Warning), Severity::Warning);
    }

    #[test]
    fn test_should_report() {
        let mut config = Config::default();

        // Default min severity is Info, so all should report
        assert!(config.should_report(Severity::Error));
        assert!(config.should_report(Severity::Warning));
        assert!(config.should_report(Severity::Info));

        // With Warning min severity
        config.min_severity = Severity::Warning;
        assert!(config.should_report(Severity::Error));
        assert!(config.should_report(Severity::Warning));
        assert!(!config.should_report(Severity::Info));

        // With Error min severity
        config.min_severity = Severity::Error;
        assert!(config.should_report(Severity::Error));
        assert!(!config.should_report(Severity::Warning));
        assert!(!config.should_report(Severity::Info));
    }

    #[test]
    fn test_merge_cli() {
        let mut config = Config::default();
        config.disabled_rules = vec!["existing-rule".to_string()];
        config.min_severity = Severity::Info;

        config.merge_cli(CliOptions {
            enabled_rules: Some(vec!["cli-enabled".to_string()]),
            disabled_rules: vec!["cli-disabled".to_string()],
            extend_ignore: vec!["cli-extend-ignore".to_string()],
            min_severity: Some(Severity::Error),
            verbose: true,
            statistics: true,
            max_errors: Some(10),
            jobs: Some(4),
        });

        assert_eq!(config.enabled_rules, Some(vec!["cli-enabled".to_string()]));
        assert!(config.disabled_rules.contains(&"existing-rule".to_string()));
        assert!(config.disabled_rules.contains(&"cli-disabled".to_string()));
        assert!(config.disabled_rules.contains(&"cli-extend-ignore".to_string()));
        assert_eq!(config.min_severity, Severity::Error);
        assert!(config.verbose);
        assert!(config.statistics);
        assert_eq!(config.max_errors, 10);
        assert_eq!(config.jobs, 4);
    }

    #[test]
    fn test_merge_cli_partial() {
        let mut config = Config::default();
        config.enabled_rules = Some(vec!["original-rule".to_string()]);
        config.max_errors = 5;
        config.jobs = 2;

        // Merge with None/default values - should keep original
        config.merge_cli(CliOptions::default());

        assert_eq!(config.enabled_rules, Some(vec!["original-rule".to_string()]));
        assert_eq!(config.max_errors, 5);
        assert_eq!(config.jobs, 2);
    }

    #[test]
    fn test_extend_ignore_combines() {
        let json = r#"{
            "ignore": ["rule1"],
            "extendIgnore": ["rule2", "rule3"]
        }"#;
        let config_file: ConfigFile = serde_json::from_str(json).unwrap();
        let config = Config::from_config_file(config_file).unwrap();

        assert!(config.disabled_rules.contains(&"rule1".to_string()));
        assert!(config.disabled_rules.contains(&"rule2".to_string()));
        assert!(config.disabled_rules.contains(&"rule3".to_string()));
    }

    #[test]
    fn test_config_file_default() {
        let config_file = ConfigFile::default();
        assert!(config_file.select.is_empty());
        assert!(config_file.ignore.is_empty());
        assert!(config_file.extend_ignore.is_empty());
        assert!(config_file.min_severity.is_none());
        assert!(config_file.exclude.is_empty());
        assert!(config_file.filename.is_empty());
        assert!(config_file.per_file_ignores.is_empty());
        assert!(config_file.severity.is_empty());
        assert_eq!(config_file.max_errors, 0);
        assert_eq!(config_file.jobs, 0);
    }

    #[test]
    fn test_parse_yaml_style_config() {
        // Test that camelCase fields work
        let json = r#"{
            "minSeverity": "error",
            "maxErrors": 100,
            "perFileIgnores": {}
        }"#;
        let config_file: ConfigFile = serde_json::from_str(json).unwrap();

        assert_eq!(config_file.min_severity, Some("error".to_string()));
        assert_eq!(config_file.max_errors, 100);
    }

    #[test]
    fn test_severity_override_parsing() {
        let json = r#"{
            "severity": {
                "rule-a": "error",
                "rule-b": "warning",
                "rule-c": "info"
            }
        }"#;
        let config_file: ConfigFile = serde_json::from_str(json).unwrap();
        let config = Config::from_config_file(config_file).unwrap();

        assert_eq!(config.severity_overrides.get("rule-a"), Some(&Severity::Error));
        assert_eq!(config.severity_overrides.get("rule-b"), Some(&Severity::Warning));
        assert_eq!(config.severity_overrides.get("rule-c"), Some(&Severity::Info));
    }

    #[test]
    fn test_empty_select_means_all_enabled() {
        let json = r#"{
            "select": []
        }"#;
        let config_file: ConfigFile = serde_json::from_str(json).unwrap();
        let config = Config::from_config_file(config_file).unwrap();

        // Empty select should mean enabled_rules is None (all rules enabled)
        assert!(config.enabled_rules.is_none());
        assert!(config.is_rule_enabled("any-rule"));
    }

    #[test]
    fn test_rule_disabled_globally_not_enabled_for_file() {
        let mut config = Config::default();
        config.disabled_rules = vec!["global-disabled".to_string()];

        // Globally disabled rules should not be enabled for any file
        assert!(!config.is_rule_enabled_for_file("global-disabled", Path::new("any/file.wxs")));
    }
}
