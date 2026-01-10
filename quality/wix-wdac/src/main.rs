//! wix-wdac CLI - WDAC compatibility checker
//!
//! Usage:
//!   wix-wdac check product.wxs           # Check for WDAC issues
//!   wix-wdac check product.wxs --json    # JSON output
//!   wix-wdac rules product.wxs           # Generate WDAC allow rules

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use wix_wdac::*;

#[derive(Parser)]
#[command(name = "wix-wdac")]
#[command(about = "WDAC (Windows Defender Application Control) compatibility checker")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Check WiX source for WDAC compatibility issues
    Check {
        /// WiX source file(s)
        #[arg(required = true)]
        files: Vec<PathBuf>,

        /// Output format (text, json)
        #[arg(short, long, default_value = "text")]
        format: String,

        /// Fail if any blockers found (for CI)
        #[arg(long)]
        strict: bool,
    },

    /// Generate WDAC allow rules for identified issues
    Rules {
        /// WiX source file(s)
        #[arg(required = true)]
        files: Vec<PathBuf>,

        /// Output file (default: stdout)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// List custom actions and their WDAC compatibility
    Actions {
        /// WiX source file(s)
        #[arg(required = true)]
        files: Vec<PathBuf>,
    },

    /// Show WDAC best practices for WiX installers
    BestPractices,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Check { files, format, strict } => {
            let mut all_content = String::new();
            for file in &files {
                if !file.exists() {
                    eprintln!("Warning: File not found: {}", file.display());
                    continue;
                }
                all_content.push_str(&std::fs::read_to_string(file)?);
                all_content.push('\n');
            }

            let analysis = WdacAnalyzer::analyze(&all_content);

            match format.as_str() {
                "json" => {
                    println!("{}", serde_json::to_string_pretty(&analysis)?);
                }
                _ => {
                    println!("{}", WdacAnalyzer::generate_report(&analysis));
                }
            }

            if strict && !analysis.is_wdac_compatible {
                std::process::exit(1);
            }
        }

        Commands::Rules { files, output } => {
            let mut all_content = String::new();
            for file in &files {
                if !file.exists() {
                    continue;
                }
                all_content.push_str(&std::fs::read_to_string(file)?);
            }

            let analysis = WdacAnalyzer::analyze(&all_content);
            let rules = WdacAnalyzer::generate_allow_rules(&analysis);

            if let Some(out_path) = output {
                std::fs::write(&out_path, &rules)?;
                println!("Rules written to: {}", out_path.display());
            } else {
                println!("{}", rules);
            }
        }

        Commands::Actions { files } => {
            let mut all_content = String::new();
            for file in &files {
                if !file.exists() {
                    continue;
                }
                all_content.push_str(&std::fs::read_to_string(file)?);
            }

            let analysis = WdacAnalyzer::analyze(&all_content);

            println!("Custom Actions WDAC Analysis");
            println!("{}", "=".repeat(50));
            println!();

            if analysis.custom_actions.is_empty() {
                println!("No custom actions found.");
            } else {
                for ca in &analysis.custom_actions {
                    let status = match ca.action_type {
                        CustomActionType::VBScript | CustomActionType::JScript => "BLOCKED",
                        CustomActionType::PowerShell => "RISKY",
                        CustomActionType::NativeDll | CustomActionType::ManagedDll => {
                            if ca.is_signed == Some(true) { "OK" } else { "CHECK" }
                        }
                        CustomActionType::PropertySet | CustomActionType::DirectorySet => "OK",
                        _ => "UNKNOWN",
                    };

                    println!("[{}] {} - {:?}", status, ca.id, ca.action_type);
                    if let Some(ref source) = ca.source {
                        println!("    Source: {}", source);
                    }
                    if let Some(ref target) = ca.target {
                        println!("    Target: {}", target);
                    }
                    println!();
                }
            }
        }

        Commands::BestPractices => {
            println!("WDAC Best Practices for WiX Installers");
            println!("{}", "=".repeat(50));
            println!();

            println!("1. AVOID SCRIPT-BASED CUSTOM ACTIONS");
            println!("   - VBScript and JScript are blocked by default in WDAC");
            println!("   - Convert to signed native or managed DLLs");
            println!();

            println!("2. SIGN ALL CUSTOM ACTION DLLS");
            println!("   - Use a code signing certificate from a trusted CA");
            println!("   - Consider Azure Trusted Signing for cloud-based signing");
            println!("   - Sign during build, not just at release");
            println!();

            println!("3. HANDLE UNSIGNED WIX EXTENSION DLLS");
            println!("   - wixca.dll and other WiX DLLs are not signed");
            println!("   - Add file hash rules to your WDAC policy");
            println!("   - Request signed versions from WiX team (Issue #5329)");
            println!();

            println!("4. POWERSHELL CONSIDERATIONS");
            println!("   - PowerShell may work in Constrained Language Mode");
            println!("   - Prefer compiled code for reliability");
            println!("   - Test with WDAC audit mode first");
            println!();

            println!("5. TEST WITH AUDIT MODE");
            println!("   - Enable WDAC in audit mode before enforcement");
            println!("   - Check Windows Event Log for blocked events");
            println!("   - Event ID 3076 (audit) and 3077 (block)");
            println!();

            println!("6. USE THE WDAC POLICY WIZARD");
            println!("   - Microsoft's WDAC Policy Wizard simplifies rule creation");
            println!("   - Create file hash or publisher rules for exceptions");
            println!();

            println!("7. DOCUMENT YOUR WDAC REQUIREMENTS");
            println!("   - List all required exceptions for your installer");
            println!("   - Provide WDAC policy snippets to IT administrators");
            println!("   - Use 'wix-wdac rules' to generate allow rules");
            println!();

            println!("Resources:");
            println!("  - https://learn.microsoft.com/windows/security/application-security/application-control/");
            println!("  - https://github.com/wixtoolset/issues/issues/5329");
            println!("  - https://github.com/MicrosoftDocs/WDAC-Toolkit");
        }
    }

    Ok(())
}
