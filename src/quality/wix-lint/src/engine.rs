//! Lint engine - orchestrates rule evaluation

use crate::config::Config;
use crate::diagnostics::{Diagnostic, Location};
use crate::parser::{ParseError, WixDocument};
use crate::rules::{ConditionEvaluator, MessageContext, Rule};
use crate::Severity;
use std::collections::HashMap;
use std::path::Path;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum LintError {
    #[error("Failed to parse file: {0}")]
    Parse(#[from] ParseError),
}

/// Statistics about lint results
#[derive(Debug, Default, Clone)]
pub struct LintStatistics {
    /// Count per rule ID
    pub per_rule: HashMap<String, usize>,
    /// Count per severity
    pub per_severity: HashMap<Severity, usize>,
    /// Total files linted
    pub files_linted: usize,
    /// Files with errors
    pub files_with_errors: usize,
}

impl LintStatistics {
    /// Record a diagnostic
    pub fn record(&mut self, diagnostic: &Diagnostic) {
        *self.per_rule.entry(diagnostic.rule_id.clone()).or_insert(0) += 1;
        *self.per_severity.entry(diagnostic.severity).or_insert(0) += 1;
    }

    /// Merge another statistics into this one
    pub fn merge(&mut self, other: &LintStatistics) {
        for (rule, count) in &other.per_rule {
            *self.per_rule.entry(rule.clone()).or_insert(0) += count;
        }
        for (severity, count) in &other.per_severity {
            *self.per_severity.entry(*severity).or_insert(0) += count;
        }
        self.files_linted += other.files_linted;
        self.files_with_errors += other.files_with_errors;
    }

    /// Get error count
    pub fn error_count(&self) -> usize {
        *self.per_severity.get(&Severity::Error).unwrap_or(&0)
    }

    /// Get warning count
    pub fn warning_count(&self) -> usize {
        *self.per_severity.get(&Severity::Warning).unwrap_or(&0)
    }

    /// Get info count
    pub fn info_count(&self) -> usize {
        *self.per_severity.get(&Severity::Info).unwrap_or(&0)
    }
}

/// The main lint engine
pub struct LintEngine {
    /// All loaded rules, grouped by target element
    rules_by_element: HashMap<String, Vec<Rule>>,
    /// Configuration
    config: Config,
}

impl LintEngine {
    /// Create a new lint engine
    pub fn new(rules: Vec<Rule>, config: Config) -> Self {
        // Group rules by target element for efficient lookup
        let mut rules_by_element: HashMap<String, Vec<Rule>> = HashMap::new();

        for rule in rules {
            // Only include enabled rules
            if config.is_rule_enabled(&rule.id) {
                rules_by_element
                    .entry(rule.element.clone())
                    .or_default()
                    .push(rule);
            }
        }

        Self {
            rules_by_element,
            config,
        }
    }

    /// Lint a file and return diagnostics
    pub fn lint_file(&self, path: &Path) -> Result<Vec<Diagnostic>, LintError> {
        // Check if file is excluded
        if self.config.is_file_excluded(path) {
            return Ok(Vec::new());
        }

        let doc = WixDocument::parse_file(path)?;
        self.lint_document(&doc, path)
    }

    /// Lint a document and return diagnostics
    pub fn lint_document(&self, doc: &WixDocument, path: &Path) -> Result<Vec<Diagnostic>, LintError> {
        let mut diagnostics = Vec::new();

        // Iterate over all elements
        for (idx, element) in doc.iter() {
            // Find rules that apply to this element type
            if let Some(rules) = self.rules_by_element.get(&element.name) {
                for rule in rules {
                    // Check per-file ignores
                    if !self.config.is_rule_enabled_for_file(&rule.id, path) {
                        continue;
                    }

                    // Check inline disable comments
                    if doc.is_rule_disabled_at_line(&rule.id, element.line) {
                        continue;
                    }

                    // Get effective severity (considering overrides)
                    let severity = self.config.get_severity(&rule.id, rule.severity);

                    // Skip if severity is below threshold
                    if !self.config.should_report(severity) {
                        continue;
                    }

                    // Evaluate the rule's condition
                    let evaluator = ConditionEvaluator::new(doc, element, idx);

                    if evaluator.evaluate(&rule.condition) {
                        // Rule triggered - create diagnostic
                        let location = Location {
                            file: path.to_path_buf(),
                            line: element.line,
                            column: element.column,
                            length: element.name.len() + 2, // <ElementName
                        };

                        // Build message context for template rendering
                        let context = MessageContext {
                            attributes: element.attributes.clone(),
                            child_count: Some(doc.count_children(element, None)),
                            ..Default::default()
                        };

                        let message = rule.render_message(&context);

                        let mut diagnostic =
                            Diagnostic::new(&rule.id, severity, &message, location);

                        // Add source line
                        if let Some(source_line) = doc.get_source_line(element.line) {
                            diagnostic = diagnostic.with_source_line(source_line);
                        }

                        // Add help text (rule description)
                        diagnostic = diagnostic.with_help(&rule.description);

                        // Add fix suggestion if available
                        if let Some(fix_desc) = rule.fix_description() {
                            if let Some(ref fix) = rule.fix {
                                let replacement = format!(
                                    "{}=\"{}\"",
                                    fix.attribute.as_deref().unwrap_or(""),
                                    fix.value.as_deref().unwrap_or("")
                                );
                                diagnostic = diagnostic.with_fix(fix_desc, replacement);
                            }
                        }

                        diagnostics.push(diagnostic);
                    }
                }
            }
        }

        // Sort diagnostics by location
        diagnostics.sort_by(|a, b| {
            a.location
                .line
                .cmp(&b.location.line)
                .then_with(|| a.location.column.cmp(&b.location.column))
        });

        Ok(diagnostics)
    }

    /// Get count of loaded rules
    pub fn rule_count(&self) -> usize {
        self.rules_by_element.values().map(|v| v.len()).sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagnostics::Location;
    use crate::Severity;

    fn make_test_rules() -> Vec<Rule> {
        vec![
            Rule {
                id: "test-missing-attr".to_string(),
                name: "Test Missing Attr".to_string(),
                description: "Test that attributes are required".to_string(),
                severity: Severity::Error,
                element: "Package".to_string(),
                condition: "!attributes.UpgradeCode".to_string(),
                message: "Package must have UpgradeCode".to_string(),
                fix: None,
                ..Default::default()
            },
            Rule {
                id: "test-multiple-files".to_string(),
                name: "Test Multiple Files".to_string(),
                description: "Warn about multiple files".to_string(),
                severity: Severity::Warning,
                element: "Component".to_string(),
                condition: "countChildren('File') > 1".to_string(),
                message: "Component has multiple files".to_string(),
                fix: None,
                ..Default::default()
            },
        ]
    }

    #[test]
    fn test_lint_missing_attribute() {
        let xml = r#"<?xml version="1.0"?>
<Wix xmlns="http://wixtoolset.org/schemas/v4/wxs">
  <Package Name="Test" Version="1.0.0" Manufacturer="Acme">
  </Package>
</Wix>"#;

        let doc = WixDocument::parse_str(xml).unwrap();
        let engine = LintEngine::new(make_test_rules(), Config::default());
        let diagnostics = engine
            .lint_document(&doc, Path::new("test.wxs"))
            .unwrap();

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].rule_id, "test-missing-attr");
        assert_eq!(diagnostics[0].severity, Severity::Error);
    }

    #[test]
    fn test_lint_multiple_files() {
        let xml = r#"<?xml version="1.0"?>
<Wix xmlns="http://wixtoolset.org/schemas/v4/wxs">
  <Package Name="Test" Version="1.0.0" Manufacturer="Acme" UpgradeCode="12345678-1234-1234-1234-123456789012">
    <Component Guid="*">
      <File Source="a.exe" />
      <File Source="b.dll" />
      <File Source="c.txt" />
    </Component>
  </Package>
</Wix>"#;

        let doc = WixDocument::parse_str(xml).unwrap();
        let engine = LintEngine::new(make_test_rules(), Config::default());
        let diagnostics = engine
            .lint_document(&doc, Path::new("test.wxs"))
            .unwrap();

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].rule_id, "test-multiple-files");
        assert_eq!(diagnostics[0].severity, Severity::Warning);
    }

    #[test]
    fn test_no_issues() {
        let xml = r#"<?xml version="1.0"?>
<Wix xmlns="http://wixtoolset.org/schemas/v4/wxs">
  <Package Name="Test" Version="1.0.0" Manufacturer="Acme" UpgradeCode="12345678-1234-1234-1234-123456789012">
    <Component Guid="*">
      <File Source="app.exe" KeyPath="yes" />
    </Component>
  </Package>
</Wix>"#;

        let doc = WixDocument::parse_str(xml).unwrap();
        let engine = LintEngine::new(make_test_rules(), Config::default());
        let diagnostics = engine
            .lint_document(&doc, Path::new("test.wxs"))
            .unwrap();

        assert!(diagnostics.is_empty());
    }

    #[test]
    fn test_inline_disable() {
        let xml = r#"<?xml version="1.0"?>
<Wix xmlns="http://wixtoolset.org/schemas/v4/wxs">
  <!-- wix-lint-disable-next-line test-missing-attr -->
  <Package Name="Test" Version="1.0.0" Manufacturer="Acme">
  </Package>
</Wix>"#;

        let doc = WixDocument::parse_str(xml).unwrap();
        let engine = LintEngine::new(make_test_rules(), Config::default());
        let diagnostics = engine
            .lint_document(&doc, Path::new("test.wxs"))
            .unwrap();

        // Rule should be disabled by inline comment
        assert!(diagnostics.is_empty());
    }

    #[test]
    fn test_statistics_record() {
        let mut stats = LintStatistics::default();
        let loc = Location::default();
        let diag1 = Diagnostic::new("rule-a", Severity::Error, "msg", loc.clone());
        let diag2 = Diagnostic::new("rule-a", Severity::Error, "msg", loc.clone());
        let diag3 = Diagnostic::new("rule-b", Severity::Warning, "msg", loc);

        stats.record(&diag1);
        stats.record(&diag2);
        stats.record(&diag3);

        assert_eq!(stats.error_count(), 2);
        assert_eq!(stats.warning_count(), 1);
        assert_eq!(stats.info_count(), 0);
        assert_eq!(*stats.per_rule.get("rule-a").unwrap(), 2);
        assert_eq!(*stats.per_rule.get("rule-b").unwrap(), 1);
    }

    #[test]
    fn test_statistics_merge() {
        let mut stats1 = LintStatistics::default();
        let mut stats2 = LintStatistics::default();

        let loc = Location::default();
        let diag1 = Diagnostic::new("rule-a", Severity::Error, "msg", loc.clone());
        let diag2 = Diagnostic::new("rule-a", Severity::Error, "msg", loc);

        stats1.record(&diag1);
        stats1.files_linted = 1;
        stats1.files_with_errors = 1;

        stats2.record(&diag2);
        stats2.files_linted = 2;
        stats2.files_with_errors = 1;

        stats1.merge(&stats2);

        assert_eq!(stats1.error_count(), 2);
        assert_eq!(stats1.files_linted, 3);
        assert_eq!(stats1.files_with_errors, 2);
        assert_eq!(*stats1.per_rule.get("rule-a").unwrap(), 2);
    }

    #[test]
    fn test_statistics_default() {
        let stats = LintStatistics::default();
        assert_eq!(stats.error_count(), 0);
        assert_eq!(stats.warning_count(), 0);
        assert_eq!(stats.info_count(), 0);
        assert_eq!(stats.files_linted, 0);
        assert_eq!(stats.files_with_errors, 0);
    }

    #[test]
    fn test_rule_count() {
        let engine = LintEngine::new(make_test_rules(), Config::default());
        assert_eq!(engine.rule_count(), 2);
    }

    #[test]
    fn test_disabled_rule_not_loaded() {
        let mut config = Config::default();
        config.disabled_rules = vec!["test-missing-attr".to_string()];

        let engine = LintEngine::new(make_test_rules(), config);
        assert_eq!(engine.rule_count(), 1); // Only the other rule should be loaded
    }

    #[test]
    fn test_severity_override() {
        let xml = r#"<?xml version="1.0"?>
<Wix xmlns="http://wixtoolset.org/schemas/v4/wxs">
  <Package Name="Test" Version="1.0.0" Manufacturer="Acme">
  </Package>
</Wix>"#;

        let mut config = Config::default();
        config.severity_overrides.insert("test-missing-attr".to_string(), Severity::Warning);

        let doc = WixDocument::parse_str(xml).unwrap();
        let engine = LintEngine::new(make_test_rules(), config);
        let diagnostics = engine.lint_document(&doc, Path::new("test.wxs")).unwrap();

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].severity, Severity::Warning); // Overridden from Error
    }

    #[test]
    fn test_min_severity_filter() {
        let xml = r#"<?xml version="1.0"?>
<Wix xmlns="http://wixtoolset.org/schemas/v4/wxs">
  <Package Name="Test" Version="1.0.0" Manufacturer="Acme" UpgradeCode="12345678-1234-1234-1234-123456789012">
    <Component Guid="*">
      <File Source="a.exe" />
      <File Source="b.dll" />
      <File Source="c.txt" />
    </Component>
  </Package>
</Wix>"#;

        let mut config = Config::default();
        config.min_severity = Severity::Error; // Only report errors

        let doc = WixDocument::parse_str(xml).unwrap();
        let engine = LintEngine::new(make_test_rules(), config);
        let diagnostics = engine.lint_document(&doc, Path::new("test.wxs")).unwrap();

        // The multiple files warning should be filtered out
        assert!(diagnostics.is_empty());
    }

    #[test]
    fn test_per_file_ignore() {
        let xml = r#"<?xml version="1.0"?>
<Wix xmlns="http://wixtoolset.org/schemas/v4/wxs">
  <Package Name="Test" Version="1.0.0" Manufacturer="Acme">
  </Package>
</Wix>"#;

        let mut config = Config::default();
        config.per_file_ignores.insert("tests/*.wxs".to_string(), vec!["test-missing-attr".to_string()]);

        let doc = WixDocument::parse_str(xml).unwrap();
        let engine = LintEngine::new(make_test_rules(), config);

        // This file matches the pattern, so the rule should be ignored
        let diagnostics = engine.lint_document(&doc, Path::new("tests/test.wxs")).unwrap();
        assert!(diagnostics.is_empty());

        // This file doesn't match, so the rule should apply
        let diagnostics2 = engine.lint_document(&doc, Path::new("src/main.wxs")).unwrap();
        assert_eq!(diagnostics2.len(), 1);
    }

    #[test]
    fn test_diagnostics_sorted_by_line() {
        let xml = r#"<?xml version="1.0"?>
<Wix xmlns="http://wixtoolset.org/schemas/v4/wxs">
  <Package Name="Test" Version="1.0.0" Manufacturer="Acme">
    <Component Guid="*">
      <File Source="a.exe" />
      <File Source="b.dll" />
      <File Source="c.txt" />
    </Component>
  </Package>
</Wix>"#;

        let doc = WixDocument::parse_str(xml).unwrap();
        let engine = LintEngine::new(make_test_rules(), Config::default());
        let diagnostics = engine.lint_document(&doc, Path::new("test.wxs")).unwrap();

        // Verify diagnostics are sorted by line
        for i in 1..diagnostics.len() {
            assert!(diagnostics[i].location.line >= diagnostics[i - 1].location.line);
        }
    }
}
