//! wix-upgrade CLI - Upgrade path validator
//!
//! Usage:
//!   wix-upgrade validate file.wxs          # Validate single project
//!   wix-upgrade compare old.wxs new.wxs    # Compare versions for upgrade compatibility
//!   wix-upgrade check --old-dir v1/ --new-dir v2/  # Compare entire projects

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use wix_upgrade::{IssueSeverity, UpgradeValidator};

#[derive(Parser)]
#[command(name = "wix-upgrade")]
#[command(about = "Upgrade path validator for WiX/MSI installations")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Validate a single WiX project for upgrade readiness
    Validate {
        /// WiX source files to validate
        #[arg(required = true)]
        files: Vec<PathBuf>,

        /// Output format (text, json)
        #[arg(short, long, default_value = "text")]
        format: String,

        /// Exit with error if issues above threshold (error, warning, info)
        #[arg(long)]
        fail_on: Option<String>,
    },

    /// Compare two versions for upgrade compatibility
    Compare {
        /// Old version WiX file(s)
        #[arg(long, required = true)]
        old: Vec<PathBuf>,

        /// New version WiX file(s)
        #[arg(long, required = true)]
        new: Vec<PathBuf>,

        /// Output format (text, json)
        #[arg(short, long, default_value = "text")]
        format: String,

        /// Exit with error if issues above threshold
        #[arg(long)]
        fail_on: Option<String>,
    },

    /// List all upgrade validation rules
    Rules {
        /// Output format (text, json)
        #[arg(short, long, default_value = "text")]
        format: String,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let validator = UpgradeValidator::new();

    match cli.command {
        Commands::Validate { files, format, fail_on } => {
            let mut all_content = String::new();

            for file in &files {
                if !file.exists() {
                    eprintln!("Warning: File not found: {}", file.display());
                    continue;
                }
                let content = std::fs::read_to_string(file)?;
                all_content.push_str(&content);
                all_content.push('\n');
            }

            let filename = files.first()
                .map(|f| f.to_string_lossy().to_string())
                .unwrap_or_else(|| "unknown".to_string());

            let info = validator.extract_info(&all_content, &filename);
            let result = validator.validate_single(&info);

            match format.as_str() {
                "json" => {
                    println!("{}", serde_json::to_string_pretty(&result)?);
                }
                _ => {
                    print_validation_result(&result, &filename);
                }
            }

            // Check fail threshold
            if let Some(threshold) = fail_on {
                let threshold_level = parse_severity(&threshold)?;
                let has_failure = result.issues.iter().any(|i|
                    severity_level(&i.severity) <= severity_level(&threshold_level)
                );
                if has_failure {
                    std::process::exit(1);
                }
            }
        }

        Commands::Compare { old, new, format, fail_on } => {
            // Read old version files
            let mut old_content = String::new();
            for file in &old {
                if !file.exists() {
                    eprintln!("Warning: File not found: {}", file.display());
                    continue;
                }
                old_content.push_str(&std::fs::read_to_string(file)?);
                old_content.push('\n');
            }

            // Read new version files
            let mut new_content = String::new();
            for file in &new {
                if !file.exists() {
                    eprintln!("Warning: File not found: {}", file.display());
                    continue;
                }
                new_content.push_str(&std::fs::read_to_string(file)?);
                new_content.push('\n');
            }

            let old_filename = old.first()
                .map(|f| f.to_string_lossy().to_string())
                .unwrap_or_else(|| "old".to_string());
            let new_filename = new.first()
                .map(|f| f.to_string_lossy().to_string())
                .unwrap_or_else(|| "new".to_string());

            let old_info = validator.extract_info(&old_content, &old_filename);
            let new_info = validator.extract_info(&new_content, &new_filename);
            let result = validator.validate_upgrade(&old_info, &new_info);

            match format.as_str() {
                "json" => {
                    let output = serde_json::json!({
                        "old_version": old_info.product.version,
                        "new_version": new_info.product.version,
                        "upgrade_type": format!("{:?}", result.upgrade_type),
                        "issues": result.issues,
                        "summary": {
                            "errors": result.issues.iter().filter(|i| i.severity == IssueSeverity::Error).count(),
                            "warnings": result.issues.iter().filter(|i| i.severity == IssueSeverity::Warning).count(),
                            "info": result.issues.iter().filter(|i| i.severity == IssueSeverity::Info).count(),
                        }
                    });
                    println!("{}", serde_json::to_string_pretty(&output)?);
                }
                _ => {
                    print_comparison_result(&result, &old_info, &new_info);
                }
            }

            // Check fail threshold
            if let Some(threshold) = fail_on {
                let threshold_level = parse_severity(&threshold)?;
                let has_failure = result.issues.iter().any(|i|
                    severity_level(&i.severity) <= severity_level(&threshold_level)
                );
                if has_failure {
                    std::process::exit(1);
                }
            }
        }

        Commands::Rules { format } => {
            let rules = get_rules_info();

            if format == "json" {
                println!("{}", serde_json::to_string_pretty(&rules)?);
            } else {
                println!("WiX Upgrade Validation Rules");
                println!("{}", "=".repeat(60));
                println!();

                for rule in &rules {
                    println!("{} - {}", rule.id, rule.title);
                    println!("  Severity: {}", rule.severity);
                    println!("  {}", rule.description);
                    println!();
                }
            }
        }
    }

    Ok(())
}

