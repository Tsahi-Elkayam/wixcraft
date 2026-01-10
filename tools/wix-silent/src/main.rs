//! wix-silent CLI - Silent install parameter generator
//!
//! Usage:
//!   wix-silent analyze product.wxs       # Extract properties and features
//!   wix-silent docs product.wxs          # Generate documentation
//!   wix-silent command product.msi       # Generate install command

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::collections::HashMap;
use std::path::PathBuf;
use wix_silent::*;

#[derive(Parser)]
#[command(name = "wix-silent")]
#[command(about = "Silent install parameter generator and validator")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Analyze WiX source for public properties and features
    Analyze {
        /// WiX source file(s)
        #[arg(required = true)]
        files: Vec<PathBuf>,

        /// Output format (text, json, markdown)
        #[arg(short, long, default_value = "text")]
        format: String,
    },

    /// Generate silent install documentation
    Docs {
        /// WiX source file(s)
        #[arg(required = true)]
        files: Vec<PathBuf>,

        /// Output file (default: stdout)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Generate silent install command line
    Command {
        /// MSI file path
        msi: String,

        /// Set property value (format: NAME=value)
        #[arg(short, long)]
        property: Vec<String>,

        /// Features to install (comma-separated)
        #[arg(short, long)]
        features: Option<String>,

        /// Log file path
        #[arg(short, long)]
        log: Option<String>,

        /// Generate uninstall command instead
        #[arg(long)]
        uninstall: bool,

        /// Generate repair command instead
        #[arg(long)]
        repair: bool,
    },

    /// Show standard MSI properties
    Properties {
        /// Output format (text, json)
        #[arg(short, long, default_value = "text")]
        format: String,
    },

    /// Validate property values
    Validate {
        /// Property to validate (format: NAME=value)
        #[arg(required = true)]
        property: String,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let analyzer = SilentAnalyzer::new();

    match cli.command {
        Commands::Analyze { files, format } => {
            let mut all_properties = Vec::new();
            let mut all_features = Vec::new();

            for file in &files {
                if !file.exists() {
                    eprintln!("Warning: File not found: {}", file.display());
                    continue;
                }
                let content = std::fs::read_to_string(file)?;
                all_properties.extend(analyzer.extract_properties(&content));
                all_features.extend(analyzer.extract_features(&content));
            }

            // Deduplicate by name
            all_properties.sort_by(|a, b| a.name.cmp(&b.name));
            all_properties.dedup_by(|a, b| a.name == b.name);

            match format.as_str() {
                "json" => {
                    let output = serde_json::json!({
                        "properties": all_properties,
                        "features": all_features,
                    });
                    println!("{}", serde_json::to_string_pretty(&output)?);
                }
                "markdown" => {
                    let docs = analyzer.generate_docs(&all_properties, &all_features);
                    println!("{}", docs);
                }
                _ => {
                    println!("Public Properties");
                    println!("{}", "=".repeat(60));
                    for prop in &all_properties {
                        println!(
                            "  {} ({}){}",
                            prop.name,
                            prop.property_type,
                            if prop.secure { " [SECURE]" } else { "" }
                        );
                        if let Some(ref default) = prop.default_value {
                            println!("    Default: {}", default);
                        }
                        if let Some(ref desc) = prop.description {
                            println!("    {}", desc);
                        }
                    }

                    println!();
                    println!("Features");
                    println!("{}", "=".repeat(60));
                    for feature in &all_features {
                        println!(
                            "  {} - {} (Level: {})",
                            feature.id,
                            feature.title.as_deref().unwrap_or(""),
                            feature.level
                        );
                    }
                }
            }
        }

        Commands::Docs { files, output } => {
            let mut all_properties = Vec::new();
            let mut all_features = Vec::new();

            for file in &files {
                if !file.exists() {
                    eprintln!("Warning: File not found: {}", file.display());
                    continue;
                }
                let content = std::fs::read_to_string(file)?;
                all_properties.extend(analyzer.extract_properties(&content));
                all_features.extend(analyzer.extract_features(&content));
            }

            let docs = analyzer.generate_docs(&all_properties, &all_features);

            if let Some(out_path) = output {
                std::fs::write(&out_path, &docs)?;
                println!("Documentation written to: {}", out_path.display());
            } else {
                println!("{}", docs);
            }
        }

        Commands::Command {
            msi,
            property,
            features,
            log,
            uninstall,
            repair,
        } => {
            if uninstall {
                let mut cmd = format!("msiexec /x \"{}\" /qn", msi);
                if let Some(ref log_file) = log {
                    cmd.push_str(&format!(" /l*v \"{}\"", log_file));
                }
                println!("{}", cmd);
            } else if repair {
                let mut cmd = format!("msiexec /f \"{}\" /qn", msi);
                if let Some(ref log_file) = log {
                    cmd.push_str(&format!(" /l*v \"{}\"", log_file));
                }
                println!("{}", cmd);
            } else {
                let mut props = HashMap::new();
                for p in &property {
                    let parts: Vec<&str> = p.splitn(2, '=').collect();
                    if parts.len() == 2 {
                        props.insert(parts[0].to_string(), parts[1].to_string());
                    }
                }

                let feats: Option<Vec<String>> = features.map(|f| {
                    f.split(',')
                        .map(|s| s.trim().to_string())
                        .collect()
                });

                let cmd = analyzer.generate_command(
                    &msi,
                    &props,
                    feats.as_deref(),
                    log.as_deref(),
                );
                println!("{}", cmd);
            }
        }

        Commands::Properties { format } => {
            let props = get_standard_properties();

            if format == "json" {
                println!("{}", serde_json::to_string_pretty(&props)?);
            } else {
                println!("Standard MSI Properties for Silent Install");
                println!("{}", "=".repeat(60));
                println!();

                for prop in &props {
                    println!("{}", prop.name);
                    println!("  Type: {}", prop.property_type);
                    if let Some(ref default) = prop.default_value {
                        println!("  Default: {}", default);
                    }
                    if let Some(ref desc) = prop.description {
                        println!("  {}", desc);
                    }
                    println!();
                }
            }
        }

        Commands::Validate { property } => {
            let parts: Vec<&str> = property.splitn(2, '=').collect();
            if parts.len() != 2 {
                eprintln!("Invalid format. Use: NAME=value");
                std::process::exit(1);
            }

            let name = parts[0];
            let value = parts[1];

            println!("Validating: {}={}", name, value);

            let mut issues = Vec::new();

            // Check if public property (uppercase)
            if !name.chars().all(|c| c.is_uppercase() || c.is_numeric() || c == '_') {
                issues.push("Property name should be UPPERCASE for public properties");
            }

            // Check specific properties
            match name {
                "ALLUSERS" => {
                    if value != "1" && value != "" && value != "2" {
                        issues.push("ALLUSERS should be 1, 2, or empty");
                    }
                }
                "REBOOT" => {
                    if !["Force", "Suppress", "ReallySuppress"].contains(&value) {
                        issues.push("REBOOT should be Force, Suppress, or ReallySuppress");
                    }
                }
                name if name.ends_with("DIR") || name.ends_with("PATH") => {
                    if !value.contains('\\') && !value.contains('/') && !value.starts_with('[') {
                        issues.push("Path properties should contain path separators or WiX properties");
                    }
                }
                _ => {}
            }

            if issues.is_empty() {
                println!("Valid");
            } else {
                println!("Issues:");
                for issue in &issues {
                    println!("  - {}", issue);
                }
                std::process::exit(1);
            }
        }
    }

    Ok(())
}
