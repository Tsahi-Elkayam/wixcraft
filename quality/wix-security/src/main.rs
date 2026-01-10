//! wix-security CLI - MSI security scanner
//!
//! Usage:
//!   wix-security scan file.wxs          # Scan a WiX source file
//!   wix-security scan *.wxs             # Scan multiple files
//!   wix-security scan --format sarif    # Output SARIF for CI/CD
//!   wix-security rules                  # List all security rules

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use wix_security::{to_sarif, SecurityScanner, Severity};

#[derive(Parser)]
#[command(name = "wix-security")]
#[command(about = "MSI security scanner for privilege escalation vulnerabilities")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Scan WiX source files for security vulnerabilities
    Scan {
        /// Files to scan
        #[arg(required = true)]
        files: Vec<PathBuf>,

        /// Output format (text, json, sarif)
        #[arg(short, long, default_value = "text")]
        format: String,

        /// Minimum severity to report (critical, high, medium, low, info)
        #[arg(short, long, default_value = "low")]
        min_severity: String,

        /// Exit with error if findings above threshold
        #[arg(long)]
        fail_on: Option<String>,
    },

    /// List all security rules
    Rules {
        /// Output format (text, json)
        #[arg(short, long, default_value = "text")]
        format: String,
    },

    /// Show information about a specific vulnerability
    Info {
        /// Rule ID (e.g., SEC001)
        id: String,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let scanner = SecurityScanner::new();

    match cli.command {
        Commands::Scan {
            files,
            format,
            min_severity,
            fail_on,
        } => {
            let min_sev = parse_severity(&min_severity)?;

            // Read and scan all files
            let mut all_content = String::new();
            let mut all_filenames = Vec::new();

            for file in &files {
                if !file.exists() {
                    eprintln!("Warning: File not found: {}", file.display());
                    continue;
                }

                let content = std::fs::read_to_string(file)?;
                let filename = file.to_string_lossy().to_string();

                let result = scanner.scan_source(&content, &filename);

                // Filter by minimum severity
                let filtered: Vec<_> = result
                    .findings
                    .into_iter()
                    .filter(|f| severity_level(&f.severity) <= severity_level(&min_sev))
                    .collect();

                all_content.push_str(&content);
                all_filenames.push(filename);

                // Output based on format
                match format.as_str() {
                    "json" => {
                        let output = serde_json::json!({
                            "file": file.display().to_string(),
                            "findings": filtered,
                            "summary": {
                                "critical": filtered.iter().filter(|f| f.severity == Severity::Critical).count(),
                                "high": filtered.iter().filter(|f| f.severity == Severity::High).count(),
                                "medium": filtered.iter().filter(|f| f.severity == Severity::Medium).count(),
                                "low": filtered.iter().filter(|f| f.severity == Severity::Low).count(),
                                "info": filtered.iter().filter(|f| f.severity == Severity::Info).count(),
                            }
                        });
                        println!("{}", serde_json::to_string_pretty(&output)?);
                    }
                    "sarif" => {
                        let mut sarif_result = wix_security::ScanResult::default();
                        for f in &filtered {
                            sarif_result.add_finding(f.clone());
                        }
                        let sarif = to_sarif(&sarif_result, "wix-security");
                        println!("{}", serde_json::to_string_pretty(&sarif)?);
                    }
                    _ => {
                        // Text format
                        if filtered.is_empty() {
                            println!("{}: No security issues found", file.display());
                        } else {
                            println!("Security Scan: {}", file.display());
                            println!("{}", "=".repeat(60));
                            println!();

                            for finding in &filtered {
                                print_finding(finding);
                            }

                            // Summary
                            let critical = filtered.iter().filter(|f| f.severity == Severity::Critical).count();
                            let high = filtered.iter().filter(|f| f.severity == Severity::High).count();
                            let medium = filtered.iter().filter(|f| f.severity == Severity::Medium).count();
                            let low = filtered.iter().filter(|f| f.severity == Severity::Low).count();

                            println!("Summary: {} findings", filtered.len());
                            println!(
                                "  Critical: {}  High: {}  Medium: {}  Low: {}",
                                critical, high, medium, low
                            );

                            if critical > 0 || high > 0 {
                                println!();
                                println!("Action required: Address critical and high severity findings before deployment.");
                            }
                        }
                    }
                }

                // Check fail threshold
                if let Some(ref threshold) = fail_on {
                    let threshold_sev = parse_severity(threshold)?;
                    let has_failure = filtered.iter().any(|f| severity_level(&f.severity) <= severity_level(&threshold_sev));
                    if has_failure {
                        std::process::exit(1);
                    }
                }
            }
        }

        Commands::Rules { format } => {
            let rules = get_rules_info();

            if format == "json" {
                println!("{}", serde_json::to_string_pretty(&rules)?);
            } else {
                println!("WiX Security Rules");
                println!("{}", "=".repeat(60));
                println!();

                for rule in &rules {
                    println!("{} - {}", rule.id, rule.title);
                    println!("  Severity: {}", rule.default_severity);
                    println!("  {}", rule.description);
                    println!();
                }
            }
        }

        Commands::Info { id } => {
            let rules = get_rules_info();
            let rule = rules.iter().find(|r| r.id == id.to_uppercase());

            match rule {
                Some(r) => {
                    println!("{}: {}", r.id, r.title);
                    println!("{}", "=".repeat(60));
                    println!();
                    println!("Severity: {}", r.default_severity);
                    println!();
                    println!("Description:");
                    println!("  {}", r.description);
                    println!();
                    println!("Detection:");
                    println!("  {}", r.detection);
                    println!();
                    println!("Remediation:");
                    println!("  {}", r.remediation);
                    if let Some(cve) = &r.related_cve {
                        println!();
                        println!("Related CVE: {}", cve);
                    }
                }
                None => {
                    eprintln!("Unknown rule: {}", id);
                    eprintln!("Use 'wix-security rules' to list all rules.");
                    std::process::exit(1);
                }
            }
        }
    }

    Ok(())
}

