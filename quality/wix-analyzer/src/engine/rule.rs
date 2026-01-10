//! Rule definitions for the analysis engine
//!
//! Supports both data-driven rules (conditions) and code-based rules (trait).

use super::condition::Condition;
use super::types::Document;

/// Rule severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RuleSeverity {
    /// Informational - suggestion only
    Info,
    /// Low - minor issue
    Low,
    /// Medium - should be fixed
    Medium,
    /// High - likely bug or security issue
    High,
    /// Critical - definite bug or security vulnerability
    Critical,
    /// Blocker - must be fixed before build
    Blocker,
}

impl RuleSeverity {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "info" | "information" => Some(RuleSeverity::Info),
            "low" => Some(RuleSeverity::Low),
            "medium" | "warning" => Some(RuleSeverity::Medium),
            "high" => Some(RuleSeverity::High),
            "critical" | "error" => Some(RuleSeverity::Critical),
            "blocker" => Some(RuleSeverity::Blocker),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            RuleSeverity::Info => "info",
            RuleSeverity::Low => "low",
            RuleSeverity::Medium => "medium",
            RuleSeverity::High => "high",
            RuleSeverity::Critical => "critical",
            RuleSeverity::Blocker => "blocker",
        }
    }
}

/// Rule categories
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RuleCategory {
    /// Validation - correctness checks
    Validation,
    /// Best practices - idioms and conventions
    BestPractice,
    /// Security - security vulnerabilities
    Security,
    /// Dead code - unused or unreachable code
    DeadCode,
    /// Performance - performance issues
    Performance,
    /// Maintainability - code quality
    Maintainability,
}

impl RuleCategory {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "validation" | "val" => Some(RuleCategory::Validation),
            "best-practice" | "bestpractice" | "bp" => Some(RuleCategory::BestPractice),
            "security" | "sec" => Some(RuleCategory::Security),
            "dead-code" | "deadcode" | "dead" => Some(RuleCategory::DeadCode),
            "performance" | "perf" => Some(RuleCategory::Performance),
            "maintainability" | "maint" => Some(RuleCategory::Maintainability),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            RuleCategory::Validation => "validation",
            RuleCategory::BestPractice => "best-practice",
            RuleCategory::Security => "security",
            RuleCategory::DeadCode => "dead-code",
            RuleCategory::Performance => "performance",
            RuleCategory::Maintainability => "maintainability",
        }
    }
}

/// A data-driven rule definition
#[derive(Debug, Clone)]
pub struct DataRule {
    /// Unique rule identifier (e.g., "BP-IDIOM-001")
    pub id: String,

    /// Human-readable name
    pub name: String,

    /// Full description
    pub description: String,

    /// Severity level
    pub severity: RuleSeverity,

    /// Category
    pub category: RuleCategory,

    /// Element this rule applies to (e.g., "Component")
    /// If None, applies to all elements
    pub element: Option<String>,

    /// The condition that triggers this rule
    pub condition: Condition,

    /// Message template with {{placeholders}}
    pub message: String,

    /// Additional help text
    pub help: Option<String>,

    /// Auto-fix template
    pub fix: Option<FixTemplate>,

    /// Tags for filtering
    pub tags: Vec<String>,

    /// Whether the rule is enabled by default
    pub enabled: bool,
}

impl DataRule {
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: String::new(),
            severity: RuleSeverity::Medium,
            category: RuleCategory::BestPractice,
            element: None,
            condition: Condition::Never,
            message: String::new(),
            help: None,
            fix: None,
            tags: Vec::new(),
            enabled: true,
        }
    }

    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = desc.into();
        self
    }

    pub fn with_severity(mut self, severity: RuleSeverity) -> Self {
        self.severity = severity;
        self
    }

    pub fn with_category(mut self, category: RuleCategory) -> Self {
        self.category = category;
        self
    }

    pub fn with_element(mut self, element: impl Into<String>) -> Self {
        self.element = Some(element.into());
        self
    }

    pub fn with_condition(mut self, condition: Condition) -> Self {
        self.condition = condition;
        self
    }

    pub fn with_message(mut self, message: impl Into<String>) -> Self {
        self.message = message.into();
        self
    }

    pub fn with_help(mut self, help: impl Into<String>) -> Self {
        self.help = Some(help.into());
        self
    }

    pub fn with_fix(mut self, fix: FixTemplate) -> Self {
        self.fix = Some(fix);
        self
    }

    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }

    pub fn disabled(mut self) -> Self {
        self.enabled = false;
        self
    }
}

/// Template for auto-fixes
#[derive(Debug, Clone)]
pub struct FixTemplate {
    /// Description of what the fix does
    pub description: String,

    /// The fix action to apply
    pub action: FixAction,
}

/// Types of fix actions
#[derive(Debug, Clone)]
pub enum FixAction {
    /// Add an attribute: { name: "Guid", value: "*" }
    AddAttribute { name: String, value: String },

    /// Remove an attribute
    RemoveAttribute { name: String },

    /// Replace attribute value
    ReplaceAttribute { name: String, new_value: String },

    /// Add a child element
    AddElement {
        element: String,
        attributes: Vec<(String, String)>,
        position: ElementPosition,
    },

    /// Remove the element
    RemoveElement,

    /// Replace text content
    ReplaceText { new_text: String },
}

/// Position for inserting elements
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ElementPosition {
    First,
    Last,
    Before(usize), // index
    After(usize),  // index
}

/// A diagnostic produced by a rule
#[derive(Debug, Clone)]
pub struct Diagnostic {
    /// Rule that produced this diagnostic
    pub rule_id: String,

