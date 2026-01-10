//! wixkb - Query the WiX Knowledge Base
//!
//! Usage:
//!     wixkb element <name>         Show element details
//!     wixkb attribute <elem> <attr>  Show attribute details
//!     wixkb search <query>         Full-text search
//!     wixkb children <element>     List child elements
//!     wixkb parents <element>      List parent elements
//!     wixkb rule <id>              Show lint rule details
//!     wixkb error <code>           Show error code details
//!     wixkb ice <code>             Show ICE rule details
//!     wixkb stats                  Show database statistics

use clap::{Parser, Subcommand};
use wixkb::{WixKb, Result};

#[derive(Parser)]
#[command(name = "wixkb")]
#[command(about = "Query the WiX Knowledge Base")]
#[command(version)]
struct Cli {
    /// Path to database file
    #[arg(long, short = 'd')]
    database: Option<String>,

    /// Output format
    #[arg(long, short = 'f', default_value = "text")]
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

    /// Full-text search
    Search {
        /// Search query
        query: String,
        /// Maximum results
        #[arg(long, short = 'n', default_value = "10")]
        limit: usize,
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

    /// Show lint rule details
    Rule {
        /// Rule ID (e.g., COMP001)
        id: String,
    },

    /// List rules by category
    Rules {
        /// Category (component, file, directory, etc.)
        #[arg(long, short = 'c')]
        category: Option<String>,
    },

    /// Show error code details
    Error {
        /// Error code (e.g., WIX0001)
        code: String,
    },

    /// Show ICE rule details
    Ice {
        /// ICE code (e.g., ICE03)
        code: String,
    },

    /// Show standard directory details
    Directory {
        /// Directory name (e.g., ProgramFilesFolder)
        name: String,
    },

    /// Show snippets
    Snippets {
        /// Prefix filter
        #[arg(long, short = 'p')]
        prefix: Option<String>,
    },

    /// Show database statistics
    Stats,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let kb = if let Some(path) = &cli.database {
        WixKb::open(path)?
    } else {
        WixKb::open_default()?
    };

    match cli.command {
        Commands::Element { name } => {
            if let Some(elem) = kb.get_element(&name)? {
                match cli.format {
                    OutputFormat::Text => print_element(&elem, &kb)?,
                    OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&elem).unwrap()),
                }
            } else {
                eprintln!("Element not found: {}", name);
                std::process::exit(1);
            }
        }

