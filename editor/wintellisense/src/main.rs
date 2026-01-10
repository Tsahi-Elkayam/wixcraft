//! Wintellisense CLI - Context-aware autocomplete for WiX XML files
//!
//! # Usage
//!
//! ```bash
//! # Get completions at position
//! wintellisense complete file.wxs --line 10 --column 5
//!
//! # Get go-to-definition
//! wintellisense definition file.wxs --line 10 --column 5
//!
//! # Get hover info
//! wintellisense hover file.wxs --line 10 --column 5
//!
//! # Index project and get stats
//! wintellisense index ./src
//! ```

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use wintellisense::{parse_context, Engine, SchemaData};

#[derive(Parser)]
#[command(name = "wintellisense")]
#[command(about = "Context-aware autocomplete engine for WiX XML files")]
#[command(version)]
struct Cli {
    /// Path to wixkb data directory
    #[arg(long, env = "WIXKB_PATH")]
    wixkb: Option<PathBuf>,

    /// Output format (text, json)
    #[arg(long, short, default_value = "text")]
    format: OutputFormat,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Clone, Copy, Default, clap::ValueEnum)]
enum OutputFormat {
    #[default]
    Text,
    Json,
}

#[derive(Subcommand)]
enum Commands {
    /// Get completions at position
    Complete {
        /// Path to WiX file
        file: PathBuf,

        /// Line number (1-based)
        #[arg(long, short)]
        line: u32,

        /// Column number (1-based)
        #[arg(long, short)]
        column: u32,

        /// Maximum completions to return
        #[arg(long, default_value = "50")]
        max: usize,

        /// Project directory to index for cross-file references
        #[arg(long)]
        project: Option<PathBuf>,
    },

    /// Go to definition for symbol at position
    Definition {
        /// Path to WiX file
        file: PathBuf,

        /// Line number (1-based)
        #[arg(long, short)]
        line: u32,

        /// Column number (1-based)
        #[arg(long, short)]
        column: u32,

        /// Project directory to index for cross-file references
        #[arg(long)]
        project: Option<PathBuf>,
    },

    /// Get hover information at position
    Hover {
        /// Path to WiX file
        file: PathBuf,

        /// Line number (1-based)
        #[arg(long, short)]
        line: u32,

        /// Column number (1-based)
        #[arg(long, short)]
        column: u32,
    },

    /// Parse and show cursor context
    Context {
        /// Path to WiX file
        file: PathBuf,

        /// Line number (1-based)
        #[arg(long, short)]
        line: u32,

        /// Column number (1-based)
        #[arg(long, short)]
        column: u32,
    },

    /// Index a project directory
    Index {
        /// Project directory to index
        path: PathBuf,

        /// Show all indexed symbols
        #[arg(long)]
        verbose: bool,
    },

