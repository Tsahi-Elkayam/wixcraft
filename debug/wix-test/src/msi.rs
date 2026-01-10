//! MSI structure testing module
//!
//! Provides test case definitions and validation for MSI installers.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Test result status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TestStatus {
    Passed,
    Failed,
    Skipped,
    Pending,
}

/// Test case type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TestType {
    /// Validate MSI structure
    Structure,
    /// Check file presence
    FilePresence,
    /// Verify registry entries
    Registry,
    /// Validate properties
    Property,
    /// Check component structure
    Component,
    /// Feature validation
    Feature,
    /// Custom action validation
    CustomAction,
    /// UI sequence validation
    UISequence,
    /// Install sequence validation
    InstallSequence,
}

/// A single test case
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestCase {
    pub name: String,
    pub description: Option<String>,
    pub test_type: TestType,
    pub assertions: Vec<Assertion>,
    pub tags: Vec<String>,
    pub enabled: bool,
}

impl TestCase {
    pub fn new(name: &str, test_type: TestType) -> Self {
        Self {
            name: name.to_string(),
            description: None,
            test_type,
            assertions: Vec::new(),
            tags: Vec::new(),
            enabled: true,
        }
    }

    pub fn with_description(mut self, desc: &str) -> Self {
        self.description = Some(desc.to_string());
        self
    }

    pub fn with_assertion(mut self, assertion: Assertion) -> Self {
        self.assertions.push(assertion);
        self
    }

    pub fn with_tag(mut self, tag: &str) -> Self {
        self.tags.push(tag.to_string());
        self
    }

    pub fn disabled(mut self) -> Self {
        self.enabled = false;
        self
    }
}

/// Test assertion types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Assertion {
    /// Assert file exists in MSI
    FileExists { path: String },
    /// Assert file has specific content
    FileContains { path: String, content: String },
    /// Assert registry key exists
    RegistryKeyExists { root: String, key: String },
    /// Assert registry value
    RegistryValue { root: String, key: String, name: String, expected: String },
    /// Assert property value
    PropertyValue { name: String, expected: String },
    /// Assert property exists
    PropertyExists { name: String },
    /// Assert component exists
    ComponentExists { id: String },
    /// Assert feature exists
    FeatureExists { id: String },
    /// Assert feature contains component
    FeatureContainsComponent { feature: String, component: String },
    /// Assert custom action exists
    CustomActionExists { id: String },
    /// Assert table exists
    TableExists { name: String },
    /// Assert row count in table
    TableRowCount { table: String, expected: usize },
    /// Assert condition
    Condition { expression: String },
}

/// Test result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestResult {
    pub test_name: String,
    pub status: TestStatus,
    pub duration_ms: u64,
    pub message: Option<String>,
    pub assertion_results: Vec<AssertionResult>,
}

/// Individual assertion result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssertionResult {
    pub assertion: String,
    pub passed: bool,
    pub message: Option<String>,
}

/// Test suite configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestSuite {
    pub name: String,
    pub msi_path: Option<PathBuf>,
    pub tests: Vec<TestCase>,
    pub setup: Option<TestSetup>,
    pub teardown: Option<TestTeardown>,
}

/// Test setup configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestSetup {
    pub commands: Vec<String>,
    pub environment: HashMap<String, String>,
}

/// Test teardown configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestTeardown {
    pub commands: Vec<String>,
    pub cleanup_paths: Vec<PathBuf>,
}

impl TestSuite {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            msi_path: None,
            tests: Vec::new(),
            setup: None,
            teardown: None,
        }
    }

    pub fn with_msi(mut self, path: PathBuf) -> Self {
        self.msi_path = Some(path);
        self
    }

    pub fn add_test(&mut self, test: TestCase) -> &mut Self {
        self.tests.push(test);
        self
    }

    pub fn test_count(&self) -> usize {
        self.tests.len()
    }

    pub fn enabled_tests(&self) -> impl Iterator<Item = &TestCase> {
        self.tests.iter().filter(|t| t.enabled)
    }

    pub fn tests_by_type(&self, test_type: TestType) -> impl Iterator<Item = &TestCase> {
        self.tests.iter().filter(move |t| t.test_type == test_type)
    }

    pub fn tests_by_tag(&self, tag: &str) -> impl Iterator<Item = &TestCase> {
        let tag = tag.to_string();
        self.tests.iter().filter(move |t| t.tags.contains(&tag))
    }
}

/// Test report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestReport {
    pub suite_name: String,
    pub total: usize,
    pub passed: usize,
    pub failed: usize,
    pub skipped: usize,
    pub duration_ms: u64,
    pub results: Vec<TestResult>,
}

