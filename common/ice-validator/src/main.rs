//! ICE Validator CLI
//!
//! Validate MSI files against ICE rules.

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use ice_validator::{rules, Severity, Validator};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "ice-validator")]
#[command(author, version, about = "Cross-platform ICE validator for MSI files")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Validate an MSI file
    Validate {
        /// Path to MSI file
        msi: PathBuf,

        /// Path to wixkb database (default: ~/.wixcraft/wixkb.db)
        #[arg(short, long)]
        db: Option<PathBuf>,

        /// Use only built-in rules (ignore wixkb)
        #[arg(long)]
        builtin: bool,

        /// Output format (text, json)
        #[arg(short, long, default_value = "text")]
        format: String,

        /// Treat warnings as errors
        #[arg(short = 'W', long)]
        warnings_as_errors: bool,

        /// Only show errors (hide warnings and info)
        #[arg(short, long)]
        errors_only: bool,
    },

    /// List available ICE rules
    Rules {
        /// Path to wixkb database
        #[arg(short, long)]
        db: Option<PathBuf>,

        /// Use only built-in rules
        #[arg(long)]
        builtin: bool,

        /// Output format (text, json)
        #[arg(short, long, default_value = "text")]
        format: String,
    },

    /// Show information about a specific ICE rule
    Info {
        /// Rule code (e.g., ICE03)
        code: String,

        /// Path to wixkb database
        #[arg(short, long)]
        db: Option<PathBuf>,
    },
}

fn main() -> Result<()> {
    env_logger::init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Validate {
            msi,
            db,
            builtin,
            format,
            warnings_as_errors,
            errors_only,
        } => cmd_validate(&msi, db, builtin, &format, warnings_as_errors, errors_only),
        Commands::Rules { db, builtin, format } => cmd_rules(db, builtin, &format),
        Commands::Info { code, db } => cmd_info(&code, db),
    }
}

fn cmd_validate(
    msi_path: &PathBuf,
    db_path: Option<PathBuf>,
    builtin: bool,
    format: &str,
    warnings_as_errors: bool,
    errors_only: bool,
) -> Result<()> {
    let validator = create_validator(db_path, builtin)?;

    let result = validator
        .validate(msi_path)
        .context("Failed to validate MSI")?;

    if format == "json" {
        println!("{}", serde_json::to_string_pretty(&result)?);
    } else {
        println!("Validating: {}", msi_path.display());
        println!();

        let mut shown = 0;
        for violation in &result.violations {
            if errors_only && violation.severity != Severity::Error {
                continue;
            }
            println!("{}", violation);
            shown += 1;
        }

        if shown > 0 {
            println!();
        }

        let (errors, warnings, infos) = result.count_by_severity();
        println!(
            "Result: {} errors, {} warnings, {} info",
            errors, warnings, infos
        );
        println!(
            "Checked {} rules in {}ms",
            result.rules_checked, result.duration_ms
        );

        // Exit code
        let has_errors = errors > 0 || (warnings_as_errors && warnings > 0);
        if has_errors {
            std::process::exit(1);
        }
    }

    Ok(())
}

fn cmd_rules(db_path: Option<PathBuf>, builtin: bool, format: &str) -> Result<()> {
    let rules = if builtin {
        rules::builtin_rules()
    } else {
        let db = db_path
            .or_else(rules::default_wixkb_path)
            .context("No wixkb database found")?;

        if db.exists() {
            rules::load_from_wixkb(&db)?
        } else {
            eprintln!("Warning: wixkb database not found, using built-in rules");
            rules::builtin_rules()
        }
    };

    if format == "json" {
        println!("{}", serde_json::to_string_pretty(&rules)?);
    } else {
        println!("Available ICE rules ({} total):\n", rules.len());
        for rule in &rules {
            println!("{}: {}", rule.code, rule.description);
        }
    }

    Ok(())
}

fn cmd_info(code: &str, db_path: Option<PathBuf>) -> Result<()> {
    let db = db_path
        .or_else(rules::default_wixkb_path)
        .context("No wixkb database found")?;

    let rules = if db.exists() {
        rules::load_from_wixkb(&db)?
    } else {
        rules::builtin_rules()
    };

    let rule = rules
        .iter()
        .find(|r| r.code.eq_ignore_ascii_case(code))
        .context(format!("Rule '{}' not found", code))?;

    println!("Rule: {}", rule.code);
    println!("Severity: {}", rule.severity);
    println!("Description: {}", rule.description);

    if let Some(ref resolution) = rule.resolution {
        println!("Resolution: {}", resolution);
    }

    if !rule.tables_affected.is_empty() {
        println!("Tables: {}", rule.tables_affected.join(", "));
    }

    if let Some(ref url) = rule.documentation_url {
        println!("Documentation: {}", url);
    }

    Ok(())
}

fn create_validator(db_path: Option<PathBuf>, builtin: bool) -> Result<Validator> {
    if builtin {
        return Ok(Validator::with_builtin_rules());
    }

    let db = db_path.or_else(rules::default_wixkb_path);

    match db {
        Some(path) if path.exists() => {
            Validator::from_wixkb(&path).context("Failed to load rules from wixkb")
        }
        _ => {
            eprintln!("Warning: wixkb database not found, using built-in rules");
            Ok(Validator::with_builtin_rules())
        }
    }
}
