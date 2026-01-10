//! Quality Gate - Pass/Fail criteria for CI/CD pipelines
//!
//! Quality gates define thresholds that code must meet before release.
//! Inspired by SonarQube's Quality Gate feature.
//!
//! # Usage
//!
//! ```rust,ignore
//! use wix_analyzer::core::{QualityGate, GateCondition};
//!
//! let gate = QualityGate::default()
//!     .with_condition(GateCondition::max_blockers(0))
//!     .with_condition(GateCondition::max_vulnerabilities(0))
//!     .with_condition(GateCondition::min_maintainability_rating('A'));
//!
//! let result = gate.evaluate(&analysis_results);
//! if result.passed {
//!     println!("Quality gate passed!");
//! } else {
//!     for failure in &result.failures {
//!         println!("Failed: {}", failure);
//!     }
//! }
//! ```

use serde::{Deserialize, Serialize};

use crate::core::debt::{DebtRating, TechnicalDebt};
use crate::core::{AnalysisResult, IssueType, Severity};

/// Quality gate with configurable conditions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityGate {
    /// Gate name
    pub name: String,
    /// Gate description
    pub description: Option<String>,
    /// Conditions that must all pass
    pub conditions: Vec<GateCondition>,
}

impl Default for QualityGate {
    fn default() -> Self {
        Self::sonar_way()
    }
}

impl QualityGate {
    /// Create an empty quality gate
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: None,
            conditions: Vec::new(),
        }
    }

    /// Create SonarQube "Sonar Way" quality gate (recommended default)
    pub fn sonar_way() -> Self {
        Self {
            name: "Sonar Way".to_string(),
            description: Some("Default quality gate based on SonarQube best practices".to_string()),
            conditions: vec![
                GateCondition::MaxIssueCount {
                    severity: Some(Severity::Blocker),
                    issue_type: None,
                    max: 0,
                },
                GateCondition::MaxIssueCount {
                    severity: Some(Severity::High),
                    issue_type: Some(IssueType::Vulnerability),
                    max: 0,
                },
                GateCondition::MinRating {
                    rating_type: RatingType::Security,
                    min: 'A',
                },
                GateCondition::MinRating {
                    rating_type: RatingType::Reliability,
                    min: 'A',
                },
            ],
        }
    }

    /// Create a strict quality gate (all checks must pass)
    pub fn strict() -> Self {
        Self {
            name: "Strict".to_string(),
            description: Some("Strict quality gate - no issues allowed".to_string()),
            conditions: vec![
                GateCondition::MaxIssueCount {
                    severity: Some(Severity::Blocker),
                    issue_type: None,
                    max: 0,
                },
                GateCondition::MaxIssueCount {
                    severity: Some(Severity::High),
                    issue_type: None,
                    max: 0,
                },
                GateCondition::MaxIssueCount {
                    severity: Some(Severity::Medium),
                    issue_type: Some(IssueType::Vulnerability),
                    max: 0,
                },
                GateCondition::MaxIssueCount {
                    severity: None,
                    issue_type: Some(IssueType::Secret),
                    max: 0,
                },
                GateCondition::MinRating {
                    rating_type: RatingType::Security,
                    min: 'A',
                },
                GateCondition::MinRating {
                    rating_type: RatingType::Reliability,
                    min: 'A',
                },
                GateCondition::MinRating {
                    rating_type: RatingType::Maintainability,
                    min: 'B',
                },
                GateCondition::MaxDebtRatio { max_percent: 5.0 },
            ],
        }
    }

    /// Create a relaxed quality gate (only critical issues)
    pub fn relaxed() -> Self {
        Self {
            name: "Relaxed".to_string(),
            description: Some("Relaxed quality gate - only blocks on critical issues".to_string()),
            conditions: vec![
                GateCondition::MaxIssueCount {
                    severity: Some(Severity::Blocker),
                    issue_type: None,
                    max: 0,
                },
                GateCondition::MaxIssueCount {
                    severity: None,
                    issue_type: Some(IssueType::Secret),
                    max: 0,
                },
            ],
        }
    }

    /// Add a condition to the gate
    pub fn with_condition(mut self, condition: GateCondition) -> Self {
        self.conditions.push(condition);
        self
    }

    /// Set description
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// Evaluate the quality gate against analysis results
    pub fn evaluate(&self, results: &[AnalysisResult], lines_of_code: usize) -> GateResult {
        let debt = TechnicalDebt::from_results(results, lines_of_code);
        let mut failures = Vec::new();

        for condition in &self.conditions {
            if let Some(failure) = condition.check(results, &debt) {
                failures.push(failure);
            }
        }

        GateResult {
            gate_name: self.name.clone(),
            passed: failures.is_empty(),
            failures,
            conditions_checked: self.conditions.len(),
        }
    }
}

