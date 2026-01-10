//! database - WiX Knowledge Base maintenance tool
//!
//! Usage:
//!     database init [--path <path>]      Create new database
//!     database stats                     Show statistics
//!     database vacuum                    Optimize database
//!     database export <file>             Export to JSON
//!     database import <file>             Import from JSON
//!     database backup <file>             Create backup
//!     database reset                     Reset to empty

use clap::{Parser, Subcommand};
use rusqlite::backup::Backup;
use wixkb::db::Database;
use wixkb::{Result, WixKbError, default_db_path};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "database")]
#[command(about = "WiX Knowledge Base maintenance")]
#[command(version)]
struct Cli {
    /// Path to database file
    #[arg(long, short = 'd', global = true)]
    database: Option<PathBuf>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new database
    Init {
        /// Force overwrite existing database
        #[arg(long, short = 'f')]
        force: bool,
    },

    /// Show database statistics
    Stats,

    /// Optimize database (VACUUM)
    Vacuum,

    /// Export database to JSON
    Export {
        /// Output file path
        file: PathBuf,
    },

    /// Import data from JSON
    Import {
        /// Input file path
        file: PathBuf,
        /// Merge with existing data
        #[arg(long)]
        merge: bool,
    },

    /// Create database backup
    Backup {
        /// Backup file path
        file: PathBuf,
    },

    /// Reset database to empty
    Reset {
        /// Skip confirmation
        #[arg(long, short = 'y')]
        yes: bool,
    },

    /// Check database integrity
    Check,

    /// Show database location
    Path,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let db_path = cli.database.clone().unwrap_or_else(|| {
        default_db_path().unwrap_or_else(|_| PathBuf::from("wixkb.db"))
    });

    match cli.command {
        Commands::Init { force } => {
            if db_path.exists() && !force {
                eprintln!("Database already exists: {}", db_path.display());
                eprintln!("Use --force to overwrite");
                std::process::exit(1);
            }

            if db_path.exists() {
                std::fs::remove_file(&db_path)?;
            }

            let db = Database::create(&db_path)?;
            let stats = db.get_stats()?;
            println!("Created database: {}", db_path.display());
            println!("Schema version: {}", stats.schema_version);
        }

        Commands::Stats => {
            let db = Database::open(&db_path)?;
            let stats = db.get_stats()?;

            println!("Database: {}", db_path.display());
            println!("Schema version: {}", stats.schema_version);
            if let Some(updated) = &stats.last_updated {
                println!("Last updated: {}", updated);
            }
            println!();
            println!("Contents:");
            println!("  Elements:    {:>6}", stats.elements);
            println!("  Attributes:  {:>6}", stats.attributes);
            println!("  Rules:       {:>6}", stats.rules);
            println!("  Errors:      {:>6}", stats.errors);
            println!("  ICE rules:   {:>6}", stats.ice_rules);
            println!("  MSI tables:  {:>6}", stats.msi_tables);
            println!("  Snippets:    {:>6}", stats.snippets);
            println!("  Keywords:    {:>6}", stats.keywords);

            // File size
            if let Ok(metadata) = std::fs::metadata(&db_path) {
                let size = metadata.len();
                let size_str = if size > 1024 * 1024 {
                    format!("{:.1} MB", size as f64 / (1024.0 * 1024.0))
                } else if size > 1024 {
                    format!("{:.1} KB", size as f64 / 1024.0)
                } else {
                    format!("{} bytes", size)
                };
                println!("\nFile size: {}", size_str);
            }
        }

        Commands::Vacuum => {
            let db = Database::open(&db_path)?;
            let before = std::fs::metadata(&db_path)?.len();

            db.conn().execute("VACUUM", [])?;

            let after = std::fs::metadata(&db_path)?.len();
            let saved = before.saturating_sub(after);

            println!("Vacuumed database: {}", db_path.display());
            if saved > 0 {
                println!("Freed {} bytes", saved);
            } else {
                println!("No space recovered");
            }
        }

        Commands::Export { file } => {
            let db = Database::open(&db_path)?;

            let export = export_database(&db)?;
            let json = serde_json::to_string_pretty(&export)
                .map_err(|e| WixKbError::Parse(e.to_string()))?;

            std::fs::write(&file, json)?;
            println!("Exported to: {}", file.display());
        }

        Commands::Import { file, merge } => {
            if !file.exists() {
                eprintln!("File not found: {}", file.display());
                std::process::exit(1);
            }

            let db = if merge && db_path.exists() {
                Database::open(&db_path)?
            } else {
                if db_path.exists() {
                    std::fs::remove_file(&db_path)?;
                }
                Database::create(&db_path)?
            };

            let content = std::fs::read_to_string(&file)?;
            let count = import_database(&db, &content)?;

            println!("Imported {} items from: {}", count, file.display());
        }

        Commands::Backup { file } => {
            let db = Database::open(&db_path)?;

            // Use SQLite's backup API
            let mut backup_conn = rusqlite::Connection::open(&file)?;
            let backup = Backup::new(db.conn(), &mut backup_conn)?;
            backup.run_to_completion(100, std::time::Duration::from_millis(10), None)?;

            println!("Backed up to: {}", file.display());
        }

        Commands::Reset { yes } => {
            if !yes {
                eprintln!("This will delete all data. Use --yes to confirm.");
                std::process::exit(1);
            }

            if db_path.exists() {
                std::fs::remove_file(&db_path)?;
            }

            Database::create(&db_path)?;
            println!("Database reset: {}", db_path.display());
        }

        Commands::Check => {
            let db = Database::open(&db_path)?;

            let result: String = db.conn().query_row(
                "PRAGMA integrity_check",
                [],
                |row| row.get(0),
            )?;

            if result == "ok" {
                println!("Database integrity: OK");
            } else {
                eprintln!("Database integrity check failed:");
                eprintln!("{}", result);
                std::process::exit(1);
            }
        }

        Commands::Path => {
            println!("{}", db_path.display());
        }
    }

