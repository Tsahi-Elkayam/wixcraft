//! Rule evaluator - runs rules against documents
//!
//! The evaluator handles both data-driven and code-based rules.

use super::condition::ConditionEvaluator;
use super::plugin::LanguagePlugin;
use super::rule::{DataRule, Diagnostic, Fix, RuleImpl, RuleSeverity};
use super::types::{Document, Node};
use std::collections::HashMap;
use std::sync::Arc;

/// Configuration for the rule evaluator
#[derive(Debug, Clone, Default)]
pub struct EvaluatorConfig {
    /// Minimum severity to report
    pub min_severity: Option<RuleSeverity>,

    /// Rules to enable (if empty, all are enabled)
    pub enabled_rules: Vec<String>,

    /// Rules to disable
    pub disabled_rules: Vec<String>,

    /// Categories to include (if empty, all are included)
    pub categories: Vec<String>,

    /// Tags to filter by
    pub tags: Vec<String>,

    /// Maximum diagnostics to report (0 = unlimited)
    pub max_diagnostics: usize,
}

impl EvaluatorConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_min_severity(mut self, severity: RuleSeverity) -> Self {
        self.min_severity = Some(severity);
        self
    }

    pub fn with_enabled_rules(mut self, rules: Vec<String>) -> Self {
        self.enabled_rules = rules;
        self
    }

    pub fn with_disabled_rules(mut self, rules: Vec<String>) -> Self {
        self.disabled_rules = rules;
        self
    }

    pub fn with_max_diagnostics(mut self, max: usize) -> Self {
        self.max_diagnostics = max;
        self
    }
}

/// Statistics from an evaluation run
#[derive(Debug, Clone, Default)]
pub struct EvaluatorStats {
    /// Number of files analyzed
    pub files_analyzed: usize,

    /// Number of rules evaluated
    pub rules_evaluated: usize,

    /// Number of nodes checked
    pub nodes_checked: usize,

    /// Diagnostics by severity
    pub by_severity: HashMap<RuleSeverity, usize>,

    /// Diagnostics by rule
    pub by_rule: HashMap<String, usize>,

    /// Time taken in milliseconds
    pub time_ms: u64,
}

/// The main rule evaluator
pub struct RuleEvaluator {
    /// Registered language plugins
    plugins: Vec<Arc<dyn LanguagePlugin>>,

    /// Configuration
    config: EvaluatorConfig,

    /// Condition evaluator (shared)
    condition_evaluator: ConditionEvaluator,

    /// Collected statistics
    stats: EvaluatorStats,
}

impl RuleEvaluator {
    pub fn new() -> Self {
        Self {
            plugins: Vec::new(),
            config: EvaluatorConfig::default(),
            condition_evaluator: ConditionEvaluator::new(),
            stats: EvaluatorStats::default(),
        }
    }

    pub fn with_config(mut self, config: EvaluatorConfig) -> Self {
        self.config = config;
        self
    }

    /// Register a language plugin
    pub fn register_plugin(&mut self, plugin: Arc<dyn LanguagePlugin>) {
        self.plugins.push(plugin);
    }

    /// Get all registered plugins
    pub fn plugins(&self) -> &[Arc<dyn LanguagePlugin>] {
        &self.plugins
    }

    /// Check if a rule should be evaluated based on config
    fn should_evaluate_rule(&self, rule: &RuleImpl) -> bool {
        // Check if explicitly disabled
        if self.config.disabled_rules.contains(&rule.id().to_string()) {
            return false;
        }

        // Check if enabled list is specified and rule is not in it
        if !self.config.enabled_rules.is_empty()
            && !self.config.enabled_rules.contains(&rule.id().to_string())
        {
            return false;
        }

        // Check minimum severity
        if let Some(min) = self.config.min_severity {
            if rule.severity() < min {
                return false;
            }
        }

        // Check categories
        if !self.config.categories.is_empty()
            && !self
                .config
                .categories
                .contains(&rule.category().as_str().to_string())
        {
            return false;
        }

        // Check if rule is enabled by default
        rule.enabled()
    }

