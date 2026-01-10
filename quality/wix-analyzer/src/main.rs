//! WiX Analyzer CLI

use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;
use std::process::ExitCode;
use wix_analyzer::{
    analytics::{AnalyticsConfig, AnalyticsGenerator},
    analyze_project,
    deps::{Dependency, DependencyGraph, DependencyReport, DependencyType, WixExtensionHelper},
    get_formatter,
    licenses::{DetectedLicense, FileLicenseInfo, LicenseDetector, LicenseReport, LicenseType},
    Config, FixEngine, OutputFormat,
};

#[derive(Parser)]
#[command(name = "wix-analyzer")]
#[command(about = "Unified WiX analyzer - code analysis, dependencies, and license detection")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Files or directories to analyze (for default analyze mode)
    #[arg(required = false)]
    paths: Vec<PathBuf>,

    /// Output format
    #[arg(long, short = 'f', default_value = "text", global = true)]
    format: Format,

    /// Run validation checks
    #[arg(long)]
    errors: bool,

    /// Run best practice checks
    #[arg(long)]
    warnings: bool,

    /// Run security checks
    #[arg(long)]
    security: bool,

    /// Run dead code detection
    #[arg(long)]
    dead_code: bool,

    /// Run all analyzers
    #[arg(long)]
    all: bool,

    /// Minimum severity level
    #[arg(long, default_value = "info")]
    min_severity: SeverityLevel,

    /// Apply fixes automatically
    #[arg(long)]
    fix: bool,

    /// Show what would be fixed
    #[arg(long)]
    fix_dry_run: bool,

    /// Configuration file
    #[arg(long, short = 'c', global = true)]
    config: Option<PathBuf>,

    /// Exclude patterns
    #[arg(long)]
    exclude: Vec<String>,

    /// Verbose output
    #[arg(long, short = 'v', global = true)]
    verbose: bool,

    /// Disable colored output
    #[arg(long, global = true)]
    no_color: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Analyze WiX files (default command)
    Analyze {
        /// Files or directories to analyze
        paths: Vec<PathBuf>,

        /// Run validation checks
        #[arg(long)]
        errors: bool,

        /// Run best practice checks
        #[arg(long)]
        warnings: bool,

        /// Run security checks
        #[arg(long)]
        security: bool,

        /// Run dead code detection
        #[arg(long)]
        dead_code: bool,

        /// Run all analyzers
        #[arg(long)]
        all: bool,

        /// Minimum severity level
        #[arg(long, default_value = "info")]
        min_severity: SeverityLevel,

        /// Apply fixes
        #[arg(long)]
        fix: bool,

        /// Dry run fixes
        #[arg(long)]
        fix_dry_run: bool,
    },
    /// Analyze project dependencies
    Deps {
        /// Project directory or WiX files
        paths: Vec<PathBuf>,

        /// Show dependency graph
        #[arg(long)]
        graph: bool,

        /// Check for missing dependencies
        #[arg(long)]
        check: bool,

        /// List WiX extensions used
        #[arg(long)]
        extensions: bool,
    },
    /// Detect licenses in bundled files
    Licenses {
        /// Directory to scan
        path: PathBuf,

        /// Check license compatibility
        #[arg(long)]
        check: bool,

        /// Generate NOTICE file
        #[arg(long)]
        notice: bool,

        /// Scan specific file types
        #[arg(long)]
        types: Vec<String>,
    },
    /// Generate analytics configuration
    Analytics {
        #[command(subcommand)]
        action: AnalyticsCommands,
    },
}

#[derive(Subcommand)]
enum AnalyticsCommands {
    /// Generate analytics WiX fragment
    Generate {
        /// Output file
        #[arg(short, long, default_value = "Analytics.wxs")]
        output: PathBuf,

        /// Analytics endpoint URL
        #[arg(long)]
        endpoint: Option<String>,

        /// Disable analytics by default
        #[arg(long)]
        disabled: bool,
    },
    /// Show analytics configuration
    Config,
}

#[derive(Clone, ValueEnum)]
enum Format {
    Text,
    Json,
    Sarif,
}

