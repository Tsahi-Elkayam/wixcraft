//! wix-bundle CLI - Burn bootstrapper wizard

use clap::{Parser, Subcommand, ValueEnum};
use std::fs;
use std::path::PathBuf;
use wix_bundle::{BootstrapperUI, Bundle, BundlePackage, BundleTemplates};

#[derive(Parser)]
#[command(name = "wix-bundle")]
#[command(about = "Burn bootstrapper wizard for WiX installers")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a new bundle from scratch
    New {
        /// Bundle name
        name: String,

        /// Bundle version
        #[arg(short, long, default_value = "1.0.0")]
        version: String,

        /// Manufacturer name
        #[arg(short, long)]
        manufacturer: Option<String>,

        /// Output file
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// UI theme
        #[arg(long, value_enum, default_value = "hyperlink-license")]
        ui: UiTheme,
    },

    /// Add a package to a bundle
    Add {
        /// Existing bundle WXS file
        bundle: PathBuf,

        /// Package type
        #[arg(value_enum)]
        package_type: PkgType,

        /// Package source file
        source: String,

        /// Package ID (auto-generated if not specified)
        #[arg(long)]
        id: Option<String>,

        /// Install condition
        #[arg(long)]
        install_condition: Option<String>,

        /// Detect condition
        #[arg(long)]
        detect_condition: Option<String>,

        /// Install arguments (for EXE packages)
        #[arg(long)]
        install_args: Option<String>,
    },

    /// Create a bundle from a template
    Template {
        /// Template name
        #[arg(value_enum)]
        template: TemplateName,

        /// Bundle name
        name: String,

        /// Bundle version
        #[arg(short, long, default_value = "1.0.0")]
        version: String,

        /// Main MSI file
        #[arg(short, long)]
        msi: String,

        /// .NET version (for dotnet template)
        #[arg(long, default_value = "8.0")]
        dotnet: String,

        /// Output file
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Show package chain for an existing bundle
    Info {
        /// Bundle WXS file
        file: PathBuf,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Validate a bundle configuration
    Validate {
        /// Bundle WXS file
        file: PathBuf,
    },
}

#[derive(Clone, Copy, ValueEnum)]
enum UiTheme {
    HyperlinkLicense,
    HyperlinkSideLicense,
    HyperlinkLargeLicense,
    RtfLicense,
    RtfSideLicense,
    RtfLargeLicense,
    None,
}

impl From<UiTheme> for BootstrapperUI {
    fn from(t: UiTheme) -> Self {
        match t {
            UiTheme::HyperlinkLicense => BootstrapperUI::HyperlinkLicense,
            UiTheme::HyperlinkSideLicense => BootstrapperUI::HyperlinkSideLicense,
            UiTheme::HyperlinkLargeLicense => BootstrapperUI::HyperlinkLargeLicense,
            UiTheme::RtfLicense => BootstrapperUI::RtfLicense,
            UiTheme::RtfSideLicense => BootstrapperUI::RtfSideLicense,
            UiTheme::RtfLargeLicense => BootstrapperUI::RtfLargeLicense,
            UiTheme::None => BootstrapperUI::None,
        }
    }
}

#[derive(Clone, Copy, ValueEnum)]
enum PkgType {
    Msi,
    Exe,
    Msp,
    Msu,
    Bundle,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum TemplateName {
    /// Simple bundle with one MSI
    Simple,
    /// Bundle with .NET prerequisite
    Dotnet,
    /// Bundle with VC++ redistributable
    Vcredist,
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::New {
            name,
            version,
            manufacturer,
            output,
            ui,
        } => {
            let mut bundle = Bundle::new(&name, &version).ui(ui.into());

            if let Some(ref mfr) = manufacturer {
                bundle = bundle.manufacturer(mfr);
            }

            // Add a placeholder package
            bundle = bundle.package(
                BundlePackage::msi("$(var.ProductMsi)")
                    .id("MainProduct")
                    .name(&name),
            );

            let wxs = bundle.generate();

            if let Some(path) = output {
                match fs::write(&path, &wxs) {
                    Ok(_) => {
                        println!("Created bundle: {}", path.display());
                        println!("  Name: {}", name);
                        println!("  Version: {}", version);
                        if let Some(ref mfr) = manufacturer {
                            println!("  Manufacturer: {}", mfr);
                        }
                        println!();
                        println!("Next steps:");
                        println!("  1. Replace PUT-GUID-HERE with a new GUID");
                        println!("  2. Update $(var.ProductMsi) with your MSI path");
                        println!("  3. Add any prerequisite packages");
                    }
                    Err(e) => {
                        eprintln!("Failed to write output: {}", e);
                        std::process::exit(1);
                    }
                }
            } else {
                println!("{}", wxs);
            }
        }

        Commands::Add {
            bundle: _,
            package_type,
            source,
            id,
            install_condition,
            detect_condition,
            install_args,
        } => {
            // Create the package element to add
            let mut pkg = match package_type {
                PkgType::Msi => BundlePackage::msi(&source),
                PkgType::Exe => BundlePackage::exe(&source),
                PkgType::Msp => BundlePackage::msp(&source),
                PkgType::Msu => BundlePackage::msu(&source),
                PkgType::Bundle => BundlePackage::bundle(&source),
            };

            if let Some(id) = id {
                pkg = pkg.id(id);
            }

            if let Some(cond) = install_condition {
                pkg = pkg.install_condition(cond);
            }

            if let Some(cond) = detect_condition {
                pkg = pkg.detect_condition(cond);
            }

            if let Some(args) = install_args {
                pkg = pkg.install_args(args);
            }

            // Generate package XML snippet
            let bundle = Bundle::new("temp", "1.0.0").package(pkg);
            let full_wxs = bundle.generate();

            // Extract just the package element
            if let Some(start) = full_wxs.find("<MsiPackage")
                .or_else(|| full_wxs.find("<ExePackage"))
                .or_else(|| full_wxs.find("<MspPackage"))
                .or_else(|| full_wxs.find("<MsuPackage"))
                .or_else(|| full_wxs.find("<BundlePackage"))
            {
                if let Some(end) = full_wxs[start..].find("/>") {
                    let snippet = &full_wxs[start..start + end + 2];
                    println!("Add this package to your bundle's <Chain>:\n");
                    println!("{}", snippet);
                    println!();
                    println!("(Manual insertion required - future version will auto-insert)");
                }
            }
        }

        Commands::Template {
            template,
            name,
            version,
            msi,
            dotnet,
            output,
        } => {
            let bundle = match template {
                TemplateName::Simple => BundleTemplates::simple_setup(&name, &version, &msi),
                TemplateName::Dotnet => BundleTemplates::with_dotnet(&name, &version, &msi, &dotnet),
                TemplateName::Vcredist => BundleTemplates::with_vcredist(&name, &version, &msi),
            };

            let wxs = bundle.generate();

            if let Some(path) = output {
                match fs::write(&path, &wxs) {
                    Ok(_) => {
                        println!("Created bundle from template: {}", path.display());
                        println!("  Template: {:?}", template);
                        println!("  Name: {}", name);
                        println!("  Version: {}", version);
                        println!("  MSI: {}", msi);
                        println!("  Packages: {}", bundle.packages.len());
                    }
                    Err(e) => {
                        eprintln!("Failed to write output: {}", e);
                        std::process::exit(1);
                    }
                }
            } else {
                println!("{}", wxs);
            }
        }

        Commands::Info { file, json } => {
            let content = match fs::read_to_string(&file) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("Failed to read {}: {}", file.display(), e);
                    std::process::exit(1);
                }
            };

            // Parse basic info from the WXS
            let info = parse_bundle_info(&content);

            if json {
                println!("{}", serde_json::to_string_pretty(&info).unwrap());
            } else {
                println!("Bundle Information: {}", file.display());
                println!("{}", "=".repeat(50));
                println!();
                println!("Name:         {}", info.name.as_deref().unwrap_or("N/A"));
                println!("Version:      {}", info.version.as_deref().unwrap_or("N/A"));
                println!("Manufacturer: {}", info.manufacturer.as_deref().unwrap_or("N/A"));
                println!("UpgradeCode:  {}", info.upgrade_code.as_deref().unwrap_or("N/A"));
                println!();

                if !info.packages.is_empty() {
                    println!("Packages ({}):", info.packages.len());
                    for pkg in &info.packages {
                        println!("  {} - {} ({})", pkg.id, pkg.source, pkg.package_type);
                    }
                }
            }
        }

        Commands::Validate { file } => {
            let content = match fs::read_to_string(&file) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("Failed to read {}: {}", file.display(), e);
                    std::process::exit(1);
                }
            };

            let info = parse_bundle_info(&content);
            let mut errors = Vec::new();
            let mut warnings = Vec::new();

            // Check required fields
            if info.name.is_none() {
                errors.push("Missing bundle Name attribute");
            }

            if info.version.is_none() {
                errors.push("Missing bundle Version attribute");
            }

            if info.upgrade_code.as_deref() == Some("PUT-GUID-HERE") {
                errors.push("UpgradeCode is still set to placeholder");
            }

            if info.upgrade_code.is_none() {
                errors.push("Missing bundle UpgradeCode attribute");
            }

            if info.packages.is_empty() {
                errors.push("Bundle has no packages in the chain");
            }

            // Check packages
            for pkg in &info.packages {
                if pkg.source.starts_with("$(var.") && !pkg.source.ends_with(")") {
                    warnings.push(format!("Package {} has malformed variable: {}", pkg.id, pkg.source));
                }
            }

            // Report results
            if errors.is_empty() && warnings.is_empty() {
                println!("Bundle validation passed");
            } else {
                if !errors.is_empty() {
                    println!("Errors:");
                    for err in &errors {
                        println!("  {} {}", "x", err);
                    }
                }

                if !warnings.is_empty() {
                    println!("Warnings:");
                    for warn in &warnings {
                        println!("  ! {}", warn);
                    }
                }

                if !errors.is_empty() {
                    std::process::exit(1);
                }
            }
        }
    }
}

