//! Plugin system for custom analyzer rules
//!
//! Allows loading custom rules from external files or implementing
//! custom analyzers programmatically.
//!
//! # Plugin Format
//!
//! Plugins are defined as JSON files with rule definitions:
//!
//! ```json
//! {
//!   "name": "my-custom-rules",
//!   "version": "1.0.0",
//!   "rules": [
//!     {
//!       "id": "CUSTOM-001",
//!       "name": "require-description",
//!       "description": "All packages must have a Description attribute",
//!       "category": "best_practice",
//!       "severity": "medium",
//!       "element": "Package",
//!       "condition": {
//!         "type": "missing_attribute",
//!         "attribute": "Description"
//!       },
//!       "message": "Package is missing Description attribute",
//!       "help": "Add a Description attribute to improve maintainability"
//!     }
//!   ]
//! }
//! ```

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

use crate::core::{AnalysisResult, Category, Diagnostic, Location, Severity, WixDocument};

/// Plugin manifest
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManifest {
    /// Plugin name
    pub name: String,
    /// Plugin version
    pub version: String,
    /// Optional description
    #[serde(default)]
    pub description: Option<String>,
    /// Optional author
    #[serde(default)]
    pub author: Option<String>,
    /// Rule definitions
    pub rules: Vec<PluginRule>,
}

/// A plugin rule definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginRule {
    /// Rule ID (e.g., "CUSTOM-001")
    pub id: String,
    /// Rule name
    pub name: String,
    /// Rule description
    pub description: String,
    /// Category
    #[serde(default)]
    pub category: PluginCategory,
    /// Severity
    #[serde(default)]
    pub severity: PluginSeverity,
    /// Element to match (optional, matches all if not specified)
    #[serde(default)]
    pub element: Option<String>,
    /// Condition to check
    pub condition: RuleCondition,
    /// Error message template
    pub message: String,
    /// Help text
    #[serde(default)]
    pub help: Option<String>,
    /// Effort in minutes to fix
    #[serde(default)]
    pub effort_minutes: Option<u32>,
    /// Tags
    #[serde(default)]
    pub tags: Vec<String>,
    /// Whether this rule is deprecated
    #[serde(default)]
    pub deprecated: bool,
    /// Replacement rule ID (if deprecated)
    #[serde(default)]
    pub deprecated_by: Option<String>,
    /// Version when rule was deprecated
    #[serde(default)]
    pub deprecated_since: Option<String>,
    /// Documentation URL for the rule
    #[serde(default)]
    pub doc_url: Option<String>,
}

impl PluginRule {
    /// Create a new plugin rule with minimal required fields
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        description: impl Into<String>,
        condition: RuleCondition,
        message: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: description.into(),
            category: PluginCategory::default(),
            severity: PluginSeverity::default(),
            element: None,
            condition,
            message: message.into(),
            help: None,
            effort_minutes: None,
            tags: Vec::new(),
            deprecated: false,
            deprecated_by: None,
            deprecated_since: None,
            doc_url: None,
        }
    }

    /// Set the element filter
    pub fn for_element(mut self, element: impl Into<String>) -> Self {
        self.element = Some(element.into());
        self
    }

    /// Set category
    pub fn with_category(mut self, category: PluginCategory) -> Self {
        self.category = category;
        self
    }

    /// Set severity
    pub fn with_severity(mut self, severity: PluginSeverity) -> Self {
        self.severity = severity;
        self
    }

    /// Mark as deprecated
    pub fn deprecated(mut self) -> Self {
        self.deprecated = true;
        self
    }

    /// Mark as deprecated with replacement rule
    pub fn deprecated_with_replacement(mut self, replacement: impl Into<String>) -> Self {
        self.deprecated = true;
        self.deprecated_by = Some(replacement.into());
        self
    }

    /// Set deprecation version
    pub fn deprecated_since_version(mut self, version: impl Into<String>) -> Self {
        self.deprecated_since = Some(version.into());
        self
    }

    /// Set documentation URL
    pub fn with_doc_url(mut self, url: impl Into<String>) -> Self {
        self.doc_url = Some(url.into());
        self
    }
}

