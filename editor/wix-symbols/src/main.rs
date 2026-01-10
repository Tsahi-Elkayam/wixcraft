//! wix-symbols CLI - WiX document symbols extractor

use clap::{Parser, ValueEnum};
use std::fs;
use std::io::{self, Read};
use std::process::ExitCode;
use wix_symbols::{extract_symbols, filter_symbols, flatten_symbols};

#[derive(Parser)]
#[command(name = "wix-symbols")]
#[command(about = "WiX document symbols extractor")]
#[command(version)]
struct Cli {
    /// WiX file (or - for stdin)
    file: String,

    /// Output format
    #[arg(long, value_enum, default_value = "text")]
    format: OutputFormat,

    /// Flat list instead of hierarchy
    #[arg(long)]
    flat: bool,

    /// Filter symbols by name (workspace symbol mode)
    #[arg(long)]
    query: Option<String>,

    /// Verbose output
    #[arg(short, long)]
    verbose: bool,
}

#[derive(Clone, Copy, ValueEnum)]
enum OutputFormat {
    Text,
    Json,
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    if let Err(e) = run(cli) {
        eprintln!("Error: {}", e);
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}

fn run(cli: Cli) -> Result<(), Box<dyn std::error::Error>> {
    // Read source
    let source = if cli.file == "-" {
        let mut buffer = String::new();
        io::stdin().read_to_string(&mut buffer)?;
        buffer
    } else {
        fs::read_to_string(&cli.file)?
    };

    if cli.verbose {
        eprintln!("Parsing source ({} bytes)", source.len());
    }

    // Extract symbols
    let symbols = extract_symbols(&source)?;

    if cli.verbose {
        let count = flatten_symbols(&symbols).len();
        eprintln!("Found {} symbols", count);
    }

    // Apply query filter if provided
    if let Some(ref query) = cli.query {
        let filtered = filter_symbols(&symbols, query);

        if cli.verbose {
            eprintln!("Query '{}' matched {} symbols", query, filtered.len());
        }

        match cli.format {
            OutputFormat::Text => {
                for symbol in filtered {
                    print!("{}", symbol.format_text(0));
                }
            }
            OutputFormat::Json => {
                println!("{}", serde_json::to_string_pretty(&filtered)?);
            }
        }

        return Ok(());
    }

    // Output symbols
    match cli.format {
        OutputFormat::Text => {
            if cli.flat {
                let flat = flatten_symbols(&symbols);
                for symbol in flat {
                    // Print without children in flat mode
                    let kind_name = symbol.kind.display_name();
                    let detail = symbol
                        .detail
                        .as_ref()
                        .map(|d| format!(" ({})", d))
                        .unwrap_or_default();
                    let range = format!(
                        "[{}:{}-{}:{}]",
                        symbol.range.start.line,
                        symbol.range.start.character,
                        symbol.range.end.line,
                        symbol.range.end.character
                    );
                    println!("{}: {}{} {}", kind_name, symbol.name, detail, range);
                }
            } else {
                for symbol in &symbols {
                    print!("{}", symbol.format_text(0));
                }
            }
        }
        OutputFormat::Json => {
            if cli.flat {
                let flat = flatten_symbols(&symbols);
                println!("{}", serde_json::to_string_pretty(&flat)?);
            } else {
                println!("{}", serde_json::to_string_pretty(&symbols)?);
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_output_format_from_str() {
        // Test that ValueEnum works
        use clap::ValueEnum;
        let _text = OutputFormat::from_str("text", true).unwrap();
        let _json = OutputFormat::from_str("json", true).unwrap();
    }
}