#[derive(Debug, serde::Serialize)]
struct BundleInfo {
    name: Option<String>,
    version: Option<String>,
    manufacturer: Option<String>,
    upgrade_code: Option<String>,
    packages: Vec<PackageInfo>,
}

#[derive(Debug, serde::Serialize)]
struct PackageInfo {
    id: String,
    package_type: String,
    source: String,
}

fn parse_bundle_info(content: &str) -> BundleInfo {
    let mut info = BundleInfo {
        name: None,
        version: None,
        manufacturer: None,
        upgrade_code: None,
        packages: Vec::new(),
    };

    // Simple regex-based parsing (not a full XML parser)
    if let Some(name) = extract_attr(content, "Name") {
        info.name = Some(name);
    }

    if let Some(version) = extract_attr(content, "Version") {
        info.version = Some(version);
    }

    if let Some(manufacturer) = extract_attr(content, "Manufacturer") {
        info.manufacturer = Some(manufacturer);
    }

    if let Some(upgrade_code) = extract_attr(content, "UpgradeCode") {
        info.upgrade_code = Some(upgrade_code);
    }

    // Find packages
    let package_types = ["MsiPackage", "ExePackage", "MspPackage", "MsuPackage", "BundlePackage"];

    for pkg_type in package_types {
        let pattern = format!("<{}", pkg_type);
        for (idx, _) in content.match_indices(&pattern) {
            if let Some(end) = content[idx..].find("/>").or_else(|| content[idx..].find('>')) {
                let element = &content[idx..idx + end + 2];

                let id = extract_attr(element, "Id").unwrap_or_else(|| "unknown".to_string());
                let source = extract_attr(element, "SourceFile").unwrap_or_else(|| "unknown".to_string());

                info.packages.push(PackageInfo {
                    id,
                    package_type: pkg_type.to_string(),
                    source,
                });
            }
        }
    }

    info
}

fn extract_attr(content: &str, attr: &str) -> Option<String> {
    let pattern = format!("{}=\"", attr);
    if let Some(start) = content.find(&pattern) {
        let value_start = start + pattern.len();
        if let Some(end) = content[value_start..].find('"') {
            return Some(content[value_start..value_start + end].to_string());
        }
    }
    None
}
