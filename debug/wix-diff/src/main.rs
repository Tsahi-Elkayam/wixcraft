//! wix-diff CLI - Compare WiX/MSI versions and show changes

use clap::{Parser, Subcommand};
use std::fs;
use std::path::PathBuf;
use wix_diff::{DiffOptions, TextDiff, WixDiff};

#[derive(Parser)]
#[command(name = "wix-diff")]
#[command(about = "Compare WiX/MSI versions and show changes")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Compare two WXS files semantically
    Semantic {
        /// Old/original WXS file
        old: PathBuf,
        /// New/modified WXS file
        new: PathBuf,
        /// Output as JSON
        #[arg(long)]
        json: bool,
        /// Show unchanged elements
        #[arg(long)]
        show_unchanged: bool,
    },
    /// Show unified text diff of two files
    Text {
        /// Old/original file
        old: PathBuf,
        /// New/modified file
        new: PathBuf,
        /// Number of context lines
        #[arg(short = 'C', long, default_value = "3")]
        context: usize,
        /// Show statistics only
        #[arg(long)]
        stats: bool,
    },
    /// Show diff statistics only
    Stats {
        /// Old/original file
        old: PathBuf,
        /// New/modified file
        new: PathBuf,
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Semantic {
            old,
            new,
            json,
            show_unchanged,
        } => {
            let old_content = read_file(&old);
            let new_content = read_file(&new);

            let options = DiffOptions {
                show_unchanged,
                ..Default::default()
            };

            let diff = WixDiff::new(options);
            match diff.compare(&old_content, &new_content) {
                Ok(result) => {
                    if json {
                        println!("{}", serde_json::to_string_pretty(&result).unwrap());
                    } else {
                        println!("WiX Diff: {} → {}", old.display(), new.display());
                        println!("═══════════════════════════════════════════════════");
                        println!();

                        println!(
                            "Summary: {} added, {} removed, {} modified",
                            result.summary.added,
                            result.summary.removed,
                            result.summary.modified
                        );

                        if !result.summary.by_element_type.is_empty() {
                            println!();
                            println!("By Element Type:");
                            for (elem_type, summary) in &result.summary.by_element_type {
                                println!(
                                    "  {}: +{} -{} ~{}",
                                    elem_type, summary.added, summary.removed, summary.modified
                                );
                            }
                        }

                        if !result.changes.is_empty() {
                            println!();
                            println!("Changes:");
                            println!("───────────────────────────────────────────────────");

                            for change in &result.changes {
                                let symbol = match change.change_type {
                                    wix_diff::ChangeType::Added => "+",
                                    wix_diff::ChangeType::Removed => "-",
                                    wix_diff::ChangeType::Modified => "~",
                                    wix_diff::ChangeType::Unchanged => " ",
                                };

                                let id = change
                                    .element_id
                                    .as_ref()
                                    .map(|i| format!(" ({})", i))
                                    .unwrap_or_default();

                                println!("[{}] {}{}", symbol, change.element_type, id);

                                if !change.attribute_changes.is_empty() {
                                    for attr in &change.attribute_changes {
                                        let attr_sym = match attr.change_type {
                                            wix_diff::ChangeType::Added => "+",
                                            wix_diff::ChangeType::Removed => "-",
                                            wix_diff::ChangeType::Modified => "~",
                                            wix_diff::ChangeType::Unchanged => " ",
                                        };

                                        match attr.change_type {
                                            wix_diff::ChangeType::Added => {
                                                println!(
                                                    "    {} {}: {}",
                                                    attr_sym,
                                                    attr.name,
                                                    attr.new_value.as_deref().unwrap_or("")
                                                );
                                            }
                                            wix_diff::ChangeType::Removed => {
                                                println!(
                                                    "    {} {}: {}",
                                                    attr_sym,
                                                    attr.name,
                                                    attr.old_value.as_deref().unwrap_or("")
                                                );
                                            }
                                            wix_diff::ChangeType::Modified => {
                                                println!(
                                                    "    {} {}: {} → {}",
                                                    attr_sym,
                                                    attr.name,
                                                    attr.old_value.as_deref().unwrap_or(""),
                                                    attr.new_value.as_deref().unwrap_or("")
                                                );
                                            }
                                            _ => {}
                                        }
                                    }
                                }
                            }
                        }

                        if result.changes.is_empty() {
                            println!();
                            println!("No differences found.");
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Error: {}", e);
                    std::process::exit(1);
                }
            }
        }

        Commands::Text { old, new, context, stats } => {
            let old_content = read_file(&old);
            let new_content = read_file(&new);

            let diff = TextDiff::new(context);

            if stats {
                let stats = diff.stats(&old_content, &new_content);
                println!("Lines added:     {}", stats.lines_added);
                println!("Lines removed:   {}", stats.lines_removed);
                println!("Lines unchanged: {}", stats.lines_unchanged);
                println!("Change ratio:    {:.1}%", stats.change_percentage());
            } else {
                let output = diff.unified_diff(
                    &old_content,
                    &new_content,
                    &old.display().to_string(),
                    &new.display().to_string(),
                );
                print!("{}", output);
            }
        }

        Commands::Stats { old, new, json } => {
            let old_content = read_file(&old);
            let new_content = read_file(&new);

            let diff = TextDiff::default();
            let stats = diff.stats(&old_content, &new_content);

            if json {
                println!("{}", serde_json::to_string_pretty(&stats).unwrap());
            } else {
                println!("Diff Statistics: {} → {}", old.display(), new.display());
                println!();
                println!("Old file lines:  {}", stats.total_old_lines);
                println!("New file lines:  {}", stats.total_new_lines);
                println!();
                println!("Lines added:     {} (+)", stats.lines_added);
                println!("Lines removed:   {} (-)", stats.lines_removed);
                println!("Lines unchanged: {}", stats.lines_unchanged);
                println!();
                println!("Change ratio:    {:.1}%", stats.change_percentage());
            }
        }
    }
}

fn read_file(path: &PathBuf) -> String {
    match fs::read_to_string(path) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Failed to read {}: {}", path.display(), e);
            std::process::exit(1);
        }
    }
}
