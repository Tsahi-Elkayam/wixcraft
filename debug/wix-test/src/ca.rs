//! Custom action unit testing module
//!
//! Unit testing framework for WiX Toolset custom actions.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Custom action test configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CATest {
    pub name: String,
    pub description: Option<String>,
    pub dll_path: PathBuf,
    pub entry_point: String,
    pub session_data: SessionData,
    pub expected_result: CAResult,
    pub timeout_ms: Option<u64>,
}

impl CATest {
    pub fn new(name: &str, dll: PathBuf, entry_point: &str) -> Self {
        Self {
            name: name.to_string(),
            description: None,
            dll_path: dll,
            entry_point: entry_point.to_string(),
            session_data: SessionData::default(),
            expected_result: CAResult::Success,
            timeout_ms: Some(30000),
        }
    }

    pub fn with_description(mut self, desc: &str) -> Self {
        self.description = Some(desc.to_string());
        self
    }

    pub fn with_property(mut self, name: &str, value: &str) -> Self {
        self.session_data.properties.insert(name.to_string(), value.to_string());
        self
    }

    pub fn with_component(mut self, id: &str, state: ComponentState) -> Self {
        self.session_data.components.insert(id.to_string(), state);
        self
    }

    pub fn with_feature(mut self, id: &str, state: FeatureState) -> Self {
        self.session_data.features.insert(id.to_string(), state);
        self
    }

    pub fn expect_success(mut self) -> Self {
        self.expected_result = CAResult::Success;
        self
    }

    pub fn expect_failure(mut self) -> Self {
        self.expected_result = CAResult::Failure;
        self
    }

    pub fn expect_skip(mut self) -> Self {
        self.expected_result = CAResult::Skip;
        self
    }

    pub fn with_timeout(mut self, ms: u64) -> Self {
        self.timeout_ms = Some(ms);
        self
    }
}

/// Mock session data for testing
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SessionData {
    pub properties: HashMap<String, String>,
    pub components: HashMap<String, ComponentState>,
    pub features: HashMap<String, FeatureState>,
    pub directories: HashMap<String, String>,
    pub install_mode: InstallMode,
    pub ui_level: UILevel,
}

impl SessionData {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_install_dir(mut self, dir: &str) -> Self {
        self.properties.insert("INSTALLDIR".to_string(), dir.to_string());
        self.properties.insert("INSTALLFOLDER".to_string(), dir.to_string());
        self
    }

    pub fn with_product_info(mut self, name: &str, version: &str, manufacturer: &str) -> Self {
        self.properties.insert("ProductName".to_string(), name.to_string());
        self.properties.insert("ProductVersion".to_string(), version.to_string());
        self.properties.insert("Manufacturer".to_string(), manufacturer.to_string());
        self
    }

    pub fn install_mode(mut self, mode: InstallMode) -> Self {
        self.install_mode = mode;
        self
    }

    pub fn ui_level(mut self, level: UILevel) -> Self {
        self.ui_level = level;
        self
    }
}

/// Component installation state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ComponentState {
    Local,
    Source,
    Absent,
    Unknown,
}

/// Feature installation state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FeatureState {
    Local,
    Source,
    Absent,
    Advertise,
    Unknown,
}

/// Installation mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum InstallMode {
    #[default]
    Install,
    Repair,
    Remove,
    Modify,
}

/// UI Level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum UILevel {
    None,
    Basic,
    Reduced,
    #[default]
    Full,
}

/// Expected custom action result
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CAResult {
    Success,
    Failure,
    Skip,
    Cancel,
    Suspend,
    Retry,
}

impl CAResult {
    pub fn from_code(code: i32) -> Self {
        match code {
            0 => CAResult::Failure,
            1 => CAResult::Success,
            2 => CAResult::Cancel,
            3 => CAResult::Suspend,
            4 => CAResult::Retry,
            259 => CAResult::Skip,
            _ => CAResult::Failure,
        }
    }

