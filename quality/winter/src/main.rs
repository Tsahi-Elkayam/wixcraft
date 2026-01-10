//! Linter CLI - Universal XML Linter Engine
//!
//! A fast, modular linter for XML files with plugin support.

use clap::{Parser, Subcommand, ValueEnum};
use colored::Colorize;
use glob::glob;
use std::path::PathBuf;
use std::process::Command;
use std::sync::Arc;
use winter::baseline::Baseline;
use winter::cache::{default_cache_path, hash_config, LintCache};
use winter::config::{ColorMode, Config, OutputFormat};
use winter::engine::Engine;
use winter::fixer::Fixer;
use winter::output::{
    AzureFormatter, CompactFormatter, GithubFormatter, GitlabFormatter, GroupedFormatter,
    JUnitFormatter, JsonFormatter, OutputFormatter, SarifFormatter, TextFormatter,
};
use winter::plugin_manager::PluginManager;
use winter::plugins::wix::WixPlugin;
use winter::plugins::xml::XmlPlugin;
use winter::watch::Watcher;
use winter::Plugin;

#[derive(Parser)]
#[command(
    name = "winter",
    version,
    about = "WiX Installer Linter",
    long_about = "A fast, modular linter for WiX XML files. 241 rules from wix-data database."
)]
struct Cli {
    /// Files or glob patterns to lint
    files: Vec<String>,

    /// Configuration file path
    #[arg(short, long)]
    config: Option<PathBuf>,

    /// Output format
    #[arg(short, long, value_enum, default_value = "text")]
    format: Format,

    /// Enable verbose output
    #[arg(short, long)]
    verbose: bool,

    /// Disable colored output
    #[arg(long)]
    no_color: bool,

    /// Number of parallel jobs (0 = auto)
    #[arg(short, long, default_value = "0")]
    jobs: usize,

    /// Disable specific rules (comma-separated)
    #[arg(long, value_delimiter = ',')]
    disable: Option<Vec<String>>,

    /// Only enable specific rules (comma-separated)
    #[arg(long, value_delimiter = ',')]
    select: Option<Vec<String>>,

    /// Select rules by prefix (e.g., 'WIX' selects all WIX* rules)
    #[arg(long, value_delimiter = ',')]
    extend: Option<Vec<String>>,

    /// Ignore rules by prefix (e.g., 'WIX' ignores all WIX* rules)
    #[arg(long, value_delimiter = ',')]
    ignore: Option<Vec<String>>,

    /// Minimum severity to report
    #[arg(long, value_enum)]
    min_severity: Option<MinSeverity>,

    /// Custom rules directory
    #[arg(long)]
    rules_dir: Option<PathBuf>,

    /// Custom plugins directory
    #[arg(long)]
    plugins_dir: Option<PathBuf>,

    /// Show statistics
    #[arg(long)]
    stats: bool,

    /// List available rules and exit
    #[arg(long)]
    list_rules: bool,

    /// List available plugins and exit
    #[arg(long)]
    list_plugins: bool,

    /// Enable cross-file validation (checks references across multiple files)
    #[arg(long)]
    cross_file: bool,

    /// Disable built-in WiX plugin
    #[arg(long)]
    no_wix: bool,

    /// Disable built-in XML plugin
    #[arg(long)]
    no_xml: bool,

    /// Auto-fix issues where possible (dry-run by default, use with --write to apply)
    #[arg(long)]
    fix: bool,

    /// Write fixes to files (requires --fix)
    #[arg(long, requires = "fix")]
    write: bool,

    /// Use baseline file to ignore existing issues (creates if not found)
    #[arg(long)]
    baseline: Option<PathBuf>,

    /// Update baseline with current issues (instead of filtering)
    #[arg(long, requires = "baseline")]
    update_baseline: bool,

    /// Enable caching for faster incremental runs
    #[arg(long)]
    cache: bool,

    /// Only lint files changed in git (requires git repository)
    #[arg(long)]
    changed: bool,

    /// Git ref to compare against (default: HEAD, use with --changed)
    #[arg(long, default_value = "HEAD", requires = "changed")]
    since: String,

    /// Watch files and re-lint on changes
    #[arg(long, short = 'w')]
    watch: bool,

    /// Clear screen before each lint run (use with --watch)
    #[arg(long, requires = "watch")]
    clear: bool,

    /// Enable preview/experimental rules
    #[arg(long)]
    preview: bool,

    /// Only enable rules from specific categories (comma-separated)
    #[arg(long, value_delimiter = ',')]
    categories: Option<Vec<String>>,

    /// Show detailed information about a specific rule
    #[arg(long)]
    explain: Option<String>,

    /// Show complexity metrics for files
    #[arg(long)]
    complexity: bool,

    /// Show diff of changes instead of applying fixes
    #[arg(long)]
    diff: bool,

    /// Show all fixes that would be applied
    #[arg(long)]
    show_fixes: bool,

