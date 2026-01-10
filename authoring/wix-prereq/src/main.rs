//! wix-prereq CLI - Prerequisites detection and bundle helper

use clap::{Parser, Subcommand, ValueEnum};
use std::fs;
use std::path::PathBuf;
use wix_prereq::{Architecture, BundleGenerator, PrereqCatalog, PrereqDetector, PrereqKind, Prerequisite};

#[derive(Parser)]
#[command(name = "wix-prereq")]
#[command(about = "Prerequisites detection and bundle helper for WiX installers")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Detect prerequisites from project files
    Detect {
        /// Project files to scan (csproj, package.json, pom.xml, etc.)
        files: Vec<PathBuf>,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Generate WiX bundle fragment for prerequisites
    Generate {
        /// Prerequisite specifications (KIND:VERSION, e.g., dotnet:8.0, vcredist:2022)
        prereqs: Vec<String>,

        /// Output file (default: stdout)
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Architecture for VC++ redistributables
        #[arg(short, long, value_enum, default_value = "x64")]
        arch: Arch,
    },

    /// List available prerequisite types
    List {
        /// Show details for a specific type
        #[arg(short, long)]
        kind: Option<String>,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Generate a check script for prerequisites
    Check {
        /// Prerequisite specifications or project files
        input: Vec<String>,

        /// Output file (default: stdout)
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Script format
        #[arg(short, long, value_enum, default_value = "powershell")]
        format: ScriptFormat,
    },

    /// Scan project and generate bundle
    Scan {
        /// Directory to scan for project files
        #[arg(default_value = ".")]
        directory: PathBuf,

        /// Output WXS file
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Also generate check script
        #[arg(long)]
        check_script: Option<PathBuf>,

        /// Output as JSON (list detected prereqs)
        #[arg(long)]
        json: bool,
    },
}

#[derive(Clone, Copy, ValueEnum)]
enum Arch {
    X86,
    X64,
    Arm64,
}

impl From<Arch> for Architecture {
    fn from(a: Arch) -> Self {
        match a {
            Arch::X86 => Architecture::X86,
            Arch::X64 => Architecture::X64,
            Arch::Arm64 => Architecture::Arm64,
        }
    }
}

#[derive(Clone, Copy, ValueEnum)]
enum ScriptFormat {
    Powershell,
    Batch,
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Detect { files, json } => {
            if files.is_empty() {
                eprintln!("Error: No files specified");
                std::process::exit(1);
            }

            let detector = PrereqDetector::new();
            let mut all_prereqs = Vec::new();

            for file in &files {
                let content = match fs::read_to_string(file) {
                    Ok(c) => c,
                    Err(e) => {
                        eprintln!("Warning: Failed to read {}: {}", file.display(), e);
                        continue;
                    }
                };

                let filename = file.file_name().unwrap_or_default().to_string_lossy();
                let prereqs = detector.detect_from_content(&content, &filename);
                all_prereqs.extend(prereqs);
            }

            if json {
                println!("{}", serde_json::to_string_pretty(&all_prereqs).unwrap());
            } else {
                if all_prereqs.is_empty() {
                    println!("No prerequisites detected.");
                } else {
                    println!("Detected Prerequisites:");
                    println!("{}", "=".repeat(50));
                    println!();

                    for prereq in &all_prereqs {
                        println!("  {} {}", prereq.kind.as_str(), prereq.version);
                        if let Some(ref source) = prereq.source {
                            println!("    Source: {}", source);
                        }
                        if let Some(ref url) = prereq.download_url {
                            println!("    Download: {}", url);
                        }
                        println!();
                    }

                    println!("Total: {} prerequisite(s)", all_prereqs.len());
                }
            }
        }

        Commands::Generate { prereqs, output, arch } => {
            if prereqs.is_empty() {
                eprintln!("Error: No prerequisites specified");
                eprintln!("Usage: wix-prereq generate dotnet:8.0 vcredist:2022");
                std::process::exit(1);
            }

            let mut parsed_prereqs = Vec::new();

            for spec in &prereqs {
                match parse_prereq_spec(spec, arch.into()) {
                    Ok(prereq) => parsed_prereqs.push(prereq),
                    Err(e) => {
                        eprintln!("Error parsing '{}': {}", spec, e);
                        std::process::exit(1);
                    }
                }
            }

            let generator = BundleGenerator::new();
            let wxs = generator.generate(&parsed_prereqs);

            if let Some(path) = output {
                match fs::write(&path, &wxs) {
                    Ok(_) => {
                        println!("Generated bundle fragment: {}", path.display());
                        println!("  Prerequisites: {}", parsed_prereqs.len());
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

        Commands::List { kind, json } => {
            if let Some(ref kind_str) = kind {
                // Show details for specific kind
                let prereqs = match kind_str.to_lowercase().as_str() {
                    "dotnet" | "net" => PrereqCatalog::dotnet(),
                    "dotnetframework" | "netfx" | "framework" => PrereqCatalog::dotnet_framework(),
                    "vcredist" | "vc" | "cpp" => PrereqCatalog::vc_redist(),
                    _ => {
                        eprintln!("Unknown prerequisite kind: {}", kind_str);
                        eprintln!("Available: dotnet, dotnetframework, vcredist");
                        std::process::exit(1);
                    }
                };

                if json {
                    println!("{}", serde_json::to_string_pretty(&prereqs).unwrap());
                } else {
                    println!("Available versions for {}:", kind_str);
                    println!();
                    for prereq in prereqs {
                        println!("  {} {}", prereq.kind.as_str(), prereq.version);
                        if let Some(ref url) = prereq.download_url {
                            println!("    Download: {}", url);
                        }
                    }
                }
            } else {
                // List all kinds
                let kinds = vec![
                    ("dotnet", ".NET", "Modern .NET (6.0+)"),
                    ("dotnetframework", ".NET Framework", "Legacy .NET Framework (4.x)"),
                    ("vcredist", "VC++ Redistributable", "Visual C++ Runtime"),
                    ("java", "Java", "Java Runtime Environment"),
                    ("nodejs", "Node.js", "Node.js Runtime"),
                    ("python", "Python", "Python Interpreter"),
                ];

                if json {
                    let items: Vec<_> = kinds
                        .iter()
                        .map(|(id, name, desc)| {
                            serde_json::json!({
                                "id": id,
                                "name": name,
                                "description": desc
                            })
                        })
                        .collect();
                    println!("{}", serde_json::to_string_pretty(&items).unwrap());
                } else {
                    println!("Available Prerequisite Types:");
                    println!("{}", "=".repeat(50));
                    println!();
                    for (id, name, desc) in kinds {
                        println!("  {:<20} {}", id, name);
                        println!("  {:<20} {}", "", desc);
                        println!();
                    }
                    println!("Use 'wix-prereq list --kind <type>' to see available versions");
                }
            }
        }

        Commands::Check { input, output, format } => {
            let detector = PrereqDetector::new();
            let mut prereqs = Vec::new();

            for item in &input {
                let path = PathBuf::from(item);
                if path.exists() {
                    // It's a file, detect from it
                    let content = match fs::read_to_string(&path) {
                        Ok(c) => c,
                        Err(e) => {
                            eprintln!("Warning: Failed to read {}: {}", path.display(), e);
                            continue;
                        }
                    };
                    let filename = path.file_name().unwrap_or_default().to_string_lossy();
                    prereqs.extend(detector.detect_from_content(&content, &filename));
                } else if item.contains(':') {
                    // It's a spec
                    match parse_prereq_spec(item, Architecture::X64) {
                        Ok(prereq) => prereqs.push(prereq),
                        Err(e) => {
                            eprintln!("Warning: Invalid spec '{}': {}", item, e);
                        }
                    }
                }
            }

            if prereqs.is_empty() {
                eprintln!("No prerequisites to check");
                std::process::exit(1);
            }

            let generator = BundleGenerator::new();
            let script = match format {
                ScriptFormat::Powershell => generator.generate_check_script(&prereqs),
                ScriptFormat::Batch => generate_batch_script(&prereqs),
            };

            if let Some(path) = output {
                match fs::write(&path, &script) {
                    Ok(_) => println!("Generated check script: {}", path.display()),
                    Err(e) => {
                        eprintln!("Failed to write output: {}", e);
                        std::process::exit(1);
                    }
                }
            } else {
                println!("{}", script);
            }
        }

        Commands::Scan {
            directory,
            output,
            check_script,
            json,
        } => {
            let detector = PrereqDetector::new();
            let mut all_prereqs = Vec::new();
            let mut scanned_files = Vec::new();

            // Find project files
            let patterns = [
                "*.csproj",
                "*.vbproj",
                "*.fsproj",
                "*.vcxproj",
                "package.json",
                "pom.xml",
                "build.gradle",
                "pyproject.toml",
                "requirements.txt",
            ];

            for entry in walkdir(&directory, 3) {
                let filename = entry.file_name().unwrap_or_default().to_string_lossy();
                let matches = patterns.iter().any(|p| {
                    if p.starts_with('*') {
                        filename.ends_with(&p[1..])
                    } else {
                        filename == *p
                    }
                });

                if matches {
                    if let Ok(content) = fs::read_to_string(&entry) {
                        let prereqs = detector.detect_from_content(&content, &filename);
                        if !prereqs.is_empty() {
                            scanned_files.push(entry.display().to_string());
                            all_prereqs.extend(prereqs);
                        }
                    }
                }
            }

            // Deduplicate
            let mut seen = std::collections::HashSet::new();
            all_prereqs.retain(|p| seen.insert((format!("{:?}", p.kind), p.version.clone())));

            if json {
                let result = serde_json::json!({
                    "scanned_files": scanned_files,
                    "prerequisites": all_prereqs
                });
                println!("{}", serde_json::to_string_pretty(&result).unwrap());
                return;
            }

            println!("Scanned {} file(s):", scanned_files.len());
            for file in &scanned_files {
                println!("  {}", file);
            }
            println!();

            if all_prereqs.is_empty() {
                println!("No prerequisites detected.");
                return;
            }

            println!("Detected Prerequisites:");
            for prereq in &all_prereqs {
                println!("  {} {}", prereq.kind.as_str(), prereq.version);
            }
            println!();

            // Generate bundle fragment
            if let Some(path) = output {
                let generator = BundleGenerator::new();
                let wxs = generator.generate(&all_prereqs);
                match fs::write(&path, &wxs) {
                    Ok(_) => println!("Generated: {}", path.display()),
                    Err(e) => eprintln!("Failed to write {}: {}", path.display(), e),
                }
            }

            // Generate check script
            if let Some(path) = check_script {
                let generator = BundleGenerator::new();
                let script = generator.generate_check_script(&all_prereqs);
                match fs::write(&path, &script) {
                    Ok(_) => println!("Generated: {}", path.display()),
                    Err(e) => eprintln!("Failed to write {}: {}", path.display(), e),
                }
            }
        }
    }
}

fn parse_prereq_spec(spec: &str, arch: Architecture) -> Result<Prerequisite, String> {
    let parts: Vec<&str> = spec.splitn(2, ':').collect();
    if parts.len() != 2 {
        return Err("Expected format KIND:VERSION (e.g., dotnet:8.0)".to_string());
    }

    let kind = match parts[0].to_lowercase().as_str() {
        "dotnet" | "net" => PrereqKind::DotNet,
        "dotnetframework" | "netfx" | "framework" => PrereqKind::DotNetFramework,
        "dotnetcore" | "netcore" => PrereqKind::DotNetCore,
        "vcredist" | "vc" | "cpp" => PrereqKind::VcRedist,
        "java" | "jre" | "jdk" => PrereqKind::Java,
        "nodejs" | "node" => PrereqKind::NodeJs,
        "python" | "py" => PrereqKind::Python,
        _ => return Err(format!("Unknown prerequisite kind: {}", parts[0])),
    };

    let version = parts[1].to_string();

    let mut prereq = Prerequisite::new(kind, version);
    if kind == PrereqKind::VcRedist {
        prereq = prereq.with_architecture(arch);
    }

    Ok(prereq)
}

fn generate_batch_script(prereqs: &[Prerequisite]) -> String {
    let mut script = String::new();
    script.push_str("@echo off\n");
    script.push_str("REM Prerequisites Check Script\n");
    script.push_str("REM Generated by wix-prereq\n\n");

    script.push_str("setlocal\n");
    script.push_str("set MISSING=0\n\n");

    for prereq in prereqs {
        script.push_str(&format!(
            "REM Check {} {}\n",
            prereq.kind.as_str(),
            prereq.version
        ));

        if let Some(reg) = prereq.get_registry_detection() {
            script.push_str(&format!(
                "reg query \"{}\\{}\" >nul 2>&1\n",
                reg.root, reg.key
            ));
            script.push_str("if errorlevel 1 (\n");
            script.push_str(&format!(
                "    echo Missing: {} {}\n",
                prereq.kind.as_str(),
                prereq.version
            ));
            script.push_str("    set /a MISSING+=1\n");
            script.push_str(")\n\n");
        }
    }

    script.push_str("if %MISSING% GTR 0 (\n");
    script.push_str("    echo.\n");
    script.push_str("    echo %MISSING% prerequisite(s) missing!\n");
    script.push_str("    exit /b 1\n");
    script.push_str(") else (\n");
    script.push_str("    echo All prerequisites are installed.\n");
    script.push_str("    exit /b 0\n");
    script.push_str(")\n");

    script
}

fn walkdir(dir: &std::path::Path, max_depth: usize) -> Vec<PathBuf> {
    let mut results = Vec::new();
    walkdir_recursive(dir, 0, max_depth, &mut results);
    results
}

fn walkdir_recursive(dir: &std::path::Path, depth: usize, max_depth: usize, results: &mut Vec<PathBuf>) {
    if depth > max_depth {
        return;
    }

    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            // Skip common non-project directories
            let name = path.file_name().unwrap_or_default().to_string_lossy();
            if !["node_modules", "target", "bin", "obj", ".git", "vendor"].contains(&name.as_ref())
            {
                walkdir_recursive(&path, depth + 1, max_depth, results);
            }
        } else {
            results.push(path);
        }
    }
}
