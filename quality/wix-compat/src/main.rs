//! wix-compat CLI - Windows version compatibility checker
//!
//! Usage:
//!   wix-compat check product.wxs           # Check for compatibility issues
//!   wix-compat matrix product.wxs          # Show compatibility matrix
//!   wix-compat issues                      # List known Windows issues

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use wix_compat::*;

#[derive(Parser)]
#[command(name = "wix-compat")]
#[command(about = "Windows version compatibility checker for WiX installers")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Check WiX source for compatibility issues
    Check {
        /// WiX source file(s)
        #[arg(required = true)]
        files: Vec<PathBuf>,

        /// Output format (text, json)
        #[arg(short, long, default_value = "text")]
        format: String,

        /// Fail if critical issues found (for CI)
        #[arg(long)]
        strict: bool,
    },

    /// Show compatibility matrix
    Matrix {
        /// WiX source file(s)
        #[arg(required = true)]
        files: Vec<PathBuf>,
    },

    /// List known Windows compatibility issues
    Issues,

    /// Show Windows version information
    Versions,
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

            let analysis = CompatChecker::analyze(&all_content);

            match format.as_str() {
                "json" => {
                    println!("{}", serde_json::to_string_pretty(&analysis)?);
                }
                _ => {
                    println!("{}", CompatChecker::generate_report(&analysis));
                }
            }

            if strict && analysis.critical_count > 0 {
                std::process::exit(1);
            }
        }

        Commands::Matrix { files } => {
            let mut all_content = String::new();
            for file in &files {
                if !file.exists() {
                    continue;
                }
                all_content.push_str(&std::fs::read_to_string(file)?);
            }

            let analysis = CompatChecker::analyze(&all_content);
            println!("{}", CompatChecker::generate_matrix(&analysis));
        }

        Commands::Issues => {
            println!("Known Windows Compatibility Issues for MSI Installers");
            println!("{}", "=".repeat(60));
            println!();

            println!("1. WINDOWS SERVER 2025 - MSI Installation Hangs");
            println!("   Severity: Critical");
            println!("   Affected: Windows Server 2025 promoted to Domain Controller");
            println!("   Symptoms: MSI installations hang for ~30 minutes, Error 1603");
            println!("   Cause: Conflict with certain services (e.g., Splashtop Remote)");
            println!("   Workaround: Set conflicting services to Disabled or Delayed Start");
            println!("   Reference: https://techcommunity.microsoft.com/discussions/windowsserver");
            println!();

            println!("2. WINDOWS 11 24H2 - UAC Prompts for Per-User MSI (August 2025)");
            println!("   Severity: Warning");
            println!("   Affected: Windows 11 24H2 after KB5063878");
            println!("   Symptoms: Per-user MSI repairs trigger UAC, Error 1730");
            println!("   Cause: Security fix CVE-2025-50173 changed MSI behavior");
            println!("   Workaround: Use per-machine install or apply KB5070773");
            println!("   Reference: https://support.microsoft.com/kb/5063878");
            println!();

            println!("3. WINDOWS SERVER 2025 - VersionNT64 Detection");
            println!("   Severity: Warning");
            println!("   Affected: Packages using VersionNT64 condition checks");
            println!("   Symptoms: Version detection fails or returns unexpected values");
            println!("   Cause: Older packaging tools unaware of Server 2025 build numbers");
            println!("   Workaround: Update WiX Toolset, use build number checks");
            println!();

            println!("4. ARM64 WINDOWS - DifxApp Driver Installation");
            println!("   Severity: Critical");
            println!("   Affected: Packages using difx:Driver element");
            println!("   Symptoms: Driver installation fails on ARM64 devices");
            println!("   Cause: Microsoft never shipped DifxApp for ARM64");
            println!("   Workaround: Use pnputil.exe or DIFxAPI directly");
            println!("   Reference: https://github.com/wixtoolset/issues/issues/6251");
            println!();

            println!("5. ARM64 WINDOWS - ProcessorArchitecture Detection");
            println!("   Severity: Warning");
            println!("   Affected: x64 bundles running on ARM64");
            println!("   Symptoms: ProcessorArchitecture reports x64 instead of ARM64");
            println!("   Cause: Emulation layer reports emulated architecture");
            println!("   Workaround: Use IsWow64Process2 or environment variable checks");
            println!("   Reference: https://github.com/wixtoolset/issues/issues/6556");
            println!();

            println!("6. WIX V6 - Upgrade Migration Exceptions");
            println!("   Severity: Warning");
            println!("   Affected: Projects upgrading from WiX v5 to v6");
            println!("   Symptoms: WIX0001 System.InvalidOperationException during build");
            println!("   Cause: Duplicate sequence element detection bug");
            println!("   Workaround: Update to WiX v6.0.1 or later");
            println!("   Reference: https://github.com/wixtoolset/issues/issues/9028");
        }

        Commands::Versions => {
            println!("Windows Version Reference");
            println!("{}", "=".repeat(60));
            println!();
            println!("| Name                    | Version     | Build | VersionNT |");
            println!("|-------------------------|-------------|-------|-----------|");
            println!("| Windows 10 1809         | 10.0.17763  | 17763 | 1000      |");
            println!("| Windows 10 21H2         | 10.0.19044  | 19044 | 1000      |");
            println!("| Windows 10 22H2         | 10.0.19045  | 19045 | 1000      |");
            println!("| Windows 11 21H2         | 10.0.22000  | 22000 | 1100      |");
            println!("| Windows 11 22H2         | 10.0.22621  | 22621 | 1100      |");
            println!("| Windows 11 23H2         | 10.0.22631  | 22631 | 1100      |");
            println!("| Windows 11 24H2         | 10.0.26100  | 26100 | 1100      |");
            println!("| Windows Server 2019     | 10.0.17763  | 17763 | 1000      |");
            println!("| Windows Server 2022     | 10.0.20348  | 20348 | 1000      |");
            println!("| Windows Server 2025     | 10.0.26100  | 26100 | 1000      |");
            println!();
            println!("Note: VersionNT returns 1000 for Windows 10/Server, 1100 for Windows 11");
            println!("Use build numbers for precise version detection.");
        }
    }

    Ok(())
}