    /// Evaluate all rules against a document
    pub fn evaluate(&mut self, document: &dyn Document) -> Vec<Diagnostic> {
        let start = std::time::Instant::now();
        let mut diagnostics = Vec::new();

        self.stats.files_analyzed += 1;

        // Collect all applicable rules from plugins first to avoid borrow issues
        let mut rules_to_check: Vec<RuleImpl> = Vec::new();
        for plugin in &self.plugins {
            if !plugin.can_handle(document.path()) {
                continue;
            }
            for rule in plugin.rules() {
                if self.should_evaluate_rule(&rule) {
                    rules_to_check.push(rule);
                }
            }
        }

        // Now evaluate each rule
        for rule in rules_to_check {
            self.stats.rules_evaluated += 1;

            match &rule {
                RuleImpl::Data(data_rule) => {
                    let rule_diagnostics = self.evaluate_data_rule(data_rule, document);
                    diagnostics.extend(rule_diagnostics);
                }
                RuleImpl::Code(code_rule) => {
                    let rule_diagnostics = code_rule.check(document);
                    for d in &rule_diagnostics {
                        *self.stats.by_severity.entry(d.severity).or_insert(0) += 1;
                        *self.stats.by_rule.entry(d.rule_id.clone()).or_insert(0) += 1;
                    }
                    diagnostics.extend(rule_diagnostics);
                }
            }

            // Check max diagnostics
            if self.config.max_diagnostics > 0 && diagnostics.len() >= self.config.max_diagnostics {
                break;
            }
        }

        self.stats.time_ms = start.elapsed().as_millis() as u64;

        diagnostics
    }

    /// Evaluate a data rule against a document
    fn evaluate_data_rule(&mut self, rule: &DataRule, document: &dyn Document) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        // Get nodes to check
        let nodes: Vec<&dyn Node> = if let Some(element) = &rule.element {
            document.nodes_of_kind(element)
        } else {
            // All nodes
            let root = document.root();
            let mut all_nodes = vec![root];
            all_nodes.extend(root.descendants());
            all_nodes
        };

        for node in nodes {
            self.stats.nodes_checked += 1;

            if self.condition_evaluator.evaluate(&rule.condition, node) {
                let message = self.format_message(&rule.message, node);
                let range = node.range();

                let diagnostic = Diagnostic {
                    rule_id: rule.id.clone(),
                    path: document.path().to_path_buf(),
                    range,
                    severity: rule.severity,
                    message,
                    help: rule.help.clone(),
                    fix: rule.fix.as_ref().map(|f| self.create_fix(f, node)),
                };

                *self.stats.by_severity.entry(rule.severity).or_insert(0) += 1;
                *self.stats.by_rule.entry(rule.id.clone()).or_insert(0) += 1;

                diagnostics.push(diagnostic);
            }
        }

        diagnostics
    }

    /// Format a message template with node data
    fn format_message(&self, template: &str, node: &dyn Node) -> String {
        let mut message = template.to_string();

        // Replace {{kind}} with node kind
        message = message.replace("{{kind}}", node.kind());

        // Replace {{parent}} with parent kind
        if let Some(parent) = node.parent() {
            message = message.replace("{{parent}}", parent.kind());
        } else {
            message = message.replace("{{parent}}", "(root)");
        }

        // Replace {{attribute.Name}} with attribute value
        for attr in node.attributes() {
            let placeholder = format!("{{{{attribute.{}}}}}", attr.name);
            message = message.replace(&placeholder, &attr.value);
        }

        // Replace {{id}} shorthand - try Id first, then Name, then fallback
        let id_value = node
            .attribute("Id")
            .or_else(|| node.attribute("Name"))
            .unwrap_or("(unnamed)");
        message = message.replace("{{id}}", id_value);

        message
    }

    /// Create a fix from a template
    fn create_fix(&self, template: &super::rule::FixTemplate, _node: &dyn Node) -> Fix {
        // For now, return empty fix - full implementation would convert
        // FixAction to TextEdits based on node position
        Fix {
            description: template.description.clone(),
            edits: Vec::new(),
        }
    }

    /// Get evaluation statistics
    pub fn stats(&self) -> &EvaluatorStats {
        &self.stats
    }

    /// Reset statistics
    pub fn reset_stats(&mut self) {
        self.stats = EvaluatorStats::default();
    }
}

