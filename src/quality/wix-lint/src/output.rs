//! Output formatters for lint results

use crate::diagnostics::{Diagnostic, Severity};
use serde::Serialize;
use std::io::{self, Write};

/// Print diagnostics in human-readable text format
pub fn print_text(diagnostics: &[Diagnostic]) {
    let stdout = io::stdout();
    let mut handle = stdout.lock();

    for diag in diagnostics {
        // Header: severity[rule-id]: message
        let _ = writeln!(
            handle,
            "{}[{}]: {}",
            diag.severity.colored(),
            diag.rule_id,
            diag.message
        );

        // Location: --> file:line:column
        let _ = writeln!(
            handle,
            "  \x1b[1;34m-->\x1b[0m {}:{}:{}",
            diag.location.file.display(),
            diag.location.line,
            diag.location.column
        );

        // Source line with line number
        if let Some(ref source) = diag.source_line {
            let line_num = diag.location.line.to_string();
            let padding = " ".repeat(line_num.len());

            let _ = writeln!(handle, "   \x1b[1;34m{}\x1b[0m |", padding);
            let _ = writeln!(handle, " \x1b[1;34m{}\x1b[0m | {}", line_num, source);

            // Underline the problematic part
            let col = diag.location.column.saturating_sub(1);
            let underline_padding = " ".repeat(col);
            let underline = "^".repeat(diag.location.length.max(1));
            let underline_color = match diag.severity {
                Severity::Error => "\x1b[1;31m",
                Severity::Warning => "\x1b[1;33m",
                Severity::Info => "\x1b[1;36m",
            };

            let _ = writeln!(
                handle,
                "   \x1b[1;34m{}\x1b[0m | {}{}{}\x1b[0m",
                padding, underline_padding, underline_color, underline
            );
        }

        // Help text
        if let Some(ref help) = diag.help {
            let _ = writeln!(handle, "   \x1b[1;34m=\x1b[0m \x1b[1mhelp\x1b[0m: {}", help);
        }

        // Fix suggestion
        if let Some(ref fix) = diag.fix {
            let _ = writeln!(
                handle,
                "   \x1b[1;34m=\x1b[0m \x1b[1;32mfix\x1b[0m: {}",
                fix.description
            );
        }

        let _ = writeln!(handle);
    }
}

/// JSON output format
#[derive(Serialize)]
struct JsonOutput<'a> {
    diagnostics: Vec<JsonDiagnostic<'a>>,
    summary: JsonSummary,
}

#[derive(Serialize)]
struct JsonDiagnostic<'a> {
    rule_id: &'a str,
    severity: &'a str,
    message: &'a str,
    file: String,
    line: usize,
    column: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    help: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    fix: Option<JsonFix<'a>>,
}

#[derive(Serialize)]
struct JsonFix<'a> {
    description: &'a str,
    replacement: &'a str,
}

#[derive(Serialize)]
struct JsonSummary {
    total: usize,
    errors: usize,
    warnings: usize,
    info: usize,
}

/// Print diagnostics in JSON format
pub fn print_json(diagnostics: &[Diagnostic]) -> io::Result<()> {
    let json_diagnostics: Vec<JsonDiagnostic> = diagnostics
        .iter()
        .map(|d| JsonDiagnostic {
            rule_id: &d.rule_id,
            severity: d.severity.as_str(),
            message: &d.message,
            file: d.location.file.display().to_string(),
            line: d.location.line,
            column: d.location.column,
            help: d.help.as_deref(),
            fix: d.fix.as_ref().map(|f| JsonFix {
                description: &f.description,
                replacement: &f.replacement,
            }),
        })
        .collect();

    let summary = JsonSummary {
        total: diagnostics.len(),
        errors: diagnostics
            .iter()
            .filter(|d| d.severity == Severity::Error)
            .count(),
        warnings: diagnostics
            .iter()
            .filter(|d| d.severity == Severity::Warning)
            .count(),
        info: diagnostics
            .iter()
            .filter(|d| d.severity == Severity::Info)
            .count(),
    };

    let output = JsonOutput {
        diagnostics: json_diagnostics,
        summary,
    };

    let stdout = io::stdout();
    let handle = stdout.lock();
    serde_json::to_writer_pretty(handle, &output)?;
    println!();

    Ok(())
}

