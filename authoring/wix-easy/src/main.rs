//! wix-easy CLI - Ansible-like YAML to WiX/MSI generator
//!
//! Usage:
//!   wix-easy init                    # Create sample wix-easy.yaml
//!   wix-easy generate wix-easy.yaml  # Generate WiX XML
//!   wix-easy build wix-easy.yaml     # Generate and build MSI
//!   wix-easy validate wix-easy.yaml  # Validate configuration

use clap::{Parser, Subcommand};
use std::path::PathBuf;
use wix_easy::InstallerDef;

mod tui;

#[derive(Parser)]
#[command(name = "wix-easy")]
#[command(author, version, about = "Ansible-like YAML to WiX/MSI generator")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a sample wix-easy.yaml configuration
    Init {
        /// Output file name
        #[arg(short, long, default_value = "wix-easy.yaml")]
        output: PathBuf,

        /// Template: minimal, basic, service, bundle
        #[arg(short, long, default_value = "basic")]
        template: String,
    },

    /// Generate WiX XML from YAML configuration
    Generate {
        /// YAML configuration file
        config: PathBuf,

        /// Output WiX file (stdout if not specified)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Validate YAML configuration
    Validate {
        /// YAML configuration file
        config: PathBuf,

        /// Show detailed output
        #[arg(short, long)]
        verbose: bool,
    },

    /// Build MSI from YAML configuration
    Build {
        /// YAML configuration file
        config: PathBuf,

        /// Output MSI file
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Keep generated .wxs files
        #[arg(short, long)]
        keep: bool,

        /// Show WiX build output
        #[arg(short, long)]
        verbose: bool,
    },

    /// Show example configurations
    Examples {
        /// Example type: basic, service, registry, shortcuts, bundle
        #[arg(default_value = "basic")]
        example: String,
    },

    /// Interactive TUI wizard for creating configurations
    Interactive {
        /// Output YAML file
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init { output, template } => cmd_init(&output, &template),
        Commands::Generate { config, output } => cmd_generate(&config, output),
        Commands::Validate { config, verbose } => cmd_validate(&config, verbose),
        Commands::Build { config, output, keep, verbose } => cmd_build(&config, output, keep, verbose),
        Commands::Examples { example } => cmd_examples(&example),
        Commands::Interactive { output } => tui::run_interactive(output),
    }
}

fn cmd_init(output: &PathBuf, template: &str) -> anyhow::Result<()> {
    let content = match template {
        "minimal" => TEMPLATE_MINIMAL,
        "service" => TEMPLATE_SERVICE,
        "bundle" => TEMPLATE_BUNDLE,
        _ => TEMPLATE_BASIC,
    };

    std::fs::write(output, content)?;
    println!("Created {} (template: {})", output.display(), template);
    println!();
    println!("Next steps:");
    println!("  1. Edit {} with your application details", output.display());
    println!("  2. Run: wix-easy generate {}", output.display());
    println!("  3. Build with WiX: wix build output.wxs");

    Ok(())
}

fn cmd_generate(config: &PathBuf, output: Option<PathBuf>) -> anyhow::Result<()> {
    let def = InstallerDef::from_file(config)?;
    let base_path = config.parent();
    let wix = def.generate_wix(base_path)?;

    match output {
        Some(path) => {
            std::fs::write(&path, &wix)?;
            println!("Generated: {}", path.display());
        }
        None => {
            print!("{}", wix);
        }
    }

    Ok(())
}

fn cmd_validate(config: &PathBuf, verbose: bool) -> anyhow::Result<()> {
    match InstallerDef::from_file(config) {
        Ok(def) => {
            println!("Configuration is valid");
            println!();
            println!("Package:");
            println!("  Name:         {}", def.package.name);
            println!("  Version:      {}", def.package.version);
            println!("  Manufacturer: {}", def.package.manufacturer);

            if verbose {
                println!();
                println!("Details:");
                println!("  Files:        {}", def.install.files.len());
                println!("  Shortcuts:    {}", def.shortcuts.len());
                println!("  Registry:     {}", def.registry.len());
                println!("  Services:     {}", def.services.len());
                println!("  Environment:  {}", def.environment.len());
                println!("  Prerequisites:{}", def.prerequisites.len());
                println!("  Features:     {}", if def.features.is_empty() { 1 } else { def.features.len() });
            }

            Ok(())
        }
        Err(e) => {
            eprintln!("Validation failed: {}", e);
            std::process::exit(1);
        }
    }
}

