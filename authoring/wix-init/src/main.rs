//! wix-init CLI - Unified WiX project lifecycle tool
//!
//! Consolidates project scaffolding, GUID generation, installation management,
//! environment setup, and licensing into a single tool.

use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;
use wix_init::{
    Project, Template, WixVersion,
    Guid, GuidFormat, GuidGenerator, GuidBatch,
    InstallOptions, UILevel, MsiExecCommand, InstallPresets,
};

#[derive(Parser)]
#[command(name = "wix-init")]
#[command(about = "Unified WiX project lifecycle tool")]
#[command(long_about = "Create projects, generate GUIDs, manage installations, and configure environments for WiX development.")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a new WiX project
    New {
        /// Project name
        name: String,

        /// Template to use
        #[arg(short, long, default_value = "simple-msi")]
        template: String,

        /// Manufacturer name
        #[arg(short, long, default_value = "My Company")]
        manufacturer: String,

        /// Version string
        #[arg(short, long, default_value = "1.0.0")]
        version: String,

        /// Description
        #[arg(short, long)]
        description: Option<String>,

        /// WiX version (v4, v5)
        #[arg(long, default_value = "v5")]
        wix: String,

        /// Output directory
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Interactive wizard mode
        #[arg(long)]
        wizard: bool,
    },

    /// List available templates
    List {
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Generate GUIDs
    Guid {
        #[command(subcommand)]
        action: GuidAction,
    },

    /// Install an MSI package
    Install {
        /// MSI file path
        msi: PathBuf,

        /// UI level
        #[arg(short, long, value_enum, default_value = "full")]
        ui: UILevelArg,

        /// Target installation directory
        #[arg(short, long)]
        target: Option<String>,

        /// Log file path
        #[arg(short, long)]
        log: Option<PathBuf>,

        /// Set property (KEY=VALUE)
        #[arg(short = 'P', long)]
        property: Vec<String>,

        /// Silent mode (no UI)
        #[arg(long)]
        silent: bool,

        /// Suppress restart
        #[arg(long)]
        no_restart: bool,

        /// Dry run (show command only)
        #[arg(long)]
        dry_run: bool,
    },

    /// Uninstall an MSI package
    Uninstall {
        /// Product code or MSI path
        target: String,

        /// Silent mode
        #[arg(long)]
        silent: bool,

        /// Log file path
        #[arg(short, long)]
        log: Option<PathBuf>,

        /// Dry run
        #[arg(long)]
        dry_run: bool,
    },

    /// Update an installed MSI
    Update {
        /// MSI file path
        msi: PathBuf,

        /// Silent mode
        #[arg(long)]
        silent: bool,

        /// Log file path
        #[arg(short, long)]
        log: Option<PathBuf>,
    },

    /// Repair an MSI installation
    Repair {
        /// Product code or MSI path
        target: String,

        /// Silent mode
        #[arg(long)]
        silent: bool,

        /// Log file path
        #[arg(short, long)]
        log: Option<PathBuf>,
    },

    /// Check WiX development environment
    Doctor,

    /// Setup WiX development environment
    Setup {
        /// Skip IDE configuration
        #[arg(long)]
        no_ide: bool,

        /// Enable Windows Sandbox for testing
        #[arg(long)]
        sandbox: bool,
    },

    /// Generate license file
    License {
        /// License type
        #[arg(value_enum)]
        license_type: LicenseTypeArg,

        /// Output file
        #[arg(short, long, default_value = "LICENSE.rtf")]
        output: PathBuf,

        /// Company/Author name
        #[arg(short, long)]
        author: Option<String>,

        /// Year
        #[arg(short, long)]
        year: Option<u32>,
    },

    /// Generate Windows Sandbox configuration
    Sandbox {
        /// MSI file to test
        msi: Option<PathBuf>,

        /// Output .wsb file
        #[arg(short, long, default_value = "test.wsb")]
        output: PathBuf,
    },
}

