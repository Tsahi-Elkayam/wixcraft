//! JSON output formatter

use super::Formatter;
use crate::core::{AnalysisResult, Category, Diagnostic, IssueType, Severity};
use serde::Serialize;

/// JSON formatter
pub struct JsonFormatter;

impl JsonFormatter {
    pub fn new() -> Self {
        Self
    }
}

impl Default for JsonFormatter {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Serialize)]
struct JsonOutput {
    diagnostics: Vec<JsonDiagnostic>,
    summary: JsonSummary,
}

#[derive(Serialize)]
struct JsonDiagnostic {
    rule_id: String,
    category: String,
    issue_type: String,
    severity: String,
    message: String,
    file: String,
    line: usize,
    column: usize,
    end_line: usize,
    end_column: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    help: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    fix: Option<JsonFix>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    related: Vec<JsonRelated>,
    #[serde(skip_serializing_if = "Option::is_none")]
    effort_minutes: Option<u32>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    tags: Vec<String>,
}

#[derive(Serialize)]
struct JsonFix {
    description: String,
}

#[derive(Serialize)]
struct JsonRelated {
    message: String,
    file: String,
    line: usize,
    column: usize,
}

#[derive(Serialize)]
struct JsonSummary {
    total: usize,
    blockers: usize,
    errors: usize,
    warnings: usize,
    info: usize,
    bugs: usize,
    vulnerabilities: usize,
    code_smells: usize,
    effort_minutes: u32,
}

fn category_to_string(cat: Category) -> &'static str {
    match cat {
        Category::Validation => "validation",
        Category::BestPractice => "best-practice",
        Category::Security => "security",
        Category::DeadCode => "dead-code",
    }
}

fn issue_type_to_string(it: IssueType) -> &'static str {
    match it {
        IssueType::Bug => "bug",
        IssueType::Vulnerability => "vulnerability",
        IssueType::CodeSmell => "code_smell",
        IssueType::SecurityHotspot => "security_hotspot",
        IssueType::Secret => "secret",
    }
}

fn severity_to_string(sev: Severity) -> &'static str {
    match sev {
        Severity::Blocker => "blocker",
        Severity::High => "error",
        Severity::Medium => "warning",
        Severity::Low => "info",
        Severity::Info => "hint",
    }
}

impl Formatter for JsonFormatter {
    fn format(&self, results: &[AnalysisResult]) -> String {
        let mut diagnostics = Vec::new();
        let mut blockers = 0;
        let mut errors = 0;
        let mut warnings = 0;
        let mut info = 0;
        let mut bugs = 0;
        let mut vulnerabilities = 0;
        let mut code_smells = 0;
        let mut total_effort: u32 = 0;

        for result in results {
            for diag in &result.diagnostics {
                match diag.severity {
                    Severity::Blocker => blockers += 1,
                    Severity::High => errors += 1,
                    Severity::Medium => warnings += 1,
                    Severity::Low | Severity::Info => info += 1,
                }

                match diag.issue_type {
                    IssueType::Bug => bugs += 1,
                    IssueType::Vulnerability | IssueType::SecurityHotspot | IssueType::Secret => {
                        vulnerabilities += 1
                    }
                    IssueType::CodeSmell => code_smells += 1,
                }

                if let Some(effort) = diag.effort_minutes {
                    total_effort += effort;
                }

                diagnostics.push(JsonDiagnostic {
                    rule_id: diag.rule_id.clone(),
                    category: category_to_string(diag.category).to_string(),
                    issue_type: issue_type_to_string(diag.issue_type).to_string(),
                    severity: severity_to_string(diag.severity).to_string(),
                    message: diag.message.clone(),
                    file: diag.location.file.display().to_string(),
                    line: diag.location.range.start.line,
                    column: diag.location.range.start.character,
                    end_line: diag.location.range.end.line,
                    end_column: diag.location.range.end.character,
                    help: diag.help.clone(),
                    fix: diag.fix.as_ref().map(|f| JsonFix {
                        description: f.description.clone(),
                    }),
                    related: diag
                        .related
                        .iter()
                        .map(|r| JsonRelated {
                            message: r.message.clone(),
                            file: r.location.file.display().to_string(),
                            line: r.location.range.start.line,
                            column: r.location.range.start.character,
                        })
                        .collect(),
                    effort_minutes: diag.effort_minutes,
                    tags: diag.tags.clone(),
                });
            }
        }

        let output = JsonOutput {
            diagnostics,
            summary: JsonSummary {
                total: blockers + errors + warnings + info,
                blockers,
                errors,
                warnings,
                info,
                bugs,
                vulnerabilities,
                code_smells,
                effort_minutes: total_effort,
            },
        };

        serde_json::to_string_pretty(&output).unwrap_or_else(|_| "{}".to_string())
    }

