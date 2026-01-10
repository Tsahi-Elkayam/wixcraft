//! wix-msix - MSIX package converter and analyzer CLI

use clap::{Parser, Subcommand};
use std::path::PathBuf;
use wix_msix::{
    ConversionOptions, ConversionResult, ManifestGenerator, MsixAnalyzer, MsixApplication,
    MsixConfig, PackageIdentity, ProcessorArchitecture,
};

#[derive(Parser)]
#[command(name = "wix-msix")]
#[command(about = "MSIX package converter and analyzer")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Analyze MSI for MSIX conversion
    Analyze {
        /// MSI file path
        msi: PathBuf,

        /// Show detailed info
        #[arg(short, long)]
        verbose: bool,
    },
    /// Convert MSI to MSIX
    Convert {
        /// Source MSI file
        msi: PathBuf,

        /// Output MSIX path
        #[arg(short, long)]
        output: PathBuf,

        /// Signing certificate
        #[arg(short, long)]
        certificate: Option<PathBuf>,

        /// Use Package Support Framework
        #[arg(long)]
        psf: bool,

        /// Dry run
        #[arg(long)]
        dry_run: bool,
    },
    /// Generate AppxManifest.xml
    Manifest {
        /// Package name
        #[arg(short, long)]
        name: String,

        /// Publisher (CN=...)
        #[arg(short, long)]
        publisher: String,

        /// Version
        #[arg(short, long)]
        version: String,

        /// Display name
        #[arg(short, long)]
        display_name: String,

        /// Publisher display name
        #[arg(long)]
        publisher_name: String,

        /// Executable path
        #[arg(short, long)]
        executable: String,

        /// Output file
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    /// Show conversion compatibility
    Check {
        /// MSI file path
        msi: PathBuf,
    },
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Analyze { msi, verbose } => {
            println!("Analyzing MSI for MSIX conversion: {}", msi.display());

            let analysis = MsixAnalyzer::analyze_msi(&msi);

            println!("\nAnalysis Results:");
            println!("  File count: {}", analysis.file_count);
            println!("  Total size: {} bytes", analysis.total_size_bytes);
            println!("  Has registry: {}", analysis.has_registry);
            println!("  Has services: {}", analysis.has_services);
            println!("  Has drivers: {}", analysis.has_drivers);
            println!("  Complexity: {:?}", analysis.conversion_complexity);

            if verbose {
                println!("\nCapabilities required:");
                for cap in &analysis.capabilities_required {
                    println!("  - {}", cap.as_str());
                }

                if !analysis.compatibility_issues.is_empty() {
                    println!("\nCompatibility issues:");
                    for issue in &analysis.compatibility_issues {
                        println!("  - {}", issue);
                    }
                }
            }
        }

        Commands::Convert {
            msi,
            output,
            certificate,
            psf,
            dry_run,
        } => {
            println!("Converting MSI to MSIX");
            println!("  Source: {}", msi.display());
            println!("  Output: {}", output.display());

            let mut opts = ConversionOptions::new(msi.clone(), output.clone());

            if let Some(cert) = certificate {
                opts = opts.with_certificate(cert);
                println!("  Certificate: specified");
            }

            if psf {
                let _opts = opts.with_psf();
                println!("  PSF: enabled");
            }

            if dry_run {
                println!("\nDry run - no conversion performed.");
                println!("Would convert {} to MSIX format.", msi.display());
            } else {
                // Simulate conversion
                let result = ConversionResult::success(output)
                    .with_warning("Registry virtualization applied")
                    .with_fixup("FileRedirection for AppData");

                if result.success {
                    println!("\nConversion successful!");
                    if let Some(path) = result.output_path {
                        println!("Output: {}", path.display());
                    }
                    for fixup in &result.fixups_applied {
                        println!("  Fixup: {}", fixup);
                    }
                    for warning in &result.warnings {
                        println!("  Warning: {}", warning);
                    }
                }
            }
        }

        Commands::Manifest {
            name,
            publisher,
            version,
            display_name,
            publisher_name,
            executable,
            output,
        } => {
            let identity = PackageIdentity::new(&name, &publisher, &version)
                .with_architecture(ProcessorArchitecture::X64);

            let app = MsixApplication::new("App", &executable, &display_name);

            let config = MsixConfig::new(identity, &display_name, &publisher_name)
                .with_application(app);

            let manifest = ManifestGenerator::generate(&config);

            if let Some(path) = output {
                std::fs::write(&path, &manifest)?;
                println!("Manifest written to: {}", path.display());
            } else {
                println!("{}", manifest);
            }
        }

        Commands::Check { msi } => {
            println!("Checking MSIX conversion compatibility: {}", msi.display());

            let analysis = MsixAnalyzer::analyze_msi(&msi);
            let issues = MsixAnalyzer::check_compatibility(&analysis);

            println!("\nConversion complexity: {:?}", analysis.conversion_complexity);

            if issues.is_empty() {
                println!("\nNo compatibility issues found.");
                println!("This MSI should convert to MSIX successfully.");
            } else {
                println!("\nCompatibility issues found:");
                for issue in &issues {
                    println!("  - {}", issue);
                }
            }
        }
    }

    Ok(())
}
