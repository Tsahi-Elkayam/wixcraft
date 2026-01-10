//! wix-help CLI - PowerShell-like help for WiX

use clap::{Parser, Subcommand};
use std::path::PathBuf;
use std::process::ExitCode;
use wix_help::{HelpSystem, OutputFormat};

#[derive(Parser)]
#[command(name = "wix-help")]
#[command(about = "PowerShell-like help for WiX elements, errors, and snippets")]
#[command(version)]
struct Cli {
    /// Path to wix-data directory
    #[arg(long, env = "WIX_DATA_PATH")]
    wix_data: Option<PathBuf>,

    /// Output format (text, json, markdown)
    #[arg(short, long, default_value = "text")]
    format: String,

    #[command(subcommand)]
    command: Option<Commands>,

    /// Topic to get help for (element, error, snippet, or rule)
    topic: Option<String>,

    /// Show examples
    #[arg(short, long)]
    examples: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Get help for a WiX element
    Element {
        /// Element name
        name: String,
        /// Show examples
        #[arg(short, long)]
        examples: bool,
    },
    /// Get help for a WiX error code
    Error {
        /// Error code (e.g., WIX0001, ICE03)
        code: String,
    },
    /// Get help for a code snippet
    Snippet {
        /// Snippet name or prefix
        name: String,
    },
    /// Get help for a lint rule
    Rule {
        /// Rule ID
        id: String,
    },
    /// List available topics
    List {
        /// Type of items to list (elements, snippets, errors, rules)
        #[arg(default_value = "elements")]
        item_type: String,
    },
    /// Search for topics
    Search {
        /// Search query
        query: String,
        /// Type to search (elements, snippets, errors, rules, all)
        #[arg(short, long, default_value = "all")]
        item_type: String,
    },
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    // Parse output format
    let format = match cli.format.parse::<OutputFormat>() {
        Ok(f) => f,
        Err(e) => {
            eprintln!("Error: {}", e);
            return ExitCode::FAILURE;
        }
    };

    // Find wix-data directory
    let wix_data_path = match find_wix_data(&cli.wix_data) {
        Some(path) => path,
        None => {
            eprintln!("Error: Could not find wix-data directory.");
            eprintln!("Please specify --wix-data or set WIX_DATA_PATH environment variable.");
            return ExitCode::FAILURE;
        }
    };

    // Load help system
    let help = match HelpSystem::load(&wix_data_path) {
        Ok(h) => h,
        Err(e) => {
            eprintln!("Error loading knowledge base: {}", e);
            return ExitCode::FAILURE;
        }
    };

    // Process command
    match cli.command {
        Some(Commands::Element { name, examples }) => {
            if let Some(output) = help.get_element_help(&name, format, examples) {
                println!("{}", output);
            } else {
                eprintln!("Element '{}' not found.", name);
                suggest_similar(&help, &name, "elements");
                return ExitCode::FAILURE;
            }
        }
        Some(Commands::Error { code }) => {
            if let Some(output) = help.get_error_help(&code, format) {
                println!("{}", output);
            } else {
                eprintln!("Error code '{}' not found.", code);
                return ExitCode::FAILURE;
            }
        }
        Some(Commands::Snippet { name }) => {
            if let Some(output) = help.get_snippet_help(&name, format) {
                println!("{}", output);
            } else {
                eprintln!("Snippet '{}' not found.", name);
                suggest_similar(&help, &name, "snippets");
                return ExitCode::FAILURE;
            }
        }
        Some(Commands::Rule { id }) => {
            if let Some(output) = help.get_rule_help(&id, format) {
                println!("{}", output);
            } else {
                eprintln!("Rule '{}' not found.", id);
                suggest_similar(&help, &id, "rules");
                return ExitCode::FAILURE;
            }
        }
        Some(Commands::List { item_type }) => {
            let output = match item_type.to_lowercase().as_str() {
                "elements" | "element" => help.list_elements(format),
                "snippets" | "snippet" => help.list_snippets(format),
                "errors" | "error" => help.list_errors(format),
                "rules" | "rule" => help.list_rules(format),
                _ => {
                    eprintln!(
                        "Unknown type '{}'. Use: elements, snippets, errors, or rules.",
                        item_type
                    );
                    return ExitCode::FAILURE;
                }
            };
            println!("{}", output);
        }
        Some(Commands::Search { query, item_type }) => {
            let output = match item_type.to_lowercase().as_str() {
                "elements" | "element" => help.search_elements(&query, format),
                "snippets" | "snippet" => help.search_snippets(&query, format),
                "errors" | "error" => help.search_errors(&query, format),
                "rules" | "rule" => help.search_rules(&query, format),
                "all" => {
                    let mut results = Vec::new();
                    results.push(help.search_elements(&query, format));
                    results.push(help.search_snippets(&query, format));
                    results.push(help.search_errors(&query, format));
                    results.push(help.search_rules(&query, format));
                    results.join("\n")
                }
                _ => {
                    eprintln!(
                        "Unknown type '{}'. Use: elements, snippets, errors, rules, or all.",
                        item_type
                    );
                    return ExitCode::FAILURE;
                }
            };
            println!("{}", output);
        }
        None => {
            // If topic provided, try to get help for it
            if let Some(topic) = cli.topic {
                if let Some(output) = help.get_help(&topic, format, cli.examples) {
                    println!("{}", output);
                } else {
                    eprintln!("Topic '{}' not found.", topic);
                    suggest_similar(&help, &topic, "all");
                    return ExitCode::FAILURE;
                }
            } else {
                // No command or topic - show usage
                println!("wix-help - PowerShell-like help for WiX");
                println!();
                println!("USAGE:");
                println!("  wix-help <TOPIC>           Get help for a topic (auto-detected)");
                println!("  wix-help element <NAME>    Get help for an element");
                println!("  wix-help error <CODE>      Get help for an error code");
                println!("  wix-help snippet <NAME>    Get help for a snippet");
                println!("  wix-help rule <ID>         Get help for a lint rule");
                println!("  wix-help list <TYPE>       List available topics");
                println!("  wix-help search <QUERY>    Search for topics");
                println!();
                println!("OPTIONS:");
                println!("  -f, --format <FORMAT>      Output format (text, json, markdown)");
                println!("  -e, --examples             Show examples");
                println!("  --wix-data <PATH>          Path to wix-data directory");
                println!();
                println!("EXAMPLES:");
                println!("  wix-help Component");
                println!("  wix-help element Component --examples");
                println!("  wix-help error WIX0001");
                println!("  wix-help list elements");
                println!("  wix-help search upgrade");
            }
        }
    }

