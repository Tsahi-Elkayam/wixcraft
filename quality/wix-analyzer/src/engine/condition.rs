//! Condition types for data-driven rules
//!
//! Conditions are declarative expressions that can be evaluated against nodes.
//! They form the core of data-driven rules.

use super::types::Node;
use regex::Regex;
use std::collections::HashMap;

/// A condition that can be evaluated against a node
#[derive(Debug, Clone)]
pub enum Condition {
    /// Attribute is missing: { name: "Id" }
    AttributeMissing { name: String },

    /// Attribute equals value: { name: "Guid", value: "*" }
    AttributeEquals { name: String, value: String },

    /// Attribute does not equal value
    AttributeNotEquals { name: String, value: String },

    /// Attribute matches regex: { name: "Source", pattern: "^[A-Z]:\\\\" }
    AttributeMatches { name: String, pattern: String },

    /// Attribute does not match regex
    AttributeNotMatches { name: String, pattern: String },

    /// Attribute value is in list: { name: "Type", values: ["string", "integer"] }
    AttributeIn { name: String, values: Vec<String> },

    /// Attribute value is not in list
    AttributeNotIn { name: String, values: Vec<String> },

    /// Attribute exists (has any value)
    AttributeExists { name: String },

    /// Element has child of type: { element: "File" }
    HasChild { element: String },

    /// Element is missing child of type: { element: "MajorUpgrade" }
    MissingChild { element: String },

    /// Child count comparison: { element: "File", op: ">", value: 1 }
    ChildCount {
        element: String,
        op: CompareOp,
        value: usize,
    },

    /// Parent element is: { element: "Component" }
    ParentIs { element: String },

    /// Parent element is not
    ParentNot { element: String },

    /// Parent element is one of the listed elements
    ParentIn { elements: Vec<String> },

    /// Parent element is not one of the listed elements (invalid parent)
    ParentNotIn { elements: Vec<String> },

    /// Element depth exceeds max: { max: 5 }
    DepthExceeds { max: usize },

    /// Text content matches regex
    TextMatches { pattern: String },

    /// Text content contains substring
    TextContains { substring: String },

    /// All conditions must be true (AND)
    All(Vec<Condition>),

    /// Any condition must be true (OR)
    Any(Vec<Condition>),

    /// Condition must be false (NOT)
    Not(Box<Condition>),

    /// Always true (useful for element-only rules)
    Always,

    /// Always false
    Never,
}

/// Comparison operators for numeric conditions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompareOp {
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
}

impl CompareOp {
    pub fn evaluate(&self, left: usize, right: usize) -> bool {
        match self {
            CompareOp::Eq => left == right,
            CompareOp::Ne => left != right,
            CompareOp::Lt => left < right,
            CompareOp::Le => left <= right,
            CompareOp::Gt => left > right,
            CompareOp::Ge => left >= right,
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "==" | "=" => Some(CompareOp::Eq),
            "!=" | "<>" => Some(CompareOp::Ne),
            "<" => Some(CompareOp::Lt),
            "<=" => Some(CompareOp::Le),
            ">" => Some(CompareOp::Gt),
            ">=" => Some(CompareOp::Ge),
            _ => None,
        }
    }
}

/// Evaluates conditions against nodes
pub struct ConditionEvaluator {
    /// Cached compiled regexes
    regex_cache: HashMap<String, Regex>,
}

impl Default for ConditionEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

impl ConditionEvaluator {
    pub fn new() -> Self {
        Self {
            regex_cache: HashMap::new(),
        }
    }