    /// Include unsafe fixes (may change code behavior)
    #[arg(long)]
    unsafe_fixes: bool,

    /// Exit with 0 even if errors are found
    #[arg(long)]
    exit_zero: bool,

    /// Exit non-zero if any fixes were applied
    #[arg(long)]
    exit_non_zero_on_fix: bool,

    /// Show source context lines around errors
    #[arg(long, default_value = "0")]
    context: usize,

    /// Show per-rule timing statistics
    #[arg(long)]
    timing: bool,

    /// Show warnings when deprecated rules are used
    #[arg(long)]
    warn_deprecated: bool,

    /// Subcommands
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Show detailed information about a rule
    Explain {
        /// Rule ID to explain
        rule_id: String,
    },
    /// Initialize a configuration file
    Init {
        /// Preset to use (recommended, strict, minimal)
        #[arg(long, default_value = "recommended")]
        preset: String,

        /// Run interactive wizard
        #[arg(short, long)]
        interactive: bool,

        /// Output format (yaml, json)
        #[arg(long, default_value = "yaml")]
        output_format: String,
    },
}

#[derive(Clone, Copy, ValueEnum)]
enum Format {
    Text,
    Json,
    Sarif,
    Github,
    Compact,
    Grouped,
    Junit,
    Gitlab,
    Azure,
}

#[derive(Clone, Copy, ValueEnum)]
enum MinSeverity {
    Info,
    Warning,
    Error,
}

/// Helper function to print a rule in a consistent format
fn print_rule(rule: &winter::Rule) {
    let severity = match rule.severity {
        winter::Severity::Error => "error".red(),
        winter::Severity::Warning => "warning".yellow(),
        winter::Severity::Info => "info".blue(),
    };

    // Stability markers
    let stability_marker = match rule.stability {
        winter::RuleStability::Preview => " [preview]".yellow(),
        winter::RuleStability::Deprecated => " [deprecated]".red(),
        winter::RuleStability::Stable => "".normal(),
    };

    println!(
        "    {} [{}] ({}){}",
        rule.id.cyan(),
        severity,
        rule.category,
        stability_marker
    );
    if let Some(desc) = &rule.description {
        println!("      {}", desc);
    }
    if !rule.tags.is_empty() {
        println!("      Tags: {}", rule.tags.join(", "));
    }
}

/// Print detailed rule explanation
fn explain_rule(rule: &winter::Rule) {
    println!("{}", "Rule Details".bold());
    println!();
    println!("  {}: {}", "ID".bold(), rule.id.cyan());

    if let Some(name) = &rule.name {
        println!("  {}: {}", "Name".bold(), name);
    }

    println!(
        "  {}: {}",
        "Severity".bold(),
        match rule.severity {
            winter::Severity::Error => "error".red(),
            winter::Severity::Warning => "warning".yellow(),
            winter::Severity::Info => "info".blue(),
        }
    );

    println!("  {}: {}", "Category".bold(), rule.category);
    println!("  {}: {}", "Stability".bold(), rule.stability);

    if let Some(desc) = &rule.description {
        println!();
        println!("  {}", "Description".bold());
        println!("  {}", desc);
    }

    if let Some(rationale) = &rule.rationale {
        println!();
        println!("  {}", "Rationale".bold());
        println!("  {}", rationale);
    }

    if let Some(bad) = &rule.example_bad {
        println!();
        println!("  {} {}", "Example".bold(), "(incorrect)".red());
        for line in bad.lines() {
            println!("    {}", line);
        }
    }

    if let Some(good) = &rule.example_good {
        println!();
        println!("  {} {}", "Example".bold(), "(correct)".green());
        for line in good.lines() {
            println!("    {}", line);
        }
    }

    if let Some(fix) = &rule.fix {
        println!();
        println!("  {}", "Auto-fix Available".bold());
        if let Some(desc) = &fix.description {
            println!("  {}", desc);
        }
    }

    if let Some(docs) = &rule.docs {
        println!();
        println!("  {}: {}", "Documentation".bold(), docs.blue());
    }

    if !rule.tags.is_empty() {
        println!();
        println!("  {}: {}", "Tags".bold(), rule.tags.join(", "));
    }

    if !rule.related.is_empty() {
        println!();
        println!("  {}: {}", "Related Rules".bold(), rule.related.join(", "));
    }
}

