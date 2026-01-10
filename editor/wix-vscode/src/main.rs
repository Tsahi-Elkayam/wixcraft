//! wix-vscode - VS Code extension generator for WiX
//!
//! Generates VS Code extension package with language support.

use clap::{Parser, Subcommand};
use std::fs;
use std::path::PathBuf;
use wix_vscode::{ExtensionGenerator, ExtensionManifest, generate_extension_ts};

#[derive(Parser)]
#[command(name = "wix-vscode")]
#[command(about = "Generate VS Code extension for WiX language support")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate complete VS Code extension
    Generate {
        /// Output directory for the extension
        #[arg(short, long, default_value = "wix-vscode-extension")]
        output: PathBuf,
    },
    /// Generate only package.json
    Manifest {
        /// Output file path
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    /// Generate only the TypeScript source
    Extension {
        /// Output file path
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    /// Generate only the tsconfig.json
    Tsconfig {
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
            fs::create_dir_all(output.join("src"))?;
            fs::create_dir_all(output.join("syntaxes"))?;
            fs::create_dir_all(output.join("snippets"))?;

            // Generate package.json
            let manifest = ExtensionManifest::new();
            fs::write(output.join("package.json"), manifest.to_json())?;

            // Generate extension.ts
            fs::write(output.join("src/extension.ts"), generate_extension_ts())?;

            // Generate tsconfig.json
            fs::write(output.join("tsconfig.json"), ExtensionGenerator::generate_tsconfig())?;

            // Generate .vscodeignore
            let vscodeignore = r#".vscode/**
.vscode-test/**
src/**
**/*.ts
**/*.map
.gitignore
tsconfig.json
"#;
            fs::write(output.join(".vscodeignore"), vscodeignore)?;

            // Generate README.md
            let readme = r#"# WiX Language Support

Language support for WiX Toolset files in Visual Studio Code.

## Features

- Syntax highlighting for `.wxs`, `.wxi`, and `.wxl` files
- Code snippets for common WiX patterns
- IntelliSense support via WiX Language Server

## Installation

1. Build the extension: `npm install && npm run compile`
2. Package: `npx vsce package`
3. Install the `.vsix` file in VS Code

## Configuration

- `wix.enableLsp`: Enable/disable WiX Language Server (default: true)
- `wix.lspPath`: Path to wix-lsp executable (default: "wix-lsp")
"#;
            fs::write(output.join("README.md"), readme)?;

            println!("Generated VS Code extension in: {}", output.display());
            println!("\nNext steps:");
            println!("  1. cd {}", output.display());
            println!("  2. npm install");
            println!("  3. npm run compile");
            println!("  4. npx vsce package");
        }

        Commands::Manifest { output } => {
            let manifest = ExtensionManifest::new();
            let json = manifest.to_json();
            if let Some(path) = output {
                fs::write(&path, &json)?;
                println!("Generated package.json at: {}", path.display());
            } else {
                println!("{}", json);
            }
        }

        Commands::Extension { output } => {
            let ts = generate_extension_ts();
            if let Some(path) = output {
                fs::write(&path, &ts)?;
                println!("Generated extension.ts at: {}", path.display());
            } else {
                println!("{}", ts);
            }
        }

        Commands::Tsconfig { output } => {
            let config = ExtensionGenerator::generate_tsconfig();
            if let Some(path) = output {
                fs::write(&path, &config)?;
                println!("Generated tsconfig.json at: {}", path.display());
            } else {
                println!("{}", config);
            }
        }
    }

    Ok(())
}
