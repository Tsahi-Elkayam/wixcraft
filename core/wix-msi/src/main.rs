//! wix-msi CLI - Cross-platform MSI compiler
//!
//! Usage:
//!   wix-msi compile product.wxs -o product.msi
//!   wix-msi tables product.wxs              # Show MSI tables
//!   wix-msi validate product.wxs            # Validate WiX source

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use wix_msi::*;

#[derive(Parser)]
#[command(name = "wix-msi")]
#[command(about = "Cross-platform MSI compiler - build MSI on any OS")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Compile WiX source to MSI database (preview mode)
    Compile {
        /// WiX source file(s)
        #[arg(required = true)]
        files: Vec<PathBuf>,

        /// Output MSI file
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Output format (json for preview, msi for actual build)
        #[arg(short, long, default_value = "json")]
        format: String,
    },

    /// Show MSI tables that would be generated
    Tables {
        /// WiX source file(s)
        #[arg(required = true)]
        files: Vec<PathBuf>,

        /// Specific table to show
        #[arg(short, long)]
        table: Option<String>,

        /// Output format (text, json)
        #[arg(short, long, default_value = "text")]
        format: String,
    },

    /// Validate WiX source for MSI compatibility
    Validate {
        /// WiX source file(s)
        #[arg(required = true)]
        files: Vec<PathBuf>,
    },

    /// Generate GUID
    Guid {
        /// Number of GUIDs to generate
        #[arg(short, long, default_value = "1")]
        count: usize,

        /// Format (braces, plain, uppercase)
        #[arg(short, long, default_value = "braces")]
        format: String,
    },

    /// Show MSI table schema
    Schema {
        /// Table name (Property, Directory, Component, Feature, File)
        table: Option<String>,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Compile {
            files,
            output,
            format,
        } => {
            let mut all_content = String::new();
            for file in &files {
                if !file.exists() {
                    eprintln!("Warning: File not found: {}", file.display());
                    continue;
                }
                all_content.push_str(&std::fs::read_to_string(file)?);
                all_content.push('\n');
            }

            let db = MsiCompiler::compile(&all_content)?;

            match format.as_str() {
                "msi" => {
                    eprintln!("Note: Full MSI binary generation is not yet implemented.");
                    eprintln!("Use --format json for database preview.");
                    eprintln!();
                    eprintln!("The MSI binary format requires:");
                    eprintln!("  - OLE Structured Storage implementation");
                    eprintln!("  - CAB compression for file streams");
                    eprintln!("  - MSI table serialization");
                    eprintln!();
                    eprintln!("For now, use WiX Toolset on Windows or Wine for actual MSI generation.");

                    // Still output the JSON to the specified file
                    if let Some(out_path) = output {
                        let json_path = out_path.with_extension("json");
                        std::fs::write(&json_path, serde_json::to_string_pretty(&db)?)?;
                        println!("Database preview written to: {}", json_path.display());
                    }
                }
                _ => {
                    let json = serde_json::to_string_pretty(&db)?;
                    if let Some(out_path) = output {
                        std::fs::write(&out_path, &json)?;
                        println!("Written to: {}", out_path.display());
                    } else {
                        println!("{}", json);
                    }
                }
            }
        }

        Commands::Tables { files, table, format } => {
            let mut all_content = String::new();
            for file in &files {
                if !file.exists() {
                    continue;
                }
                all_content.push_str(&std::fs::read_to_string(file)?);
            }

            let db = MsiCompiler::compile(&all_content)?;

            if format == "json" {
                if let Some(ref table_name) = table {
                    if let Some(t) = db.tables.get(table_name) {
                        println!("{}", serde_json::to_string_pretty(t)?);
                    } else {
                        eprintln!("Table not found: {}", table_name);
                    }
                } else {
                    println!("{}", serde_json::to_string_pretty(&db.tables)?);
                }
            } else {
                if let Some(ref table_name) = table {
                    if let Some(t) = db.tables.get(table_name) {
                        print_table(t);
                    } else {
                        eprintln!("Table not found: {}", table_name);
                    }
                } else {
                    println!("MSI Tables:");
                    println!("{}", "=".repeat(50));
                    for (name, t) in &db.tables {
                        println!("\n{} ({} rows)", name, t.rows.len());
                        println!("{}", "-".repeat(40));
                        if !t.rows.is_empty() {
                            print_table(t);
                        }
                    }
                }
            }
        }

        Commands::Validate { files } => {
            let mut all_content = String::new();
            for file in &files {
                if !file.exists() {
                    eprintln!("Error: File not found: {}", file.display());
                    std::process::exit(1);
                }
                all_content.push_str(&std::fs::read_to_string(file)?);
            }

            println!("Validating WiX source...");

            match MsiCompiler::compile(&all_content) {
                Ok(db) => {
                    println!("Validation passed.");
                    println!();
                    println!("Summary:");
                    println!("  Tables: {}", db.tables.len());

                    let props = db.tables.get("Property").map(|t| t.rows.len()).unwrap_or(0);
                    let dirs = db.tables.get("Directory").map(|t| t.rows.len()).unwrap_or(0);
                    let comps = db.tables.get("Component").map(|t| t.rows.len()).unwrap_or(0);
                    let feats = db.tables.get("Feature").map(|t| t.rows.len()).unwrap_or(0);

                    println!("  Properties: {}", props);
                    println!("  Directories: {}", dirs);
                    println!("  Components: {}", comps);
                    println!("  Features: {}", feats);
                }
                Err(e) => {
                    eprintln!("Validation failed: {}", e);
                    std::process::exit(1);
                }
            }
        }

        Commands::Guid { count, format } => {
            for _ in 0..count {
                let guid = generate_guid();
                let output = match format.as_str() {
                    "plain" => guid.trim_matches(|c| c == '{' || c == '}').to_lowercase(),
                    "uppercase" => guid.trim_matches(|c| c == '{' || c == '}').to_string(),
                    _ => guid,
                };
                println!("{}", output);
            }
        }

        Commands::Schema { table } => {
            let mut db = MsiDatabase::new();
            db.init_standard_tables();

            if let Some(table_name) = table {
                if let Some(t) = db.tables.get(&table_name) {
                    println!("Table: {}", t.name);
                    println!("{}", "=".repeat(50));
                    println!();
                    println!("Columns:");
                    for col in &t.columns {
                        let pk = if col.primary_key { " [PK]" } else { "" };
                        let null = if col.nullable { " (nullable)" } else { "" };
                        let size = col.size.map(|s| format!("({})", s)).unwrap_or_default();
                        println!(
                            "  {} - {:?}{}{}{}",
                            col.name, col.column_type, size, null, pk
                        );
                    }
                } else {
                    eprintln!("Unknown table: {}", table_name);
                    eprintln!("Available: Property, Directory, Component, Feature, File");
                }
            } else {
                println!("Available MSI Tables:");
                println!("{}", "=".repeat(50));
                for name in db.tables.keys() {
                    println!("  {}", name);
                }
                println!();
                println!("Use 'wix-msi schema <table>' for details.");
            }
        }
    }

    Ok(())
}

fn print_table(table: &MsiTable) {
    // Print column headers
    let headers: Vec<&str> = table.columns.iter().map(|c| c.name.as_str()).collect();
    println!("{}", headers.join(" | "));
    println!("{}", "-".repeat(headers.len() * 15));

    // Print rows (limited)
    for row in table.rows.iter().take(10) {
        let values: Vec<String> = row
            .iter()
            .map(|v| match v {
                MsiValue::Null => "NULL".to_string(),
                MsiValue::Integer(i) => i.to_string(),
                MsiValue::String(s) => {
                    if s.len() > 30 {
                        format!("{}...", &s[..27])
                    } else {
                        s.clone()
                    }
                }
                MsiValue::Binary(_) => "[BINARY]".to_string(),
            })
            .collect();
        println!("{}", values.join(" | "));
    }

    if table.rows.len() > 10 {
        println!("... and {} more rows", table.rows.len() - 10);
    }
}