/// Plugin category (maps to internal Category)
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PluginCategory {
    Validation,
    #[default]
    BestPractice,
    Security,
    DeadCode,
}

impl From<PluginCategory> for Category {
    fn from(pc: PluginCategory) -> Self {
        match pc {
            PluginCategory::Validation => Category::Validation,
            PluginCategory::BestPractice => Category::BestPractice,
            PluginCategory::Security => Category::Security,
            PluginCategory::DeadCode => Category::DeadCode,
        }
    }
}

/// Plugin severity (maps to internal Severity)
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PluginSeverity {
    Blocker,
    High,
    #[default]
    Medium,
    Low,
    Info,
}

impl From<PluginSeverity> for Severity {
    fn from(ps: PluginSeverity) -> Self {
        match ps {
            PluginSeverity::Blocker => Severity::Blocker,
            PluginSeverity::High => Severity::High,
            PluginSeverity::Medium => Severity::Medium,
            PluginSeverity::Low => Severity::Low,
            PluginSeverity::Info => Severity::Info,
        }
    }
}

/// Rule condition types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum RuleCondition {
    /// Element is missing a required attribute
    MissingAttribute { attribute: String },
    /// Attribute has a specific value
    AttributeEquals { attribute: String, value: String },
    /// Attribute matches a pattern
    AttributeMatches { attribute: String, pattern: String },
    /// Attribute does NOT match a pattern
    AttributeNotMatches { attribute: String, pattern: String },
    /// Element has no children
    NoChildren,
    /// Element has specific child element
    HasChild { element: String },
    /// Element is missing specific child element
    MissingChild { element: String },
    /// Attribute value is in a forbidden list
    AttributeIn {
        attribute: String,
        values: Vec<String>,
    },
    /// Attribute value is NOT in an allowed list
    AttributeNotIn {
        attribute: String,
        values: Vec<String>,
    },
    /// Compound condition: all must match
    All { conditions: Vec<RuleCondition> },
    /// Compound condition: any must match
    Any { conditions: Vec<RuleCondition> },
    /// Negation
    Not { condition: Box<RuleCondition> },
}

/// Plugin registry
#[derive(Debug, Default)]
pub struct PluginRegistry {
    plugins: Vec<PluginManifest>,
    rules_by_element: HashMap<String, Vec<(usize, usize)>>, // (plugin_idx, rule_idx)
    global_rules: Vec<(usize, usize)>,                      // Rules without element filter
}