/// SARIF (Static Analysis Results Interchange Format) output
#[derive(Serialize)]
struct SarifOutput<'a> {
    #[serde(rename = "$schema")]
    schema: &'static str,
    version: &'static str,
    runs: Vec<SarifRun<'a>>,
}

#[derive(Serialize)]
struct SarifRun<'a> {
    tool: SarifTool,
    results: Vec<SarifResult<'a>>,
}

#[derive(Serialize)]
struct SarifTool {
    driver: SarifDriver,
}

#[derive(Serialize)]
struct SarifDriver {
    name: &'static str,
    version: &'static str,
    #[serde(rename = "informationUri")]
    information_uri: &'static str,
}

#[derive(Serialize)]
struct SarifResult<'a> {
    #[serde(rename = "ruleId")]
    rule_id: &'a str,
    level: &'static str,
    message: SarifMessage<'a>,
    locations: Vec<SarifLocation<'a>>,
}

#[derive(Serialize)]
struct SarifMessage<'a> {
    text: &'a str,
}

#[derive(Serialize)]
struct SarifLocation<'a> {
    #[serde(rename = "physicalLocation")]
    physical_location: SarifPhysicalLocation<'a>,
}

#[derive(Serialize)]
struct SarifPhysicalLocation<'a> {
    #[serde(rename = "artifactLocation")]
    artifact_location: SarifArtifactLocation<'a>,
    region: SarifRegion,
}

#[derive(Serialize)]
struct SarifArtifactLocation<'a> {
    uri: &'a str,
}

#[derive(Serialize)]
struct SarifRegion {
    #[serde(rename = "startLine")]
    start_line: usize,
    #[serde(rename = "startColumn")]
    start_column: usize,
}

/// Print diagnostics in SARIF format (for CI/CD integration)
pub fn print_sarif(diagnostics: &[Diagnostic]) -> io::Result<()> {
    let results: Vec<SarifResult> = diagnostics
        .iter()
        .map(|d| SarifResult {
            rule_id: &d.rule_id,
            level: match d.severity {
                Severity::Error => "error",
                Severity::Warning => "warning",
                Severity::Info => "note",
            },
            message: SarifMessage { text: &d.message },
            locations: vec![SarifLocation {
                physical_location: SarifPhysicalLocation {
                    artifact_location: SarifArtifactLocation {
                        uri: d.location.file.to_str().unwrap_or(""),
                    },
                    region: SarifRegion {
                        start_line: d.location.line,
                        start_column: d.location.column,
                    },
                },
            }],
        })
        .collect();

    let sarif = SarifOutput {
        schema: "https://raw.githubusercontent.com/oasis-tcs/sarif-spec/master/Schemata/sarif-schema-2.1.0.json",
        version: "2.1.0",
        runs: vec![SarifRun {
            tool: SarifTool {
                driver: SarifDriver {
                    name: "wix-lint",
                    version: env!("CARGO_PKG_VERSION"),
                    information_uri: "https://github.com/wixcraft/wixcraft",
                },
            },
            results,
        }],
    };

    let stdout = io::stdout();
    let handle = stdout.lock();
    serde_json::to_writer_pretty(handle, &sarif)?;
    println!();

    Ok(())
}

/// Format diagnostics as JSON string (for testing)
pub fn format_json(diagnostics: &[Diagnostic]) -> String {
    let json_diagnostics: Vec<JsonDiagnostic> = diagnostics
        .iter()
        .map(|d| JsonDiagnostic {
            rule_id: &d.rule_id,
            severity: d.severity.as_str(),
            message: &d.message,
            file: d.location.file.display().to_string(),
            line: d.location.line,
            column: d.location.column,
            help: d.help.as_deref(),
            fix: d.fix.as_ref().map(|f| JsonFix {
                description: &f.description,
                replacement: &f.replacement,
            }),
        })
        .collect();

    let summary = JsonSummary {
        total: diagnostics.len(),
        errors: diagnostics
            .iter()
            .filter(|d| d.severity == Severity::Error)
            .count(),
        warnings: diagnostics
            .iter()
            .filter(|d| d.severity == Severity::Warning)
            .count(),
        info: diagnostics
            .iter()
            .filter(|d| d.severity == Severity::Info)
            .count(),
    };

    let output = JsonOutput {
        diagnostics: json_diagnostics,
        summary,
    };

    serde_json::to_string_pretty(&output).unwrap_or_default()
}