/// Handle the explain command/flag
fn handle_explain(rule_id: &str, cli: &Cli) {
    // Build plugins to get rules
    let wix_plugin = if !cli.no_wix {
        Some(WixPlugin::new())
    } else {
        None
    };

    let xml_plugin = if !cli.no_xml {
        Some(XmlPlugin::new())
    } else {
        None
    };

    // Search for the rule
    let mut found = false;

    if let Some(ref wix) = wix_plugin {
        for rule in wix.rules() {
            if rule.id == rule_id {
                explain_rule(rule);
                found = true;
                break;
            }
        }
    }

    if !found {
        if let Some(ref xml) = xml_plugin {
            for rule in xml.rules() {
                if rule.id == rule_id {
                    explain_rule(rule);
                    found = true;
                    break;
                }
            }
        }
    }

    if !found {
        eprintln!("{}: Rule '{}' not found", "error".red().bold(), rule_id);
        eprintln!();
        eprintln!("Use {} to see all available rules", "--list-rules".cyan());
        std::process::exit(1);
    }
}

/// Prompt user for input with a default value
fn prompt_with_default(question: &str, default: &str) -> String {
    use std::io::{self, Write};

    print!("{} [{}]: ", question, default);
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    let input = input.trim();

    if input.is_empty() {
        default.to_string()
    } else {
        input.to_string()
    }
}

/// Prompt user for yes/no with a default value
fn prompt_yes_no(question: &str, default: bool) -> bool {
    let default_str = if default { "Y/n" } else { "y/N" };
    let response = prompt_with_default(question, default_str);
    match response.to_lowercase().as_str() {
        "y" | "yes" | "Y/n" => true,
        "n" | "no" | "y/N" => false,
        _ => default,
    }
}

/// Prompt user to select from options
fn prompt_options(question: &str, options: &[&str], default: usize) -> usize {
    use std::io::{self, Write};

    println!("{}", question);
    for (i, opt) in options.iter().enumerate() {
        let marker = if i == default { ">" } else { " " };
        println!("  {} {}. {}", marker, i + 1, opt);
    }
    print!("Choice [{}]: ", default + 1);
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    let input = input.trim();

    if input.is_empty() {
        default
    } else {
        input
            .parse::<usize>()
            .unwrap_or(default + 1)
            .saturating_sub(1)
            .min(options.len() - 1)
    }
}

/// Handle the init command
fn handle_init(preset: &str, interactive: bool, output_format: &str) {
    let config = if interactive {
        run_interactive_wizard()
    } else {
        match Config::preset(preset) {
            Some(c) => c,
            None => {
                eprintln!(
                    "{}: Unknown preset '{}'. Available: recommended, strict, minimal",
                    "error".red().bold(),
                    preset
                );
                std::process::exit(1);
            }
        }
    };

    // Determine filename based on format
    let filename = if output_format == "json" {
        ".linterrc.json"
    } else {
        ".linterrc.yaml"
    };

    if std::path::Path::new(filename).exists() {
        eprintln!(
            "{}: {} already exists. Remove it first to reinitialize.",
            "error".red().bold(),
            filename
        );
        std::process::exit(1);
    }

    let content = if output_format == "json" {
        let json = serde_json::to_string_pretty(&config).unwrap_or_default();
        format!(
            "{{\n  \"$schema\": \"https://winter.dev/schema/config.json\",\n{}",
            &json[1..] // Remove leading '{'
        )
    } else {
        let yaml = serde_yaml::to_string(&config).unwrap_or_default();
        format!(
            "# Linter configuration\n# Generated with: linter init{}\n\n{}",
            if interactive { " --interactive" } else { "" },
            yaml
        )
    };

    if let Err(e) = std::fs::write(filename, content) {
        eprintln!(
            "{}: Failed to write {}: {}",
            "error".red().bold(),
            filename,
            e
        );
        std::process::exit(1);
    }

    println!("{} Created {}", "success".green().bold(), filename);
    println!();
    println!("Next steps:");
    println!("  1. Review and customize the configuration");
    println!("  2. Run {} to lint your files", "linter **/*.wxs".cyan());
}