impl PluginRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Load a plugin from a JSON file
    pub fn load_plugin(&mut self, path: &Path) -> Result<(), PluginError> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| PluginError::LoadError(path.to_path_buf(), e.to_string()))?;

        let manifest: PluginManifest = serde_json::from_str(&content)
            .map_err(|e| PluginError::ParseError(path.to_path_buf(), e.to_string()))?;

        self.register_plugin(manifest);
        Ok(())
    }

    /// Register a plugin manifest
    pub fn register_plugin(&mut self, manifest: PluginManifest) {
        let plugin_idx = self.plugins.len();

        for (rule_idx, rule) in manifest.rules.iter().enumerate() {
            if let Some(element) = &rule.element {
                self.rules_by_element
                    .entry(element.clone())
                    .or_default()
                    .push((plugin_idx, rule_idx));
            } else {
                self.global_rules.push((plugin_idx, rule_idx));
            }
        }

        self.plugins.push(manifest);
    }

    /// Load all plugins from a directory
    pub fn load_plugins_dir(&mut self, dir: &Path) -> Vec<PluginError> {
        let mut errors = Vec::new();

        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.filter_map(|e| e.ok()) {
                let path = entry.path();
                if path.extension().map(|e| e == "json").unwrap_or(false) {
                    if let Err(e) = self.load_plugin(&path) {
                        errors.push(e);
                    }
                }
            }
        }

        errors
    }

    /// Get all registered plugin names
    pub fn plugin_names(&self) -> Vec<&str> {
        self.plugins.iter().map(|p| p.name.as_str()).collect()
    }

    /// Get total rule count
    pub fn rule_count(&self) -> usize {
        self.plugins.iter().map(|p| p.rules.len()).sum()
    }

    /// Get count of deprecated rules
    pub fn deprecated_rule_count(&self) -> usize {
        self.plugins
            .iter()
            .flat_map(|p| &p.rules)
            .filter(|r| r.deprecated)
            .count()
    }

    /// Get all deprecated rules with their replacement info
    pub fn deprecated_rules(&self) -> Vec<DeprecatedRuleInfo> {
        self.plugins
            .iter()
            .flat_map(|p| p.rules.iter().map(move |r| (p, r)))
            .filter(|(_, r)| r.deprecated)
            .map(|(p, r)| DeprecatedRuleInfo {
                rule_id: r.id.clone(),
                plugin_name: p.name.clone(),
                deprecated_by: r.deprecated_by.clone(),
                deprecated_since: r.deprecated_since.clone(),
                message: format!(
                    "Rule '{}' is deprecated{}{}",
                    r.id,
                    r.deprecated_by
                        .as_ref()
                        .map(|r| format!(", use '{}' instead", r))
                        .unwrap_or_default(),
                    r.deprecated_since
                        .as_ref()
                        .map(|v| format!(" (since {})", v))
                        .unwrap_or_default(),
                ),
            })
            .collect()
    }

    /// Check if a rule is deprecated
    pub fn is_rule_deprecated(&self, rule_id: &str) -> bool {
        self.plugins
            .iter()
            .flat_map(|p| &p.rules)
            .any(|r| r.id == rule_id && r.deprecated)
    }

    /// Get rule by ID
    pub fn get_rule(&self, rule_id: &str) -> Option<&PluginRule> {
        self.plugins
            .iter()
            .flat_map(|p| &p.rules)
            .find(|r| r.id == rule_id)
    }

    /// Run plugin rules against a document
    pub fn analyze(&self, doc: &WixDocument) -> AnalysisResult {
        let mut result = AnalysisResult::new();

        for node in doc.root().descendants() {
            if !node.is_element() {
                continue;
            }

            let element_name = node.tag_name().name();

            // Check element-specific rules
            if let Some(rules) = self.rules_by_element.get(element_name) {
                for (plugin_idx, rule_idx) in rules {
                    let rule = &self.plugins[*plugin_idx].rules[*rule_idx];
                    if self.check_condition(&node, &rule.condition) {
                        let range = doc.node_range(&node);
                        let location = Location::new(doc.file().to_path_buf(), range);
                        let diag = self.create_diagnostic(rule, location, &node);
                        result.add(diag);
                    }
                }
            }

            // Check global rules
            for (plugin_idx, rule_idx) in &self.global_rules {
                let rule = &self.plugins[*plugin_idx].rules[*rule_idx];
                if self.check_condition(&node, &rule.condition) {
                    let range = doc.node_range(&node);
                    let location = Location::new(doc.file().to_path_buf(), range);
                    let diag = self.create_diagnostic(rule, location, &node);
                    result.add(diag);
                }
            }
        }

        result
    }

    fn check_condition(&self, node: &roxmltree::Node, condition: &RuleCondition) -> bool {
        match condition {
            RuleCondition::MissingAttribute { attribute } => {
                node.attribute(attribute.as_str()).is_none()
            }
            RuleCondition::AttributeEquals { attribute, value } => {
                node.attribute(attribute.as_str()) == Some(value.as_str())
            }
            RuleCondition::AttributeMatches { attribute, pattern } => {
                if let Some(val) = node.attribute(attribute.as_str()) {
                    regex::Regex::new(pattern)
                        .map(|re| re.is_match(val))
                        .unwrap_or(false)
                } else {
                    false
                }
            }
            RuleCondition::AttributeNotMatches { attribute, pattern } => {
                if let Some(val) = node.attribute(attribute.as_str()) {
                    regex::Regex::new(pattern)
                        .map(|re| !re.is_match(val))
                        .unwrap_or(true)
                } else {
                    true
                }
            }
            RuleCondition::NoChildren => !node.children().any(|c| c.is_element()),
            RuleCondition::HasChild { element } => node
                .children()
                .any(|c| c.is_element() && c.tag_name().name() == element),
            RuleCondition::MissingChild { element } => !node
                .children()
                .any(|c| c.is_element() && c.tag_name().name() == element),
            RuleCondition::AttributeIn { attribute, values } => {
                if let Some(val) = node.attribute(attribute.as_str()) {
                    values.iter().any(|v| v == val)
                } else {
                    false
                }
            }
            RuleCondition::AttributeNotIn { attribute, values } => {
                if let Some(val) = node.attribute(attribute.as_str()) {
                    !values.iter().any(|v| v == val)
                } else {
                    true
                }
            }
            RuleCondition::All { conditions } => {
                conditions.iter().all(|c| self.check_condition(node, c))
            }
            RuleCondition::Any { conditions } => {
                conditions.iter().any(|c| self.check_condition(node, c))
            }
            RuleCondition::Not { condition } => !self.check_condition(node, condition),
        }
    }

    fn create_diagnostic(
        &self,
        rule: &PluginRule,
        location: Location,
        node: &roxmltree::Node,
    ) -> Diagnostic {
        // Expand message template with node info
        let message = self.expand_template(&rule.message, node);

        let mut diag = Diagnostic::new(
            &rule.id,
            rule.category.into(),
            rule.severity.into(),
            message,
            location,
        );

        if let Some(help) = &rule.help {
            diag = diag.with_help(self.expand_template(help, node));
        }

        if let Some(effort) = rule.effort_minutes {
            diag = diag.with_effort(effort);
        }

        if !rule.tags.is_empty() {
            diag = diag.with_tags(rule.tags.clone());
        }

        diag
    }

    fn expand_template(&self, template: &str, node: &roxmltree::Node) -> String {
        let mut result = template.to_string();

        // Replace {element} with element name
        result = result.replace("{element}", node.tag_name().name());

        // Replace {attr:NAME} with attribute value
        let attr_pattern = regex::Regex::new(r"\{attr:(\w+)\}").unwrap();
        result = attr_pattern
            .replace_all(&result, |caps: &regex::Captures| {
                let attr_name = &caps[1];
                node.attribute(attr_name).unwrap_or("(none)").to_string()
            })
            .to_string();

        result
    }
}

