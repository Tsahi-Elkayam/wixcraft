//! Technical debt calculation for WiX projects
//!
//! Provides debt metrics, ratios, and ratings following SonarQube methodology.

use super::types::{AnalysisResult, Diagnostic, IssueType, Severity};
use serde::{Deserialize, Serialize};

/// Technical debt summary for a project
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TechnicalDebt {
    /// Total debt in minutes
    pub total_minutes: u32,
    /// Debt from bugs
    pub bug_debt_minutes: u32,
    /// Debt from vulnerabilities
    pub vulnerability_debt_minutes: u32,
    /// Debt from code smells (maintainability)
    pub code_smell_debt_minutes: u32,
    /// Debt from security hotspots
    pub security_hotspot_debt_minutes: u32,
    /// Lines of code analyzed
    pub lines_of_code: usize,
    /// Development time estimate (for ratio calculation)
    pub dev_time_minutes: u32,
    /// Debt ratio (debt / dev_time as percentage)
    pub debt_ratio: f64,
    /// Technical debt rating
    pub rating: DebtRating,
    /// Issue counts by severity for context
    pub issues_by_severity: SeverityCounts,
}

/// Issue counts by severity
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SeverityCounts {
    pub blocker: usize,
    pub high: usize,
    pub medium: usize,
    pub low: usize,
    pub info: usize,
}

impl TechnicalDebt {
    /// Calculate technical debt from analysis results
    pub fn from_results(results: &[AnalysisResult], lines_of_code: usize) -> Self {
        let mut debt = Self {
            lines_of_code,
            ..Default::default()
        };

        for result in results {
            for diagnostic in &result.diagnostics {
                let effort = diagnostic.effort_minutes.unwrap_or_else(|| default_effort(diagnostic));
                debt.total_minutes += effort;

                match diagnostic.issue_type {
                    IssueType::Bug => debt.bug_debt_minutes += effort,
                    IssueType::Vulnerability => debt.vulnerability_debt_minutes += effort,
                    IssueType::CodeSmell => debt.code_smell_debt_minutes += effort,
                    IssueType::SecurityHotspot => debt.security_hotspot_debt_minutes += effort,
                    IssueType::Secret => debt.vulnerability_debt_minutes += effort,
                }

                match diagnostic.severity {
                    Severity::Blocker => debt.issues_by_severity.blocker += 1,
                    Severity::High => debt.issues_by_severity.high += 1,
                    Severity::Medium => debt.issues_by_severity.medium += 1,
                    Severity::Low => debt.issues_by_severity.low += 1,
                    Severity::Info => debt.issues_by_severity.info += 1,
                }
            }
        }

        // Estimate development time: ~30 minutes per 10 lines of code
        // This is a rough heuristic that can be configured
        debt.dev_time_minutes = (lines_of_code as u32 / 10) * 30;
        if debt.dev_time_minutes == 0 {
            debt.dev_time_minutes = 60; // Minimum 1 hour
        }

        debt.calculate_ratio();
        debt.calculate_rating();

        debt
    }

    /// Calculate debt ratio as percentage
    fn calculate_ratio(&mut self) {
        if self.dev_time_minutes > 0 {
            self.debt_ratio = (self.total_minutes as f64 / self.dev_time_minutes as f64) * 100.0;
        }
    }

    /// Calculate debt rating based on ratio (SonarQube methodology)
    fn calculate_rating(&mut self) {
        // SonarQube uses these thresholds:
        // A: <= 5%
        // B: <= 10%
        // C: <= 20%
        // D: <= 50%
        // E: > 50%
        self.rating = if self.debt_ratio <= 5.0 {
            DebtRating::A
        } else if self.debt_ratio <= 10.0 {
            DebtRating::B
        } else if self.debt_ratio <= 20.0 {
            DebtRating::C
        } else if self.debt_ratio <= 50.0 {
            DebtRating::D
        } else {
            DebtRating::E
        };
    }

    /// Format total debt as human-readable string
    pub fn total_display(&self) -> String {
        format_duration(self.total_minutes)
    }

    /// Format development time as human-readable string
    pub fn dev_time_display(&self) -> String {
        format_duration(self.dev_time_minutes)
    }