    ExitCode::SUCCESS
}

/// Find wix-data directory
fn find_wix_data(explicit: &Option<PathBuf>) -> Option<PathBuf> {
    // Use explicit path if provided
    if let Some(path) = explicit {
        if path.exists() {
            return Some(path.clone());
        }
    }

    // Try common locations relative to current directory
    let candidates = [
        "wix-data",
        "../wix-data",
        "../../wix-data",
        "../../../wix-data",
        "src/core/wix-data",
        "../core/wix-data",
        "../../core/wix-data",
        "../../../core/wix-data",
    ];

    for candidate in &candidates {
        let path = PathBuf::from(candidate);
        if path.exists() && path.join("elements").exists() {
            return Some(path);
        }
    }

    None
}

/// Suggest similar items when a lookup fails
fn suggest_similar(help: &HelpSystem, query: &str, item_type: &str) {
    let suggestions: Vec<String> = match item_type {
        "elements" => help
            .kb()
            .search_elements(query)
            .iter()
            .take(3)
            .map(|e| e.name.clone())
            .collect(),
        "snippets" => help
            .kb()
            .search_snippets(query)
            .iter()
            .take(3)
            .map(|s| s.name.clone())
            .collect(),
        "rules" => help
            .kb()
            .search_rules(query)
            .iter()
            .take(3)
            .map(|r| r.id.clone())
            .collect(),
        "all" => {
            let mut all = Vec::new();
            all.extend(
                help.kb()
                    .search_elements(query)
                    .iter()
                    .take(2)
                    .map(|e| e.name.clone()),
            );
            all.extend(
                help.kb()
                    .search_snippets(query)
                    .iter()
                    .take(2)
                    .map(|s| s.name.clone()),
            );
            all
        }
        _ => Vec::new(),
    };

    if !suggestions.is_empty() {
        eprintln!("Did you mean: {}?", suggestions.join(", "));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_wix_data_none() {
        let result = find_wix_data(&None);
        // This will depend on the current directory
        // Just verify it doesn't panic
        let _ = result;
    }

    #[test]
    fn test_find_wix_data_explicit_not_found() {
        // When explicit path doesn't exist, function falls back to searching
        // Just verify it doesn't panic
        let path = PathBuf::from("/nonexistent/path/that/really/does/not/exist/xyz");
        let result = find_wix_data(&Some(path));
        // Result depends on whether we can find wix-data via relative paths
        let _ = result;
    }

    #[test]
    fn test_cli_parse() {
        let cli = Cli::parse_from(["wix-help", "Component"]);
        assert_eq!(cli.topic, Some("Component".to_string()));
    }

    #[test]
    fn test_cli_parse_with_format() {
        let cli = Cli::parse_from(["wix-help", "-f", "json", "Component"]);
        assert_eq!(cli.format, "json");
    }

    #[test]
    fn test_cli_parse_element_command() {
        let cli = Cli::parse_from(["wix-help", "element", "Component", "--examples"]);
        match cli.command {
            Some(Commands::Element { name, examples }) => {
                assert_eq!(name, "Component");
                assert!(examples);
            }
            _ => panic!("Expected Element command"),
        }
    }

    #[test]
    fn test_cli_parse_error_command() {
        let cli = Cli::parse_from(["wix-help", "error", "WIX0001"]);
        match cli.command {
            Some(Commands::Error { code }) => {
                assert_eq!(code, "WIX0001");
            }
            _ => panic!("Expected Error command"),
        }
    }

    #[test]
    fn test_cli_parse_list_command() {
        let cli = Cli::parse_from(["wix-help", "list", "elements"]);
        match cli.command {
            Some(Commands::List { item_type }) => {
                assert_eq!(item_type, "elements");
            }
            _ => panic!("Expected List command"),
        }
    }

    #[test]
    fn test_cli_parse_search_command() {
        let cli = Cli::parse_from(["wix-help", "search", "upgrade", "--item-type", "all"]);
        match cli.command {
            Some(Commands::Search { query, item_type }) => {
                assert_eq!(query, "upgrade");
                assert_eq!(item_type, "all");
            }
            _ => panic!("Expected Search command"),
        }
    }

    #[test]
    fn test_cli_parse_snippet_command() {
        let cli = Cli::parse_from(["wix-help", "snippet", "comp"]);
        match cli.command {
            Some(Commands::Snippet { name }) => {
                assert_eq!(name, "comp");
            }
            _ => panic!("Expected Snippet command"),
        }
    }

    #[test]
    fn test_cli_parse_rule_command() {
        let cli = Cli::parse_from(["wix-help", "rule", "component-requires-guid"]);
        match cli.command {
            Some(Commands::Rule { id }) => {
                assert_eq!(id, "component-requires-guid");
            }
            _ => panic!("Expected Rule command"),
        }
    }
}