/// Information about a deprecated rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeprecatedRuleInfo {
    /// The deprecated rule ID
    pub rule_id: String,
    /// Plugin that contains the rule
    pub plugin_name: String,
    /// Replacement rule ID (if any)
    pub deprecated_by: Option<String>,
    /// Version when rule was deprecated
    pub deprecated_since: Option<String>,
    /// Human-readable deprecation message
    pub message: String,
}

/// Plugin error types
#[derive(Debug)]
pub enum PluginError {
    LoadError(std::path::PathBuf, String),
    ParseError(std::path::PathBuf, String),
}

impl std::fmt::Display for PluginError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::LoadError(path, msg) => {
                write!(f, "Failed to load plugin '{}': {}", path.display(), msg)
            }
            Self::ParseError(path, msg) => {
                write!(f, "Failed to parse plugin '{}': {}", path.display(), msg)
            }
        }
    }
}

impl std::error::Error for PluginError {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::WixDocument;
    use std::path::Path;

    fn make_plugin() -> PluginManifest {
        PluginManifest {
            name: "test-plugin".to_string(),
            version: "1.0.0".to_string(),
            description: Some("Test plugin".to_string()),
            author: None,
            rules: vec![PluginRule {
                id: "TEST-001".to_string(),
                name: "require-id".to_string(),
                description: "Components must have Id".to_string(),
                category: PluginCategory::Validation,
                severity: PluginSeverity::High,
                element: Some("Component".to_string()),
                condition: RuleCondition::MissingAttribute {
                    attribute: "Id".to_string(),
                },
                message: "Component is missing Id attribute".to_string(),
                help: Some("Add an Id attribute".to_string()),
                effort_minutes: Some(5),
                tags: vec!["required".to_string()],
                deprecated: false,
                deprecated_by: None,
                deprecated_since: None,
                doc_url: None,
            }],
        }
    }