    /// Show engine statistics
    Stats {
        /// Project directory to index for stats
        #[arg(long)]
        project: Option<PathBuf>,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let schema = if let Some(ref wixkb_path) = cli.wixkb {
        SchemaData::load(wixkb_path)?
    } else {
        // Try to find wixkb in common locations
        let paths = [
            PathBuf::from("../wixkb"),
            PathBuf::from("../../common/wixkb"),
            PathBuf::from("../../../common/wixkb"),
        ];

        let mut found_schema = None;
        for path in &paths {
            if path.exists() {
                if let Ok(schema) = SchemaData::load(path) {
                    found_schema = Some(schema);
                    break;
                }
            }
        }

        found_schema.unwrap_or_default()
    };

    let mut engine = Engine::with_schema(schema);

    match cli.command {
        Commands::Complete {
            file,
            line,
            column,
            max,
            project,
        } => {
            if let Some(project_path) = project {
                engine.index_project(&project_path)?;
            }

            let engine = engine.with_max_completions(max);
            let source = std::fs::read_to_string(&file)?;
            let result = engine.complete(&source, line, column);

            match cli.format {
                OutputFormat::Json => {
                    println!("{}", serde_json::to_string_pretty(&result)?);
                }
                OutputFormat::Text => {
                    if result.items.is_empty() {
                        println!("No completions found");
                    } else {
                        println!("Completions ({}):", result.items.len());
                        for item in &result.items {
                            let kind = format!("{:?}", item.kind);
                            let required = if item.required { " [required]" } else { "" };
                            let detail = item.detail.as_deref().unwrap_or("");
                            println!(
                                "  {:20} {:10} {:40}{}",
                                item.label, kind, detail, required
                            );
                        }
                    }
                }
            }
        }

        Commands::Definition {
            file,
            line,
            column,
            project,
        } => {
            if let Some(project_path) = project {
                engine.index_project(&project_path)?;
            }

            let source = std::fs::read_to_string(&file)?;
            let result = engine.go_to_definition(&source, line, column);

            match cli.format {
                OutputFormat::Json => {
                    println!("{}", serde_json::to_string_pretty(&result)?);
                }
                OutputFormat::Text => {
                    if result.definitions.is_empty() {
                        println!("No definition found");
                    } else {
                        for def in &result.definitions {
                            println!(
                                "{} ({}) at {}:{}:{}",
                                def.name,
                                def.kind,
                                def.location.path.display(),
                                def.location.start.line,
                                def.location.start.column
                            );
                            if let Some(ref preview) = def.preview {
                                println!("  {}", preview);
                            }
                        }
                    }
                }
            }
        }

        Commands::Hover { file, line, column } => {
            let source = std::fs::read_to_string(&file)?;
            let result = engine.hover(&source, line, column);

            match cli.format {
                OutputFormat::Json => {
                    println!("{}", serde_json::to_string_pretty(&result)?);
                }
                OutputFormat::Text => {
                    if let Some(info) = result.info {
                        println!("{}", info.content);
                    } else {
                        println!("No hover information");
                    }
                }
            }
        }

        Commands::Context { file, line, column } => {
            let source = std::fs::read_to_string(&file)?;
            let ctx = parse_context(&source, line, column);

            match cli.format {
                OutputFormat::Json => {
                    #[derive(serde::Serialize)]
                    struct ContextOutput {
                        parent_element: Option<String>,
                        current_element: Option<String>,
                        current_attribute: Option<String>,
                        in_opening_tag: bool,
                        in_attribute_value: bool,
                        in_element_content: bool,
                        prefix: String,
                        word_at_cursor: Option<String>,
                        existing_attributes: Vec<String>,
                    }

                    let output = ContextOutput {
                        parent_element: ctx.parent_element.clone(),
                        current_element: ctx.current_element.clone(),
                        current_attribute: ctx.current_attribute.clone(),
                        in_opening_tag: ctx.in_opening_tag,
                        in_attribute_value: ctx.in_attribute_value,
                        in_element_content: ctx.in_element_content,
                        prefix: ctx.prefix.clone(),
                        word_at_cursor: ctx.word_at_cursor.clone(),
                        existing_attributes: ctx.existing_attributes.clone(),
                    };

                    println!("{}", serde_json::to_string_pretty(&output)?);
                }
                OutputFormat::Text => {
                    println!("Cursor Context at {}:{}", line, column);
                    println!("  Parent element:    {:?}", ctx.parent_element);
                    println!("  Current element:   {:?}", ctx.current_element);
                    println!("  Current attribute: {:?}", ctx.current_attribute);
                    println!("  In opening tag:    {}", ctx.in_opening_tag);
                    println!("  In attribute value:{}", ctx.in_attribute_value);
                    println!("  In element content:{}", ctx.in_element_content);
                    println!("  Prefix:            {:?}", ctx.prefix);
                    println!("  Word at cursor:    {:?}", ctx.word_at_cursor);
                    println!("  Existing attrs:    {:?}", ctx.existing_attributes);
                    println!();
                    println!("Should suggest:");
                    println!("  Elements:   {}", ctx.should_suggest_elements());
                    println!("  Attributes: {}", ctx.should_suggest_attributes());
                    println!("  Values:     {}", ctx.should_suggest_values());
                }
            }
        }

        Commands::Index { path, verbose } => {
            let count = engine.index_project(&path)?;
            let stats = engine.stats();

            match cli.format {
                OutputFormat::Json => {
                    #[derive(serde::Serialize)]
                    struct IndexOutput {
                        files_indexed: usize,
                        symbols_found: usize,
                    }

                    let output = IndexOutput {
                        files_indexed: count,
                        symbols_found: stats.indexed_symbols,
                    };

                    println!("{}", serde_json::to_string_pretty(&output)?);
                }
                OutputFormat::Text => {
                    println!("Indexed {} files", count);
                    println!("Found {} symbols", stats.indexed_symbols);

                    if verbose {
                        let index = engine.index();
                        for kind in ["Component", "Directory", "Feature", "Property", "CustomAction"]
                        {
                            let symbols = index.get_symbols_by_kind(kind);
                            if !symbols.is_empty() {
                                println!("\n{}s:", kind);
                                for sym in symbols {
                                    println!(
                                        "  {} at {}:{}",
                                        sym.name,
                                        sym.location.path.display(),
                                        sym.location.start.line
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }

        Commands::Stats { project } => {
            if let Some(project_path) = project {
                engine.index_project(&project_path)?;
            }

            let stats = engine.stats();

            match cli.format {
                OutputFormat::Json => {
                    #[derive(serde::Serialize)]
                    struct StatsOutput {
                        elements: usize,
                        snippets: usize,
                        indexed_files: usize,
                        indexed_symbols: usize,
                    }

                    let output = StatsOutput {
                        elements: stats.elements,
                        snippets: stats.snippets,
                        indexed_files: stats.indexed_files,
                        indexed_symbols: stats.indexed_symbols,
                    };

                    println!("{}", serde_json::to_string_pretty(&output)?);
                }
                OutputFormat::Text => {
                    println!("Engine Statistics:");
                    println!("  Schema elements:  {}", stats.elements);
                    println!("  Snippets:         {}", stats.snippets);
                    println!("  Indexed files:    {}", stats.indexed_files);
                    println!("  Indexed symbols:  {}", stats.indexed_symbols);
                }
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_parsing() {
        let cli = Cli::try_parse_from([
            "wintellisense",
            "complete",
            "test.wxs",
            "--line",
            "10",
            "--column",
            "5",
        ]);
        assert!(cli.is_ok());
    }

    #[test]
    fn test_cli_with_format() {
        let cli = Cli::try_parse_from([
            "wintellisense",
            "--format",
            "json",
            "hover",
            "test.wxs",
            "-l",
            "1",
            "-c",
            "1",
        ]);
        assert!(cli.is_ok());
    }
}