impl TestReport {
    pub fn new(suite_name: &str) -> Self {
        Self {
            suite_name: suite_name.to_string(),
            total: 0,
            passed: 0,
            failed: 0,
            skipped: 0,
            duration_ms: 0,
            results: Vec::new(),
        }
    }

    pub fn add_result(&mut self, result: TestResult) {
        self.total += 1;
        self.duration_ms += result.duration_ms;
        match result.status {
            TestStatus::Passed => self.passed += 1,
            TestStatus::Failed => self.failed += 1,
            TestStatus::Skipped => self.skipped += 1,
            TestStatus::Pending => {}
        }
        self.results.push(result);
    }

    pub fn success_rate(&self) -> f64 {
        if self.total == 0 {
            return 0.0;
        }
        (self.passed as f64 / self.total as f64) * 100.0
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap()
    }

    pub fn summary(&self) -> String {
        format!(
            "Test Suite: {}\n\
             Total: {} | Passed: {} | Failed: {} | Skipped: {}\n\
             Success Rate: {:.1}%\n\
             Duration: {}ms",
            self.suite_name,
            self.total,
            self.passed,
            self.failed,
            self.skipped,
            self.success_rate(),
            self.duration_ms
        )
    }
}

/// Test builder for common patterns
pub struct TestBuilder;

impl TestBuilder {
    /// Create file presence test
    pub fn file_exists(name: &str, path: &str) -> TestCase {
        TestCase::new(name, TestType::FilePresence)
            .with_assertion(Assertion::FileExists { path: path.to_string() })
    }

    /// Create property value test
    pub fn property_equals(name: &str, prop: &str, value: &str) -> TestCase {
        TestCase::new(name, TestType::Property)
            .with_assertion(Assertion::PropertyValue {
                name: prop.to_string(),
                expected: value.to_string(),
            })
    }

    /// Create component exists test
    pub fn component_exists(name: &str, component_id: &str) -> TestCase {
        TestCase::new(name, TestType::Component)
            .with_assertion(Assertion::ComponentExists { id: component_id.to_string() })
    }

    /// Create feature exists test
    pub fn feature_exists(name: &str, feature_id: &str) -> TestCase {
        TestCase::new(name, TestType::Feature)
            .with_assertion(Assertion::FeatureExists { id: feature_id.to_string() })
    }

    /// Create registry key test
    pub fn registry_key_exists(name: &str, root: &str, key: &str) -> TestCase {
        TestCase::new(name, TestType::Registry)
            .with_assertion(Assertion::RegistryKeyExists {
                root: root.to_string(),
                key: key.to_string(),
            })
    }

    /// Create custom action exists test
    pub fn custom_action_exists(name: &str, action_id: &str) -> TestCase {
        TestCase::new(name, TestType::CustomAction)
            .with_assertion(Assertion::CustomActionExists { id: action_id.to_string() })
    }

    /// Create table exists test
    pub fn table_exists(name: &str, table: &str) -> TestCase {
        TestCase::new(name, TestType::Structure)
            .with_assertion(Assertion::TableExists { name: table.to_string() })
    }
}

/// Test suite loader
pub struct TestLoader;

impl TestLoader {
    /// Load test suite from JSON
    pub fn from_json(content: &str) -> Result<TestSuite, String> {
        serde_json::from_str(content).map_err(|e| e.to_string())
    }

