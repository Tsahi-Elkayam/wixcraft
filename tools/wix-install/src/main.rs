//! wix-install - WiX development environment preparation tool

use clap::{Parser, Subcommand};
use std::fs;
use std::path::PathBuf;
use wix_install::{
    default_prerequisites, scripts, CheckResult, EnvironmentCheck, OfflinePackage,
    PrereqStatus, SandboxConfig,
};

#[derive(Parser)]
#[command(name = "wix-install")]
#[command(about = "WiX development environment preparation tool")]
#[command(version)]
#[command(long_about = "Check, install, and configure prerequisites for WiX MSI development.\nSupports offline packages and Windows Sandbox testing.")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Check if all prerequisites are installed
    Check {
        /// Output format (text, json)
        #[arg(short, long, default_value = "text")]
        format: String,
    },

    /// Interactive setup - install missing components
    Setup {
        /// Install all optional components too
        #[arg(long)]
        all: bool,

        /// Non-interactive mode (use defaults)
        #[arg(short, long)]
        yes: bool,

        /// Generate setup scripts only (don't execute)
        #[arg(long)]
        scripts_only: bool,

        /// Output directory for scripts
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Create Windows Sandbox configuration for MSI testing
    Sandbox {
        /// Path to MSI file or folder to test
        path: PathBuf,

        /// Sandbox name
        #[arg(short, long, default_value = "WixTest")]
        name: String,

        /// Memory in MB
        #[arg(short, long, default_value = "4096")]
        memory: u32,

        /// Disable networking in sandbox
        #[arg(long)]
        no_network: bool,

        /// Output .wsb file path
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Launch sandbox immediately
        #[arg(long)]
        launch: bool,
    },

    /// Create offline installation package
    Offline {
        /// Output directory
        output: PathBuf,

        /// Package type (minimal, standard, full)
        #[arg(short, long, default_value = "standard")]
        package: String,

        /// Download files (requires internet)
        #[arg(long)]
        download: bool,

        /// Include setup scripts
        #[arg(long, default_value = "true")]
        scripts: bool,
    },

    /// Show information about prerequisites
    Info {
        /// Prerequisite ID (dotnet, wix, vsbuildtools, windowssdk, git)
        prereq: Option<String>,
    },

    /// Export setup scripts
    Scripts {
        /// Output directory
        output: PathBuf,

        /// Script type (check, install, offline, sandbox, all)
        #[arg(short, long, default_value = "all")]
        script_type: String,
    },
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Check { format } => {
            let prereqs = default_prerequisites();
            let results = check_prerequisites();
            let check = EnvironmentCheck::new(results.clone(), &prereqs);

            if format == "json" {
                println!("{}", serde_json::to_string_pretty(&check)?);
            } else {
                println!();
                println!("WiX Development Environment Check");
                println!("==================================");
                println!();

                for result in &results {
                    let status_icon = match result.status {
                        PrereqStatus::Installed => "\x1b[32m[OK]\x1b[0m",
                        PrereqStatus::NotInstalled => "\x1b[31m[MISSING]\x1b[0m",
                        PrereqStatus::Outdated => "\x1b[33m[OUTDATED]\x1b[0m",
                        PrereqStatus::Unknown => "\x1b[33m[?]\x1b[0m",
                    };
                    let version = result.version.as_deref().unwrap_or("");
                    println!("{} {} {}", status_icon, result.prerequisite, version);
                }

                println!();
                if check.ready {
                    println!("\x1b[32mEnvironment is ready for WiX development!\x1b[0m");
                } else {
                    println!("\x1b[31mMissing required components:\x1b[0m");
                    for missing in &check.missing_required {
                        println!("  - {}", missing);
                    }
                    println!();
                    println!("Run 'wix-install setup' to install missing components.");
                }
                println!();
            }
        }

        Commands::Setup {
            all,
            yes,
            scripts_only,
            output,
        } => {
            let output_dir = output.unwrap_or_else(|| PathBuf::from("."));

            if scripts_only {
                // Export scripts only
                fs::create_dir_all(&output_dir)?;

                let check_path = output_dir.join("check-env.ps1");
                fs::write(&check_path, scripts::CHECK_ENV_PS1)?;
                println!("Created: {}", check_path.display());

                let install_path = output_dir.join("install-wix.ps1");
                fs::write(&install_path, scripts::INSTALL_WIX_PS1)?;
                println!("Created: {}", install_path.display());

                println!();
                println!("Run these scripts on Windows:");
                println!("  powershell -ExecutionPolicy Bypass -File check-env.ps1");
                println!("  powershell -ExecutionPolicy Bypass -File install-wix.ps1");
            } else {
                println!();
                println!("WiX Development Environment Setup");
                println!("==================================");
                println!();

                let prereqs = default_prerequisites();

                for prereq in &prereqs {
                    if !prereq.required && !all {
                        continue;
                    }

                    println!("{}: {}", prereq.name, prereq.description);
                    if let Some(ref url) = prereq.download_url {
                        println!("  Download: {}", url);
                    }
                    if let Some(ref args) = prereq.install_args {
                        println!("  Install: {} {}", prereq.install_command.as_deref().unwrap_or(""), args.join(" "));
                    }
                    println!();
                }

                if !yes {
                    println!("This tool generates setup scripts for Windows.");
                    println!("Use --scripts-only to export scripts without prompts.");
                }
            }
        }

        Commands::Sandbox {
            path,
            name,
            memory,
            no_network,
            output,
            launch,
        } => {
            let config = SandboxConfig::new(&name)
                .with_memory(memory)
                .with_networking(!no_network)
                .add_folder(path.clone(), Some("C:\\TestMSI".to_string()), true);

            let wsb_content = config.to_wsb();

            let wsb_path = output.unwrap_or_else(|| {
                let mut p = std::env::temp_dir();
                p.push(format!("{}.wsb", name));
                p
            });

            fs::write(&wsb_path, &wsb_content)?;
            println!("Created sandbox configuration: {}", wsb_path.display());
            println!();
            println!("Configuration:");
            println!("  Memory: {} MB", memory);
            println!("  Networking: {}", if no_network { "Disabled" } else { "Enabled" });
            println!("  Mapped folder: {} -> C:\\TestMSI", path.display());

            if launch {
                #[cfg(target_os = "windows")]
                {
                    println!();
                    println!("Launching Windows Sandbox...");
                    std::process::Command::new("cmd")
                        .args(["/c", "start", "", wsb_path.to_str().unwrap()])
                        .spawn()?;
                }

                #[cfg(not(target_os = "windows"))]
                {
                    println!();
                    println!("Note: Windows Sandbox can only be launched on Windows.");
                    println!("Copy {} to a Windows machine to use.", wsb_path.display());
                }
            } else {
                println!();
                println!("To launch: double-click the .wsb file or run:");
                println!("  start {}", wsb_path.display());
            }
        }

        Commands::Offline {
            output,
            package,
            download,
            scripts,
        } => {
            fs::create_dir_all(&output)?;

            let pkg = match package.as_str() {
                "minimal" => OfflinePackage::minimal(output.clone()),
                "full" => OfflinePackage::full(output.clone()),
                _ => OfflinePackage::standard(output.clone()),
            };

            println!("Creating offline package: {}", pkg.name);
            println!("Output directory: {}", output.display());
            println!();

            println!("Components:");
            for comp in &pkg.components {
                let size = comp.size_mb.map(|s| format!(" (~{} MB)", s)).unwrap_or_default();
                println!("  - {}{}", comp.name, size);
                println!("    {}", comp.download_url);
            }
            println!();
            println!("Total estimated size: {} MB", pkg.total_size_mb());

            if scripts {
                let install_bat = output.join("install.bat");
                fs::write(&install_bat, scripts::OFFLINE_INSTALL_BAT)?;
                println!();
                println!("Created: {}", install_bat.display());
            }

            if download {
                println!();
                println!("Downloading components...");
                for comp in &pkg.components {
                    let dest = output.join(&comp.filename);
                    println!("  Downloading {} -> {}", comp.name, dest.display());
                    // In a real implementation, use reqwest or similar to download
                    println!("    URL: {}", comp.download_url);
                }
                println!();
                println!("Note: Actual download not implemented. Use a browser or curl to download.");
            } else {
                println!();
                println!("Use --download to fetch files (requires internet).");
                println!("Or manually download the files above.");
            }
        }

        Commands::Info { prereq } => {
            let prereqs = default_prerequisites();

            if let Some(id) = prereq {
                if let Some(p) = prereqs.iter().find(|p| p.id == id) {
                    println!();
                    println!("{}", p.name);
                    println!("{}", "=".repeat(p.name.len()));
                    println!();
                    println!("Description: {}", p.description);
                    println!("Required: {}", if p.required { "Yes" } else { "No" });
                    if let Some(ref url) = p.download_url {
                        println!("Download: {}", url);
                    }
                    if let Some(ref cmd) = p.check_command {
                        println!("Check command: {}", cmd);
                    }
                    if let Some(ref args) = p.install_args {
                        println!("Install: {} {}", p.install_command.as_deref().unwrap_or(""), args.join(" "));
                    }
                    println!();
                } else {
                    eprintln!("Unknown prerequisite: {}", id);
                    eprintln!("Available: dotnet, wix, vsbuildtools, windowssdk, git");
                }
            } else {
                println!();
                println!("Prerequisites for WiX Development");
                println!("==================================");
                println!();
                println!("Required:");
                for p in prereqs.iter().filter(|p| p.required) {
                    println!("  {} - {}", p.id, p.description);
                }
                println!();
                println!("Optional:");
                for p in prereqs.iter().filter(|p| !p.required) {
                    println!("  {} - {}", p.id, p.description);
                }
                println!();
                println!("Use 'wix-install info <prereq>' for details.");
            }
        }

        Commands::Scripts { output, script_type } => {
            fs::create_dir_all(&output)?;

            let scripts_to_export: Vec<(&str, &str, &str)> = match script_type.as_str() {
                "check" => vec![("check-env.ps1", scripts::CHECK_ENV_PS1, "PowerShell environment check")],
                "install" => vec![("install-wix.ps1", scripts::INSTALL_WIX_PS1, "PowerShell WiX installer")],
                "offline" => vec![("install.bat", scripts::OFFLINE_INSTALL_BAT, "Batch offline installer")],
                "sandbox" => vec![("create-sandbox.ps1", scripts::CREATE_SANDBOX_PS1, "PowerShell sandbox creator")],
                _ => vec![
                    ("check-env.ps1", scripts::CHECK_ENV_PS1, "PowerShell environment check"),
                    ("install-wix.ps1", scripts::INSTALL_WIX_PS1, "PowerShell WiX installer"),
                    ("install.bat", scripts::OFFLINE_INSTALL_BAT, "Batch offline installer"),
                    ("create-sandbox.ps1", scripts::CREATE_SANDBOX_PS1, "PowerShell sandbox creator"),
                ],
            };

            println!("Exporting scripts to: {}", output.display());
            println!();

            for (filename, content, description) in scripts_to_export {
                let path = output.join(filename);
                fs::write(&path, content)?;
                println!("  {} - {}", filename, description);
            }

            println!();
            println!("Run PowerShell scripts with:");
            println!("  powershell -ExecutionPolicy Bypass -File <script.ps1>");
        }
    }

    Ok(())
}

