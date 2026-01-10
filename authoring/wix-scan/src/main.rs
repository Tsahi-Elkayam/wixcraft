//! wix-scan CLI - Directory scanner for WiX

use clap::Parser;
use std::fs;
use std::path::PathBuf;
use wix_scan::{ScanOptions, Scanner};

#[derive(Parser)]
#[command(name = "wix-scan")]
#[command(about = "Directory scanner for WiX - scan directories and generate WXS fragments")]
#[command(version)]
struct Cli {
    /// Directory to scan
    path: PathBuf,

    /// Output file (default: stdout)
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Component group ID
    #[arg(short = 'g', long, default_value = "HarvestedComponents")]
    component_group: String,

    /// Directory reference ID
    #[arg(short = 'd', long, default_value = "INSTALLFOLDER")]
    directory_ref: String,

    /// Component ID prefix
    #[arg(long, default_value = "cmp")]
    prefix: String,

    /// Generate GUIDs (default: use "*")
    #[arg(long)]
    generate_guids: bool,

    /// Include hidden files
    #[arg(long)]
    include_hidden: bool,

    /// Generate 64-bit components
    #[arg(long)]
    win64: bool,

    /// Source path variable (e.g., "SourceDir")
    #[arg(long)]
    source_var: Option<String>,

    /// Exclude patterns (can be used multiple times)
    #[arg(short = 'x', long = "exclude")]
    exclude: Vec<String>,

    /// Include only patterns (can be used multiple times)
    #[arg(short = 'i', long = "include")]
    include: Vec<String>,

    /// Suppress root directory element
    #[arg(long)]
    suppress_root: bool,

    /// Generate registry keys for ref counting
    #[arg(long)]
    registry_keys: bool,

    /// Output as JSON instead of WXS
    #[arg(long)]
    json: bool,

    /// Show statistics only
    #[arg(long)]
    stats: bool,
}

fn main() {
    let cli = Cli::parse();

    let exclude_patterns = if cli.exclude.is_empty() {
        ScanOptions::default().exclude_patterns
    } else {
        cli.exclude
    };

    let options = ScanOptions {
        generate_guids: cli.generate_guids,
        component_group: cli.component_group,
        directory_ref: cli.directory_ref,
        component_prefix: cli.prefix,
        include_hidden: cli.include_hidden,
        exclude_patterns,
        include_patterns: cli.include,
        win64: cli.win64,
        source_var: cli.source_var,
        suppress_root_dir: cli.suppress_root,
        generate_registry_key: cli.registry_keys,
        preserve_structure: true,
    };

    let scanner = Scanner::new(options);

    match scanner.scan(&cli.path) {
        Ok(result) => {
            if cli.stats {
                let stats = result.stats();
                println!("Scan Statistics");
                println!("══════════════════");
                println!("Files:       {}", stats.total_files);
                println!("Directories: {}", stats.total_directories);
                println!("Components:  {}", stats.total_components);
                return;
            }

            let output = if cli.json {
                serde_json::to_string_pretty(&result).unwrap()
            } else {
                result.to_wxs()
            };

            if let Some(output_path) = cli.output {
                match fs::write(&output_path, &output) {
                    Ok(_) => {
                        eprintln!(
                            "Scanned {} files to {}",
                            result.files.len(),
                            output_path.display()
                        );
                    }
                    Err(e) => {
                        eprintln!("Failed to write output: {}", e);
                        std::process::exit(1);
                    }
                }
            } else {
                println!("{}", output);
            }
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}