    /// Evaluate a condition against a node
    pub fn evaluate(&mut self, condition: &Condition, node: &dyn Node) -> bool {
        match condition {
            Condition::AttributeMissing { name } => node.attribute(name).is_none(),

            Condition::AttributeEquals { name, value } => {
                node.attribute(name).is_some_and(|v| v == value)
            }

            Condition::AttributeNotEquals { name, value } => {
                node.attribute(name).is_none_or(|v| v != value)
            }

            Condition::AttributeMatches { name, pattern } => {
                if let Some(value) = node.attribute(name) {
                    self.matches_regex(pattern, value)
                } else {
                    false
                }
            }

            Condition::AttributeNotMatches { name, pattern } => {
                if let Some(value) = node.attribute(name) {
                    !self.matches_regex(pattern, value)
                } else {
                    true
                }
            }

            Condition::AttributeIn { name, values } => node
                .attribute(name)
                .is_some_and(|v| values.iter().any(|val| val == v)),

            Condition::AttributeNotIn { name, values } => node
                .attribute(name)
                .is_none_or(|v| !values.iter().any(|val| val == v)),

            Condition::AttributeExists { name } => node.attribute(name).is_some(),

            Condition::HasChild { element } => node.has_child(element),

            Condition::MissingChild { element } => !node.has_child(element),

            Condition::ChildCount { element, op, value } => {
                let count = node.count_children(element);
                op.evaluate(count, *value)
            }

            Condition::ParentIs { element } => node.parent().is_some_and(|p| p.kind() == element),

            Condition::ParentNot { element } => node.parent().is_none_or(|p| p.kind() != element),

            Condition::ParentIn { elements } => node
                .parent()
                .is_some_and(|p| elements.iter().any(|e| e == p.kind())),

            Condition::ParentNotIn { elements } => node
                .parent()
                .is_some_and(|p| !elements.iter().any(|e| e == p.kind())),

            Condition::DepthExceeds { max } => {
                let depth = self.calculate_depth(node);
                depth > *max
            }

            Condition::TextMatches { pattern } => self.matches_regex(pattern, node.text()),

            Condition::TextContains { substring } => node.text().contains(substring),

            Condition::All(conditions) => conditions.iter().all(|c| self.evaluate(c, node)),

            Condition::Any(conditions) => conditions.iter().any(|c| self.evaluate(c, node)),

            Condition::Not(condition) => !self.evaluate(condition, node),

            Condition::Always => true,

            Condition::Never => false,
        }
    }

    fn matches_regex(&mut self, pattern: &str, text: &str) -> bool {
        if let Some(regex) = self.regex_cache.get(pattern) {
            return regex.is_match(text);
        }

        match Regex::new(pattern) {
            Ok(regex) => {
                let result = regex.is_match(text);
                self.regex_cache.insert(pattern.to_string(), regex);
                result
            }
            Err(_) => false,
        }
    }