        Commands::Attribute { element, attribute } => {
            let attrs = kb.get_attributes(&element)?;
            if let Some(attr) = attrs.iter().find(|a| a.name.eq_ignore_ascii_case(&attribute)) {
                match cli.format {
                    OutputFormat::Text => print_attribute(attr),
                    OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&attr).unwrap()),
                }
            } else {
                eprintln!("Attribute not found: {}/@{}", element, attribute);
                std::process::exit(1);
            }
        }

        Commands::Search { query, limit } => {
            let results = kb.search_elements_fts(&query, limit)?;
            match cli.format {
                OutputFormat::Text => {
                    if results.is_empty() {
                        println!("No results found");
                    } else {
                        for elem in &results {
                            println!("{}: {}", elem.name, elem.description.as_deref().unwrap_or("-"));
                        }
                    }
                }
                OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&results).unwrap()),
            }
        }

        Commands::Children { element } => {
            let children = kb.get_children(&element)?;
            match cli.format {
                OutputFormat::Text => {
                    if children.is_empty() {
                        println!("No children found for {}", element);
                    } else {
                        for child in &children {
                            println!("  {}", child);
                        }
                    }
                }
                OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&children).unwrap()),
            }
        }

        Commands::Parents { element } => {
            let parents = kb.get_parents(&element)?;
            match cli.format {
                OutputFormat::Text => {
                    if parents.is_empty() {
                        println!("No parents found for {}", element);
                    } else {
                        for parent in &parents {
                            println!("  {}", parent);
                        }
                    }
                }
                OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&parents).unwrap()),
            }
        }

        Commands::Rule { id } => {
            if let Some(rule) = kb.get_rule(&id)? {
                match cli.format {
                    OutputFormat::Text => print_rule(&rule),
                    OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&rule).unwrap()),
                }
            } else {
                eprintln!("Rule not found: {}", id);
                std::process::exit(1);
            }
        }

        Commands::Rules { category } => {
            let rules = if let Some(cat) = &category {
                kb.get_rules_by_category(cat)?
            } else {
                kb.get_enabled_rules()?
            };

            match cli.format {
                OutputFormat::Text => {
                    for rule in &rules {
                        println!("[{}] {} - {}", rule.rule_id, rule.severity, rule.name);
                    }
                }
                OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&rules).unwrap()),
            }
        }

        Commands::Error { code } => {
            if let Some(err) = kb.get_error(&code)? {
                match cli.format {
                    OutputFormat::Text => print_error(&err),
                    OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&err).unwrap()),
                }
            } else {
                eprintln!("Error code not found: {}", code);
                std::process::exit(1);
            }
        }

        Commands::Ice { code } => {
            if let Some(ice) = kb.get_ice_rule(&code)? {
                match cli.format {
                    OutputFormat::Text => print_ice_rule(&ice),
                    OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&ice).unwrap()),
                }
            } else {
                eprintln!("ICE rule not found: {}", code);
                std::process::exit(1);
            }
        }

        Commands::Directory { name } => {
            if let Some(dir) = kb.get_standard_directory(&name)? {
                match cli.format {
                    OutputFormat::Text => print_directory(&dir),
                    OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&dir).unwrap()),
                }
            } else {
                eprintln!("Directory not found: {}", name);
                std::process::exit(1);
            }
        }

        Commands::Snippets { prefix } => {
            let snippets = if let Some(p) = &prefix {
                kb.get_snippets(p)?
            } else {
                kb.get_all_snippets()?
            };

            match cli.format {
                OutputFormat::Text => {
                    for snippet in &snippets {
                        println!("{}: {}", snippet.prefix, snippet.name);
                    }
                }
                OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&snippets).unwrap()),
            }
        }

        Commands::Stats => {
            let stats = kb.get_stats()?;
            match cli.format {
                OutputFormat::Text => print_stats(&stats),
                OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&stats).unwrap()),
            }
        }
    }

    Ok(())
}

fn print_element(elem: &wixkb::models::Element, kb: &WixKb) -> Result<()> {
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

    let attrs = kb.get_attributes(&elem.name)?;
    if !attrs.is_empty() {
        println!("\nAttributes:");
        for attr in &attrs {
            let req = if attr.required { " (required)" } else { "" };
            println!("  @{}: {}{}", attr.name, attr.attr_type, req);
        }
    }

    let children = kb.get_children(&elem.name)?;
    if !children.is_empty() {
        println!("\nChildren: {}", children.join(", "));
    }

    let parents = kb.get_parents(&elem.name)?;
    if !parents.is_empty() {
        println!("Parents: {}", parents.join(", "));
    }

    Ok(())
}

fn print_attribute(attr: &wixkb::models::Attribute) {
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

fn print_rule(rule: &wixkb::models::Rule) {
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

fn print_error(err: &wixkb::models::WixError) {
    println!("[{}] {}", err.code, err.severity);
    println!("\nMessage: {}", err.message_template);
    if let Some(desc) = &err.description {
        println!("\n{}", desc);
    }
    if let Some(resolution) = &err.resolution {
        println!("\nResolution: {}", resolution);
    }
}

fn print_ice_rule(ice: &wixkb::models::IceRule) {
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

fn print_directory(dir: &wixkb::models::StandardDirectory) {
    println!("Directory: {}", dir.name);
    if let Some(desc) = &dir.description {
        println!("{}", desc);
    }
    if let Some(path) = &dir.windows_path {
        println!("\nPath: {}", path);
    }
}

fn print_stats(stats: &wixkb::models::DbStats) {
    println!("WiX Knowledge Base Statistics");
    println!("==============================");
    println!("Schema version: {}", stats.schema_version);
    if let Some(updated) = &stats.last_updated {
        println!("Last updated: {}", updated);
    }
    println!();
    println!("Elements:    {:>6}", stats.elements);
    println!("Attributes:  {:>6}", stats.attributes);
    println!("Rules:       {:>6}", stats.rules);
    println!("Errors:      {:>6}", stats.errors);
    println!("ICE rules:   {:>6}", stats.ice_rules);
    println!("MSI tables:  {:>6}", stats.msi_tables);
    println!("Snippets:    {:>6}", stats.snippets);
    println!("Keywords:    {:>6}", stats.keywords);
}
