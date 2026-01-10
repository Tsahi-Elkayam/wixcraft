//! wix-sublime - Sublime Text package generator for WiX
//!
//! Generates Sublime Text package with language support.

use clap::{Parser, Subcommand};
use std::fs;
use std::path::PathBuf;
use wix_sublime::PackageGenerator;

#[derive(Parser)]
#[command(name = "wix-sublime")]
#[command(about = "Generate Sublime Text package for WiX language support")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate complete Sublime Text package
    Generate {
        /// Output directory for the package
        #[arg(short, long, default_value = "WiX")]
        output: PathBuf,
    },
    /// Generate only syntax definition
    Syntax {
        /// Output file path
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    /// Generate only settings
    Settings {
        /// Output file path
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    /// Generate only build system
    Build {
        /// Output file path
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    /// Generate only completions
    Completions {
        /// Output file path
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Generate { output } => {
            // Create directory structure
            fs::create_dir_all(&output)?;

            // Generate syntax file
            fs::write(output.join("WiX.sublime-syntax"), PackageGenerator::generate_syntax())?;

            // Generate settings
            fs::write(output.join("WiX.sublime-settings"), PackageGenerator::generate_settings())?;

            // Generate build system
            fs::write(output.join("WiX.sublime-build"), PackageGenerator::generate_build_system())?;

            // Generate completions
            fs::write(output.join("WiX.sublime-completions"), PackageGenerator::generate_completions())?;

            // Generate README
            let readme = r#"# WiX Package for Sublime Text

Language support for WiX Toolset files in Sublime Text.

## Features

- Syntax highlighting for `.wxs`, `.wxi`, and `.wxl` files
- Code completions for common WiX elements
- Build system for compiling WiX projects

## Installation

### Package Control (Recommended)
1. Open Command Palette (Ctrl+Shift+P / Cmd+Shift+P)
2. Select "Package Control: Install Package"
3. Search for "WiX"

### Manual Installation
1. Download this package
2. Open Sublime Text
3. Go to Preferences > Browse Packages...
4. Copy the WiX folder to the Packages directory

## Build Commands

- **Build**: Compile the current file with `wix build`
- **Build Release**: Compile with x64 architecture
- **Validate**: Run wix-lint on the current file
"#;
            fs::write(output.join("README.md"), readme)?;

            println!("Generated Sublime Text package in: {}", output.display());
            println!("\nInstallation:");
            println!("  Copy the '{}' folder to your Sublime Text Packages directory", output.display());
        }

        Commands::Syntax { output } => {
            let syntax = PackageGenerator::generate_syntax();
            if let Some(path) = output {
                fs::write(&path, &syntax)?;
                println!("Generated syntax at: {}", path.display());
            } else {
                println!("{}", syntax);
            }
        }

        Commands::Settings { output } => {
            let settings = PackageGenerator::generate_settings();
            if let Some(path) = output {
                fs::write(&path, &settings)?;
                println!("Generated settings at: {}", path.display());
            } else {
                println!("{}", settings);
            }
        }

        Commands::Build { output } => {
            let build = PackageGenerator::generate_build_system();
            if let Some(path) = output {
                fs::write(&path, &build)?;
                println!("Generated build system at: {}", path.display());
            } else {
                println!("{}", build);
            }
        }

        Commands::Completions { output } => {
            let completions = PackageGenerator::generate_completions();
            if let Some(path) = output {
                fs::write(&path, &completions)?;
                println!("Generated completions at: {}", path.display());
            } else {
                println!("{}", completions);
            }
        }
    }

    Ok(())
}
