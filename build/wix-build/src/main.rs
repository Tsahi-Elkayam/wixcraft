//! wix-build CLI - Unified build CLI for WiX installers

use clap::{Parser, Subcommand, ValueEnum};
use std::fs;
use std::path::PathBuf;
use wix_build::preview::{FileAttributes, FileEntry, InstallPreview, PreviewGenerator, RegistryEntry};
use wix_build::{Architecture, BuildConfig, OutputType, WixToolset};

#[derive(Parser)]
#[command(name = "wix-build")]
#[command(about = "Unified build CLI for WiX installers with preview capability")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Build WXS source files to MSI/MSM/Bundle
    Build {
        /// WXS source file(s)
        sources: Vec<PathBuf>,

        /// Output file (.msi, .msm, .exe)
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Target architecture
        #[arg(short, long, value_enum)]
        arch: Option<Arch>,

        /// Preprocessor defines (NAME=VALUE or NAME)
        #[arg(short = 'D', long = "define")]
        defines: Vec<String>,

        /// WiX extensions to use
        #[arg(short, long)]
        ext: Vec<String>,

        /// Include search paths
        #[arg(short = 'I', long = "include")]
        include: Vec<PathBuf>,

        /// Bind paths for file resolution
        #[arg(short, long)]
        bind: Vec<PathBuf>,

        /// Localization files (.wxl)
        #[arg(short = 'L', long = "loc")]
        loc: Vec<PathBuf>,

        /// Cultures (e.g., en-US, fr-FR)
        #[arg(long)]
        culture: Vec<String>,

        /// Output type (v4+ only)
        #[arg(short = 't', long, value_enum)]
        r#type: Option<OutType>,

        /// Intermediate output directory
        #[arg(long)]
        intermediate: Option<PathBuf>,

        /// Suppress warning codes
        #[arg(long = "sw")]
        suppress_warnings: Vec<String>,

        /// Treat warnings as errors
        #[arg(long)]
        wx: bool,

        /// Skip ICE validation
        #[arg(long)]
        skip_validation: bool,

        /// Verbose output
        #[arg(short, long)]
        verbose: bool,

        /// Pedantic mode (v4+ only)
        #[arg(long)]
        pedantic: bool,

        /// Just print the command, don't run it
        #[arg(long)]
        dry_run: bool,

        /// Use specific WiX version
        #[arg(long, value_enum)]
        wix_version: Option<WixVer>,
    },

    /// Detect installed WiX toolset
    Detect,

    /// Show build command without executing
    Show {
        /// WXS source file(s)
        sources: Vec<PathBuf>,

        /// Output file
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Target architecture
        #[arg(short, long, value_enum)]
        arch: Option<Arch>,

        /// Preprocessor defines
        #[arg(short = 'D', long = "define")]
        defines: Vec<String>,

        /// WiX extensions
        #[arg(short, long)]
        ext: Vec<String>,

        /// Use specific WiX version
        #[arg(long, value_enum)]
        wix_version: Option<WixVer>,
    },

    /// Build using a preset profile
    Profile {
        /// Profile name (debug, release, ci)
        #[arg(value_enum)]
        profile: ProfileName,

        /// WXS source file(s)
        sources: Vec<PathBuf>,

        /// Output file
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Additional defines
        #[arg(short = 'D', long = "define")]
        defines: Vec<String>,

        /// Just print the command
        #[arg(long)]
        dry_run: bool,
    },

    /// Preview installer content without building
    Preview {
        #[command(subcommand)]
        action: PreviewCommands,
    },
}

