//! wix-fmt CLI - WiX XML formatter

use clap::{Parser, ValueEnum};
use glob::glob;
use std::fs;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use wix_fmt::{FormatConfig, Formatter, WixData};

#[derive(Parser)]
#[command(name = "wix-fmt")]
#[command(about = "WiX XML formatter with data-driven element ordering")]
#[command(version)]
struct Cli {
    /// WiX files to format (supports globs, - for stdin)
    #[arg(required = true)]
    files: Vec<String>,

    /// Check if files are formatted (exit 1 if not)
    #[arg(short, long)]
    check: bool,

    /// Write formatted output back to files
    #[arg(short, long)]
    write: bool,

    /// Write to specific file (single input only)
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Path to use for stdin (for config lookup)
    #[arg(long)]
    stdin_filepath: Option<PathBuf>,

    /// Path to wix-data directory
    #[arg(long)]
    wix_data: Option<PathBuf>,

    /// Path to config file
    #[arg(long)]
    config: Option<PathBuf>,

    /// Indent style
    #[arg(long, value_enum, default_value = "space")]
    indent_style: IndentStyleArg,

    /// Indent size
    #[arg(long, default_value = "2")]
    indent_size: usize,

    /// Max line width before wrapping
    #[arg(long, default_value = "120")]
    max_line_width: usize,

    /// Number of attributes before multiline
    #[arg(long, default_value = "3")]
    attr_threshold: usize,

    /// Sort child elements by canonical wix-data order
    #[arg(long)]
    sort_elements: bool,

    /// Sort attributes (Id first, then alphabetical)
    #[arg(long)]
    sort_attributes: bool,

    /// Verbose output
    #[arg(short, long)]
    verbose: bool,
}

#[derive(Clone, Copy, ValueEnum)]
enum IndentStyleArg {
    Space,
    Tab,
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
    // Load wix-data if provided
    let wix_data = if let Some(ref path) = cli.wix_data {
        match WixData::load(path) {
            Ok(data) => {
                if cli.verbose {
                    eprintln!("Loaded wix-data from {}", path.display());
                    eprintln!("  {} elements", data.elements.len());
                }
                Some(data)
            }
            Err(e) => {
                return Err(format!("Failed to load wix-data: {}", e).into());
            }
        }
    } else {
        None
    };

    // Build configuration
    let config = build_config(&cli)?;

    if cli.verbose {
        eprintln!("Configuration:");
        eprintln!("  indent_style: {:?}", config.indent_style);
        eprintln!("  indent_size: {}", config.indent_size);
        eprintln!("  max_line_width: {}", config.max_line_width);
        eprintln!("  attr_threshold: {}", config.attr_threshold);
        eprintln!("  sort_elements: {}", config.sort_elements);
        eprintln!("  sort_attributes: {}", config.sort_attributes);
    }

    // Create formatter
    let formatter = if let Some(data) = wix_data {
        Formatter::with_wix_data(config, data)
    } else {
        Formatter::new(config)
    };

    // Expand file globs
    let files = expand_files(&cli.files)?;

    if cli.verbose {
        eprintln!("Processing {} file(s)", files.len());
    }

    // Track if any files need formatting (for --check mode)
    let mut needs_formatting = false;

    for file in &files {
        let result = if file == "-" {
            process_stdin(&formatter, &cli)
        } else {
            process_file(&formatter, Path::new(file), &cli)
        };

        match result {
            Ok(changed) => {
                if changed {
                    needs_formatting = true;
                }
            }
            Err(e) => {
                eprintln!("Error processing {}: {}", file, e);
                return Err(e);
            }
        }
    }

    if cli.check && needs_formatting {
        return Err("Some files need formatting".into());
    }

    Ok(())
}

fn build_config(cli: &Cli) -> Result<FormatConfig, Box<dyn std::error::Error>> {
    // Start with default or loaded config
    let mut config = if let Some(ref path) = cli.config {
        FormatConfig::load(path).map_err(|e| format!("Failed to load config: {}", e))?
    } else if let Some(ref path) = cli.stdin_filepath {
        FormatConfig::find_and_load(path).unwrap_or_default()
    } else if let Some(first_file) = cli.files.first() {
        if first_file != "-" {
            let path = Path::new(first_file);
            if let Some(parent) = path.parent() {
                FormatConfig::find_and_load(parent).unwrap_or_default()
            } else {
                FormatConfig::default()
            }
        } else {
            FormatConfig::default()
        }
    } else {
        FormatConfig::default()
    };

    // CLI overrides
    config.indent_style = match cli.indent_style {
        IndentStyleArg::Space => wix_fmt::IndentStyle::Space,
        IndentStyleArg::Tab => wix_fmt::IndentStyle::Tab,
    };
    config.indent_size = cli.indent_size;
    config.max_line_width = cli.max_line_width;
    config.attr_threshold = cli.attr_threshold;

    if cli.sort_elements {
        config.sort_elements = true;
    }
    if cli.sort_attributes {
        config.sort_attributes = true;
    }

    Ok(config)
}

fn expand_files(patterns: &[String]) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let mut files = Vec::new();

    for pattern in patterns {
        if pattern == "-" {
            files.push("-".to_string());
        } else if pattern.contains('*') || pattern.contains('?') {
            for entry in glob(pattern)? {
                files.push(entry?.to_string_lossy().to_string());
            }
        } else {
            files.push(pattern.clone());
        }
    }

    Ok(files)
}

fn process_stdin(
    formatter: &Formatter,
    cli: &Cli,
) -> Result<bool, Box<dyn std::error::Error>> {
    let mut source = String::new();
    io::stdin().read_to_string(&mut source)?;

    let formatted = formatter.format(&source)?;

    if cli.check {
        Ok(source != formatted)
    } else if let Some(ref output_path) = cli.output {
        fs::write(output_path, &formatted)?;
        if cli.verbose {
            eprintln!("Wrote to {}", output_path.display());
        }
        Ok(false)
    } else {
        io::stdout().write_all(formatted.as_bytes())?;
        Ok(false)
    }
}

fn process_file(
    formatter: &Formatter,
    path: &Path,
    cli: &Cli,
) -> Result<bool, Box<dyn std::error::Error>> {
    let source = fs::read_to_string(path)?;
    let formatted = formatter.format(&source)?;

    let changed = source != formatted;

    if cli.check {
        if changed && cli.verbose {
            eprintln!("{}: needs formatting", path.display());
        }
        Ok(changed)
    } else if cli.write {
        if changed {
            fs::write(path, &formatted)?;
            if cli.verbose {
                eprintln!("{}: formatted", path.display());
            }
        } else if cli.verbose {
            eprintln!("{}: already formatted", path.display());
        }
        Ok(changed)
    } else if let Some(ref output_path) = cli.output {
        fs::write(output_path, &formatted)?;
        if cli.verbose {
            eprintln!("Wrote to {}", output_path.display());
        }
        Ok(changed)
    } else {
        io::stdout().write_all(formatted.as_bytes())?;
        Ok(changed)
    }
}
