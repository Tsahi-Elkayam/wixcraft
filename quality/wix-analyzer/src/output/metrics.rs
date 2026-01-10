//! Metrics summary output formatter
//!
//! Provides aggregated project-level metrics in text and JSON formats.

use crate::core::{AnalysisResult, Category, IssueType, Severity};
use crate::output::Formatter;
use serde::Serialize;
use std::collections::HashMap;

/// Metrics summary for a project
#[derive(Debug, Clone, Serialize)]
pub struct MetricsSummary {
    /// Total number of files analyzed
    pub files_analyzed: usize,
    /// Total number of issues
    pub total_issues: usize,
    /// Issues by severity
    pub by_severity: SeverityCounts,
    /// Issues by type
    pub by_type: TypeCounts,
    /// Issues by category
    pub by_category: CategoryCounts,
    /// Issues by rule (top N rules with most issues)
    pub top_rules: Vec<RuleCount>,
    /// Technical debt in minutes
    pub debt_minutes: u32,
    /// Technical debt as human-readable string
    pub debt_display: String,
    /// Issue density (issues per file)
    pub issue_density: f64,
    /// Maintainability rating (A-E)
    pub maintainability_rating: char,
    /// Reliability rating (A-E)
    pub reliability_rating: char,
    /// Security rating (A-E)
    pub security_rating: char,
}

/// Issue counts by severity
#[derive(Debug, Clone, Default, Serialize)]
pub struct SeverityCounts {
    pub blocker: usize,
    pub high: usize,
    pub medium: usize,
    pub low: usize,
    pub info: usize,
}

/// Issue counts by type
#[derive(Debug, Clone, Default, Serialize)]
pub struct TypeCounts {
    pub bug: usize,
    pub vulnerability: usize,
    pub code_smell: usize,
    pub security_hotspot: usize,
    pub secret: usize,
}

/// Issue counts by category
#[derive(Debug, Clone, Default, Serialize)]
pub struct CategoryCounts {
    pub validation: usize,
    pub best_practice: usize,
    pub security: usize,
    pub dead_code: usize,
}

/// Count of issues per rule
#[derive(Debug, Clone, Serialize)]
pub struct RuleCount {
    pub rule_id: String,
    pub count: usize,
}

impl MetricsSummary {
    /// Calculate metrics from analysis results
    pub fn from_results(results: &[AnalysisResult]) -> Self {
        let files_analyzed = results.iter()
            .flat_map(|r| &r.files)
            .collect::<std::collections::HashSet<_>>()
            .len();

        let all_diagnostics: Vec<_> = results.iter()
            .flat_map(|r| &r.diagnostics)
            .collect();

        let total_issues = all_diagnostics.len();

        // Count by severity
        let by_severity = SeverityCounts {
            blocker: all_diagnostics.iter().filter(|d| d.severity == Severity::Blocker).count(),
            high: all_diagnostics.iter().filter(|d| d.severity == Severity::High).count(),
            medium: all_diagnostics.iter().filter(|d| d.severity == Severity::Medium).count(),
            low: all_diagnostics.iter().filter(|d| d.severity == Severity::Low).count(),
            info: all_diagnostics.iter().filter(|d| d.severity == Severity::Info).count(),
        };

        // Count by type
        let by_type = TypeCounts {
            bug: all_diagnostics.iter().filter(|d| d.issue_type == IssueType::Bug).count(),
            vulnerability: all_diagnostics.iter().filter(|d| d.issue_type == IssueType::Vulnerability).count(),
            code_smell: all_diagnostics.iter().filter(|d| d.issue_type == IssueType::CodeSmell).count(),
            security_hotspot: all_diagnostics.iter().filter(|d| d.issue_type == IssueType::SecurityHotspot).count(),
            secret: all_diagnostics.iter().filter(|d| d.issue_type == IssueType::Secret).count(),
        };

        // Count by category
        let by_category = CategoryCounts {
            validation: all_diagnostics.iter().filter(|d| d.category == Category::Validation).count(),
            best_practice: all_diagnostics.iter().filter(|d| d.category == Category::BestPractice).count(),
            security: all_diagnostics.iter().filter(|d| d.category == Category::Security).count(),
            dead_code: all_diagnostics.iter().filter(|d| d.category == Category::DeadCode).count(),
        };

        // Count by rule
        let mut rule_counts: HashMap<String, usize> = HashMap::new();
        for d in &all_diagnostics {
            *rule_counts.entry(d.rule_id.clone()).or_default() += 1;
        }
        let mut top_rules: Vec<_> = rule_counts
            .into_iter()
            .map(|(rule_id, count)| RuleCount { rule_id, count })
            .collect();
        top_rules.sort_by(|a, b| b.count.cmp(&a.count));
        top_rules.truncate(10); // Top 10 rules

        // Technical debt
        let debt_minutes: u32 = all_diagnostics.iter()
            .filter_map(|d| d.effort_minutes)
            .sum();
        let debt_display = format_debt(debt_minutes);

        // Issue density
        let issue_density = if files_analyzed > 0 {
            total_issues as f64 / files_analyzed as f64
        } else {
            0.0
        };

        // Calculate ratings
        let maintainability_rating = calculate_maintainability_rating(by_type.code_smell, files_analyzed);
        let reliability_rating = calculate_reliability_rating(by_type.bug, &by_severity);
        let security_rating = calculate_security_rating(
            by_type.vulnerability + by_type.secret,
            &by_severity,
        );

        Self {
            files_analyzed,
            total_issues,
            by_severity,
            by_type,
            by_category,
            top_rules,
            debt_minutes,
            debt_display,
            issue_density,
            maintainability_rating,
            reliability_rating,
            security_rating,
        }
    }

