//! Rule definition and evaluation

use crate::diagnostic::Severity;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fmt;

/// Rule category for grouping related rules
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum RuleCategory {
    /// Code that is definitely wrong or useless
    Correctness,
    /// Code that is likely wrong or suspicious
    Suspicious,
    /// Idiomatic and consistent style rules
    #[default]
    Style,
    /// Rules that improve runtime/build performance
    Perf,
    /// Extra strict rules that may have false positives
    Pedantic,
    /// Rules that ban specific patterns or features
    Restriction,
    /// Rules under development (may change or be removed)
    Nursery,
}

impl fmt::Display for RuleCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RuleCategory::Correctness => write!(f, "correctness"),
            RuleCategory::Suspicious => write!(f, "suspicious"),
            RuleCategory::Style => write!(f, "style"),
            RuleCategory::Perf => write!(f, "perf"),
            RuleCategory::Pedantic => write!(f, "pedantic"),
            RuleCategory::Restriction => write!(f, "restriction"),
            RuleCategory::Nursery => write!(f, "nursery"),
        }
    }
}

impl std::str::FromStr for RuleCategory {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "correctness" => Ok(RuleCategory::Correctness),
            "suspicious" => Ok(RuleCategory::Suspicious),
            "style" => Ok(RuleCategory::Style),
            "perf" | "performance" => Ok(RuleCategory::Perf),
            "pedantic" => Ok(RuleCategory::Pedantic),
            "restriction" => Ok(RuleCategory::Restriction),
            "nursery" | "experimental" => Ok(RuleCategory::Nursery),
            _ => Err(format!("Unknown category: {}", s)),
        }
    }
}

/// Rule stability level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum RuleStability {
    /// Rule is stable and recommended for use
    #[default]
    Stable,
    /// Rule is in preview/experimental stage
    Preview,
    /// Rule is deprecated and will be removed
    Deprecated,
}

impl fmt::Display for RuleStability {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RuleStability::Stable => write!(f, "stable"),
            RuleStability::Preview => write!(f, "preview"),
            RuleStability::Deprecated => write!(f, "deprecated"),
        }
    }
}

/// Target specification for a rule
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Target {
    /// Node kind (e.g., "element", "attribute")
    #[serde(default)]
    pub kind: Option<String>,

    /// Node name pattern (e.g., "Package", "Component*")
    #[serde(default)]
    pub name: Option<String>,

    /// Parent element name (for context-sensitive rules)
    #[serde(default)]
    pub parent: Option<String>,
}

/// Suggested fix for a rule violation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FixSuggestion {
    /// Type of fix action
    pub action: FixAction,

    /// Attribute name (for attribute-related fixes)
    #[serde(default)]
    pub attribute: Option<String>,

    /// Value to set (can contain placeholders like "{generate-guid}")
    #[serde(default)]
    pub value: Option<String>,

    /// Description of the fix
    #[serde(default)]
    pub description: Option<String>,
}

/// Types of fix actions
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FixAction {
    /// Add an attribute
    AddAttribute,
    /// Remove an attribute
    RemoveAttribute,
    /// Set/change an attribute value
    SetAttribute,
    /// Replace the entire element
    ReplaceElement,
    /// Remove the element
    RemoveElement,
    /// Rename the element
    RenameElement,
    /// Custom fix (message only)
    Custom,
}

