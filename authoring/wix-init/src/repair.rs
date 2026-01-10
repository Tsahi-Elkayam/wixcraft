//! wix-repair - MSI repair and validation tool
//!
//! Validates MSI integrity and repairs installations.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Repair mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RepairMode {
    /// Repair files only
    Files,
    /// Repair registry only
    Registry,
    /// Repair shortcuts only
    Shortcuts,
    /// Full repair (all components)
    Full,
    /// Reinstall all features
    Reinstall,
}

impl RepairMode {
    pub fn to_msiexec_flag(&self) -> &'static str {
        match self {
            RepairMode::Files => "p",
            RepairMode::Registry => "m",
            RepairMode::Shortcuts => "s",
            RepairMode::Full => "omus",
            RepairMode::Reinstall => "vomus",
        }
    }
}

/// Validation issue severity
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IssueSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

/// Validation issue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationIssue {
    pub severity: IssueSeverity,
    pub category: String,
    pub message: String,
    pub path: Option<String>,
    pub suggestion: Option<String>,
}

impl ValidationIssue {
    pub fn new(severity: IssueSeverity, category: &str, message: &str) -> Self {
        Self {
            severity,
            category: category.to_string(),
            message: message.to_string(),
            path: None,
            suggestion: None,
        }
    }

    pub fn with_path(mut self, path: &str) -> Self {
        self.path = Some(path.to_string());
        self
    }

    pub fn with_suggestion(mut self, suggestion: &str) -> Self {
        self.suggestion = Some(suggestion.to_string());
        self
    }
}

/// Component status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentStatus {
    pub component_id: String,
    pub installed: bool,
    pub file_missing: bool,
    pub registry_missing: bool,
    pub needs_repair: bool,
}

impl ComponentStatus {
    pub fn new(component_id: &str) -> Self {
        Self {
            component_id: component_id.to_string(),
            installed: true,
            file_missing: false,
            registry_missing: false,
            needs_repair: false,
        }
    }

    pub fn mark_file_missing(&mut self) {
        self.file_missing = true;
        self.needs_repair = true;
    }

    pub fn mark_registry_missing(&mut self) {
        self.registry_missing = true;
        self.needs_repair = true;
    }
}

/// MSI validator
#[derive(Debug, Clone, Default)]
pub struct MsiValidator {
    issues: Vec<ValidationIssue>,
    components: HashMap<String, ComponentStatus>,
}

impl MsiValidator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn validate_msi(&mut self, _msi_path: &PathBuf) -> &[ValidationIssue] {
        // Simulate validation
        self.issues.clear();
        &self.issues
    }

    pub fn validate_installation(&mut self, product_code: &str) -> &[ValidationIssue] {
        self.issues.clear();
        // Simulate checking installation
        self.issues.push(
            ValidationIssue::new(
                IssueSeverity::Info,
                "Installation",
                &format!("Product {} is installed", product_code),
            )
        );
        &self.issues
    }

    pub fn check_component(&mut self, component_id: &str, file_path: &PathBuf) -> &ComponentStatus {
        let status = self
            .components
            .entry(component_id.to_string())
            .or_insert_with(|| ComponentStatus::new(component_id));

        if !file_path.exists() {
            status.mark_file_missing();
            self.issues.push(
                ValidationIssue::new(
                    IssueSeverity::Error,
                    "File",
                    "Missing file",
                )
                .with_path(&file_path.to_string_lossy())
                .with_suggestion("Run repair to restore missing files"),
            );
        }

        self.components.get(component_id).unwrap()
    }

    pub fn get_issues(&self) -> &[ValidationIssue] {
        &self.issues
    }

    pub fn has_errors(&self) -> bool {
        self.issues
            .iter()
            .any(|i| matches!(i.severity, IssueSeverity::Error | IssueSeverity::Critical))
    }

    pub fn needs_repair(&self) -> bool {
        self.components.values().any(|c| c.needs_repair)
    }
}