#[derive(Subcommand)]
enum GuidAction {
    /// Generate random GUIDs
    Random {
        /// Number of GUIDs
        #[arg(default_value = "1")]
        count: usize,

        /// Output format
        #[arg(short, long, value_enum, default_value = "braces")]
        format: FormatArg,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Generate deterministic GUID from input
    Hash {
        /// Input string
        input: String,

        /// Product name for namespace
        #[arg(short, long, default_value = "WixCraft")]
        product: String,

        /// Version for namespace
        #[arg(short = 'V', long, default_value = "1.0.0")]
        version: String,

        /// Output format
        #[arg(short, long, value_enum, default_value = "braces")]
        format: FormatArg,
    },

    /// Generate component GUIDs from paths
    Component {
        /// File paths
        paths: Vec<String>,

        /// Product name
        #[arg(short, long, default_value = "WixCraft")]
        product: String,

        /// Version
        #[arg(short = 'V', long, default_value = "1.0.0")]
        version: String,

        /// Output format
        #[arg(short, long, value_enum, default_value = "braces")]
        format: FormatArg,
    },

    /// Generate product and upgrade codes
    Product {
        /// Product name
        name: String,

        /// Version
        #[arg(short, long, default_value = "1.0.0")]
        version: String,

        /// Output format
        #[arg(short, long, value_enum, default_value = "braces")]
        format: FormatArg,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Parse and validate a GUID
    Parse {
        /// GUID to parse
        guid: String,

        /// Output format
        #[arg(short, long, value_enum, default_value = "braces")]
        format: FormatArg,
    },
}

#[derive(Clone, Copy, ValueEnum)]
enum FormatArg {
    Braces,
    Hyphens,
    Plain,
    Registry,
}

impl From<FormatArg> for GuidFormat {
    fn from(f: FormatArg) -> Self {
        match f {
            FormatArg::Braces => GuidFormat::Braces,
            FormatArg::Hyphens => GuidFormat::Hyphens,
            FormatArg::Plain => GuidFormat::Plain,
            FormatArg::Registry => GuidFormat::Registry,
        }
    }
}

#[derive(Clone, Copy, ValueEnum)]
enum UILevelArg {
    None,
    Basic,
    Reduced,
    Full,
}

impl From<UILevelArg> for UILevel {
    fn from(arg: UILevelArg) -> Self {
        match arg {
            UILevelArg::None => UILevel::None,
            UILevelArg::Basic => UILevel::Basic,
            UILevelArg::Reduced => UILevel::Reduced,
            UILevelArg::Full => UILevel::Full,
        }
    }
}

#[derive(Clone, Copy, ValueEnum)]
enum LicenseTypeArg {
    Mit,
    Apache2,
    Gpl3,
    Bsd3,
    Proprietary,
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::New {
            name,
            template,
            manufacturer,
            version,
            description,
            wix,
            output,
            wizard: _wizard,
        } => {
            let template = match template.parse::<Template>() {
                Ok(t) => t,
                Err(e) => {
                    eprintln!("Error: {}", e);
                    eprintln!("Run 'wix-init list' to see available templates");
                    std::process::exit(1);
                }
            };

            let wix_version = match wix.to_lowercase().as_str() {
                "v4" | "4" => WixVersion::V4,
                "v5" | "5" => WixVersion::V5,
                _ => {
                    eprintln!("Error: Invalid WiX version '{}'. Use v4 or v5", wix);
                    std::process::exit(1);
                }
            };

            let mut project = Project::new(&name, template)
                .with_manufacturer(&manufacturer)
                .with_version(&version)
                .with_wix_version(wix_version);

            if let Some(desc) = description {
                project = project.with_description(desc);
            }

            let base_path = output.unwrap_or_else(|| PathBuf::from("."));

            match project.create(&base_path) {
                Ok(result) => {
                    println!("Created project '{}' at {}", name, result.path.display());
                    println!("\nFiles created:");
                    for file in &result.files {
                        println!("  {}", file);
                    }
                    println!("\nNext steps:");
                    println!("  cd {}", result.path.display());
                    println!("  dotnet build");
                }
                Err(e) => {
                    eprintln!("Error: {}", e);
                    std::process::exit(1);
                }
            }
        }

        Commands::List { json } => {
            if json {
                let templates: Vec<_> = Template::all()
                    .iter()
                    .map(|t| {
                        serde_json::json!({
                            "name": t.to_string(),
                            "description": t.description()
                        })
                    })
                    .collect();
                println!("{}", serde_json::to_string_pretty(&templates).unwrap());
            } else {
                println!("Available templates:\n");
                for template in Template::all() {
                    println!("  {:15} - {}", template, template.description());
                }
                println!("\nUsage: wix-init new <name> --template <template>");
            }
        }

        Commands::Guid { action } => {
            match action {
                GuidAction::Random { count, format, json } => {
                    let batch = GuidBatch::random(count);
                    let formatted = batch.format_all(format.into());

                    if json {
                        println!("{}", serde_json::to_string_pretty(&formatted).unwrap());
                    } else {
                        for guid in formatted {
                            println!("{}", guid);
                        }
                    }
                }

                GuidAction::Hash { input, product, version, format } => {
                    let generator = GuidGenerator::new(&product, &version);
                    let guid = generator.custom_guid(&input);
                    println!("{}", guid.format(format.into()));
                }

                GuidAction::Component { paths, product, version, format } => {
                    let generator = GuidGenerator::new(&product, &version);
                    println!("Component GUIDs for {} v{}:\n", product, version);
                    for path in paths {
                        let guid = generator.component_guid(&path);
                        println!("  {} -> {}", path, guid.format(format.into()));
                    }
                }

                GuidAction::Product { name, version, format, json } => {
                    let generator = GuidGenerator::new(&name, &version);
                    let product_code = generator.product_code();
                    let upgrade_code = generator.upgrade_code();
                    let fmt: GuidFormat = format.into();

                    if json {
                        let result = serde_json::json!({
                            "product": name,
                            "version": version,
                            "productCode": product_code.format(fmt),
                            "upgradeCode": upgrade_code.format(fmt)
                        });
                        println!("{}", serde_json::to_string_pretty(&result).unwrap());
                    } else {
                        println!("Product GUIDs for {} v{}:\n", name, version);
                        println!("  ProductCode: {}", product_code.format(fmt));
                        println!("  UpgradeCode: {}", upgrade_code.format(fmt));
                        println!();
                        println!("Note: UpgradeCode should remain constant across versions.");
                    }
                }

                GuidAction::Parse { guid, format } => {
                    match Guid::parse(&guid) {
                        Ok(parsed) => {
                            println!("{}", parsed.format(format.into()));
                        }
                        Err(e) => {
                            eprintln!("Error: {}", e);
                            std::process::exit(1);
                        }
                    }
                }
            }
        }

        Commands::Install {
            msi,
            ui,
            target,
            log,
            property,
            silent,
            no_restart,
            dry_run,
        } => {
            let mut opts = if silent {
                InstallPresets::silent_install(msi)
            } else {
                let mut o = InstallOptions::new(msi);
                o.ui_level = ui.into();
                o
            };

            if let Some(dir) = target {
                opts = opts.with_target_dir(&dir);
            }

            if let Some(log_path) = log {
                opts = opts.with_log(log_path);
            }

            for prop in property {
                if let Some((key, value)) = prop.split_once('=') {
                    opts = opts.with_property(key, value);
                }
            }

            if no_restart {
                opts = opts.no_restart();
            }

            let cmd = MsiExecCommand::build_string(&opts);

            if dry_run {
                println!("Would execute:\n  {}", cmd);
            } else {
                println!("Executing: {}", cmd);
                let result = MsiExecCommand::execute(&opts);
                if result.success {
                    println!("Success: {}", result.message);
                } else {
                    eprintln!("Failed (exit code {}): {}", result.exit_code, result.message);
                    std::process::exit(result.exit_code);
                }
            }
        }

        Commands::Uninstall { target, silent, log, dry_run } => {
            let mut opts = InstallOptions::new(PathBuf::from(&target)).uninstall();

            if silent {
                opts = opts.silent();
            }

            if let Some(log_path) = log {
                opts = opts.with_log(log_path);
            }

            let cmd = MsiExecCommand::build_string(&opts);

            if dry_run {
                println!("Would execute:\n  {}", cmd);
            } else {
                println!("Executing: {}", cmd);
                let result = MsiExecCommand::execute(&opts);
                if result.success {
                    println!("Success: {}", result.message);
                } else {
                    eprintln!("Failed (exit code {}): {}", result.exit_code, result.message);
                    std::process::exit(result.exit_code);
                }
            }
        }

        Commands::Update { msi, silent, log } => {
            let mut opts = InstallOptions::new(msi);

            if silent {
                opts = opts.silent();
            }

            if let Some(log_path) = log {
                opts = opts.with_log(log_path);
            }

            opts = opts.with_property("REINSTALL", "ALL");
            opts = opts.with_property("REINSTALLMODE", "vomus");

            let cmd = MsiExecCommand::build_string(&opts);
            println!("Executing: {}", cmd);
            let result = MsiExecCommand::execute(&opts);
            if result.success {
                println!("Success: {}", result.message);
            } else {
                eprintln!("Failed (exit code {}): {}", result.exit_code, result.message);
                std::process::exit(result.exit_code);
            }
        }

        Commands::Repair { target, silent, log } => {
            let mut opts = InstallOptions::new(PathBuf::from(&target)).repair();

            if silent {
                opts = opts.silent();
            }

            if let Some(log_path) = log {
                opts = opts.with_log(log_path);
            }

            let cmd = MsiExecCommand::build_string(&opts);
            println!("Executing: {}", cmd);
            let result = MsiExecCommand::execute(&opts);
            if result.success {
                println!("Success: {}", result.message);
            } else {
                eprintln!("Failed (exit code {}): {}", result.exit_code, result.message);
                std::process::exit(result.exit_code);
            }
        }

        Commands::Doctor => {
            println!("WiX Environment Check\n");
            println!("Checking WiX installation...");

            #[cfg(windows)]
            {
                // Check for WiX in PATH
                if let Ok(output) = std::process::Command::new("wix").arg("--version").output() {
                    if output.status.success() {
                        let version = String::from_utf8_lossy(&output.stdout);
                        println!("  WiX CLI: {} (OK)", version.trim());
                    }
                } else {
                    println!("  WiX CLI: Not found (run 'wix-init setup')");
                }
            }

            #[cfg(not(windows))]
            {
                println!("  WiX: Only available on Windows");
                println!("  Cross-compilation support: Limited");
            }

            println!("\nChecking .NET SDK...");
            if let Ok(output) = std::process::Command::new("dotnet").arg("--version").output() {
                if output.status.success() {
                    let version = String::from_utf8_lossy(&output.stdout);
                    println!("  .NET SDK: {} (OK)", version.trim());
                }
            } else {
                println!("  .NET SDK: Not found");
            }
        }

        Commands::Setup { no_ide: _no_ide, sandbox: _sandbox } => {
            println!("WiX Development Environment Setup\n");

            #[cfg(windows)]
            {
                println!("Installing WiX Toolset...");
                println!("  Run: dotnet tool install --global wix");

                if !_no_ide {
                    println!("\nIDE Integration:");
                    println!("  VS Code: Install 'WiX Toolset' extension");
                    println!("  Visual Studio: Install 'WiX Toolset Visual Studio Extension'");
                }

                if _sandbox {
                    println!("\nWindows Sandbox:");
                    println!("  Enable via: Enable-WindowsOptionalFeature -FeatureName 'Containers-DisposableClientVM' -Online");
                    println!("  Use 'wix-init sandbox' to generate .wsb file");
                }
            }

            #[cfg(not(windows))]
            {
                println!("WiX development requires Windows.");
                println!("Consider using Windows in a VM or WSL2.");
            }
        }

        Commands::License { license_type, output, author, year } => {
            let year = year.unwrap_or_else(|| {
                use std::time::{SystemTime, UNIX_EPOCH};
                let secs = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
                (1970 + secs / 31_536_000) as u32
            });

            let author = author.unwrap_or_else(|| "Your Company".to_string());

            let license_text = match license_type {
                LicenseTypeArg::Mit => format!(
                    "MIT License\n\nCopyright (c) {} {}\n\nPermission is hereby granted...",
                    year, author
                ),
                LicenseTypeArg::Apache2 => format!(
                    "Apache License\nVersion 2.0, January 2004\n\nCopyright {} {}",
                    year, author
                ),
                LicenseTypeArg::Gpl3 => "GNU General Public License v3.0".to_string(),
                LicenseTypeArg::Bsd3 => format!(
                    "BSD 3-Clause License\n\nCopyright (c) {} {}\nAll rights reserved.",
                    year, author
                ),
                LicenseTypeArg::Proprietary => format!(
                    "PROPRIETARY LICENSE\n\nCopyright (c) {} {}\nAll rights reserved.",
                    year, author
                ),
            };

            // For RTF output
            let rtf_content = format!(
                r"{{\rtf1\ansi\deff0 {{\fonttbl{{\f0 Courier New;}}}}
\f0\fs20
{}
}}",
                license_text.replace('\n', "\\par\n")
            );

            if let Err(e) = std::fs::write(&output, rtf_content) {
                eprintln!("Error writing license file: {}", e);
                std::process::exit(1);
            }

            println!("Created license file: {}", output.display());
        }

        Commands::Sandbox { msi, output } => {
            let msi_mount = msi.map(|p| format!(
                r#"<MappedFolder>
      <HostFolder>{}</HostFolder>
      <SandboxFolder>C:\Install</SandboxFolder>
      <ReadOnly>true</ReadOnly>
    </MappedFolder>"#,
                p.parent().unwrap_or(&PathBuf::from(".")).canonicalize().unwrap_or_default().display()
            )).unwrap_or_default();

            let wsb_content = format!(
                r#"<Configuration>
  <VGpu>Disable</VGpu>
  <Networking>Enable</Networking>
  {}
  <LogonCommand>
    <Command>explorer.exe C:\Install</Command>
  </LogonCommand>
</Configuration>"#,
                msi_mount
            );

            if let Err(e) = std::fs::write(&output, wsb_content) {
                eprintln!("Error writing sandbox file: {}", e);
                std::process::exit(1);
            }

            println!("Created sandbox configuration: {}", output.display());
            println!("\nDouble-click the .wsb file to launch Windows Sandbox.");
        }
    }
}
