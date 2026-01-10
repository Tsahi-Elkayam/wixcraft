//! wix-repl - Interactive REPL for WiX development

use clap::Parser;
use std::io::{self, BufRead, Write};
use wix_repl::{CommandParser, ReplContext, ReplExecutor};

#[derive(Parser)]
#[command(name = "wix-repl")]
#[command(about = "Interactive REPL for WiX development and testing")]
#[command(version)]
struct Cli {
    /// Execute single command and exit
    #[arg(short, long)]
    command: Option<String>,

    /// Load file on startup
    #[arg(short, long)]
    load: Option<String>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let mut context = ReplContext::new();

    // Load file if specified
    if let Some(ref path) = cli.load {
        context.loaded_files.push(path.clone());
        println!("Loaded: {}", path);
    }

    // Single command mode
    if let Some(ref cmd) = cli.command {
        let command = CommandParser::parse(cmd);
        let result = ReplExecutor::execute(&command, &mut context);
        if let Some(output) = result.output {
            println!("{}", output);
        }
        if let Some(error) = result.error {
            eprintln!("Error: {}", error);
        }
        return Ok(());
    }

    // Interactive mode
    println!("WiX REPL v0.1.0");
    println!("Type 'help' for available commands, 'exit' to quit.\n");

    let stdin = io::stdin();
    let mut stdout = io::stdout();

    loop {
        print!("wix> ");
        stdout.flush()?;

        let mut input = String::new();
        if stdin.lock().read_line(&mut input)? == 0 {
            break; // EOF
        }

        let input = input.trim();
        if input.is_empty() {
            continue;
        }

        context.add_to_history(input);
        let command = CommandParser::parse(input);
        let result = ReplExecutor::execute(&command, &mut context);

        if result.should_clear {
            // Clear screen (simple version)
            print!("\x1B[2J\x1B[1;1H");
            stdout.flush()?;
        }

        if let Some(output) = result.output {
            println!("{}", output);
        }

        if let Some(error) = result.error {
            eprintln!("Error: {}", error);
        }

        if result.should_exit {
            break;
        }
    }

    Ok(())
}
