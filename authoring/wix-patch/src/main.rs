//! wix-patch CLI - Simplified patch/MSP generation

use clap::{Parser, Subcommand, ValueEnum};
use std::fs;
use std::path::PathBuf;
use wix_patch::{Patch, PatchClassification, PatchFamily, PatchTemplates};

#[derive(Parser)]
#[command(name = "wix-patch")]
#[command(about = "Simplified patch/MSP generation for WiX installers")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a new patch definition
    New {
        /// Patch name
        name: String,

        /// Patch version
        #[arg(short, long, default_value = "1.0.1")]
        version: String,

        /// Target product code (GUID)
        #[arg(short, long)]
        target: String,

        /// Patch classification
        #[arg(short, long, value_enum, default_value = "update")]
        classification: Classification,

        /// Manufacturer name
        #[arg(short, long)]
        manufacturer: Option<String>,

        /// Description
        #[arg(short = 'D', long)]
        description: Option<String>,

        /// Output file (default: stdout)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Create a patch from a template
    Template {
        /// Template type
        #[arg(value_enum)]
        template: TemplateType,

        /// Patch name
        name: String,

        /// Patch version
        #[arg(short, long, default_value = "1.0.1")]
        version: String,

        /// Target product code
        #[arg(short, long)]
        target: String,

        /// Versions to supersede (comma-separated)
        #[arg(long)]
        supersede: Option<String>,

        /// Output file
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Generate build scripts for a patch
    Script {
        /// Patch WXS file
        file: PathBuf,

        /// Baseline MSI path
        #[arg(long)]
        baseline: String,

        /// Updated MSI path
        #[arg(long)]
        updated: String,

        /// Script format
        #[arg(short, long, value_enum, default_value = "powershell")]
        format: ScriptFormat,

        /// Output file
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Validate a patch definition
    Validate {
        /// Patch WXS file
        file: PathBuf,
    },

    /// Compare two MSIs and suggest patch content
    Compare {
        /// Baseline MSI
        baseline: PathBuf,

        /// Updated MSI
        updated: PathBuf,

        /// Output patch WXS
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
}

#[derive(Clone, Copy, ValueEnum)]
enum Classification {
    Critical,
    Hotfix,
    Security,
    Update,
    ServicePack,
    Upgrade,
}

impl From<Classification> for PatchClassification {
    fn from(c: Classification) -> Self {
        match c {
            Classification::Critical => PatchClassification::Critical,
            Classification::Hotfix => PatchClassification::Hotfix,
            Classification::Security => PatchClassification::Security,
            Classification::Update => PatchClassification::Update,
            Classification::ServicePack => PatchClassification::ServicePack,
            Classification::Upgrade => PatchClassification::Upgrade,
        }
    }
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum TemplateType {
    /// Simple hotfix patch
    Hotfix,
    /// Security update (non-removable)
    Security,
    /// Cumulative update with supersede
    Cumulative,
}

#[derive(Clone, Copy, ValueEnum)]
enum ScriptFormat {
    Powershell,
    Batch,
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::New {
            name,
            version,
            target,
            classification,
            manufacturer,
            description,
            output,
        } => {
            let mut patch = Patch::new(&name, &version)
                .classification(classification.into())
                .target_product_code(&target)
                .family(PatchFamily::new(format!("{}Family", name.replace(' ', ""))));

            if let Some(ref mfr) = manufacturer {
                patch = patch.manufacturer(mfr);
            }

            if let Some(ref desc) = description {
                patch = patch.description(desc);
            }

            let wxs = patch.generate();

            if let Some(path) = output {
                match fs::write(&path, &wxs) {
                    Ok(_) => {
                        println!("Created patch definition: {}", path.display());
                        println!("  Name: {}", name);
                        println!("  Version: {}", version);
                        println!("  Target: {}", target);
                        println!(
                            "  Classification: {}",
                            PatchClassification::from(classification).as_str()
                        );
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

        Commands::Template {
            template,
            name,
            version,
            target,
            supersede,
            output,
        } => {
            let patch = match template {
                TemplateType::Hotfix => PatchTemplates::hotfix(&name, &version, &target),
                TemplateType::Security => PatchTemplates::security_update(&name, &version, &target),
                TemplateType::Cumulative => {
                    let superseded: Vec<&str> = supersede
                        .as_deref()
                        .unwrap_or("")
                        .split(',')
                        .map(|s| s.trim())
                        .filter(|s| !s.is_empty())
                        .collect();

                    if superseded.is_empty() {
                        eprintln!("Cumulative template requires --supersede with versions");
                        eprintln!("Example: --supersede \"1.0.0,1.0.1,1.1.0\"");
                        std::process::exit(1);
                    }

                    PatchTemplates::cumulative_update(&name, &version, &target, superseded)
                }
            };

            let wxs = patch.generate();

            if let Some(path) = output {
                match fs::write(&path, &wxs) {
                    Ok(_) => {
                        println!("Created patch from template: {}", path.display());
                        println!("  Template: {:?}", template);
                        println!("  Name: {}", name);
                        println!("  Version: {}", version);
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

        Commands::Script {
            file,
            baseline,
            updated,
            format,
            output,
        } => {
            // Parse the patch file to get metadata
            let content = match fs::read_to_string(&file) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("Failed to read {}: {}", file.display(), e);
                    std::process::exit(1);
                }
            };

            // Extract patch name and version from content
            let name = extract_attr(&content, "DisplayName").unwrap_or_else(|| "Patch".to_string());
            let version = extract_version(&name).unwrap_or_else(|| "1.0.0".to_string());

            let patch = Patch::new(&name, &version)
                .baseline_msi(&baseline)
                .updated_msi(&updated)
                .target_product_code("{PLACEHOLDER}")
                .family(PatchFamily::new("Family"));

            let script = match format {
                ScriptFormat::Powershell => patch.generate_powershell_script(),
                ScriptFormat::Batch => patch.generate_build_script(),
            };

            if let Some(path) = output {
                match fs::write(&path, &script) {
                    Ok(_) => println!("Generated build script: {}", path.display()),
                    Err(e) => {
                        eprintln!("Failed to write output: {}", e);
                        std::process::exit(1);
                    }
                }
            } else {
                println!("{}", script);
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

            let mut errors = Vec::new();
            let mut warnings = Vec::new();

            // Check for required elements
            if !content.contains("<Patch") {
                errors.push("Missing <Patch> element");
            }

            if !content.contains("Classification=") {
                errors.push("Missing Classification attribute");
            }

            if !content.contains("<PatchFamily") {
                errors.push("Missing <PatchFamily> element");
            }

            if !content.contains("<TargetProductCode") && !content.contains("<TargetProductCodes") {
                errors.push("Missing target product specification");
            }

            // Check for common issues
            if content.contains("PUT-GUID-HERE") {
                warnings.push("Contains placeholder GUID that needs to be replaced");
            }

            if content.contains("Id=\"*\"") && !content.contains("UpgradeCode=") {
                warnings.push("Auto-generated patch ID without UpgradeCode may cause issues");
            }

            // Report results
            println!("Validating: {}", file.display());
            println!();

            if errors.is_empty() && warnings.is_empty() {
                println!("Validation passed - patch definition looks valid");
            } else {
                if !errors.is_empty() {
                    println!("Errors:");
                    for err in &errors {
                        println!("  x {}", err);
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

        Commands::Compare {
            baseline,
            updated,
            output,
        } => {
            // This would compare MSIs and suggest patch content
            // For now, we'll generate a template with placeholders

            println!("Comparing MSIs:");
            println!("  Baseline: {}", baseline.display());
            println!("  Updated:  {}", updated.display());
            println!();

            let patch = Patch::new("GeneratedPatch", "1.0.0")
                .baseline_msi(baseline.to_string_lossy())
                .updated_msi(updated.to_string_lossy())
                .target_product_code("{TARGET-PRODUCT-CODE}")
                .family(PatchFamily::new("GeneratedPatchFamily"))
                .description("Automatically generated patch");

            let wxs = patch.generate();

            if let Some(path) = output {
                match fs::write(&path, &wxs) {
                    Ok(_) => {
                        println!("Generated patch template: {}", path.display());
                        println!();
                        println!("Next steps:");
                        println!("  1. Replace {{TARGET-PRODUCT-CODE}} with actual product GUID");
                        println!("  2. Update patch name and version");
                        println!("  3. Add component references for changed files");
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
    }
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

fn extract_version(name: &str) -> Option<String> {
    // Try to extract version from name like "MyPatch 1.0.1"
    let parts: Vec<&str> = name.split_whitespace().collect();
    for part in parts.iter().rev() {
        if part.chars().next().map(|c| c.is_ascii_digit()).unwrap_or(false)
            && part.contains('.')
        {
            return Some(part.to_string());
        }
    }
    None
}