/// A lint rule definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rule {
    /// Unique rule identifier (e.g., "package-requires-upgradecode")
    pub id: String,

    /// Human-readable name
    #[serde(default)]
    pub name: Option<String>,

    /// Detailed description
    #[serde(default)]
    pub description: Option<String>,

    /// Default severity level
    #[serde(default)]
    pub severity: Severity,

    /// Rule category (correctness, style, perf, etc.)
    #[serde(default)]
    pub category: RuleCategory,

    /// Rule stability (stable, preview, deprecated)
    #[serde(default)]
    pub stability: RuleStability,

    /// Target specification (what nodes this rule applies to)
    #[serde(default)]
    pub target: Target,

    /// Condition expression that triggers the rule
    pub condition: String,

    /// Error message template (can contain placeholders)
    pub message: String,

    /// Suggested fix (optional)
    #[serde(default)]
    pub fix: Option<FixSuggestion>,

    /// Documentation URL
    #[serde(default)]
    pub docs: Option<String>,

    /// Tags for categorization
    #[serde(default)]
    pub tags: Vec<String>,

    /// Whether this rule is enabled by default
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// WiX version this rule applies to (None = all)
    #[serde(default)]
    pub wix_version: Option<u8>,

    /// Language context this rule applies to (e.g., "xml", "shell", "powershell")
    /// If empty/None, applies to the main document language.
    /// Used for linting embedded code blocks within documents.
    #[serde(default)]
    pub context: Vec<String>,

    /// Rationale explaining why this rule exists
    #[serde(default)]
    pub rationale: Option<String>,

    /// Example of code that violates this rule
    #[serde(default)]
    pub example_bad: Option<String>,

    /// Example of correct code
    #[serde(default)]
    pub example_good: Option<String>,

    /// Related rule IDs
    #[serde(default)]
    pub related: Vec<String>,

    /// Version when this rule was deprecated (e.g., "0.2.0")
    #[serde(default)]
    pub deprecated_since: Option<String>,

    /// Replacement rule ID when deprecated
    #[serde(default)]
    pub replacement: Option<String>,

    /// Deprecation message explaining migration
    #[serde(default)]
    pub deprecation_message: Option<String>,
}

fn default_true() -> bool {
    true
}

impl Rule {
    /// Create a new rule with minimal required fields
    pub fn new(id: &str, condition: &str, message: &str) -> Self {
        Self {
            id: id.to_string(),
            name: None,
            description: None,
            severity: Severity::Warning,
            category: RuleCategory::default(),
            stability: RuleStability::default(),
            target: Target::default(),
            condition: condition.to_string(),
            message: message.to_string(),
            fix: None,
            docs: None,
            tags: Vec::new(),
            enabled: true,
            wix_version: None,
            context: Vec::new(),
            rationale: None,
            example_bad: None,
            example_good: None,
            related: Vec::new(),
            deprecated_since: None,
            replacement: None,
            deprecation_message: None,
        }
    }

    /// Get the deprecation warning message
    pub fn deprecation_warning(&self) -> Option<String> {
        if !self.is_deprecated() {
            return None;
        }

        let mut msg = format!("Rule '{}' is deprecated", self.id);

        if let Some(since) = &self.deprecated_since {
            msg.push_str(&format!(" since {}", since));
        }

        if let Some(replacement) = &self.replacement {
            msg.push_str(&format!(". Use '{}' instead", replacement));
        }

        if let Some(custom_msg) = &self.deprecation_message {
            msg.push_str(&format!(". {}", custom_msg));
        }

        Some(msg)
    }

    /// Mark this rule as deprecated
    pub fn deprecate(mut self, since: Option<&str>, replacement: Option<&str>) -> Self {
        self.stability = RuleStability::Deprecated;
        self.deprecated_since = since.map(String::from);
        self.replacement = replacement.map(String::from);
        self
    }

    /// Set a custom deprecation message
    pub fn with_deprecation_message(mut self, message: &str) -> Self {
        self.deprecation_message = Some(message.to_string());
        self
    }

    /// Set the rule category
    pub fn with_category(mut self, category: RuleCategory) -> Self {
        self.category = category;
        self
    }

    /// Set the rule stability
    pub fn with_stability(mut self, stability: RuleStability) -> Self {
        self.stability = stability;
        self
    }

    /// Mark rule as preview/experimental
    pub fn preview(mut self) -> Self {
        self.stability = RuleStability::Preview;
        self
    }

