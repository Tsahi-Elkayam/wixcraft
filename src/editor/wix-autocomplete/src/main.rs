//! wix-autocomplete CLI

use clap::Parser;
use std::io::{self, Read};
use std::path::PathBuf;
use wix_autocomplete::AutocompleteEngine;

#[derive(Parser)]
#[command(name = "wix-autocomplete")]
#[command(about = "Context-aware autocomplete for WiX XML files")]
#[command(version)]
struct Cli {
    /// WiX file to analyze (use - for stdin)
    file: PathBuf,

    /// Cursor line (1-based)
    line: u32,

    /// Cursor column (1-based)
    column: u32,

    /// Path to wix-data directory
    #[arg(short = 'w', long = "wix-data")]
    wix_data: Option<PathBuf>,

    /// Output format
    #[arg(short, long, default_value = "json")]
    format: OutputFormat,

    /// Maximum number of completions
    #[arg(short, long, default_value = "50")]
    limit: usize,

    /// Verbose output (show context info)
    #[arg(short, long)]
    verbose: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum OutputFormat {
    Json,
    Plain,
}

impl std::str::FromStr for OutputFormat {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "json" => Ok(OutputFormat::Json),
            "plain" | "text" => Ok(OutputFormat::Plain),
            _ => Err(format!("Unknown format: {}. Use 'json' or 'plain'", s)),
        }
    }
}

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    // Read source
    let source = if cli.file.to_string_lossy() == "-" {
        let mut buf = String::new();
        io::stdin().read_to_string(&mut buf)?;
        buf
    } else {
        std::fs::read_to_string(&cli.file)?
    };

    // Find wix-data directory
    let wix_data_path = cli.wix_data.unwrap_or_else(|| {
        // Try to find it relative to current dir or executable
        let candidates = [
            PathBuf::from("src/core/wix-data"),
            PathBuf::from("../wix-data"),
            PathBuf::from("../../core/wix-data"),
        ];

        for candidate in candidates {
            if candidate.exists() {
                return candidate;
            }
        }

        // Default fallback
        PathBuf::from("wix-data")
    });

    // Load engine
    let engine = AutocompleteEngine::from_wix_data(&wix_data_path)?
        .with_max_completions(cli.limit);

    if cli.verbose {
        eprintln!("Loaded {} elements, {} snippets", engine.element_count(), engine.snippet_count());
        eprintln!("Analyzing {}:{}:{}", cli.file.display(), cli.line, cli.column);

        let context = wix_autocomplete::parse_context(&source, cli.line, cli.column);
        eprintln!("Context: {:?}", context);
    }

    // Get completions
    let completions = engine.complete(&source, cli.line, cli.column);

    // Output results
    match cli.format {
        OutputFormat::Json => {
            let json = serde_json::to_string_pretty(&completions)?;
            println!("{}", json);
        }
        OutputFormat::Plain => {
            for item in &completions {
                let detail = item.detail.as_deref().unwrap_or("");
                println!("{:<30} {:<12?} {}", item.label, item.kind, detail);
            }
            if cli.verbose {
                eprintln!("\n{} completions", completions.len());
            }
        }
    }

    Ok(())
}