    /// Format as text
    pub fn format_text(&self) -> String {
        let mut output = String::new();

        output.push_str("=== Metrics Summary ===\n\n");

        // Overview
        output.push_str(&format!("Files Analyzed: {}\n", self.files_analyzed));
        output.push_str(&format!("Total Issues:   {}\n", self.total_issues));
        output.push_str(&format!("Issue Density:  {:.2} issues/file\n", self.issue_density));
        output.push_str(&format!("Tech Debt:      {}\n", self.debt_display));
        output.push_str("\n");

        // Ratings
        output.push_str("--- Ratings ---\n");
        output.push_str(&format!("Reliability:     {} (bugs)\n", self.reliability_rating));
        output.push_str(&format!("Security:        {} (vulnerabilities)\n", self.security_rating));
        output.push_str(&format!("Maintainability: {} (code smells)\n", self.maintainability_rating));
        output.push_str("\n");

        // By severity
        output.push_str("--- By Severity ---\n");
        output.push_str(&format!("Blocker: {}\n", self.by_severity.blocker));
        output.push_str(&format!("High:    {}\n", self.by_severity.high));
        output.push_str(&format!("Medium:  {}\n", self.by_severity.medium));
        output.push_str(&format!("Low:     {}\n", self.by_severity.low));
        output.push_str(&format!("Info:    {}\n", self.by_severity.info));
        output.push_str("\n");

        // By type
        output.push_str("--- By Type ---\n");
        output.push_str(&format!("Bugs:             {}\n", self.by_type.bug));
        output.push_str(&format!("Vulnerabilities:  {}\n", self.by_type.vulnerability));
        output.push_str(&format!("Code Smells:      {}\n", self.by_type.code_smell));
        output.push_str(&format!("Security Hotspots:{}\n", self.by_type.security_hotspot));
        output.push_str(&format!("Secrets:          {}\n", self.by_type.secret));
        output.push_str("\n");

        // Top rules
        if !self.top_rules.is_empty() {
            output.push_str("--- Top Rules ---\n");
            for (i, rule) in self.top_rules.iter().enumerate() {
                output.push_str(&format!("{}. {} ({})\n", i + 1, rule.rule_id, rule.count));
            }
        }

        output
    }

    /// Format as JSON
    pub fn format_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_else(|_| "{}".to_string())
    }
}

fn format_debt(minutes: u32) -> String {
    if minutes == 0 {
        return "0min".to_string();
    }
    let hours = minutes / 60;
    let remaining_minutes = minutes % 60;
    let days = hours / 8;
    let remaining_hours = hours % 8;

    if days > 0 {
        format!("{}d {}h {}min", days, remaining_hours, remaining_minutes)
    } else if hours > 0 {
        format!("{}h {}min", hours, remaining_minutes)
    } else {
        format!("{}min", minutes)
    }
}

fn calculate_maintainability_rating(code_smells: usize, files: usize) -> char {
    if files == 0 {
        return 'A';
    }
    let density = code_smells as f64 / files as f64;
    match density {
        d if d < 0.5 => 'A',
        d if d < 1.0 => 'B',
        d if d < 2.0 => 'C',
        d if d < 5.0 => 'D',
        _ => 'E',
    }
}

fn calculate_reliability_rating(bugs: usize, severity: &SeverityCounts) -> char {
    if severity.blocker > 0 || bugs > 10 {
        'E'
    } else if severity.high > 0 || bugs > 5 {
        'D'
    } else if severity.medium > 0 || bugs > 2 {
        'C'
    } else if bugs > 0 {
        'B'
    } else {
        'A'
    }
}