/// Run the interactive configuration wizard
fn run_interactive_wizard() -> Config {
    println!("{}", "Winter Linter Configuration Wizard".bold().cyan());
    println!("{}", "=".repeat(40));
    println!();

    // 1. Choose preset as base
    let presets = [
        "recommended (balanced defaults)",
        "strict (all rules)",
        "minimal (errors only)",
    ];
    let preset_idx = prompt_options("Which preset do you want to start with?", &presets, 0);
    let preset_name = match preset_idx {
        0 => "recommended",
        1 => "strict",
        _ => "minimal",
    };
    let mut config = Config::preset(preset_name).unwrap_or_default();
    println!();

    // 2. Enable preview rules?
    config.preview = prompt_yes_no("Enable preview/experimental rules?", false);
    println!();

    // 3. Output format
    let formats = ["text", "json", "sarif", "github"];
    let format_idx = prompt_options("Preferred output format?", &formats, 0);
    config.output.format = match format_idx {
        1 => OutputFormat::Json,
        2 => OutputFormat::Sarif,
        3 => OutputFormat::Github,
        _ => OutputFormat::Text,
    };
    println!();

    // 4. Parallel processing
    config.engine.parallel = prompt_yes_no("Enable parallel processing?", true);
    if config.engine.parallel {
        let jobs_str = prompt_with_default("Number of parallel jobs (0 = auto)?", "0");
        config.engine.jobs = jobs_str.parse().unwrap_or(0);
    }
    println!();

    // 5. Categories
    println!("Which rule categories do you want to enable?");
    let cat_correctness = prompt_yes_no("  - correctness (errors)?", true);
    let cat_suspicious = prompt_yes_no("  - suspicious (likely bugs)?", true);
    let cat_style = prompt_yes_no("  - style (formatting)?", true);
    let cat_perf = prompt_yes_no("  - perf (performance)?", preset_idx == 1);
    let cat_pedantic = prompt_yes_no("  - pedantic (strict)?", preset_idx == 1);

    config.categories = Vec::new();
    if cat_correctness {
        config.categories.push("correctness".to_string());
    }
    if cat_suspicious {
        config.categories.push("suspicious".to_string());
    }
    if cat_style {
        config.categories.push("style".to_string());
    }
    if cat_perf {
        config.categories.push("perf".to_string());
    }
    if cat_pedantic {
        config.categories.push("pedantic".to_string());
    }
    println!();

    // 6. File patterns
    println!("File patterns (press Enter to use defaults):");
    let include = prompt_with_default("  Include patterns?", "**/*.wxs, **/*.wxi");
    if include != "**/*.wxs, **/*.wxi" {
        config.files.include = include.split(',').map(|s| s.trim().to_string()).collect();
    }
    let exclude = prompt_with_default("  Exclude patterns?", "**/generated/**, **/*.g.wxs");
    if exclude != "**/generated/**, **/*.g.wxs" {
        config.files.exclude = exclude.split(',').map(|s| s.trim().to_string()).collect();
    }
    println!();

    println!("{}", "Configuration complete!".green().bold());
    println!();

    config
}

