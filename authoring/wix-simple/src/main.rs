//! wix-simple CLI - Generate WiX installers from simple config
//!
//! Usage:
//!   wix-simple init > myapp.json          # Create example config
//!   wix-simple generate myapp.json        # Generate WiX from config
//!   wix-simple quick --name MyApp --file app.exe  # Quick one-liner

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use wix_simple::*;

#[derive(Parser)]
#[command(name = "wix-simple")]
#[command(about = "Generate WiX installers from simple configuration")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate example configuration file
    Init {
        /// Output file (default: stdout)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Generate WiX source from config file
    Generate {
        /// Config file (JSON)
        config: PathBuf,

        /// Output file (default: stdout)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Quick generator with command-line options
    Quick {
        /// Application name
        #[arg(short, long)]
        name: String,

        /// Version
        #[arg(short, long, default_value = "1.0.0")]
        version: String,

        /// Manufacturer
        #[arg(short, long, default_value = "My Company")]
        manufacturer: String,

        /// Files to install
        #[arg(short, long)]
        file: Vec<String>,

        /// Create start menu shortcut
        #[arg(long)]
        shortcut: bool,

        /// Platform (x86, x64)
        #[arg(short, long, default_value = "x64")]
        platform: String,

        /// Output file
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Validate a config file
    Validate {
        /// Config file to validate
        config: PathBuf,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init { output } => {
            let config = SimpleGenerator::example_config();
            let json = serde_json::to_string_pretty(&config)?;

            if let Some(out_path) = output {
                std::fs::write(&out_path, &json)?;
                println!("Example config written to: {}", out_path.display());
            } else {
                println!("{}", json);
            }
        }

        Commands::Generate { config, output } => {
            if !config.exists() {
                eprintln!("Error: Config file not found: {}", config.display());
                std::process::exit(1);
            }

            let json = std::fs::read_to_string(&config)?;
            let wix = SimpleGenerator::generate_from_json(&json)?;

            if let Some(out_path) = output {
                std::fs::write(&out_path, &wix)?;
                println!("WiX source written to: {}", out_path.display());
            } else {
                println!("{}", wix);
            }
        }

        Commands::Quick {
            name,
            version,
            manufacturer,
            file,
            shortcut,
            platform,
            output,
        } => {
            if file.is_empty() {
                eprintln!("Error: At least one file is required (--file)");
                std::process::exit(1);
            }

            let files: Vec<FileConfig> = file
                .iter()
                .map(|f| FileConfig {
                    source: f.clone(),
                    subdir: None,
                })
                .collect();

            let shortcuts = if shortcut {
                // Use first executable file for shortcut
                let exe_file = file
                    .iter()
                    .find(|f| f.to_lowercase().ends_with(".exe"))
                    .unwrap_or(&file[0]);
                let target = std::path::Path::new(exe_file)
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or(exe_file);

                vec![ShortcutConfig {
                    name: name.clone(),
                    target: target.to_string(),
                    location: "start_menu".to_string(),
                }]
            } else {
                vec![]
            };

            let config = SimpleConfig {
                name,
                version,
                manufacturer,
                files,
                shortcuts,
                platform,
                install_dir: None,
                upgrade_code: None,
            };

            let wix = SimpleGenerator::generate(&config);

            if let Some(out_path) = output {
                std::fs::write(&out_path, &wix)?;
                println!("WiX source written to: {}", out_path.display());
            } else {
                println!("{}", wix);
            }
        }

        Commands::Validate { config } => {
            if !config.exists() {
                eprintln!("Error: Config file not found: {}", config.display());
                std::process::exit(1);
            }

            let json = std::fs::read_to_string(&config)?;
            match serde_json::from_str::<SimpleConfig>(&json) {
                Ok(cfg) => {
                    println!("Configuration is valid!");
                    println!();
                    println!("Summary:");
                    println!("  Name: {}", cfg.name);
                    println!("  Version: {}", cfg.version);
                    println!("  Manufacturer: {}", cfg.manufacturer);
                    println!("  Platform: {}", cfg.platform);
                    println!("  Files: {}", cfg.files.len());
                    println!("  Shortcuts: {}", cfg.shortcuts.len());

                    if cfg.files.is_empty() {
                        println!();
                        println!("Warning: No files specified. Add files to the 'files' array.");
                    }
                }
                Err(e) => {
                    eprintln!("Configuration is invalid:");
                    eprintln!("  {}", e);
                    std::process::exit(1);
                }
            }
        }
    }

    Ok(())
}
