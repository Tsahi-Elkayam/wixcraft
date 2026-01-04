//! wix-lint CLI entry point

use clap::Parser;
use miette::{IntoDiagnostic, Result};
use rayon::prelude::*;
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;
use wix_lint::{Config, Diagnostic, LintEngine, LintStatistics, PluginManager, RuleLoader, Severity, WixDocument};

#[derive(Parser, Debug)]
#[command(name = "wix-lint")]
#[command(author, version, about = "A linter for WiX XML files", long_about = None)]
struct Cli {
    /// WiX files to lint (.wxs, .wxi). Use "-" for stdin.
    #[arg(required = true)]
    files: Vec<PathBuf>,

    /// Output format
    #[arg(short, long, value_enum, default_value = "text")]
    format: OutputFormat,

    /// Config file path (default: auto-detect .wixlintrc.json)
    #[arg(short, long)]
    config: Option<PathBuf>,

    /// Enable specific rule (can be used multiple times)
    #[arg(short, long = "rule", value_name = "RULE")]
    rules: Vec<String>,

    /// Disable specific rule (can be used multiple times)
    #[arg(short, long = "ignore", value_name = "RULE")]
    ignore: Vec<String>,

    /// Add to ignored rules without replacing (can be used multiple times)
    #[arg(long = "extend-ignore", value_name = "RULE")]
    extend_ignore: Vec<String>,

    /// Minimum severity level to report
    #[arg(short, long, value_enum)]
    severity: Option<SeverityFilter>,

    /// Only output errors (equivalent to --severity=error)
    #[arg(short, long)]
    quiet: bool,

    /// Show statistics at the end
    #[arg(long)]
    statistics: bool,

    /// Only show count of errors (no details)
    #[arg(long)]
    count: bool,

    /// Verbose output
    #[arg(short, long)]
    verbose: bool,

    /// Path to wix-data directory (for rule definitions)
    #[arg(long, env = "WIX_DATA_PATH")]
    wix_data: Option<PathBuf>,

    /// Exclude files matching pattern (can be used multiple times)
    #[arg(long = "exclude", value_name = "PATTERN")]
    exclude: Vec<String>,

    /// Only lint files matching pattern (can be used multiple times)
    #[arg(long = "filename", value_name = "PATTERN")]
    filename: Vec<String>,

    /// Stop after this many errors (0 = unlimited)
    #[arg(long = "max-errors", value_name = "N", default_value = "0")]
    max_errors: usize,

    /// Number of parallel jobs (0 = auto, 1 = sequential)
    #[arg(short = 'j', long = "jobs", value_name = "N")]
    jobs: Option<usize>,

    /// Only lint lines changed in git diff
    #[arg(long)]
    diff: bool,

    /// Show total count of each error code
    #[arg(long = "show-pep8-errors")]
    show_pep8_errors: bool,

    /// Additional plugin path (can be used multiple times)
    #[arg(long = "plugin-path", value_name = "PATH")]
    plugin_paths: Vec<PathBuf>,

    /// Disable plugin loading
    #[arg(long = "no-plugins")]
    no_plugins: bool,
}

#[derive(clap::ValueEnum, Clone, Debug, Default)]
enum OutputFormat {
    #[default]
    Text,
    Json,
    Sarif,
}

#[derive(clap::ValueEnum, Clone, Debug)]
enum SeverityFilter {
    Error,
    Warning,
    Info,
}

fn main() -> ExitCode {
    match run() {
        Ok(code) => code,
        Err(e) => {
            eprintln!("Error: {e:?}");
            ExitCode::from(2)
        }
    }
}

