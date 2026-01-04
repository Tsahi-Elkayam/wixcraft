//! Rule definitions and evaluation

mod evaluator;
mod helpers;

pub use evaluator::ConditionEvaluator;
pub use helpers::Helpers;

use crate::Severity;

/// A lint rule definition
#[derive(Debug, Clone)]
pub struct Rule {
    /// Unique rule identifier
    pub id: String,
    /// Human-readable name
    pub name: String,
    /// Detailed description
    pub description: String,
    /// Severity level
    pub severity: Severity,
    /// Target element type (e.g., "Package", "Component")
    pub element: String,
    /// Condition expression that triggers the rule
    pub condition: String,
    /// User-facing message template
    pub message: String,
    /// Optional fix suggestion
    pub fix: Option<FixTemplate>,
    /// Version when this rule was added (e.g., "1.0.0")
    pub since: Option<String>,
    /// Whether this rule is deprecated
    pub deprecated: bool,
    /// Message explaining why the rule is deprecated
    pub deprecated_message: Option<String>,
    /// Rule ID that replaces this deprecated rule
    pub replaced_by: Option<String>,
}

impl Default for Rule {
    fn default() -> Self {
        Self {
            id: String::new(),
            name: String::new(),
            description: String::new(),
            severity: Severity::Info,
            element: String::new(),
            condition: String::new(),
            message: String::new(),
            fix: None,
            since: None,
            deprecated: false,
            deprecated_message: None,
            replaced_by: None,
        }
    }
}

/// A fix suggestion template
#[derive(Debug, Clone)]
pub struct FixTemplate {
    /// Action type (e.g., "addAttribute")
    pub action: String,
    /// Target attribute name
    pub attribute: Option<String>,
    /// Suggested value
    pub value: Option<String>,
}

impl Rule {
    /// Check if this rule applies to a given element type
    pub fn applies_to(&self, element_name: &str) -> bool {
        self.element.eq_ignore_ascii_case(element_name)
    }

    /// Render the message template with actual values
    pub fn render_message(&self, context: &MessageContext) -> String {
        let mut message = self.message.clone();

        // Replace {{attributes.X}} patterns
        for (key, value) in &context.attributes {
            let pattern = format!("{{{{attributes.{}}}}}", key);
            message = message.replace(&pattern, value);
        }

        // Replace {{countChildren('X')}} patterns
        if let Some(count) = context.child_count {
            // Match patterns like {{countChildren('File')}} or {{countChildren("File")}}
            let re = regex::Regex::new(r#"\{\{countChildren\(['"](\w+)['"]\)\}\}"#).unwrap();
            message = re.replace_all(&message, count.to_string().as_str()).to_string();
        }

        // Replace {{getDepth()}}
        if let Some(depth) = context.depth {
            message = message.replace("{{getDepth()}}", &depth.to_string());
        }

        message
    }

    /// Generate fix description if available
    pub fn fix_description(&self) -> Option<String> {
        self.fix.as_ref().map(|f| {
            match f.action.as_str() {
                "addAttribute" => {
                    let attr = f.attribute.as_deref().unwrap_or("attribute");
                    let val = f.value.as_deref().unwrap_or("value");
                    format!("Add {}=\"{}\"", attr, val)
                }
                _ => format!("Apply fix: {}", f.action),
            }
        })
    }
}

/// Context for rendering rule messages
#[derive(Debug, Default)]
pub struct MessageContext {
    pub attributes: std::collections::HashMap<String, String>,
    pub child_count: Option<usize>,
    pub depth: Option<usize>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn make_test_rule() -> Rule {
        Rule {
            id: "test-rule".to_string(),
            name: "Test Rule".to_string(),
            description: "A test rule description".to_string(),
            severity: Severity::Error,
            element: "Package".to_string(),
            condition: "!attributes.Name".to_string(),
            message: "Package needs a name".to_string(),
            fix: None,
            ..Default::default()
        }
    }

    #[test]
    fn test_rule_applies_to() {
        let rule = make_test_rule();
        assert!(rule.applies_to("Package"));
        assert!(rule.applies_to("package")); // Case insensitive
        assert!(rule.applies_to("PACKAGE")); // Case insensitive
        assert!(!rule.applies_to("Component"));
    }

    #[test]
    fn test_render_message_simple() {
        let rule = make_test_rule();
        let context = MessageContext::default();
        let message = rule.render_message(&context);
        assert_eq!(message, "Package needs a name");
    }

    #[test]
    fn test_render_message_with_attribute() {
        let rule = Rule {
            id: "test".to_string(),
            name: "Test".to_string(),
            description: "Test".to_string(),
            severity: Severity::Warning,
            element: "Component".to_string(),
            condition: "true".to_string(),
            message: "Component '{{attributes.Id}}' has issues".to_string(),
            fix: None,
            ..Default::default()
        };

        let mut attrs = HashMap::new();
        attrs.insert("Id".to_string(), "MyComponent".to_string());

        let context = MessageContext {
            attributes: attrs,
            child_count: None,
            depth: None,
        };

        let message = rule.render_message(&context);
        assert_eq!(message, "Component 'MyComponent' has issues");
    }

