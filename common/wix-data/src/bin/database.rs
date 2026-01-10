//! database - WiX Data Layer database management
//!
//! All database operations in one place:
//!   database init              Create new database
//!   database stats             Show statistics
//!   database vacuum            Optimize database
//!   database check             Check integrity
//!   database backup <file>     Create backup
//!   database export <file>     Export to JSON
//!   database import <file>     Import from JSON
//!   database reset             Reset to empty
//!   database path              Show database location
//!
//! Query operations:
//!   database query element <name>       Show element details
//!   database query attribute <e> <a>    Show attribute details
//!   database query rule <id>            Show rule details
//!   database query ice <code>           Show ICE rule
//!   database query error <code>         Show error details
//!   database query directory <name>     Show standard directory
//!   database query children <element>   List child elements
//!   database query parents <element>    List parent elements
//!   database search <query>             Full-text search
//!   database list elements              List all elements
//!   database list rules [--category]    List rules
//!   database list snippets              List snippets

use clap::{Parser, Subcommand};
use rusqlite::backup::Backup;
use std::path::PathBuf;
use wix_data::db::Database;
use wix_data::{default_db_path, Result, WixData, WixDataError};

#[derive(Parser)]
#[command(name = "database")]
#[command(about = "WiX Data Layer database management")]
#[command(version)]
struct Cli {
    /// Path to database file
    #[arg(long, short = 'd', global = true)]
    database: Option<PathBuf>,

    /// Output format (text or json)
    #[arg(long, short = 'f', global = true, default_value = "text")]
    format: OutputFormat,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Clone, Copy, Debug, Default, clap::ValueEnum)]
enum OutputFormat {
    #[default]
    Text,
    Json,
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

    /// Check database integrity
    Check,

    /// Create database backup
    Backup {
        /// Backup file path
        file: PathBuf,
    },

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

    /// Reset database to empty
    Reset {
        /// Skip confirmation
        #[arg(long, short = 'y')]
        yes: bool,
    },

    /// Show database location
    Path,

    /// Query database
    Query {
        #[command(subcommand)]
        query: QueryCommands,
    },

    /// Full-text search
    Search {
        /// Search query
        query: String,
        /// Maximum results
        #[arg(long, short = 'n', default_value = "10")]
        limit: usize,
    },

    /// List items
    List {
        #[command(subcommand)]
        list: ListCommands,
    },
}

#[derive(Subcommand)]
enum QueryCommands {
    /// Show element details
    Element {
        /// Element name
        name: String,
    },

    /// Show attribute details
    Attribute {
        /// Element name
        element: String,
        /// Attribute name
        attribute: String,
    },

    /// Show lint rule details
    Rule {
        /// Rule ID (e.g., COMP001)
        id: String,
    },

    /// Show ICE rule details
    Ice {
        /// ICE code (e.g., ICE03)
        code: String,
    },

    /// Show error code details
    Error {
        /// Error code (e.g., WIX0001)
        code: String,
    },

    /// Show standard directory details
    Directory {
        /// Directory name (e.g., ProgramFilesFolder)
        name: String,
    },

    /// List child elements
    Children {
        /// Parent element name
        element: String,
    },

    /// List parent elements
    Parents {
        /// Child element name
        element: String,
    },
}

#[derive(Subcommand)]
enum ListCommands {
    /// List all elements
    Elements {
        /// Filter by namespace
        #[arg(long)]
        namespace: Option<String>,
    },

    /// List rules
    Rules {
        /// Filter by category
        #[arg(long, short = 'c')]
        category: Option<String>,
    },

    /// List snippets
    Snippets {
        /// Filter by prefix
        #[arg(long, short = 'p')]
        prefix: Option<String>,
    },

    /// List ICE rules
    Ice,

    /// List standard directories
    Directories,

    /// List MSI tables
    Tables,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let db_path = cli
        .database
        .clone()
        .unwrap_or_else(|| default_db_path().unwrap_or_else(|_| PathBuf::from("wix-data.db")));