fn run() -> Result<ExitCode> {
    let cli = Cli::parse();

    // Determine wix-data path
    let wix_data_path = cli.wix_data.clone().unwrap_or_else(|| {
        PathBuf::from("src/core/wix-data")
    });

    // Load or create configuration
    let mut config = if let Some(ref config_path) = cli.config {
        Config::from_file(config_path).into_diagnostic()?
    } else {
        let start_dir = std::env::current_dir().into_diagnostic()?;
        match Config::find_and_load(&start_dir) {
            Ok(Some((path, cfg))) => {
                if cli.verbose {
                    eprintln!("Using config: {}", path.display());
                }
                cfg
            }
            Ok(None) => Config::default(),
            Err(e) => {
                eprintln!("Warning: Failed to load config: {}", e);
                Config::default()
            }
        }
    };

    // Merge CLI options
    let cli_severity = if cli.quiet {
        Some(Severity::Error)
    } else {
        cli.severity.map(|s| match s {
            SeverityFilter::Error => Severity::Error,
            SeverityFilter::Warning => Severity::Warning,
            SeverityFilter::Info => Severity::Info,
        })
    };

    config.merge_cli(wix_lint::config::CliOptions {
        enabled_rules: if cli.rules.is_empty() { None } else { Some(cli.rules) },
        disabled_rules: cli.ignore,
        extend_ignore: cli.extend_ignore,
        min_severity: cli_severity,
        verbose: cli.verbose,
        statistics: cli.statistics || cli.show_pep8_errors,
        max_errors: if cli.max_errors > 0 { Some(cli.max_errors) } else { None },
        jobs: cli.jobs,
    });

    // Load rules from wix-data
    let loader = RuleLoader::new(&wix_data_path);
    let mut rules = loader.load_all().into_diagnostic()?;

    if cli.verbose {
        eprintln!("Loaded {} rules from {}", rules.len(), wix_data_path.display());
    }

    // Load plugin rules
    if !cli.no_plugins {
        let mut plugin_manager = PluginManager::new();
        for path in &cli.plugin_paths {
            plugin_manager.add_search_path(path.clone());
        }
        if let Err(e) = plugin_manager.load_all() {
            if cli.verbose {
                eprintln!("Warning: Failed to load plugins: {}", e);
            }
        }
        let plugin_rules = plugin_manager.get_all_rules();
        if cli.verbose && plugin_manager.plugin_count() > 0 {
            eprintln!(
                "Loaded {} rules from {} plugin(s)",
                plugin_rules.len(),
                plugin_manager.plugin_count()
            );
        }
        rules.extend(plugin_rules);
    }

    // Get changed lines if diff mode
    let changed_lines = if cli.diff {
        Some(get_git_diff_lines()?)
    } else {
        None
    };

    // Create lint engine
    let engine = LintEngine::new(rules, config.clone());

    // Collect files to lint
    let mut files_to_lint = Vec::new();

    for pattern in &cli.files {
        let pattern_str = pattern.to_string_lossy();

        // Handle stdin
        if pattern_str == "-" {
            files_to_lint.push(PathBuf::from("-"));
            continue;
        }

        if pattern_str.contains('*') {
            for entry in glob::glob(&pattern_str).into_diagnostic()? {
                let path = entry.into_diagnostic()?;
                if should_lint_file(&path, &config) {
                    files_to_lint.push(path);
                }
            }
        } else if should_lint_file(pattern, &config) {
            files_to_lint.push(pattern.clone());
        }
    }

    if files_to_lint.is_empty() {
        eprintln!("No files to lint");
        return Ok(ExitCode::from(0));
    }

    // Track errors for max-errors limit
    let error_count = AtomicUsize::new(0);
    let max_errors = config.max_errors;

    // Lint files (parallel or sequential)
    let all_diagnostics: Vec<(PathBuf, Vec<Diagnostic>)> = if config.jobs == 1 {
        // Sequential
        lint_files_sequential(&files_to_lint, &engine, &changed_lines, &error_count, max_errors, cli.verbose)
    } else {
        // Parallel
        if let Some(j) = cli.jobs {
            if j > 0 {
                rayon::ThreadPoolBuilder::new()
                    .num_threads(j)
                    .build_global()
                    .ok();
            }
        }
        lint_files_parallel(&files_to_lint, &engine, &changed_lines, &error_count, max_errors, cli.verbose)
    };

    // Flatten diagnostics and build stats
    let mut all_diags = Vec::new();
    let mut stats = LintStatistics::default();

    for (_path, diagnostics) in all_diagnostics {
        let has_errors = diagnostics.iter().any(|d| d.severity == Severity::Error);
        stats.files_linted += 1;
        if has_errors {
            stats.files_with_errors += 1;
        }
        for d in &diagnostics {
            stats.record(d);
        }
        all_diags.extend(diagnostics);
    }

    // Count-only mode
    if cli.count {
        let total = all_diags.len();
        println!("{}", total);
        return Ok(if stats.error_count() > 0 {
            ExitCode::from(2)
        } else if stats.warning_count() > 0 {
            ExitCode::from(1)
        } else {
            ExitCode::from(0)
        });
    }

    // Output results
    match cli.format {
        OutputFormat::Text => {
            wix_lint::output::print_text(&all_diags);
        }
        OutputFormat::Json => {
            wix_lint::output::print_json(&all_diags).into_diagnostic()?;
        }
        OutputFormat::Sarif => {
            wix_lint::output::print_sarif(&all_diags).into_diagnostic()?;
        }
    }

    // Print statistics if requested
    if cli.statistics || cli.show_pep8_errors {
        print_statistics(&stats);
    }

    // Print summary
    if !cli.quiet && !cli.count {
        let file_count = stats.files_linted;
        let file_word = if file_count == 1 { "file" } else { "files" };
        let error_count = stats.error_count();
        let warning_count = stats.warning_count();

        if error_count == 0 && warning_count == 0 {
            eprintln!("\nNo issues found in {} {}", file_count, file_word);
        } else {
            eprintln!(
                "\nFound {} error{} and {} warning{} in {} {}",
                error_count,
                if error_count == 1 { "" } else { "s" },
                warning_count,
                if warning_count == 1 { "" } else { "s" },
                file_count,
                file_word
            );
        }
    }

    // Exit codes
    if stats.error_count() > 0 {
        Ok(ExitCode::from(2))
    } else if stats.warning_count() > 0 {
        Ok(ExitCode::from(1))
    } else {
        Ok(ExitCode::from(0))
    }
}