    /// Mark rule as deprecated
    pub fn deprecated(mut self) -> Self {
        self.stability = RuleStability::Deprecated;
        self
    }

    /// Check if rule is preview/experimental
    pub fn is_preview(&self) -> bool {
        self.stability == RuleStability::Preview
    }

    /// Check if rule is deprecated
    pub fn is_deprecated(&self) -> bool {
        self.stability == RuleStability::Deprecated
    }

    /// Check if rule is stable
    pub fn is_stable(&self) -> bool {
        self.stability == RuleStability::Stable
    }

    /// Set the rationale
    pub fn with_rationale(mut self, rationale: &str) -> Self {
        self.rationale = Some(rationale.to_string());
        self
    }

    /// Set bad example
    pub fn with_example_bad(mut self, example: &str) -> Self {
        self.example_bad = Some(example.to_string());
        self
    }

    /// Set good example
    pub fn with_example_good(mut self, example: &str) -> Self {
        self.example_good = Some(example.to_string());
        self
    }

    /// Add a related rule
    pub fn with_related(mut self, rule_id: &str) -> Self {
        self.related.push(rule_id.to_string());
        self
    }

    /// Set the severity
    pub fn with_severity(mut self, severity: Severity) -> Self {
        self.severity = severity;
        self
    }

    /// Set the target
    pub fn with_target(mut self, kind: Option<&str>, name: Option<&str>) -> Self {
        self.target = Target {
            kind: kind.map(String::from),
            name: name.map(String::from),
            parent: None,
        };
        self
    }

    /// Add a tag
    pub fn with_tag(mut self, tag: &str) -> Self {
        self.tags.push(tag.to_string());
        self
    }

    /// Set documentation URL
    pub fn with_docs(mut self, url: &str) -> Self {
        self.docs = Some(url.to_string());
        self
    }

    /// Set the description
    pub fn with_description(mut self, desc: &str) -> Self {
        self.description = Some(desc.to_string());
        self
    }

    /// Check if rule matches the given tags
    pub fn has_tag(&self, tag: &str) -> bool {
        self.tags.iter().any(|t| t == tag)
    }

    /// Check if rule matches any of the given tags
    pub fn has_any_tag(&self, tags: &HashSet<String>) -> bool {
        self.tags.iter().any(|t| tags.contains(t))
    }

    /// Set the language context(s) this rule applies to
    pub fn with_context(mut self, contexts: &[&str]) -> Self {
        self.context = contexts.iter().map(|s| s.to_string()).collect();
        self
    }

    /// Add a single language context
    pub fn with_context_single(mut self, ctx: &str) -> Self {
        self.context.push(ctx.to_string());
        self
    }

    /// Check if rule applies to the given language context
    /// Returns true if:
    /// - Rule has no specific context (applies to main language)
    /// - Rule's context list contains the given context
    /// - Rule's context list contains "*" (all contexts)
    pub fn applies_to_context(&self, ctx: &str) -> bool {
        if self.context.is_empty() {
            return ctx.is_empty() || ctx == "main";
        }
        self.context.iter().any(|c| c == ctx || c == "*")
    }

    /// Set a custom fix suggestion
    pub fn with_fix(mut self, description: &str, value: &str) -> Self {
        self.fix = Some(FixSuggestion {
            action: FixAction::Custom,
            attribute: None,
            value: if value.is_empty() {
                None
            } else {
                Some(value.to_string())
            },
            description: Some(description.to_string()),
        });
        self
    }
}