fn print_validation_result(result: &wix_upgrade::UpgradeValidation, filename: &str) {
    println!("Upgrade Readiness: {}", filename);
    println!("{}", "=".repeat(60));
    println!();

    if result.issues.is_empty() {
        println!("No upgrade issues found. Project is ready for upgrades.");
        return;
    }

    for issue in &result.issues {
        let severity_color = match issue.severity {
            IssueSeverity::Error => "\x1b[91m",   // Red
            IssueSeverity::Warning => "\x1b[93m", // Yellow
            IssueSeverity::Info => "\x1b[36m",    // Cyan
        };
        let reset = "\x1b[0m";

        println!(
            "{}[{}]{} {} - {}",
            severity_color, issue.severity, reset, issue.id, issue.title
        );
        if !issue.affected.is_empty() {
            println!("  Affected: {}", issue.affected);
        }
        println!("  {}", issue.description);
        println!("  Suggestion: {}", issue.suggestion);
        println!();
    }

    // Summary
    let errors = result.issues.iter().filter(|i| i.severity == IssueSeverity::Error).count();
    let warnings = result.issues.iter().filter(|i| i.severity == IssueSeverity::Warning).count();
    let info = result.issues.iter().filter(|i| i.severity == IssueSeverity::Info).count();

    println!("Summary: {} issues", result.issues.len());
    println!("  Errors: {}  Warnings: {}  Info: {}", errors, warnings, info);

    if errors > 0 {
        println!();
        println!("Action required: Fix errors before releasing this version.");
    }
}

fn print_comparison_result(
    result: &wix_upgrade::UpgradeValidation,
    old_info: &wix_upgrade::ProjectInfo,
    new_info: &wix_upgrade::ProjectInfo,
) {
    println!("Upgrade Compatibility Analysis");
    println!("{}", "=".repeat(60));
    println!();
    println!("Old Version: {} ({})",
        old_info.product.version.as_deref().unwrap_or("unknown"),
        old_info.source_file);
    println!("New Version: {} ({})",
        new_info.product.version.as_deref().unwrap_or("unknown"),
        new_info.source_file);
    println!("Upgrade Type: {:?}", result.upgrade_type);
    println!();

    if result.issues.is_empty() {
        println!("No compatibility issues found. Upgrade path is valid.");
        return;
    }

    println!("Issues Found:");
    println!("{}", "-".repeat(60));
    println!();

    for issue in &result.issues {
        let severity_color = match issue.severity {
            IssueSeverity::Error => "\x1b[91m",
            IssueSeverity::Warning => "\x1b[93m",
            IssueSeverity::Info => "\x1b[36m",
        };
        let reset = "\x1b[0m";

        println!(
            "{}[{}]{} {} - {}",
            severity_color, issue.severity, reset, issue.id, issue.title
        );
        if !issue.affected.is_empty() {
            println!("  Affected: {}", issue.affected);
        }
        println!("  {}", issue.description);
        println!("  Suggestion: {}", issue.suggestion);
        println!();
    }

    // Summary
    let errors = result.issues.iter().filter(|i| i.severity == IssueSeverity::Error).count();
    let warnings = result.issues.iter().filter(|i| i.severity == IssueSeverity::Warning).count();

    println!("Summary: {} issues ({} errors, {} warnings)",
        result.issues.len(), errors, warnings);

    if errors > 0 {
        println!();
        println!("This upgrade may fail or cause issues. Address errors before release.");
    }
}

