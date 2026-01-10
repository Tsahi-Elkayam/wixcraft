//! config - Manage WiX Knowledge Base configuration
//!
//! Usage:
//!     config show                    Show current configuration
//!     config sources list            List configured sources
//!     config sources add <...>       Add a source
//!     config sources remove <name>   Remove a source
//!     config rules enable <id>       Enable a lint rule
//!     config rules disable <id>      Disable a lint rule
//!     config lint show               Show lint configuration
//!     config lint init               Initialize .wixlintrc.json

use clap::{Parser, Subcommand};
use wixkb::config::{SourcesConfig, LintConfig};
use wixkb::Result;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "config")]
#[command(about = "Manage WiX Knowledge Base configuration")]
#[command(version)]
struct Cli {
    /// Configuration file path
    #[arg(long, short = 'c', global = true)]
    config: Option<PathBuf>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Show current configuration
    Show,

    /// Manage sources
    Sources {
        #[command(subcommand)]
        action: SourcesAction,
    },

    /// Manage lint rules
    Rules {
        #[command(subcommand)]
        action: RulesAction,
    },

    /// Manage lint configuration file
    Lint {
        #[command(subcommand)]
        action: LintAction,
    },
}

#[derive(Subcommand)]
enum SourcesAction {
    /// List all sources
    List {
        /// Filter by category
        #[arg(long)]
        category: Option<String>,
    },

    /// Add a new source
    Add {
        /// Category
        category: String,
        /// Source name
        name: String,
        /// URL or path
        #[arg(long)]
        url: Option<String>,
        /// Local path
        #[arg(long)]
        path: Option<String>,
        /// Parser type
        #[arg(long, default_value = "json")]
        parser: String,
    },

    /// Remove a source
    Remove {
        /// Category
        category: String,
        /// Source name
        name: String,
    },

    /// Enable a source
    Enable {
        /// Category
        category: String,
        /// Source name
        name: String,
    },

    /// Disable a source
    Disable {
        /// Category
        category: String,
        /// Source name
        name: String,
    },
}

#[derive(Subcommand)]
enum RulesAction {
    /// Enable a rule
    Enable {
        /// Rule ID
        id: String,
    },

    /// Disable a rule
    Disable {
        /// Rule ID
        id: String,
    },

    /// Set rule severity
    Severity {
        /// Rule ID
        id: String,
        /// New severity (error, warning, info)
        severity: String,
    },

    /// List rule configurations
    List,
}

#[derive(Subcommand)]
enum LintAction {
    /// Show lint configuration
    Show,

    /// Initialize .wixlintrc.json
    Init {
        /// Force overwrite
        #[arg(long, short = 'f')]
        force: bool,
    },

    /// Validate lint configuration
    Validate,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let config_path = cli.config.clone().unwrap_or_else(|| {
        PathBuf::from("config/sources.yaml")
    });