    pub fn to_code(&self) -> i32 {
        match self {
            CAResult::Failure => 0,
            CAResult::Success => 1,
            CAResult::Cancel => 2,
            CAResult::Suspend => 3,
            CAResult::Retry => 4,
            CAResult::Skip => 259,
        }
    }
}

/// Test result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CATestResult {
    pub test_name: String,
    pub passed: bool,
    pub actual_result: Option<CAResult>,
    pub expected_result: CAResult,
    pub duration_ms: u64,
    pub error_message: Option<String>,
    pub log_output: Vec<String>,
    pub property_changes: HashMap<String, String>,
}

impl CATestResult {
    pub fn success(test_name: &str, duration_ms: u64) -> Self {
        Self {
            test_name: test_name.to_string(),
            passed: true,
            actual_result: Some(CAResult::Success),
            expected_result: CAResult::Success,
            duration_ms,
            error_message: None,
            log_output: Vec::new(),
            property_changes: HashMap::new(),
        }
    }

    pub fn failure(test_name: &str, error: &str, duration_ms: u64) -> Self {
        Self {
            test_name: test_name.to_string(),
            passed: false,
            actual_result: None,
            expected_result: CAResult::Success,
            duration_ms,
            error_message: Some(error.to_string()),
            log_output: Vec::new(),
            property_changes: HashMap::new(),
        }
    }
}

/// Test suite for custom actions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CATestSuite {
    pub name: String,
    pub tests: Vec<CATest>,
    pub default_session: SessionData,
}

impl CATestSuite {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            tests: Vec::new(),
            default_session: SessionData::default(),
        }
    }

    pub fn with_default_session(mut self, session: SessionData) -> Self {
        self.default_session = session;
        self
    }

    pub fn add_test(&mut self, test: CATest) -> &mut Self {
        self.tests.push(test);
        self
    }

    pub fn test_count(&self) -> usize {
        self.tests.len()
    }
}

/// Test report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CATestReport {
    pub suite_name: String,
    pub total: usize,
    pub passed: usize,
    pub failed: usize,
    pub duration_ms: u64,
    pub results: Vec<CATestResult>,
}

impl CATestReport {
    pub fn new(suite_name: &str) -> Self {
        Self {
            suite_name: suite_name.to_string(),
            total: 0,
            passed: 0,
            failed: 0,
            duration_ms: 0,
            results: Vec::new(),
        }
    }

    pub fn add_result(&mut self, result: CATestResult) {
        self.total += 1;
        self.duration_ms += result.duration_ms;
        if result.passed {
            self.passed += 1;
        } else {
            self.failed += 1;
        }
        self.results.push(result);
    }

    pub fn all_passed(&self) -> bool {
        self.failed == 0
    }

    pub fn success_rate(&self) -> f64 {
        if self.total == 0 {
            return 0.0;
        }
        (self.passed as f64 / self.total as f64) * 100.0
    }

    pub fn summary(&self) -> String {
        format!(
            "CA Test Suite: {}\n\
             Total: {} | Passed: {} | Failed: {}\n\
             Success Rate: {:.1}%\n\
             Duration: {}ms",
            self.suite_name,
            self.total,
            self.passed,
            self.failed,
            self.success_rate(),
            self.duration_ms
        )
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap()
    }
}

/// Assertion helpers for custom action testing
pub struct CAAssert;

impl CAAssert {
    pub fn property_equals(session: &SessionData, name: &str, expected: &str) -> bool {
        session.properties.get(name).map(|v| v == expected).unwrap_or(false)
    }

    pub fn property_exists(session: &SessionData, name: &str) -> bool {
        session.properties.contains_key(name)
    }

    pub fn property_not_empty(session: &SessionData, name: &str) -> bool {
        session.properties.get(name).map(|v| !v.is_empty()).unwrap_or(false)
    }

    pub fn component_state(session: &SessionData, id: &str, expected: ComponentState) -> bool {
        session.components.get(id).map(|s| *s == expected).unwrap_or(false)
    }

    pub fn feature_state(session: &SessionData, id: &str, expected: FeatureState) -> bool {
        session.features.get(id).map(|s| *s == expected).unwrap_or(false)
    }

