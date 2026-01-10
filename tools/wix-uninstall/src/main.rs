//! wix-uninstall CLI - Uninstall script generator
//!
//! Usage:
//!   wix-uninstall command --code {GUID}      # Generate uninstall command
//!   wix-uninstall script --name "MyApp"      # Generate script to find and uninstall
//!   wix-uninstall list                       # Generate product listing script

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use wix_uninstall::*;

#[derive(Parser)]
#[command(name = "wix-uninstall")]
#[command(about = "Uninstall script generator for MSI packages")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate msiexec command for uninstall
    Command {
        /// Product Code GUID
        #[arg(long)]
        code: Option<String>,

        /// MSI file path
        #[arg(long)]
        msi: Option<String>,

        /// Product name (for lookup)
        #[arg(long)]
        name: Option<String>,

        /// Log file path
        #[arg(short, long)]
        log: Option<String>,

        /// Silent uninstall
        #[arg(short, long)]
        silent: bool,
    },

    /// Generate uninstall script
    Script {
        /// Product Code GUID
        #[arg(long)]
        code: Option<String>,

        /// MSI file path
        #[arg(long)]
        msi: Option<String>,

        /// Product name (searches registry)
        #[arg(long)]
        name: Option<String>,

        /// Generate batch script
        #[arg(long)]
        batch: bool,

        /// Generate PowerShell script
        #[arg(long)]
        powershell: bool,

        /// Log file path
        #[arg(short, long, default_value = "uninstall.log")]
        log: String,

        /// Silent mode
        #[arg(short, long)]
        silent: bool,

        /// Include cleanup steps
        #[arg(long)]
        cleanup: bool,

        /// Output file
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Generate script to list installed products
    List {
        /// Filter pattern
        #[arg(short, long, default_value = "*")]
        filter: String,

        /// Output file
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Show uninstall methods and error codes
    Info,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Command {
            code,
            msi,
            name,
            log,
            silent,
        } => {
            let (method, identifier) = if let Some(c) = code {
                (UninstallMethod::ProductCode, c)
            } else if let Some(m) = msi {
                (UninstallMethod::MsiFile, m)
            } else if let Some(n) = name {
                (UninstallMethod::ProductName, n)
            } else {
                return Err(anyhow::anyhow!(
                    "Specify --code, --msi, or --name"
                ));
            };

            let config = UninstallConfig {
                method,
                identifier,
                log_file: log,
                silent,
                ..Default::default()
            };

            let cmd = UninstallGenerator::generate_msiexec_command(&config);
            println!("{}", cmd);
        }

        Commands::Script {
            code,
            msi,
            name,
            batch,
            powershell,
            log,
            silent,
            cleanup,
            output,
        } => {
            let (method, identifier) = if let Some(c) = code {
                (UninstallMethod::ProductCode, c)
            } else if let Some(m) = msi {
                (UninstallMethod::MsiFile, m)
            } else if let Some(n) = name {
                (UninstallMethod::ProductName, n)
            } else {
                return Err(anyhow::anyhow!(
                    "Specify --code, --msi, or --name"
                ));
            };

            let config = UninstallConfig {
                method,
                identifier,
                log_file: Some(log),
                silent,
                cleanup,
                ..Default::default()
            };

            let script = if batch {
                UninstallGenerator::generate_batch(&config)
            } else if powershell || !batch {
                UninstallGenerator::generate_powershell(&config)
            } else {
                UninstallGenerator::generate_batch(&config)
            };

            if let Some(path) = output {
                std::fs::write(&path, &script)?;
                println!("Script written to: {}", path.display());
            } else {
                println!("{}", script);
            }
        }

        Commands::List { filter, output } => {
            let mut script = UninstallGenerator::generate_list_products_script();
            if filter != "*" {
                script = script.replace("$Filter = \"*\"", &format!("$Filter = \"*{}*\"", filter));
            }

            if let Some(path) = output {
                std::fs::write(&path, &script)?;
                println!("Script written to: {}", path.display());
            } else {
                println!("{}", script);
            }
        }

        Commands::Info => {
            println!("MSI Uninstall Methods");
            println!("{}", "=".repeat(60));
            println!();
            println!("By Product Code:");
            println!("  msiexec /x {{GUID}} /qn");
            println!("  Most reliable method - GUID never changes");
            println!();
            println!("By MSI File:");
            println!("  msiexec /x product.msi /qn");
            println!("  Requires original MSI file");
            println!();
            println!("By Product Name (PowerShell):");
            println!("  Get-CimInstance Win32_Product | Where Name -like '*App*'");
            println!("  Then use IdentifyingNumber property");
            println!();
            println!("Common Exit Codes:");
            println!("  0     - Success");
            println!("  1605  - Product not installed");
            println!("  1618  - Another installation in progress");
            println!("  1619  - Installation package could not be opened");
            println!("  3010  - Success, restart required");
            println!();
            println!("Options:");
            println!("  /qn         - Silent (no UI)");
            println!("  /qb         - Basic UI (progress bar)");
            println!("  /l*v log    - Verbose logging");
            println!("  /norestart  - Suppress automatic restart");
        }
    }

    Ok(())
}