    fn format_diagnostic(&self, diag: &Diagnostic) -> String {
        let json_diag = JsonDiagnostic {
            rule_id: diag.rule_id.clone(),
            category: category_to_string(diag.category).to_string(),
            issue_type: issue_type_to_string(diag.issue_type).to_string(),
            severity: severity_to_string(diag.severity).to_string(),
            message: diag.message.clone(),
            file: diag.location.file.display().to_string(),
            line: diag.location.range.start.line,
            column: diag.location.range.start.character,
            end_line: diag.location.range.end.line,
            end_column: diag.location.range.end.character,
            help: diag.help.clone(),
            fix: diag.fix.as_ref().map(|f| JsonFix {
                description: f.description.clone(),
            }),
            related: diag
                .related
                .iter()
                .map(|r| JsonRelated {
                    message: r.message.clone(),
                    file: r.location.file.display().to_string(),
                    line: r.location.range.start.line,
                    column: r.location.range.start.character,
                })
                .collect(),
            effort_minutes: diag.effort_minutes,
            tags: diag.tags.clone(),
        };

        serde_json::to_string(&json_diag).unwrap_or_else(|_| "{}".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{Category, Fix, FixAction, Location, Position, Range, RelatedInfo};
    use std::path::PathBuf;

    fn make_location() -> Location {
        Location::new(
            PathBuf::from("test.wxs"),
            Range::new(Position::new(10, 5), Position::new(10, 20)),
        )
    }

    #[test]
    fn test_json_formatter_default() {
        let formatter = JsonFormatter::default();
        let results: Vec<AnalysisResult> = vec![];
        let output = formatter.format(&results);
        assert!(output.contains("diagnostics"));
    }

    #[test]
    fn test_json_output() {
        let formatter = JsonFormatter::new();
        let results = vec![AnalysisResult {
            files: Vec::new(),
            diagnostics: vec![Diagnostic::error(
                "TEST-001",
                Category::Validation,
                "Test error",
                make_location(),
            )],
        }];

        let output = formatter.format(&results);
        assert!(output.contains("\"rule_id\": \"TEST-001\""));
        assert!(output.contains("\"severity\": \"error\""));
        assert!(output.contains("\"line\": 10"));
    }

    #[test]
    fn test_json_summary() {
        let formatter = JsonFormatter::new();
        let results = vec![AnalysisResult {
            files: Vec::new(),
            diagnostics: vec![
                Diagnostic::error("E1", Category::Validation, "Error", make_location()),
                Diagnostic::warning("W1", Category::BestPractice, "Warning", make_location()),
            ],
        }];

        let output = formatter.format(&results);
        assert!(output.contains("\"errors\": 1"));
        assert!(output.contains("\"warnings\": 1"));
        assert!(output.contains("\"total\": 2"));
    }

    #[test]
    fn test_json_all_categories() {
        let formatter = JsonFormatter::new();
        let results = vec![AnalysisResult {
            files: Vec::new(),
            diagnostics: vec![
                Diagnostic::error(
                    "VAL-001",
                    Category::Validation,
                    "Validation",
                    make_location(),
                ),
                Diagnostic::warning(
                    "BP-001",
                    Category::BestPractice,
                    "Best practice",
                    make_location(),
                ),
                Diagnostic::warning("SEC-001", Category::Security, "Security", make_location()),
                Diagnostic::warning("DEAD-001", Category::DeadCode, "Dead code", make_location()),
            ],
        }];

        let output = formatter.format(&results);
        assert!(output.contains("\"category\": \"validation\""));
        assert!(output.contains("\"category\": \"best-practice\""));
        assert!(output.contains("\"category\": \"security\""));
        assert!(output.contains("\"category\": \"dead-code\""));
    }

    #[test]
    fn test_json_info_severity() {
        let formatter = JsonFormatter::new();
        let results = vec![AnalysisResult {
            files: Vec::new(),
            diagnostics: vec![Diagnostic::info(
                "INFO-001",
                Category::BestPractice,
                "Info message",
                make_location(),
            )],
        }];

        let output = formatter.format(&results);
        assert!(output.contains("\"severity\": \"hint\"")); // Info maps to hint
        assert!(output.contains("\"info\": 1"));
    }

    #[test]
    fn test_json_with_help() {
        let formatter = JsonFormatter::new();
        let results = vec![AnalysisResult {
            files: Vec::new(),
            diagnostics: vec![Diagnostic::error(
                "E1",
                Category::Validation,
                "Error",
                make_location(),
            )
            .with_help("This is help text")],
        }];

        let output = formatter.format(&results);
        assert!(output.contains("\"help\": \"This is help text\""));
    }

    #[test]
    fn test_json_with_fix() {
        let formatter = JsonFormatter::new();
        let fix = Fix::new(
            "Add missing attribute",
            FixAction::AddAttribute {
                range: Range::new(Position::new(1, 1), Position::new(1, 10)),
                name: "Id".to_string(),
                value: "MyId".to_string(),
            },
        );
        let results = vec![AnalysisResult {
            files: Vec::new(),
            diagnostics: vec![Diagnostic::error(
                "E1",
                Category::Validation,
                "Error",
                make_location(),
            )
            .with_fix(fix)],
        }];

        let output = formatter.format(&results);
        assert!(output.contains("\"fix\""));
        assert!(output.contains("\"description\": \"Add missing attribute\""));
    }

    #[test]
    fn test_json_with_related() {
        let formatter = JsonFormatter::new();
        let related_loc = Location::new(
            PathBuf::from("other.wxs"),
            Range::new(Position::new(5, 1), Position::new(5, 10)),
        );
        let results = vec![AnalysisResult {
            files: Vec::new(),
            diagnostics: vec![Diagnostic::error(
                "E1",
                Category::Validation,
                "Error",
                make_location(),
            )
            .with_related(RelatedInfo::new(related_loc, "Related info"))],
        }];

        let output = formatter.format(&results);
        assert!(output.contains("\"related\""));
        assert!(output.contains("\"message\": \"Related info\""));
        assert!(output.contains("other.wxs"));
    }

    #[test]
    fn test_format_diagnostic() {
        let formatter = JsonFormatter::new();
        let diag = Diagnostic::error("E1", Category::Validation, "Error", make_location());

        let output = formatter.format_diagnostic(&diag);
        assert!(output.contains("\"rule_id\":\"E1\""));
        assert!(output.contains("\"severity\":\"error\""));
    }

    #[test]
    fn test_format_diagnostic_with_all_fields() {
        let formatter = JsonFormatter::new();
        let related_loc = Location::new(
            PathBuf::from("other.wxs"),
            Range::new(Position::new(5, 1), Position::new(5, 10)),
        );
        let fix = Fix::new(
            "Fix it",
            FixAction::RemoveElement {
                range: Range::new(Position::new(1, 1), Position::new(1, 10)),
            },
        );
        let diag = Diagnostic::error("E1", Category::Security, "Security error", make_location())
            .with_help("Help text")
            .with_fix(fix)
            .with_related(RelatedInfo::new(related_loc, "See here"));

        let output = formatter.format_diagnostic(&diag);
        assert!(output.contains("\"help\":\"Help text\""));
        assert!(output.contains("\"fix\""));
        assert!(output.contains("\"related\""));
        assert!(output.contains("\"category\":\"security\""));
    }
}
