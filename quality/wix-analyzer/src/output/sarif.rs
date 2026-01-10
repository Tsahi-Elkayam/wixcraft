//! SARIF (Static Analysis Results Interchange Format) output
//! https://docs.oasis-open.org/sarif/sarif/v2.1.0/sarif-v2.1.0.html

use crate::core::{AnalysisResult, Category, Diagnostic, Severity};
use serde::Serialize;
use std::collections::HashMap;
use super::Formatter;

const SARIF_VERSION: &str = "2.1.0";
const SARIF_SCHEMA: &str = "https://raw.githubusercontent.com/oasis-tcs/sarif-spec/master/Schemata/sarif-schema-2.1.0.json";
const TOOL_NAME: &str = "wix-analyzer";
const TOOL_VERSION: &str = env!("CARGO_PKG_VERSION");

/// SARIF formatter
pub struct SarifFormatter;

impl SarifFormatter {
    pub fn new() -> Self {
        Self
    }
}

impl Default for SarifFormatter {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SarifLog {
    #[serde(rename = "$schema")]
    schema: &'static str,
    version: &'static str,
    runs: Vec<SarifRun>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SarifRun {
    tool: SarifTool,
    results: Vec<SarifResult>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SarifTool {
    driver: SarifDriver,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SarifDriver {
    name: &'static str,
    version: &'static str,
    information_uri: &'static str,
    rules: Vec<SarifRule>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SarifRule {
    id: String,
    short_description: SarifMessage,
    #[serde(skip_serializing_if = "Option::is_none")]
    full_description: Option<SarifMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    help: Option<SarifMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    help_uri: Option<String>,
    default_configuration: SarifConfiguration,
    properties: SarifRuleProperties,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SarifConfiguration {
    level: &'static str,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SarifRuleProperties {
    category: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    tags: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    security_severity: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SarifResult {
    rule_id: String,
    level: &'static str,
    message: SarifMessage,
    locations: Vec<SarifLocation>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    related_locations: Vec<SarifRelatedLocation>,
    /// Stable fingerprints for tracking issues across runs
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    partial_fingerprints: HashMap<String, String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    properties: Option<SarifResultProperties>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SarifResultProperties {
    #[serde(skip_serializing_if = "Option::is_none")]
    effort_minutes: Option<u32>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    tags: Vec<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SarifMessage {
    text: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SarifLocation {
    physical_location: SarifPhysicalLocation,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SarifRelatedLocation {
    id: usize,
    physical_location: SarifPhysicalLocation,
    message: SarifMessage,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SarifPhysicalLocation {
    artifact_location: SarifArtifactLocation,
    region: SarifRegion,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SarifArtifactLocation {
    uri: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SarifRegion {
    start_line: usize,
    start_column: usize,
    end_line: usize,
    end_column: usize,
}

fn severity_to_level(severity: Severity) -> &'static str {
    match severity {
        Severity::Blocker | Severity::High => "error",
        Severity::Medium => "warning",
        Severity::Low => "note",
        Severity::Info => "none",
    }
}

/// Map severity to CVSS-style security severity (for GitHub)
fn severity_to_security_severity(severity: Severity) -> &'static str {
    match severity {
        Severity::Blocker => "10.0",
        Severity::High => "8.0",
        Severity::Medium => "5.0",
        Severity::Low => "3.0",
        Severity::Info => "1.0",
    }
}

fn category_to_string(cat: Category) -> String {
    match cat {
        Category::Validation => "Validation".to_string(),
        Category::BestPractice => "BestPractice".to_string(),
        Category::Security => "Security".to_string(),
        Category::DeadCode => "DeadCode".to_string(),
    }
}

/// Build tags from diagnostic (including CWE/OWASP)
fn build_tags(diag: &Diagnostic) -> Vec<String> {
    let mut tags = diag.tags.clone();

    // Add security standard tags
    if let Some(ref sec) = diag.security {
        if let Some(ref cwe) = sec.cwe {
            tags.push(format!("external/cwe/{}", cwe.to_lowercase().replace("cwe-", "")));
        }
        if let Some(ref owasp) = sec.owasp {
            tags.push(format!("external/owasp/{}", owasp.to_lowercase().replace(":", "-")));
        }
        if let Some(rank) = sec.sans_top25 {
            tags.push(format!("external/sans-top25/{}", rank));
        }
    }

    // Add category as tag
    tags.push(diag.category.as_str().to_string());

    tags
}

impl Formatter for SarifFormatter {
    fn format(&self, results: &[AnalysisResult]) -> String {
        let mut all_diagnostics: Vec<&Diagnostic> = Vec::new();
        let mut rules_map = std::collections::HashMap::new();

        for result in results {
            for diag in &result.diagnostics {
                all_diagnostics.push(diag);

                // Collect unique rules
                if !rules_map.contains_key(&diag.rule_id) {
                    let tags = build_tags(diag);
                    let is_security = diag.category == Category::Security || diag.security.is_some();

                    rules_map.insert(
                        diag.rule_id.clone(),
                        SarifRule {
                            id: diag.rule_id.clone(),
                            short_description: SarifMessage {
                                text: first_sentence(&diag.message),
                            },
                            full_description: Some(SarifMessage {
                                text: diag.message.clone(),
                            }),
                            help: diag.help.as_ref().map(|h| SarifMessage { text: h.clone() }),
                            help_uri: diag.doc_url.clone(),
                            default_configuration: SarifConfiguration {
                                level: severity_to_level(diag.severity),
                            },
                            properties: SarifRuleProperties {
                                category: category_to_string(diag.category),
                                tags,
                                security_severity: if is_security {
                                    Some(severity_to_security_severity(diag.severity).to_string())
                                } else {
                                    None
                                },
                            },
                        },
                    );
                }
            }
        }

        let sarif_results: Vec<SarifResult> = all_diagnostics
            .iter()
            .map(|diag| {
                let related_locations: Vec<SarifRelatedLocation> = diag
                    .related
                    .iter()
                    .enumerate()
                    .map(|(idx, rel)| SarifRelatedLocation {
                        id: idx + 1,
                        physical_location: SarifPhysicalLocation {
                            artifact_location: SarifArtifactLocation {
                                uri: rel.location.file.display().to_string(),
                            },
                            region: SarifRegion {
                                start_line: rel.location.range.start.line,
                                start_column: rel.location.range.start.character,
                                end_line: rel.location.range.end.line,
                                end_column: rel.location.range.end.character,
                            },
                        },
                        message: SarifMessage {
                            text: rel.message.clone(),
                        },
                    })
                    .collect();

                // Build partial fingerprints for issue tracking
                let mut partial_fingerprints = HashMap::new();
                partial_fingerprints.insert(
                    "primaryLocationLineHash/v1".to_string(),
                    diag.fingerprint(),
                );

                // Build result properties
                let has_props = diag.effort_minutes.is_some() || !diag.tags.is_empty();
                let properties = if has_props {
                    Some(SarifResultProperties {
                        effort_minutes: diag.effort_minutes,
                        tags: diag.tags.clone(),
                    })
                } else {
                    None
                };

                SarifResult {
                    rule_id: diag.rule_id.clone(),
                    level: severity_to_level(diag.severity),
                    message: SarifMessage {
                        text: diag.message.clone(),
                    },
                    locations: vec![SarifLocation {
                        physical_location: SarifPhysicalLocation {
                            artifact_location: SarifArtifactLocation {
                                uri: diag.location.file.display().to_string(),
                            },
                            region: SarifRegion {
                                start_line: diag.location.range.start.line,
                                start_column: diag.location.range.start.character,
                                end_line: diag.location.range.end.line,
                                end_column: diag.location.range.end.character,
                            },
                        },
                    }],
                    related_locations,
                    partial_fingerprints,
                    properties,
                }
            })
            .collect();

        let log = SarifLog {
            schema: SARIF_SCHEMA,
            version: SARIF_VERSION,
            runs: vec![SarifRun {
                tool: SarifTool {
                    driver: SarifDriver {
                        name: TOOL_NAME,
                        version: TOOL_VERSION,
                        information_uri: "https://github.com/hyperlight/wixcraft",
                        rules: rules_map.into_values().collect(),
                    },
                },
                results: sarif_results,
            }],
        };

        serde_json::to_string_pretty(&log).unwrap_or_else(|_| "{}".to_string())
    }

    fn format_diagnostic(&self, diag: &Diagnostic) -> String {
        // For single diagnostic, create a minimal SARIF with just that result
        let results = vec![AnalysisResult {
            files: Vec::new(),
            diagnostics: vec![diag.clone()],
        }];
        self.format(&results)
    }
}

/// Extract first sentence from a message (for short description)
fn first_sentence(text: &str) -> String {
    text.split_once(". ")
        .map(|(first, _)| format!("{}.", first))
        .unwrap_or_else(|| text.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{Category, Location, Position, Range, RelatedInfo};
    use std::path::PathBuf;

    fn make_location() -> Location {
        Location::new(
            PathBuf::from("test.wxs"),
            Range::new(Position::new(10, 5), Position::new(10, 20)),
        )
    }

    #[test]
    fn test_sarif_formatter_default() {
        let formatter = SarifFormatter::default();
        let results: Vec<AnalysisResult> = vec![];
        let output = formatter.format(&results);
        assert!(output.contains("\"$schema\""));
    }

    #[test]
    fn test_sarif_structure() {
        let formatter = SarifFormatter::new();
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
        assert!(output.contains("\"$schema\""));
        assert!(output.contains("\"version\": \"2.1.0\""));
        assert!(output.contains("\"driver\""));
        assert!(output.contains("\"results\""));
    }

    #[test]
    fn test_sarif_result() {
        let formatter = SarifFormatter::new();
        let results = vec![AnalysisResult {
            files: Vec::new(),
            diagnostics: vec![Diagnostic::warning(
                "BP-001",
                Category::BestPractice,
                "Best practice warning",
                Location::new(
                    PathBuf::from("product.wxs"),
                    Range::new(Position::new(5, 1), Position::new(5, 50)),
                ),
            )],
        }];

        let output = formatter.format(&results);
        assert!(output.contains("\"ruleId\": \"BP-001\""));
        assert!(output.contains("\"level\": \"warning\""));
        assert!(output.contains("\"startLine\": 5"));
    }

    #[test]
    fn test_sarif_fingerprints() {
        let formatter = SarifFormatter::new();
        let results = vec![AnalysisResult {
            files: Vec::new(),
            diagnostics: vec![Diagnostic::error(
                "VAL-001",
                Category::Validation,
                "Test error",
                make_location(),
            )],
        }];

        let output = formatter.format(&results);
        assert!(output.contains("\"partialFingerprints\""));
        assert!(output.contains("\"primaryLocationLineHash/v1\""));
    }

    #[test]
    fn test_sarif_with_cwe_owasp() {
        let formatter = SarifFormatter::new();
        let diag = Diagnostic::high("SEC-001", crate::core::IssueType::Vulnerability, "SQL Injection", make_location())
            .with_cwe("CWE-89")
            .with_owasp("A03:2021-Injection");

        let results = vec![AnalysisResult {
            files: Vec::new(),
            diagnostics: vec![diag],
        }];

        let output = formatter.format(&results);
        assert!(output.contains("external/cwe/89"));
        assert!(output.contains("external/owasp/a03-2021-injection"));
        assert!(output.contains("\"securitySeverity\""));
    }

    #[test]
    fn test_sarif_with_doc_url() {
        let formatter = SarifFormatter::new();
        let diag = Diagnostic::error("VAL-001", Category::Validation, "Error", make_location())
            .with_doc_url("https://docs.example.com/rules/VAL-001");

        let results = vec![AnalysisResult {
            files: Vec::new(),
            diagnostics: vec![diag],
        }];

        let output = formatter.format(&results);
        assert!(output.contains("\"helpUri\""));
        assert!(output.contains("https://docs.example.com/rules/VAL-001"));
    }

    #[test]
    fn test_sarif_rules() {
        let formatter = SarifFormatter::new();
        let results = vec![AnalysisResult {
            files: Vec::new(),
            diagnostics: vec![
                Diagnostic::error(
                    "VAL-001",
                    Category::Validation,
                    "Validation error",
                    make_location(),
                ),
                Diagnostic::error(
                    "VAL-001",
                    Category::Validation,
                    "Another validation error",
                    Location::new(
                        PathBuf::from("test.wxs"),
                        Range::new(Position::new(2, 1), Position::new(2, 10)),
                    ),
                ),
            ],
        }];

        let output = formatter.format(&results);
        // Should only have one rule definition even with multiple results
        let rule_count = output.matches("\"id\": \"VAL-001\"").count();
        assert_eq!(rule_count, 1, "Rule should only be defined once");
    }

    #[test]
    fn test_sarif_info_level() {
        let formatter = SarifFormatter::new();
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
        assert!(output.contains("\"level\": \"none\"")); // Info maps to none in SARIF
    }

    #[test]
    fn test_sarif_all_categories() {
        let formatter = SarifFormatter::new();
        let results = vec![AnalysisResult {
            files: Vec::new(),
            diagnostics: vec![
                Diagnostic::error("VAL-001", Category::Validation, "Val", make_location()),
                Diagnostic::warning("BP-001", Category::BestPractice, "BP", make_location()),
                Diagnostic::warning("SEC-001", Category::Security, "Sec", make_location()),
                Diagnostic::warning("DEAD-001", Category::DeadCode, "Dead", make_location()),
            ],
        }];

        let output = formatter.format(&results);
        assert!(output.contains("\"category\": \"Validation\""));
        assert!(output.contains("\"category\": \"BestPractice\""));
        assert!(output.contains("\"category\": \"Security\""));
        assert!(output.contains("\"category\": \"DeadCode\""));
    }

    #[test]
    fn test_sarif_with_help() {
        let formatter = SarifFormatter::new();
        let results = vec![AnalysisResult {
            files: Vec::new(),
            diagnostics: vec![Diagnostic::error("E1", Category::Validation, "Error", make_location())
                .with_help("Help text here")],
        }];

        let output = formatter.format(&results);
        assert!(output.contains("\"help\""));
        assert!(output.contains("Help text here"));
    }

    #[test]
    fn test_sarif_with_related_locations() {
        let formatter = SarifFormatter::new();
        let related_loc = Location::new(
            PathBuf::from("other.wxs"),
            Range::new(Position::new(5, 1), Position::new(5, 10)),
        );
        let results = vec![AnalysisResult {
            files: Vec::new(),
            diagnostics: vec![Diagnostic::error("E1", Category::Validation, "Error", make_location())
                .with_related(RelatedInfo::new(related_loc, "See definition"))],
        }];

        let output = formatter.format(&results);
        assert!(output.contains("\"relatedLocations\""));
        assert!(output.contains("\"id\": 1")); // First related location gets id 1
        assert!(output.contains("other.wxs"));
        assert!(output.contains("See definition"));
    }

    #[test]
    fn test_sarif_format_diagnostic() {
        let formatter = SarifFormatter::new();
        let diag = Diagnostic::error("E1", Category::Validation, "Error", make_location());

        let output = formatter.format_diagnostic(&diag);
        assert!(output.contains("\"$schema\""));
        assert!(output.contains("\"ruleId\": \"E1\""));
    }

    #[test]
    fn test_sarif_with_effort() {
        let formatter = SarifFormatter::new();
        let diag = Diagnostic::error("E1", Category::Validation, "Error", make_location())
            .with_effort(30);

        let results = vec![AnalysisResult {
            files: Vec::new(),
            diagnostics: vec![diag],
        }];

        let output = formatter.format(&results);
        assert!(output.contains("\"effortMinutes\": 30"));
    }

    #[test]
    fn test_first_sentence() {
        assert_eq!(first_sentence("Hello world. More text."), "Hello world.");
        assert_eq!(first_sentence("No period here"), "No period here");
        assert_eq!(first_sentence("Single."), "Single.");
    }
}