/// A condition that must be met for the gate to pass
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum GateCondition {
    /// Maximum number of issues allowed
    MaxIssueCount {
        #[serde(skip_serializing_if = "Option::is_none")]
        severity: Option<Severity>,
        #[serde(skip_serializing_if = "Option::is_none")]
        issue_type: Option<IssueType>,
        max: usize,
    },
    /// Minimum rating required (A, B, C, D, E)
    MinRating { rating_type: RatingType, min: char },
    /// Maximum technical debt ratio (percentage)
    MaxDebtRatio { max_percent: f64 },
    /// Maximum technical debt in minutes
    MaxDebtMinutes { max: u32 },
    /// Maximum complexity rating
    MaxComplexityRating { max: char },
}

impl GateCondition {
    // === Convenience constructors ===

    /// No blocker issues allowed
    pub fn max_blockers(max: usize) -> Self {
        Self::MaxIssueCount {
            severity: Some(Severity::Blocker),
            issue_type: None,
            max,
        }
    }

    /// No high-severity issues allowed
    pub fn max_high(max: usize) -> Self {
        Self::MaxIssueCount {
            severity: Some(Severity::High),
            issue_type: None,
            max,
        }
    }

    /// No vulnerabilities allowed
    pub fn max_vulnerabilities(max: usize) -> Self {
        Self::MaxIssueCount {
            severity: None,
            issue_type: Some(IssueType::Vulnerability),
            max,
        }
    }

    /// No secrets allowed
    pub fn max_secrets(max: usize) -> Self {
        Self::MaxIssueCount {
            severity: None,
            issue_type: Some(IssueType::Secret),
            max,
        }
    }

    /// Minimum security rating
    pub fn min_security_rating(min: char) -> Self {
        Self::MinRating {
            rating_type: RatingType::Security,
            min,
        }
    }

    /// Minimum reliability rating
    pub fn min_reliability_rating(min: char) -> Self {
        Self::MinRating {
            rating_type: RatingType::Reliability,
            min,
        }
    }

    /// Minimum maintainability rating
    pub fn min_maintainability_rating(min: char) -> Self {
        Self::MinRating {
            rating_type: RatingType::Maintainability,
            min,
        }
    }

    /// Check if condition passes, returns failure message if not
    fn check(&self, results: &[AnalysisResult], debt: &TechnicalDebt) -> Option<GateFailure> {
        match self {
            Self::MaxIssueCount {
                severity,
                issue_type,
                max,
            } => {
                let count = count_issues(results, *severity, *issue_type);
                if count > *max {
                    Some(GateFailure {
                        condition: self.description(),
                        expected: format!("<= {}", max),
                        actual: count.to_string(),
                    })
                } else {
                    None
                }
            }
            Self::MinRating { rating_type, min } => {
                let actual = get_rating(results, debt, *rating_type);
                if actual > *min {
                    Some(GateFailure {
                        condition: self.description(),
                        expected: format!(">= {}", min),
                        actual: actual.to_string(),
                    })
                } else {
                    None
                }
            }
            Self::MaxDebtRatio { max_percent } => {
                if debt.debt_ratio > *max_percent {
                    Some(GateFailure {
                        condition: self.description(),
                        expected: format!("<= {:.1}%", max_percent),
                        actual: format!("{:.1}%", debt.debt_ratio),
                    })
                } else {
                    None
                }
            }
            Self::MaxDebtMinutes { max } => {
                if debt.total_minutes > *max {
                    Some(GateFailure {
                        condition: self.description(),
                        expected: format!("<= {} min", max),
                        actual: format!("{} min", debt.total_minutes),
                    })
                } else {
                    None
                }
            }
            Self::MaxComplexityRating { max: _ } => {
                // TODO: Integrate with complexity analyzer
                // For now, always passes
                None
            }
        }
    }

    /// Get human-readable description of this condition
    pub fn description(&self) -> String {
        match self {
            Self::MaxIssueCount {
                severity,
                issue_type,
                max,
            } => {
                let mut parts = Vec::new();
                if let Some(s) = severity {
                    parts.push(s.as_str().to_string());
                }
                if let Some(t) = issue_type {
                    parts.push(t.display_name().to_string());
                }
                if parts.is_empty() {
                    format!("Total issues <= {}", max)
                } else {
                    format!("{} issues <= {}", parts.join(" "), max)
                }
            }
            Self::MinRating { rating_type, min } => {
                format!("{} rating >= {}", rating_type.as_str(), min)
            }
            Self::MaxDebtRatio { max_percent } => {
                format!("Technical debt ratio <= {:.1}%", max_percent)
            }
            Self::MaxDebtMinutes { max } => {
                format!("Technical debt <= {} minutes", max)
            }
            Self::MaxComplexityRating { max } => {
                format!("Complexity rating <= {}", max)
            }
        }
    }
}