fn main() {
    // Initialize logger
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("warn")).init();

    let cli = Cli::parse();

    // Handle --no-color
    if cli.no_color {
        colored::control::set_override(false);
    }

    // Handle subcommands
    if let Some(cmd) = &cli.command {
        match cmd {
            Commands::Explain { rule_id } => {
                handle_explain(rule_id, &cli);
                return;
            }
            Commands::Init {
                preset,
                interactive,
                output_format,
            } => {
                handle_init(preset, *interactive, output_format);
                return;
            }
        }
    }

    // Handle --explain flag (alternative to subcommand)
    if let Some(rule_id) = &cli.explain {
        handle_explain(rule_id, &cli);
        return;
    }

    // Load configuration
    let mut config = if let Some(config_path) = &cli.config {
        Config::load(config_path).unwrap_or_else(|e| {
            eprintln!("{}: Failed to load config: {}", "error".red().bold(), e);
            std::process::exit(1);
        })
    } else {
        Config::load_default().unwrap_or_default()
    };

    // Merge CLI arguments
    let format = match cli.format {
        Format::Text => OutputFormat::Text,
        Format::Json => OutputFormat::Json,
        Format::Sarif => OutputFormat::Sarif,
        Format::Github => OutputFormat::Github,
        Format::Compact => OutputFormat::Compact,
        Format::Grouped => OutputFormat::Grouped,
        Format::Junit => OutputFormat::Junit,
        Format::Gitlab => OutputFormat::Gitlab,
        Format::Azure => OutputFormat::Azure,
    };

    // Set preview mode from CLI
    if cli.preview {
        config.preview = true;
    }

    // Set categories from CLI
    if let Some(cats) = &cli.categories {
        config.categories = cats.clone();
    }

    config.merge_cli(
        Some(format),
        Some(cli.verbose),
        Some(cli.jobs),
        cli.disable,
        cli.select,
    );

    // Apply prefix-based rule selection
    if let Some(extend_prefixes) = cli.extend {
        config.add_extend_prefixes(extend_prefixes);
    }
    if let Some(ignore_prefixes) = cli.ignore {
        config.add_ignore_prefixes(ignore_prefixes);
    }

    // Initialize plugin manager for dynamic plugins
    let mut plugin_manager = PluginManager::new();

    // Add custom plugins directory if specified
    if let Some(plugins_dir) = &cli.plugins_dir {
        plugin_manager.add_search_path(plugins_dir.clone());
    }

    // Load dynamic plugins
    let load_results = plugin_manager.load_all();
    let mut dynamic_plugin_count = 0;
    for result in &load_results {
        match result {
            Ok(id) => {
                dynamic_plugin_count += 1;
                if cli.verbose {
                    eprintln!("Loaded plugin: {}", id.cyan());
                }
            }
            Err(e) => {
                if cli.verbose {
                    eprintln!("{}: {}", "warning".yellow(), e);
                }
            }
        }
    }

    // Create built-in plugins
    let mut wix_plugin = if !cli.no_wix {
        Some(WixPlugin::new())
    } else {
        None
    };

    let xml_plugin = if !cli.no_xml {
        Some(XmlPlugin::new())
    } else {
        None
    };

    if cli.verbose {
        if let Some(ref wix) = wix_plugin {
            if wix.is_using_db_rules() {
                eprintln!(
                    "WiX plugin: {} rules from wix-data database",
                    wix.rules().len()
                );
            } else {
                eprintln!("WiX plugin: {} built-in rules", wix.rules().len());
            }
        }
        if let Some(ref xml) = xml_plugin {
            eprintln!("XML plugin: {} rules", xml.rules().len());
        }
        if dynamic_plugin_count > 0 {
            eprintln!("Dynamic plugins: {}", dynamic_plugin_count);
        }
    }

    // Load custom rules for WiX plugin if specified
    if let Some(rules_dir) = &cli.rules_dir {
        if let Some(ref mut wix) = wix_plugin {
            match wix.load_rules(rules_dir) {
                Ok(count) => {
                    if cli.verbose {
                        eprintln!("Loaded {} custom rules from {}", count, rules_dir.display());
                    }
                }
                Err(e) => {
                    eprintln!(
                        "{}: Failed to load rules from {}: {}",
                        "warning".yellow().bold(),
                        rules_dir.display(),
                        e
                    );
                }
            }
        }
    }

    // Show deprecation warnings for enabled rules
    if cli.warn_deprecated {
        let mut deprecated_count = 0;

        if let Some(ref wix) = wix_plugin {
            for rule in wix.rules() {
                if rule.is_deprecated() && config.is_rule_enabled(&rule.id) {
                    if let Some(warning) = rule.deprecation_warning() {
                        eprintln!("{}: {}", "deprecated".yellow(), warning);
                        deprecated_count += 1;
                    }
                }
            }
        }

        if let Some(ref xml) = xml_plugin {
            for rule in xml.rules() {
                if rule.is_deprecated() && config.is_rule_enabled(&rule.id) {
                    if let Some(warning) = rule.deprecation_warning() {
                        eprintln!("{}: {}", "deprecated".yellow(), warning);
                        deprecated_count += 1;
                    }
                }
            }
        }

        for plugin_id in plugin_manager.list_plugins() {
            if let Some(plugin) = plugin_manager.get_plugin(plugin_id) {
                for rule in plugin.rules() {
                    if rule.is_deprecated() && config.is_rule_enabled(&rule.id) {
                        if let Some(warning) = rule.deprecation_warning() {
                            eprintln!("{}: {}", "deprecated".yellow(), warning);
                            deprecated_count += 1;
                        }
                    }
                }
            }
        }

        if deprecated_count > 0 {
            eprintln!();
            eprintln!(
                "{}: {} deprecated rule(s) are enabled",
                "warning".yellow().bold(),
                deprecated_count
            );
            eprintln!();
        }
    }

    // Handle --list-plugins
    if cli.list_plugins {
        println!("{}", "Available plugins:".bold());
        println!();

        // Built-in plugins
        println!("  {}", "Built-in:".cyan());
        if let Some(ref wix) = wix_plugin {
            println!(
                "    {} - {} ({} rules)",
                "wix".green(),
                wix.description(),
                wix.rules().len()
            );
        }
        if let Some(ref xml) = xml_plugin {
            println!(
                "    {} - {} ({} rules)",
                "xml".green(),
                xml.description(),
                xml.rules().len()
            );
        }

        // Dynamic plugins
        if !plugin_manager.list_plugins().is_empty() {
            println!();
            println!("  {}", "Dynamic:".cyan());
            for plugin_id in plugin_manager.list_plugins() {
                if let Some(plugin) = plugin_manager.get_plugin(plugin_id) {
                    println!(
                        "    {} - {} ({} rules)",
                        plugin_id.green(),
                        plugin.description(),
                        plugin.rules().len()
                    );
                    let exts = plugin.extensions_owned();
                    if !exts.is_empty() {
                        println!("      Extensions: {}", exts.join(", "));
                    }
                }
            }
        }
        println!();
        return;
    }

    // Validate that files are provided (unless using --list-rules or --list-plugins)
    if cli.files.is_empty() && !cli.list_rules && !cli.list_plugins {
        eprintln!("{}: No files specified", "error".red().bold());
        eprintln!();
        eprintln!("Usage: winter [OPTIONS] <FILES>...");
        eprintln!();
        eprintln!("For more information, try '--help'");
        std::process::exit(2);
    }

    // Handle --list-rules
    if cli.list_rules {
        println!("{}", "Available rules:".bold());
        println!();

        // WiX plugin rules
        if let Some(ref wix) = wix_plugin {
            let source = if wix.is_using_db_rules() {
                "wix-data database"
            } else {
                "built-in"
            };
            println!(
                "  {} ({} from {}):",
                "WiX Plugin".cyan(),
                wix.rules().len(),
                source
            );
            for rule in wix.rules() {
                print_rule(rule);
            }
            println!();
        }

        // XML plugin rules
        if let Some(ref xml) = xml_plugin {
            println!("  {} ({} rules):", "XML Plugin".cyan(), xml.rules().len());
            for rule in xml.rules() {
                print_rule(rule);
            }
            println!();
        }

        // Dynamic plugin rules
        for plugin_id in plugin_manager.list_plugins() {
            if let Some(plugin) = plugin_manager.get_plugin(plugin_id) {
                println!(
                    "  {} ({} rules):",
                    format!("{} Plugin", plugin_id).cyan(),
                    plugin.rules().len()
                );
                for rule in plugin.rules() {
                    print_rule(rule);
                }
                println!();
            }
        }

        return;
    }

    // Create engine
    let mut engine = Engine::new(config.clone());

    // Set context lines if specified
    if cli.context > 0 {
        engine.set_context_lines(cli.context);
    }

    // Register built-in plugins
    if let Some(wix) = wix_plugin {
        engine.register_plugin(Arc::new(wix));
    }
    if let Some(xml) = xml_plugin {
        engine.register_plugin(Arc::new(xml));
    }

    // Register dynamic plugins
    for plugin in plugin_manager.all_plugins() {
        engine.register_plugin(plugin);
    }

    // Expand glob patterns
    let mut files: Vec<PathBuf> = Vec::new();
    for pattern in &cli.files {
        match glob(pattern) {
            Ok(paths) => {
                for entry in paths.flatten() {
                    if entry.is_file() {
                        files.push(entry);
                    }
                }
            }
            Err(e) => {
                eprintln!(
                    "{}: Invalid pattern '{}': {}",
                    "error".red().bold(),
                    pattern,
                    e
                );
                std::process::exit(1);
            }
        }
    }

    if files.is_empty() {
        eprintln!("{}: No files found to lint", "error".red().bold());
        std::process::exit(1);
    }

    // Filter to git-changed files if requested
    if cli.changed {
        let changed_files = get_git_changed_files(&cli.since);
        match changed_files {
            Ok(git_files) => {
                let git_set: std::collections::HashSet<_> = git_files.iter().collect();
                let original_count = files.len();
                files.retain(|f| {
                    f.canonicalize()
                        .ok()
                        .map(|p| git_set.contains(&p))
                        .unwrap_or(false)
                });
                if cli.verbose {
                    eprintln!(
                        "Filtered to {} changed files (from {} total)",
                        files.len(),
                        original_count
                    );
                }
            }
            Err(e) => {
                eprintln!(
                    "{}: Failed to get git changed files: {}",
                    "warning".yellow(),
                    e
                );
            }
        }
    }

    if files.is_empty() {
        if cli.verbose {
            eprintln!("No files to lint (all filtered out)");
        }
        std::process::exit(0);
    }

    if cli.verbose {
        eprintln!("Linting {} files...", files.len());
        if cli.cross_file {
            eprintln!("Cross-file validation enabled");
        }
    }

    // Load cache if enabled
    let mut cache = if cli.cache {
        let cache_path = default_cache_path();
        if cli.verbose {
            eprintln!("Using cache: {}", cache_path.display());
        }
        LintCache::load(&cache_path).unwrap_or_else(|_| LintCache::new())
    } else {
        LintCache::new()
    };

    // Set config hash for cache invalidation
    if cli.cache {
        cache.set_config_hash(&hash_config(&config));
    }

    // Run linting
    let mut result = if cli.cross_file {
        engine.lint_with_cross_file(&files)
    } else {
        engine.lint(&files)
    };

    // Save cache
    if cli.cache {
        let cache_path = default_cache_path();
        if let Err(e) = cache.save(&cache_path) {
            if cli.verbose {
                eprintln!("{}: Failed to save cache: {}", "warning".yellow(), e);
            }
        }
    }

    // Filter by minimum severity
    if let Some(min_sev) = cli.min_severity {
        let min = match min_sev {
            MinSeverity::Info => winter::Severity::Info,
            MinSeverity::Warning => winter::Severity::Warning,
            MinSeverity::Error => winter::Severity::Error,
        };
        result.diagnostics.retain(|d| d.severity >= min);

        // Recalculate counts
        result.error_count = result
            .diagnostics
            .iter()
            .filter(|d| d.severity == winter::Severity::Error)
            .count();
        result.warning_count = result
            .diagnostics
            .iter()
            .filter(|d| d.severity == winter::Severity::Warning)
            .count();
        result.info_count = result
            .diagnostics
            .iter()
            .filter(|d| d.severity == winter::Severity::Info)
            .count();
    }

    // Handle baseline
    if let Some(baseline_path) = &cli.baseline {
        if cli.update_baseline {
            // Update baseline with current issues
            let mut baseline = Baseline::load(baseline_path).unwrap_or_else(|_| Baseline::new());
            baseline.add_diagnostics(&result.diagnostics);
            if let Err(e) = baseline.save(baseline_path) {
                eprintln!("{}: Failed to save baseline: {}", "error".red().bold(), e);
            } else if cli.verbose {
                eprintln!("Updated baseline with {} issues", baseline.issue_count());
            }
        } else {
            // Filter out baselined issues
            match Baseline::load(baseline_path) {
                Ok(baseline) => {
                    let original_count = result.diagnostics.len();
                    result.diagnostics = baseline.filter_diagnostics(result.diagnostics);
                    if cli.verbose {
                        eprintln!(
                            "Filtered out {} baselined issues",
                            original_count - result.diagnostics.len()
                        );
                    }
                    // Recalculate counts
                    result.error_count = result
                        .diagnostics
                        .iter()
                        .filter(|d| d.severity == winter::Severity::Error)
                        .count();
                    result.warning_count = result
                        .diagnostics
                        .iter()
                        .filter(|d| d.severity == winter::Severity::Warning)
                        .count();
                    result.info_count = result
                        .diagnostics
                        .iter()
                        .filter(|d| d.severity == winter::Severity::Info)
                        .count();
                }
                Err(_) => {
                    // Create new baseline file
                    let mut baseline = Baseline::new();
                    baseline.add_diagnostics(&result.diagnostics);
                    if let Err(e) = baseline.save(baseline_path) {
                        eprintln!("{}: Failed to create baseline: {}", "error".red().bold(), e);
                    } else {
                        eprintln!(
                            "Created baseline with {} issues at {}",
                            baseline.issue_count(),
                            baseline_path.display()
                        );
                    }
                }
            }
        }
    }

    // Handle auto-fix
    // Handle fix-related flags
    let mut fixes_applied = 0;
    if cli.fix || cli.diff || cli.show_fixes {
        let dry_run = !cli.write;
        let mut fixer = Fixer::new(dry_run);

        // Configure fixer mode
        if cli.diff {
            fixer = fixer.with_diff_mode();
        } else if cli.show_fixes {
            fixer = fixer.with_show_only();
        }

        // Handle unsafe fixes
        if cli.unsafe_fixes {
            fixer = fixer.with_unsafe_fixes(true);
        }

        fixer.collect_from_diagnostics(&result.diagnostics);

        if cli.show_fixes {
            // Just show what fixes are available
            println!("{}", fixer.format_fixes());
        } else if fixer.pending_count() > 0 {
            let fix_result = fixer.apply_all();
            fixes_applied = fix_result.fixes_applied;

            if cli.diff {
                // Show diff output
                println!("{}", fixer.format_diffs(&fix_result));
                if fix_result.fixes_skipped > 0 {
                    eprintln!(
                        "{}: {} unsafe fixes skipped (use --unsafe-fixes to include)",
                        "note".blue(),
                        fix_result.fixes_skipped
                    );
                }
            } else if dry_run {
                eprintln!(
                    "{}: {} fixes available ({} safe, {} unsafe)",
                    "dry-run".cyan(),
                    fixer.pending_count(),
                    fix_result.safe_fixes_applied,
                    fix_result.unsafe_fixes_applied
                );
                if fix_result.fixes_skipped > 0 {
                    eprintln!(
                        "{}: {} unsafe fixes skipped (use --unsafe-fixes to include)",
                        "note".blue(),
                        fix_result.fixes_skipped
                    );
                }
                eprintln!("Use --write to apply fixes");
            } else {
                eprintln!(
                    "Applied {} fixes to {} files ({} safe, {} unsafe)",
                    fix_result.fixes_applied,
                    fix_result.files_modified,
                    fix_result.safe_fixes_applied,
                    fix_result.unsafe_fixes_applied
                );
                if fix_result.fixes_failed > 0 {
                    eprintln!(
                        "{}: {} fixes failed",
                        "warning".yellow(),
                        fix_result.fixes_failed
                    );
                }
                if fix_result.fixes_skipped > 0 {
                    eprintln!(
                        "{}: {} unsafe fixes skipped (use --unsafe-fixes to include)",
                        "note".blue(),
                        fix_result.fixes_skipped
                    );
                }
            }
        } else if cli.verbose {
            eprintln!("No auto-fixes available");
        }
    }

    // Create formatter
    let formatter: Box<dyn OutputFormatter> = match config.output.format {
        OutputFormat::Text => {
            let mut f = TextFormatter::new();
            if cli.no_color || config.output.color == ColorMode::Never {
                f = f.without_color();
            }
            f.show_stats = cli.stats || config.output.statistics;
            Box::new(f)
        }
        OutputFormat::Compact => Box::new(CompactFormatter::new()),
        OutputFormat::Grouped => {
            let mut f = GroupedFormatter::new();
            if cli.no_color || config.output.color == ColorMode::Never {
                f = f.without_colors();
            }
            Box::new(f)
        }
        OutputFormat::Json => Box::new(JsonFormatter::new().pretty()),
        OutputFormat::Sarif => Box::new(SarifFormatter::new("winter", env!("CARGO_PKG_VERSION"))),
        OutputFormat::Github => Box::new(GithubFormatter::new()),
        OutputFormat::Junit => Box::new(JUnitFormatter::new()),
        OutputFormat::Gitlab => Box::new(GitlabFormatter::new()),
        OutputFormat::Azure => Box::new(AzureFormatter::new()),
    };

    // Output results
    let output = formatter.format(&result);
    print!("{}", output);

    // Show timing statistics if requested
    if cli.timing {
        eprintln!();
        eprintln!("{}", result.format_timings());
    }

    // Watch mode
    if cli.watch {
        eprintln!();
        eprintln!(
            "{} Watching for changes... (press Ctrl+C to stop)",
            "[watch]".cyan().bold()
        );

        // Get extensions from plugins
        let extensions: Vec<&str> = vec!["wxs", "wxi", "xml"];

        // Create watcher
        let watch_paths: Vec<PathBuf> = files
            .iter()
            .map(|f| {
                f.parent()
                    .unwrap_or(std::path::Path::new("."))
                    .to_path_buf()
            })
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();

        match Watcher::new(&watch_paths, &extensions) {
            Ok(watcher) => {
                loop {
                    if let Some(event) = watcher.wait() {
                        if cli.clear {
                            print!("\x1B[2J\x1B[1;1H"); // Clear screen
                        }

                        eprintln!();
                        eprintln!(
                            "{} Files changed: {:?}",
                            "[watch]".cyan().bold(),
                            event
                                .paths
                                .iter()
                                .map(|p| p.file_name().unwrap_or_default().to_string_lossy())
                                .collect::<Vec<_>>()
                        );
                        eprintln!();

                        // Re-lint changed files
                        let lint_files: Vec<PathBuf> =
                            event.paths.into_iter().filter(|p| p.exists()).collect();

                        if !lint_files.is_empty() {
                            let result = engine.lint(&lint_files);

                            let output = formatter.format(&result);
                            print!("{}", output);
                        }

                        eprintln!();
                        eprintln!("{} Watching for changes...", "[watch]".cyan().bold());
                    }
                }
            }
            Err(e) => {
                eprintln!(
                    "{}: Failed to start file watcher: {}",
                    "error".red().bold(),
                    e
                );
                std::process::exit(1);
            }
        }
    }

    // Exit with appropriate code
    let exit_code = if cli.exit_zero {
        0
    } else if cli.exit_non_zero_on_fix && fixes_applied > 0 {
        2 // Special exit code for "fixes were applied"
    } else {
        result.exit_code()
    };
    std::process::exit(exit_code);
}