    pub fn is_install(session: &SessionData) -> bool {
        session.install_mode == InstallMode::Install
    }

    pub fn is_remove(session: &SessionData) -> bool {
        session.install_mode == InstallMode::Remove
    }

    pub fn is_repair(session: &SessionData) -> bool {
        session.install_mode == InstallMode::Repair
    }
}

/// Test data generators
pub struct CATestData;

impl CATestData {
    /// Create standard install session
    pub fn install_session() -> SessionData {
        SessionData::new()
            .install_mode(InstallMode::Install)
            .ui_level(UILevel::Full)
            .with_install_dir("C:\\Program Files\\TestApp")
            .with_product_info("Test Application", "1.0.0", "Test Corp")
    }

    /// Create standard repair session
    pub fn repair_session() -> SessionData {
        SessionData::new()
            .install_mode(InstallMode::Repair)
            .ui_level(UILevel::Basic)
            .with_install_dir("C:\\Program Files\\TestApp")
            .with_product_info("Test Application", "1.0.0", "Test Corp")
    }

    /// Create standard remove session
    pub fn remove_session() -> SessionData {
        SessionData::new()
            .install_mode(InstallMode::Remove)
            .ui_level(UILevel::None)
            .with_install_dir("C:\\Program Files\\TestApp")
            .with_product_info("Test Application", "1.0.0", "Test Corp")
    }