    #[test]
    fn test_plugin_registry_new() {
        let registry = PluginRegistry::new();
        assert_eq!(registry.rule_count(), 0);
    }

    #[test]
    fn test_register_plugin() {
        let mut registry = PluginRegistry::new();
        registry.register_plugin(make_plugin());

        assert_eq!(registry.plugin_names(), vec!["test-plugin"]);
        assert_eq!(registry.rule_count(), 1);
    }

    #[test]
    fn test_missing_attribute_condition() {
        let mut registry = PluginRegistry::new();
        registry.register_plugin(make_plugin());

        let source = r#"<Wix><Component /></Wix>"#;
        let doc = WixDocument::parse(source, Path::new("test.wxs")).unwrap();
        let result = registry.analyze(&doc);

        assert_eq!(result.len(), 1);
        assert_eq!(result.diagnostics[0].rule_id, "TEST-001");
    }

    #[test]
    fn test_attribute_equals_condition() {
        let mut registry = PluginRegistry::new();
        registry.register_plugin(PluginManifest {
            name: "test".to_string(),
            version: "1.0".to_string(),
            description: None,
            author: None,
            rules: vec![PluginRule::new(
                "TEST-002",
                "no-local-system",
                "No LocalSystem",
                RuleCondition::AttributeEquals {
                    attribute: "Account".to_string(),
                    value: "LocalSystem".to_string(),
                },
                "Service uses LocalSystem",
            )
            .for_element("ServiceInstall")
            .with_category(PluginCategory::Security)
            .with_severity(PluginSeverity::High)],
        });

        let source = r#"<Wix><ServiceInstall Account="LocalSystem" /></Wix>"#;
        let doc = WixDocument::parse(source, Path::new("test.wxs")).unwrap();
        let result = registry.analyze(&doc);

        assert_eq!(result.len(), 1);
    }

    #[test]
    fn test_attribute_matches_condition() {
        let mut registry = PluginRegistry::new();
        registry.register_plugin(PluginManifest {
            name: "test".to_string(),
            version: "1.0".to_string(),
            description: None,
            author: None,
            rules: vec![
                PluginRule::new(
                    "TEST-003",
                    "valid-guid",
                    "Invalid GUID",
                    RuleCondition::AttributeNotMatches {
                        attribute: "Guid".to_string(),
                        pattern: r"^[0-9A-Fa-f]{8}-[0-9A-Fa-f]{4}-[0-9A-Fa-f]{4}-[0-9A-Fa-f]{4}-[0-9A-Fa-f]{12}$".to_string(),
                    },
                    "Invalid GUID format",
                )
                .for_element("Component")
                .with_category(PluginCategory::Validation)
                .with_severity(PluginSeverity::High),
            ],
        });

        let source = r#"<Wix><Component Guid="invalid" /></Wix>"#;
        let doc = WixDocument::parse(source, Path::new("test.wxs")).unwrap();
        let result = registry.analyze(&doc);

        assert_eq!(result.len(), 1);
    }