fn should_lint_file(path: &Path, config: &Config) -> bool {
    !config.is_file_excluded(path) && config.matches_filename_pattern(path)
}

fn lint_files_sequential(
    files: &[PathBuf],
    engine: &LintEngine,
    changed_lines: &Option<std::collections::HashMap<PathBuf, std::collections::HashSet<usize>>>,
    error_count: &AtomicUsize,
    max_errors: usize,
    verbose: bool,
) -> Vec<(PathBuf, Vec<Diagnostic>)> {
    let mut results = Vec::new();

    for file in files {
        if max_errors > 0 && error_count.load(Ordering::Relaxed) >= max_errors {
            break;
        }

        if verbose {
            eprintln!("Linting: {}", file.display());
        }

        match lint_single_file(file, engine, changed_lines) {
            Ok(diagnostics) => {
                let errors = diagnostics.iter().filter(|d| d.severity == Severity::Error).count();
                error_count.fetch_add(errors, Ordering::Relaxed);
                results.push((file.clone(), diagnostics));
            }
            Err(e) => {
                eprintln!("Failed to lint {}: {}", file.display(), e);
                error_count.fetch_add(1, Ordering::Relaxed);
            }
        }
    }

    results
}

fn lint_files_parallel(
    files: &[PathBuf],
    engine: &LintEngine,
    changed_lines: &Option<std::collections::HashMap<PathBuf, std::collections::HashSet<usize>>>,
    error_count: &AtomicUsize,
    max_errors: usize,
    verbose: bool,
) -> Vec<(PathBuf, Vec<Diagnostic>)> {
    let results = Mutex::new(Vec::new());

    files.par_iter().for_each(|file| {
        if max_errors > 0 && error_count.load(Ordering::Relaxed) >= max_errors {
            return;
        }

        if verbose {
            eprintln!("Linting: {}", file.display());
        }

        match lint_single_file(file, engine, changed_lines) {
            Ok(diagnostics) => {
                let errors = diagnostics.iter().filter(|d| d.severity == Severity::Error).count();
                error_count.fetch_add(errors, Ordering::Relaxed);
                results.lock().unwrap().push((file.clone(), diagnostics));
            }
            Err(e) => {
                eprintln!("Failed to lint {}: {}", file.display(), e);
                error_count.fetch_add(1, Ordering::Relaxed);
            }
        }
    });

    results.into_inner().unwrap()
}

