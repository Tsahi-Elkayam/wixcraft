//! wix-diagnostics CLI - Real-time WiX diagnostics

use clap::{Parser, ValueEnum};
use std::fs;
use std::io::{self, Read};
use std::path::PathBuf;
use std::process::ExitCode;
use walkdir::WalkDir;
use wix_diagnostics::{DiagnosticsEngine, DiagnosticsResult};

#[derive(Parser)]
#[command(name = "wix-diagnostics")]
#[command(about = "Real-time WiX diagnostics and validation")]
#[command(version)]
struct Cli {
    /// WiX file to diagnose (or - for stdin)
    file: String,

    /// Additional files to include for reference validation
    #[arg(long, value_delimiter = ',')]
    include: Option<Vec<PathBuf>>,

    /// Project directory to index for cross-file validation
    #[arg(long)]
    project: Option<PathBuf>,

    /// Validators to run (default: all)
    #[arg(long, value_enum, value_delimiter = ',')]
    validators: Option<Vec<ValidatorKind>>,

    /// Output format
    #[arg(long, value_enum, default_value = "text")]
    format: OutputFormat,

    /// Only show errors (hide warnings and info)
    #[arg(long)]
    errors_only: bool,

    /// Verbose output
    #[arg(short, long)]
    verbose: bool,
}

#[derive(Clone, Copy, ValueEnum, PartialEq)]
enum ValidatorKind {
    References,
    Relationships,
    Attributes,
}

#[derive(Clone, Copy, ValueEnum)]
enum OutputFormat {
    Text,
    Json,
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    match run(cli) {
        Ok(has_errors) => {
            if has_errors {
                ExitCode::FAILURE
            } else {
                ExitCode::SUCCESS
            }
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            ExitCode::FAILURE
        }
    }
}

fn run(cli: Cli) -> Result<bool, Box<dyn std::error::Error>> {
    let mut engine = DiagnosticsEngine::new();

    // Read source
    let (source, file_path) = if cli.file == "-" {
        let mut buffer = String::new();
        io::stdin().read_to_string(&mut buffer)?;
        (buffer, PathBuf::from("stdin.wxs"))
    } else {
        let path = PathBuf::from(&cli.file);
        let content = fs::read_to_string(&path)?;
        (content, path)
    };

    if cli.verbose {
        eprintln!("Diagnosing: {}", file_path.display());
    }

    // Index the main file
    engine.index_file(&source)?;

    // Index additional files
    if let Some(files) = &cli.include {
        for file in files {
            if cli.verbose {
                eprintln!("Indexing: {}", file.display());
            }
            let content = fs::read_to_string(file)?;
            engine.index_file(&content)?;
        }
    }

    // Index project directory
    if let Some(dir) = &cli.project {
        let mut count = 0;
        for entry in WalkDir::new(dir)
            .follow_links(true)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if path.extension().map(|e| e == "wxs").unwrap_or(false) {
                if let Ok(content) = fs::read_to_string(path) {
                    let _ = engine.index_file(&content);
                    count += 1;
                }
            }
        }
        if cli.verbose {
            eprintln!("Indexed {} files from {}", count, dir.display());
        }
    }

    // Run diagnostics
    let mut result = DiagnosticsResult::new(file_path.clone());

    let run_all = cli.validators.is_none();
    let validators = cli.validators.unwrap_or_default();

    if run_all || validators.contains(&ValidatorKind::References) {
        result.extend(engine.diagnose_references(&source, &file_path)?.diagnostics);
    }
    if run_all || validators.contains(&ValidatorKind::Relationships) {
        result.extend(
            engine
                .diagnose_relationships(&source, &file_path)?
                .diagnostics,
        );
    }
    if run_all || validators.contains(&ValidatorKind::Attributes) {
        result.extend(engine.diagnose_attributes(&source, &file_path)?.diagnostics);
    }

    // Filter if errors only
    if cli.errors_only {
        result.diagnostics.retain(|d| {
            d.severity == wix_diagnostics::DiagnosticSeverity::Error
        });
    }

    let has_errors = result.error_count() > 0;

    // Output
    match cli.format {
        OutputFormat::Text => {
            for diag in &result.diagnostics {
                let severity = diag.severity.as_str().to_uppercase();
                let code = diag.code.as_deref().unwrap_or("");
                let code_str = if code.is_empty() {
                    String::new()
                } else {
                    format!("[{}] ", code)
                };

                println!(
                    "{}:{}:{}: {} {}{}",
                    result.file.display(),
                    diag.range.start.line,
                    diag.range.start.character,
                    severity,
                    code_str,
                    diag.message
                );
            }

            if !result.is_empty() {
                println!();
            }

            println!(
                "{} error(s), {} warning(s)",
                result.error_count(),
                result.warning_count()
            );
        }
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
    }

    Ok(has_errors)
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_cli_parse() {
        // Basic test that CLI can be constructed
    }
}
