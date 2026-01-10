//! wix-preview CLI - Preview installer without building
//!
//! Usage:
//!   wix-preview product.wxs             # Show installation preview
//!   wix-preview product.wxs --json      # JSON output
//!   wix-preview product.wxs --tree      # Tree view

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use wix_preview::*;

#[derive(Parser)]
#[command(name = "wix-preview")]
#[command(about = "Preview installer file layout without building")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Preview installation layout
    Show {
        /// WiX source file(s)
        #[arg(required = true)]
        files: Vec<PathBuf>,

        /// Output format (text, json, tree)
        #[arg(short, long, default_value = "tree")]
        format: String,
    },

    /// Show files that will be installed
    Files {
        /// WiX source file(s)
        #[arg(required = true)]
        files: Vec<PathBuf>,

        /// Group by directory
        #[arg(long)]
        by_directory: bool,
    },

    /// Show feature tree
    Features {
        /// WiX source file(s)
        #[arg(required = true)]
        files: Vec<PathBuf>,
    },

    /// Show registry entries
    Registry {
        /// WiX source file(s)
        #[arg(required = true)]
        files: Vec<PathBuf>,
    },

    /// Show shortcuts
    Shortcuts {
        /// WiX source file(s)
        #[arg(required = true)]
        files: Vec<PathBuf>,
    },

    /// Show services
    Services {
        /// WiX source file(s)
        #[arg(required = true)]
        files: Vec<PathBuf>,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Show { files, format } => {
            let preview = load_preview(&files)?;

            match format.as_str() {
                "json" => {
                    println!("{}", serde_json::to_string_pretty(&preview)?);
                }
                "tree" => {
                    println!("{}", PreviewGenerator::generate_tree(&preview));
                }
                _ => {
                    print_preview(&preview);
                }
            }
        }

        Commands::Files { files, by_directory } => {
            let preview = load_preview(&files)?;

            println!("Files to be installed ({} total):", preview.total_files);
            println!("{}", "=".repeat(50));

            if by_directory {
                for dir in &preview.directories {
                    if !dir.files.is_empty() {
                        println!("\n{}:", dir.name);
                        for file in &dir.files {
                            println!("  {}", file.name);
                        }
                    }
                }
            } else {
                for dir in &preview.directories {
                    for file in &dir.files {
                        println!("  {} -> {}/{}", file.source, dir.name, file.name);
                    }
                }
            }
        }

        Commands::Features { files } => {
            let preview = load_preview(&files)?;

            println!("Feature Tree:");
            println!("{}", "=".repeat(50));

            for feature in &preview.features {
                print_feature(feature, 0);
            }
        }

        Commands::Registry { files } => {
            let preview = load_preview(&files)?;

            println!("Registry Entries ({}):", preview.registry.len());
            println!("{}", "=".repeat(50));

            for entry in &preview.registry {
                println!(
                    "{}\\{}",
                    entry.root, entry.key
                );
                if let Some(ref name) = entry.name {
                    println!(
                        "  {} = {} ({})",
                        name,
                        entry.value.as_deref().unwrap_or(""),
                        entry.value_type
                    );
                }
            }
        }

        Commands::Shortcuts { files } => {
            let preview = load_preview(&files)?;

            println!("Shortcuts ({}):", preview.shortcuts.len());
            println!("{}", "=".repeat(50));

            for shortcut in &preview.shortcuts {
                println!("{}", shortcut.name);
                println!("  Target: {}", shortcut.target);
                println!("  Location: {}", shortcut.directory);
                if let Some(ref icon) = shortcut.icon {
                    println!("  Icon: {}", icon);
                }
                println!();
            }
        }

        Commands::Services { files } => {
            let preview = load_preview(&files)?;

            println!("Services ({}):", preview.services.len());
            println!("{}", "=".repeat(50));

            for service in &preview.services {
                println!("{}", service.name);
                if let Some(ref display) = service.display_name {
                    println!("  Display Name: {}", display);
                }
                println!("  Start Type: {}", service.start_type);
                if let Some(ref account) = service.account {
                    println!("  Account: {}", account);
                }
                println!();
            }
        }
    }

    Ok(())
}

fn load_preview(files: &[PathBuf]) -> Result<InstallPreview> {
    let mut all_content = String::new();

    for file in files {
        if !file.exists() {
            eprintln!("Warning: File not found: {}", file.display());
            continue;
        }
        all_content.push_str(&std::fs::read_to_string(file)?);
        all_content.push('\n');
    }

    Ok(PreviewGenerator::generate(&all_content))
}

fn print_preview(preview: &InstallPreview) {
    println!(
        "{} v{}",
        preview.product_name.as_deref().unwrap_or("Unknown Product"),
        preview.product_version.as_deref().unwrap_or("0.0.0")
    );
    println!(
        "Manufacturer: {}",
        preview.manufacturer.as_deref().unwrap_or("Unknown")
    );
    println!();
    println!("Summary:");
    println!("  Files: {}", preview.total_files);
    println!("  Directories: {}", preview.directories.len());
    println!("  Features: {}", preview.features.len());
    println!("  Registry Entries: {}", preview.registry.len());
    println!("  Shortcuts: {}", preview.shortcuts.len());
    println!("  Services: {}", preview.services.len());
}

fn print_feature(feature: &FeatureEntry, indent: usize) {
    let prefix = "  ".repeat(indent);
    let level_str = if feature.level == 0 {
        " (disabled)"
    } else {
        ""
    };

    println!(
        "{}[{}] {}{}",
        prefix,
        feature.id,
        feature.title.as_deref().unwrap_or(""),
        level_str
    );

    if let Some(ref desc) = feature.description {
        println!("{}  {}", prefix, desc);
    }

    if !feature.components.is_empty() {
        println!("{}  Components: {}", prefix, feature.components.join(", "));
    }

    for child in &feature.children {
        print_feature(child, indent + 1);
    }
}