fn print_finding(finding: &wix_security::SecurityFinding) {
    let severity_color = match finding.severity {
        Severity::Critical => "\x1b[91m", // Red
        Severity::High => "\x1b[93m",     // Yellow
        Severity::Medium => "\x1b[33m",   // Orange
        Severity::Low => "\x1b[36m",      // Cyan
        Severity::Info => "\x1b[90m",     // Gray
    };
    let reset = "\x1b[0m";

    println!(
        "{}[{}]{} {} (Score: {:.1})",
        severity_color, finding.severity, reset, finding.id, finding.score
    );
    println!("  {}", finding.title);
    println!("  Location: {}", finding.location);
    println!("  Affected: {}", finding.affected);
    println!();
    println!("  {}", finding.description);
    println!();
    println!("  Remediation: {}", finding.remediation);
    if let Some(cve) = &finding.cve {
        println!("  Related CVE: {}", cve);
    }
    println!();
}

fn parse_severity(s: &str) -> Result<Severity> {
    match s.to_lowercase().as_str() {
        "critical" => Ok(Severity::Critical),
        "high" => Ok(Severity::High),
        "medium" => Ok(Severity::Medium),
        "low" => Ok(Severity::Low),
        "info" => Ok(Severity::Info),
        _ => Err(anyhow::anyhow!("Invalid severity: {}. Use: critical, high, medium, low, info", s)),
    }
}

fn severity_level(s: &Severity) -> u8 {
    match s {
        Severity::Critical => 0,
        Severity::High => 1,
        Severity::Medium => 2,
        Severity::Low => 3,
        Severity::Info => 4,
    }
}

#[derive(serde::Serialize)]
struct RuleInfo {
    id: &'static str,
    title: &'static str,
    default_severity: &'static str,
    description: &'static str,
    detection: &'static str,
    remediation: &'static str,
    related_cve: Option<&'static str>,
}