    /// Get debt breakdown by category
    pub fn breakdown(&self) -> DebtBreakdown {
        DebtBreakdown {
            bugs: DebtCategory {
                minutes: self.bug_debt_minutes,
                display: format_duration(self.bug_debt_minutes),
                percentage: if self.total_minutes > 0 {
                    (self.bug_debt_minutes as f64 / self.total_minutes as f64) * 100.0
                } else {
                    0.0
                },
            },
            vulnerabilities: DebtCategory {
                minutes: self.vulnerability_debt_minutes,
                display: format_duration(self.vulnerability_debt_minutes),
                percentage: if self.total_minutes > 0 {
                    (self.vulnerability_debt_minutes as f64 / self.total_minutes as f64) * 100.0
                } else {
                    0.0
                },
            },
            code_smells: DebtCategory {
                minutes: self.code_smell_debt_minutes,
                display: format_duration(self.code_smell_debt_minutes),
                percentage: if self.total_minutes > 0 {
                    (self.code_smell_debt_minutes as f64 / self.total_minutes as f64) * 100.0
                } else {
                    0.0
                },
            },
            security_hotspots: DebtCategory {
                minutes: self.security_hotspot_debt_minutes,
                display: format_duration(self.security_hotspot_debt_minutes),
                percentage: if self.total_minutes > 0 {
                    (self.security_hotspot_debt_minutes as f64 / self.total_minutes as f64) * 100.0
                } else {
                    0.0
                },
            },
        }
    }
}

/// Default effort estimate based on issue type and severity
fn default_effort(diagnostic: &Diagnostic) -> u32 {
    let base = match diagnostic.issue_type {
        IssueType::Bug => 30,           // 30 min for bugs
        IssueType::Vulnerability => 60, // 1 hour for vulnerabilities
        IssueType::CodeSmell => 10,     // 10 min for code smells
        IssueType::SecurityHotspot => 45, // 45 min for security review
        IssueType::Secret => 15,        // 15 min for secret removal
    };

    // Adjust by severity
    match diagnostic.severity {
        Severity::Blocker => base * 2,
        Severity::High => (base as f64 * 1.5) as u32,
        Severity::Medium => base,
        Severity::Low => base / 2,
        Severity::Info => 5, // Minimal effort for info
    }
}

/// Format duration in minutes to human-readable string
fn format_duration(minutes: u32) -> String {
    if minutes == 0 {
        return "0min".to_string();
    }

    let hours = minutes / 60;
    let remaining_minutes = minutes % 60;
    let days = hours / 8; // 8-hour workday
    let remaining_hours = hours % 8;

    if days > 0 {
        format!("{}d {}h", days, remaining_hours)
    } else if hours > 0 {
        format!("{}h {}min", hours, remaining_minutes)
    } else {
        format!("{}min", minutes)
    }
}

/// Technical debt rating (A = best, E = worst)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Default)]
pub enum DebtRating {
    #[default]
    A = 1,
    B = 2,
    C = 3,
    D = 4,
    E = 5,
}

impl DebtRating {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::A => "A",
            Self::B => "B",
            Self::C => "C",
            Self::D => "D",
            Self::E => "E",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            Self::A => "Excellent - minimal technical debt",
            Self::B => "Good - low technical debt",
            Self::C => "Moderate - acceptable technical debt",
            Self::D => "Poor - high technical debt, plan remediation",
            Self::E => "Critical - very high debt, immediate action needed",
        }
    }
}

/// Debt breakdown by category
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebtBreakdown {
    pub bugs: DebtCategory,
    pub vulnerabilities: DebtCategory,
    pub code_smells: DebtCategory,
    pub security_hotspots: DebtCategory,
}

/// Single debt category
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebtCategory {
    pub minutes: u32,
    pub display: String,
    pub percentage: f64,
}

/// Quality gate for technical debt
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebtQualityGate {
    /// Maximum allowed debt ratio (percentage)
    pub max_debt_ratio: f64,
    /// Maximum allowed new debt in minutes
    pub max_new_debt_minutes: u32,
    /// Minimum required rating
    pub min_rating: DebtRating,
    /// Block on blocker issues
    pub block_on_blockers: bool,
    /// Block on high severity issues
    pub block_on_high: bool,
}

impl Default for DebtQualityGate {
    fn default() -> Self {
        Self {
            max_debt_ratio: 10.0,  // Max 10% debt ratio
            max_new_debt_minutes: 60, // Max 1 hour new debt
            min_rating: DebtRating::C, // At least C rating
            block_on_blockers: true,
            block_on_high: false,
        }
    }
}