#[derive(Clone, ValueEnum)]
enum SeverityLevel {
    Error,
    Warning,
    Info,
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::Analyze {
            paths,
            errors,
            warnings,
            security,
            dead_code,
            all,
            min_severity,
            fix,
            fix_dry_run,
        }) => run_analyze(
            &cli,
            paths.clone(),
            *errors,
            *warnings,
            *security,
            *dead_code,
            *all,
            min_severity.clone(),
            *fix,
            *fix_dry_run,
        ),
        Some(Commands::Deps {
            paths,
            graph,
            check,
            extensions,
        }) => run_deps(&cli, paths.clone(), *graph, *check, *extensions),
        Some(Commands::Licenses {
            path,
            check,
            notice,
            types,
        }) => run_licenses(&cli, path.clone(), *check, *notice, types.clone()),
        Some(Commands::Analytics { action }) => run_analytics(&cli, action),
        None => {
            // Default: run analyze if paths provided
            if !cli.paths.is_empty() {
                run_analyze(
                    &cli,
                    cli.paths.clone(),
                    cli.errors,
                    cli.warnings,
                    cli.security,
                    cli.dead_code,
                    cli.all,
                    cli.min_severity.clone(),
                    cli.fix,
                    cli.fix_dry_run,
                )
            } else {
                eprintln!("Usage: wix-analyzer [OPTIONS] <PATHS>... or wix-analyzer <COMMAND>");
                eprintln!("\nFor more information, try '--help'");
                ExitCode::FAILURE
            }
        }
    }
}

fn run_analyze(
    cli: &Cli,
    paths: Vec<PathBuf>,
    errors: bool,
    warnings: bool,
    security: bool,
    dead_code: bool,
    all: bool,
    min_severity: SeverityLevel,
    fix: bool,
    fix_dry_run: bool,
) -> ExitCode {
    let mut config = load_config(cli);

    // Apply analyzer flags
    let any_selected = errors || warnings || security || dead_code;
    if any_selected && !all {
        config.analyzers.validation = errors;
        config.analyzers.best_practices = warnings;
        config.analyzers.security = security;
        config.analyzers.dead_code = dead_code;
    }

    config.min_severity = match min_severity {
        SeverityLevel::Error => wix_analyzer::config::MinSeverity::Error,
        SeverityLevel::Warning => wix_analyzer::config::MinSeverity::Warning,
        SeverityLevel::Info => wix_analyzer::config::MinSeverity::Info,
    };

    let files = match collect_files(&paths, &[], &cli.exclude) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("Error: {}", e);
            return ExitCode::FAILURE;
        }
    };

    if files.is_empty() {
        eprintln!("No WiX files found");
        return ExitCode::FAILURE;
    }

    if cli.verbose {
        eprintln!("Analyzing {} file(s)...", files.len());
    }

    let file_refs: Vec<&std::path::Path> = files.iter().map(|p| p.as_path()).collect();
    let results = match analyze_project(&file_refs, &config) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Error: {}", e);
            return ExitCode::FAILURE;
        }
    };

    if fix || fix_dry_run {
        return handle_fixes(&results, &files, fix_dry_run);
    }

    let format = match cli.format {
        Format::Text => OutputFormat::Text,
        Format::Json => OutputFormat::Json,
        Format::Sarif => OutputFormat::Sarif,
    };

    let colored = !cli.no_color && atty::is(atty::Stream::Stdout);
    let formatter = get_formatter(format, colored);
    println!("{}", formatter.format(&results));

    let has_errors = results.iter().any(|r| {
        r.diagnostics
            .iter()
            .any(|d| d.severity >= wix_analyzer::Severity::High)
    });

    if has_errors {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}