/// Check prerequisites (simulated - actual checks require Windows)
fn check_prerequisites() -> Vec<CheckResult> {
    let prereqs = default_prerequisites();
    let mut results = Vec::new();

    for prereq in prereqs {
        // On non-Windows, we can't actually check, so report as unknown
        #[cfg(not(target_os = "windows"))]
        {
            results.push(CheckResult::unknown(
                &prereq.id,
                "Cannot check on non-Windows platform",
            ));
        }

        #[cfg(target_os = "windows")]
        {
            use std::process::Command;

            if let Some(ref check_cmd) = prereq.check_command {
                let parts: Vec<&str> = check_cmd.split_whitespace().collect();
                if !parts.is_empty() {
                    match Command::new(parts[0]).args(&parts[1..]).output() {
                        Ok(output) => {
                            if output.status.success() {
                                let version = String::from_utf8_lossy(&output.stdout)
                                    .trim()
                                    .to_string();
                                results.push(CheckResult::installed(&prereq.id, &version, None));
                            } else {
                                results.push(CheckResult::not_installed(&prereq.id));
                            }
                        }
                        Err(_) => {
                            results.push(CheckResult::not_installed(&prereq.id));
                        }
                    }
                }
            } else {
                results.push(CheckResult::unknown(&prereq.id, "No check command defined"));
            }
        }
    }

    results
}
