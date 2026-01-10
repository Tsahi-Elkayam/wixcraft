//! wix-migrate CLI - Migration assistant for WiX versions

use clap::{Parser, Subcommand, ValueEnum};
use std::fs;
use std::path::PathBuf;
use wix_migrate::{breaking_changes, detect_version, Migrator, WixVersion};

#[derive(Parser)]
#[command(name = "wix-migrate")]
#[command(about = "Migration assistant for WiX v3 to v4 to v5")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Migrate a WXS file to a newer version
    Migrate {
        /// Input WXS file
        input: PathBuf,

        /// Target WiX version
        #[arg(short, long, value_enum)]
        to: Version,

        /// Output file (default: overwrite input)
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Don't write changes, just show what would change
        #[arg(long)]
        dry_run: bool,

        /// Output as JSON report
        #[arg(long)]
        json: bool,
    },

    /// Detect WiX version of a file
    Detect {
        /// Input WXS file
        input: PathBuf,
    },

    /// Show breaking changes between versions
    Breaking {
        /// Source version
        #[arg(value_enum)]
        from: Version,

        /// Target version
        #[arg(value_enum)]
        to: Version,
    },

    /// Check a file for migration issues
    Check {
        /// Input WXS file
        input: PathBuf,

        /// Target version to check against
        #[arg(short, long, value_enum)]
        to: Version,
    },
}

#[derive(Clone, Copy, ValueEnum)]
enum Version {
    V3,
    V4,
    V5,
}

impl From<Version> for WixVersion {
    fn from(v: Version) -> Self {
        match v {
            Version::V3 => WixVersion::V3,
            Version::V4 => WixVersion::V4,
            Version::V5 => WixVersion::V5,
        }
    }
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Migrate {
            input,
            to,
            output,
            dry_run,
            json,
        } => {
            let content = match fs::read_to_string(&input) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("Failed to read {}: {}", input.display(), e);
                    std::process::exit(1);
                }
            };

            let target_version: WixVersion = to.into();

            let result = match Migrator::auto_migrate(&content, target_version) {
                Ok(r) => r,
                Err(e) => {
                    eprintln!("Migration error: {}", e);
                    std::process::exit(1);
                }
            };

            if json {
                println!("{}", serde_json::to_string_pretty(&result).unwrap());
                return;
            }

            println!("Migration: {} -> {}", result.from_version, result.to_version);
            println!("===================================================");
            println!();

            if result.changes.is_empty() {
                println!("No changes needed.");
            } else {
                println!(
                    "Changes: {} total, {} breaking",
                    result.change_count(),
                    result.breaking_count()
                );
                println!();

                for change in &result.changes {
                    let breaking_marker = if change.breaking { " [BREAKING]" } else { "" };
                    let line_info = change
                        .line
                        .map(|l| format!("Line {}: ", l))
                        .unwrap_or_default();

                    println!("  {}{}{}", line_info, change.description, breaking_marker);
                    println!("    - {}", truncate(&change.original, 60));
                    println!("    + {}", truncate(&change.replacement, 60));
                    println!();
                }
            }

            if !result.warnings.is_empty() {
                println!("Warnings:");
                for warning in &result.warnings {
                    println!("  ! {}", warning);
                }
                println!();
            }

            if dry_run {
                println!("Dry run - no files modified.");
            } else {
                let output_path = output.unwrap_or(input);
                match fs::write(&output_path, &result.new_content) {
                    Ok(_) => {
                        println!("Migrated file written to: {}", output_path.display());
                    }
                    Err(e) => {
                        eprintln!("Failed to write output: {}", e);
                        std::process::exit(1);
                    }
                }
            }
        }

        Commands::Detect { input } => {
            let content = match fs::read_to_string(&input) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("Failed to read {}: {}", input.display(), e);
                    std::process::exit(1);
                }
            };

            match detect_version(&content) {
                Some(version) => {
                    println!("{}: WiX {}", input.display(), version.as_str());
                }
                None => {
                    println!("{}: Unknown WiX version", input.display());
                    std::process::exit(1);
                }
            }
        }

        Commands::Breaking { from, to } => {
            let from_version: WixVersion = from.into();
            let to_version: WixVersion = to.into();

            let changes = breaking_changes(from_version, to_version);

            if changes.is_empty() {
                println!(
                    "No breaking changes between {} and {}",
                    from_version.as_str(),
                    to_version.as_str()
                );
            } else {
                println!(
                    "Breaking changes from {} to {}:",
                    from_version.as_str(),
                    to_version.as_str()
                );
                println!();
                for change in changes {
                    println!("  - {}", change);
                }
            }
        }

        Commands::Check { input, to } => {
            let content = match fs::read_to_string(&input) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("Failed to read {}: {}", input.display(), e);
                    std::process::exit(1);
                }
            };

            let target_version: WixVersion = to.into();

            match Migrator::auto_migrate(&content, target_version) {
                Ok(result) => {
                    if result.changes.is_empty() && result.warnings.is_empty() {
                        println!("[OK] {} is ready for {}", input.display(), result.to_version);
                    } else {
                        println!(
                            "[NEEDS MIGRATION] {} needs {} changes for {}",
                            input.display(),
                            result.change_count(),
                            result.to_version
                        );

                        if result.has_breaking_changes {
                            println!(
                                "  ({} breaking changes require manual review)",
                                result.breaking_count()
                            );
                        }

                        if !result.warnings.is_empty() {
                            println!("  ({} warnings)", result.warnings.len());
                        }

                        std::process::exit(1);
                    }
                }
                Err(e) => {
                    eprintln!("Check failed: {}", e);
                    std::process::exit(1);
                }
            }
        }
    }
}

fn truncate(s: &str, max: usize) -> String {
    let s = s.trim();
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}...", &s[..max - 3])
    }
}