fn lint_single_file(
    file: &PathBuf,
    engine: &LintEngine,
    changed_lines: &Option<std::collections::HashMap<PathBuf, std::collections::HashSet<usize>>>,
) -> Result<Vec<Diagnostic>> {
    // Handle stdin
    if file.to_string_lossy() == "-" {
        let mut content = String::new();
        io::stdin().read_to_string(&mut content).into_diagnostic()?;
        let doc = WixDocument::parse_str(&content).into_diagnostic()?;
        let diagnostics = engine.lint_document(&doc, file).into_diagnostic()?;
        return Ok(filter_by_changed_lines(diagnostics, file, changed_lines));
    }

    let diagnostics = engine.lint_file(file).into_diagnostic()?;
    Ok(filter_by_changed_lines(diagnostics, file, changed_lines))
}

fn filter_by_changed_lines(
    diagnostics: Vec<Diagnostic>,
    file: &PathBuf,
    changed_lines: &Option<std::collections::HashMap<PathBuf, std::collections::HashSet<usize>>>,
) -> Vec<Diagnostic> {
    match changed_lines {
        Some(lines_map) => {
            if let Some(lines) = lines_map.get(file) {
                diagnostics
                    .into_iter()
                    .filter(|d| lines.contains(&d.location.line))
                    .collect()
            } else {
                Vec::new() // File not in diff, skip all diagnostics
            }
        }
        None => diagnostics,
    }
}

fn get_git_diff_lines() -> Result<std::collections::HashMap<PathBuf, std::collections::HashSet<usize>>> {
    use std::collections::{HashMap, HashSet};
    use std::process::Command;

    let output = Command::new("git")
        .args(["diff", "--unified=0", "HEAD"])
        .output()
        .into_diagnostic()?;

    let mut result: HashMap<PathBuf, HashSet<usize>> = HashMap::new();
    let stdout = String::from_utf8_lossy(&output.stdout);

    let mut current_file: Option<PathBuf> = None;

    for line in stdout.lines() {
        // Parse diff header: +++ b/path/to/file.wxs
        if let Some(path) = line.strip_prefix("+++ b/") {
            current_file = Some(PathBuf::from(path));
        }
        // Parse hunk header: @@ -old,count +new,count @@
        else if line.starts_with("@@") && current_file.is_some() {
            if let Some(plus_pos) = line.find('+') {
                let rest = &line[plus_pos + 1..];
                if let Some(space_pos) = rest.find(' ') {
                    let range = &rest[..space_pos];
                    let parts: Vec<&str> = range.split(',').collect();
                    if let Ok(start) = parts[0].parse::<usize>() {
                        let count = if parts.len() > 1 {
                            parts[1].parse::<usize>().unwrap_or(1)
                        } else {
                            1
                        };
                        if let Some(ref file) = current_file {
                            let lines = result.entry(file.clone()).or_default();
                            for i in start..start + count {
                                lines.insert(i);
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(result)
}

fn print_statistics(stats: &LintStatistics) {
    eprintln!("\n\x1b[1mStatistics:\x1b[0m");
    eprintln!("  Files linted: {}", stats.files_linted);
    eprintln!("  Files with errors: {}", stats.files_with_errors);
    eprintln!();

    if !stats.per_rule.is_empty() {
        eprintln!("  \x1b[1mBy rule:\x1b[0m");
        let mut rules: Vec<_> = stats.per_rule.iter().collect();
        rules.sort_by(|a, b| b.1.cmp(a.1));
        for (rule, count) in rules {
            eprintln!("    {:40} {}", rule, count);
        }
    }

    eprintln!();
    eprintln!("  \x1b[1mBy severity:\x1b[0m");
    eprintln!("    \x1b[1;31mErrors:\x1b[0m   {}", stats.error_count());
    eprintln!("    \x1b[1;33mWarnings:\x1b[0m {}", stats.warning_count());
    eprintln!("    \x1b[1;36mInfo:\x1b[0m     {}", stats.info_count());
}
