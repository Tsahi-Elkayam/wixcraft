//! wix-references CLI - WiX symbol references and Go to Definition

use clap::{Parser, Subcommand, ValueEnum};
use std::fs;
use std::io::{self, Read};
use std::path::PathBuf;
use std::process::ExitCode;
use wix_references::{
    find_definition_by_id, find_references, find_references_by_id, go_to_definition, SymbolIndex,
};

#[derive(Parser)]
#[command(name = "wix-references")]
#[command(about = "WiX symbol references and Go to Definition")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Go to definition at a position
    Definition {
        /// WiX file (or - for stdin)
        file: String,

        /// Line number (1-based)
        #[arg(long)]
        line: u32,

        /// Column number (1-based)
        #[arg(long)]
        column: u32,

        /// Additional files to include in index
        #[arg(long, value_delimiter = ',')]
        include: Option<Vec<PathBuf>>,

        /// Directory to index for cross-file references
        #[arg(long)]
        project: Option<PathBuf>,

        /// Output format
        #[arg(long, value_enum, default_value = "text")]
        format: OutputFormat,
    },

    /// Find all references to a symbol
    References {
        /// WiX file (or - for stdin)
        file: String,

        /// Line number (1-based)
        #[arg(long)]
        line: u32,

        /// Column number (1-based)
        #[arg(long)]
        column: u32,

        /// Include the definition in results
        #[arg(long)]
        include_definition: bool,

        /// Additional files to include in index
        #[arg(long, value_delimiter = ',')]
        include: Option<Vec<PathBuf>>,

        /// Directory to index for cross-file references
        #[arg(long)]
        project: Option<PathBuf>,

        /// Output format
        #[arg(long, value_enum, default_value = "text")]
        format: OutputFormat,
    },

    /// Find definition by id directly
    FindDefinition {
        /// Element type (Component, Directory, Feature, etc.)
        #[arg(long, short = 't')]
        element_type: String,

        /// Symbol id
        #[arg(long, short)]
        id: String,

        /// Files to index
        files: Vec<PathBuf>,

        /// Directory to index
        #[arg(long)]
        project: Option<PathBuf>,

        /// Output format
        #[arg(long, value_enum, default_value = "text")]
        format: OutputFormat,
    },

    /// Find references by id directly
    FindReferences {
        /// Element type (Component, Directory, Feature, etc.)
        #[arg(long, short = 't')]
        element_type: String,

        /// Symbol id
        #[arg(long, short)]
        id: String,

        /// Files to index
        files: Vec<PathBuf>,

        /// Directory to index
        #[arg(long)]
        project: Option<PathBuf>,

        /// Include definition in results
        #[arg(long)]
        include_definition: bool,

        /// Output format
        #[arg(long, value_enum, default_value = "text")]
        format: OutputFormat,
    },

    /// Index a project and show statistics
    Index {
        /// Directory to index
        dir: PathBuf,

        /// Output format
        #[arg(long, value_enum, default_value = "text")]
        format: OutputFormat,

        /// Show all symbols
        #[arg(long)]
        show_all: bool,
    },
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
    match cli.command {
        Command::Definition {
            file,
            line,
            column,
            include,
            project,
            format,
        } => {
            let (source, file_path) = read_source(&file)?;
            let mut index = SymbolIndex::new();

            // Index the main file
            index.index_file(&file_path, &source)?;

            // Index additional files
            if let Some(files) = include {
                index.index_files(&files)?;
            }

            // Index project directory
            if let Some(dir) = project {
                index.index_directory(&dir)?;
            }

            let result = go_to_definition(&source, line, column, &index);

            match format {
                OutputFormat::Text => {
                    if let Some(def) = &result.definition {
                        println!(
                            "{}:{}:{} - {} {}",
                            def.location.file.display(),
                            def.location.range.start.line,
                            def.location.range.start.character,
                            def.kind.element_name(),
                            def.id
                        );
                    } else if let Some(err) = &result.error {
                        println!("{}", err);
                    }
                }
                OutputFormat::Json => {
                    println!("{}", serde_json::to_string_pretty(&result)?);
                }
            }
        }

        Command::References {
            file,
            line,
            column,
            include_definition,
            include,
            project,
            format,
        } => {
            let (source, file_path) = read_source(&file)?;
            let mut index = SymbolIndex::new();

            index.index_file(&file_path, &source)?;

            if let Some(files) = include {
                index.index_files(&files)?;
            }

            if let Some(dir) = project {
                index.index_directory(&dir)?;
            }

            let result = find_references(&source, line, column, &index, include_definition);

            match result {
                Some(refs) => match format {
                    OutputFormat::Text => {
                        if include_definition {
                            if let Some(def) = &refs.definition {
                                println!(
                                    "[definition] {}:{}:{} - {}",
                                    def.location.file.display(),
                                    def.location.range.start.line,
                                    def.location.range.start.character,
                                    def.id
                                );
                            }
                        }
                        for r in &refs.references {
                            println!(
                                "{}:{}:{} - {} {}",
                                r.location.file.display(),
                                r.location.range.start.line,
                                r.location.range.start.character,
                                r.kind.element_name(),
                                r.id
                            );
                        }
                        println!("\n{} reference(s) found", refs.count);
                    }
                    OutputFormat::Json => {
                        println!("{}", serde_json::to_string_pretty(&refs)?);
                    }
                },
                None => {
                    println!("No symbol at this position");
                }
            }
        }

        Command::FindDefinition {
            element_type,
            id,
            files,
            project,
            format,
        } => {
            let mut index = SymbolIndex::new();

            index.index_files(&files)?;

            if let Some(dir) = project {
                index.index_directory(&dir)?;
            }

            let result = find_definition_by_id(&element_type, &id, &index);

            match format {
                OutputFormat::Text => {
                    if let Some(def) = &result.definition {
                        println!(
                            "{}:{}:{}",
                            def.location.file.display(),
                            def.location.range.start.line,
                            def.location.range.start.character
                        );
                    } else if let Some(err) = &result.error {
                        println!("{}", err);
                    }
                }
                OutputFormat::Json => {
                    println!("{}", serde_json::to_string_pretty(&result)?);
                }
            }
        }

        Command::FindReferences {
            element_type,
            id,
            files,
            project,
            include_definition,
            format,
        } => {
            let mut index = SymbolIndex::new();

            index.index_files(&files)?;

            if let Some(dir) = project {
                index.index_directory(&dir)?;
            }

            let result = find_references_by_id(&element_type, &id, &index, include_definition);

            match format {
                OutputFormat::Text => {
                    for r in &result.references {
                        println!(
                            "{}:{}:{}",
                            r.location.file.display(),
                            r.location.range.start.line,
                            r.location.range.start.character
                        );
                    }
                    println!("\n{} reference(s) found", result.count);
                }
                OutputFormat::Json => {
                    println!("{}", serde_json::to_string_pretty(&result)?);
                }
            }
        }

        Command::Index {
            dir,
            format,
            show_all,
        } => {
            let mut index = SymbolIndex::new();
            let count = index.index_directory(&dir)?;

            match format {
                OutputFormat::Text => {
                    println!("Indexed {} files", count);
                    println!("  {} definitions", index.definition_count());
                    println!("  {} references", index.reference_count());

                    let missing = index.find_missing_definitions();
                    if !missing.is_empty() {
                        println!("\nMissing definitions:");
                        for (r, kind) in &missing {
                            println!(
                                "  {} {} - {}:{}",
                                kind,
                                r.id,
                                r.location.file.display(),
                                r.location.range.start.line
                            );
                        }
                    }

                    if show_all {
                        println!("\nDefinitions:");
                        for def in index.all_definitions() {
                            println!(
                                "  {} {} - {}:{}",
                                def.kind.element_name(),
                                def.id,
                                def.location.file.display(),
                                def.location.range.start.line
                            );
                        }
                    }
                }
                OutputFormat::Json => {
                    let stats = serde_json::json!({
                        "files": count,
                        "definitions": index.definition_count(),
                        "references": index.reference_count(),
                        "missing": index.find_missing_definitions().len()
                    });
                    println!("{}", serde_json::to_string_pretty(&stats)?);
                }
            }
        }
    }

    Ok(())
}

/// Read source from file or stdin
fn read_source(file: &str) -> Result<(String, PathBuf), Box<dyn std::error::Error>> {
    if file == "-" {
        let mut buffer = String::new();
        io::stdin().read_to_string(&mut buffer)?;
        Ok((buffer, PathBuf::from("stdin.wxs")))
    } else {
        let path = PathBuf::from(file);
        let source = fs::read_to_string(&path)?;
        Ok((source, path))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_source_from_file() {
        // This would need a real file to test properly
    }
}