    Ok(())
}

#[derive(serde::Serialize, serde::Deserialize)]
struct DatabaseExport {
    version: String,
    elements: Vec<wixkb::models::Element>,
    rules: Vec<wixkb::models::Rule>,
    errors: Vec<wixkb::models::WixError>,
    ice_rules: Vec<wixkb::models::IceRule>,
    snippets: Vec<wixkb::models::Snippet>,
}

fn export_database(db: &Database) -> Result<DatabaseExport> {
    let stats = db.get_stats()?;

    // Get all elements
    let mut elements = Vec::new();
    let mut stmt = db.conn().prepare(
        "SELECT id, name, namespace, since_version, deprecated_version,
                description, documentation_url, remarks, example
         FROM elements ORDER BY name"
    )?;

    let rows = stmt.query_map([], |row| {
        Ok(wixkb::models::Element {
            id: row.get(0)?,
            name: row.get(1)?,
            namespace: row.get(2)?,
            since_version: row.get(3)?,
            deprecated_version: row.get(4)?,
            description: row.get(5)?,
            documentation_url: row.get(6)?,
            remarks: row.get(7)?,
            example: row.get(8)?,
        })
    })?;

    for row in rows {
        elements.push(row?);
    }

    Ok(DatabaseExport {
        version: stats.schema_version,
        elements,
        rules: db.get_enabled_rules()?,
        errors: Vec::new(), // TODO: implement get_all_errors
        ice_rules: db.get_all_ice_rules()?,
        snippets: db.get_all_snippets()?,
    })
}

fn import_database(db: &Database, content: &str) -> Result<usize> {
    let export: DatabaseExport = serde_json::from_str(content)
        .map_err(|e| WixKbError::Parse(e.to_string()))?;

    let mut count = 0;

    for elem in &export.elements {
        db.insert_element(elem)?;
        count += 1;
    }

    for rule in &export.rules {
        db.insert_rule(rule)?;
        count += 1;
    }

    db.set_last_updated()?;

    Ok(count)
}