fn cmd_build(config: &PathBuf, output: Option<PathBuf>, keep: bool, verbose: bool) -> anyhow::Result<()> {
    let def = InstallerDef::from_file(config)?;
    let base_path = config.parent();

    // Generate WiX XML
    let wix = def.generate_wix(base_path)?;

    // Determine output names
    let stem = config.file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("output");
    let wxs_path = PathBuf::from(format!("{}.wxs", stem));
    let msi_path = output.unwrap_or_else(|| PathBuf::from(format!("{}.msi", stem)));

    // Write WiX file
    std::fs::write(&wxs_path, &wix)?;
    println!("Generated: {}", wxs_path.display());

    // Try to build with WiX
    println!("Building MSI...");

    let status = std::process::Command::new("wix")
        .args(["build", wxs_path.to_str().unwrap(), "-o", msi_path.to_str().unwrap()])
        .stdout(if verbose { std::process::Stdio::inherit() } else { std::process::Stdio::null() })
        .stderr(std::process::Stdio::inherit())
        .status();

    match status {
        Ok(s) if s.success() => {
            println!("Built: {}", msi_path.display());

            if !keep {
                let _ = std::fs::remove_file(&wxs_path);
            }

            Ok(())
        }
        Ok(_) => {
            eprintln!("Build failed. The .wxs file has been kept for debugging.");
            std::process::exit(1);
        }
        Err(e) => {
            eprintln!("Failed to run WiX: {}", e);
            eprintln!();
            eprintln!("WiX Toolset not found. Install from: https://wixtoolset.org/");
            eprintln!();
            eprintln!("The generated .wxs file can be built manually:");
            eprintln!("  wix build {} -o {}", wxs_path.display(), msi_path.display());
            std::process::exit(1);
        }
    }
}

fn cmd_examples(example: &str) -> anyhow::Result<()> {
    let content = match example {
        "service" => TEMPLATE_SERVICE,
        "registry" => EXAMPLE_REGISTRY,
        "shortcuts" => EXAMPLE_SHORTCUTS,
        "bundle" => TEMPLATE_BUNDLE,
        "full" => EXAMPLE_FULL,
        _ => TEMPLATE_BASIC,
    };

    println!("# Example: {}", example);
    println!("{}", content);

    Ok(())
}

const TEMPLATE_MINIMAL: &str = r#"# Minimal wix-easy configuration
package:
  name: MyApp
  version: 1.0.0
  manufacturer: My Company