/// Rating types for quality metrics
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RatingType {
    Security,
    Reliability,
    Maintainability,
}

impl RatingType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Security => "Security",
            Self::Reliability => "Reliability",
            Self::Maintainability => "Maintainability",
        }
    }
}

/// Result of evaluating a quality gate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GateResult {
    /// Name of the gate that was evaluated
    pub gate_name: String,
    /// Whether all conditions passed
    pub passed: bool,
    /// List of failed conditions
    pub failures: Vec<GateFailure>,
    /// Number of conditions checked
    pub conditions_checked: usize,
}

impl GateResult {
    /// Get the exit code for CI (0 = pass, 1 = fail)
    pub fn exit_code(&self) -> i32 {
        if self.passed {
            0
        } else {
            1
        }
    }

    /// Get a summary message
    pub fn summary(&self) -> String {
        if self.passed {
            format!(
                "Quality Gate '{}' PASSED ({}/{} conditions)",
                self.gate_name, self.conditions_checked, self.conditions_checked
            )
        } else {
            format!(
                "Quality Gate '{}' FAILED ({}/{} conditions failed)",
                self.gate_name,
                self.failures.len(),
                self.conditions_checked
            )
        }
    }
}

/// A single gate condition failure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GateFailure {
    /// Description of the condition
    pub condition: String,
    /// Expected value
    pub expected: String,
    /// Actual value
    pub actual: String,
}

impl std::fmt::Display for GateFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}: expected {}, got {}",
            self.condition, self.expected, self.actual
        )
    }
}

// === Helper functions ===

fn count_issues(
    results: &[AnalysisResult],
    severity: Option<Severity>,
    issue_type: Option<IssueType>,
) -> usize {
    results
        .iter()
        .flat_map(|r| &r.diagnostics)
        .filter(|d| {
            severity.is_none_or(|s| d.severity >= s)
                && issue_type.is_none_or(|t| d.issue_type == t)
        })
        .count()
}