fn calculate_security_rating(vulns: usize, severity: &SeverityCounts) -> char {
    if severity.blocker > 0 || vulns > 5 {
        'E'
    } else if severity.high > 0 || vulns > 2 {
        'D'
    } else if severity.medium > 0 || vulns > 0 {
        'C'
    } else {
        'A'
    }
}

/// Metrics formatter (text output)
pub struct MetricsFormatter {
    json: bool,
}

impl MetricsFormatter {
    /// Create a new metrics formatter
    pub fn new(json: bool) -> Self {
        Self { json }
    }

    /// Create a text metrics formatter
    pub fn text() -> Self {
        Self { json: false }
    }

    /// Create a JSON metrics formatter
    pub fn json() -> Self {
        Self { json: true }
    }
}

impl Formatter for MetricsFormatter {
    fn format(&self, results: &[AnalysisResult]) -> String {
        let summary = MetricsSummary::from_results(results);
        if self.json {
            summary.format_json()
        } else {
            summary.format_text()
        }
    }

    fn format_diagnostic(&self, _diagnostic: &crate::core::Diagnostic) -> String {
        // Metrics formatter doesn't format individual diagnostics
        String::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{Diagnostic, Location, Position, Range};
    use std::path::PathBuf;

    fn make_location() -> Location {
        Location::new(
            PathBuf::from("test.wxs"),
            Range::new(Position::new(1, 1), Position::new(1, 10)),
        )
    }

    #[test]
    fn test_metrics_summary_empty() {
        let results: Vec<AnalysisResult> = vec![];
        let summary = MetricsSummary::from_results(&results);

        assert_eq!(summary.files_analyzed, 0);
        assert_eq!(summary.total_issues, 0);
        assert_eq!(summary.issue_density, 0.0);
        assert_eq!(summary.maintainability_rating, 'A');
        assert_eq!(summary.reliability_rating, 'A');
        assert_eq!(summary.security_rating, 'A');
    }

    #[test]
    fn test_metrics_summary_with_issues() {
        let mut result = AnalysisResult::new();
        result.add_file(PathBuf::from("test.wxs"));
        result.add(Diagnostic::error("VAL-001", Category::Validation, "Error 1", make_location()));
        result.add(Diagnostic::warning("BP-001", Category::BestPractice, "Warning 1", make_location()));
        result.add(Diagnostic::info("INFO-001", Category::BestPractice, "Info 1", make_location()));

        let summary = MetricsSummary::from_results(&[result]);

        assert_eq!(summary.files_analyzed, 1);
        assert_eq!(summary.total_issues, 3);
        assert_eq!(summary.by_severity.high, 1);
        assert_eq!(summary.by_severity.medium, 1);
        assert_eq!(summary.by_severity.info, 1);
        assert_eq!(summary.by_category.validation, 1);
        assert_eq!(summary.by_category.best_practice, 2);
    }

    #[test]
    fn test_metrics_summary_by_type() {
        let mut result = AnalysisResult::new();
        result.add_file(PathBuf::from("test.wxs"));
        result.add(Diagnostic::high("BUG-001", IssueType::Bug, "Bug", make_location()));
        result.add(Diagnostic::high("SEC-001", IssueType::Vulnerability, "Vuln", make_location()));
        result.add(Diagnostic::medium("CS-001", IssueType::CodeSmell, "Smell", make_location()));

        let summary = MetricsSummary::from_results(&[result]);

        assert_eq!(summary.by_type.bug, 1);
        assert_eq!(summary.by_type.vulnerability, 1);
        assert_eq!(summary.by_type.code_smell, 1);
    }

    #[test]
    fn test_metrics_summary_top_rules() {
        let mut result = AnalysisResult::new();
        result.add_file(PathBuf::from("test.wxs"));

        // Add multiple issues with same rule
        for _ in 0..5 {
            result.add(Diagnostic::error("VAL-001", Category::Validation, "Error", make_location()));
        }
        for _ in 0..3 {
            result.add(Diagnostic::warning("BP-001", Category::BestPractice, "Warning", make_location()));
        }

        let summary = MetricsSummary::from_results(&[result]);

        assert_eq!(summary.top_rules.len(), 2);
        assert_eq!(summary.top_rules[0].rule_id, "VAL-001");
        assert_eq!(summary.top_rules[0].count, 5);
        assert_eq!(summary.top_rules[1].rule_id, "BP-001");
        assert_eq!(summary.top_rules[1].count, 3);
    }

    #[test]
    fn test_metrics_summary_debt() {
        let mut result = AnalysisResult::new();
        result.add_file(PathBuf::from("test.wxs"));
        result.add(Diagnostic::error("VAL-001", Category::Validation, "Error", make_location()).with_effort(30));
        result.add(Diagnostic::warning("BP-001", Category::BestPractice, "Warning", make_location()).with_effort(60));

        let summary = MetricsSummary::from_results(&[result]);

        assert_eq!(summary.debt_minutes, 90);
        assert_eq!(summary.debt_display, "1h 30min");
    }

    #[test]
    fn test_format_debt() {
        assert_eq!(format_debt(0), "0min");
        assert_eq!(format_debt(30), "30min");
        assert_eq!(format_debt(90), "1h 30min");
        assert_eq!(format_debt(480), "1d 0h 0min"); // 8h = 1 day
        assert_eq!(format_debt(600), "1d 2h 0min");
    }

    #[test]
    fn test_maintainability_rating() {
        assert_eq!(calculate_maintainability_rating(0, 10), 'A');
        assert_eq!(calculate_maintainability_rating(3, 10), 'A');
        assert_eq!(calculate_maintainability_rating(7, 10), 'B');
        assert_eq!(calculate_maintainability_rating(15, 10), 'C');
        assert_eq!(calculate_maintainability_rating(40, 10), 'D');
        assert_eq!(calculate_maintainability_rating(100, 10), 'E');
    }

    #[test]
    fn test_reliability_rating() {
        let no_issues = SeverityCounts::default();
        assert_eq!(calculate_reliability_rating(0, &no_issues), 'A');

        let minor = SeverityCounts { low: 1, ..Default::default() };
        assert_eq!(calculate_reliability_rating(1, &minor), 'B');

        let medium = SeverityCounts { medium: 1, ..Default::default() };
        assert_eq!(calculate_reliability_rating(2, &medium), 'C');

        let high = SeverityCounts { high: 1, ..Default::default() };
        assert_eq!(calculate_reliability_rating(5, &high), 'D');

        let blocker = SeverityCounts { blocker: 1, ..Default::default() };
        assert_eq!(calculate_reliability_rating(0, &blocker), 'E');
    }

    #[test]
    fn test_security_rating() {
        let no_issues = SeverityCounts::default();
        assert_eq!(calculate_security_rating(0, &no_issues), 'A');

        let medium = SeverityCounts { medium: 1, ..Default::default() };
        assert_eq!(calculate_security_rating(1, &medium), 'C');

        let high = SeverityCounts { high: 1, ..Default::default() };
        assert_eq!(calculate_security_rating(1, &high), 'D');

        let blocker = SeverityCounts { blocker: 1, ..Default::default() };
        assert_eq!(calculate_security_rating(0, &blocker), 'E');
    }

    #[test]
    fn test_metrics_format_text() {
        let results: Vec<AnalysisResult> = vec![];
        let summary = MetricsSummary::from_results(&results);
        let text = summary.format_text();

        assert!(text.contains("Metrics Summary"));
        assert!(text.contains("Files Analyzed"));
        assert!(text.contains("Ratings"));
    }

    #[test]
    fn test_metrics_format_json() {
        let results: Vec<AnalysisResult> = vec![];
        let summary = MetricsSummary::from_results(&results);
        let json = summary.format_json();

        assert!(json.contains("files_analyzed"));
        assert!(json.contains("total_issues"));
        assert!(json.contains("by_severity"));
    }

    #[test]
    fn test_metrics_formatter_text() {
        let formatter = MetricsFormatter::text();
        let results: Vec<AnalysisResult> = vec![];
        let output = formatter.format(&results);

        assert!(output.contains("Metrics Summary"));
    }

    #[test]
    fn test_metrics_formatter_json() {
        let formatter = MetricsFormatter::json();
        let results: Vec<AnalysisResult> = vec![];
        let output = formatter.format(&results);

        assert!(output.contains("files_analyzed"));
    }

    #[test]
    fn test_metrics_formatter_diagnostic() {
        let formatter = MetricsFormatter::text();
        let diagnostic = Diagnostic::error("VAL-001", Category::Validation, "Error", make_location());
        let output = formatter.format_diagnostic(&diagnostic);

        // Metrics formatter doesn't format individual diagnostics
        assert!(output.is_empty());
    }

    #[test]
    fn test_issue_density() {
        let mut result1 = AnalysisResult::new();
        result1.add_file(PathBuf::from("test1.wxs"));
        for _ in 0..10 {
            result1.add(Diagnostic::error("VAL-001", Category::Validation, "Error", make_location()));
        }

        let mut result2 = AnalysisResult::new();
        result2.add_file(PathBuf::from("test2.wxs"));
        for _ in 0..5 {
            result2.add(Diagnostic::error("VAL-001", Category::Validation, "Error", make_location()));
        }

        let summary = MetricsSummary::from_results(&[result1, result2]);

        assert_eq!(summary.files_analyzed, 2);
        assert_eq!(summary.total_issues, 15);
        assert_eq!(summary.issue_density, 7.5); // 15 issues / 2 files
    }
}
