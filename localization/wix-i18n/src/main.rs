//! wix-i18n CLI - Localization helper for WiX installers

use clap::{Parser, Subcommand};
use std::fs;
use std::path::PathBuf;
use wix_i18n::{Language, LocalizationManager, StringEntry};

#[derive(Parser)]
#[command(name = "wix-i18n")]
#[command(about = "Localization helper for WiX installers")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate a WXL template with common strings
    Template {
        /// Target language (e.g., en, fr, de)
        language: String,

        /// Output file (default: stdout)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// List all supported languages
    Languages {
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Check translation coverage
    Check {
        /// WXL files to check
        files: Vec<PathBuf>,

        /// Base language for comparison
        #[arg(short, long, default_value = "en")]
        base: String,
    },

    /// Find missing translations
    Missing {
        /// Base WXL file (e.g., en-US.wxl)
        base: PathBuf,

        /// Target WXL file to check
        target: PathBuf,
    },

    /// Extract strings from WXS files
    Extract {
        /// WXS files to extract from
        files: Vec<PathBuf>,

        /// Output WXL file
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Target language
        #[arg(short, long, default_value = "en")]
        language: String,
    },

    /// Merge multiple WXL files
    Merge {
        /// WXL files to merge
        files: Vec<PathBuf>,

        /// Output file
        #[arg(short, long)]
        output: PathBuf,
    },

    /// Convert between formats (WXL, JSON)
    Convert {
        /// Input file
        input: PathBuf,

        /// Output file
        output: PathBuf,

        /// Output format (wxl, json)
        #[arg(short, long, default_value = "wxl")]
        format: String,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Template { language, output } => {
            let lang = match Language::from_str(&language) {
                Some(l) => l,
                None => {
                    eprintln!("Unknown language: {}", language);
                    eprintln!("Use 'wix-i18n languages' to see available languages");
                    std::process::exit(1);
                }
            };

            let mgr = LocalizationManager::new().with_common_strings(lang);
            let wxl = mgr.generate_wxl(lang);

            if let Some(path) = output {
                match fs::write(&path, &wxl) {
                    Ok(_) => println!("Generated {} template: {}", lang.name(), path.display()),
                    Err(e) => {
                        eprintln!("Failed to write file: {}", e);
                        std::process::exit(1);
                    }
                }
            } else {
                println!("{}", wxl);
            }
        }

        Commands::Languages { json } => {
            let languages: Vec<_> = Language::all()
                .into_iter()
                .map(|l| LanguageInfo {
                    code: l.code().to_string(),
                    name: l.name().to_string(),
                    culture: l.culture().to_string(),
                    lcid: l.lcid(),
                })
                .collect();

            if json {
                println!("{}", serde_json::to_string_pretty(&languages).unwrap());
            } else {
                println!("Supported Languages:");
                println!();
                println!("{:<6} {:<25} {:<10} {}", "Code", "Name", "Culture", "LCID");
                println!("{}", "-".repeat(60));
                for lang in languages {
                    println!(
                        "{:<6} {:<25} {:<10} {}",
                        lang.code, lang.name, lang.culture, lang.lcid
                    );
                }
            }
        }

        Commands::Check { files, base } => {
            let base_lang = match Language::from_str(&base) {
                Some(l) => l,
                None => {
                    eprintln!("Unknown base language: {}", base);
                    std::process::exit(1);
                }
            };

            let mut mgr = LocalizationManager::new();
            mgr.set_base_language(base_lang);

            for file in &files {
                let content = match fs::read_to_string(file) {
                    Ok(c) => c,
                    Err(e) => {
                        eprintln!("Failed to read {}: {}", file.display(), e);
                        continue;
                    }
                };

                match mgr.import_wxl(&content) {
                    Ok(lang) => {
                        println!("{}: {} ({})", file.display(), lang.name(), lang.code());
                    }
                    Err(e) => {
                        eprintln!("{}: Error - {}", file.display(), e);
                    }
                }
            }

            println!();
            println!("Coverage Report:");
            println!("{}", "-".repeat(40));

            for lang in mgr.languages() {
                let coverage = mgr.coverage(lang);
                let missing = mgr.find_missing(lang);
                let status = if missing.is_empty() { "✓" } else { "!" };

                println!(
                    "{} {:<20} {:>6.1}% ({} missing)",
                    status,
                    lang.name(),
                    coverage,
                    missing.len()
                );
            }
        }

        Commands::Missing { base, target } => {
            let mut mgr = LocalizationManager::new();

            // Import base file
            let base_content = match fs::read_to_string(&base) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("Failed to read {}: {}", base.display(), e);
                    std::process::exit(1);
                }
            };

            let base_lang = match mgr.import_wxl(&base_content) {
                Ok(l) => l,
                Err(e) => {
                    eprintln!("Failed to parse {}: {}", base.display(), e);
                    std::process::exit(1);
                }
            };

            // Import target file
            let target_content = match fs::read_to_string(&target) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("Failed to read {}: {}", target.display(), e);
                    std::process::exit(1);
                }
            };

            let target_lang = match mgr.import_wxl(&target_content) {
                Ok(l) => l,
                Err(e) => {
                    eprintln!("Failed to parse {}: {}", target.display(), e);
                    std::process::exit(1);
                }
            };

            let missing = mgr.find_missing(target_lang);

            if missing.is_empty() {
                println!(
                    "✓ {} has all strings from {}",
                    target_lang.name(),
                    base_lang.name()
                );
            } else {
                println!(
                    "Missing translations in {} (from {}):",
                    target_lang.name(),
                    base_lang.name()
                );
                println!();
                for id in &missing {
                    let base_value = mgr.get_string(id, base_lang).unwrap_or("");
                    println!("  {} = \"{}\"", id, truncate(base_value, 50));
                }
                println!();
                println!("Total: {} missing strings", missing.len());
                std::process::exit(1);
            }
        }

        Commands::Extract {
            files,
            output,
            language,
        } => {
            let lang = match Language::from_str(&language) {
                Some(l) => l,
                None => {
                    eprintln!("Unknown language: {}", language);
                    std::process::exit(1);
                }
            };

            let mut mgr = LocalizationManager::new();

            for file in &files {
                let content = match fs::read_to_string(file) {
                    Ok(c) => c,
                    Err(e) => {
                        eprintln!("Failed to read {}: {}", file.display(), e);
                        continue;
                    }
                };

                // Simple extraction of !(loc.XXX) references
                for cap in regex_lite::Regex::new(r"!\(loc\.([^)]+)\)")
                    .unwrap()
                    .captures_iter(&content)
                {
                    let id = &cap[1];
                    mgr.add_entry(
                        StringEntry::new(id, format!("[TODO: {}]", id)),
                        lang,
                    );
                }
            }

            let wxl = mgr.generate_wxl(lang);

            if let Some(path) = output {
                match fs::write(&path, &wxl) {
                    Ok(_) => {
                        println!(
                            "Extracted {} strings to {}",
                            mgr.string_ids().len(),
                            path.display()
                        );
                    }
                    Err(e) => {
                        eprintln!("Failed to write file: {}", e);
                        std::process::exit(1);
                    }
                }
            } else {
                println!("{}", wxl);
            }
        }

        Commands::Merge { files, output } => {
            let mut mgr = LocalizationManager::new();
            let mut target_lang = None;

            for file in &files {
                let content = match fs::read_to_string(file) {
                    Ok(c) => c,
                    Err(e) => {
                        eprintln!("Failed to read {}: {}", file.display(), e);
                        continue;
                    }
                };

                match mgr.import_wxl(&content) {
                    Ok(lang) => {
                        if target_lang.is_none() {
                            target_lang = Some(lang);
                        }
                        println!("Imported: {}", file.display());
                    }
                    Err(e) => {
                        eprintln!("Failed to parse {}: {}", file.display(), e);
                    }
                }
            }

            if let Some(lang) = target_lang {
                let wxl = mgr.generate_wxl(lang);
                match fs::write(&output, &wxl) {
                    Ok(_) => {
                        println!(
                            "Merged {} strings to {}",
                            mgr.string_ids().len(),
                            output.display()
                        );
                    }
                    Err(e) => {
                        eprintln!("Failed to write file: {}", e);
                        std::process::exit(1);
                    }
                }
            } else {
                eprintln!("No valid WXL files to merge");
                std::process::exit(1);
            }
        }

        Commands::Convert {
            input,
            output,
            format,
        } => {
            let content = match fs::read_to_string(&input) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("Failed to read {}: {}", input.display(), e);
                    std::process::exit(1);
                }
            };

            let mut mgr = LocalizationManager::new();

            // Try to parse as WXL first
            let lang = match mgr.import_wxl(&content) {
                Ok(l) => l,
                Err(_) => {
                    eprintln!("Failed to parse input file");
                    std::process::exit(1);
                }
            };

            let output_content = match format.as_str() {
                "json" => mgr.to_json(),
                "wxl" => mgr.generate_wxl(lang),
                _ => {
                    eprintln!("Unknown format: {}", format);
                    std::process::exit(1);
                }
            };

            match fs::write(&output, &output_content) {
                Ok(_) => println!("Converted to {}", output.display()),
                Err(e) => {
                    eprintln!("Failed to write file: {}", e);
                    std::process::exit(1);
                }
            }
        }
    }
}

#[derive(serde::Serialize)]
struct LanguageInfo {
    code: String,
    name: String,
    culture: String,
    lcid: u32,
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}...", &s[..max - 3])
    }
}

mod regex_lite {
    pub struct Regex(regex::Regex);

    impl Regex {
        pub fn new(pattern: &str) -> Result<Self, regex::Error> {
            regex::Regex::new(pattern).map(Regex)
        }

        pub fn captures_iter<'t>(&self, text: &'t str) -> regex::CaptureMatches<'_, 't> {
            self.0.captures_iter(text)
        }
    }
}
