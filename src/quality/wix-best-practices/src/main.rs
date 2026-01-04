//! wix-best-practices CLI

use clap::{Parser, ValueEnum};
use colored::Colorize;
use std::io::{self, Read};
use std::path::PathBuf;
use std::process::ExitCode;
use wix_best_practices::{
    AnalysisResult, AnalyzerConfig, BestPracticesAnalyzer, Impact, PracticeCategory,
};

#[derive(Parser)]
#[command(name = "wix-best-practices")]
#[command(about = "WiX best practices analyzer - checks for efficiency, idioms, performance, and maintainability")]
#[command(version)]
struct Cli {
    /// File or directory to analyze (use - for stdin)
    path: String,

    /// Categories to check
    #[arg(long, value_enum, value_delimiter = ',')]
    categories: Option<Vec<CategoryArg>>,

    /// Minimum impact level to report
    #[arg(long, value_enum, default_value = "low")]
    min_impact: ImpactArg,

    /// Output format
    #[arg(long, value_enum, default_value = "text")]
    format: OutputFormat,

    /// Show rule IDs in output
    #[arg(long)]
    show_rules: bool,

    /// Verbose output
    #[arg(short, long)]
    verbose: bool,
}

#[derive(Clone, Copy, ValueEnum)]
enum CategoryArg {
    Efficiency,
    Idiom,
    Performance,
    Maintainability,
}

impl From<CategoryArg> for PracticeCategory {
    fn from(arg: CategoryArg) -> Self {
        match arg {
            CategoryArg::Efficiency => PracticeCategory::Efficiency,
            CategoryArg::Idiom => PracticeCategory::Idiom,
            CategoryArg::Performance => PracticeCategory::Performance,
            CategoryArg::Maintainability => PracticeCategory::Maintainability,
        }
    }
}

#[derive(Clone, Copy, ValueEnum)]
enum ImpactArg {
    Low,
    Medium,
    High,
}

impl From<ImpactArg> for Impact {
    fn from(arg: ImpactArg) -> Self {
        match arg {
            ImpactArg::Low => Impact::Low,
            ImpactArg::Medium => Impact::Medium,
            ImpactArg::High => Impact::High,
        }
    }
}

#[derive(Clone, Copy, ValueEnum)]
enum OutputFormat {
    Text,
    Json,
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    match run(cli) {
        Ok(has_issues) => {
            if has_issues {
                ExitCode::from(1)
            } else {
                ExitCode::SUCCESS
            }
        }
        Err(e) => {
            eprintln!("{}: {}", "Error".red().bold(), e);
            ExitCode::from(2)
        }
    }
}

fn run(cli: Cli) -> Result<bool, Box<dyn std::error::Error>> {
    // Build config
    let categories = cli.categories.map(|cats| {
        cats.into_iter().map(PracticeCategory::from).collect()
    }).unwrap_or_else(|| vec![
        PracticeCategory::Efficiency,
        PracticeCategory::Idiom,
        PracticeCategory::Performance,
        PracticeCategory::Maintainability,
    ]);

    let config = AnalyzerConfig {
        categories,
        min_impact: cli.min_impact.into(),
        ..Default::default()
    };

    let analyzer = BestPracticesAnalyzer::with_config(config);

    // Analyze
    let result = if cli.path == "-" {
        // Read from stdin
        let mut source = String::new();
        io::stdin().read_to_string(&mut source)?;
        analyzer.analyze_source(&source, &PathBuf::from("stdin.wxs"))?
    } else {
        let path = PathBuf::from(&cli.path);
        if path.is_dir() {
            if cli.verbose {
                eprintln!("Analyzing directory: {}", path.display());
            }
            analyzer.analyze_directory(&path)?
        } else {
            if cli.verbose {
                eprintln!("Analyzing file: {}", path.display());
            }
            analyzer.analyze_file(&path)?
        }
    };

    let has_issues = !result.is_empty();

    // Output
    match cli.format {
        OutputFormat::Text => output_text(&result, cli.show_rules, cli.verbose),
        OutputFormat::Json => output_json(&result)?,
    }

    Ok(has_issues)
}

fn output_text(result: &AnalysisResult, show_rules: bool, verbose: bool) {
    for suggestion in &result.suggestions {
        let impact_str = match suggestion.impact {
            Impact::High => "HIGH".red().bold(),
            Impact::Medium => "MEDIUM".yellow(),
            Impact::Low => "LOW".blue(),
        };

        let category_str = suggestion.category.as_str().cyan();

        let rule_str = if show_rules {
            format!("[{}] ", suggestion.rule_id.dimmed())
        } else {
            String::new()
        };

        println!(
            "{}:{}:{}: {} {} {}{}",
            suggestion.location.file.display(),
            suggestion.location.range.start.line,
            suggestion.location.range.start.character,
            impact_str,
            category_str,
            rule_str,
            suggestion.title
        );

        if verbose {
            println!("  {}", suggestion.message);
            if let Some(fix) = &suggestion.fix {
                println!("  {}: {}", "Fix".green(), fix.description);
            }
            println!();
        }
    }

    // Summary
    println!();
    println!(
        "{}: {} file(s) analyzed, {} suggestion(s)",
        "Summary".bold(),
        result.files.len(),
        result.len()
    );

    if !result.is_empty() {
        println!(
            "  {} high, {} medium, {} low",
            result.count_by_impact(Impact::High).to_string().red(),
            result.count_by_impact(Impact::Medium).to_string().yellow(),
            result.count_by_impact(Impact::Low).to_string().blue()
        );
    }
}

fn output_json(result: &AnalysisResult) -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", serde_json::to_string_pretty(result)?);
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_cli_parse() {
        // Basic smoke test
    }
}