install:
  directory: ProgramFiles/MyCompany/MyApp
  files:
    - src: ./bin/*
"#;

const TEMPLATE_BASIC: &str = r#"# Basic wix-easy configuration
package:
  name: MyApp
  version: 1.0.0
  manufacturer: My Company
  description: My awesome application
  scope: per-machine  # or per-user

install:
  directory: ProgramFiles/MyCompany/MyApp
  files:
    - src: ./bin/*
    - src: ./config/default.json
      dest: config/

shortcuts:
  - name: MyApp
    target: MyApp.exe
    location: both  # desktop, startmenu, or both

ui:
  style: basic  # minimal, basic, full, or none
"#;

const TEMPLATE_SERVICE: &str = r#"# Windows Service configuration
package:
  name: MyService
  version: 1.0.0
  manufacturer: My Company
  description: My Windows Service

install:
  directory: ProgramFiles/MyCompany/MyService
  files:
    - src: ./bin/MyService.exe
    - src: ./config/*
      dest: config/

services:
  - name: MyService
    display_name: My Application Service
    executable: MyService.exe
    description: Runs background tasks for My Application
    start: auto  # auto, manual, disabled
    account: LocalSystem  # or LocalService, NetworkService

registry:
  - key: HKLM/Software/MyCompany/MyService
    values:
      LogLevel: "Info"
      ConfigPath: "[INSTALLFOLDER]config"
"#;

const TEMPLATE_BUNDLE: &str = r#"# Bundle with prerequisites
package:
  name: MyApp
  version: 1.0.0
  manufacturer: My Company

install:
  directory: ProgramFiles/MyCompany/MyApp
  files:
    - src: ./bin/*

prerequisites:
  - dotnet: "8.0"
  - vcredist: "2022"

shortcuts:
  - name: MyApp
    target: MyApp.exe
    location: startmenu
"#;

const EXAMPLE_REGISTRY: &str = r#"# Registry configuration example
package:
  name: MyApp
  version: 1.0.0
  manufacturer: My Company

install:
  directory: ProgramFiles/MyCompany/MyApp
  files:
    - src: ./bin/*

registry:
  # Per-user settings
  - key: HKCU/Software/MyCompany/MyApp
    values:
      Version: "1.0.0"
      InstallPath: "[INSTALLFOLDER]"
      LastRun: ""

  # Per-machine settings
  - key: HKLM/Software/MyCompany/MyApp
    values:
      ProductCode: "[ProductCode]"

  # File associations (advanced)
  - key: HKCR/.myapp
    values:
      "@": "MyApp.Document"

  - key: HKCR/MyApp.Document/shell/open/command
    values:
      "@": "\"[INSTALLFOLDER]MyApp.exe\" \"%1\""
"#;

const EXAMPLE_SHORTCUTS: &str = r#"# Shortcuts configuration example
package:
  name: MyApp
  version: 1.0.0
  manufacturer: My Company

install:
  directory: ProgramFiles/MyCompany/MyApp
  files:
    - src: ./bin/*

shortcuts:
  # Desktop shortcut
  - name: MyApp
    target: MyApp.exe
    location: desktop
    description: Launch MyApp

  # Start menu shortcuts
  - name: MyApp
    target: MyApp.exe
    location: startmenu
    icon: MyApp.exe

  - name: MyApp Help
    target: help.pdf
    location: startmenu
    description: Open help documentation

  - name: Uninstall MyApp
    target: uninstall.exe
    location: startmenu
    arguments: /uninstall
"#;

const EXAMPLE_FULL: &str = r#"# Full-featured wix-easy configuration
package:
  name: MyApp
  version: 2.0.0
  manufacturer: My Company Inc.
  description: A comprehensive desktop application
  icon: ./assets/app.ico
  license: ./LICENSE.rtf
  scope: per-machine
  architecture: x64

install:
  directory: ProgramFiles/MyCompany/MyApp
  files:
    - src: ./bin/*
    - src: ./lib/*.dll
      dest: lib/
    - src: ./config/default.json
      dest: config/
    - src: ./docs/*
      dest: docs/
  directories:
    - logs
    - data

features:
  - id: Core
    title: Core Application
    level: 1
    description: Required application files

  - id: Documentation
    title: Documentation
    level: 1
    description: User guides and help files

  - id: Tools
    title: Additional Tools
    level: 2
    description: Optional command-line utilities

shortcuts:
  - name: MyApp
    target: MyApp.exe
    location: both
    icon: MyApp.exe
    description: Launch MyApp

  - name: MyApp Documentation
    target: docs/index.html
    location: startmenu

registry:
  - key: HKCU/Software/MyCompany/MyApp
    values:
      Version: "2.0.0"
      InstallDate: "[Date]"
      InstallPath: "[INSTALLFOLDER]"

environment:
  - name: MYAPP_HOME
    value: "[INSTALLFOLDER]"
    action: set
    scope: user

  - name: PATH
    value: "[INSTALLFOLDER]bin"
    action: append
    scope: user

prerequisites:
  - dotnet: 8.0
  - vcredist: 2022

upgrade:
  allow_downgrade: false
  allow_same_version: true
  schedule: early

ui:
  style: full
  banner: ./assets/banner.png
  dialog: ./assets/dialog.png
  eula: ./LICENSE.rtf
"#;