    /// Create silent install session
    pub fn silent_session() -> SessionData {
        SessionData::new()
            .install_mode(InstallMode::Install)
            .ui_level(UILevel::None)
            .with_install_dir("C:\\Program Files\\TestApp")
            .with_product_info("Test Application", "1.0.0", "Test Corp")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ca_test_creation() {
        let test = CATest::new("test1", PathBuf::from("test.dll"), "CustomAction1");
        assert_eq!(test.name, "test1");
        assert_eq!(test.entry_point, "CustomAction1");
    }

    #[test]
    fn test_ca_test_with_property() {
        let test = CATest::new("test1", PathBuf::from("test.dll"), "CA1")
            .with_property("PROP1", "value1");
        assert_eq!(test.session_data.properties.get("PROP1"), Some(&"value1".to_string()));
    }

    #[test]
    fn test_ca_test_with_component() {
        let test = CATest::new("test1", PathBuf::from("test.dll"), "CA1")
            .with_component("Comp1", ComponentState::Local);
        assert_eq!(test.session_data.components.get("Comp1"), Some(&ComponentState::Local));
    }

    #[test]
    fn test_ca_test_expect_success() {
        let test = CATest::new("test1", PathBuf::from("test.dll"), "CA1")
            .expect_success();
        assert_eq!(test.expected_result, CAResult::Success);
    }

    #[test]
    fn test_ca_test_expect_failure() {
        let test = CATest::new("test1", PathBuf::from("test.dll"), "CA1")
            .expect_failure();
        assert_eq!(test.expected_result, CAResult::Failure);
    }

    #[test]
    fn test_session_data_new() {
        let session = SessionData::new();
        assert!(session.properties.is_empty());
    }

    #[test]
    fn test_session_data_with_install_dir() {
        let session = SessionData::new().with_install_dir("C:\\Test");
        assert_eq!(session.properties.get("INSTALLDIR"), Some(&"C:\\Test".to_string()));
    }

    #[test]
    fn test_session_data_with_product_info() {
        let session = SessionData::new().with_product_info("App", "1.0", "Corp");
        assert_eq!(session.properties.get("ProductName"), Some(&"App".to_string()));
        assert_eq!(session.properties.get("ProductVersion"), Some(&"1.0".to_string()));
    }

    #[test]
    fn test_ca_result_from_code() {
        assert_eq!(CAResult::from_code(1), CAResult::Success);
        assert_eq!(CAResult::from_code(0), CAResult::Failure);
        assert_eq!(CAResult::from_code(259), CAResult::Skip);
    }

    #[test]
    fn test_ca_result_to_code() {
        assert_eq!(CAResult::Success.to_code(), 1);
        assert_eq!(CAResult::Failure.to_code(), 0);
        assert_eq!(CAResult::Skip.to_code(), 259);
    }

    #[test]
    fn test_ca_test_result_success() {
        let result = CATestResult::success("test1", 100);
        assert!(result.passed);
        assert_eq!(result.duration_ms, 100);
    }

    #[test]
    fn test_ca_test_result_failure() {
        let result = CATestResult::failure("test1", "Error occurred", 50);
        assert!(!result.passed);
        assert_eq!(result.error_message, Some("Error occurred".to_string()));
    }

    #[test]
    fn test_ca_test_suite_creation() {
        let suite = CATestSuite::new("MySuite");
        assert_eq!(suite.name, "MySuite");
        assert_eq!(suite.test_count(), 0);
    }

    #[test]
    fn test_ca_test_suite_add_test() {
        let mut suite = CATestSuite::new("MySuite");
        suite.add_test(CATest::new("test1", PathBuf::from("test.dll"), "CA1"));
        assert_eq!(suite.test_count(), 1);
    }

    #[test]
    fn test_ca_test_report_creation() {
        let report = CATestReport::new("MySuite");
        assert_eq!(report.total, 0);
        assert!(report.all_passed());
    }

    #[test]
    fn test_ca_test_report_add_result() {
        let mut report = CATestReport::new("MySuite");
        report.add_result(CATestResult::success("test1", 100));
        assert_eq!(report.total, 1);
        assert_eq!(report.passed, 1);
    }

    #[test]
    fn test_ca_test_report_all_passed() {
        let mut report = CATestReport::new("MySuite");
        report.add_result(CATestResult::success("test1", 100));
        report.add_result(CATestResult::success("test2", 100));
        assert!(report.all_passed());
    }

    #[test]
    fn test_ca_test_report_with_failure() {
        let mut report = CATestReport::new("MySuite");
        report.add_result(CATestResult::success("test1", 100));
        report.add_result(CATestResult::failure("test2", "Error", 50));
        assert!(!report.all_passed());
        assert_eq!(report.failed, 1);
    }

    #[test]
    fn test_ca_test_report_success_rate() {
        let mut report = CATestReport::new("MySuite");
        report.add_result(CATestResult::success("test1", 100));
        report.add_result(CATestResult::failure("test2", "Error", 50));
        assert_eq!(report.success_rate(), 50.0);
    }

    #[test]
    fn test_ca_assert_property_equals() {
        let session = SessionData::new().with_install_dir("C:\\Test");
        assert!(CAAssert::property_equals(&session, "INSTALLDIR", "C:\\Test"));
        assert!(!CAAssert::property_equals(&session, "INSTALLDIR", "C:\\Other"));
    }

    #[test]
    fn test_ca_assert_property_exists() {
        let session = SessionData::new().with_install_dir("C:\\Test");
        assert!(CAAssert::property_exists(&session, "INSTALLDIR"));
        assert!(!CAAssert::property_exists(&session, "NONEXISTENT"));
    }

    #[test]
    fn test_ca_assert_install_mode() {
        let install = SessionData::new().install_mode(InstallMode::Install);
        let remove = SessionData::new().install_mode(InstallMode::Remove);

        assert!(CAAssert::is_install(&install));
        assert!(!CAAssert::is_install(&remove));
        assert!(CAAssert::is_remove(&remove));
    }

    #[test]
    fn test_ca_test_data_install_session() {
        let session = CATestData::install_session();
        assert_eq!(session.install_mode, InstallMode::Install);
        assert!(session.properties.contains_key("ProductName"));
    }

    #[test]
    fn test_ca_test_data_remove_session() {
        let session = CATestData::remove_session();
        assert_eq!(session.install_mode, InstallMode::Remove);
    }

    #[test]
    fn test_ca_test_data_silent_session() {
        let session = CATestData::silent_session();
        assert_eq!(session.ui_level, UILevel::None);
    }
}
