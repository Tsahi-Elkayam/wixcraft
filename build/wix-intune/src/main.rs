//! wix-intune CLI - Microsoft Intune deployment package generator
//!
//! Usage:
//!   wix-intune generate product.wxs -o output/     # Generate all deployment files
//!   wix-intune manifest product.wxs                # Generate Intune manifest JSON
//!   wix-intune scripts product.wxs                 # Generate install/uninstall scripts

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use wix_intune::*;

#[derive(Parser)]
#[command(name = "wix-intune")]
#[command(about = "Generate Microsoft Intune deployment packages from WiX installers")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate all Intune deployment files
    Generate {
        /// WiX source file
        #[arg(required = true)]
        file: PathBuf,

        /// MSI filename (default: derived from WiX source)
        #[arg(short, long)]
        msi: Option<String>,

        /// Output directory
        #[arg(short, long, default_value = ".")]
        output: PathBuf,
    },

    /// Generate Intune app manifest JSON
    Manifest {
        /// WiX source file
        #[arg(required = true)]
        file: PathBuf,

        /// MSI filename
        #[arg(short, long)]
        msi: Option<String>,
    },

    /// Generate install/uninstall PowerShell scripts
    Scripts {
        /// WiX source file
        #[arg(required = true)]
        file: PathBuf,

        /// MSI filename
        #[arg(short, long)]
        msi: Option<String>,

        /// Output directory
        #[arg(short, long, default_value = ".")]
        output: PathBuf,
    },

    /// Show content prep tool instructions
    Instructions {
        /// WiX source file
        #[arg(required = true)]
        file: PathBuf,

        /// MSI filename
        #[arg(short, long)]
        msi: Option<String>,
    },

    /// Generate detection script
    Detection {
        /// WiX source file
        #[arg(required = true)]
        file: PathBuf,

        /// MSI filename
        #[arg(short, long)]
        msi: Option<String>,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Generate { file, msi, output } => {
            if !file.exists() {
                eprintln!("Error: File not found: {}", file.display());
                std::process::exit(1);
            }

            let content = std::fs::read_to_string(&file)?;
            let msi_name = msi.unwrap_or_else(|| {
                file.file_stem()
                    .and_then(|s| s.to_str())
                    .map(|s| format!("{}.msi", s))
                    .unwrap_or_else(|| "output.msi".to_string())
            });

            let config = IntuneGenerator::from_wix(&content, &msi_name);

            // Create output directory if needed
            std::fs::create_dir_all(&output)?;

            // Generate manifest
            let manifest = IntuneGenerator::generate_manifest(&config);
            let manifest_path = output.join("intune-manifest.json");
            std::fs::write(&manifest_path, &manifest)?;
            println!("Created: {}", manifest_path.display());

            // Generate install script
            let install_script = IntuneGenerator::generate_install_script(&config);
            let install_path = output.join("Install.ps1");
            std::fs::write(&install_path, &install_script)?;
            println!("Created: {}", install_path.display());

            // Generate uninstall script
            let uninstall_script = IntuneGenerator::generate_uninstall_script(&config);
            let uninstall_path = output.join("Uninstall.ps1");
            std::fs::write(&uninstall_path, &uninstall_script)?;
            println!("Created: {}", uninstall_path.display());

            // Generate detection script
            let detection_script = IntuneGenerator::generate_detection_script(&config);
            let detection_path = output.join("Detect.ps1");
            std::fs::write(&detection_path, &detection_script)?;
            println!("Created: {}", detection_path.display());

            // Generate instructions
            let instructions = IntuneGenerator::generate_prep_instructions(&config);
            let instructions_path = output.join("INTUNE_DEPLOYMENT.txt");
            std::fs::write(&instructions_path, &instructions)?;
            println!("Created: {}", instructions_path.display());

            println!();
            println!("Intune deployment files generated successfully!");
            println!();
            println!("Next steps:");
            println!("1. Copy your MSI file ({}) to the output directory", msi_name);
            println!("2. Run IntuneWinAppUtil.exe to create .intunewin package");
            println!("3. Upload to Microsoft Intune admin center");
        }

        Commands::Manifest { file, msi } => {
            if !file.exists() {
                eprintln!("Error: File not found: {}", file.display());
                std::process::exit(1);
            }

            let content = std::fs::read_to_string(&file)?;
            let msi_name = msi.unwrap_or_else(|| "output.msi".to_string());
            let config = IntuneGenerator::from_wix(&content, &msi_name);

            println!("{}", IntuneGenerator::generate_manifest(&config));
        }

        Commands::Scripts { file, msi, output } => {
            if !file.exists() {
                eprintln!("Error: File not found: {}", file.display());
                std::process::exit(1);
            }

            let content = std::fs::read_to_string(&file)?;
            let msi_name = msi.unwrap_or_else(|| "output.msi".to_string());
            let config = IntuneGenerator::from_wix(&content, &msi_name);

            std::fs::create_dir_all(&output)?;

            let install_path = output.join("Install.ps1");
            std::fs::write(&install_path, IntuneGenerator::generate_install_script(&config))?;
            println!("Created: {}", install_path.display());

            let uninstall_path = output.join("Uninstall.ps1");
            std::fs::write(&uninstall_path, IntuneGenerator::generate_uninstall_script(&config))?;
            println!("Created: {}", uninstall_path.display());
        }

        Commands::Instructions { file, msi } => {
            if !file.exists() {
                eprintln!("Error: File not found: {}", file.display());
                std::process::exit(1);
            }

            let content = std::fs::read_to_string(&file)?;
            let msi_name = msi.unwrap_or_else(|| "output.msi".to_string());
            let config = IntuneGenerator::from_wix(&content, &msi_name);

            println!("{}", IntuneGenerator::generate_prep_instructions(&config));
        }

        Commands::Detection { file, msi } => {
            if !file.exists() {
                eprintln!("Error: File not found: {}", file.display());
                std::process::exit(1);
            }

            let content = std::fs::read_to_string(&file)?;
            let msi_name = msi.unwrap_or_else(|| "output.msi".to_string());
            let config = IntuneGenerator::from_wix(&content, &msi_name);

            println!("{}", IntuneGenerator::generate_detection_script(&config));
        }
    }

    Ok(())
}