fn get_rules_info() -> Vec<RuleInfo> {
    vec![
        RuleInfo {
            id: "SEC001",
            title: "Elevated Custom Action Without Impersonation",
            default_severity: "High",
            description: "Custom action runs in deferred context without Impersonate=\"yes\", executing as SYSTEM.",
            detection: "CustomAction with Execute=\"deferred\" and no Impersonate=\"yes\"",
            remediation: "Add Impersonate=\"yes\" or review if SYSTEM privileges are required.",
            related_cve: Some("CVE-2024-38014"),
        },
        RuleInfo {
            id: "SEC002",
            title: "Explicitly Elevated Custom Action",
            default_severity: "Critical",
            description: "Custom action explicitly set to run as SYSTEM with Impersonate=\"no\".",
            detection: "CustomAction with Execute=\"deferred\" and Impersonate=\"no\"",
            remediation: "Remove Impersonate=\"no\" or change to \"yes\".",
            related_cve: Some("CVE-2024-38014"),
        },
        RuleInfo {
            id: "SEC003",
            title: "Dangerous Executable in Custom Action",
            default_severity: "High",
            description: "Custom action executes a command interpreter or dangerous executable.",
            detection: "ExeCommand containing cmd.exe, powershell, net.exe, etc.",
            remediation: "Use compiled DLLs instead of command interpreters.",
            related_cve: None,
        },
        RuleInfo {
            id: "SEC004",
            title: "Script-Based Custom Action",
            default_severity: "Medium",
            description: "Custom action uses VBScript or JScript which can be modified.",
            detection: "CustomAction with Script, VBScriptCall, or JScriptCall attribute",
            remediation: "Use compiled DLLs and sign the MSI package.",
            related_cve: None,
        },
        RuleInfo {
            id: "SEC005",
            title: "Temp Folder Used for Extraction",
            default_severity: "Medium",
            description: "Files extracted to temp folder can be replaced before execution.",
            detection: "[TempFolder], %TEMP%, or %TMP% in paths",
            remediation: "Extract to secure location with restricted permissions.",
            related_cve: Some("CVE-2023-26078"),
        },
        RuleInfo {
            id: "SEC006",
            title: "Executable in Writable Location",
            default_severity: "Medium",
            description: "Executable installed to user-writable location enables replacement attacks.",
            detection: "EXE/DLL in AppData, LocalAppData, ProgramData, or Public folders",
            remediation: "Install executables to Program Files with proper ACLs.",
            related_cve: None,
        },
        RuleInfo {
            id: "SEC007",
            title: "Command Line Execution Detected",
            default_severity: "High",
            description: "Shell commands in custom actions can be exploited for privilege escalation.",
            detection: "Patterns like cmd /c, powershell -Command, net user, etc.",
            remediation: "Use Windows API calls in compiled code instead of shell commands.",
            related_cve: None,
        },
        RuleInfo {
            id: "SEC008",
            title: "Service Running as LocalSystem",
            default_severity: "Medium",
            description: "Service runs with full SYSTEM privileges, increasing attack surface.",
            detection: "ServiceInstall without Account or with Account=\"LocalSystem\"",
            remediation: "Use LocalService, NetworkService, or dedicated service account.",
            related_cve: None,
        },
        RuleInfo {
            id: "SEC009",
            title: "Sensitive Registry Modification",
            default_severity: "Medium",
            description: "Modification of sensitive registry locations used for persistence.",
            detection: "RegistryKey/Value in Run, IFEO, Services, or Policies keys",
            remediation: "Review if registry modification is necessary and properly secured.",
            related_cve: None,
        },
        RuleInfo {
            id: "SEC010",
            title: "Binary Table Extraction",
            default_severity: "Low",
            description: "Binaries extracted from MSI at runtime should be verified.",
            detection: "Binary element with .exe or .dll source",
            remediation: "Ensure extraction location is secure and verify integrity.",
            related_cve: None,
        },
        RuleInfo {
            id: "SEC011",
            title: "Sensitive Property Modification",
            default_severity: "Info",
            description: "Properties affecting installation security context are modified.",
            detection: "ALLUSERS, MSIINSTALLPERUSER, or system folder properties",
            remediation: "Review property usage to ensure it cannot be exploited.",
            related_cve: None,
        },
        RuleInfo {
            id: "SEC012",
            title: "Unquoted Path with Spaces",
            default_severity: "Medium",
            description: "Unquoted paths can lead to arbitrary code execution.",
            detection: "Paths containing spaces without proper quoting",
            remediation: "Ensure all paths with spaces are properly quoted.",
            related_cve: None,
        },
    ]
}