    #[test]
    fn test_render_message_with_child_count() {
        let rule = Rule {
            id: "test".to_string(),
            name: "Test".to_string(),
            description: "Test".to_string(),
            severity: Severity::Warning,
            element: "Component".to_string(),
            condition: "true".to_string(),
            message: "Component has {{countChildren('File')}} files".to_string(),
            fix: None,
            ..Default::default()
        };

        let context = MessageContext {
            attributes: HashMap::new(),
            child_count: Some(5),
            depth: None,
        };

        let message = rule.render_message(&context);
        assert_eq!(message, "Component has 5 files");
    }

    #[test]
    fn test_render_message_with_depth() {
        let rule = Rule {
            id: "test".to_string(),
            name: "Test".to_string(),
            description: "Test".to_string(),
            severity: Severity::Info,
            element: "Directory".to_string(),
            condition: "true".to_string(),
            message: "Directory at depth {{getDepth()}}".to_string(),
            fix: None,
            ..Default::default()
        };

        let context = MessageContext {
            attributes: HashMap::new(),
            child_count: None,
            depth: Some(3),
        };

        let message = rule.render_message(&context);
        assert_eq!(message, "Directory at depth 3");
    }

    #[test]
    fn test_fix_description_add_attribute() {
        let rule = Rule {
            id: "test".to_string(),
            name: "Test".to_string(),
            description: "Test".to_string(),
            severity: Severity::Error,
            element: "Component".to_string(),
            condition: "!attributes.Guid".to_string(),
            message: "Missing Guid".to_string(),
            fix: Some(FixTemplate {
                action: "addAttribute".to_string(),
                attribute: Some("Guid".to_string()),
                value: Some("*".to_string()),
            }),
            ..Default::default()
        };

        let desc = rule.fix_description();
        assert!(desc.is_some());
        assert_eq!(desc.unwrap(), "Add Guid=\"*\"");
    }

    #[test]
    fn test_fix_description_other_action() {
        let rule = Rule {
            id: "test".to_string(),
            name: "Test".to_string(),
            description: "Test".to_string(),
            severity: Severity::Warning,
            element: "File".to_string(),
            condition: "true".to_string(),
            message: "Issue".to_string(),
            fix: Some(FixTemplate {
                action: "removeElement".to_string(),
                attribute: None,
                value: None,
            }),
            ..Default::default()
        };

        let desc = rule.fix_description();
        assert!(desc.is_some());
        assert_eq!(desc.unwrap(), "Apply fix: removeElement");
    }

    #[test]
    fn test_fix_description_none() {
        let rule = make_test_rule();
        assert!(rule.fix_description().is_none());
    }

    #[test]
    fn test_message_context_default() {
        let ctx = MessageContext::default();
        assert!(ctx.attributes.is_empty());
        assert!(ctx.child_count.is_none());
        assert!(ctx.depth.is_none());
    }

    #[test]
    fn test_fix_template_clone() {
        let fix = FixTemplate {
            action: "addAttribute".to_string(),
            attribute: Some("Guid".to_string()),
            value: Some("*".to_string()),
        };

        let cloned = fix.clone();
        assert_eq!(cloned.action, "addAttribute");
        assert_eq!(cloned.attribute, Some("Guid".to_string()));
        assert_eq!(cloned.value, Some("*".to_string()));
    }

    #[test]
    fn test_rule_clone() {
        let rule = make_test_rule();
        let cloned = rule.clone();

        assert_eq!(cloned.id, rule.id);
        assert_eq!(cloned.name, rule.name);
        assert_eq!(cloned.description, rule.description);
        assert_eq!(cloned.severity, rule.severity);
        assert_eq!(cloned.element, rule.element);
        assert_eq!(cloned.condition, rule.condition);
        assert_eq!(cloned.message, rule.message);
    }

    #[test]
    fn test_render_multiple_attributes() {
        let rule = Rule {
            id: "test".to_string(),
            name: "Test".to_string(),
            description: "Test".to_string(),
            severity: Severity::Warning,
            element: "File".to_string(),
            condition: "true".to_string(),
            message: "File '{{attributes.Source}}' in '{{attributes.Id}}'".to_string(),
            fix: None,
            ..Default::default()
        };

        let mut attrs = HashMap::new();
        attrs.insert("Source".to_string(), "app.exe".to_string());
        attrs.insert("Id".to_string(), "MainFile".to_string());

        let context = MessageContext {
            attributes: attrs,
            child_count: None,
            depth: None,
        };

        let message = rule.render_message(&context);
        assert_eq!(message, "File 'app.exe' in 'MainFile'");
    }

    #[test]
    fn test_render_missing_attribute() {
        let rule = Rule {
            id: "test".to_string(),
            name: "Test".to_string(),
            description: "Test".to_string(),
            severity: Severity::Warning,
            element: "File".to_string(),
            condition: "true".to_string(),
            message: "File '{{attributes.Missing}}'".to_string(),
            fix: None,
            ..Default::default()
        };

        let context = MessageContext::default();
        let message = rule.render_message(&context);

        // Missing attribute placeholder remains unchanged
        assert_eq!(message, "File '{{attributes.Missing}}'");
    }
}
