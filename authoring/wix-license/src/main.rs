//! wix-license CLI - License key validation wizard
//!
//! Usage:
//!   wix-license generate --format microsoft    # Generate sample key
//!   wix-license wix --format microsoft         # Generate WiX code
//!   wix-license validate XXXXX-XXXXX-...       # Validate a key

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use wix_license::*;

#[derive(Parser)]
#[command(name = "wix-license")]
#[command(about = "License key validation wizard for WiX installers")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate sample license keys
    Generate {
        /// Key format (microsoft, short, guid)
        #[arg(short, long, default_value = "microsoft")]
        format: String,

        /// Number of keys to generate
        #[arg(short, long, default_value = "1")]
        count: usize,
    },

    /// Generate WiX code for license validation
    Wix {
        /// Key format (microsoft, short, guid)
        #[arg(short, long, default_value = "microsoft")]
        format: String,

        /// Validation type (format, checksum, online, dll)
        #[arg(short, long, default_value = "format")]
        validation: String,

        /// Property name for the license key
        #[arg(short, long, default_value = "LICENSEKEY")]
        property: String,

        /// Output file (default: stdout)
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Generate only specific component (dialog, properties, action, all)
        #[arg(long, default_value = "all")]
        component: String,
    },

    /// Validate a license key
    Validate {
        /// License key to validate
        key: String,

        /// Expected format (microsoft, short, guid)
        #[arg(short, long, default_value = "microsoft")]
        format: String,

        /// Use checksum validation
        #[arg(long)]
        checksum: bool,
    },

    /// Show available formats and patterns
    Formats,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Generate { format, count } => {
            let fmt = parse_format(&format)?;

            for _ in 0..count {
                let key = LicenseGenerator::generate_sample(fmt);
                println!("{}", key);
            }
        }

        Commands::Wix {
            format,
            validation,
            property,
            output,
            component,
        } => {
            let fmt = parse_format(&format)?;
            let val = parse_validation(&validation)?;

            let config = LicenseConfig {
                format: fmt,
                pattern: None,
                validation: val,
                property_name: property,
                ..Default::default()
            };

            let gen = WixLicenseGenerator::new(config);

            let content = match component.as_str() {
                "dialog" => gen.generate_dialog(),
                "properties" => gen.generate_properties(),
                "action" => gen.generate_custom_action(),
                _ => gen.generate_fragment(),
            };

            if let Some(path) = output {
                std::fs::write(&path, &content)?;
                println!("Written to: {}", path.display());
            } else {
                println!("{}", content);
            }
        }

        Commands::Validate {
            key,
            format,
            checksum,
        } => {
            let fmt = parse_format(&format)?;
            let pattern = LicenseGenerator::get_pattern(fmt);
            let regex = regex_lite::Regex::new(&pattern).unwrap();

            println!("Key: {}", key);
            println!("Format: {}", fmt);
            println!();

            let format_valid = regex.is_match(&key.to_uppercase());
            println!(
                "Format check: {}",
                if format_valid { "PASS" } else { "FAIL" }
            );

            if checksum {
                let checksum_valid = LicenseGenerator::validate_checksum(&key);
                println!(
                    "Checksum check: {}",
                    if checksum_valid { "PASS" } else { "FAIL" }
                );

                if !format_valid || !checksum_valid {
                    std::process::exit(1);
                }
            } else if !format_valid {
                std::process::exit(1);
            }
        }

        Commands::Formats => {
            println!("Available License Key Formats");
            println!("{}", "=".repeat(60));
            println!();

            println!("microsoft - {}", LicenseFormat::Microsoft);
            println!("  Pattern: {}", LicenseGenerator::get_pattern(LicenseFormat::Microsoft));
            println!("  Example: {}", LicenseGenerator::generate_sample(LicenseFormat::Microsoft));
            println!();

            println!("short - {}", LicenseFormat::Short);
            println!("  Pattern: {}", LicenseGenerator::get_pattern(LicenseFormat::Short));
            println!("  Example: {}", LicenseGenerator::generate_sample(LicenseFormat::Short));
            println!();

            println!("guid - {}", LicenseFormat::Guid);
            println!("  Pattern: {}", LicenseGenerator::get_pattern(LicenseFormat::Guid));
            println!("  Example: {}", LicenseGenerator::generate_sample(LicenseFormat::Guid));
            println!();

            println!("Validation Types:");
            println!("  format   - Regex pattern matching only");
            println!("  checksum - Built-in Luhn-like checksum validation");
            println!("  online   - Server-side validation (requires implementation)");
            println!("  dll      - Custom DLL validation (requires implementation)");
        }
    }

    Ok(())
}

fn parse_format(s: &str) -> Result<LicenseFormat> {
    match s.to_lowercase().as_str() {
        "microsoft" | "ms" => Ok(LicenseFormat::Microsoft),
        "short" => Ok(LicenseFormat::Short),
        "guid" => Ok(LicenseFormat::Guid),
        "custom" => Ok(LicenseFormat::Custom),
        _ => Err(anyhow::anyhow!(
            "Unknown format: {}. Use: microsoft, short, guid",
            s
        )),
    }
}

fn parse_validation(s: &str) -> Result<ValidationType> {
    match s.to_lowercase().as_str() {
        "format" | "regex" => Ok(ValidationType::FormatOnly),
        "checksum" | "check" => Ok(ValidationType::Checksum),
        "online" | "server" => Ok(ValidationType::Online),
        "dll" | "custom" => Ok(ValidationType::CustomDll),
        _ => Err(anyhow::anyhow!(
            "Unknown validation: {}. Use: format, checksum, online, dll",
            s
        )),
    }
}

mod regex_lite {
    // Minimal regex implementation for format validation
    pub struct Regex {
        pattern: String,
    }

    impl Regex {
        pub fn new(pattern: &str) -> Result<Self, &'static str> {
            Ok(Self {
                pattern: pattern.to_string(),
            })
        }

        pub fn is_match(&self, text: &str) -> bool {
            // Simplified matching for common license patterns
            let text = text.trim();

            if self.pattern.contains("{5}-") {
                // Microsoft format: XXXXX-XXXXX-XXXXX-XXXXX
                let parts: Vec<&str> = text.split('-').collect();
                parts.len() == 4 && parts.iter().all(|p| p.len() == 5 && p.chars().all(|c| c.is_alphanumeric()))
            } else if self.pattern.contains("{4}-") && !self.pattern.contains("{8}") {
                // Short format: XXXX-XXXX-XXXX-XXXX
                let parts: Vec<&str> = text.split('-').collect();
                parts.len() == 4 && parts.iter().all(|p| p.len() == 4 && p.chars().all(|c| c.is_alphanumeric()))
            } else if self.pattern.contains("{8}-") {
                // GUID format
                let parts: Vec<&str> = text.split('-').collect();
                parts.len() == 5
                    && parts[0].len() == 8
                    && parts[1].len() == 4
                    && parts[2].len() == 4
                    && parts[3].len() == 4
                    && parts[4].len() == 12
                    && parts.iter().all(|p| p.chars().all(|c| c.is_ascii_hexdigit()))
            } else {
                true // Custom pattern - always match
            }
        }
    }
}