#[derive(Subcommand)]
enum PreviewCommands {
    /// Preview files that would be installed
    Files {
        /// WiX source file or MSI
        input: PathBuf,

        /// Output format
        #[arg(short, long, value_enum, default_value = "tree")]
        format: FormatArg,
    },
    /// Preview registry entries
    Registry {
        /// WiX source file or MSI
        input: PathBuf,

        /// Output format
        #[arg(short, long, value_enum, default_value = "tree")]
        format: FormatArg,
    },
    /// Preview all content
    All {
        /// WiX source file or MSI
        input: PathBuf,

        /// Output format
        #[arg(short, long, value_enum, default_value = "text")]
        format: FormatArg,
    },
    /// Show summary statistics
    Summary {
        /// WiX source file or MSI
        input: PathBuf,
    },
    /// Export preview to file
    Export {
        /// WiX source file or MSI
        input: PathBuf,

        /// Output file
        #[arg(short, long)]
        output: PathBuf,

        /// Output format
        #[arg(short, long, value_enum, default_value = "json")]
        format: FormatArg,
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
enum OutType {
    Msi,
    Msm,
    Bundle,
    Wixlib,
}

impl From<OutType> for OutputType {
    fn from(t: OutType) -> Self {
        match t {
            OutType::Msi => OutputType::Msi,
            OutType::Msm => OutputType::Msm,
            OutType::Bundle => OutputType::Bundle,
            OutType::Wixlib => OutputType::Wixlib,
        }
    }
}

#[derive(Clone, Copy, ValueEnum)]
enum WixVer {
    V3,
    V4,
    V5,
}

impl From<WixVer> for wix_build::WixVersion {
    fn from(v: WixVer) -> Self {
        match v {
            WixVer::V3 => wix_build::WixVersion::V3,
            WixVer::V4 => wix_build::WixVersion::V4,
            WixVer::V5 => wix_build::WixVersion::V5,
        }
    }
}

#[derive(Clone, Copy, ValueEnum)]
enum ProfileName {
    Debug,
    Release,
    Ci,
}

#[derive(Clone, ValueEnum)]
enum FormatArg {
    Text,
    Json,
    Tree,
    Table,
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Build {
            sources,
            output,
            arch,
            defines,
            ext,
            include,
            bind,
            loc,
            culture,
            r#type,
            intermediate,
            suppress_warnings,
            wx,
            skip_validation,
            verbose,
            pedantic,
            dry_run,
            wix_version,
        } => {
            run_build(
                sources,
                output,
                arch,
                defines,
                ext,
                include,
                bind,
                loc,
                culture,
                r#type,
                intermediate,
                suppress_warnings,
                wx,
                skip_validation,
                verbose,
                pedantic,
                dry_run,
                wix_version,
            );
        }

        Commands::Detect => {
            run_detect();
        }

        Commands::Show {
            sources,
            output,
            arch,
            defines,
            ext,
            wix_version,
        } => {
            run_show(sources, output, arch, defines, ext, wix_version);
        }

        Commands::Profile {
            profile,
            sources,
            output,
            defines,
            dry_run,
        } => {
            run_profile(profile, sources, output, defines, dry_run);
        }

        Commands::Preview { action } => {
            if let Err(e) = run_preview(action) {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn run_build(
    sources: Vec<PathBuf>,
    output: Option<PathBuf>,
    arch: Option<Arch>,
    defines: Vec<String>,
    ext: Vec<String>,
    include: Vec<PathBuf>,
    bind: Vec<PathBuf>,
    loc: Vec<PathBuf>,
    culture: Vec<String>,
    out_type: Option<OutType>,
    intermediate: Option<PathBuf>,
    suppress_warnings: Vec<String>,
    wx: bool,
    skip_validation: bool,
    verbose: bool,
    pedantic: bool,
    dry_run: bool,
    wix_version: Option<WixVer>,
) {
    if sources.is_empty() {
        eprintln!("Error: No source files specified");
        std::process::exit(1);
    }

    let toolset = if let Some(ver) = wix_version {
        WixToolset::with_version(ver.into())
    } else {
        match WixToolset::detect() {
            Ok(t) => t,
            Err(e) => {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
    };

    let mut config = BuildConfig::from_sources(&sources);

    if let Some(out) = output {
        config = config.output(out);
    }

    if let Some(a) = arch {
        config = config.architecture(a.into());
    }

    for def in defines {
        if let Some((key, value)) = def.split_once('=') {
            config = config.define(key, value);
        } else {
            config = config.define_flag(def);
        }
    }

    for e in ext {
        config = config.extension(e);
    }

    for p in include {
        config = config.include_path(p);
    }

    for p in bind {
        config = config.bind_path(p);
    }

    for l in loc {
        config = config.localization_file(l);
    }

    if !culture.is_empty() {
        config = config.cultures(culture);
    }

    if let Some(t) = out_type {
        config = config.output_type(t.into());
    }

    if let Some(dir) = intermediate {
        config = config.intermediate_dir(dir);
    }

    for sw in suppress_warnings {
        config = config.suppress_warning(sw);
    }

    config = config
        .warnings_as_errors(wx)
        .skip_validation(skip_validation)
        .verbose(verbose)
        .pedantic(pedantic);

    if let Err(e) = config.validate() {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }

    let cmd = toolset.build_command(&config);

    println!("WiX Toolset: {} ({})", toolset.version.as_str(), toolset.path.display());
    println!();

    if dry_run {
        println!("Command:");
        println!("  {}", cmd);
    } else {
        println!("Building...");
        println!("  {}", cmd);
        println!();

        // For v3, we need to run candle then light separately
        if toolset.version == wix_build::WixVersion::V3 {
            let (candle_cmd, light_cmd) = toolset.build_v3_commands(&config);

            println!("Running: {}", candle_cmd);
            let status = std::process::Command::new("cmd")
                .args(["/C", &candle_cmd])
                .status();

            match status {
                Ok(s) if s.success() => {
                    println!("Candle succeeded");
                }
                Ok(s) => {
                    eprintln!("Candle failed with exit code: {:?}", s.code());
                    std::process::exit(1);
                }
                Err(e) => {
                    eprintln!("Failed to run candle: {}", e);
                    std::process::exit(1);
                }
            }

            println!();
            println!("Running: {}", light_cmd);
            let status = std::process::Command::new("cmd")
                .args(["/C", &light_cmd])
                .status();

            match status {
                Ok(s) if s.success() => {
                    println!();
                    println!("Build completed successfully");
                }
                Ok(s) => {
                    eprintln!("Light failed with exit code: {:?}", s.code());
                    std::process::exit(1);
                }
                Err(e) => {
                    eprintln!("Failed to run light: {}", e);
                    std::process::exit(1);
                }
            }
        } else {
            // v4/v5: single wix build command
            let status = std::process::Command::new("cmd")
                .args(["/C", &cmd])
                .status();

            match status {
                Ok(s) if s.success() => {
                    println!();
                    println!("Build completed successfully");
                }
                Ok(s) => {
                    eprintln!("Build failed with exit code: {:?}", s.code());
                    std::process::exit(1);
                }
                Err(e) => {
                    eprintln!("Failed to run build: {}", e);
                    std::process::exit(1);
                }
            }
        }
    }
}

fn run_detect() {
    match WixToolset::detect() {
        Ok(toolset) => {
            println!("WiX Toolset Detected");
            println!("{}", "=".repeat(40));
            println!();
            println!("Version:  {}", toolset.version.as_str());
            println!("Path:     {}", toolset.path.display());

            if let Some(ref wix) = toolset.wix_path {
                println!("wix:      {}", wix.display());
            }
            if let Some(ref candle) = toolset.candle_path {
                println!("candle:   {}", candle.display());
            }
            if let Some(ref light) = toolset.light_path {
                println!("light:    {}", light.display());
            }
        }
        Err(e) => {
            eprintln!("WiX toolset not found: {}", e);
            eprintln!();
            eprintln!("Install from: https://wixtoolset.org/");
            std::process::exit(1);
        }
    }
}

fn run_show(
    sources: Vec<PathBuf>,
    output: Option<PathBuf>,
    arch: Option<Arch>,
    defines: Vec<String>,
    ext: Vec<String>,
    wix_version: Option<WixVer>,
) {
    if sources.is_empty() {
        eprintln!("Error: No source files specified");
        std::process::exit(1);
    }

    let toolset = if let Some(ver) = wix_version {
        WixToolset::with_version(ver.into())
    } else {
        match WixToolset::detect() {
            Ok(t) => t,
            Err(_) => WixToolset::with_version(wix_build::WixVersion::V4),
        }
    };

    let mut config = BuildConfig::from_sources(&sources);

    if let Some(out) = output {
        config = config.output(out);
    }

    if let Some(a) = arch {
        config = config.architecture(a.into());
    }

    for def in defines {
        if let Some((key, value)) = def.split_once('=') {
            config = config.define(key, value);
        } else {
            config = config.define_flag(def);
        }
    }

    for e in ext {
        config = config.extension(e);
    }

    println!("WiX {} command:", toolset.version.as_str());
    println!();

    if toolset.version == wix_build::WixVersion::V3 {
        let (candle, light) = toolset.build_v3_commands(&config);
        println!("1. Compile:");
        println!("   {}", candle);
        println!();
        println!("2. Link:");
        println!("   {}", light);
    } else {
        println!("   {}", toolset.build_command(&config));
    }
}

fn run_profile(
    profile: ProfileName,
    sources: Vec<PathBuf>,
    output: Option<PathBuf>,
    defines: Vec<String>,
    dry_run: bool,
) {
    if sources.is_empty() {
        eprintln!("Error: No source files specified");
        std::process::exit(1);
    }

    let toolset = match WixToolset::detect() {
        Ok(t) => t,
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    };

    let mut config = match profile {
        ProfileName::Debug => {
            println!("Using DEBUG profile");
            BuildConfig::from_sources(&sources)
                .define("Configuration", "Debug")
                .verbose(true)
        }
        ProfileName::Release => {
            println!("Using RELEASE profile");
            BuildConfig::from_sources(&sources)
                .define("Configuration", "Release")
                .warnings_as_errors(true)
        }
        ProfileName::Ci => {
            println!("Using CI profile");
            BuildConfig::from_sources(&sources)
                .define("Configuration", "Release")
                .warnings_as_errors(true)
                .pedantic(true)
        }
    };

    if let Some(out) = output {
        config = config.output(out);
    }

    for def in defines {
        if let Some((key, value)) = def.split_once('=') {
            config = config.define(key, value);
        } else {
            config = config.define_flag(def);
        }
    }

    let cmd = toolset.build_command(&config);
    println!();

    if dry_run {
        println!("Command:");
        println!("  {}", cmd);
    } else {
        println!("Building...");
        println!("  {}", cmd);

        let status = std::process::Command::new("cmd")
            .args(["/C", &cmd])
            .status();

        match status {
            Ok(s) if s.success() => {
                println!();
                println!("Build completed successfully");
            }
            Ok(s) => {
                eprintln!("Build failed with exit code: {:?}", s.code());
                std::process::exit(1);
            }
            Err(e) => {
                eprintln!("Failed to run build: {}", e);
                std::process::exit(1);
            }
        }
    }
}

fn run_preview(action: PreviewCommands) -> Result<(), Box<dyn std::error::Error>> {
    match action {
        PreviewCommands::Files { input, format } => {
            let preview = parse_input(&input)?;

            match format {
                FormatArg::Tree => {
                    println!("File Tree Preview for: {}\n", input.display());
                    println!("{}", PreviewGenerator::file_tree(&preview.files));
                }
                FormatArg::Json => {
                    println!("{}", serde_json::to_string_pretty(&preview.files)?);
                }
                FormatArg::Table => {
                    println!("{:<40} {:<40}", "Source", "Destination");
                    println!("{:-<80}", "");
                    for file in &preview.files {
                        println!("{:<40} {:<40}", file.source, file.destination);
                    }
                }
                FormatArg::Text => {
                    println!("Files ({}):", preview.files.len());
                    for file in &preview.files {
                        println!("  {} -> {}", file.source, file.destination);
                    }
                }
            }
        }

        PreviewCommands::Registry { input, format } => {
            let preview = parse_input(&input)?;

            match format {
                FormatArg::Tree => {
                    println!("Registry Preview for: {}\n", input.display());
                    println!("{}", PreviewGenerator::registry_tree(&preview.registry));
                }
                FormatArg::Json => {
                    println!("{}", serde_json::to_string_pretty(&preview.registry)?);
                }
                _ => {
                    println!("Registry Entries ({}):", preview.registry.len());
                    for entry in &preview.registry {
                        let name = entry.name.as_deref().unwrap_or("(Default)");
                        let value = entry.value.as_deref().unwrap_or("");
                        println!("  {}\\{}", entry.root, entry.key);
                        println!("    {} = {}", name, value);
                    }
                }
            }
        }

        PreviewCommands::All { input, format } => {
            let preview = parse_input(&input)?;

            match format {
                FormatArg::Json => {
                    println!("{}", serde_json::to_string_pretty(&preview)?);
                }
                _ => {
                    println!("{}", preview.summary().to_string_report());
                    println!("\n--- Files ---");
                    println!("{}", PreviewGenerator::file_tree(&preview.files));
                    if !preview.registry.is_empty() {
                        println!("\n--- Registry ---");
                        println!("{}", PreviewGenerator::registry_tree(&preview.registry));
                    }
                }
            }
        }

        PreviewCommands::Summary { input } => {
            let preview = parse_input(&input)?;
            println!("{}", preview.summary().to_string_report());
        }

        PreviewCommands::Export {
            input,
            output,
            format,
        } => {
            let preview = parse_input(&input)?;

            let content = match format {
                FormatArg::Json => serde_json::to_string_pretty(&preview)?,
                FormatArg::Text => preview.summary().to_string_report(),
                FormatArg::Tree => {
                    let mut s = String::new();
                    s.push_str("Files:\n");
                    s.push_str(&PreviewGenerator::file_tree(&preview.files));
                    s.push_str("\nRegistry:\n");
                    s.push_str(&PreviewGenerator::registry_tree(&preview.registry));
                    s
                }
                FormatArg::Table => {
                    let mut s = String::new();
                    s.push_str(&format!("{:<40} {:<40}\n", "Source", "Destination"));
                    s.push_str(&format!("{:-<80}\n", ""));
                    for file in &preview.files {
                        s.push_str(&format!("{:<40} {:<40}\n", file.source, file.destination));
                    }
                    s
                }
            };

            fs::write(&output, &content)?;
            println!("Exported preview to: {}", output.display());
        }
    }

    Ok(())
}

fn parse_input(path: &PathBuf) -> Result<InstallPreview, Box<dyn std::error::Error>> {
    // In a real implementation, this would parse WiX XML or read MSI tables
    // For now, create a sample preview
    let mut preview = InstallPreview::new();

    // Read file and try to extract some information
    if path.exists() {
        let content = fs::read_to_string(path).unwrap_or_default();

        // Set product info from file name
        preview.product_name = Some(
            path.file_stem()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_else(|| "Unknown".to_string()),
        );
        preview.version = Some("1.0.0".to_string());

        // Simple extraction of File elements
        if content.contains("<File") {
            // Add sample file entries based on content analysis
            preview.files.push(FileEntry {
                source: "app.exe".to_string(),
                destination: "[INSTALLFOLDER]\\app.exe".to_string(),
                component: "MainComponent".to_string(),
                feature: Some("ProductFeature".to_string()),
                attributes: FileAttributes::default(),
            });
        }

        // Check for registry entries
        if content.contains("<Registry") || content.contains("<RegistryKey") {
            preview.registry.push(RegistryEntry {
                root: "HKLM".to_string(),
                key: "Software\\Company\\Product".to_string(),
                name: Some("Version".to_string()),
                value: Some("1.0.0".to_string()),
                value_type: "string".to_string(),
                component: "RegistryComponent".to_string(),
            });
        }
    }

    Ok(preview)
}