    /// File path
    pub path: std::path::PathBuf,

    /// Source range (start_line, start_col, end_line, end_col)
    pub range: (usize, usize, usize, usize),

    /// Severity level
    pub severity: RuleSeverity,

    /// Message
    pub message: String,

    /// Additional help
    pub help: Option<String>,

    /// Suggested fix
    pub fix: Option<Fix>,
}

/// A concrete fix to apply
#[derive(Debug, Clone)]
pub struct Fix {
    /// Description
    pub description: String,

    /// Text edits to apply
    pub edits: Vec<TextEdit>,
}

/// A text edit
#[derive(Debug, Clone)]
pub struct TextEdit {
    /// Range to replace (start_offset, end_offset in bytes)
    pub range: (usize, usize),

    /// New text
    pub new_text: String,
}

/// Trait for code-based rules that need programmatic analysis
pub trait CodeRule: Send + Sync {
    /// Rule identifier
    fn id(&self) -> &str;

    /// Rule name
    fn name(&self) -> &str;

    /// Rule description
    fn description(&self) -> &str;

    /// Severity level
    fn severity(&self) -> RuleSeverity;

    /// Category
    fn category(&self) -> RuleCategory;

    /// Check a single document and return diagnostics
    fn check(&self, document: &dyn Document) -> Vec<Diagnostic>;

    /// Check multiple documents (for cross-file analysis)
    /// Default implementation calls check() on each document
    fn check_project(&self, documents: &[&dyn Document]) -> Vec<Diagnostic> {
        documents.iter().flat_map(|doc| self.check(*doc)).collect()
    }

    /// Whether this rule requires cross-file analysis
    fn needs_project_context(&self) -> bool {
        false
    }

    /// Tags for filtering
    fn tags(&self) -> &[String] {
        &[]
    }

    /// Whether the rule is enabled by default
    fn enabled(&self) -> bool {
        true
    }
}

/// Unified rule interface
pub enum RuleImpl {
    /// Data-driven rule
    Data(DataRule),

    /// Code-based rule
    Code(Box<dyn CodeRule>),
}

impl RuleImpl {
    pub fn id(&self) -> &str {
        match self {
            RuleImpl::Data(rule) => &rule.id,
            RuleImpl::Code(rule) => rule.id(),
        }
    }

    pub fn name(&self) -> &str {
        match self {
            RuleImpl::Data(rule) => &rule.name,
            RuleImpl::Code(rule) => rule.name(),
        }
    }

    pub fn severity(&self) -> RuleSeverity {
        match self {
            RuleImpl::Data(rule) => rule.severity,
            RuleImpl::Code(rule) => rule.severity(),
        }
    }

    pub fn category(&self) -> RuleCategory {
        match self {
            RuleImpl::Data(rule) => rule.category,
            RuleImpl::Code(rule) => rule.category(),
        }
    }

    pub fn enabled(&self) -> bool {
        match self {
            RuleImpl::Data(rule) => rule.enabled,
            RuleImpl::Code(rule) => rule.enabled(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_severity_ordering() {
        assert!(RuleSeverity::Info < RuleSeverity::Low);
        assert!(RuleSeverity::Low < RuleSeverity::Medium);
        assert!(RuleSeverity::Medium < RuleSeverity::High);
        assert!(RuleSeverity::High < RuleSeverity::Critical);
        assert!(RuleSeverity::Critical < RuleSeverity::Blocker);
    }

    #[test]
    fn test_severity_from_str() {
        assert_eq!(RuleSeverity::from_str("info"), Some(RuleSeverity::Info));
        assert_eq!(RuleSeverity::from_str("INFO"), Some(RuleSeverity::Info));
        assert_eq!(RuleSeverity::from_str("warning"), Some(RuleSeverity::Medium));
        assert_eq!(RuleSeverity::from_str("error"), Some(RuleSeverity::Critical));
        assert_eq!(RuleSeverity::from_str("unknown"), None);
    }

    #[test]
    fn test_category_from_str() {
        assert_eq!(RuleCategory::from_str("validation"), Some(RuleCategory::Validation));
        assert_eq!(RuleCategory::from_str("best-practice"), Some(RuleCategory::BestPractice));
        assert_eq!(RuleCategory::from_str("bp"), Some(RuleCategory::BestPractice));
        assert_eq!(RuleCategory::from_str("security"), Some(RuleCategory::Security));
        assert_eq!(RuleCategory::from_str("unknown"), None);
    }

    #[test]
    fn test_data_rule_builder() {
        let rule = DataRule::new("TEST-001", "test-rule")
            .with_description("A test rule")
            .with_severity(RuleSeverity::High)
            .with_category(RuleCategory::Security)
            .with_element("Component")
            .with_condition(Condition::AttributeMissing {
                name: "Guid".to_string(),
            })
            .with_message("Missing Guid attribute")
            .with_tags(vec!["security".to_string()]);

        assert_eq!(rule.id, "TEST-001");
        assert_eq!(rule.name, "test-rule");
        assert_eq!(rule.severity, RuleSeverity::High);
        assert_eq!(rule.category, RuleCategory::Security);
        assert_eq!(rule.element, Some("Component".to_string()));
        assert!(rule.enabled);
    }

    #[test]
    fn test_rule_impl() {
        let data_rule = DataRule::new("DATA-001", "data-rule")
            .with_severity(RuleSeverity::Medium);

        let rule = RuleImpl::Data(data_rule);
        assert_eq!(rule.id(), "DATA-001");
        assert_eq!(rule.name(), "data-rule");
        assert_eq!(rule.severity(), RuleSeverity::Medium);
    }
}
