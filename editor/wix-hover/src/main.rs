//! wix-hover CLI - WiX hover documentation provider.
//!
//! Standalone CLI for getting hover documentation from WiX files.
//!
//! # Usage
//!
//! ```bash
//! # Get hover info at position
//! wix-hover file.wxs 10 15 --wix-data ./wix-data
//!
//! # Read from stdin
//! cat file.wxs | wix-hover - 10 15 --wix-data ./wix-data
//!
//! # Different output formats
//! wix-hover file.wxs 10 15 --wix-data ./wix-data --format json
//! wix-hover file.wxs 10 15 --wix-data ./wix-data --format plain
//! ```

use clap::{Parser, ValueEnum};
use std::fs;
use std::io::{self, Read};
use std::path::PathBuf;
use std::process::ExitCode;
use wix_hover::{HoverProvider, WixData};

#[derive(Parser)]
#[command(name = "wix-hover")]
#[command(about = "WiX hover documentation provider")]
#[command(version)]
struct Cli {
    /// WiX file (or - for stdin)
    file: String,

    /// Line number (1-based)
    line: u32,

    /// Column number (1-based)
    column: u32,

    /// Path to wix-data directory
    #[arg(long)]
    wix_data: Option<PathBuf>,

    /// Output format
    #[arg(long, value_enum, default_value = "markdown")]
    format: OutputFormat,

    /// Verbose output
    #[arg(short, long)]
    verbose: bool,
}

#[derive(Clone, Copy, ValueEnum)]
enum OutputFormat {
    Markdown,
    Plain,
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
    // Load wix-data
    let data = if let Some(ref path) = cli.wix_data {
        match WixData::load(path) {
            Ok(data) => {
                if cli.verbose {
                    eprintln!("Loaded wix-data from {}", path.display());
                    eprintln!("  {} elements", data.elements.len());
                }
                data
            }
            Err(e) => {
                return Err(format!("Failed to load wix-data: {}", e).into());
            }
        }
    } else {
        return Err("--wix-data is required".into());
    };

    // Read source
    let source = if cli.file == "-" {
        let mut buffer = String::new();
        io::stdin().read_to_string(&mut buffer)?;
        buffer
    } else {
        fs::read_to_string(&cli.file)?
    };

    if cli.verbose {
        eprintln!("Position: line {}, column {}", cli.line, cli.column);
    }

    // Get hover info
    let provider = HoverProvider::new(data);
    let hover = provider.hover(&source, cli.line, cli.column);

    match hover {
        Some(info) => {
            let output = match cli.format {
                OutputFormat::Markdown => info.contents,
                OutputFormat::Plain => strip_markdown(&info.contents),
                OutputFormat::Json => serde_json::to_string_pretty(&info)?,
            };
            println!("{}", output);
        }
        None => {
            if cli.verbose {
                eprintln!("No hover information available at this position");
            }
            // Exit with success but no output
        }
    }

    Ok(())
}

/// Strip markdown formatting for plain text output.
fn strip_markdown(md: &str) -> String {
    md.lines()
        .map(|line| {
            let line = line.trim_start_matches('#').trim();
            let line = line.replace("**", "");
            let line = line.replace('`', "");
            // Remove markdown links [text](url) -> text
            let mut result = String::new();
            let mut chars = line.chars().peekable();
            while let Some(c) = chars.next() {
                if c == '[' {
                    // Collect link text
                    let mut text = String::new();
                    for c in chars.by_ref() {
                        if c == ']' {
                            break;
                        }
                        text.push(c);
                    }
                    // Skip URL part
                    if chars.peek() == Some(&'(') {
                        for c in chars.by_ref() {
                            if c == ')' {
                                break;
                            }
                        }
                    }
                    result.push_str(&text);
                } else {
                    result.push(c);
                }
            }
            result
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_markdown_headers() {
        let md = "### Component";
        let plain = strip_markdown(md);
        assert_eq!(plain, "Component");
    }

    #[test]
    fn test_strip_markdown_bold() {
        let md = "**Type:** string";
        let plain = strip_markdown(md);
        assert_eq!(plain, "Type: string");
    }

    #[test]
    fn test_strip_markdown_code() {
        let md = "Use `*` for auto-generation";
        let plain = strip_markdown(md);
        assert_eq!(plain, "Use * for auto-generation");
    }

    #[test]
    fn test_strip_markdown_link() {
        let md = "[Documentation](https://example.com)";
        let plain = strip_markdown(md);
        assert_eq!(plain, "Documentation");
    }

    #[test]
    fn test_strip_markdown_multiline() {
        let md = "### Title\n\n**Bold** text";
        let plain = strip_markdown(md);
        assert!(plain.contains("Title"));
        assert!(plain.contains("Bold text"));
    }
}
