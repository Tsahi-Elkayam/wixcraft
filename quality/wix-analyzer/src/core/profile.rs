//! Quality Profiles - Predefined rule configurations
//!
//! Quality profiles are curated sets of rules optimized for different use cases:
//! - **default**: Balanced rules for most projects
//! - **strict**: Maximum coverage, all rules enabled
//! - **relaxed**: Minimal rules, only critical issues
//! - **security**: Security-focused rules only
//! - **ci**: Optimized for CI pipelines (fast, critical only)
//!
//! # Usage
//!
//! ```rust,ignore
//! use wix_analyzer::core::QualityProfile;
//!
//! let profile = QualityProfile::strict();
//! let config = profile.to_config();
//! ```

use serde::{Deserialize, Serialize};
use std::collections::HashSet;

use crate::config::{AnalyzerConfig, Config, MinSeverity, RulesConfig};

/// Predefined quality profile names
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProfileName {
    /// Balanced rules for most projects
    Default,
    /// Maximum coverage, all rules enabled
    Strict,
    /// Minimal rules, only critical issues
    Relaxed,
    /// Security-focused rules only
    Security,
    /// CI-optimized (fast, critical only)
    Ci,
    /// Custom profile (user-defined)
    Custom,
}

impl std::str::FromStr for ProfileName {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "default" => Ok(Self::Default),
            "strict" => Ok(Self::Strict),
            "relaxed" => Ok(Self::Relaxed),
            "security" => Ok(Self::Security),
            "ci" => Ok(Self::Ci),
            "custom" => Ok(Self::Custom),
            _ => Err(format!("Unknown profile: {}", s)),
        }
    }
}

impl std::fmt::Display for ProfileName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Default => write!(f, "default"),
            Self::Strict => write!(f, "strict"),
            Self::Relaxed => write!(f, "relaxed"),
            Self::Security => write!(f, "security"),
            Self::Ci => write!(f, "ci"),
            Self::Custom => write!(f, "custom"),
        }
    }
}

/// A quality profile configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityProfile {
    /// Profile name
    pub name: ProfileName,
    /// Human-readable description
    pub description: String,
    /// Enabled analyzers
    pub analyzers: AnalyzerConfig,
    /// Minimum severity to report
    pub min_severity: MinSeverity,
    /// Rules to enable (empty = all)
    pub enable_rules: HashSet<String>,
    /// Rules to disable
    pub disable_rules: HashSet<String>,
    /// Whether this profile inherits from another
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extends: Option<ProfileName>,
}

impl QualityProfile {
    /// Create the default quality profile
    pub fn default_profile() -> Self {
        Self {
            name: ProfileName::Default,
            description: "Balanced rules for most projects".to_string(),
            analyzers: AnalyzerConfig::default(),
            min_severity: MinSeverity::Info,
            enable_rules: HashSet::new(),
            disable_rules: HashSet::new(),
            extends: None,
        }
    }

    /// Create a strict profile (all rules, all analyzers)
    pub fn strict() -> Self {
        Self {
            name: ProfileName::Strict,
            description: "Maximum coverage with all rules enabled".to_string(),
            analyzers: AnalyzerConfig {
                validation: true,
                best_practices: true,
                security: true,
                dead_code: true,
            },
            min_severity: MinSeverity::Info,
            enable_rules: HashSet::new(),
            disable_rules: HashSet::new(),
            extends: None,
        }
    }

    /// Create a relaxed profile (critical issues only)
    pub fn relaxed() -> Self {
        Self {
            name: ProfileName::Relaxed,
            description: "Minimal rules for legacy or prototype projects".to_string(),
            analyzers: AnalyzerConfig {
                validation: true,
                best_practices: false,
                security: true,
                dead_code: false,
            },
            min_severity: MinSeverity::Error,
            enable_rules: HashSet::new(),
            disable_rules: Self::relaxed_disabled_rules(),
            extends: None,
        }
    }

    /// Create a security-focused profile
    pub fn security() -> Self {
        Self {
            name: ProfileName::Security,
            description: "Security-focused analysis only".to_string(),
            analyzers: AnalyzerConfig {
                validation: false,
                best_practices: false,
                security: true,
                dead_code: false,
            },
            min_severity: MinSeverity::Info,
            enable_rules: Self::security_rules(),
            disable_rules: HashSet::new(),
            extends: None,
        }
    }

    /// Create a CI-optimized profile (fast, critical only)
    pub fn ci() -> Self {
        Self {
            name: ProfileName::Ci,
            description: "Optimized for CI pipelines - fast checks, critical issues".to_string(),
            analyzers: AnalyzerConfig {
                validation: true,
                best_practices: false,
                security: true,
                dead_code: false,
            },
            min_severity: MinSeverity::Warning,
            enable_rules: HashSet::new(),
            disable_rules: Self::ci_disabled_rules(),
            extends: None,
        }
    }

