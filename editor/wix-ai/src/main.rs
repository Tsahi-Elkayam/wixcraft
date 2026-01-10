//! wix-ai CLI - AI-assisted WiX code generation
//!
//! Usage:
//!   wix-ai generate "create installer for MyApp"
//!   wix-ai template basic_installer --var NAME=MyApp
//!   wix-ai suggest "<Component"
//!   wix-ai list

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::collections::HashMap;
use wix_ai::*;

#[derive(Parser)]
#[command(name = "wix-ai")]
#[command(about = "AI-assisted WiX code generation")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate WiX code from natural language prompt
    Generate {
        /// Natural language description
        prompt: String,

        /// Variable assignments (KEY=VALUE)
        #[arg(short, long)]
        var: Vec<String>,

        /// Output format (text, json)
        #[arg(short, long, default_value = "text")]
        format: String,
    },

    /// Use a specific template
    Template {
        /// Template name
        name: String,

        /// Variable assignments (KEY=VALUE)
        #[arg(short, long)]
        var: Vec<String>,
    },

    /// Suggest completions for partial code
    Suggest {
        /// Partial WiX code context
        context: String,
    },

    /// List available templates
    List,

    /// Interactive mode
    Interactive,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let ai = WixAi::new();

    match cli.command {
        Commands::Generate { prompt, var, format } => {
            let variables = parse_variables(&var);
            let result = ai.generate(&prompt, &variables);

            match format.as_str() {
                "json" => {
                    println!("{}", serde_json::to_string_pretty(&result)?);
                }
                _ => {
                    println!("Intent: {:?}", result.intent);
                    println!();
                    println!("{}", result.explanation);
                    println!();
                    println!("Generated Code:");
                    println!("{}", "-".repeat(50));
                    println!("{}", result.code);
                    println!("{}", "-".repeat(50));

                    if !result.suggestions.is_empty() {
                        println!();
                        println!("Suggestions:");
                        for suggestion in &result.suggestions {
                            println!("  - {}", suggestion);
                        }
                    }

                    if !result.references.is_empty() {
                        println!();
                        println!("References:");
                        for reference in &result.references {
                            println!("  - {}", reference);
                        }
                    }
                }
            }
        }

        Commands::Template { name, var } => {
            match ai.get_template(&name) {
                Some(template) => {
                    let variables = parse_variables(&var);
                    let mut code = template.code.to_string();

                    for (key, value) in &variables {
                        code = code.replace(&format!("{{{{{}}}}}", key), value);
                    }

                    println!("Template: {}", template.name);
                    println!("Description: {}", template.description);
                    println!();
                    println!("Variables:");
                    for v in template.variables {
                        let status = if variables.contains_key(*v) { "provided" } else { "needed" };
                        println!("  {} - {}", v, status);
                    }
                    println!();
                    println!("Code:");
                    println!("{}", "-".repeat(50));
                    println!("{}", code);
                    println!("{}", "-".repeat(50));
                }
                None => {
                    eprintln!("Template '{}' not found.", name);
                    eprintln!();
                    eprintln!("Available templates:");
                    for (name, desc) in ai.list_templates() {
                        eprintln!("  {} - {}", name, desc);
                    }
                    std::process::exit(1);
                }
            }
        }

        Commands::Suggest { context } => {
            let suggestions = ai.suggest_completions(&context);

            if suggestions.is_empty() {
                println!("No suggestions available for this context.");
            } else {
                println!("Suggestions:");
                for (i, suggestion) in suggestions.iter().enumerate() {
                    println!("{}. {}", i + 1, suggestion);
                }
            }
        }

        Commands::List => {
            println!("Available Templates");
            println!("{}", "=".repeat(50));
            println!();

            for (name, description) in ai.list_templates() {
                println!("{}", name);
                println!("  {}", description);
                println!();
            }

            println!("Usage:");
            println!("  wix-ai template <name> --var KEY=VALUE");
            println!();
            println!("Example:");
            println!("  wix-ai template basic_installer --var NAME=MyApp --var VERSION=1.0.0");
        }

        Commands::Interactive => {
            println!("WiX AI Interactive Mode");
            println!("{}", "=".repeat(50));
            println!();
            println!("Describe what you want to create in natural language.");
            println!("Examples:");
            println!("  - create new installer project");
            println!("  - add start menu shortcut");
            println!("  - install windows service");
            println!("  - add registry key");
            println!();
            println!("Type 'quit' to exit, 'list' to see templates.");
            println!();

            let stdin = std::io::stdin();
            loop {
                print!("> ");
                use std::io::Write;
                std::io::stdout().flush()?;

                let mut input = String::new();
                stdin.read_line(&mut input)?;
                let input = input.trim();

                if input.eq_ignore_ascii_case("quit") || input.eq_ignore_ascii_case("exit") {
                    break;
                }

                if input.eq_ignore_ascii_case("list") {
                    for (name, desc) in ai.list_templates() {
                        println!("  {} - {}", name, desc);
                    }
                    continue;
                }

                if input.is_empty() {
                    continue;
                }

                let result = ai.generate(input, &HashMap::new());
                println!();
                println!("Intent: {:?}", result.intent);
                println!();
                if !result.code.is_empty() {
                    println!("{}", result.code);
                } else {
                    println!("{}", result.explanation);
                }
                println!();

                if !result.suggestions.is_empty() {
                    for suggestion in &result.suggestions {
                        println!("Tip: {}", suggestion);
                    }
                    println!();
                }
            }
        }
    }

    Ok(())
}

fn parse_variables(vars: &[String]) -> HashMap<String, String> {
    let mut map = HashMap::new();
    for var in vars {
        if let Some((key, value)) = var.split_once('=') {
            map.insert(key.to_uppercase(), value.to_string());
        }
    }
    map
}
