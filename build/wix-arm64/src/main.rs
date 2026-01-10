//! wix-arm64 CLI - ARM64 support helper for WiX
//!
//! Usage:
//!   wix-arm64 check product.wxs             # Check ARM64 compatibility
//!   wix-arm64 config --platforms x64,arm64  # Generate multi-platform config
//!   wix-arm64 build-script MyApp            # Generate build script

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use wix_arm64::*;

#[derive(Parser)]
#[command(name = "wix-arm64")]
#[command(about = "ARM64 support helper for WiX multi-platform builds")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Check WiX source for ARM64 compatibility
    Check {
        /// WiX source file(s)
        #[arg(required = true)]
        files: Vec<PathBuf>,

        /// Output format (text, json)
        #[arg(short, long, default_value = "text")]
        format: String,
    },

    /// Generate multi-platform configuration
    Config {
        /// Target platforms (comma-separated: x86,x64,arm64)
        #[arg(short, long, default_value = "x64,arm64")]
        platforms: String,

        /// Output file
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Generate multi-platform build script
    BuildScript {
        /// Project name
        project: String,

        /// Target platforms
        #[arg(short, long, default_value = "x64,arm64")]
        platforms: String,

        /// Output file
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Generate ARM64 detection condition
    Condition,

    /// Show ARM64 support information
    Info,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Check { files, format } => {
            let mut all_content = String::new();
            for file in &files {
                if !file.exists() {
                    eprintln!("Warning: File not found: {}", file.display());
                    continue;
                }
                all_content.push_str(&std::fs::read_to_string(file)?);
                all_content.push('\n');
            }

            let analysis = Arm64Analyzer::analyze(&all_content);

            match format.as_str() {
                "json" => {
                    println!("{}", serde_json::to_string_pretty(&analysis)?);
                }
                _ => {
                    println!("ARM64 Compatibility Analysis");
                    println!("{}", "=".repeat(50));
                    println!();

                    if let Some(ref platform) = analysis.current_platform {
                        println!("Current Platform: {}", platform);
                    } else {
                        println!("Current Platform: Not specified");
                    }

                    println!("Native Custom Actions: {}", analysis.has_native_custom_actions);
                    println!("Driver Installation: {}", analysis.has_driver_install);
                    println!("32-bit Components: {}", analysis.has_32bit_components);

                    if !analysis.issues.is_empty() {
                        println!();
                        println!("Issues:");
                        println!("{}", "-".repeat(50));

                        for (i, issue) in analysis.issues.iter().enumerate() {
                            let severity = match issue.severity {
                                IssueSeverity::Error => "ERROR",
                                IssueSeverity::Warning => "WARNING",
                                IssueSeverity::Info => "INFO",
                            };
                            println!();
                            println!("{}. [{}] {}", i + 1, severity, issue.message);
                            if let Some(ref elem) = issue.element {
                                println!("   Element: {}", elem);
                            }
                            println!("   Suggestion: {}", issue.suggestion);
                        }
                    } else {
                        println!();
                        println!("No ARM64 compatibility issues found.");
                    }
                }
            }
        }

        Commands::Config { platforms, output } => {
            let platform_list: Vec<Platform> = platforms
                .split(',')
                .filter_map(|s| Platform::from_str(s.trim()))
                .collect();

            if platform_list.is_empty() {
                eprintln!("Error: No valid platforms specified");
                eprintln!("Valid platforms: x86, x64, arm64");
                std::process::exit(1);
            }

            let config = Arm64Analyzer::generate_multiplatform_config(&platform_list);

            if let Some(out_path) = output {
                std::fs::write(&out_path, &config)?;
                println!("Configuration written to: {}", out_path.display());
            } else {
                println!("{}", config);
            }
        }

        Commands::BuildScript { project, platforms, output } => {
            let platform_list: Vec<Platform> = platforms
                .split(',')
                .filter_map(|s| Platform::from_str(s.trim()))
                .collect();

            if platform_list.is_empty() {
                eprintln!("Error: No valid platforms specified");
                std::process::exit(1);
            }

            let script = Arm64Analyzer::generate_build_script(&platform_list, &project);

            if let Some(out_path) = output {
                std::fs::write(&out_path, &script)?;
                println!("Build script written to: {}", out_path.display());
            } else {
                println!("{}", script);
            }
        }

        Commands::Condition => {
            println!("{}", Arm64Analyzer::generate_arm64_condition());
        }

        Commands::Info => {
            println!("ARM64 Support in WiX Toolset");
            println!("{}", "=".repeat(50));
            println!();

            println!("SUPPORTED IN WIX V4+:");
            println!("  - ARM64 platform target (Platform=\"arm64\")");
            println!("  - Native ARM64 custom actions");
            println!("  - ARM64 extension DLLs");
            println!("  - Burn bootstrapper ARM64 support");
            println!();

            println!("NOT SUPPORTED:");
            println!("  - DifxApp driver installation (use pnputil.exe instead)");
            println!("  - Some legacy extension custom actions");
            println!();

            println!("BUILDING FOR ARM64:");
            println!("  wix build -arch arm64 -o output.arm64.msi product.wxs");
            println!();

            println!("MULTI-PLATFORM BUILD:");
            println!("  # Build for both x64 and ARM64");
            println!("  wix build -arch x64 -o output.x64.msi product.wxs");
            println!("  wix build -arch arm64 -o output.arm64.msi product.wxs");
            println!();

            println!("PLATFORM VARIABLES:");
            println!("  $(sys.BUILDARCH)     - Current build architecture");
            println!("  $(sys.BUILDARCHSHORT)- Short form (e.g., 'A64' for ARM64)");
            println!();

            println!("DETECTING ARM64 AT RUNTIME:");
            println!("  - ProcessorArchitecture property: 12 = ARM64");
            println!("  - Environment variable: PROCESSOR_ARCHITECTURE=ARM64");
            println!("  - Note: x64 emulation reports x64, not ARM64");
            println!();

            println!("REFERENCES:");
            println!("  - https://github.com/wixtoolset/issues/issues/5558");
            println!("  - https://github.com/wixtoolset/issues/issues/6251");
            println!("  - https://linaro.atlassian.net/wiki/spaces/WOAR/");
        }
    }

    Ok(())
}