    match cli.command {
        Commands::Show => {
            if config_path.exists() {
                let config = SourcesConfig::load(&config_path)?;
                println!("Configuration: {}", config_path.display());
                println!("Version: {}", config.version);
                println!();
                println!("Sources:");
                for cat in config.categories() {
                    let count = config.list_sources(cat).len();
                    println!("  {}: {} sources", cat, count);
                }
                println!();
                println!("Total: {} sources", config.total_sources());
                println!();
                println!("Parsers:");
                for (name, parser) in &config.parsers {
                    println!("  {}: {}", name, parser.parser_type);
                }
                println!();
                println!("Harvest settings:");
                println!("  Cache: {}", config.harvest.cache_dir);
                println!("  Timeout: {}s", config.harvest.timeout_seconds);
                println!("  Retries: {}", config.harvest.retry_count);
            } else {
                eprintln!("Configuration not found: {}", config_path.display());
                std::process::exit(1);
            }
        }

        Commands::Sources { action } => {
            match action {
                SourcesAction::List { category } => {
                    let config = SourcesConfig::load(&config_path)?;

                    if let Some(cat) = category {
                        if let Some(sources) = config.get_sources(&cat) {
                            println!("[{}]", cat);
                            for (name, source) in sources {
                                let loc = source.url.as_deref()
                                    .or(source.path.as_deref())
                                    .unwrap_or("-");
                                println!("  {}: {}", name, loc);
                            }
                        }
                    } else {
                        for cat in config.categories() {
                            println!("[{}]", cat);
                            for name in config.list_sources(cat) {
                                println!("  {}", name);
                            }
                        }
                    }
                }

                SourcesAction::Add { category, name, url, path, parser } => {
                    println!("Adding source: {}/{}", category, name);
                    println!("URL: {:?}", url);
                    println!("Path: {:?}", path);
                    println!("Parser: {}", parser);
                    println!();
                    println!("Note: Manual editing of sources.yaml is recommended for now.");
                }

                SourcesAction::Remove { category, name } => {
                    println!("Removing source: {}/{}", category, name);
                    println!("Note: Manual editing of sources.yaml is recommended for now.");
                }

                SourcesAction::Enable { category, name } => {
                    println!("Enabling source: {}/{}", category, name);
                }

                SourcesAction::Disable { category, name } => {
                    println!("Disabling source: {}/{}", category, name);
                }
            }
        }

        Commands::Rules { action } => {
            let lint_path = PathBuf::from(".wixlintrc.json");

            match action {
                RulesAction::Enable { id } => {
                    let mut config = if lint_path.exists() {
                        LintConfig::load(&lint_path)?
                    } else {
                        LintConfig::default()
                    };

                    config.disabled_rules.retain(|r| r != &id);
                    if !config.enabled_rules.contains(&id) && !config.enabled_rules.is_empty() {
                        config.enabled_rules.push(id.clone());
                    }

                    let json = serde_json::to_string_pretty(&config).unwrap();
                    std::fs::write(&lint_path, json)?;
                    println!("Enabled rule: {}", id);
                }

                RulesAction::Disable { id } => {
                    let mut config = if lint_path.exists() {
                        LintConfig::load(&lint_path)?
                    } else {
                        LintConfig::default()
                    };

                    config.enabled_rules.retain(|r| r != &id);
                    if !config.disabled_rules.contains(&id) {
                        config.disabled_rules.push(id.clone());
                    }

                    let json = serde_json::to_string_pretty(&config).unwrap();
                    std::fs::write(&lint_path, json)?;
                    println!("Disabled rule: {}", id);
                }

                RulesAction::Severity { id, severity } => {
                    let valid = ["error", "warning", "info"];
                    if !valid.contains(&severity.as_str()) {
                        eprintln!("Invalid severity. Must be: error, warning, or info");
                        std::process::exit(1);
                    }

                    let mut config = if lint_path.exists() {
                        LintConfig::load(&lint_path)?
                    } else {
                        LintConfig::default()
                    };

                    config.severity_overrides.insert(id.clone(), severity.clone());

                    let json = serde_json::to_string_pretty(&config).unwrap();
                    std::fs::write(&lint_path, json)?;
                    println!("Set {} severity to: {}", id, severity);
                }

                RulesAction::List => {
                    if lint_path.exists() {
                        let config = LintConfig::load(&lint_path)?;

                        if !config.enabled_rules.is_empty() {
                            println!("Enabled rules:");
                            for rule in &config.enabled_rules {
                                println!("  {}", rule);
                            }
                        }

                        if !config.disabled_rules.is_empty() {
                            println!("Disabled rules:");
                            for rule in &config.disabled_rules {
                                println!("  {}", rule);
                            }
                        }

                        if !config.severity_overrides.is_empty() {
                            println!("Severity overrides:");
                            for (rule, severity) in &config.severity_overrides {
                                println!("  {}: {}", rule, severity);
                            }
                        }
                    } else {
                        println!("No lint configuration found.");
                        println!("Run 'config lint init' to create one.");
                    }
                }
            }
        }

        Commands::Lint { action } => {
            let lint_path = PathBuf::from(".wixlintrc.json");

            match action {
                LintAction::Show => {
                    if lint_path.exists() {
                        let content = std::fs::read_to_string(&lint_path)?;
                        println!("{}", content);
                    } else {
                        println!("No .wixlintrc.json found in current directory.");
                    }
                }

                LintAction::Init { force } => {
                    if lint_path.exists() && !force {
                        eprintln!(".wixlintrc.json already exists. Use --force to overwrite.");
                        std::process::exit(1);
                    }

                    let config = LintConfig::default();
                    let json = serde_json::to_string_pretty(&config).unwrap();
                    std::fs::write(&lint_path, json)?;
                    println!("Created .wixlintrc.json");
                }

                LintAction::Validate => {
                    if lint_path.exists() {
                        match LintConfig::load(&lint_path) {
                            Ok(_) => println!("Lint configuration is valid."),
                            Err(e) => {
                                eprintln!("Invalid configuration: {}", e);
                                std::process::exit(1);
                            }
                        }
                    } else {
                        eprintln!("No .wixlintrc.json found.");
                        std::process::exit(1);
                    }
                }
            }
        }
    }

    Ok(())
}