    #[test]
    fn test_no_children_condition() {
        let mut registry = PluginRegistry::new();
        registry.register_plugin(PluginManifest {
            name: "test".to_string(),
            version: "1.0".to_string(),
            description: None,
            author: None,
            rules: vec![PluginRule::new(
                "TEST-004",
                "empty-feature",
                "Empty feature",
                RuleCondition::NoChildren,
                "Feature has no children",
            )
            .for_element("Feature")
            .with_severity(PluginSeverity::Medium)],
        });

        let source = r#"<Wix><Feature Id="Main" /></Wix>"#;
        let doc = WixDocument::parse(source, Path::new("test.wxs")).unwrap();
        let result = registry.analyze(&doc);

        assert_eq!(result.len(), 1);
    }

    #[test]
    fn test_has_child_condition() {
        let mut registry = PluginRegistry::new();
        registry.register_plugin(PluginManifest {
            name: "test".to_string(),
            version: "1.0".to_string(),
            description: None,
            author: None,
            rules: vec![PluginRule::new(
                "TEST-005",
                "has-condition",
                "Has condition",
                RuleCondition::HasChild {
                    element: "Condition".to_string(),
                },
                "Component has condition",
            )
            .for_element("Component")
            .with_severity(PluginSeverity::Info)],
        });

        let source = r#"<Wix><Component Id="C1"><Condition>1</Condition></Component></Wix>"#;
        let doc = WixDocument::parse(source, Path::new("test.wxs")).unwrap();
        let result = registry.analyze(&doc);

        assert_eq!(result.len(), 1);
    }

    #[test]
    fn test_missing_child_condition() {
        let mut registry = PluginRegistry::new();
        registry.register_plugin(PluginManifest {
            name: "test".to_string(),
            version: "1.0".to_string(),
            description: None,
            author: None,
            rules: vec![PluginRule::new(
                "TEST-006",
                "missing-file",
                "Missing file",
                RuleCondition::MissingChild {
                    element: "File".to_string(),
                },
                "Component has no File",
            )
            .for_element("Component")
            .with_category(PluginCategory::Validation)
            .with_severity(PluginSeverity::Medium)],
        });

        let source = r#"<Wix><Component Id="C1" /></Wix>"#;
        let doc = WixDocument::parse(source, Path::new("test.wxs")).unwrap();
        let result = registry.analyze(&doc);

        assert_eq!(result.len(), 1);
    }

    #[test]
    fn test_attribute_in_condition() {
        let mut registry = PluginRegistry::new();
        registry.register_plugin(PluginManifest {
            name: "test".to_string(),
            version: "1.0".to_string(),
            description: None,
            author: None,
            rules: vec![PluginRule::new(
                "TEST-007",
                "forbidden-type",
                "Forbidden type",
                RuleCondition::AttributeIn {
                    attribute: "Execute".to_string(),
                    values: vec!["deferred".to_string(), "rollback".to_string()],
                },
                "Using elevated execution",
            )
            .for_element("CustomAction")
            .with_category(PluginCategory::Security)
            .with_severity(PluginSeverity::High)],
        });

        let source = r#"<Wix><CustomAction Id="CA1" Execute="deferred" /></Wix>"#;
        let doc = WixDocument::parse(source, Path::new("test.wxs")).unwrap();
        let result = registry.analyze(&doc);

        assert_eq!(result.len(), 1);
    }

    #[test]
    fn test_compound_all_condition() {
        let mut registry = PluginRegistry::new();
        registry.register_plugin(PluginManifest {
            name: "test".to_string(),
            version: "1.0".to_string(),
            description: None,
            author: None,
            rules: vec![PluginRule::new(
                "TEST-008",
                "compound",
                "Compound test",
                RuleCondition::All {
                    conditions: vec![
                        RuleCondition::MissingAttribute {
                            attribute: "Id".to_string(),
                        },
                        RuleCondition::MissingAttribute {
                            attribute: "Guid".to_string(),
                        },
                    ],
                },
                "Missing both Id and Guid",
            )
            .for_element("Component")
            .with_category(PluginCategory::Validation)
            .with_severity(PluginSeverity::High)],
        });

        let source = r#"<Wix><Component /></Wix>"#;
        let doc = WixDocument::parse(source, Path::new("test.wxs")).unwrap();
        let result = registry.analyze(&doc);

        assert_eq!(result.len(), 1);
    }

