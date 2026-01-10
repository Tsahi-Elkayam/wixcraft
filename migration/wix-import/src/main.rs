//! wix-import CLI - Import installers from other formats to WiX

use clap::{Parser, Subcommand, ValueEnum};
use std::fs;
use std::path::PathBuf;
use wix_import::{ImportFormat, Importer};

#[derive(Parser)]
#[command(name = "wix-import")]
#[command(about = "Import installers from NSIS, InnoSetup, and other formats to WiX")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Import an installer script to WXS
    Import {
        /// Input file (NSIS .nsi or InnoSetup .iss)
        input: PathBuf,

        /// Output WXS file
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Force specific format (auto-detect if not specified)
        #[arg(short, long, value_enum)]
        format: Option<Format>,

        /// Output parsed info as JSON instead of WXS
        #[arg(long)]
        json: bool,
    },

    /// Detect the format of an installer script
    Detect {
        /// Input file
        input: PathBuf,
    },

    /// Show parsed information from an installer script
    Info {
        /// Input file
        input: PathBuf,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
}

#[derive(Clone, Copy, ValueEnum)]
enum Format {
    Nsis,
    Innosetup,
}

impl From<Format> for ImportFormat {
    fn from(f: Format) -> Self {
        match f {
            Format::Nsis => ImportFormat::Nsis,
            Format::Innosetup => ImportFormat::InnoSetup,
        }
    }
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Import {
            input,
            output,
            format,
            json,
        } => {
            let content = match fs::read_to_string(&input) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("Failed to read {}: {}", input.display(), e);
                    std::process::exit(1);
                }
            };

            let importer = if let Some(fmt) = format {
                Importer::new(fmt.into())
            } else {
                match Importer::auto_detect(&content) {
                    Ok(i) => i,
                    Err(e) => {
                        eprintln!("Could not detect format: {}", e);
                        eprintln!("Use --format to specify the format manually");
                        std::process::exit(1);
                    }
                }
            };

            let info = match importer.parse(&content) {
                Ok(i) => i,
                Err(e) => {
                    eprintln!("Parse error: {}", e);
                    std::process::exit(1);
                }
            };

            if json {
                println!("{}", serde_json::to_string_pretty(&info).unwrap());
                return;
            }

            let wxs = importer.generate_wxs(&info);

            // Show warnings
            if !info.warnings.is_empty() {
                eprintln!("Warnings:");
                for warning in &info.warnings {
                    eprintln!("  ⚠ {}", warning);
                }
                eprintln!();
            }

            if let Some(out_path) = output {
                match fs::write(&out_path, &wxs) {
                    Ok(_) => {
                        println!(
                            "Imported {} ({}) → {}",
                            input.display(),
                            info.source_format,
                            out_path.display()
                        );
                        println!("  Product: {}", info.product_name.as_deref().unwrap_or("N/A"));
                        println!("  Version: {}", info.version.as_deref().unwrap_or("N/A"));
                        println!("  Files: {}", info.files.len());
                        println!("  Shortcuts: {}", info.shortcuts.len());
                        println!("  Registry entries: {}", info.registry.len());
                    }
                    Err(e) => {
                        eprintln!("Failed to write output: {}", e);
                        std::process::exit(1);
                    }
                }
            } else {
                println!("{}", wxs);
            }
        }

        Commands::Detect { input } => {
            let content = match fs::read_to_string(&input) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("Failed to read {}: {}", input.display(), e);
                    std::process::exit(1);
                }
            };

            match ImportFormat::detect(&content) {
                Some(fmt) => {
                    println!("{}: {}", input.display(), fmt.as_str());
                }
                None => {
                    println!("{}: Unknown format", input.display());
                    std::process::exit(1);
                }
            }
        }

        Commands::Info { input, json } => {
            let content = match fs::read_to_string(&input) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("Failed to read {}: {}", input.display(), e);
                    std::process::exit(1);
                }
            };

            let importer = match Importer::auto_detect(&content) {
                Ok(i) => i,
                Err(e) => {
                    eprintln!("Could not detect format: {}", e);
                    std::process::exit(1);
                }
            };

            let info = match importer.parse(&content) {
                Ok(i) => i,
                Err(e) => {
                    eprintln!("Parse error: {}", e);
                    std::process::exit(1);
                }
            };

            if json {
                println!("{}", serde_json::to_string_pretty(&info).unwrap());
            } else {
                println!("Installer Information: {}", input.display());
                println!("═══════════════════════════════════════════════════");
                println!();
                println!("Format:      {}", info.source_format);
                println!(
                    "Product:     {}",
                    info.product_name.as_deref().unwrap_or("N/A")
                );
                println!(
                    "Version:     {}",
                    info.version.as_deref().unwrap_or("N/A")
                );
                println!(
                    "Publisher:   {}",
                    info.publisher.as_deref().unwrap_or("N/A")
                );
                println!(
                    "Install Dir: {}",
                    info.install_dir.as_deref().unwrap_or("N/A")
                );
                println!(
                    "Output File: {}",
                    info.output_file.as_deref().unwrap_or("N/A")
                );
                println!();

                if !info.files.is_empty() {
                    println!("Files ({}):", info.files.len());
                    for file in info.files.iter().take(10) {
                        let recursive = if file.recursive { " (recursive)" } else { "" };
                        println!("  {} → {}{}", file.source, file.destination, recursive);
                    }
                    if info.files.len() > 10 {
                        println!("  ... and {} more", info.files.len() - 10);
                    }
                    println!();
                }

                if !info.shortcuts.is_empty() {
                    println!("Shortcuts ({}):", info.shortcuts.len());
                    for shortcut in &info.shortcuts {
                        println!("  {} → {:?}", shortcut.name, shortcut.location);
                    }
                    println!();
                }

                if !info.registry.is_empty() {
                    println!("Registry ({}):", info.registry.len());
                    for reg in info.registry.iter().take(5) {
                        println!("  {}\\{}", reg.root, reg.key);
                    }
                    if info.registry.len() > 5 {
                        println!("  ... and {} more", info.registry.len() - 5);
                    }
                    println!();
                }

                if !info.warnings.is_empty() {
                    println!("Warnings:");
                    for warning in &info.warnings {
                        println!("  ⚠ {}", warning);
                    }
                }
            }
        }
    }
}
