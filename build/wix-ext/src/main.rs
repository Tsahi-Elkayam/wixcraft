//! wix-ext CLI - WiX extension manager with version pinning
//!
//! Usage:
//!   wix-ext init              # Create wixext.toml configuration
//!   wix-ext add ui 5.0.0      # Add extension with pinned version
//!   wix-ext list              # List installed extensions
//!   wix-ext sync              # Sync extensions from config
//!   wix-ext check             # Check for issues
//!   wix-ext detect *.wxs      # Detect required extensions

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::fs;
use std::path::PathBuf;
use wix_ext::{detect_used_extensions, ExtensionConfig, ExtensionManager, IssueType, KNOWN_EXTENSIONS};

#[derive(Parser)]
#[command(name = "wix-ext")]
#[command(about = "WiX extension manager with version pinning for CI/CD")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Path to wixext.toml config file
    #[arg(short, long, default_value = "wixext.toml")]
    config: PathBuf,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new wixext.toml configuration
    Init {
        /// WiX version to target
        #[arg(short, long, default_value = "5.0")]
        wix_version: String,
    },

    /// Add an extension with pinned version
    Add {
        /// Extension short name (e.g., ui, util, bal)
        name: String,

        /// Version to pin (e.g., 5.0.0)
        version: String,
    },

    /// Remove an extension from configuration
    Remove {
        /// Extension short name
        name: String,
    },

    /// List installed extensions
    List {
        /// Output format (text, json)
        #[arg(short, long, default_value = "text")]
        format: String,

        /// Also show available extensions
        #[arg(short, long)]
        available: bool,
    },

    /// Sync installed extensions with configuration
    Sync {
        /// Dry run - show what would be done
        #[arg(short, long)]
        dry_run: bool,
    },

    /// Check for extension issues
    Check {
        /// Output format (text, json)
        #[arg(short, long, default_value = "text")]
        format: String,
    },

    /// Detect extensions used in WiX files
    Detect {
        /// WiX file(s) to scan
        #[arg(required = true)]
        files: Vec<PathBuf>,

        /// Add detected extensions to config
        #[arg(short, long)]
        add: bool,

        /// Default version for detected extensions
        #[arg(short, long, default_value = "5.0.0")]
        version: String,
    },

    /// Generate lockfile for CI/CD
    Lock,

    /// Show available extensions
    Available {
        /// Output format (text, json)
        #[arg(short, long, default_value = "text")]
        format: String,
    },

    /// Generate CI/CD configuration
    CiConfig {
        /// CI system (github, azure, gitlab)
        #[arg(default_value = "github")]
        system: String,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let manager = ExtensionManager::new();

    match cli.command {
        Commands::Init { wix_version } => {
            if cli.config.exists() {
                println!("Config file already exists: {}", cli.config.display());
                println!("Use 'wix-ext add' to add extensions.");
                return Ok(());
            }

            let config = ExtensionConfig {
                wix_version,
                extensions: std::collections::HashMap::new(),
            };
            config.save(&cli.config)?;

            println!("Created {}", cli.config.display());
            println!();
            println!("Next steps:");
            println!("  wix-ext add ui 5.0.0       # Add UI extension");
            println!("  wix-ext add util 5.0.0     # Add Util extension");
            println!("  wix-ext sync               # Install extensions");
        }

        Commands::Add { name, version } => {
            let mut config = if cli.config.exists() {
                ExtensionConfig::from_file(&cli.config)?
            } else {
                ExtensionConfig::default()
            };

            config.add(&name, &version)?;
            config.save(&cli.config)?;

            println!("Added {} v{} to config", name, version);
            println!("Run 'wix-ext sync' to install.");
        }

        Commands::Remove { name } => {
            let mut config = ExtensionConfig::from_file(&cli.config)?;

            if config.remove(&name) {
                config.save(&cli.config)?;
                println!("Removed {} from config", name);
            } else {
                println!("{} not found in config", name);
            }
        }

        Commands::List { format, available } => {
            if available {
                // Show available extensions
                if format == "json" {
                    let list: Vec<_> = KNOWN_EXTENSIONS
                        .iter()
                        .map(|(name, package, desc)| {
                            serde_json::json!({
                                "name": name,
                                "package": package,
                                "description": desc
                            })
                        })
                        .collect();
                    println!("{}", serde_json::to_string_pretty(&list)?);
                } else {
                    println!("Available WiX Extensions:");
                    println!("{}", "=".repeat(60));
                    println!("{:<12} {:<35} {}", "Name", "Package", "Description");
                    println!("{}", "-".repeat(80));
                    for (name, package, desc) in KNOWN_EXTENSIONS {
                        println!("{:<12} {:<35} {}", name, package, desc);
                    }
                }
                return Ok(());
            }

            // Show installed extensions
            match manager.list_installed() {
                Ok(installed) => {
                    if format == "json" {
                        println!("{}", serde_json::to_string_pretty(&installed)?);
                    } else {
                        println!("Installed Extensions:");
                        println!("{}", "=".repeat(50));

                        if installed.is_empty() {
                            println!("No extensions installed.");
                        } else {
                            println!("{:<12} {:<30} {}", "Name", "Package", "Version");
                            println!("{}", "-".repeat(60));
                            for ext in &installed {
                                println!("{:<12} {:<30} {}", ext.name, ext.package, ext.version);
                            }
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Failed to list extensions: {}", e);
                    eprintln!("Is WiX Toolset installed?");
                }
            }
        }

        Commands::Sync { dry_run } => {
            let config = ExtensionConfig::from_file(&cli.config)?;

            if dry_run {
                println!("Dry run - would sync the following extensions:");
                for (name, entry) in &config.extensions {
                    println!("  {} -> {} v{}", name, entry.package, entry.version);
                }
                return Ok(());
            }

            println!("Syncing extensions from {}...", cli.config.display());
            match manager.sync(&config) {
                Ok(actions) => {
                    if actions.is_empty() {
                        println!("All extensions are up to date.");
                    } else {
                        for action in actions {
                            println!("  {}", action);
                        }
                        println!("\nSync complete.");
                    }
                }
                Err(e) => {
                    eprintln!("Sync failed: {}", e);
                    std::process::exit(1);
                }
            }
        }

        Commands::Check { format } => {
            let config = ExtensionConfig::from_file(&cli.config)?;

            match manager.check(&config) {
                Ok(issues) => {
                    if format == "json" {
                        println!("{}", serde_json::to_string_pretty(&issues)?);
                    } else if issues.is_empty() {
                        println!("No issues found.");
                    } else {
                        println!("Extension Issues:");
                        println!("{}", "=".repeat(50));

                        for issue in &issues {
                            let level = match issue.issue_type {
                                IssueType::VersionMismatch => "WARNING",
                                IssueType::NotInstalled => "ERROR",
                                IssueType::Prerelease => "INFO",
                                IssueType::Deprecated => "WARNING",
                                IssueType::UnknownExtension => "INFO",
                            };
                            println!("[{}] {}: {}", level, issue.extension, issue.message);
                            println!("       -> {}", issue.suggestion);
                        }

                        let error_count = issues
                            .iter()
                            .filter(|i| matches!(i.issue_type, IssueType::NotInstalled))
                            .count();

                        if error_count > 0 {
                            std::process::exit(1);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Check failed: {}", e);
                    std::process::exit(1);
                }
            }
        }

        Commands::Detect { files, add, version } => {
            let mut all_detected = Vec::new();

            for file in &files {
                let content = fs::read_to_string(file)?;
                let detected = detect_used_extensions(&content);

                if !detected.is_empty() {
                    println!("{}:", file.display());
                    for ext in &detected {
                        println!("  - {}", ext);
                        if !all_detected.contains(ext) {
                            all_detected.push(ext.clone());
                        }
                    }
                }
            }

            if all_detected.is_empty() {
                println!("No extensions detected in provided files.");
            } else if add {
                let mut config = if cli.config.exists() {
                    ExtensionConfig::from_file(&cli.config)?
                } else {
                    ExtensionConfig::default()
                };

                for ext in &all_detected {
                    if !config.extensions.contains_key(ext) {
                        config.add(ext, &version)?;
                        println!("Added {} v{}", ext, version);
                    }
                }

                config.save(&cli.config)?;
                println!("\nConfig updated. Run 'wix-ext sync' to install.");
            }
        }

        Commands::Lock => {
            let config = ExtensionConfig::from_file(&cli.config)?;
            let lockfile = manager.generate_lockfile(&config);

            let lock_path = cli.config.with_extension("lock");
            fs::write(&lock_path, &lockfile)?;

            println!("Generated {}", lock_path.display());
        }

        Commands::Available { format } => {
            if format == "json" {
                let list: Vec<_> = KNOWN_EXTENSIONS
                    .iter()
                    .map(|(name, package, desc)| {
                        serde_json::json!({
                            "name": name,
                            "package": package,
                            "description": desc
                        })
                    })
                    .collect();
                println!("{}", serde_json::to_string_pretty(&list)?);
            } else {
                println!("Available WiX Extensions:");
                println!("{}", "=".repeat(70));
                println!("{:<12} {:<40} {}", "Name", "Package", "Description");
                println!("{}", "-".repeat(85));
                for (name, package, desc) in KNOWN_EXTENSIONS {
                    println!("{:<12} {:<40} {}", name, package, desc);
                }
            }
        }

        Commands::CiConfig { system } => {
            match system.as_str() {
                "github" => println!("{}", CI_GITHUB),
                "azure" => println!("{}", CI_AZURE),
                "gitlab" => println!("{}", CI_GITLAB),
                _ => {
                    eprintln!("Unknown CI system: {}", system);
                    eprintln!("Supported: github, azure, gitlab");
                    std::process::exit(1);
                }
            }
        }
    }

    Ok(())
}

const CI_GITHUB: &str = r#"# GitHub Actions workflow for WiX builds
# Add to .github/workflows/build.yml

name: Build MSI

on: [push, pull_request]

jobs:
  build:
    runs-on: windows-latest

    steps:
      - uses: actions/checkout@v4

      - name: Install WiX Toolset
        run: |
          dotnet tool install --global wix --version 5.0.0

      - name: Install wix-ext (optional)
        run: |
          # Download wix-ext from releases
          # curl -LO https://github.com/wixcraft/wixcraft/releases/latest/download/wix-ext.exe

      - name: Sync Extensions
        run: |
          wix extension add WixToolset.UI.wixext/5.0.0
          wix extension add WixToolset.Util.wixext/5.0.0
          # Or use: wix-ext sync

      - name: Build MSI
        run: wix build src/installer/*.wxs -o output/installer.msi

      - name: Upload Artifact
        uses: actions/upload-artifact@v4
        with:
          name: installer
          path: output/installer.msi
"#;

const CI_AZURE: &str = r#"# Azure DevOps pipeline for WiX builds
# Add to azure-pipelines.yml

trigger:
  - main

pool:
  vmImage: 'windows-latest'

steps:
  - task: UseDotNet@2
    inputs:
      packageType: 'sdk'
      version: '8.0.x'

  - script: |
      dotnet tool install --global wix --version 5.0.0
    displayName: 'Install WiX Toolset'

  - script: |
      wix extension add WixToolset.UI.wixext/5.0.0
      wix extension add WixToolset.Util.wixext/5.0.0
    displayName: 'Install Extensions'

  - script: |
      wix build src/installer/*.wxs -o $(Build.ArtifactStagingDirectory)/installer.msi
    displayName: 'Build MSI'

  - task: PublishBuildArtifacts@1
    inputs:
      PathtoPublish: '$(Build.ArtifactStagingDirectory)'
      ArtifactName: 'installer'
"#;

const CI_GITLAB: &str = r#"# GitLab CI pipeline for WiX builds
# Add to .gitlab-ci.yml

stages:
  - build

build-msi:
  stage: build
  tags:
    - windows
  script:
    - dotnet tool install --global wix --version 5.0.0
    - $env:PATH += ";$env:USERPROFILE\.dotnet\tools"
    - wix extension add WixToolset.UI.wixext/5.0.0
    - wix extension add WixToolset.Util.wixext/5.0.0
    - wix build src/installer/*.wxs -o output/installer.msi
  artifacts:
    paths:
      - output/installer.msi
    expire_in: 1 week
"#;