    #[test]
    fn test_compound_any_condition() {
        let mut registry = PluginRegistry::new();
        registry.register_plugin(PluginManifest {
            name: "test".to_string(),
            version: "1.0".to_string(),
            description: None,
            author: None,
            rules: vec![PluginRule::new(
                "TEST-009",
                "any-missing",
                "Any missing",
                RuleCondition::Any {
                    conditions: vec![
                        RuleCondition::MissingAttribute {
                            attribute: "Id".to_string(),
                        },
                        RuleCondition::MissingAttribute {
                            attribute: "Guid".to_string(),
                        },
                    ],
                },
                "Missing Id or Guid",
            )
            .for_element("Component")
            .with_category(PluginCategory::Validation)
            .with_severity(PluginSeverity::Medium)],
        });

        let source = r#"<Wix><Component Id="C1" /></Wix>"#;
        let doc = WixDocument::parse(source, Path::new("test.wxs")).unwrap();
        let result = registry.analyze(&doc);

        assert_eq!(result.len(), 1); // Missing Guid
    }

    #[test]
    fn test_not_condition() {
        let mut registry = PluginRegistry::new();
        registry.register_plugin(PluginManifest {
            name: "test".to_string(),
            version: "1.0".to_string(),
            description: None,
            author: None,
            rules: vec![PluginRule::new(
                "TEST-010",
                "has-id",
                "Has Id",
                RuleCondition::Not {
                    condition: Box::new(RuleCondition::MissingAttribute {
                        attribute: "Id".to_string(),
                    }),
                },
                "Component has Id",
            )
            .for_element("Component")
            .with_severity(PluginSeverity::Info)],
        });

        let source = r#"<Wix><Component Id="C1" /></Wix>"#;
        let doc = WixDocument::parse(source, Path::new("test.wxs")).unwrap();
        let result = registry.analyze(&doc);

        assert_eq!(result.len(), 1);
    }

    #[test]
    fn test_message_template_expansion() {
        let mut registry = PluginRegistry::new();
        registry.register_plugin(PluginManifest {
            name: "test".to_string(),
            version: "1.0".to_string(),
            description: None,
            author: None,
            rules: vec![PluginRule::new(
                "TEST-011",
                "template",
                "Template test",
                RuleCondition::MissingAttribute {
                    attribute: "Guid".to_string(),
                },
                "{element} '{attr:Id}' is missing Guid",
            )
            .for_element("Component")
            .with_category(PluginCategory::Validation)
            .with_severity(PluginSeverity::Medium)],
        });

        let source = r#"<Wix><Component Id="MyComp" /></Wix>"#;
        let doc = WixDocument::parse(source, Path::new("test.wxs")).unwrap();
        let result = registry.analyze(&doc);

        assert_eq!(result.len(), 1);
        assert!(result.diagnostics[0].message.contains("Component"));
        assert!(result.diagnostics[0].message.contains("MyComp"));
    }

    #[test]
    fn test_global_rule_no_element() {
        let mut registry = PluginRegistry::new();
        registry.register_plugin(PluginManifest {
            name: "test".to_string(),
            version: "1.0".to_string(),
            description: None,
            author: None,
            rules: vec![
                PluginRule::new(
                    "TEST-012",
                    "global",
                    "Global test",
                    RuleCondition::MissingAttribute {
                        attribute: "Id".to_string(),
                    },
                    "Element missing Id",
                )
                .with_severity(PluginSeverity::Info),
                // No element filter - applies to all
            ],
        });

        let source = r#"<Wix><Component /><Feature /></Wix>"#;
        let doc = WixDocument::parse(source, Path::new("test.wxs")).unwrap();
        let result = registry.analyze(&doc);

        // Should match both Component and Feature (and Wix root)
        assert!(result.len() >= 2);
    }