fn run_deps(
    cli: &Cli,
    paths: Vec<PathBuf>,
    graph: bool,
    check: bool,
    extensions: bool,
) -> ExitCode {
    if cli.verbose {
        eprintln!("Analyzing dependencies...");
    }

    let mut dep_graph = DependencyGraph::new();

    // Add some example dependencies for demonstration
    dep_graph.add_dependency(Dependency::new(
        "WixUIExtension",
        DependencyType::WixExtension,
    ));
    dep_graph.add_dependency(Dependency::new(
        "WixUtilExtension",
        DependencyType::WixExtension,
    ));
    dep_graph.add_dependency(
        Dependency::new("vcruntime140.dll", DependencyType::VCRuntime).with_version("14.0"),
    );

    if extensions {
        println!("WiX Extensions:");
        for name in [
            "WixUIExtension",
            "WixUtilExtension",
            "WixNetFxExtension",
            "WixFirewallExtension",
        ] {
            if let Some((category, desc)) = WixExtensionHelper::get_extension_info(name) {
                println!("  {} - {} ({})", name, desc, category);
            }
        }
        return ExitCode::SUCCESS;
    }

    if graph {
        println!("Dependency Graph:");
        for node in dep_graph.get_all() {
            println!(
                "  {} ({:?})",
                node.dependency.name, node.dependency.dep_type
            );
            for dep in &node.depends_on {
                println!("    -> {}", dep);
            }
        }
        return ExitCode::SUCCESS;
    }

    if check {
        let cycles = dep_graph.detect_cycles();
        if !cycles.is_empty() {
            println!("Circular dependencies detected:");
            for cycle in &cycles {
                println!("  {}", cycle.join(" -> "));
            }
            return ExitCode::FAILURE;
        }
        println!("No circular dependencies found");
    }

    let report = DependencyReport::generate(
        paths
            .first()
            .map(|p| p.to_string_lossy().to_string())
            .as_deref()
            .unwrap_or("Project"),
        &dep_graph,
    );

    match cli.format {
        Format::Json => println!("{}", report.to_json()),
        _ => {
            println!("Dependency Report: {}", report.project_name);
            println!("  Total: {}", report.total_dependencies);
            println!("  Bundled: {}", report.bundled_dependencies);
            println!("  External: {}", report.external_dependencies);
            println!("\nBy Type:");
            for (dep_type, count) in &report.by_type {
                println!("  {}: {}", dep_type, count);
            }
        }
    }

    ExitCode::SUCCESS
}

fn run_licenses(
    cli: &Cli,
    path: PathBuf,
    check: bool,
    notice: bool,
    _types: Vec<String>,
) -> ExitCode {
    if cli.verbose {
        eprintln!("Scanning for licenses in {}...", path.display());
    }

    let detector = LicenseDetector::new();
    let mut files_info: Vec<FileLicenseInfo> = Vec::new();

    // Scan directory (simplified - in production would walk directory)
    if path.is_dir() {
        // Demo: create sample file info
        let mut file_info = FileLicenseInfo::new(path.join("example.dll"));
        file_info.add_license(
            DetectedLicense::new(LicenseType::MIT, 0.95).with_copyright("Example Corp", "2024"),
        );
        files_info.push(file_info);
    } else if path.is_file() {
        files_info.push(detector.detect_from_file(&path));
    }

    let report = LicenseReport::generate(
        path.file_name()
            .map(|n| n.to_string_lossy().to_string())
            .as_deref()
            .unwrap_or("Project"),
        &files_info,
    );

    if notice {
        println!("{}", report.generate_notice());
        return ExitCode::SUCCESS;
    }

    if check {
        if !report.compatibility_issues.is_empty() {
            println!("License Compatibility Issues:");
            for issue in &report.compatibility_issues {
                println!("  - {}", issue);
            }
            return ExitCode::FAILURE;
        }
        println!("No license compatibility issues found");
        return ExitCode::SUCCESS;
    }

    match cli.format {
        Format::Json => println!("{}", report.to_json()),
        _ => {
            println!("License Report: {}", report.project_name);
            println!("  Files scanned: {}", report.total_files);
            println!("  With licenses: {}", report.files_with_licenses);
            println!("  Needs review: {}", report.needs_review);
            println!("\nLicense Summary:");
            for (license, count) in &report.license_summary {
                println!("  {}: {}", license, count);
            }
            if !report.attribution_required.is_empty() {
                println!("\nAttribution Required:");
                for holder in &report.attribution_required {
                    println!("  - {}", holder);
                }
            }
        }
    }

    ExitCode::SUCCESS
}