    match cli.command {
        // ============ Database Management ============
        Commands::Init { force } => {
            if db_path.exists() && !force {
                eprintln!("Database already exists: {}", db_path.display());
                eprintln!("Use --force to overwrite");
                std::process::exit(1);
            }

            if db_path.exists() {
                std::fs::remove_file(&db_path)?;
            }

            if let Some(parent) = db_path.parent() {
                std::fs::create_dir_all(parent)?;
            }

            let db = Database::create(&db_path)?;
            let stats = db.get_stats()?;
            println!("Created database: {}", db_path.display());
            println!("Schema version: {}", stats.schema_version);
        }

        Commands::Stats => {
            let db = Database::open(&db_path)?;
            let stats = db.get_stats()?;

            match cli.format {
                OutputFormat::Json => {
                    println!("{}", serde_json::to_string_pretty(&stats).unwrap());
                }
                OutputFormat::Text => {
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

        Commands::Check => {
            let db = Database::open(&db_path)?;
            let result: String =
                db.conn()
                    .query_row("PRAGMA integrity_check", [], |row| row.get(0))?;

            if result == "ok" {
                println!("Database integrity: OK");
            } else {
                eprintln!("Database integrity check failed:");
                eprintln!("{}", result);
                std::process::exit(1);
            }
        }

        Commands::Backup { file } => {
            let db = Database::open(&db_path)?;
            let mut backup_conn = rusqlite::Connection::open(&file)?;
            let backup = Backup::new(db.conn(), &mut backup_conn)?;
            backup.run_to_completion(100, std::time::Duration::from_millis(10), None)?;
            println!("Backed up to: {}", file.display());
        }

        Commands::Export { file } => {
            let db = Database::open(&db_path)?;
            let export = export_database(&db)?;
            let json = serde_json::to_string_pretty(&export)
                .map_err(|e| WixDataError::Parse(e.to_string()))?;
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

        Commands::Path => {
            println!("{}", db_path.display());
        }

        // ============ Query Operations ============
        Commands::Query { query } => {
            let data = WixData::open(&db_path)?;

            match query {
                QueryCommands::Element { name } => {
                    if let Some(elem) = data.get_element(&name)? {
                        match cli.format {
                            OutputFormat::Json => {
                                println!("{}", serde_json::to_string_pretty(&elem).unwrap());
                            }
                            OutputFormat::Text => {
                                print_element(&elem, &data)?;
                            }
                        }
                    } else {
                        eprintln!("Element not found: {}", name);
                        std::process::exit(1);
                    }
                }

                QueryCommands::Attribute { element, attribute } => {
                    let attrs = data.get_attributes(&element)?;
                    if let Some(attr) = attrs
                        .iter()
                        .find(|a| a.name.eq_ignore_ascii_case(&attribute))
                    {
                        match cli.format {
                            OutputFormat::Json => {
                                println!("{}", serde_json::to_string_pretty(&attr).unwrap());
                            }
                            OutputFormat::Text => {
                                print_attribute(attr);
                            }
                        }
                    } else {
                        eprintln!("Attribute not found: {}/@{}", element, attribute);
                        std::process::exit(1);
                    }
                }

                QueryCommands::Rule { id } => {
                    if let Some(rule) = data.get_rule(&id)? {
                        match cli.format {
                            OutputFormat::Json => {
                                println!("{}", serde_json::to_string_pretty(&rule).unwrap());
                            }
                            OutputFormat::Text => {
                                print_rule(&rule);
                            }
                        }
                    } else {
                        eprintln!("Rule not found: {}", id);
                        std::process::exit(1);
                    }
                }

                QueryCommands::Ice { code } => {
                    if let Some(ice) = data.get_ice_rule(&code)? {
                        match cli.format {
                            OutputFormat::Json => {
                                println!("{}", serde_json::to_string_pretty(&ice).unwrap());
                            }
                            OutputFormat::Text => {
                                print_ice_rule(&ice);
                            }
                        }
                    } else {
                        eprintln!("ICE rule not found: {}", code);
                        std::process::exit(1);
                    }
                }

                QueryCommands::Error { code } => {
                    if let Some(err) = data.get_error(&code)? {
                        match cli.format {
                            OutputFormat::Json => {
                                println!("{}", serde_json::to_string_pretty(&err).unwrap());
                            }
                            OutputFormat::Text => {
                                print_error(&err);
                            }
                        }
                    } else {
                        eprintln!("Error not found: {}", code);
                        std::process::exit(1);
                    }
                }

                QueryCommands::Directory { name } => {
                    if let Some(dir) = data.get_standard_directory(&name)? {
                        match cli.format {
                            OutputFormat::Json => {
                                println!("{}", serde_json::to_string_pretty(&dir).unwrap());
                            }
                            OutputFormat::Text => {
                                print_directory(&dir);
                            }
                        }
                    } else {
                        eprintln!("Directory not found: {}", name);
                        std::process::exit(1);
                    }
                }

                QueryCommands::Children { element } => {
                    let children = data.get_children(&element)?;
                    match cli.format {
                        OutputFormat::Json => {
                            println!("{}", serde_json::to_string_pretty(&children).unwrap());
                        }
                        OutputFormat::Text => {
                            if children.is_empty() {
                                println!("No children found for {}", element);
                            } else {
                                for child in &children {
                                    println!("  {}", child);
                                }
                            }
                        }
                    }
                }

                QueryCommands::Parents { element } => {
                    let parents = data.get_parents(&element)?;
                    match cli.format {
                        OutputFormat::Json => {
                            println!("{}", serde_json::to_string_pretty(&parents).unwrap());
                        }
                        OutputFormat::Text => {
                            if parents.is_empty() {
                                println!("No parents found for {}", element);
                            } else {
                                for parent in &parents {
                                    println!("  {}", parent);
                                }
                            }
                        }
                    }
                }
            }
        }

        // ============ Search ============
        Commands::Search { query, limit } => {
            let data = WixData::open(&db_path)?;
            let results = data.search_elements_fts(&query, limit)?;

            match cli.format {
                OutputFormat::Json => {
                    println!("{}", serde_json::to_string_pretty(&results).unwrap());
                }
                OutputFormat::Text => {
                    if results.is_empty() {
                        println!("No results found");
                    } else {
                        for elem in &results {
                            println!(
                                "{}: {}",
                                elem.name,
                                elem.description.as_deref().unwrap_or("-")
                            );
                        }
                    }
                }
            }
        }

        // ============ List Operations ============
        Commands::List { list } => {
            let data = WixData::open(&db_path)?;

            match list {
                ListCommands::Elements { namespace } => {
                    let elements = data.search_elements("", 1000)?;
                    let filtered: Vec<_> = if let Some(ns) = &namespace {
                        elements.iter().filter(|e| &e.namespace == ns).collect()
                    } else {
                        elements.iter().collect()
                    };

                    match cli.format {
                        OutputFormat::Json => {
                            println!("{}", serde_json::to_string_pretty(&filtered).unwrap());
                        }
                        OutputFormat::Text => {
                            for elem in filtered {
                                println!("{}", elem.name);
                            }
                        }
                    }
                }

                ListCommands::Rules { category } => {
                    let rules = if let Some(cat) = &category {
                        data.get_rules_by_category(cat)?
                    } else {
                        data.get_enabled_rules()?
                    };

                    match cli.format {
                        OutputFormat::Json => {
                            println!("{}", serde_json::to_string_pretty(&rules).unwrap());
                        }
                        OutputFormat::Text => {
                            for rule in &rules {
                                println!("[{}] {} - {}", rule.rule_id, rule.severity, rule.name);
                            }
                        }
                    }
                }

                ListCommands::Snippets { prefix } => {
                    let snippets = if let Some(p) = &prefix {
                        data.get_snippets(p)?
                    } else {
                        data.get_all_snippets()?
                    };

                    match cli.format {
                        OutputFormat::Json => {
                            println!("{}", serde_json::to_string_pretty(&snippets).unwrap());
                        }
                        OutputFormat::Text => {
                            for snippet in &snippets {
                                println!("{}: {}", snippet.prefix, snippet.name);
                            }
                        }
                    }
                }

                ListCommands::Ice => {
                    let rules = data.get_all_ice_rules()?;
                    match cli.format {
                        OutputFormat::Json => {
                            println!("{}", serde_json::to_string_pretty(&rules).unwrap());
                        }
                        OutputFormat::Text => {
                            for rule in &rules {
                                println!("[{}] {}", rule.code, rule.severity);
                            }
                        }
                    }
                }

                ListCommands::Directories => {
                    let dirs = data.get_all_standard_directories()?;
                    match cli.format {
                        OutputFormat::Json => {
                            println!("{}", serde_json::to_string_pretty(&dirs).unwrap());
                        }
                        OutputFormat::Text => {
                            for dir in &dirs {
                                println!("{}", dir.name);
                            }
                        }
                    }
                }

                ListCommands::Tables => {
                    let db = data.db();
                    let mut stmt = db.conn().prepare(
                        "SELECT name, description FROM msi_tables ORDER BY name",
                    )?;
                    let tables: Vec<(String, Option<String>)> = stmt
                        .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?
                        .filter_map(|r| r.ok())
                        .collect();

                    match cli.format {
                        OutputFormat::Json => {
                            println!("{}", serde_json::to_string_pretty(&tables).unwrap());
                        }
                        OutputFormat::Text => {
                            for (name, _desc) in &tables {
                                println!("{}", name);
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

// ============ Print Functions ============

fn print_element(elem: &wix_data::models::Element, data: &WixData) -> Result<()> {
    println!("Element: {}", elem.name);
    println!("Namespace: {}", elem.namespace);
    if let Some(since) = &elem.since_version {
        println!("Since: {}", since);
    }
    if let Some(desc) = &elem.description {
        println!("\n{}", desc);
    }
    if let Some(url) = &elem.documentation_url {
        println!("\nDocs: {}", url);
    }

    let attrs = data.get_attributes(&elem.name)?;
    if !attrs.is_empty() {
        println!("\nAttributes:");
        for attr in &attrs {
            let req = if attr.required { " (required)" } else { "" };
            println!("  @{}: {}{}", attr.name, attr.attr_type, req);
        }
    }

    let children = data.get_children(&elem.name)?;
    if !children.is_empty() {
        println!("\nChildren: {}", children.join(", "));
    }

    let parents = data.get_parents(&elem.name)?;
    if !parents.is_empty() {
        println!("Parents: {}", parents.join(", "));
    }

    Ok(())
}

fn print_attribute(attr: &wix_data::models::Attribute) {
    println!("Attribute: @{}", attr.name);
    println!("Type: {}", attr.attr_type);
    println!("Required: {}", if attr.required { "yes" } else { "no" });
    if let Some(default) = &attr.default_value {
        println!("Default: {}", default);
    }
    if let Some(desc) = &attr.description {
        println!("\n{}", desc);
    }
    if !attr.enum_values.is_empty() {
        println!("\nValid values: {}", attr.enum_values.join(", "));
    }
}

fn print_rule(rule: &wix_data::models::Rule) {
    println!("[{}] {}", rule.rule_id, rule.name);
    println!("Category: {}", rule.category);
    println!("Severity: {}", rule.severity);
    if let Some(desc) = &rule.description {
        println!("\n{}", desc);
    }
    if let Some(rationale) = &rule.rationale {
        println!("\nRationale: {}", rationale);
    }
    if let Some(fix) = &rule.fix_suggestion {
        println!("\nFix: {}", fix);
    }
}

fn print_ice_rule(ice: &wix_data::models::IceRule) {
    println!("[{}] {}", ice.code, ice.severity);
    if let Some(desc) = &ice.description {
        println!("\n{}", desc);
    }
    if !ice.tables_affected.is_empty() {
        println!("\nTables: {}", ice.tables_affected.join(", "));
    }
    if let Some(resolution) = &ice.resolution {
        println!("\nResolution: {}", resolution);
    }
}

fn print_error(err: &wix_data::models::WixError) {
    println!("[{}] {}", err.code, err.severity);
    println!("\nMessage: {}", err.message_template);
    if let Some(desc) = &err.description {
        println!("\n{}", desc);
    }
    if let Some(resolution) = &err.resolution {
        println!("\nResolution: {}", resolution);
    }
}

fn print_directory(dir: &wix_data::models::StandardDirectory) {
    println!("Directory: {}", dir.name);
    if let Some(desc) = &dir.description {
        println!("{}", desc);
    }
    if let Some(path) = &dir.windows_path {
        println!("\nPath: {}", path);
    }
}

// ============ Export/Import ============

#[derive(serde::Serialize, serde::Deserialize)]
struct DatabaseExport {
    version: String,
    elements: Vec<wix_data::models::Element>,
    rules: Vec<wix_data::models::Rule>,
    errors: Vec<wix_data::models::WixError>,
    ice_rules: Vec<wix_data::models::IceRule>,
    snippets: Vec<wix_data::models::Snippet>,
}

fn export_database(db: &Database) -> Result<DatabaseExport> {
    let stats = db.get_stats()?;

    let mut elements = Vec::new();
    let mut stmt = db.conn().prepare(
        "SELECT id, name, namespace, since_version, deprecated_version,
                description, documentation_url, remarks, example
         FROM elements ORDER BY name",
    )?;

    let rows = stmt.query_map([], |row| {
        Ok(wix_data::models::Element {
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
        errors: Vec::new(),
        ice_rules: db.get_all_ice_rules()?,
        snippets: db.get_all_snippets()?,
    })
}

fn import_database(db: &Database, content: &str) -> Result<usize> {
    let export: DatabaseExport =
        serde_json::from_str(content).map_err(|e| WixDataError::Parse(e.to_string()))?;

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