impl DebtQualityGate {
    /// Check if debt passes quality gate
    pub fn check(&self, debt: &TechnicalDebt) -> QualityGateResult {
        let mut passed = true;
        let mut failures = Vec::new();

        if debt.debt_ratio > self.max_debt_ratio {
            passed = false;
            failures.push(format!(
                "Debt ratio {:.1}% exceeds maximum {:.1}%",
                debt.debt_ratio, self.max_debt_ratio
            ));
        }

        if debt.rating > self.min_rating {
            passed = false;
            failures.push(format!(
                "Rating {} is below minimum {}",
                debt.rating.as_str(),
                self.min_rating.as_str()
            ));
        }

        if self.block_on_blockers && debt.issues_by_severity.blocker > 0 {
            passed = false;
            failures.push(format!(
                "{} blocker issue(s) found",
                debt.issues_by_severity.blocker
            ));
        }

        if self.block_on_high && debt.issues_by_severity.high > 0 {
            passed = false;
            failures.push(format!(
                "{} high severity issue(s) found",
                debt.issues_by_severity.high
            ));
        }

        QualityGateResult { passed, failures }
    }
}

/// Result of quality gate check
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityGateResult {
    pub passed: bool,
    pub failures: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{Category, Location, Position, Range};
    use std::path::PathBuf;

    fn make_location() -> Location {
        Location::new(
            PathBuf::from("test.wxs"),
            Range::new(Position::new(1, 1), Position::new(1, 10)),
        )
    }

    #[test]
    fn test_empty_results() {
        let results: Vec<AnalysisResult> = vec![];
        let debt = TechnicalDebt::from_results(&results, 100);

        assert_eq!(debt.total_minutes, 0);
        assert_eq!(debt.rating, DebtRating::A);
    }

    #[test]
    fn test_basic_debt_calculation() {
        let mut result = AnalysisResult::new();
        let diag = Diagnostic::high("BUG-001", IssueType::Bug, "Bug", make_location())
            .with_effort(30);
        result.add(diag);

        let debt = TechnicalDebt::from_results(&[result], 100);

        assert_eq!(debt.total_minutes, 30);
        assert_eq!(debt.bug_debt_minutes, 30);
    }

    #[test]
    fn test_debt_by_issue_type() {
        let mut result = AnalysisResult::new();
        result.add(Diagnostic::high("BUG-001", IssueType::Bug, "Bug", make_location()).with_effort(30));
        result.add(Diagnostic::high("SEC-001", IssueType::Vulnerability, "Vuln", make_location()).with_effort(60));
        result.add(Diagnostic::medium("CS-001", IssueType::CodeSmell, "Smell", make_location()).with_effort(10));

        let debt = TechnicalDebt::from_results(&[result], 100);

        assert_eq!(debt.total_minutes, 100);
        assert_eq!(debt.bug_debt_minutes, 30);
        assert_eq!(debt.vulnerability_debt_minutes, 60);
        assert_eq!(debt.code_smell_debt_minutes, 10);
    }

    #[test]
    fn test_default_effort() {
        let diag = Diagnostic::high("BUG-001", IssueType::Bug, "Bug", make_location());
        let effort = default_effort(&diag);
        assert_eq!(effort, 45); // 30 * 1.5 for high severity

        let diag2 = Diagnostic::blocker("BUG-002", IssueType::Bug, "Blocker", make_location());
        let effort2 = default_effort(&diag2);
        assert_eq!(effort2, 60); // 30 * 2 for blocker
    }

    #[test]
    fn test_debt_ratio() {
        let mut result = AnalysisResult::new();
        // 300 lines of code = ~900 min dev time
        // 90 min debt = 10% ratio
        result.add(Diagnostic::high("B-1", IssueType::Bug, "B", make_location()).with_effort(90));

        let debt = TechnicalDebt::from_results(&[result], 300);

        assert!(debt.debt_ratio >= 9.0 && debt.debt_ratio <= 11.0);
        assert_eq!(debt.rating, DebtRating::B); // 10% = B rating
    }

    #[test]
    fn test_debt_ratings() {
        // A: <= 5%
        let mut result = AnalysisResult::new();
        result.add(Diagnostic::high("B-1", IssueType::Bug, "B", make_location()).with_effort(10));
        let debt = TechnicalDebt::from_results(&[result], 1000);
        assert_eq!(debt.rating, DebtRating::A);

        // E: > 50%
        let mut result2 = AnalysisResult::new();
        result2.add(Diagnostic::high("B-1", IssueType::Bug, "B", make_location()).with_effort(2000));
        let debt2 = TechnicalDebt::from_results(&[result2], 100);
        assert_eq!(debt2.rating, DebtRating::E);
    }

    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(0), "0min");
        assert_eq!(format_duration(30), "30min");
        assert_eq!(format_duration(90), "1h 30min");
        assert_eq!(format_duration(480), "1d 0h"); // 8 hours = 1 day
        assert_eq!(format_duration(960), "2d 0h"); // 16 hours = 2 days
        assert_eq!(format_duration(540), "1d 1h"); // 9 hours = 1 day 1 hour
    }

    #[test]
    fn test_breakdown() {
        let mut result = AnalysisResult::new();
        result.add(Diagnostic::high("B-1", IssueType::Bug, "B", make_location()).with_effort(50));
        result.add(Diagnostic::high("S-1", IssueType::Vulnerability, "S", make_location()).with_effort(50));

        let debt = TechnicalDebt::from_results(&[result], 100);
        let breakdown = debt.breakdown();

        assert_eq!(breakdown.bugs.minutes, 50);
        assert_eq!(breakdown.vulnerabilities.minutes, 50);
        assert!((breakdown.bugs.percentage - 50.0).abs() < 0.1);
    }

    #[test]
    fn test_quality_gate_pass() {
        let mut result = AnalysisResult::new();
        result.add(Diagnostic::medium("CS-1", IssueType::CodeSmell, "Smell", make_location()).with_effort(10));

        let debt = TechnicalDebt::from_results(&[result], 1000);
        let gate = DebtQualityGate::default();
        let check = gate.check(&debt);

        assert!(check.passed);
        assert!(check.failures.is_empty());
    }

    #[test]
    fn test_quality_gate_fail_ratio() {
        let mut result = AnalysisResult::new();
        result.add(Diagnostic::high("B-1", IssueType::Bug, "B", make_location()).with_effort(1000));

        let debt = TechnicalDebt::from_results(&[result], 100);
        let gate = DebtQualityGate::default();
        let check = gate.check(&debt);

        assert!(!check.passed);
        assert!(check.failures.iter().any(|f| f.contains("Debt ratio")));
    }

    #[test]
    fn test_quality_gate_fail_blocker() {
        let mut result = AnalysisResult::new();
        result.add(Diagnostic::blocker("B-1", IssueType::Bug, "Blocker", make_location()).with_effort(10));

        let debt = TechnicalDebt::from_results(&[result], 1000);
        let gate = DebtQualityGate::default();
        let check = gate.check(&debt);

        assert!(!check.passed);
        assert!(check.failures.iter().any(|f| f.contains("blocker")));
    }

    #[test]
    fn test_severity_counts() {
        let mut result = AnalysisResult::new();
        result.add(Diagnostic::blocker("B-1", IssueType::Bug, "B", make_location()));
        result.add(Diagnostic::high("B-2", IssueType::Bug, "B", make_location()));
        result.add(Diagnostic::medium("CS-1", IssueType::CodeSmell, "CS", make_location()));
        result.add(Diagnostic::low("CS-2", IssueType::CodeSmell, "CS", make_location()));
        result.add(Diagnostic::info("INFO-1", Category::BestPractice, "I", make_location()));

        let debt = TechnicalDebt::from_results(&[result], 100);

        assert_eq!(debt.issues_by_severity.blocker, 1);
        assert_eq!(debt.issues_by_severity.high, 1);
        assert_eq!(debt.issues_by_severity.medium, 1);
        assert_eq!(debt.issues_by_severity.low, 1);
        assert_eq!(debt.issues_by_severity.info, 1);
    }

    #[test]
    fn test_rating_descriptions() {
        assert!(DebtRating::A.description().contains("Excellent"));
        assert!(DebtRating::C.description().contains("Moderate"));
        assert!(DebtRating::E.description().contains("Critical"));
    }

    #[test]
    fn test_display_methods() {
        let mut result = AnalysisResult::new();
        result.add(Diagnostic::high("B-1", IssueType::Bug, "B", make_location()).with_effort(90));

        let debt = TechnicalDebt::from_results(&[result], 100);

        assert_eq!(debt.total_display(), "1h 30min");
        assert!(!debt.dev_time_display().is_empty());
    }
}
