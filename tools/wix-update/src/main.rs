//! wix-update CLI - Update script generator
//!
//! Usage:
//!   wix-update command update.msi            # Generate msiexec command
//!   wix-update script update.msi --batch     # Generate batch script
//!   wix-update script update.msi --powershell # Generate PowerShell script

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use wix_update::*;

#[derive(Parser)]
#[command(name = "wix-update")]
#[command(about = "Update script generator for MSI packages")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate msiexec command for update
    Command {
        /// MSI file path
        msi: String,

        /// Update type (major, minor, patch, reinstall)
        #[arg(short, long, default_value = "major")]
        update_type: String,

        /// MSP patch file (for patch updates)
        #[arg(long)]
        msp: Option<String>,

        /// Log file path
        #[arg(short, long)]
        log: Option<String>,

        /// Silent installation
        #[arg(short, long)]
        silent: bool,

        /// Suppress restart
        #[arg(long)]
        no_restart: bool,

        /// Additional properties (NAME=value)
        #[arg(short, long)]
        property: Vec<String>,
    },

    /// Generate update script
    Script {
        /// MSI file path
        msi: String,

        /// Update type (major, minor, patch, reinstall)
        #[arg(short, long, default_value = "major")]
        update_type: String,

        /// Generate batch script
        #[arg(long)]
        batch: bool,

        /// Generate PowerShell script
        #[arg(long)]
        powershell: bool,

        /// MSP patch file
        #[arg(long)]
        msp: Option<String>,

        /// Log file path
        #[arg(short, long, default_value = "update.log")]
        log: String,

        /// Silent mode
        #[arg(short, long)]
        silent: bool,

        /// Suppress restart
        #[arg(long)]
        no_restart: bool,

        /// Output file
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Show update type information
    Info,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Command {
            msi,
            update_type,
            msp,
            log,
            silent,
            no_restart,
            property,
        } => {
            let config = UpdateConfig {
                update_type: parse_update_type(&update_type)?,
                msi_path: msi,
                msp_path: msp,
                log_file: log,
                silent,
                no_restart,
                properties: parse_properties(&property),
            };

            let cmd = UpdateGenerator::generate_msiexec_command(&config);
            println!("{}", cmd);
        }

        Commands::Script {
            msi,
            update_type,
            batch,
            powershell,
            msp,
            log,
            silent,
            no_restart,
            output,
        } => {
            let config = UpdateConfig {
                update_type: parse_update_type(&update_type)?,
                msi_path: msi,
                msp_path: msp,
                log_file: Some(log),
                silent,
                no_restart,
                properties: Vec::new(),
            };

            let script = if powershell || (!batch && !powershell) {
                UpdateGenerator::generate_powershell(&config)
            } else {
                UpdateGenerator::generate_batch(&config)
            };

            if let Some(path) = output {
                std::fs::write(&path, &script)?;
                println!("Script written to: {}", path.display());
            } else {
                println!("{}", script);
            }
        }

        Commands::Info => {
            println!("MSI Update Types");
            println!("{}", "=".repeat(60));
            println!();
            println!("major - Major Upgrade");
            println!("  - New ProductCode");
            println!("  - Removes old version, installs new");
            println!("  - Command: msiexec /i newversion.msi");
            println!();
            println!("minor - Minor Upgrade");
            println!("  - Same ProductCode, different version");
            println!("  - In-place upgrade");
            println!("  - Command: msiexec /i update.msi REINSTALLMODE=vomus REINSTALL=ALL");
            println!();
            println!("patch - Patch Update (MSP)");
            println!("  - Applies delta changes");
            println!("  - Smallest update size");
            println!("  - Command: msiexec /p patch.msp");
            println!();
            println!("reinstall - Reinstall/Repair");
            println!("  - Repairs current installation");
            println!("  - Restores missing files");
            println!("  - Command: msiexec /f product.msi");
            println!();
            println!("Common Options:");
            println!("  /qn         - Silent (no UI)");
            println!("  /qb         - Basic UI (progress bar)");
            println!("  /l*v log    - Verbose logging");
            println!("  /norestart  - Suppress reboot");
        }
    }

    Ok(())
}

fn parse_update_type(s: &str) -> Result<UpdateType> {
    match s.to_lowercase().as_str() {
        "major" => Ok(UpdateType::Major),
        "minor" => Ok(UpdateType::Minor),
        "patch" | "msp" => Ok(UpdateType::Patch),
        "reinstall" | "repair" => Ok(UpdateType::Reinstall),
        _ => Err(anyhow::anyhow!(
            "Unknown update type: {}. Use: major, minor, patch, reinstall",
            s
        )),
    }
}

fn parse_properties(props: &[String]) -> Vec<(String, String)> {
    props
        .iter()
        .filter_map(|p| {
            let parts: Vec<&str> = p.splitn(2, '=').collect();
            if parts.len() == 2 {
                Some((parts[0].to_string(), parts[1].to_string()))
            } else {
                None
            }
        })
        .collect()
}