    /// Save test suite to JSON
    pub fn to_json(suite: &TestSuite) -> String {
        serde_json::to_string_pretty(suite).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_test_case_creation() {
        let test = TestCase::new("test1", TestType::Structure);
        assert_eq!(test.name, "test1");
        assert!(test.enabled);
    }

    #[test]
    fn test_test_case_with_description() {
        let test = TestCase::new("test1", TestType::Structure)
            .with_description("A test");
        assert_eq!(test.description, Some("A test".to_string()));
    }

    #[test]
    fn test_test_case_with_assertion() {
        let test = TestCase::new("test1", TestType::FilePresence)
            .with_assertion(Assertion::FileExists { path: "test.exe".to_string() });
        assert_eq!(test.assertions.len(), 1);
    }

    #[test]
    fn test_test_case_with_tag() {
        let test = TestCase::new("test1", TestType::Structure)
            .with_tag("smoke")
            .with_tag("regression");
        assert_eq!(test.tags.len(), 2);
    }

    #[test]
    fn test_test_case_disabled() {
        let test = TestCase::new("test1", TestType::Structure).disabled();
        assert!(!test.enabled);
    }

    #[test]
    fn test_test_suite_creation() {
        let suite = TestSuite::new("MySuite");
        assert_eq!(suite.name, "MySuite");
        assert_eq!(suite.test_count(), 0);
    }

    #[test]
    fn test_test_suite_add_test() {
        let mut suite = TestSuite::new("MySuite");
        suite.add_test(TestCase::new("test1", TestType::Structure));
        assert_eq!(suite.test_count(), 1);
    }

    #[test]
    fn test_test_suite_with_msi() {
        let suite = TestSuite::new("MySuite")
            .with_msi(PathBuf::from("test.msi"));
        assert_eq!(suite.msi_path, Some(PathBuf::from("test.msi")));
    }

    #[test]
    fn test_enabled_tests() {
        let mut suite = TestSuite::new("MySuite");
        suite.add_test(TestCase::new("test1", TestType::Structure));
        suite.add_test(TestCase::new("test2", TestType::Structure).disabled());
        assert_eq!(suite.enabled_tests().count(), 1);
    }

    #[test]
    fn test_tests_by_type() {
        let mut suite = TestSuite::new("MySuite");
        suite.add_test(TestCase::new("test1", TestType::Structure));
        suite.add_test(TestCase::new("test2", TestType::FilePresence));
        suite.add_test(TestCase::new("test3", TestType::Structure));
        assert_eq!(suite.tests_by_type(TestType::Structure).count(), 2);
    }

    #[test]
    fn test_tests_by_tag() {
        let mut suite = TestSuite::new("MySuite");
        suite.add_test(TestCase::new("test1", TestType::Structure).with_tag("smoke"));
        suite.add_test(TestCase::new("test2", TestType::Structure).with_tag("regression"));
        suite.add_test(TestCase::new("test3", TestType::Structure).with_tag("smoke"));
        assert_eq!(suite.tests_by_tag("smoke").count(), 2);
    }

    #[test]
    fn test_test_report_creation() {
        let report = TestReport::new("MySuite");
        assert_eq!(report.total, 0);
        assert_eq!(report.passed, 0);
    }

    #[test]
    fn test_test_report_add_result() {
        let mut report = TestReport::new("MySuite");
        report.add_result(TestResult {
            test_name: "test1".to_string(),
            status: TestStatus::Passed,
            duration_ms: 100,
            message: None,
            assertion_results: vec![],
        });
        assert_eq!(report.total, 1);
        assert_eq!(report.passed, 1);
    }

    #[test]
    fn test_test_report_success_rate() {
        let mut report = TestReport::new("MySuite");
        report.add_result(TestResult {
            test_name: "test1".to_string(),
            status: TestStatus::Passed,
            duration_ms: 100,
            message: None,
            assertion_results: vec![],
        });
        report.add_result(TestResult {
            test_name: "test2".to_string(),
            status: TestStatus::Failed,
            duration_ms: 50,
            message: Some("Failed".to_string()),
            assertion_results: vec![],
        });
        assert_eq!(report.success_rate(), 50.0);
    }

    #[test]
    fn test_test_report_summary() {
        let mut report = TestReport::new("MySuite");
        report.add_result(TestResult {
            test_name: "test1".to_string(),
            status: TestStatus::Passed,
            duration_ms: 100,
            message: None,
            assertion_results: vec![],
        });
        let summary = report.summary();
        assert!(summary.contains("MySuite"));
        assert!(summary.contains("Passed: 1"));
    }

    #[test]
    fn test_builder_file_exists() {
        let test = TestBuilder::file_exists("file test", "app.exe");
        assert_eq!(test.test_type, TestType::FilePresence);
    }

    #[test]
    fn test_builder_property_equals() {
        let test = TestBuilder::property_equals("prop test", "INSTALLDIR", "C:\\App");
        assert_eq!(test.test_type, TestType::Property);
    }

    #[test]
    fn test_builder_component_exists() {
        let test = TestBuilder::component_exists("comp test", "MainComponent");
        assert_eq!(test.test_type, TestType::Component);
    }

    #[test]
    fn test_builder_feature_exists() {
        let test = TestBuilder::feature_exists("feature test", "MainFeature");
        assert_eq!(test.test_type, TestType::Feature);
    }

    #[test]
    fn test_loader_json_roundtrip() {
        let mut suite = TestSuite::new("MySuite");
        suite.add_test(TestCase::new("test1", TestType::Structure));

        let json = TestLoader::to_json(&suite);
        let loaded = TestLoader::from_json(&json).unwrap();

        assert_eq!(loaded.name, "MySuite");
        assert_eq!(loaded.test_count(), 1);
    }
}
