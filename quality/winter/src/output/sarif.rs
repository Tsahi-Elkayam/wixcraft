//! SARIF (Static Analysis Results Interchange Format) output formatter
//!
//! SARIF is a standard format for static analysis tools, supported by
//! GitHub Actions, Azure DevOps, and other CI/CD systems.

use super::OutputFormatter;
use crate::diagnostic::{Diagnostic, Severity};
use crate::engine::LintResult;
use serde::Serialize;

/// SARIF formatter for CI/CD integration
#[derive(Default)]
pub struct SarifFormatter {
    /// Tool name
    pub tool_name: String,

    /// Tool version
    pub tool_version: String,
}

impl SarifFormatter {
    /// Create a new SARIF formatter
    pub fn new(tool_name: &str, tool_version: &str) -> Self {
        Self {
            tool_name: tool_name.to_string(),
            tool_version: tool_version.to_string(),
        }
    }
}

#[derive(Serialize)]
struct SarifReport {
    #[serde(rename = "$schema")]
    schema: &'static str,
    version: &'static str,
    runs: Vec<SarifRun>,
}

#[derive(Serialize)]
struct SarifRun {
    tool: SarifTool,
    results: Vec<SarifResult>,
}

#[derive(Serialize)]
struct SarifTool {
    driver: SarifDriver,
}

#[derive(Serialize)]
struct SarifDriver {
    name: String,
    version: String,
    #[serde(rename = "informationUri")]
    information_uri: &'static str,
    rules: Vec<SarifRule>,
}

#[derive(Serialize)]
struct SarifRule {
    id: String,
    #[serde(rename = "shortDescription")]
    short_description: SarifMessage,
    #[serde(rename = "defaultConfiguration")]
    default_configuration: SarifConfiguration,
}

#[derive(Serialize)]
struct SarifConfiguration {
    level: &'static str,
}

#[derive(Serialize)]
struct SarifResult {
    #[serde(rename = "ruleId")]
    rule_id: String,
    level: &'static str,
    message: SarifMessage,
    locations: Vec<SarifLocation>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    fixes: Vec<SarifFix>,
}

#[derive(Serialize)]
struct SarifMessage {
    text: String,
}

#[derive(Serialize)]
struct SarifLocation {
    #[serde(rename = "physicalLocation")]
    physical_location: SarifPhysicalLocation,
}

#[derive(Serialize)]
struct SarifPhysicalLocation {
    #[serde(rename = "artifactLocation")]
    artifact_location: SarifArtifactLocation,
    region: SarifRegion,
}

#[derive(Serialize)]
struct SarifArtifactLocation {
    uri: String,
}

#[derive(Serialize)]
struct SarifRegion {
    #[serde(rename = "startLine")]
    start_line: usize,
    #[serde(rename = "startColumn")]
    start_column: usize,
    #[serde(rename = "endColumn", skip_serializing_if = "Option::is_none")]
    end_column: Option<usize>,
}

#[derive(Serialize)]
struct SarifFix {
    description: SarifMessage,
    #[serde(rename = "artifactChanges")]
    artifact_changes: Vec<SarifArtifactChange>,
}

#[derive(Serialize)]
struct SarifArtifactChange {
    #[serde(rename = "artifactLocation")]
    artifact_location: SarifArtifactLocation,
    replacements: Vec<SarifReplacement>,
}

#[derive(Serialize)]
struct SarifReplacement {
    #[serde(rename = "deletedRegion")]
    deleted_region: SarifRegion,
    #[serde(rename = "insertedContent")]
    inserted_content: SarifContent,
}

#[derive(Serialize)]
struct SarifContent {
    text: String,
}

fn severity_to_level(severity: Severity) -> &'static str {
    match severity {
        Severity::Error => "error",
        Severity::Warning => "warning",
        Severity::Info => "note",
    }
}