impl Default for RuleEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::condition::Condition;
    use crate::engine::plugin::{ParseResult, PluginCapabilities};
    use crate::engine::rule::RuleCategory;
    use crate::engine::types::Attribute;
    use std::path::{Path, PathBuf};

    // Mock document for testing
    struct MockDocument {
        path: PathBuf,
        source: String,
        root: MockNode,
    }

    struct MockNode {
        kind: String,
        text: String,
        attributes: Vec<Attribute>,
        children: Vec<MockNode>,
    }

    impl MockNode {
        fn new(kind: &str) -> Self {
            Self {
                kind: kind.to_string(),
                text: String::new(),
                attributes: Vec::new(),
                children: Vec::new(),
            }
        }

        fn with_attr(mut self, name: &str, value: &str) -> Self {
            self.attributes.push(Attribute::new(name, value));
            self
        }

        fn with_child(mut self, child: MockNode) -> Self {
            self.children.push(child);
            self
        }
    }

    impl Node for MockNode {
        fn kind(&self) -> &str {
            &self.kind
        }

        fn text(&self) -> &str {
            &self.text
        }

        fn range(&self) -> (usize, usize, usize, usize) {
            (1, 1, 1, 1)
        }

        fn parent(&self) -> Option<&dyn Node> {
            None
        }

        fn children(&self) -> Vec<&dyn Node> {
            self.children.iter().map(|c| c as &dyn Node).collect()
        }

        fn attribute(&self, name: &str) -> Option<&str> {
            self.attributes
                .iter()
                .find(|a| a.name == name)
                .map(|a| a.value.as_str())
        }

        fn attributes(&self) -> Vec<Attribute> {
            self.attributes.clone()
        }
    }

    impl Document for MockDocument {
        fn source(&self) -> &str {
            &self.source
        }

        fn path(&self) -> &Path {
            &self.path
        }

        fn root(&self) -> &dyn Node {
            &self.root
        }

        fn node_at(&self, _line: usize, _column: usize) -> Option<&dyn Node> {
            None
        }
    }

    // Mock plugin for testing
    struct MockPlugin {
        rules: Vec<DataRule>,
    }

    impl LanguagePlugin for MockPlugin {
        fn id(&self) -> &str {
            "mock"
        }

        fn name(&self) -> &str {
            "Mock"
        }

        fn version(&self) -> &str {
            "1.0"
        }

        fn extensions(&self) -> &[&str] {
            &[".mock"]
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

        fn data_rules(&self) -> Vec<DataRule> {
            self.rules.clone()
        }
    }

    #[test]
    fn test_evaluator_basic() {
        let rule = DataRule::new("TEST-001", "missing-id")
            .with_severity(RuleSeverity::High)
            .with_category(RuleCategory::Validation)
            .with_element("Component")
            .with_condition(Condition::AttributeMissing {
                name: "Id".to_string(),
            })
            .with_message("Component is missing Id attribute");

        let plugin = MockPlugin { rules: vec![rule] };

        let document = MockDocument {
            path: PathBuf::from("test.mock"),
            source: String::new(),
            root: MockNode::new("Package").with_child(
                MockNode::new("Component"), // Missing Id
            ),
        };

        let mut evaluator = RuleEvaluator::new();
        evaluator.register_plugin(Arc::new(plugin));

        let diagnostics = evaluator.evaluate(&document);

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].rule_id, "TEST-001");
        assert_eq!(diagnostics[0].severity, RuleSeverity::High);
    }

    #[test]
    fn test_evaluator_no_match() {
        let rule = DataRule::new("TEST-001", "missing-id")
            .with_element("Component")
            .with_condition(Condition::AttributeMissing {
                name: "Id".to_string(),
            })
            .with_message("Component is missing Id attribute");

        let plugin = MockPlugin { rules: vec![rule] };

        let document = MockDocument {
            path: PathBuf::from("test.mock"),
            source: String::new(),
            root: MockNode::new("Package").with_child(
                MockNode::new("Component").with_attr("Id", "C1"), // Has Id
            ),
        };

        let mut evaluator = RuleEvaluator::new();
        evaluator.register_plugin(Arc::new(plugin));

        let diagnostics = evaluator.evaluate(&document);
        assert!(diagnostics.is_empty());
    }

    #[test]
    fn test_evaluator_config_disable_rule() {
        let rule = DataRule::new("TEST-001", "test-rule")
            .with_element("Component")
            .with_condition(Condition::Always);

        let plugin = MockPlugin { rules: vec![rule] };

        let document = MockDocument {
            path: PathBuf::from("test.mock"),
            source: String::new(),
            root: MockNode::new("Package").with_child(MockNode::new("Component")),
        };

        let config = EvaluatorConfig::new().with_disabled_rules(vec!["TEST-001".to_string()]);

        let mut evaluator = RuleEvaluator::new().with_config(config);
        evaluator.register_plugin(Arc::new(plugin));

        let diagnostics = evaluator.evaluate(&document);
        assert!(diagnostics.is_empty());
    }

    #[test]
    fn test_evaluator_config_min_severity() {
        let low_rule = DataRule::new("LOW-001", "low-severity")
            .with_severity(RuleSeverity::Low)
            .with_element("Component")
            .with_condition(Condition::Always);

        let high_rule = DataRule::new("HIGH-001", "high-severity")
            .with_severity(RuleSeverity::High)
            .with_element("Component")
            .with_condition(Condition::Always);

        let plugin = MockPlugin {
            rules: vec![low_rule, high_rule],
        };

        let document = MockDocument {
            path: PathBuf::from("test.mock"),
            source: String::new(),
            root: MockNode::new("Package").with_child(MockNode::new("Component")),
        };

        let config = EvaluatorConfig::new().with_min_severity(RuleSeverity::High);

        let mut evaluator = RuleEvaluator::new().with_config(config);
        evaluator.register_plugin(Arc::new(plugin));

        let diagnostics = evaluator.evaluate(&document);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].rule_id, "HIGH-001");
    }

    #[test]
    fn test_message_formatting() {
        let rule = DataRule::new("TEST-001", "test")
            .with_element("Component")
            .with_condition(Condition::Always)
            .with_message("{{kind}} '{{id}}' has issue with {{attribute.Guid}}");

        let plugin = MockPlugin { rules: vec![rule] };

        let document = MockDocument {
            path: PathBuf::from("test.mock"),
            source: String::new(),
            root: MockNode::new("Package").with_child(
                MockNode::new("Component")
                    .with_attr("Id", "C1")
                    .with_attr("Guid", "*"),
            ),
        };

        let mut evaluator = RuleEvaluator::new();
        evaluator.register_plugin(Arc::new(plugin));

        let diagnostics = evaluator.evaluate(&document);
        assert_eq!(diagnostics[0].message, "Component 'C1' has issue with *");
    }

    #[test]
    fn test_evaluator_stats() {
        let rule = DataRule::new("TEST-001", "test")
            .with_severity(RuleSeverity::High)
            .with_element("Component")
            .with_condition(Condition::Always);

        let plugin = MockPlugin { rules: vec![rule] };

        let document = MockDocument {
            path: PathBuf::from("test.mock"),
            source: String::new(),
            root: MockNode::new("Package")
                .with_child(MockNode::new("Component"))
                .with_child(MockNode::new("Component")),
        };

        let mut evaluator = RuleEvaluator::new();
        evaluator.register_plugin(Arc::new(plugin));

        evaluator.evaluate(&document);

        let stats = evaluator.stats();
        assert_eq!(stats.files_analyzed, 1);
        assert_eq!(stats.rules_evaluated, 1);
        assert_eq!(stats.by_severity.get(&RuleSeverity::High), Some(&2));
        assert_eq!(stats.by_rule.get("TEST-001"), Some(&2));
    }
}
