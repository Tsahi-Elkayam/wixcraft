//! wix-analytics CLI - Installation telemetry generator
//!
//! Usage:
//!   wix-analytics generate --endpoint https://... --product myapp
//!   wix-analytics parse install.log

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use wix_analytics::*;

#[derive(Parser)]
#[command(name = "wix-analytics")]
#[command(about = "Installation telemetry and analytics for MSI packages")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate WiX analytics code
    Generate {
        /// Analytics endpoint URL
        #[arg(short, long)]
        endpoint: String,

        /// Product ID
        #[arg(short, long)]
        product: String,

        /// Require opt-in (default: true)
        #[arg(long, default_value = "true")]
        opt_in: bool,

        /// Output file
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Generate only specific component (properties, actions, sequence, optin, all)
        #[arg(long, default_value = "all")]
        component: String,
    },

    /// Parse MSI log file for telemetry data
    Parse {
        /// Log file to parse
        log_file: PathBuf,

        /// Output format (text, json)
        #[arg(short, long, default_value = "text")]
        format: String,
    },

    /// Show available telemetry fields and events
    Info,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Generate {
            endpoint,
            product,
            opt_in,
            output,
            component,
        } => {
            let config = AnalyticsConfig {
                endpoint_url: endpoint,
                product_id: product,
                opt_in,
                ..Default::default()
            };

            let gen = AnalyticsGenerator::new(config);

            let content = match component.as_str() {
                "properties" => gen.generate_properties(),
                "actions" => gen.generate_custom_actions(),
                "sequence" => gen.generate_sequence(),
                "optin" => gen.generate_opt_in_control(),
                _ => gen.generate_fragment(),
            };

            if let Some(path) = output {
                std::fs::write(&path, &content)?;
                println!("Written to: {}", path.display());
            } else {
                println!("{}", content);
            }
        }

        Commands::Parse { log_file, format } => {
            if !log_file.exists() {
                return Err(anyhow::anyhow!("Log file not found: {}", log_file.display()));
            }

            let content = std::fs::read_to_string(&log_file)?;
            let event = parse_msi_log(&content);

            match format.as_str() {
                "json" => {
                    println!("{}", serde_json::to_string_pretty(&event)?);
                }
                _ => {
                    println!("Telemetry Data from: {}", log_file.display());
                    println!("{}", "=".repeat(50));
                    println!("Event Type: {}", event.event_type);
                    println!("Timestamp: {}", event.timestamp);
                    if let Some(ref ver) = event.product_version {
                        println!("Product Version: {}", ver);
                    }
                    if let Some(code) = event.error_code {
                        println!("Error Code: {}", code);
                    }
                    if let Some(ms) = event.duration_ms {
                        println!("Duration: {} ms", ms);
                    }
                }
            }
        }

        Commands::Info => {
            println!("Installation Analytics");
            println!("{}", "=".repeat(50));
            println!();

            println!("Event Types:");
            println!("  install_start     - Installation began");
            println!("  install_success   - Installation completed successfully");
            println!("  install_failure   - Installation failed");
            println!("  uninstall_start   - Uninstallation began");
            println!("  uninstall_success - Uninstallation completed");
            println!("  uninstall_failure - Uninstallation failed");
            println!("  repair_start      - Repair operation began");
            println!("  repair_success    - Repair completed");
            println!("  repair_failure    - Repair failed");
            println!("  feature_change    - Feature selection changed");
            println!();

            println!("Telemetry Fields:");
            println!("  product_version - Product version being installed");
            println!("  os_version      - Windows version");
            println!("  os_arch         - 32-bit or 64-bit");
            println!("  install_path    - Installation directory");
            println!("  install_mode    - Per-user or per-machine");
            println!("  features        - Selected features");
            println!("  duration        - Installation time in milliseconds");
            println!("  error_code      - MSI error code if failed");
            println!("  locale          - System locale");
            println!("  timezone        - System timezone");
            println!();

            println!("Privacy Considerations:");
            println!("  - Use opt-in by default for GDPR compliance");
            println!("  - Avoid collecting personally identifiable information");
            println!("  - Hash any machine identifiers");
            println!("  - Document data collection in privacy policy");
        }
    }

    Ok(())
}
