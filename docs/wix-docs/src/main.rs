//! WiX Documentation Generator CLI

use clap::Parser;
use colored::Colorize;
use std::path::PathBuf;
use std::process;

use wix_docs::{generate_docs, DocsConfig, OutputFormat};

#[derive(Parser)]
#[command(name = "wix-docs")]
#[command(about = "Generate documentation for WiX projects")]
#[command(version)]
struct Cli {
    /// Source directory containing WiX files
    #[arg(default_value = ".")]
    source: PathBuf,

    /// Output directory
    #[arg(short, long, default_value = "docs")]
    output: PathBuf,

    /// Output format (html, markdown, json)
    #[arg(short, long, default_value = "html")]
    format: String,

    /// Project name (defaults to directory name)
    #[arg(short, long)]
    name: Option<String>,

    /// Include private elements (starting with _)
    #[arg(long)]
    include_private: bool,

    /// Generate table of contents
    #[arg(long, default_value = "true")]
    toc: bool,

    /// Generate dependency graph
    #[arg(long, default_value = "true")]
    graph: bool,

    /// Quiet mode (suppress output)
    #[arg(short, long)]
    quiet: bool,
}

fn main() {
    let cli = Cli::parse();

    let format = match cli.format.to_lowercase().as_str() {
        "html" => OutputFormat::Html,
        "markdown" | "md" => OutputFormat::Markdown,
        "json" => OutputFormat::Json,
        _ => {
            eprintln!(
                "{}: Unknown format '{}'. Use html, markdown, or json.",
                "Error".red().bold(),
                cli.format
            );
            process::exit(1);
        }
    };

    let config = DocsConfig {
        format,
        output_dir: cli.output.clone(),
        include_private: cli.include_private,
        generate_toc: cli.toc,
        generate_graph: cli.graph,
        project_name: cli.name,
    };

    if !cli.quiet {
        println!(
            "{} Generating documentation for {}",
            "→".blue().bold(),
            cli.source.display()
        );
    }

    match generate_docs(&cli.source, &config) {
        Ok(project) => {
            if !cli.quiet {
                println!("{} Documentation generated successfully!\n", "✓".green().bold());
                println!("  {} Components:      {}", "•".dimmed(), project.components.len());
                println!("  {} Features:        {}", "•".dimmed(), project.features.len());
                println!("  {} Custom Actions:  {}", "•".dimmed(), project.custom_actions.len());
                println!("  {} Properties:      {}", "•".dimmed(), project.properties.len());
                println!("  {} Directories:     {}", "•".dimmed(), project.directories.len());
                println!();
                println!(
                    "  Output: {}",
                    cli.output.display().to_string().cyan()
                );
            }
        }
        Err(e) => {
            eprintln!("{}: {}", "Error".red().bold(), e);
            process::exit(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_default() {
        let cli = Cli::parse_from(["wix-docs"]);
        assert_eq!(cli.source, PathBuf::from("."));
        assert_eq!(cli.output, PathBuf::from("docs"));
        assert_eq!(cli.format, "html");
    }

    #[test]
    fn test_cli_custom_args() {
        let cli = Cli::parse_from([
            "wix-docs",
            "src",
            "-o", "output",
            "-f", "markdown",
            "-n", "MyProject",
            "--include-private",
        ]);
        assert_eq!(cli.source, PathBuf::from("src"));
        assert_eq!(cli.output, PathBuf::from("output"));
        assert_eq!(cli.format, "markdown");
        assert_eq!(cli.name, Some("MyProject".to_string()));
        assert!(cli.include_private);
    }

    #[test]
    fn test_cli_quiet() {
        let cli = Cli::parse_from(["wix-docs", "-q"]);
        assert!(cli.quiet);
    }
}