impl OutputFormatter for SarifFormatter {
    fn format(&self, result: &LintResult) -> String {
        // Collect unique rules
        let mut rules_map = std::collections::HashMap::new();
        for diag in &result.diagnostics {
            rules_map
                .entry(diag.rule_id.clone())
                .or_insert_with(|| SarifRule {
                    id: diag.rule_id.clone(),
                    short_description: SarifMessage {
                        text: diag.help.clone().unwrap_or_else(|| diag.message.clone()),
                    },
                    default_configuration: SarifConfiguration {
                        level: severity_to_level(diag.severity),
                    },
                });
        }

        let rules: Vec<SarifRule> = rules_map.into_values().collect();

        let results: Vec<SarifResult> = result
            .diagnostics
            .iter()
            .map(|d| {
                let fixes = if let Some(fix) = &d.fix {
                    vec![SarifFix {
                        description: SarifMessage {
                            text: fix.description.clone(),
                        },
                        artifact_changes: vec![SarifArtifactChange {
                            artifact_location: SarifArtifactLocation {
                                uri: d.location.file.display().to_string(),
                            },
                            replacements: vec![SarifReplacement {
                                deleted_region: SarifRegion {
                                    start_line: d.location.line,
                                    start_column: d.location.column,
                                    end_column: if d.location.length > 0 {
                                        Some(d.location.column + d.location.length)
                                    } else {
                                        None
                                    },
                                },
                                inserted_content: SarifContent {
                                    text: fix.replacement.clone(),
                                },
                            }],
                        }],
                    }]
                } else {
                    vec![]
                };

                SarifResult {
                    rule_id: d.rule_id.clone(),
                    level: severity_to_level(d.severity),
                    message: SarifMessage {
                        text: d.message.clone(),
                    },
                    locations: vec![SarifLocation {
                        physical_location: SarifPhysicalLocation {
                            artifact_location: SarifArtifactLocation {
                                uri: d.location.file.display().to_string(),
                            },
                            region: SarifRegion {
                                start_line: d.location.line,
                                start_column: d.location.column,
                                end_column: if d.location.length > 0 {
                                    Some(d.location.column + d.location.length)
                                } else {
                                    None
                                },
                            },
                        },
                    }],
                    fixes,
                }
            })
            .collect();

        let report = SarifReport {
            schema: "https://raw.githubusercontent.com/oasis-tcs/sarif-spec/master/Schemata/sarif-schema-2.1.0.json",
            version: "2.1.0",
            runs: vec![SarifRun {
                tool: SarifTool {
                    driver: SarifDriver {
                        name: self.tool_name.clone(),
                        version: self.tool_version.clone(),
                        information_uri: "https://github.com/Tsahi-Elkayam/wixcraft",
                        rules,
                    },
                },
                results,
            }],
        };

        serde_json::to_string_pretty(&report).unwrap_or_default()
    }

    fn format_diagnostic(&self, diagnostic: &Diagnostic) -> String {
        let result = SarifResult {
            rule_id: diagnostic.rule_id.clone(),
            level: severity_to_level(diagnostic.severity),
            message: SarifMessage {
                text: diagnostic.message.clone(),
            },
            locations: vec![SarifLocation {
                physical_location: SarifPhysicalLocation {
                    artifact_location: SarifArtifactLocation {
                        uri: diagnostic.location.file.display().to_string(),
                    },
                    region: SarifRegion {
                        start_line: diagnostic.location.line,
                        start_column: diagnostic.location.column,
                        end_column: None,
                    },
                },
            }],
            fixes: vec![],
        };

        serde_json::to_string_pretty(&result).unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagnostic::Location;
    use std::path::PathBuf;

    #[test]
    fn test_sarif_format() {
        let formatter = SarifFormatter::new("winter", "0.1.0");
        let result = LintResult {
            diagnostics: vec![Diagnostic::new(
                "test-rule",
                Severity::Error,
                "Test message",
                Location::new(PathBuf::from("test.wxs"), 10, 5),
            )],
            files_processed: 1,
            error_count: 1,
            ..Default::default()
        };

        let output = formatter.format(&result);
        assert!(output.contains("sarif-schema-2.1.0.json"));
        assert!(output.contains("\"version\": \"2.1.0\""));
        assert!(output.contains("test-rule"));
        assert!(output.contains("\"level\": \"error\""));
    }

    #[test]
    fn test_sarif_severity_mapping() {
        assert_eq!(severity_to_level(Severity::Error), "error");
        assert_eq!(severity_to_level(Severity::Warning), "warning");
        assert_eq!(severity_to_level(Severity::Info), "note");
    }
}