/// Repair options
#[derive(Debug, Clone)]
pub struct RepairOptions {
    pub product_code: Option<String>,
    pub msi_path: Option<PathBuf>,
    pub mode: RepairMode,
    pub silent: bool,
    pub log_path: Option<PathBuf>,
}

impl RepairOptions {
    pub fn new(mode: RepairMode) -> Self {
        Self {
            product_code: None,
            msi_path: None,
            mode,
            silent: false,
            log_path: None,
        }
    }

    pub fn by_product_code(mut self, code: &str) -> Self {
        self.product_code = Some(code.to_string());
        self
    }

    pub fn by_msi(mut self, path: PathBuf) -> Self {
        self.msi_path = Some(path);
        self
    }

    pub fn silent(mut self) -> Self {
        self.silent = true;
        self
    }

    pub fn with_log(mut self, path: PathBuf) -> Self {
        self.log_path = Some(path);
        self
    }
}

/// Repair command builder
pub struct RepairCommand;

impl RepairCommand {
    /// Build msiexec repair command
    pub fn build(options: &RepairOptions) -> Vec<String> {
        let mut args = vec![format!("/f{}", options.mode.to_msiexec_flag())];

        if let Some(ref code) = options.product_code {
            args.push(code.clone());
        } else if let Some(ref path) = options.msi_path {
            args.push(path.to_string_lossy().to_string());
        }

        if options.silent {
            args.push("/qn".to_string());
        }

        if let Some(ref log_path) = options.log_path {
            args.push(format!("/l*v \"{}\"", log_path.display()));
        }

        args
    }

    /// Build command as string
    pub fn build_string(options: &RepairOptions) -> String {
        format!("msiexec {}", Self::build(options).join(" "))
    }
}

/// Repair result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepairResult {
    pub success: bool,
    pub components_repaired: u32,
    pub files_restored: u32,
    pub registry_fixed: u32,
    pub error_message: Option<String>,
}

impl RepairResult {
    pub fn success(components: u32, files: u32, registry: u32) -> Self {
        Self {
            success: true,
            components_repaired: components,
            files_restored: files,
            registry_fixed: registry,
            error_message: None,
        }
    }

    pub fn failure(error: &str) -> Self {
        Self {
            success: false,
            components_repaired: 0,
            files_restored: 0,
            registry_fixed: 0,
            error_message: Some(error.to_string()),
        }
    }
}

/// Validation report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationReport {
    pub product_code: String,
    pub issues: Vec<ValidationIssue>,
    pub error_count: usize,
    pub warning_count: usize,
    pub needs_repair: bool,
}