fn get_rating(results: &[AnalysisResult], debt: &TechnicalDebt, rating_type: RatingType) -> char {
    match rating_type {
        RatingType::Security => {
            let vuln_count = results
                .iter()
                .flat_map(|r| &r.diagnostics)
                .filter(|d| {
                    d.issue_type == IssueType::Vulnerability
                        || d.issue_type == IssueType::SecurityHotspot
                        || d.issue_type == IssueType::Secret
                })
                .count();
            if vuln_count == 0 {
                'A'
            } else if vuln_count <= 2 {
                'B'
            } else if vuln_count <= 5 {
                'C'
            } else if vuln_count <= 10 {
                'D'
            } else {
                'E'
            }
        }
        RatingType::Reliability => {
            let bug_count = results
                .iter()
                .flat_map(|r| &r.diagnostics)
                .filter(|d| d.issue_type == IssueType::Bug)
                .count();
            if bug_count == 0 {
                'A'
            } else if bug_count <= 2 {
                'B'
            } else if bug_count <= 5 {
                'C'
            } else if bug_count <= 10 {
                'D'
            } else {
                'E'
            }
        }
        RatingType::Maintainability => match debt.rating {
            DebtRating::A => 'A',
            DebtRating::B => 'B',
            DebtRating::C => 'C',
            DebtRating::D => 'D',
            DebtRating::E => 'E',
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{Category, Diagnostic, Location, Position, Range};
    use std::path::PathBuf;

    fn make_location() -> Location {
        Location::new(
            PathBuf::from("test.wxs"),
            Range::new(Position::new(1, 1), Position::new(1, 10)),
        )
    }

    fn make_results(diagnostics: Vec<Diagnostic>) -> Vec<AnalysisResult> {
        vec![AnalysisResult {
            files: vec![PathBuf::from("test.wxs")],
            diagnostics,
        }]
    }

    #[test]
    fn test_quality_gate_default() {
        let gate = QualityGate::default();
        assert_eq!(gate.name, "Sonar Way");
        assert!(!gate.conditions.is_empty());
    }

    #[test]
    fn test_quality_gate_passes_empty() {
        let gate = QualityGate::sonar_way();
        let results = make_results(vec![]);
        let result = gate.evaluate(&results, 100);

        assert!(result.passed);
        assert!(result.failures.is_empty());
    }

    #[test]
    fn test_quality_gate_fails_on_blocker() {
        let gate = QualityGate::sonar_way();
        let results = make_results(vec![Diagnostic::blocker(
            "BUG-001",
            IssueType::Bug,
            "Critical bug",
            make_location(),
        )]);
        let result = gate.evaluate(&results, 100);

        assert!(!result.passed);
        assert!(!result.failures.is_empty());
    }

    #[test]
    fn test_quality_gate_fails_on_vulnerability() {
        let gate = QualityGate::sonar_way();
        let results = make_results(vec![Diagnostic::high(
            "SEC-001",
            IssueType::Vulnerability,
            "Vulnerability",
            make_location(),
        )]);
        let result = gate.evaluate(&results, 100);

        assert!(!result.passed);
    }

    #[test]
    fn test_quality_gate_passes_with_warnings() {
        let gate = QualityGate::relaxed();
        let results = make_results(vec![
            Diagnostic::warning("BP-001", Category::BestPractice, "Warning", make_location()),
            Diagnostic::warning(
                "BP-002",
                Category::BestPractice,
                "Another warning",
                make_location(),
            ),
        ]);
        let result = gate.evaluate(&results, 100);

        assert!(result.passed);
    }

    #[test]
    fn test_quality_gate_strict() {
        let gate = QualityGate::strict();
        let results = make_results(vec![Diagnostic::high(
            "BUG-001",
            IssueType::Bug,
            "Bug",
            make_location(),
        )]);
        let result = gate.evaluate(&results, 100);

        assert!(!result.passed);
    }

    #[test]
    fn test_quality_gate_custom() {
        let gate = QualityGate::new("Custom")
            .with_description("My custom gate")
            .with_condition(GateCondition::max_blockers(0))
            .with_condition(GateCondition::max_vulnerabilities(5));

        assert_eq!(gate.name, "Custom");
        assert_eq!(gate.conditions.len(), 2);
    }

    #[test]
    fn test_gate_condition_description() {
        let cond = GateCondition::max_blockers(0);
        assert!(cond.description().contains("blocker"));

        let cond = GateCondition::min_security_rating('A');
        assert!(cond.description().contains("Security"));
    }

    #[test]
    fn test_gate_result_exit_code() {
        let passed = GateResult {
            gate_name: "Test".to_string(),
            passed: true,
            failures: vec![],
            conditions_checked: 1,
        };
        assert_eq!(passed.exit_code(), 0);

        let failed = GateResult {
            gate_name: "Test".to_string(),
            passed: false,
            failures: vec![GateFailure {
                condition: "Test".to_string(),
                expected: "0".to_string(),
                actual: "1".to_string(),
            }],
            conditions_checked: 1,
        };
        assert_eq!(failed.exit_code(), 1);
    }

    #[test]
    fn test_gate_result_summary() {
        let result = GateResult {
            gate_name: "Sonar Way".to_string(),
            passed: true,
            failures: vec![],
            conditions_checked: 4,
        };
        assert!(result.summary().contains("PASSED"));
        assert!(result.summary().contains("4/4"));
    }

    #[test]
    fn test_gate_failure_display() {
        let failure = GateFailure {
            condition: "blocker issues".to_string(),
            expected: "<= 0".to_string(),
            actual: "2".to_string(),
        };
        let display = failure.to_string();
        assert!(display.contains("blocker issues"));
        assert!(display.contains("<= 0"));
        assert!(display.contains("2"));
    }

    #[test]
    fn test_max_debt_ratio() {
        let gate = QualityGate::new("Debt Test")
            .with_condition(GateCondition::MaxDebtRatio { max_percent: 5.0 });

        // With high debt (many issues with effort)
        let results = make_results(vec![Diagnostic::high(
            "BUG-001",
            IssueType::Bug,
            "Bug",
            make_location(),
        )
        .with_effort(1000)]);
        let result = gate.evaluate(&results, 10); // Small codebase = high ratio

        // Should fail due to high debt ratio
        assert!(!result.passed);
    }

    #[test]
    fn test_rating_calculation() {
        let results = make_results(vec![]);
        let debt = TechnicalDebt::from_results(&results, 100);

        assert_eq!(get_rating(&results, &debt, RatingType::Security), 'A');
        assert_eq!(get_rating(&results, &debt, RatingType::Reliability), 'A');
    }
}
