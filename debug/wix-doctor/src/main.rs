//! wix-doctor CLI - Error decoder and log analyzer for WiX/MSI

use clap::{Parser, Subcommand};
use std::fs;
use std::path::PathBuf;
use wix_doctor::{ErrorDecoder, IssueSeverity, LogAnalyzer};

#[derive(Parser)]
#[command(name = "wix-doctor")]
#[command(about = "Error decoder and log analyzer for WiX/MSI")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Decode an MSI error code
    Decode {
        /// The error code to decode
        code: u32,
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Analyze an MSI verbose log file
    Analyze {
        /// Path to the log file
        path: PathBuf,
        /// Output as JSON
        #[arg(long)]
        json: bool,
        /// Show only errors and critical issues
        #[arg(long)]
        errors_only: bool,
    },
    /// Find the root cause of an installation failure
    RootCause {
        /// Path to the log file
        path: PathBuf,
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Extract installation timeline from log
    Timeline {
        /// Path to the log file
        path: PathBuf,
        /// Output as JSON
        #[arg(long)]
        json: bool,
        /// Show only failed actions
        #[arg(long)]
        failures_only: bool,
    },
    /// Search error codes by keyword
    Search {
        /// Search query
        query: String,
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// List all known error codes
    List {
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
}

fn main() {
    let cli = Cli::parse();
    let decoder = ErrorDecoder::new();
    let analyzer = LogAnalyzer::new();

    match cli.command {
        Commands::Decode { code, json } => {
            if let Some(info) = decoder.decode(code) {
                if json {
                    println!("{}", serde_json::to_string_pretty(info).unwrap());
                } else {
                    println!("Error {}: {}", info.code, info.description);
                    println!();
                    println!("Explanation:");
                    println!("  {}", info.explanation);
                    println!();
                    if !info.causes.is_empty() {
                        println!("Possible Causes:");
                        for cause in &info.causes {
                            println!("  - {}", cause);
                        }
                        println!();
                    }
                    if !info.fixes.is_empty() {
                        println!("Suggested Fixes:");
                        for fix in &info.fixes {
                            println!("  - {}", fix);
                        }
                        println!();
                    }
                    if !info.related.is_empty() {
                        println!(
                            "Related Error Codes: {}",
                            info.related
                                .iter()
                                .map(|c| c.to_string())
                                .collect::<Vec<_>>()
                                .join(", ")
                        );
                    }
                }
            } else {
                eprintln!("Unknown error code: {}", code);
                eprintln!("Use 'wix-doctor list' to see known error codes");
                std::process::exit(1);
            }
        }

        Commands::Analyze {
            path,
            json,
            errors_only,
        } => {
            let content = match fs::read_to_string(&path) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("Failed to read file {}: {}", path.display(), e);
                    std::process::exit(1);
                }
            };

            let mut issues = analyzer.analyze(&content);

            if errors_only {
                issues.retain(|i| {
                    matches!(i.severity, IssueSeverity::Error | IssueSeverity::Critical)
                });
            }

            let summary = analyzer.summarize(&issues);

            if json {
                let output = serde_json::json!({
                    "file": path.display().to_string(),
                    "summary": summary,
                    "issues": issues,
                });
                println!("{}", serde_json::to_string_pretty(&output).unwrap());
            } else {
                println!("Log Analysis: {}", path.display());
                println!("═══════════════════════════════════════════════════");
                println!();
                println!(
                    "Summary: {} issues found",
                    summary.total_issues()
                );
                println!(
                    "  Critical: {}  Errors: {}  Warnings: {}  Info: {}",
                    summary.critical_count,
                    summary.error_count,
                    summary.warning_count,
                    summary.info_count
                );
                println!();

                if !summary.error_codes.is_empty() {
                    println!("Error Codes Found:");
                    for (code, count) in &summary.error_codes {
                        let desc = decoder
                            .decode(*code)
                            .map(|i| i.description.as_str())
                            .unwrap_or("Unknown");
                        println!("  {} (x{}): {}", code, count, desc);
                    }
                    println!();
                }

                if issues.is_empty() {
                    println!("No issues found.");
                } else {
                    println!("Issues:");
                    println!("───────────────────────────────────────────────────");
                    for issue in &issues {
                        let severity = match issue.severity {
                            IssueSeverity::Critical => "CRITICAL",
                            IssueSeverity::Error => "ERROR",
                            IssueSeverity::Warning => "WARNING",
                            IssueSeverity::Info => "INFO",
                        };
                        println!(
                            "Line {}: [{}] {:?}",
                            issue.line, severity, issue.category
                        );
                        if let Some(ctx) = &issue.context {
                            println!("  {}", ctx);
                        }
                        println!("  > {}", truncate(&issue.text, 80));
                        if let Some(code) = issue.error_code {
                            if let Some(info) = decoder.decode(code) {
                                println!("  Tip: {}", info.fixes.first().unwrap_or(&String::new()));
                            }
                        }
                        println!();
                    }
                }
            }
        }

        Commands::RootCause { path, json } => {
            let content = match fs::read_to_string(&path) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("Failed to read file {}: {}", path.display(), e);
                    std::process::exit(1);
                }
            };

            match analyzer.find_root_cause(&content) {
                Some(analysis) => {
                    if json {
                        println!("{}", serde_json::to_string_pretty(&analysis).unwrap());
                    } else {
                        println!("Root Cause Analysis: {}", path.display());
                        println!("═══════════════════════════════════════════════════");
                        println!();
                        println!("Summary: {}", analysis.summary);
                        println!();
                        if let Some(ref action) = analysis.failing_action {
                            println!("Failing Action: {}", action);
                        }
                        println!("Root Cause: {}", analysis.root_cause);
                        println!();
                        println!("Suggestion: {}", analysis.suggestion);
                        println!();
                        if !analysis.context_lines.is_empty() {
                            println!("Context (lines leading to failure):");
                            println!("───────────────────────────────────────────────────");
                            for line in &analysis.context_lines {
                                println!("  {}", truncate(line, 78));
                            }
                        }
                    }
                }
                None => {
                    if json {
                        println!("{{\"status\": \"no_failure_detected\"}}");
                    } else {
                        println!("No installation failure detected in the log.");
                        println!("The installation may have completed successfully,");
                        println!("or the log does not contain standard failure markers.");
                    }
                }
            }
        }

        Commands::Timeline {
            path,
            json,
            failures_only,
        } => {
            let content = match fs::read_to_string(&path) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("Failed to read file {}: {}", path.display(), e);
                    std::process::exit(1);
                }
            };

            let mut timeline = analyzer.extract_timeline(&content);

            if failures_only {
                timeline.retain(|e| e.result == Some(3));
            }

            if json {
                println!("{}", serde_json::to_string_pretty(&timeline).unwrap());
            } else {
                println!("Installation Timeline: {}", path.display());
                println!("═══════════════════════════════════════════════════");
                println!();
                if timeline.is_empty() {
                    println!("No action timeline found in log.");
                } else {
                    println!(
                        "{:<12} {:<30} {:>8}",
                        "Timestamp", "Action", "Result"
                    );
                    println!("{}", "─".repeat(55));
                    for entry in &timeline {
                        let result = match entry.result {
                            Some(1) => "OK".to_string(),
                            Some(3) => "FAILED".to_string(),
                            Some(r) => format!("{}", r),
                            None => "...".to_string(),
                        };
                        let action = truncate(&entry.action, 28);
                        println!("{:<12} {:<30} {:>8}", entry.timestamp, action, result);
                    }
                    println!();

                    let failed = timeline.iter().filter(|e| e.result == Some(3)).count();
                    let success = timeline.iter().filter(|e| e.result == Some(1)).count();
                    println!(
                        "Total: {} actions ({} succeeded, {} failed)",
                        timeline.len(),
                        success,
                        failed
                    );
                }
            }
        }

        Commands::Search { query, json } => {
            let results = decoder.search(&query);

            if json {
                println!("{}", serde_json::to_string_pretty(&results).unwrap());
            } else if results.is_empty() {
                println!("No error codes match '{}'", query);
            } else {
                println!("Error codes matching '{}':", query);
                println!();
                for info in results {
                    println!("  {}: {}", info.code, info.description);
                }
            }
        }

        Commands::List { json } => {
            let codes = decoder.all_codes();

            if json {
                let list: Vec<_> = codes
                    .iter()
                    .filter_map(|c| decoder.decode(*c))
                    .collect();
                println!("{}", serde_json::to_string_pretty(&list).unwrap());
            } else {
                println!("Known MSI Error Codes:");
                println!();
                for code in codes {
                    if let Some(info) = decoder.decode(code) {
                        println!("  {}: {}", info.code, info.description);
                    }
                }
            }
        }
    }
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}...", &s[..max - 3])
    }
}