    fn calculate_depth(&self, node: &dyn Node) -> usize {
        let mut depth = 0;
        let mut current = node.parent();
        while let Some(parent) = current {
            depth += 1;
            current = parent.parent();
        }
        depth
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::types::Attribute;

    // Mock node for testing
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

        fn with_text(mut self, text: &str) -> Self {
            self.text = text.to_string();
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

    #[test]
    fn test_attribute_missing() {
        let mut evaluator = ConditionEvaluator::new();
        let node = MockNode::new("Component");

        let cond = Condition::AttributeMissing {
            name: "Id".to_string(),
        };
        assert!(evaluator.evaluate(&cond, &node));

        let node_with_id = MockNode::new("Component").with_attr("Id", "C1");
        assert!(!evaluator.evaluate(&cond, &node_with_id));
    }

    #[test]
    fn test_attribute_equals() {
        let mut evaluator = ConditionEvaluator::new();
        let node = MockNode::new("Component").with_attr("Guid", "*");

        let cond = Condition::AttributeEquals {
            name: "Guid".to_string(),
            value: "*".to_string(),
        };
        assert!(evaluator.evaluate(&cond, &node));

        let cond_other = Condition::AttributeEquals {
            name: "Guid".to_string(),
            value: "ABC".to_string(),
        };
        assert!(!evaluator.evaluate(&cond_other, &node));
    }

    #[test]
    fn test_attribute_matches() {
        let mut evaluator = ConditionEvaluator::new();
        let node = MockNode::new("File").with_attr("Source", "C:\\Program Files\\app.exe");

        let cond = Condition::AttributeMatches {
            name: "Source".to_string(),
            pattern: r"^[A-Z]:\\".to_string(),
        };
        assert!(evaluator.evaluate(&cond, &node));

        let cond_no_match = Condition::AttributeMatches {
            name: "Source".to_string(),
            pattern: r"^/usr/".to_string(),
        };
        assert!(!evaluator.evaluate(&cond_no_match, &node));
    }

    #[test]
    fn test_attribute_in() {
        let mut evaluator = ConditionEvaluator::new();
        let node = MockNode::new("Property").with_attr("Type", "string");

        let cond = Condition::AttributeIn {
            name: "Type".to_string(),
            values: vec!["string".to_string(), "integer".to_string()],
        };
        assert!(evaluator.evaluate(&cond, &node));

        let node_other = MockNode::new("Property").with_attr("Type", "binary");
        assert!(!evaluator.evaluate(&cond, &node_other));
    }

    #[test]
    fn test_has_child() {
        let mut evaluator = ConditionEvaluator::new();
        let node = MockNode::new("Component").with_child(MockNode::new("File"));

        let cond = Condition::HasChild {
            element: "File".to_string(),
        };
        assert!(evaluator.evaluate(&cond, &node));

        let cond_missing = Condition::HasChild {
            element: "RegistryKey".to_string(),
        };
        assert!(!evaluator.evaluate(&cond_missing, &node));
    }

    #[test]
    fn test_missing_child() {
        let mut evaluator = ConditionEvaluator::new();
        let node = MockNode::new("Package");

        let cond = Condition::MissingChild {
            element: "MajorUpgrade".to_string(),
        };
        assert!(evaluator.evaluate(&cond, &node));

        let node_with_upgrade = MockNode::new("Package").with_child(MockNode::new("MajorUpgrade"));
        assert!(!evaluator.evaluate(&cond, &node_with_upgrade));
    }

    #[test]
    fn test_child_count() {
        let mut evaluator = ConditionEvaluator::new();
        let node = MockNode::new("Component")
            .with_child(MockNode::new("File"))
            .with_child(MockNode::new("File"))
            .with_child(MockNode::new("RegistryKey"));

        let cond_gt = Condition::ChildCount {
            element: "File".to_string(),
            op: CompareOp::Gt,
            value: 1,
        };
        assert!(evaluator.evaluate(&cond_gt, &node));

        let cond_eq = Condition::ChildCount {
            element: "File".to_string(),
            op: CompareOp::Eq,
            value: 2,
        };
        assert!(evaluator.evaluate(&cond_eq, &node));

        let cond_lt = Condition::ChildCount {
            element: "File".to_string(),
            op: CompareOp::Lt,
            value: 2,
        };
        assert!(!evaluator.evaluate(&cond_lt, &node));
    }

    #[test]
    fn test_text_matches() {
        let mut evaluator = ConditionEvaluator::new();
        let node = MockNode::new("Condition").with_text("NETFRAMEWORK45 >= 4.5");

        let cond = Condition::TextMatches {
            pattern: r"NETFRAMEWORK\d+".to_string(),
        };
        assert!(evaluator.evaluate(&cond, &node));
    }

    #[test]
    fn test_all_conditions() {
        let mut evaluator = ConditionEvaluator::new();
        let node = MockNode::new("Component")
            .with_attr("Id", "C1")
            .with_attr("Guid", "*");

        let cond = Condition::All(vec![
            Condition::AttributeExists {
                name: "Id".to_string(),
            },
            Condition::AttributeEquals {
                name: "Guid".to_string(),
                value: "*".to_string(),
            },
        ]);
        assert!(evaluator.evaluate(&cond, &node));

        let cond_fail = Condition::All(vec![
            Condition::AttributeExists {
                name: "Id".to_string(),
            },
            Condition::AttributeEquals {
                name: "Guid".to_string(),
                value: "ABC".to_string(),
            },
        ]);
        assert!(!evaluator.evaluate(&cond_fail, &node));
    }

    #[test]
    fn test_any_conditions() {
        let mut evaluator = ConditionEvaluator::new();
        let node = MockNode::new("Component").with_attr("Guid", "*");

        let cond = Condition::Any(vec![
            Condition::AttributeExists {
                name: "Id".to_string(),
            },
            Condition::AttributeEquals {
                name: "Guid".to_string(),
                value: "*".to_string(),
            },
        ]);
        assert!(evaluator.evaluate(&cond, &node));
    }

    #[test]
    fn test_not_condition() {
        let mut evaluator = ConditionEvaluator::new();
        let node = MockNode::new("Component");

        let cond = Condition::Not(Box::new(Condition::AttributeExists {
            name: "Guid".to_string(),
        }));
        assert!(evaluator.evaluate(&cond, &node));
    }

    #[test]
    fn test_compare_op() {
        assert!(CompareOp::Eq.evaluate(5, 5));
        assert!(!CompareOp::Eq.evaluate(5, 3));
        assert!(CompareOp::Ne.evaluate(5, 3));
        assert!(CompareOp::Lt.evaluate(3, 5));
        assert!(CompareOp::Le.evaluate(5, 5));
        assert!(CompareOp::Gt.evaluate(5, 3));
        assert!(CompareOp::Ge.evaluate(5, 5));
    }

    #[test]
    fn test_compare_op_from_str() {
        assert_eq!(CompareOp::from_str("=="), Some(CompareOp::Eq));
        assert_eq!(CompareOp::from_str("="), Some(CompareOp::Eq));
        assert_eq!(CompareOp::from_str("!="), Some(CompareOp::Ne));
        assert_eq!(CompareOp::from_str("<"), Some(CompareOp::Lt));
        assert_eq!(CompareOp::from_str("<="), Some(CompareOp::Le));
        assert_eq!(CompareOp::from_str(">"), Some(CompareOp::Gt));
        assert_eq!(CompareOp::from_str(">="), Some(CompareOp::Ge));
        assert_eq!(CompareOp::from_str("invalid"), None);
    }
}