    /// Get a profile by name
    pub fn by_name(name: ProfileName) -> Self {
        match name {
            ProfileName::Default => Self::default_profile(),
            ProfileName::Strict => Self::strict(),
            ProfileName::Relaxed => Self::relaxed(),
            ProfileName::Security => Self::security(),
            ProfileName::Ci => Self::ci(),
            ProfileName::Custom => Self::default_profile(),
        }
    }

    /// Convert profile to a Config
    pub fn to_config(&self) -> Config {
        let mut config = Config::default();
        config.analyzers = self.analyzers.clone();
        config.min_severity = self.min_severity;
        config.rules = RulesConfig {
            enable: self.enable_rules.iter().cloned().collect(),
            disable: self.disable_rules.iter().cloned().collect(),
            severity: std::collections::HashMap::new(),
        };
        config
    }

    /// Create a custom profile from a config
    pub fn from_config(config: &Config) -> Self {
        Self {
            name: ProfileName::Custom,
            description: "Custom profile from configuration".to_string(),
            analyzers: config.analyzers.clone(),
            min_severity: config.min_severity,
            enable_rules: config.rules.enable.iter().cloned().collect(),
            disable_rules: config.rules.disable.iter().cloned().collect(),
            extends: None,
        }
    }

    /// Extend this profile with another
    pub fn extend(&mut self, base: &QualityProfile) {
        // Inherit analyzer settings
        self.analyzers.validation |= base.analyzers.validation;
        self.analyzers.best_practices |= base.analyzers.best_practices;
        self.analyzers.security |= base.analyzers.security;
        self.analyzers.dead_code |= base.analyzers.dead_code;

        // Inherit enable rules
        self.enable_rules.extend(base.enable_rules.iter().cloned());

        // Inherit disable rules (but our explicit enables take precedence)
        for rule in &base.disable_rules {
            if !self.enable_rules.contains(rule) {
                self.disable_rules.insert(rule.clone());
            }
        }

        self.extends = Some(base.name);
    }

    /// Check if a rule is enabled in this profile
    pub fn is_rule_enabled(&self, rule_id: &str) -> bool {
        // Check explicit disable first
        if self.matches_pattern(rule_id, &self.disable_rules) {
            return false;
        }

        // If enable list is empty, all rules are enabled
        if self.enable_rules.is_empty() {
            return true;
        }

        // Check enable patterns
        self.matches_pattern(rule_id, &self.enable_rules)
    }

