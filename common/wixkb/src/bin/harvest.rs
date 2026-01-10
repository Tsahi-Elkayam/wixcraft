//! harvest - Populate WiX Knowledge Base from sources
//!
//! Usage:
//!     harvest all                        Harvest all sources
//!     harvest source <category> <name>   Harvest specific source
//!     harvest list                       List available sources
//!     harvest status                     Show harvest status

use clap::{Parser, Subcommand};
use wixkb::config::SourcesConfig;
use wixkb::db::Database;
use wixkb::harvest::Harvester;
use wixkb::{Result, default_db_path};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "harvest")]
#[command(about = "Populate WiX Knowledge Base from sources")]
#[command(version)]
struct Cli {
    /// Path to database file
    #[arg(long, short = 'd', global = true)]
    database: Option<PathBuf>,

    /// Path to sources.yaml config
    #[arg(long, short = 'c', global = true)]
    config: Option<PathBuf>,

    /// Verbose output
    #[arg(long, short = 'v', global = true)]
    verbose: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Harvest all enabled sources
    All {
        /// Force re-harvest even if cached
        #[arg(long, short = 'f')]
        force: bool,
    },

    /// Harvest a specific source
    Source {
        /// Source category (xsd, documentation, ice, etc.)
        category: String,
        /// Source name
        name: String,
    },

    /// List available sources
    List {
        /// Show only specific category
        #[arg(long)]
        category: Option<String>,
    },

    /// Show harvest status
    Status,

    /// Clear harvest cache
    ClearCache,

    /// Validate sources configuration
    Validate,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let db_path = cli.database.clone().unwrap_or_else(|| {
        default_db_path().unwrap_or_else(|_| PathBuf::from("wixkb.db"))
    });

    let config_path = cli.config.clone().unwrap_or_else(|| {
        // Look for config relative to executable or in standard locations
        let exe_dir = std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|p| p.to_path_buf()));

        if let Some(dir) = exe_dir {
            let config = dir.join("config/sources.yaml");
            if config.exists() {
                return config;
            }
        }

        PathBuf::from("config/sources.yaml")
    });

    match cli.command {
        Commands::All { force: _ } => {
            if !db_path.exists() {
                eprintln!("Database not found. Run 'database init' first.");
                std::process::exit(1);
            }

            let db = Database::open(&db_path)?;
            let base_path = config_path.parent()
                .map(|p| p.to_path_buf())
                .unwrap_or_else(|| PathBuf::from("."));
            let harvester = Harvester::new(&config_path, &base_path)?;

            println!("Harvesting all sources...");
            let report = harvester.harvest_all(&db)?;

            println!();
            println!("Harvest complete");
            println!("================");
            println!("Sources processed: {}", report.sources_processed);
            println!("Items harvested:   {}", report.items_harvested);

            if cli.verbose && !report.source_stats.is_empty() {
                println!();
                println!("Source details:");
                for (name, stats) in &report.source_stats {
                    println!("  {}: {} items", name, stats.items);
                }
            }

            if !report.errors.is_empty() {
                println!();
                println!("Errors:");
                for error in &report.errors {
                    eprintln!("  {}", error);
                }
            }
        }

        Commands::Source { category, name } => {
            if !db_path.exists() {
                eprintln!("Database not found. Run 'database init' first.");
                std::process::exit(1);
            }

            let db = Database::open(&db_path)?;
            let base_path = config_path.parent()
                .map(|p| p.to_path_buf())
                .unwrap_or_else(|| PathBuf::from("."));
            let harvester = Harvester::new(&config_path, &base_path)?;

            let config = SourcesConfig::load(&config_path)?;
            let source = config.get_source(&category, &name);

            if let Some(source) = source {
                println!("Harvesting {}/{}...", category, name);
                let stats = harvester.harvest_source(&db, &category, &name, source)?;
                println!("Harvested {} items", stats.items);
            } else {
                eprintln!("Source not found: {}/{}", category, name);
                std::process::exit(1);
            }
        }

        Commands::List { category } => {
            let config = SourcesConfig::load(&config_path)?;

            if let Some(cat) = category {
                if let Some(sources) = config.get_sources(&cat) {
                    println!("Sources in '{}':", cat);
                    for (name, source) in sources {
                        let location = source.url.as_deref()
                            .or(source.path.as_deref())
                            .unwrap_or("-");
                        println!("  {}: {}", name, location);
                    }
                } else {
                    eprintln!("Category not found: {}", cat);
                    std::process::exit(1);
                }
            } else {
                println!("Available sources:");
                for cat in config.categories() {
                    println!();
                    println!("[{}]", cat);
                    if let Some(sources) = config.get_sources(cat) {
                        for (name, source) in sources {
                            let location = source.url.as_deref()
                                .or(source.path.as_deref())
                                .unwrap_or("-");
                            println!("  {}: {}", name, location);
                        }
                    }
                }
                println!();
                println!("Total: {} sources", config.total_sources());
            }
        }

        Commands::Status => {
            if !db_path.exists() {
                println!("Database: not initialized");
                println!("Run 'database init' to create the database.");
                return Ok(());
            }

            let db = Database::open(&db_path)?;
            let stats = db.get_stats()?;

            println!("Database: {}", db_path.display());
            println!("Status:   initialized");
            if let Some(updated) = &stats.last_updated {
                println!("Last harvest: {}", updated);
            } else {
                println!("Last harvest: never");
            }

            println!();
            println!("Contents:");
            println!("  Elements:   {:>5}", stats.elements);
            println!("  Attributes: {:>5}", stats.attributes);
            println!("  Rules:      {:>5}", stats.rules);
            println!("  Errors:     {:>5}", stats.errors);
            println!("  ICE rules:  {:>5}", stats.ice_rules);
            println!("  Snippets:   {:>5}", stats.snippets);
            println!("  Keywords:   {:>5}", stats.keywords);

            let total = stats.elements + stats.rules + stats.errors
                + stats.ice_rules + stats.snippets + stats.keywords;

            if total == 0 {
                println!();
                println!("Database is empty. Run 'harvest all' to populate.");
            }
        }

        Commands::ClearCache => {
            let base_path = config_path.parent()
                .map(|p| p.to_path_buf())
                .unwrap_or_else(|| PathBuf::from("."));
            let cache_dir = base_path.join(".cache");

            if cache_dir.exists() {
                let count = std::fs::read_dir(&cache_dir)?
                    .filter_map(|e| e.ok())
                    .count();

                std::fs::remove_dir_all(&cache_dir)?;
                std::fs::create_dir_all(&cache_dir)?;

                println!("Cleared {} cached files", count);
            } else {
                println!("Cache is empty");
            }
        }

        Commands::Validate => {
            match SourcesConfig::load(&config_path) {
                Ok(config) => {
                    println!("Configuration: valid");
                    println!("Categories:    {}", config.categories().len());
                    println!("Sources:       {}", config.total_sources());
                    println!("Parsers:       {}", config.parsers.len());
                }
                Err(e) => {
                    eprintln!("Configuration error: {}", e);
                    std::process::exit(1);
                }
            }
        }
    }

    Ok(())
}