impl ValidationReport {
    pub fn generate(product_code: &str, validator: &MsiValidator) -> Self {
        let issues = validator.get_issues().to_vec();
        let error_count = issues
            .iter()
            .filter(|i| matches!(i.severity, IssueSeverity::Error | IssueSeverity::Critical))
            .count();
        let warning_count = issues
            .iter()
            .filter(|i| matches!(i.severity, IssueSeverity::Warning))
            .count();

        Self {
            product_code: product_code.to_string(),
            issues,
            error_count,
            warning_count,
            needs_repair: validator.needs_repair(),
        }
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_repair_mode_to_flag() {
        assert_eq!(RepairMode::Files.to_msiexec_flag(), "p");
        assert_eq!(RepairMode::Full.to_msiexec_flag(), "omus");
    }

    #[test]
    fn test_validation_issue_new() {
        let issue = ValidationIssue::new(IssueSeverity::Error, "File", "Missing file");
        assert_eq!(issue.severity, IssueSeverity::Error);
        assert_eq!(issue.category, "File");
    }

    #[test]
    fn test_validation_issue_with_path() {
        let issue = ValidationIssue::new(IssueSeverity::Error, "File", "Missing")
            .with_path("C:\\file.txt");
        assert_eq!(issue.path, Some("C:\\file.txt".to_string()));
    }

    #[test]
    fn test_validation_issue_with_suggestion() {
        let issue = ValidationIssue::new(IssueSeverity::Warning, "Registry", "Missing key")
            .with_suggestion("Run repair");
        assert_eq!(issue.suggestion, Some("Run repair".to_string()));
    }

    #[test]
    fn test_component_status_new() {
        let status = ComponentStatus::new("Component1");
        assert!(status.installed);
        assert!(!status.needs_repair);
    }

    #[test]
    fn test_component_status_mark_file_missing() {
        let mut status = ComponentStatus::new("Component1");
        status.mark_file_missing();
        assert!(status.file_missing);
        assert!(status.needs_repair);
    }

    #[test]
    fn test_component_status_mark_registry_missing() {
        let mut status = ComponentStatus::new("Component1");
        status.mark_registry_missing();
        assert!(status.registry_missing);
        assert!(status.needs_repair);
    }

    #[test]
    fn test_msi_validator_new() {
        let validator = MsiValidator::new();
        assert!(validator.get_issues().is_empty());
    }

    #[test]
    fn test_msi_validator_validate_installation() {
        let mut validator = MsiValidator::new();
        validator.validate_installation("{CODE}");
        assert!(!validator.get_issues().is_empty());
    }

    #[test]
    fn test_msi_validator_has_errors() {
        let mut validator = MsiValidator::new();
        validator.issues.push(ValidationIssue::new(
            IssueSeverity::Error,
            "File",
            "Error",
        ));
        assert!(validator.has_errors());
    }

    #[test]
    fn test_msi_validator_needs_repair() {
        let mut validator = MsiValidator::new();
        let mut status = ComponentStatus::new("C1");
        status.mark_file_missing();
        validator.components.insert("C1".to_string(), status);
        assert!(validator.needs_repair());
    }

    #[test]
    fn test_repair_options_new() {
        let opts = RepairOptions::new(RepairMode::Full);
        assert_eq!(opts.mode, RepairMode::Full);
        assert!(!opts.silent);
    }

    #[test]
    fn test_repair_options_by_product_code() {
        let opts = RepairOptions::new(RepairMode::Full).by_product_code("{CODE}");
        assert_eq!(opts.product_code, Some("{CODE}".to_string()));
    }

    #[test]
    fn test_repair_options_by_msi() {
        let opts = RepairOptions::new(RepairMode::Files).by_msi(PathBuf::from("app.msi"));
        assert!(opts.msi_path.is_some());
    }

    #[test]
    fn test_repair_options_silent() {
        let opts = RepairOptions::new(RepairMode::Full).silent();
        assert!(opts.silent);
    }

    #[test]
    fn test_repair_command_build() {
        let opts = RepairOptions::new(RepairMode::Full).by_product_code("{CODE}");
        let cmd = RepairCommand::build(&opts);
        assert!(cmd.contains(&"/fomus".to_string()));
        assert!(cmd.contains(&"{CODE}".to_string()));
    }

    #[test]
    fn test_repair_command_build_string() {
        let opts = RepairOptions::new(RepairMode::Full)
            .by_product_code("{CODE}")
            .silent();
        let cmd = RepairCommand::build_string(&opts);
        assert!(cmd.starts_with("msiexec"));
        assert!(cmd.contains("/qn"));
    }

    #[test]
    fn test_repair_result_success() {
        let result = RepairResult::success(5, 10, 2);
        assert!(result.success);
        assert_eq!(result.components_repaired, 5);
    }

    #[test]
    fn test_repair_result_failure() {
        let result = RepairResult::failure("Access denied");
        assert!(!result.success);
        assert_eq!(result.error_message, Some("Access denied".to_string()));
    }

    #[test]
    fn test_validation_report_generate() {
        let validator = MsiValidator::new();
        let report = ValidationReport::generate("{CODE}", &validator);
        assert_eq!(report.product_code, "{CODE}");
        assert!(!report.needs_repair);
    }

    #[test]
    fn test_validation_report_to_json() {
        let validator = MsiValidator::new();
        let report = ValidationReport::generate("{CODE}", &validator);
        let json = report.to_json();
        assert!(json.contains("{CODE}"));
    }
}