/// Rule file format (for loading from YAML/JSON)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleFile {
    /// File format version
    #[serde(default)]
    pub version: Option<String>,

    /// Plugin this rule file belongs to
    #[serde(default)]
    pub plugin: Option<String>,

    /// Rules defined in this file
    pub rules: Vec<Rule>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rule_creation() {
        let rule = Rule::new("test-rule", "!attributes.Id", "Missing Id attribute");

        assert_eq!(rule.id, "test-rule");
        assert_eq!(rule.condition, "!attributes.Id");
        assert_eq!(rule.message, "Missing Id attribute");
        assert_eq!(rule.severity, Severity::Warning);
        assert!(rule.enabled);
    }

    #[test]
    fn test_rule_builder() {
        let rule = Rule::new("test", "condition", "message")
            .with_severity(Severity::Error)
            .with_target(Some("element"), Some("Package"))
            .with_tag("required")
            .with_docs("https://example.com");

        assert_eq!(rule.severity, Severity::Error);
        assert_eq!(rule.target.kind, Some("element".to_string()));
        assert_eq!(rule.target.name, Some("Package".to_string()));
        assert!(rule.has_tag("required"));
        assert_eq!(rule.docs, Some("https://example.com".to_string()));
    }

    #[test]
    fn test_rule_tags() {
        let rule = Rule::new("test", "cond", "msg")
            .with_tag("required")
            .with_tag("best-practice");

        assert!(rule.has_tag("required"));
        assert!(rule.has_tag("best-practice"));
        assert!(!rule.has_tag("deprecated"));

        let mut tags = HashSet::new();
        tags.insert("required".to_string());
        assert!(rule.has_any_tag(&tags));

        let mut other_tags = HashSet::new();
        other_tags.insert("other".to_string());
        assert!(!rule.has_any_tag(&other_tags));
    }

    #[test]
    fn test_rule_file_deserialize() {
        let yaml = r#"
version: "1.0"
plugin: wix
rules:
  - id: test-rule
    severity: error
    condition: "!attributes.Id"
    message: "Missing Id"
    tags:
      - required
"#;

        let file: RuleFile = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(file.version, Some("1.0".to_string()));
        assert_eq!(file.plugin, Some("wix".to_string()));
        assert_eq!(file.rules.len(), 1);
        assert_eq!(file.rules[0].id, "test-rule");
        assert_eq!(file.rules[0].severity, Severity::Error);
    }

    #[test]
    fn test_rule_context() {
        // Rule with no context applies to main language
        let rule = Rule::new("test", "cond", "msg");
        assert!(rule.applies_to_context(""));
        assert!(rule.applies_to_context("main"));
        assert!(!rule.applies_to_context("shell")); // No context means main only

        // Rule with specific context
        let rule2 = Rule::new("test2", "cond", "msg").with_context(&["shell", "powershell"]);
        assert!(rule2.applies_to_context("shell"));
        assert!(rule2.applies_to_context("powershell"));
        assert!(!rule2.applies_to_context("groovy"));
        assert!(!rule2.applies_to_context("main"));

        // Rule with wildcard context
        let rule3 = Rule::new("test3", "cond", "msg").with_context(&["*"]);
        assert!(rule3.applies_to_context("shell"));
        assert!(rule3.applies_to_context("powershell"));
        assert!(rule3.applies_to_context("groovy"));
        assert!(rule3.applies_to_context("main"));
    }

    #[test]
    fn test_rule_context_single() {
        let rule = Rule::new("test", "cond", "msg")
            .with_context_single("shell")
            .with_context_single("batch");
        assert!(rule.applies_to_context("shell"));
        assert!(rule.applies_to_context("batch"));
        assert!(!rule.applies_to_context("powershell"));
    }

    #[test]
    fn test_rule_context_deserialize() {
        let yaml = r#"
rules:
  - id: shell-rule
    condition: "content =~ /rm -rf/"
    message: "Dangerous rm command"
    context:
      - shell
      - bash
"#;

        let file: RuleFile = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(file.rules.len(), 1);
        assert_eq!(file.rules[0].context, vec!["shell", "bash"]);
        assert!(file.rules[0].applies_to_context("shell"));
        assert!(file.rules[0].applies_to_context("bash"));
        assert!(!file.rules[0].applies_to_context("powershell"));
    }
}