/// Get list of files changed in git compared to a ref
fn get_git_changed_files(since: &str) -> Result<Vec<PathBuf>, String> {
    // Get the git root directory
    let git_root = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .map_err(|e| format!("Failed to run git: {}", e))?;

    if !git_root.status.success() {
        return Err("Not in a git repository".to_string());
    }

    let root = String::from_utf8_lossy(&git_root.stdout).trim().to_string();
    let root_path = PathBuf::from(&root);

    // Get changed files (staged and unstaged)
    let diff_output = Command::new("git")
        .args(["diff", "--name-only", since])
        .output()
        .map_err(|e| format!("Failed to run git diff: {}", e))?;

    let mut files: Vec<PathBuf> = Vec::new();

    if diff_output.status.success() {
        for line in String::from_utf8_lossy(&diff_output.stdout).lines() {
            if !line.is_empty() {
                files.push(root_path.join(line));
            }
        }
    }

    // Also get staged files
    let staged_output = Command::new("git")
        .args(["diff", "--cached", "--name-only"])
        .output()
        .map_err(|e| format!("Failed to run git diff --cached: {}", e))?;

    if staged_output.status.success() {
        for line in String::from_utf8_lossy(&staged_output.stdout).lines() {
            if !line.is_empty() {
                let path = root_path.join(line);
                if !files.contains(&path) {
                    files.push(path);
                }
            }
        }
    }

    // Get untracked files
    let untracked_output = Command::new("git")
        .args(["ls-files", "--others", "--exclude-standard"])
        .output()
        .map_err(|e| format!("Failed to run git ls-files: {}", e))?;

    if untracked_output.status.success() {
        for line in String::from_utf8_lossy(&untracked_output.stdout).lines() {
            if !line.is_empty() {
                let path = root_path.join(line);
                if !files.contains(&path) {
                    files.push(path);
                }
            }
        }
    }

    // Canonicalize all paths for matching
    let canonical_files: Vec<PathBuf> = files
        .into_iter()
        .filter_map(|f| f.canonicalize().ok())
        .collect();

    Ok(canonical_files)
}
