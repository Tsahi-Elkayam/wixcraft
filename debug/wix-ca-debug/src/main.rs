//! wix-ca-debug CLI - Custom Action debugger and analyzer
//!
//! Usage:
//!   wix-ca-debug analyze Product.wxs      # Analyze custom actions
//!   wix-ca-debug debug CA_Install         # Generate debug code for an action
//!   wix-ca-debug guide CA_Install         # Show debugging guide
//!   wix-ca-debug list Product.wxs         # List all custom actions

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::fs;
use std::path::PathBuf;
use wix_ca_debug::{CustomActionAnalyzer, Severity};

#[derive(Parser)]
#[command(name = "wix-ca-debug")]
#[command(about = "Custom Action debugger and analyzer for WiX/MSI")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Analyze custom actions in WiX file(s)
    Analyze {
        /// WiX file(s) to analyze
        #[arg(required = true)]
        files: Vec<PathBuf>,

        /// Output format (text, json)
        #[arg(short, long, default_value = "text")]
        format: String,

        /// Show only issues (skip OK actions)
        #[arg(short, long)]
        issues_only: bool,
    },

    /// List all custom actions in WiX file(s)
    List {
        /// WiX file(s) to scan
        #[arg(required = true)]
        files: Vec<PathBuf>,

        /// Output format (text, json)
        #[arg(short, long, default_value = "text")]
        format: String,
    },

    /// Generate debug helper code for a custom action
    Debug {
        /// WiX file containing the custom action
        file: PathBuf,

        /// Custom action ID to generate debug code for
        action_id: String,

        /// Target language (csharp, cpp, vbscript)
        #[arg(short, long, default_value = "csharp")]
        language: String,
    },

    /// Show debugging guide for a custom action
    Guide {
        /// WiX file containing the custom action
        file: PathBuf,

        /// Custom action ID
        action_id: String,
    },

    /// Check for security vulnerabilities in custom actions
    Security {
        /// WiX file(s) to scan
        #[arg(required = true)]
        files: Vec<PathBuf>,

        /// Output format (text, json, sarif)
        #[arg(short, long, default_value = "text")]
        format: String,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let analyzer = CustomActionAnalyzer::new();

    match cli.command {
        Commands::Analyze {
            files,
            format,
            issues_only,
        } => {
            for file in files {
                let content = fs::read_to_string(&file)?;
                let result = analyzer.analyze(&content);

                if format == "json" {
                    println!("{}", serde_json::to_string_pretty(&result)?);
                } else {
                    println!("Custom Action Analysis: {}", file.display());
                    println!("{}", "=".repeat(60));
                    println!();

                    // Summary
                    println!("Summary:");
                    println!("  Total custom actions: {}", result.summary.total_custom_actions);
                    println!(
                        "  Immediate: {}  Deferred: {}  Elevated: {}",
                        result.summary.immediate_count,
                        result.summary.deferred_count,
                        result.summary.elevated_count
                    );
                    println!(
                        "  Issues: {} critical, {} error, {} warning, {} info",
                        result.summary.critical_issues,
                        result.summary.error_issues,
                        result.summary.warning_issues,
                        result.summary.info_issues
                    );
                    println!();

                    // Custom actions
                    for ca in &result.custom_actions {
                        if issues_only && ca.issues.is_empty() {
                            continue;
                        }

                        let status = if ca.issues.is_empty() {
                            "OK"
                        } else if ca.issues.iter().any(|i| matches!(i.severity, Severity::Critical | Severity::Error)) {
                            "ISSUE"
                        } else {
                            "WARN"
                        };

                        println!(
                            "[{}] {} ({:?}, {:?})",
                            status, ca.id, ca.action_type, ca.execution
                        );

                        if let Some(ref entry) = ca.dll_entry {
                            println!("     Entry: {}", entry);
                        }
                        if let Some(ref cmd) = ca.exe_command {
                            println!("     Command: {}", cmd);
                        }

                        if !ca.sequence_tables.is_empty() {
                            let tables: Vec<_> = ca.sequence_tables.iter().map(|s| s.table.as_str()).collect();
                            println!("     Scheduled in: {}", tables.join(", "));
                        }

                        for issue in &ca.issues {
                            let severity = match issue.severity {
                                Severity::Critical => "CRITICAL",
                                Severity::Error => "ERROR",
                                Severity::Warning => "WARNING",
                                Severity::Info => "INFO",
                            };
                            println!("     [{}/{}] {}", severity, issue.category, issue.message);
                            println!("       -> {}", issue.suggestion);
                        }
                        println!();
                    }
                }
            }
        }

        Commands::List { files, format } => {
            let mut all_actions = Vec::new();

            for file in &files {
                let content = fs::read_to_string(file)?;
                let result = analyzer.analyze(&content);
                for mut ca in result.custom_actions {
                    ca.issues.clear(); // Don't include issues in list
                    all_actions.push((file.display().to_string(), ca));
                }
            }

            if format == "json" {
                let output: Vec<_> = all_actions
                    .iter()
                    .map(|(f, ca)| {
                        serde_json::json!({
                            "file": f,
                            "action": ca
                        })
                    })
                    .collect();
                println!("{}", serde_json::to_string_pretty(&output)?);
            } else {
                println!("Custom Actions Found:");
                println!("{}", "=".repeat(60));
                println!();
                println!(
                    "{:<30} {:<12} {:<15} {}",
                    "ID", "Type", "Execution", "Entry/Command"
                );
                println!("{}", "-".repeat(80));

                for (file, ca) in &all_actions {
                    let type_str = format!("{:?}", ca.action_type);
                    let exec_str = format!("{:?}", ca.execution);
                    let entry = ca
                        .dll_entry
                        .as_ref()
                        .or(ca.exe_command.as_ref())
                        .or(ca.script.as_ref())
                        .map(|s| truncate(s, 30))
                        .unwrap_or_else(|| "-".to_string());

                    println!("{:<30} {:<12} {:<15} {}", ca.id, type_str, exec_str, entry);
                }

                println!();
                println!("Total: {} custom actions in {} file(s)", all_actions.len(), files.len());
            }
        }

        Commands::Debug {
            file,
            action_id,
            language,
        } => {
            let content = fs::read_to_string(&file)?;
            let result = analyzer.analyze(&content);

            let ca = result
                .custom_actions
                .iter()
                .find(|ca| ca.id == action_id)
                .ok_or_else(|| anyhow::anyhow!("Custom action '{}' not found", action_id))?;

            let code = analyzer.generate_debug_helper(ca, &language);
            println!("{}", code);
        }

        Commands::Guide { file, action_id } => {
            let content = fs::read_to_string(&file)?;
            let result = analyzer.analyze(&content);

            let ca = result
                .custom_actions
                .iter()
                .find(|ca| ca.id == action_id)
                .ok_or_else(|| anyhow::anyhow!("Custom action '{}' not found", action_id))?;

            let guide = analyzer.generate_debug_guide(ca);
            println!("{}", guide);
        }

        Commands::Security { files, format } => {
            let mut all_issues = Vec::new();

            for file in &files {
                let content = fs::read_to_string(file)?;
                let result = analyzer.analyze(&content);

                for issue in result.issues {
                    if issue.category == "Security" {
                        all_issues.push((file.display().to_string(), issue));
                    }
                }
            }

            if format == "json" {
                let output: Vec<_> = all_issues
                    .iter()
                    .map(|(f, i)| {
                        serde_json::json!({
                            "file": f,
                            "issue": i
                        })
                    })
                    .collect();
                println!("{}", serde_json::to_string_pretty(&output)?);
            } else if format == "sarif" {
                // SARIF output for CI/CD integration
                let results: Vec<_> = all_issues
                    .iter()
                    .map(|(f, i)| {
                        serde_json::json!({
                            "ruleId": format!("WIXSEC-{}", match i.severity {
                                Severity::Critical => "001",
                                Severity::Error => "002",
                                Severity::Warning => "003",
                                Severity::Info => "004",
                            }),
                            "level": match i.severity {
                                Severity::Critical | Severity::Error => "error",
                                Severity::Warning => "warning",
                                Severity::Info => "note",
                            },
                            "message": { "text": i.message },
                            "locations": [{
                                "physicalLocation": {
                                    "artifactLocation": { "uri": f },
                                    "region": {
                                        "startLine": i.line.unwrap_or(1)
                                    }
                                }
                            }]
                        })
                    })
                    .collect();

                let sarif = serde_json::json!({
                    "$schema": "https://raw.githubusercontent.com/oasis-tcs/sarif-spec/master/Schemata/sarif-schema-2.1.0.json",
                    "version": "2.1.0",
                    "runs": [{
                        "tool": {
                            "driver": {
                                "name": "wix-ca-debug",
                                "version": "0.1.0",
                                "informationUri": "https://github.com/wixcraft/wixcraft"
                            }
                        },
                        "results": results
                    }]
                });

                println!("{}", serde_json::to_string_pretty(&sarif)?);
            } else {
                println!("Security Analysis");
                println!("{}", "=".repeat(60));
                println!();

                if all_issues.is_empty() {
                    println!("No security issues found.");
                } else {
                    println!("Found {} security issue(s):", all_issues.len());
                    println!();

                    for (file, issue) in &all_issues {
                        let severity = match issue.severity {
                            Severity::Critical => "CRITICAL",
                            Severity::Error => "ERROR",
                            Severity::Warning => "WARNING",
                            Severity::Info => "INFO",
                        };

                        println!("[{}] {} - {}", severity, issue.custom_action_id, issue.message);
                        println!("  File: {}:{}", file, issue.line.unwrap_or(0));
                        println!("  Fix:  {}", issue.suggestion);
                        println!();
                    }
                }
            }
        }
    }

    Ok(())
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}...", &s[..max - 3])
    }
}