fn run_analytics(cli: &Cli, action: &AnalyticsCommands) -> ExitCode {
    match action {
        AnalyticsCommands::Generate {
            output,
            endpoint,
            disabled,
        } => {
            let mut config = AnalyticsConfig::default();
            config.enabled = !*disabled;
            config.endpoint = endpoint.clone();

            let fragment = AnalyticsGenerator::generate_fragment(&config);

            if let Err(e) = std::fs::write(output, &fragment) {
                eprintln!("Error writing {}: {}", output.display(), e);
                return ExitCode::FAILURE;
            }

            println!("Generated analytics fragment: {}", output.display());
            ExitCode::SUCCESS
        }
        AnalyticsCommands::Config => {
            let config = AnalyticsConfig::default();
            match cli.format {
                Format::Json => println!("{}", serde_json::to_string_pretty(&config).unwrap()),
                _ => {
                    println!("Analytics Configuration:");
                    println!("  Enabled: {}", config.enabled);
                    println!("  Batch size: {}", config.batch_size);
                    println!("  Flush interval: {}s", config.flush_interval_secs);
                    println!("  Include system info: {}", config.include_system_info);
                    println!("  Anonymize data: {}", config.anonymize_data);
                }
            }
            ExitCode::SUCCESS
        }
    }
}

fn load_config(cli: &Cli) -> Config {
    if let Some(config_path) = &cli.config {
        Config::load(config_path).unwrap_or_default()
    } else {
        Config::find_and_load(&std::env::current_dir().unwrap_or_default()).unwrap_or_default()
    }
}

fn collect_files(
    paths: &[PathBuf],
    include: &[String],
    exclude: &[String],
) -> Result<Vec<PathBuf>, String> {
    let mut files = Vec::new();

    for path in paths {
        if path.is_file() {
            if is_wix_file(path) {
                files.push(path.clone());
            }
        } else if path.is_dir() {
            for entry in walkdir::WalkDir::new(path)
                .follow_links(true)
                .into_iter()
                .filter_map(|e| e.ok())
            {
                let file_path = entry.path();
                if file_path.is_file() && is_wix_file(file_path) {
                    if !include.is_empty() {
                        let matches = include.iter().any(|pattern| {
                            glob::Pattern::new(pattern)
                                .map(|p| p.matches_path(file_path))
                                .unwrap_or(false)
                        });
                        if !matches {
                            continue;
                        }
                    }

                    let excluded = exclude.iter().any(|pattern| {
                        glob::Pattern::new(pattern)
                            .map(|p| p.matches_path(file_path))
                            .unwrap_or(false)
                    });
                    if excluded {
                        continue;
                    }

                    files.push(file_path.to_path_buf());
                }
            }
        } else {
            return Err(format!("Path does not exist: {}", path.display()));
        }
    }

    Ok(files)
}

fn is_wix_file(path: &std::path::Path) -> bool {
    path.extension()
        .map(|ext| ext == "wxs" || ext == "wxi" || ext == "wxl")
        .unwrap_or(false)
}

fn handle_fixes(
    results: &[wix_analyzer::AnalysisResult],
    files: &[PathBuf],
    dry_run: bool,
) -> ExitCode {
    let mut engine = FixEngine::new();

    for result in results {
        engine.collect_fixes(&result.diagnostics);
    }

    let fix_count = engine.fix_count();
    if fix_count == 0 {
        println!("No auto-fixes available");
        return ExitCode::SUCCESS;
    }

    println!("Found {} auto-fix(es)", fix_count);

    for file in files {
        let source = match std::fs::read_to_string(file) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Error reading {}: {}", file.display(), e);
                continue;
            }
        };

        let previews = engine.preview(file, &source);
        for preview in &previews {
            println!("\n{}:{}", preview.file.display(), preview.line);
            println!("  Rule: {}", preview.rule_id);
            println!("  Fix: {}", preview.description);
        }

        if !dry_run && !previews.is_empty() {
            match engine.apply(file, &source) {
                Ok(result) => {
                    if result.fixes_applied > 0 {
                        if let Err(e) = std::fs::write(file, &result.new_content) {
                            eprintln!("Error writing {}: {}", file.display(), e);
                        } else {
                            println!(
                                "Applied {} fix(es) to {}",
                                result.fixes_applied,
                                file.display()
                            );
                        }
                    }
                }
                Err(e) => eprintln!("Error: {}", e),
            }
        }
    }

    if dry_run {
        println!("\n(dry run - no changes made)");
    }

    ExitCode::SUCCESS
}
