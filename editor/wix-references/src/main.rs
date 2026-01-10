//! wix-references CLI
//!
//! Find definitions and references in WiX files.
//!
//! # Usage
//!
//! ```bash
//! # Find definition of a symbol
//! wix-references definition MainComp *.wxs
//!
//! # Find all references to a symbol
//! wix-references references MainComp *.wxs
//!
//! # Find all usages (definition + references)
//! wix-references usages MainComp *.wxs
//!
//! # List all symbols
//! wix-references list *.wxs
//!
//! # Show statistics
//! wix-references stats *.wxs
//! ```

use clap::{Parser, Subcommand};
use std::fs;
use std::path::PathBuf;
use wix_references::ReferenceIndex;

#[derive(Parser)]
#[command(name = "wix-references")]
#[command(about = "Find definitions and references in WiX files")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Find definition of a symbol
    Definition {
        /// Symbol name to find
        symbol: String,
        /// WiX files to search
        #[arg(required = true)]
        files: Vec<PathBuf>,
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Find all references to a symbol
    References {
        /// Symbol name to find
        symbol: String,
        /// WiX files to search
        #[arg(required = true)]
        files: Vec<PathBuf>,
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Find all usages (definition + references)
    Usages {
        /// Symbol name to find
        symbol: String,
        /// WiX files to search
        #[arg(required = true)]
        files: Vec<PathBuf>,
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// List all symbols in files
    List {
        /// WiX files to search
        #[arg(required = true)]
        files: Vec<PathBuf>,
        /// Output as JSON
        #[arg(long)]
        json: bool,
        /// Show only definitions
        #[arg(long)]
        definitions: bool,
        /// Show only references
        #[arg(long)]
        references: bool,
    },
    /// Show index statistics
    Stats {
        /// WiX files to index
        #[arg(required = true)]
        files: Vec<PathBuf>,
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Definition { symbol, files, json } => {
            let index = build_index(&files);
            match index.find_definition(&symbol) {
                Some(entry) => {
                    if json {
                        println!("{}", serde_json::to_string_pretty(entry).unwrap());
                    } else {
                        println!(
                            "{}:{}:{}: {} ({})",
                            entry.location.file,
                            entry.location.line,
                            entry.location.column,
                            entry.name,
                            entry.element
                        );
                    }
                }
                None => {
                    eprintln!("No definition found for '{}'", symbol);
                    std::process::exit(1);
                }
            }
        }
        Commands::References { symbol, files, json } => {
            let index = build_index(&files);
            let refs = index.find_references(&symbol);
            if refs.is_empty() {
                eprintln!("No references found for '{}'", symbol);
                std::process::exit(1);
            }
            if json {
                println!("{}", serde_json::to_string_pretty(&refs).unwrap());
            } else {
                for entry in refs {
                    println!(
                        "{}:{}:{}: {} ({})",
                        entry.location.file,
                        entry.location.line,
                        entry.location.column,
                        entry.name,
                        entry.element
                    );
                }
            }
        }
        Commands::Usages { symbol, files, json } => {
            let index = build_index(&files);
            let usages = index.find_all_usages(&symbol);
            if usages.is_empty() {
                eprintln!("No usages found for '{}'", symbol);
                std::process::exit(1);
            }
            if json {
                println!("{}", serde_json::to_string_pretty(&usages).unwrap());
            } else {
                for entry in usages {
                    let type_str = match entry.symbol_type {
                        wix_references::SymbolType::Definition => "def",
                        wix_references::SymbolType::Reference => "ref",
                    };
                    println!(
                        "{}:{}:{}: {} [{}] ({})",
                        entry.location.file,
                        entry.location.line,
                        entry.location.column,
                        entry.name,
                        type_str,
                        entry.element
                    );
                }
            }
        }
        Commands::List {
            files,
            json,
            definitions,
            references,
        } => {
            let index = build_index(&files);
            let show_defs = definitions || !references;
            let show_refs = references || !definitions;

            let mut entries = Vec::new();
            if show_defs {
                entries.extend(index.all_definitions());
            }
            if show_refs {
                entries.extend(index.all_references());
            }

            if json {
                println!("{}", serde_json::to_string_pretty(&entries).unwrap());
            } else {
                for entry in entries {
                    let type_str = match entry.symbol_type {
                        wix_references::SymbolType::Definition => "def",
                        wix_references::SymbolType::Reference => "ref",
                    };
                    println!(
                        "{}:{}:{}: {} [{}] ({})",
                        entry.location.file,
                        entry.location.line,
                        entry.location.column,
                        entry.name,
                        type_str,
                        entry.element
                    );
                }
            }
        }
        Commands::Stats { files, json } => {
            let index = build_index(&files);
            let stats = index.stats();
            if json {
                println!("{}", serde_json::to_string_pretty(&stats).unwrap());
            } else {
                println!("Files indexed:    {}", stats.file_count);
                println!("Definitions:      {}", stats.definition_count);
                println!("References:       {}", stats.reference_count);
                println!("Unique symbols:   {}", stats.unique_symbols);
            }
        }
    }
}

fn build_index(files: &[PathBuf]) -> ReferenceIndex {
    let mut index = ReferenceIndex::new();
    for path in files {
        match fs::read_to_string(path) {
            Ok(content) => {
                let path_str = path.to_string_lossy();
                if let Err(e) = index.add_file(&path_str, &content) {
                    eprintln!("Warning: Failed to parse {}: {}", path_str, e);
                }
            }
            Err(e) => {
                eprintln!("Warning: Failed to read {}: {}", path.display(), e);
            }
        }
    }
    index
}