    fn matches_pattern(&self, rule_id: &str, patterns: &HashSet<String>) -> bool {
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

    // === Helper functions for predefined rule sets ===

    fn relaxed_disabled_rules() -> HashSet<String> {
        [
            "BP-*",       // All best practice rules
            "DEAD-*",     // All dead code rules
            "VAL-INFO-*", // Info-level validation
        ]
        .iter()
        .map(|s| s.to_string())
        .collect()
    }

    fn security_rules() -> HashSet<String> {
        [
            "SEC-*", // All security rules
        ]
        .iter()
        .map(|s| s.to_string())
        .collect()
    }

    fn ci_disabled_rules() -> HashSet<String> {
        [
            "BP-MAINT-*", // Maintainability suggestions
            "BP-CONV-*",  // Convention suggestions
            "DEAD-*",     // Dead code (slower analysis)
        ]
        .iter()
        .map(|s| s.to_string())
        .collect()
    }
}

impl Default for QualityProfile {
    fn default() -> Self {
        Self::default_profile()
    }
}

/// List all available profile names
pub fn available_profiles() -> Vec<ProfileName> {
    vec![
        ProfileName::Default,
        ProfileName::Strict,
        ProfileName::Relaxed,
        ProfileName::Security,
        ProfileName::Ci,
    ]
}

/// Get profile descriptions for help text
pub fn profile_descriptions() -> Vec<(&'static str, &'static str)> {
    vec![
        ("default", "Balanced rules for most projects"),
        ("strict", "Maximum coverage with all rules enabled"),
        ("relaxed", "Minimal rules for legacy or prototype projects"),
        ("security", "Security-focused analysis only"),
        (
            "ci",
            "Optimized for CI pipelines - fast checks, critical issues",
        ),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_profile_name_from_str() {
        assert_eq!(
            "default".parse::<ProfileName>().unwrap(),
            ProfileName::Default
        );
        assert_eq!(
            "strict".parse::<ProfileName>().unwrap(),
            ProfileName::Strict
        );
        assert_eq!(
            "relaxed".parse::<ProfileName>().unwrap(),
            ProfileName::Relaxed
        );
        assert_eq!(
            "security".parse::<ProfileName>().unwrap(),
            ProfileName::Security
        );
        assert_eq!("ci".parse::<ProfileName>().unwrap(), ProfileName::Ci);
        assert!("unknown".parse::<ProfileName>().is_err());
    }

    #[test]
    fn test_profile_name_display() {
        assert_eq!(ProfileName::Default.to_string(), "default");
        assert_eq!(ProfileName::Strict.to_string(), "strict");
    }

    #[test]
    fn test_default_profile() {
        let profile = QualityProfile::default_profile();
        assert_eq!(profile.name, ProfileName::Default);
        assert!(profile.analyzers.validation);
        assert!(profile.analyzers.best_practices);
        assert!(profile.analyzers.security);
        assert!(profile.analyzers.dead_code);
    }

    #[test]
    fn test_strict_profile() {
        let profile = QualityProfile::strict();
        assert_eq!(profile.name, ProfileName::Strict);
        assert_eq!(profile.min_severity, MinSeverity::Info);
        assert!(profile.disable_rules.is_empty());
    }

    #[test]
    fn test_relaxed_profile() {
        let profile = QualityProfile::relaxed();
        assert_eq!(profile.name, ProfileName::Relaxed);
        assert!(!profile.analyzers.best_practices);
        assert!(!profile.analyzers.dead_code);
        assert_eq!(profile.min_severity, MinSeverity::Error);
        assert!(!profile.disable_rules.is_empty());
    }

    #[test]
    fn test_security_profile() {
        let profile = QualityProfile::security();
        assert_eq!(profile.name, ProfileName::Security);
        assert!(!profile.analyzers.validation);
        assert!(!profile.analyzers.best_practices);
        assert!(profile.analyzers.security);
        assert!(!profile.analyzers.dead_code);
        assert!(profile.enable_rules.contains("SEC-*"));
    }

    #[test]
    fn test_ci_profile() {
        let profile = QualityProfile::ci();
        assert_eq!(profile.name, ProfileName::Ci);
        assert!(profile.analyzers.validation);
        assert!(!profile.analyzers.best_practices);
        assert!(profile.analyzers.security);
        assert!(!profile.analyzers.dead_code);
        assert_eq!(profile.min_severity, MinSeverity::Warning);
    }

    #[test]
    fn test_by_name() {
        assert_eq!(
            QualityProfile::by_name(ProfileName::Strict).name,
            ProfileName::Strict
        );
        assert_eq!(
            QualityProfile::by_name(ProfileName::Relaxed).name,
            ProfileName::Relaxed
        );
    }

    #[test]
    fn test_to_config() {
        let profile = QualityProfile::security();
        let config = profile.to_config();

        assert!(!config.analyzers.validation);
        assert!(config.analyzers.security);
    }

    #[test]
    fn test_from_config() {
        let mut config = Config::default();
        config.analyzers.validation = false;
        config.rules.disable.push("SEC-001".to_string());

        let profile = QualityProfile::from_config(&config);
        assert_eq!(profile.name, ProfileName::Custom);
        assert!(!profile.analyzers.validation);
        assert!(profile.disable_rules.contains("SEC-001"));
    }

    #[test]
    fn test_extend() {
        let base = QualityProfile::strict();
        let mut custom = QualityProfile {
            name: ProfileName::Custom,
            description: "My custom".to_string(),
            analyzers: AnalyzerConfig {
                validation: true,
                best_practices: false,
                security: false,
                dead_code: false,
            },
            min_severity: MinSeverity::Warning,
            enable_rules: HashSet::new(),
            disable_rules: HashSet::new(),
            extends: None,
        };

        custom.extend(&base);

        // Should inherit enabled analyzers
        assert!(custom.analyzers.security);
        assert!(custom.analyzers.dead_code);
        assert!(custom.analyzers.best_practices);
        assert_eq!(custom.extends, Some(ProfileName::Strict));
    }

    #[test]
    fn test_is_rule_enabled() {
        let profile = QualityProfile::relaxed();

        // BP-* is disabled in relaxed
        assert!(!profile.is_rule_enabled("BP-001"));
        assert!(!profile.is_rule_enabled("BP-MAINT-001"));

        // SEC-* is not disabled
        assert!(profile.is_rule_enabled("SEC-001"));
    }

    #[test]
    fn test_is_rule_enabled_with_enable_list() {
        let profile = QualityProfile::security();

        // Only SEC-* is enabled
        assert!(profile.is_rule_enabled("SEC-001"));
        assert!(profile.is_rule_enabled("SEC-007"));
        assert!(!profile.is_rule_enabled("BP-001"));
        assert!(!profile.is_rule_enabled("VAL-001"));
    }

    #[test]
    fn test_available_profiles() {
        let profiles = available_profiles();
        assert_eq!(profiles.len(), 5);
        assert!(profiles.contains(&ProfileName::Default));
        assert!(profiles.contains(&ProfileName::Strict));
    }

    #[test]
    fn test_profile_descriptions() {
        let descs = profile_descriptions();
        assert_eq!(descs.len(), 5);
        assert!(descs.iter().any(|(name, _)| *name == "default"));
    }

    #[test]
    fn test_profile_default_trait() {
        let profile: QualityProfile = Default::default();
        assert_eq!(profile.name, ProfileName::Default);
    }
}