/// Format diagnostics as SARIF string (for testing)
pub fn format_sarif(diagnostics: &[Diagnostic]) -> String {
    let results: Vec<SarifResult> = diagnostics
        .iter()
        .map(|d| SarifResult {
            rule_id: &d.rule_id,
            level: match d.severity {
                Severity::Error => "error",
                Severity::Warning => "warning",
                Severity::Info => "note",
            },
            message: SarifMessage { text: &d.message },
            locations: vec![SarifLocation {
                physical_location: SarifPhysicalLocation {
                    artifact_location: SarifArtifactLocation {
                        uri: d.location.file.to_str().unwrap_or(""),
                    },
                    region: SarifRegion {
                        start_line: d.location.line,
                        start_column: d.location.column,
                    },
                },
            }],
        })
        .collect();

    let sarif = SarifOutput {
        schema: "https://raw.githubusercontent.com/oasis-tcs/sarif-spec/master/Schemata/sarif-schema-2.1.0.json",
        version: "2.1.0",
        runs: vec![SarifRun {
            tool: SarifTool {
                driver: SarifDriver {
                    name: "wix-lint",
                    version: env!("CARGO_PKG_VERSION"),
                    information_uri: "https://github.com/wixcraft/wixcraft",
                },
            },
            results,
        }],
    };

    serde_json::to_string_pretty(&sarif).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagnostics::{Fix, Location};
    use std::path::PathBuf;

    fn make_test_diagnostic(severity: Severity, rule_id: &str) -> Diagnostic {
        Diagnostic {
            rule_id: rule_id.to_string(),
            severity,
            message: format!("Test message for {}", rule_id),
            location: Location {
                file: PathBuf::from("test.wxs"),
                line: 10,
                column: 5,
                length: 20,
            },
            source_line: Some("<Component Id=\"Test\">".to_string()),
            help: Some("Test help text".to_string()),
            fix: None,
        }
    }

    #[test]
    fn test_format_json_empty() {
        let output = format_json(&[]);
        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();

        assert_eq!(parsed["diagnostics"].as_array().unwrap().len(), 0);
        assert_eq!(parsed["summary"]["total"], 0);
        assert_eq!(parsed["summary"]["errors"], 0);
        assert_eq!(parsed["summary"]["warnings"], 0);
        assert_eq!(parsed["summary"]["info"], 0);
    }

    #[test]
    fn test_format_json_single_diagnostic() {
        let diag = make_test_diagnostic(Severity::Error, "test-rule");
        let output = format_json(&[diag]);
        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();

        assert_eq!(parsed["diagnostics"].as_array().unwrap().len(), 1);
        assert_eq!(parsed["diagnostics"][0]["rule_id"], "test-rule");
        assert_eq!(parsed["diagnostics"][0]["severity"], "error");
        assert_eq!(parsed["diagnostics"][0]["line"], 10);
        assert_eq!(parsed["diagnostics"][0]["column"], 5);
        assert_eq!(parsed["summary"]["total"], 1);
        assert_eq!(parsed["summary"]["errors"], 1);
    }

    #[test]
    fn test_format_json_multiple_diagnostics() {
        let diagnostics = vec![
            make_test_diagnostic(Severity::Error, "error-rule"),
            make_test_diagnostic(Severity::Warning, "warning-rule"),
            make_test_diagnostic(Severity::Info, "info-rule"),
        ];
        let output = format_json(&diagnostics);
        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();

        assert_eq!(parsed["diagnostics"].as_array().unwrap().len(), 3);
        assert_eq!(parsed["summary"]["total"], 3);
        assert_eq!(parsed["summary"]["errors"], 1);
        assert_eq!(parsed["summary"]["warnings"], 1);
        assert_eq!(parsed["summary"]["info"], 1);
    }

    #[test]
    fn test_format_json_with_fix() {
        let mut diag = make_test_diagnostic(Severity::Warning, "fixable-rule");
        diag.fix = Some(Fix {
            description: "Add Guid attribute".to_string(),
            replacement: "Guid=\"*\"".to_string(),
        });

        let output = format_json(&[diag]);
        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();

        assert!(parsed["diagnostics"][0]["fix"].is_object());
        assert_eq!(
            parsed["diagnostics"][0]["fix"]["description"],
            "Add Guid attribute"
        );
        assert_eq!(parsed["diagnostics"][0]["fix"]["replacement"], "Guid=\"*\"");
    }

    #[test]
    fn test_format_json_without_help() {
        let mut diag = make_test_diagnostic(Severity::Error, "no-help-rule");
        diag.help = None;

        let output = format_json(&[diag]);
        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();

        // help should be absent when None
        assert!(parsed["diagnostics"][0]["help"].is_null());
    }

    #[test]
    fn test_format_sarif_empty() {
        let output = format_sarif(&[]);
        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();

        assert_eq!(parsed["version"], "2.1.0");
        assert!(parsed["runs"][0]["results"].as_array().unwrap().is_empty());
        assert_eq!(parsed["runs"][0]["tool"]["driver"]["name"], "wix-lint");
    }

    #[test]
    fn test_format_sarif_single_diagnostic() {
        let diag = make_test_diagnostic(Severity::Error, "test-rule");
        let output = format_sarif(&[diag]);
        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();

        let results = parsed["runs"][0]["results"].as_array().unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0]["ruleId"], "test-rule");
        assert_eq!(results[0]["level"], "error");
        assert_eq!(
            results[0]["locations"][0]["physicalLocation"]["region"]["startLine"],
            10
        );
        assert_eq!(
            results[0]["locations"][0]["physicalLocation"]["region"]["startColumn"],
            5
        );
    }

    #[test]
    fn test_format_sarif_severity_levels() {
        let diagnostics = vec![
            make_test_diagnostic(Severity::Error, "error-rule"),
            make_test_diagnostic(Severity::Warning, "warning-rule"),
            make_test_diagnostic(Severity::Info, "info-rule"),
        ];
        let output = format_sarif(&diagnostics);
        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();

        let results = parsed["runs"][0]["results"].as_array().unwrap();
        assert_eq!(results[0]["level"], "error");
        assert_eq!(results[1]["level"], "warning");
        assert_eq!(results[2]["level"], "note"); // Info maps to "note" in SARIF
    }

    #[test]
    fn test_format_sarif_schema() {
        let output = format_sarif(&[]);
        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();

        assert!(parsed["$schema"]
            .as_str()
            .unwrap()
            .contains("sarif-schema-2.1.0.json"));
    }

    #[test]
    fn test_format_sarif_tool_info() {
        let output = format_sarif(&[]);
        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();

        let driver = &parsed["runs"][0]["tool"]["driver"];
        assert_eq!(driver["name"], "wix-lint");
        assert!(driver["informationUri"]
            .as_str()
            .unwrap()
            .contains("wixcraft"));
    }

    #[test]
    fn test_json_summary_counts() {
        let diagnostics = vec![
            make_test_diagnostic(Severity::Error, "e1"),
            make_test_diagnostic(Severity::Error, "e2"),
            make_test_diagnostic(Severity::Warning, "w1"),
            make_test_diagnostic(Severity::Warning, "w2"),
            make_test_diagnostic(Severity::Warning, "w3"),
            make_test_diagnostic(Severity::Info, "i1"),
        ];
        let output = format_json(&diagnostics);
        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();

        assert_eq!(parsed["summary"]["total"], 6);
        assert_eq!(parsed["summary"]["errors"], 2);
        assert_eq!(parsed["summary"]["warnings"], 3);
        assert_eq!(parsed["summary"]["info"], 1);
    }

    #[test]
    fn test_format_json_file_path() {
        let mut diag = make_test_diagnostic(Severity::Error, "test");
        diag.location.file = PathBuf::from("/path/to/installer.wxs");

        let output = format_json(&[diag]);
        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();

        assert!(parsed["diagnostics"][0]["file"]
            .as_str()
            .unwrap()
            .contains("installer.wxs"));
    }

    #[test]
    fn test_format_sarif_artifact_location() {
        let mut diag = make_test_diagnostic(Severity::Error, "test");
        diag.location.file = PathBuf::from("src/installer.wxs");

        let output = format_sarif(&[diag]);
        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();

        let uri = parsed["runs"][0]["results"][0]["locations"][0]["physicalLocation"]
            ["artifactLocation"]["uri"]
            .as_str()
            .unwrap();
        assert!(uri.contains("installer.wxs"));
    }
}
