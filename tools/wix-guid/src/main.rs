//! wix-guid CLI - GUID generator for WiX installers

use clap::{Parser, Subcommand, ValueEnum};
use std::process::ExitCode;
use wix_guid::{Guid, GuidBatch, GuidFormat, GuidGenerator};

#[derive(Parser)]
#[command(name = "wix-guid")]
#[command(about = "Generate GUIDs for WiX installers")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Number of GUIDs to generate (default mode)
    #[arg(short, long, default_value = "1")]
    count: usize,

    /// Output format
    #[arg(short, long, default_value = "braces")]
    format: Format,

    /// Output as JSON array
    #[arg(long)]
    json: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate random GUIDs
    Random {
        /// Number of GUIDs
        #[arg(short, long, default_value = "1")]
        count: usize,

        /// Output format
        #[arg(short, long, default_value = "braces")]
        format: Format,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Generate deterministic GUID from input
    Hash {
        /// Input string to hash
        input: String,

        /// Output format
        #[arg(short, long, default_value = "braces")]
        format: Format,
    },

    /// Generate component GUID for a file path
    Component {
        /// File path
        path: String,

        /// Product name for namespace
        #[arg(short, long, default_value = "WixCraft")]
        product: String,

        /// Product version for namespace
        #[arg(short, long, default_value = "1.0.0")]
        version: String,

        /// Output format
        #[arg(short, long, default_value = "braces")]
        format: Format,
    },

    /// Generate product and upgrade code pair
    Product {
        /// Product name
        name: String,

        /// Product version
        #[arg(short, long, default_value = "1.0.0")]
        version: String,

        /// Output format
        #[arg(short, long, default_value = "braces")]
        format: Format,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Parse and reformat a GUID
    Parse {
        /// GUID string to parse
        guid: String,

        /// Output format
        #[arg(short, long, default_value = "braces")]
        format: Format,

        /// Show in all formats
        #[arg(long)]
        all: bool,
    },

    /// Validate a GUID string
    Validate {
        /// GUID string to validate
        guid: String,
    },

    /// Show all format options
    Formats,
}

#[derive(Clone, ValueEnum)]
enum Format {
    /// {xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx}
    Braces,
    /// xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx
    Hyphens,
    /// xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx
    Plain,
    /// {XXXXXXXX-XXXX-XXXX-XXXX-XXXXXXXXXXXX}
    Registry,
    /// new Guid("...")
    Csharp,
    /// * (WiX auto-generate)
    Auto,
}

impl From<Format> for GuidFormat {
    fn from(f: Format) -> Self {
        match f {
            Format::Braces => GuidFormat::Braces,
            Format::Hyphens => GuidFormat::Hyphens,
            Format::Plain => GuidFormat::Plain,
            Format::Registry => GuidFormat::Registry,
            Format::Csharp => GuidFormat::CSharp,
            Format::Auto => GuidFormat::WixAuto,
        }
    }
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Random { count, format, json }) => {
            cmd_random(count, format.into(), json)
        }
        Some(Commands::Hash { input, format }) => {
            cmd_hash(&input, format.into())
        }
        Some(Commands::Component { path, product, version, format }) => {
            cmd_component(&path, &product, &version, format.into())
        }
        Some(Commands::Product { name, version, format, json }) => {
            cmd_product(&name, &version, format.into(), json)
        }
        Some(Commands::Parse { guid, format, all }) => {
            cmd_parse(&guid, format.into(), all)
        }
        Some(Commands::Validate { guid }) => {
            cmd_validate(&guid)
        }
        Some(Commands::Formats) => {
            cmd_formats()
        }
        None => {
            // Default: generate random GUIDs
            cmd_random(cli.count, cli.format.into(), cli.json)
        }
    }
}

fn cmd_random(count: usize, format: GuidFormat, json: bool) -> ExitCode {
    let batch = GuidBatch::random(count);
    let formatted = batch.format_all(format);

    if json {
        println!("{}", serde_json::to_string_pretty(&formatted).unwrap());
    } else {
        for guid in formatted {
            println!("{}", guid);
        }
    }

    ExitCode::SUCCESS
}

fn cmd_hash(input: &str, format: GuidFormat) -> ExitCode {
    let guid = Guid::from_hash(input);
    println!("{}", guid.format(format));
    ExitCode::SUCCESS
}

fn cmd_component(path: &str, product: &str, version: &str, format: GuidFormat) -> ExitCode {
    let gen = GuidGenerator::new(product, version);
    let guid = gen.component_guid(path);
    println!("{}", guid.format(format));
    ExitCode::SUCCESS
}

fn cmd_product(name: &str, version: &str, format: GuidFormat, json: bool) -> ExitCode {
    let gen = GuidGenerator::new(name, version);
    let product_code = gen.product_code();
    let upgrade_code = gen.upgrade_code();

    if json {
        let output = serde_json::json!({
            "product_name": name,
            "version": version,
            "product_code": product_code.format(format),
            "upgrade_code": upgrade_code.format(format),
        });
        println!("{}", serde_json::to_string_pretty(&output).unwrap());
    } else {
        println!("Product: {} v{}", name, version);
        println!("ProductCode: {}", product_code.format(format));
        println!("UpgradeCode: {}", upgrade_code.format(format));
    }

    ExitCode::SUCCESS
}

fn cmd_parse(input: &str, format: GuidFormat, all: bool) -> ExitCode {
    match Guid::parse(input) {
        Ok(guid) => {
            if all {
                println!("Braces:   {}", guid.format(GuidFormat::Braces));
                println!("Hyphens:  {}", guid.format(GuidFormat::Hyphens));
                println!("Plain:    {}", guid.format(GuidFormat::Plain));
                println!("Registry: {}", guid.format(GuidFormat::Registry));
                println!("C#:       {}", guid.format(GuidFormat::CSharp));
            } else {
                println!("{}", guid.format(format));
            }
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            ExitCode::FAILURE
        }
    }
}

fn cmd_validate(input: &str) -> ExitCode {
    match Guid::parse(input) {
        Ok(guid) => {
            println!("Valid GUID: {}", guid);
            if guid.is_nil() {
                println!("Warning: This is the nil GUID (all zeros)");
            }
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("Invalid: {}", e);
            ExitCode::FAILURE
        }
    }
}

fn cmd_formats() -> ExitCode {
    println!("Available GUID formats:\n");

    let example = Guid::from_bytes([
        0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf0,
        0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf0,
    ]);

    println!("  braces   - {}", example.format(GuidFormat::Braces));
    println!("  hyphens  - {}", example.format(GuidFormat::Hyphens));
    println!("  plain    - {}", example.format(GuidFormat::Plain));
    println!("  registry - {}", example.format(GuidFormat::Registry));
    println!("  csharp   - {}", example.format(GuidFormat::CSharp));
    println!("  auto     - {}", example.format(GuidFormat::WixAuto));

    ExitCode::SUCCESS
}