    #[test]
    fn test_plugin_error_display() {
        let err = PluginError::LoadError(
            std::path::PathBuf::from("test.json"),
            "not found".to_string(),
        );
        assert!(err.to_string().contains("Failed to load plugin"));
        assert!(err.to_string().contains("test.json"));
    }

    #[test]
    fn test_plugin_category_default() {
        let cat: PluginCategory = Default::default();
        assert!(matches!(cat, PluginCategory::BestPractice));
    }

    #[test]
    fn test_plugin_severity_default() {
        let sev: PluginSeverity = Default::default();
        assert!(matches!(sev, PluginSeverity::Medium));
    }

    #[test]
    fn test_deprecated_rule() {
        let mut registry = PluginRegistry::new();
        registry.register_plugin(PluginManifest {
            name: "test".to_string(),
            version: "1.0".to_string(),
            description: None,
            author: None,
            rules: vec![
                PluginRule::new(
                    "OLD-001",
                    "old-rule",
                    "Old rule",
                    RuleCondition::MissingAttribute {
                        attribute: "Id".to_string(),
                    },
                    "Missing Id",
                )
                .for_element("Component")
                .deprecated_with_replacement("NEW-001")
                .deprecated_since_version("2.0.0"),
                PluginRule::new(
                    "NEW-001",
                    "new-rule",
                    "New rule",
                    RuleCondition::MissingAttribute {
                        attribute: "Id".to_string(),
                    },
                    "Missing Id",
                )
                .for_element("Component"),
            ],
        });

        assert_eq!(registry.deprecated_rule_count(), 1);
        assert!(registry.is_rule_deprecated("OLD-001"));
        assert!(!registry.is_rule_deprecated("NEW-001"));

        let deprecated = registry.deprecated_rules();
        assert_eq!(deprecated.len(), 1);
        assert_eq!(deprecated[0].rule_id, "OLD-001");
        assert_eq!(deprecated[0].deprecated_by, Some("NEW-001".to_string()));
        assert_eq!(deprecated[0].deprecated_since, Some("2.0.0".to_string()));
        assert!(deprecated[0].message.contains("deprecated"));
        assert!(deprecated[0].message.contains("NEW-001"));
    }

    #[test]
    fn test_get_rule() {
        let mut registry = PluginRegistry::new();
        registry.register_plugin(make_plugin());

        let rule = registry.get_rule("TEST-001");
        assert!(rule.is_some());
        assert_eq!(rule.unwrap().name, "require-id");

        let missing = registry.get_rule("NONEXISTENT");
        assert!(missing.is_none());
    }

    #[test]
    fn test_plugin_rule_builder() {
        let rule = PluginRule::new(
            "TEST-001",
            "test-rule",
            "Test description",
            RuleCondition::NoChildren,
            "Test message",
        )
        .for_element("Component")
        .with_category(PluginCategory::Security)
        .with_severity(PluginSeverity::High)
        .deprecated()
        .deprecated_since_version("1.0.0")
        .with_doc_url("https://example.com/rules/TEST-001");

        assert_eq!(rule.id, "TEST-001");
        assert_eq!(rule.element, Some("Component".to_string()));
        assert!(matches!(rule.category, PluginCategory::Security));
        assert!(matches!(rule.severity, PluginSeverity::High));
        assert!(rule.deprecated);
        assert_eq!(rule.deprecated_since, Some("1.0.0".to_string()));
        assert_eq!(
            rule.doc_url,
            Some("https://example.com/rules/TEST-001".to_string())
        );
    }
}