fn parse_severity(s: &str) -> Result<IssueSeverity> {
    match s.to_lowercase().as_str() {
        "error" => Ok(IssueSeverity::Error),
        "warning" => Ok(IssueSeverity::Warning),
        "info" => Ok(IssueSeverity::Info),
        _ => Err(anyhow::anyhow!("Invalid severity: {}. Use: error, warning, info", s)),
    }
}

fn severity_level(s: &IssueSeverity) -> u8 {
    match s {
        IssueSeverity::Error => 0,
        IssueSeverity::Warning => 1,
        IssueSeverity::Info => 2,
    }
}

#[derive(serde::Serialize)]
struct RuleInfo {
    id: &'static str,
    title: &'static str,
    severity: &'static str,
    description: &'static str,
}

fn get_rules_info() -> Vec<RuleInfo> {
    vec![
        RuleInfo {
            id: "UPG001",
            title: "Missing UpgradeCode",
            severity: "Error",
            description: "Package lacks UpgradeCode GUID required for all upgrade types.",
        },
        RuleInfo {
            id: "UPG002",
            title: "Component Without GUID",
            severity: "Error",
            description: "Component lacks explicit GUID, preventing stable upgrade tracking.",
        },
        RuleInfo {
            id: "UPG003",
            title: "Component Missing KeyPath",
            severity: "Warning",
            description: "Component should have explicit KeyPath for reliable detection.",
        },
        RuleInfo {
            id: "UPG004",
            title: "Duplicate Component GUID",
            severity: "Error",
            description: "Multiple components share the same GUID.",
        },
        RuleInfo {
            id: "UPG005",
            title: "Four-Part Version Number",
            severity: "Warning",
            description: "Windows Installer ignores the 4th version part for comparison.",
        },
        RuleInfo {
            id: "UPG006",
            title: "Version Number Out of Range",
            severity: "Error",
            description: "Version parts exceed Windows Installer limits (255.255.65535).",
        },
        RuleInfo {
            id: "UPG007",
            title: "Version Not Increased",
            severity: "Error",
            description: "New version must be greater than old version for upgrades.",
        },
        RuleInfo {
            id: "UPG008",
            title: "Component GUID Changed",
            severity: "Error",
            description: "Changing component GUID breaks upgrade path.",
        },
        RuleInfo {
            id: "UPG009",
            title: "Component KeyPath Changed",
            severity: "Warning",
            description: "Changing KeyPath may affect component detection during upgrade.",
        },
        RuleInfo {
            id: "UPG010",
            title: "UpgradeCode Changed",
            severity: "Error",
            description: "UpgradeCode must remain constant across all versions.",
        },
        RuleInfo {
            id: "UPG011",
            title: "Component Removed",
            severity: "Warning",
            description: "Removing components requires careful orphan handling.",
        },
        RuleInfo {
            id: "UPG012",
            title: "Feature Removed in Minor Upgrade",
            severity: "Error",
            description: "Cannot remove features in minor upgrades.",
        },
        RuleInfo {
            id: "UPG013",
            title: "New Feature in Update",
            severity: "Info",
            description: "Adding features converts small update to minor upgrade.",
        },
    ]
}
