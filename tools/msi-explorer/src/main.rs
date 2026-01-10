//! MSI Explorer CLI - A modern alternative to Microsoft Orca

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use msi_explorer::{
    diff, export, search, MsiFile, TableCategory,
    MsiBuilder, MsiMetadata, MsiDirectoryDef, MsiComponentDef, MsiFeatureDef, MsiFileDef,
};
use std::io::{self, Write};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "msi-explorer")]
#[command(author, version, about = "Cross-platform MSI database explorer")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Show MSI file information
    Info {
        /// Path to MSI file
        msi: PathBuf,
        /// Show detailed property information
        #[arg(short, long)]
        verbose: bool,
    },

    /// List all tables in MSI
    Tables {
        /// Path to MSI file
        msi: PathBuf,
        /// Group tables by category
        #[arg(short, long)]
        categorize: bool,
    },

    /// View a specific table
    Table {
        /// Path to MSI file
        msi: PathBuf,
        /// Table name
        name: String,
        /// Output format (text, json, csv)
        #[arg(short, long, default_value = "text")]
        format: String,
        /// Maximum rows to display
        #[arg(short, long)]
        limit: Option<usize>,
    },

    /// Search across all tables
    Search {
        /// Path to MSI file
        msi: PathBuf,
        /// Search query
        query: String,
        /// Case-sensitive search
        #[arg(short = 'c', long)]
        case_sensitive: bool,
        /// Search only in specific tables (comma-separated)
        #[arg(short, long)]
        tables: Option<String>,
        /// Maximum results
        #[arg(short, long)]
        max: Option<usize>,
    },

    /// Compare two MSI files
    Diff {
        /// First MSI file
        msi1: PathBuf,
        /// Second MSI file
        msi2: PathBuf,
        /// Output format (text, json)
        #[arg(short, long, default_value = "text")]
        format: String,
        /// Show only summary
        #[arg(short, long)]
        summary: bool,
    },

    /// Export MSI data
    Export {
        /// Path to MSI file
        msi: PathBuf,
        /// Output file (stdout if not specified)
        #[arg(short, long)]
        output: Option<PathBuf>,
        /// Table to export (all tables if not specified)
        #[arg(short, long)]
        table: Option<String>,
        /// Output format (json, csv, sql)
        #[arg(short, long, default_value = "json")]
        format: String,
    },

    /// Validate MSI with ICE rules
    Validate {
        /// Path to MSI file
        msi: PathBuf,
        /// Use only built-in rules
        #[arg(long)]
        builtin: bool,
        /// Output format (text, json)
        #[arg(short, long, default_value = "text")]
        format: String,
        /// Treat warnings as errors
        #[arg(short = 'W', long)]
        warnings_as_errors: bool,
    },

    /// Get a specific property value
    Property {
        /// Path to MSI file
        msi: PathBuf,
        /// Property name
        name: String,
    },

    /// Show table schema
    Schema {
        /// Path to MSI file
        msi: PathBuf,
        /// Table name
        name: String,
    },

    /// Build an MSI from configuration
    Build {
        /// Configuration file (JSON)
        #[arg(short, long)]
        config: Option<PathBuf>,
        /// Output MSI file
        output: PathBuf,
        /// Show what would be built without creating MSI
        #[arg(short = 'n', long)]
        dry_run: bool,
    },

    /// Extract files from MSI
    Extract {
        /// Path to MSI file
        msi: PathBuf,
        /// Output directory
        output: PathBuf,
        /// File pattern to extract (glob)
        #[arg(short, long)]
        pattern: Option<String>,
    },

    /// Create a demo MSI structure
    Demo {
        /// Product name
        #[arg(short, long, default_value = "Demo Product")]
        name: String,
        /// Product version
        #[arg(short, long, default_value = "1.0.0")]
        version: String,
        /// Manufacturer name
        #[arg(short, long, default_value = "Demo Company")]
        manufacturer: String,
    },

    /// Discover silent install parameters
    Silent {
        /// Path to MSI file
        msi: PathBuf,
        /// Output format (text, json, markdown)
        #[arg(short, long, default_value = "text")]
        format: String,
        /// Show only the command line example
        #[arg(short = 'c', long)]
        command_only: bool,
    },
}

fn main() -> Result<()> {
    env_logger::init();
    let cli = Cli::parse();

    match cli.command {
        Commands::Info { msi, verbose } => cmd_info(&msi, verbose),
        Commands::Tables { msi, categorize } => cmd_tables(&msi, categorize),
        Commands::Table { msi, name, format, limit } => cmd_table(&msi, &name, &format, limit),
        Commands::Search { msi, query, case_sensitive, tables, max } => {
            cmd_search(&msi, &query, case_sensitive, tables, max)
        }
        Commands::Diff { msi1, msi2, format, summary } => cmd_diff(&msi1, &msi2, &format, summary),
        Commands::Export { msi, output, table, format } => cmd_export(&msi, output, table, &format),
        Commands::Validate { msi, builtin, format, warnings_as_errors } => {
            cmd_validate(&msi, builtin, &format, warnings_as_errors)
        }
        Commands::Property { msi, name } => cmd_property(&msi, &name),
        Commands::Schema { msi, name } => cmd_schema(&msi, &name),
        Commands::Build { config, output, dry_run } => cmd_build(config, &output, dry_run),
        Commands::Extract { msi, output, pattern } => cmd_extract(&msi, &output, pattern),
        Commands::Demo { name, version, manufacturer } => cmd_demo(&name, &version, &manufacturer),
        Commands::Silent { msi, format, command_only } => cmd_silent(&msi, &format, command_only),
    }
}

fn cmd_info(path: &PathBuf, verbose: bool) -> Result<()> {
    let mut msi = MsiFile::open(path).context("Failed to open MSI file")?;

    println!("MSI File: {}", path.display());
    println!();

    let summary = msi.summary_info()?;
    println!("Summary Information:");
    if let Some(title) = &summary.title {
        println!("  Title:    {}", title);
    }
    if let Some(author) = &summary.author {
        println!("  Author:   {}", author);
    }
    if let Some(subject) = &summary.subject {
        println!("  Subject:  {}", subject);
    }
    if let Some(uuid) = &summary.uuid {
        println!("  Package:  {}", uuid);
    }
    if let Some(platform) = summary.platform() {
        println!("  Platform: {}", platform);
    }

    let stats = msi.stats()?;
    println!();
    println!("Statistics:");
    println!("  File Size:    {} bytes", stats.file_size);
    println!("  Tables:       {}", stats.table_count);
    println!("  Total Rows:   {}", stats.total_rows);
    println!("  Largest:      {} ({} rows)", stats.largest_table, stats.largest_table_rows);

    if verbose {
        println!();
        println!("Common Properties:");
        for (name, value) in msi.get_common_properties()? {
            println!("  {}: {}", name, value);
        }
    }

    Ok(())
}

fn cmd_tables(path: &PathBuf, categorize: bool) -> Result<()> {
    let mut msi = MsiFile::open(path)?;

    if categorize {
        let by_category = msi.tables_by_category();

        let categories = [
            TableCategory::Core,
            TableCategory::File,
            TableCategory::Registry,
            TableCategory::UI,
            TableCategory::CustomAction,
            TableCategory::Service,
            TableCategory::Sequence,
            TableCategory::Validation,
            TableCategory::Other,
        ];

        for category in categories {
            if let Some(tables) = by_category.get(&category) {
                if !tables.is_empty() {
                    println!("\n{}:", category.display_name());
                    for table in tables {
                        if let Ok(t) = msi.get_table(table) {
                            println!("  {} ({} rows)", table, t.row_count());
                        }
                    }
                }
            }
        }
    } else {
        let mut tables = msi.table_names();
        tables.sort();

        for name in tables {
            if let Ok(table) = msi.get_table(&name) {
                println!("{} ({} rows, {} columns)", name, table.row_count(), table.column_count());
            }
        }
    }

    Ok(())
}

fn cmd_table(path: &PathBuf, name: &str, format: &str, limit: Option<usize>) -> Result<()> {
    let mut msi = MsiFile::open(path)?;
    let table = msi.get_table(name)?;

    let mut stdout = io::stdout();

    match format {
        "json" => {
            let json = export::table_to_json(&table);
            println!("{}", serde_json::to_string_pretty(&json)?);
        }
        "csv" => {
            export::table_to_csv(&table, &mut stdout)?;
        }
        _ => {
            let headers: Vec<&str> = table.columns.iter().map(|c| c.name.as_str()).collect();
            println!("{}", headers.join("\t"));
            println!("{}", "-".repeat(headers.len() * 15));

            let rows_to_show = match limit {
                Some(l) => table.rows.iter().take(l).collect::<Vec<_>>(),
                None => table.rows.iter().collect(),
            };

            for row in rows_to_show {
                let values: Vec<String> = row.values.iter()
                    .map(|v| {
                        let s = v.display();
                        if s.len() > 40 { format!("{}...", &s[..37]) } else { s }
                    })
                    .collect();
                println!("{}", values.join("\t"));
            }

            if let Some(l) = limit {
                if table.rows.len() > l {
                    println!("\n... ({} more rows)", table.rows.len() - l);
                }
            }
        }
    }

    Ok(())
}

fn cmd_search(
    path: &PathBuf,
    query: &str,
    case_sensitive: bool,
    tables: Option<String>,
    max: Option<usize>,
) -> Result<()> {
    let mut msi = MsiFile::open(path)?;

    let options = search::SearchOptions {
        case_sensitive,
        tables: tables.map(|t| t.split(',').map(String::from).collect()),
        max_results: max,
        ..Default::default()
    };

    let results = search::search(&mut msi, query, &options)?;

    if results.is_empty() {
        println!("No results found for '{}'", query);
        return Ok(());
    }

    println!("Found {} results for '{}':\n", results.len(), query);

    for result in &results {
        println!(
            "{}.{} [{}]:",
            result.table, result.column, result.primary_key
        );
        println!("  {}", result.highlighted(">>", "<<"));
        println!();
    }

    Ok(())
}

fn cmd_diff(path1: &PathBuf, path2: &PathBuf, format: &str, summary_only: bool) -> Result<()> {
    let mut msi1 = MsiFile::open(path1)?;
    let mut msi2 = MsiFile::open(path2)?;

    let result = diff::compare(&mut msi1, &mut msi2)?;

    if format == "json" {
        println!("{}", serde_json::to_string_pretty(&result)?);
        return Ok(());
    }

    if !result.has_differences() {
        println!("No differences found.");
        return Ok(());
    }

    println!("Differences: {} total changes\n", result.change_count());

    if !result.tables_only_in_first.is_empty() {
        println!("Tables only in first file:");
        for table in &result.tables_only_in_first {
            println!("  - {}", table);
        }
        println!();
    }

    if !result.tables_only_in_second.is_empty() {
        println!("Tables only in second file:");
        for table in &result.tables_only_in_second {
            println!("  + {}", table);
        }
        println!();
    }

    if !summary_only {
        for table_diff in &result.table_diffs {
            println!("Table: {} ({} changes)", table_diff.table_name, table_diff.change_count());

            for row in &table_diff.rows_added {
                println!("  + [{}]", row.primary_key);
            }
            for row in &table_diff.rows_removed {
                println!("  - [{}]", row.primary_key);
            }
            for row_mod in &table_diff.rows_modified {
                println!("  ~ [{}]:", row_mod.primary_key);
                for cell in &row_mod.cell_changes {
                    println!("      {}: {} -> {}", cell.column, cell.old_value, cell.new_value);
                }
            }
            println!();
        }

        if !result.property_diffs.is_empty() {
            println!("Property differences:");
            for prop in &result.property_diffs {
                match (&prop.old_value, &prop.new_value) {
                    (None, Some(v)) => println!("  + {}: {}", prop.name, v),
                    (Some(v), None) => println!("  - {}: {}", prop.name, v),
                    (Some(old), Some(new)) => println!("  ~ {}: {} -> {}", prop.name, old, new),
                    _ => {}
                }
            }
        }
    }

    Ok(())
}

fn cmd_export(
    path: &PathBuf,
    output: Option<PathBuf>,
    table: Option<String>,
    format: &str,
) -> Result<()> {
    let mut msi = MsiFile::open(path)?;

    let mut writer: Box<dyn Write> = match output {
        Some(ref p) => Box::new(std::fs::File::create(p)?),
        None => Box::new(io::stdout()),
    };

    if let Some(table_name) = table {
        let tbl = msi.get_table(&table_name)?;
        match format {
            "csv" => export::table_to_csv(&tbl, &mut writer)?,
            "sql" => export::table_to_sql(&tbl, &mut writer)?,
            _ => {
                let json = export::table_to_json(&tbl);
                writeln!(writer, "{}", serde_json::to_string_pretty(&json)?)?;
            }
        }
    } else {
        let json = export::msi_to_json(&mut msi)?;
        writeln!(writer, "{}", serde_json::to_string_pretty(&json)?)?;
    }

    if let Some(p) = output {
        eprintln!("Exported to {}", p.display());
    }

    Ok(())
}

fn cmd_validate(path: &PathBuf, builtin: bool, format: &str, warnings_as_errors: bool) -> Result<()> {
    use ice_validator::Validator;

    let validator = if builtin {
        Validator::with_builtin_rules()
    } else {
        match ice_validator::rules::default_wixkb_path() {
            Some(db) if db.exists() => Validator::from_wixkb(&db)?,
            _ => {
                eprintln!("Warning: wixkb not found, using built-in rules");
                Validator::with_builtin_rules()
            }
        }
    };

    let result = validator.validate(path)?;

    if format == "json" {
        println!("{}", serde_json::to_string_pretty(&result)?);
        return Ok(());
    }

    println!("Validating: {}\n", path.display());

    for violation in &result.violations {
        println!("{}", violation);
    }

    if !result.violations.is_empty() {
        println!();
    }

    let (errors, warnings, infos) = result.count_by_severity();
    println!("Result: {} errors, {} warnings, {} info", errors, warnings, infos);
    println!("Checked {} rules in {}ms", result.rules_checked, result.duration_ms);

    let has_errors = errors > 0 || (warnings_as_errors && warnings > 0);
    if has_errors {
        std::process::exit(1);
    }

    Ok(())
}

fn cmd_property(path: &PathBuf, name: &str) -> Result<()> {
    let mut msi = MsiFile::open(path)?;

    match msi.get_property(name)? {
        Some(value) => println!("{}", value),
        None => {
            eprintln!("Property '{}' not found", name);
            std::process::exit(1);
        }
    }

    Ok(())
}

fn cmd_schema(path: &PathBuf, name: &str) -> Result<()> {
    let mut msi = MsiFile::open(path)?;
    let table = msi.get_table(name)?;

    println!("Table: {}", name);
    println!("Rows: {}", table.row_count());
    println!();
    println!("Columns:");

    for col in &table.columns {
        let mut flags = Vec::new();
        if col.primary_key {
            flags.push("PK".to_string());
        }
        if col.nullable {
            flags.push("NULL".to_string());
        }
        if col.is_foreign_key() {
            if let Some(ref_table) = col.referenced_table() {
                flags.push(format!("FK->{}", ref_table));
            }
        }

        let flags_str = if flags.is_empty() {
            String::new()
        } else {
            format!(" [{}]", flags.join(", "))
        };

        println!("  {} ({}){}", col.name, col.col_type.display_name(), flags_str);
    }

    Ok(())
}

fn cmd_build(config: Option<PathBuf>, output: &PathBuf, dry_run: bool) -> Result<()> {
    let builder = if let Some(config_path) = config {
        let content = std::fs::read_to_string(&config_path)
            .context("Failed to read config file")?;
        let metadata: MsiMetadata = serde_json::from_str(&content)
            .context("Failed to parse config file")?;
        MsiBuilder::new().with_metadata(metadata)
    } else {
        let metadata = MsiMetadata::new(
            "Default Product",
            "{00000000-0000-0000-0000-000000000000}",
            "{00000000-0000-0000-0000-000000000001}",
            "1.0.0",
        ).with_manufacturer("Default Manufacturer");
        MsiBuilder::new().with_metadata(metadata)
    };

    if dry_run {
        println!("Would build MSI: {}", output.display());
        println!();
        if let Some(meta) = builder.metadata() {
            println!("Product: {}", meta.product_name);
            println!("Version: {}", meta.version);
            println!("Manufacturer: {}", meta.manufacturer);
        }
        println!();
        println!("Directories: {}", builder.directories().len());
        println!("Components: {}", builder.components().len());
        println!("Features: {}", builder.features().len());
        return Ok(());
    }

    if let Some(meta) = builder.metadata() {
        println!("Building MSI: {}", output.display());
        println!("Product: {}", meta.product_name);
        println!("Version: {}", meta.version);
    }

    // Note: Actual MSI creation would require Windows APIs or external tools
    // This is a placeholder for the build structure
    eprintln!("Warning: MSI creation requires Windows APIs (not available on this platform)");
    eprintln!("Use 'wix-build' tool to compile WiX sources into MSI");

    Ok(())
}

fn cmd_extract(path: &PathBuf, output: &PathBuf, pattern: Option<String>) -> Result<()> {
    let mut msi = MsiFile::open(path).context("Failed to open MSI file")?;

    std::fs::create_dir_all(output).context("Failed to create output directory")?;

    let file_table = match msi.get_table("File") {
        Ok(t) => t,
        Err(_) => {
            println!("No files found in MSI");
            return Ok(());
        }
    };

    let pattern_glob = pattern.as_ref().and_then(|p| glob::Pattern::new(p).ok());

    // Find the FileName column index
    let filename_idx = file_table.columns.iter()
        .position(|c| c.name == "FileName")
        .unwrap_or(1); // FileName is typically the second column

    let mut extracted = 0;
    for row in &file_table.rows {
        if let Some(value) = row.values.get(filename_idx) {
            let file_name = value.display();
            let short_name = file_name.split('|').next().unwrap_or(&file_name);
            let long_name = file_name.split('|').nth(1).unwrap_or(short_name);

            if let Some(ref pat) = pattern_glob {
                if !pat.matches(long_name) && !pat.matches(short_name) {
                    continue;
                }
            }

            println!("  {}", long_name);
            extracted += 1;
        }
    }

    println!();
    println!("Found {} files matching pattern", extracted);
    eprintln!("Note: Actual file extraction requires CAB decompression (use external tools)");

    Ok(())
}

fn cmd_demo(name: &str, version: &str, manufacturer: &str) -> Result<()> {
    let metadata = MsiMetadata::new(
        name,
        "{00000000-0000-0000-0000-000000000000}",
        "{00000000-0000-0000-0000-000000000001}",
        version,
    ).with_manufacturer(manufacturer);

    let mut builder = MsiBuilder::new().with_metadata(metadata.clone());

    // Add standard directories
    builder.add_directory(MsiDirectoryDef::new("TARGETDIR", "SourceDir"));
    builder.add_directory(MsiDirectoryDef::new("ProgramFilesFolder", "PFiles")
        .with_parent("TARGETDIR"));
    builder.add_directory(MsiDirectoryDef::new("INSTALLDIR", name)
        .with_parent("ProgramFilesFolder"));

    // Add a component with a file
    let exe_name = format!("{}.exe", name.to_lowercase().replace(' ', ""));
    let mut component = MsiComponentDef::new(
        "MainComponent",
        "{00000000-0000-0000-0000-000000000002}",
        "INSTALLDIR",
    );
    component.add_file(MsiFileDef::new(
        "MainExe",
        &exe_name,
        PathBuf::from("main.exe"),
    ));
    builder.add_component(component);

    // Add a feature
    let mut feature = MsiFeatureDef::new("MainFeature", "Main Feature", 1)
        .with_description(&format!("Installs {}", name));
    feature.add_component("MainComponent");
    builder.add_feature(feature);

    println!("Demo MSI Structure");
    println!("==================");
    println!();
    println!("Product: {}", name);
    println!("Version: {}", version);
    println!("Manufacturer: {}", manufacturer);
    println!();
    println!("Directories:");
    println!("  TARGETDIR (SourceDir)");
    println!("    ProgramFilesFolder (PFiles)");
    println!("      INSTALLDIR ({})", name);
    println!();
    println!("Components:");
    println!("  MainComponent -> INSTALLDIR");
    println!("    MainExe: {}", exe_name);
    println!();
    println!("Features:");
    println!("  MainFeature (Level 1)");
    println!("    -> MainComponent");
    println!();

    // Output JSON config for metadata
    let json = serde_json::to_string_pretty(&metadata)?;
    println!("JSON Configuration (metadata):");
    println!("{}", json);

    Ok(())
}

/// Silent install parameter discovery
fn cmd_silent(path: &PathBuf, format: &str, command_only: bool) -> Result<()> {
    let mut msi = MsiFile::open(path).context("Failed to open MSI file")?;

    // Collect silent install info
    let info = discover_silent_params(&mut msi)?;

    if command_only {
        println!("{}", info.generate_command(path));
        return Ok(());
    }

    match format {
        "json" => {
            println!("{}", serde_json::to_string_pretty(&info)?);
        }
        "markdown" => {
            print!("{}", info.to_markdown(path));
        }
        _ => {
            print!("{}", info.to_text(path));
        }
    }

    Ok(())
}

#[derive(Debug, serde::Serialize)]
struct SilentInstallInfo {
    product_name: String,
    product_version: String,
    manufacturer: String,
    product_code: String,
    upgrade_code: Option<String>,
    public_properties: Vec<PropertyInfo>,
    features: Vec<FeatureInfo>,
    directories: Vec<DirectoryInfo>,
}

#[derive(Debug, serde::Serialize)]
struct PropertyInfo {
    name: String,
    default_value: String,
    description: Option<String>,
    is_directory: bool,
}

#[derive(Debug, serde::Serialize)]
struct FeatureInfo {
    name: String,
    title: String,
    level: i32,
    description: Option<String>,
}

#[derive(Debug, serde::Serialize)]
struct DirectoryInfo {
    name: String,
    default_value: String,
}

impl SilentInstallInfo {
    fn generate_command(&self, path: &std::path::Path) -> String {
        let msi_name = path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("installer.msi");

        let mut cmd = format!("msiexec /i \"{}\" /qn", msi_name);

        // Add common properties
        if let Some(install_dir) = self.directories.iter().find(|d|
            d.name == "INSTALLDIR" || d.name == "INSTALLLOCATION" || d.name == "APPLICATIONFOLDER"
        ) {
            cmd.push_str(&format!(" {}=\"C:\\Program Files\\{}\"",
                install_dir.name, self.product_name));
        }

        // Add feature selection if multiple features
        if self.features.len() > 1 {
            let feature_names: Vec<&str> = self.features.iter()
                .filter(|f| f.level == 1)
                .map(|f| f.name.as_str())
                .collect();
            if !feature_names.is_empty() {
                cmd.push_str(&format!(" ADDLOCAL={}", feature_names.join(",")));
            }
        }

        cmd
    }

    fn to_text(&self, path: &std::path::Path) -> String {
        let mut out = String::new();
        let msi_name = path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("installer.msi");

        out.push_str(&format!("Silent Install Parameters: {}\n", msi_name));
        out.push_str(&"=".repeat(50));
        out.push('\n');
        out.push('\n');

        out.push_str("Product Information:\n");
        out.push_str(&format!("  Name:         {}\n", self.product_name));
        out.push_str(&format!("  Version:      {}\n", self.product_version));
        out.push_str(&format!("  Manufacturer: {}\n", self.manufacturer));
        out.push_str(&format!("  ProductCode:  {}\n", self.product_code));
        if let Some(ref uc) = self.upgrade_code {
            out.push_str(&format!("  UpgradeCode:  {}\n", uc));
        }
        out.push('\n');

        if !self.directories.is_empty() {
            out.push_str("Directory Properties:\n");
            for dir in &self.directories {
                out.push_str(&format!("  {}  (default: {})\n", dir.name, dir.default_value));
            }
            out.push('\n');
        }

        if !self.features.is_empty() {
            out.push_str("Features (for ADDLOCAL):\n");
            for feature in &self.features {
                let level_str = if feature.level == 1 { " [default]" } else { "" };
                out.push_str(&format!("  {}{}\n", feature.name, level_str));
                if !feature.title.is_empty() && feature.title != feature.name {
                    out.push_str(&format!("    Title: {}\n", feature.title));
                }
                if let Some(ref desc) = feature.description {
                    out.push_str(&format!("    Description: {}\n", desc));
                }
            }
            out.push('\n');
        }

        if !self.public_properties.is_empty() {
            out.push_str("Public Properties (settable via command line):\n");
            for prop in &self.public_properties {
                let default = if prop.default_value.is_empty() {
                    "(empty)".to_string()
                } else {
                    prop.default_value.clone()
                };
                out.push_str(&format!("  {} = {}\n", prop.name, default));
            }
            out.push('\n');
        }

        out.push_str("Example Commands:\n");
        out.push_str(&format!("  Silent install:\n    {}\n\n", self.generate_command(path)));
        out.push_str(&format!("  Silent uninstall:\n    msiexec /x \"{}\" /qn\n\n", msi_name));
        out.push_str(&format!("  Uninstall by ProductCode:\n    msiexec /x {} /qn\n", self.product_code));

        out
    }

    fn to_markdown(&self, path: &std::path::Path) -> String {
        let mut out = String::new();
        let msi_name = path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("installer.msi");

        out.push_str(&format!("# Silent Install: {}\n\n", msi_name));

        out.push_str("## Product Information\n\n");
        out.push_str("| Property | Value |\n");
        out.push_str("|----------|-------|\n");
        out.push_str(&format!("| Name | {} |\n", self.product_name));
        out.push_str(&format!("| Version | {} |\n", self.product_version));
        out.push_str(&format!("| Manufacturer | {} |\n", self.manufacturer));
        out.push_str(&format!("| ProductCode | `{}` |\n", self.product_code));
        if let Some(ref uc) = self.upgrade_code {
            out.push_str(&format!("| UpgradeCode | `{}` |\n", uc));
        }
        out.push('\n');

        if !self.directories.is_empty() {
            out.push_str("## Directory Properties\n\n");
            out.push_str("| Property | Default |\n");
            out.push_str("|----------|--------|\n");
            for dir in &self.directories {
                out.push_str(&format!("| `{}` | {} |\n", dir.name, dir.default_value));
            }
            out.push('\n');
        }

        if !self.features.is_empty() {
            out.push_str("## Features\n\n");
            out.push_str("Use `ADDLOCAL=Feature1,Feature2` to select features.\n\n");
            out.push_str("| Feature | Level | Description |\n");
            out.push_str("|---------|-------|-------------|\n");
            for feature in &self.features {
                let level = if feature.level == 1 { "Default" } else { &feature.level.to_string() };
                let desc = feature.description.as_deref().unwrap_or(&feature.title);
                out.push_str(&format!("| `{}` | {} | {} |\n", feature.name, level, desc));
            }
            out.push('\n');
        }

        if !self.public_properties.is_empty() {
            out.push_str("## Public Properties\n\n");
            out.push_str("| Property | Default | Type |\n");
            out.push_str("|----------|---------|------|\n");
            for prop in &self.public_properties {
                let ptype = if prop.is_directory { "Directory" } else { "String" };
                let default = if prop.default_value.is_empty() { "-" } else { &prop.default_value };
                out.push_str(&format!("| `{}` | {} | {} |\n", prop.name, default, ptype));
            }
            out.push('\n');
        }

        out.push_str("## Command Line Examples\n\n");
        out.push_str("### Silent Install\n\n");
        out.push_str("```cmd\n");
        out.push_str(&self.generate_command(path));
        out.push_str("\n```\n\n");

        out.push_str("### Silent Uninstall\n\n");
        out.push_str("```cmd\n");
        out.push_str(&format!("msiexec /x \"{}\" /qn\n", msi_name));
        out.push_str("```\n\n");

        out.push_str("### Uninstall by ProductCode\n\n");
        out.push_str("```cmd\n");
        out.push_str(&format!("msiexec /x {} /qn\n", self.product_code));
        out.push_str("```\n");

        out
    }
}

fn discover_silent_params(msi: &mut MsiFile) -> Result<SilentInstallInfo> {
    // Get product information
    let product_name = msi.get_property("ProductName")?.unwrap_or_default();
    let product_version = msi.get_property("ProductVersion")?.unwrap_or_default();
    let manufacturer = msi.get_property("Manufacturer")?.unwrap_or_default();
    let product_code = msi.get_property("ProductCode")?.unwrap_or_default();
    let upgrade_code = msi.get_property("UpgradeCode")?;

    // Get public properties (uppercase names are public per MSI convention)
    let mut public_properties = Vec::new();
    if let Ok(prop_table) = msi.get_table("Property") {
        let name_idx = prop_table.columns.iter().position(|c| c.name == "Property").unwrap_or(0);
        let value_idx = prop_table.columns.iter().position(|c| c.name == "Value").unwrap_or(1);

        for row in &prop_table.rows {
            if let (Some(name), Some(value)) = (row.values.get(name_idx), row.values.get(value_idx)) {
                let name_str = name.display();
                // Public properties are all uppercase (MSI convention)
                if name_str.chars().all(|c| c.is_uppercase() || c.is_numeric() || c == '_') {
                    // Skip internal properties
                    if !is_internal_property(&name_str) {
                        let is_dir = name_str.ends_with("DIR") ||
                                     name_str.ends_with("FOLDER") ||
                                     name_str == "INSTALLDIR" ||
                                     name_str == "INSTALLLOCATION";
                        public_properties.push(PropertyInfo {
                            name: name_str,
                            default_value: value.display(),
                            description: None,
                            is_directory: is_dir,
                        });
                    }
                }
            }
        }
    }

    // Sort properties: directories first, then alphabetically
    public_properties.sort_by(|a, b| {
        match (a.is_directory, b.is_directory) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.name.cmp(&b.name),
        }
    });

    // Get features
    let mut features = Vec::new();
    if let Ok(feature_table) = msi.get_table("Feature") {
        let feat_idx = feature_table.columns.iter().position(|c| c.name == "Feature").unwrap_or(0);
        let title_idx = feature_table.columns.iter().position(|c| c.name == "Title").unwrap_or(2);
        let desc_idx = feature_table.columns.iter().position(|c| c.name == "Description").unwrap_or(3);
        let level_idx = feature_table.columns.iter().position(|c| c.name == "Level").unwrap_or(5);

        for row in &feature_table.rows {
            let name = row.values.get(feat_idx).map(|v| v.display()).unwrap_or_default();
            let title = row.values.get(title_idx).map(|v| v.display()).unwrap_or_default();
            let description = row.values.get(desc_idx).map(|v| v.display()).filter(|s| !s.is_empty());
            let level: i32 = row.values.get(level_idx)
                .and_then(|v| v.display().parse().ok())
                .unwrap_or(1);

            features.push(FeatureInfo {
                name,
                title,
                level,
                description,
            });
        }
    }

    // Get important directories
    let mut directories = Vec::new();
    if let Ok(dir_table) = msi.get_table("Directory") {
        let dir_idx = dir_table.columns.iter().position(|c| c.name == "Directory").unwrap_or(0);
        let name_idx = dir_table.columns.iter().position(|c| c.name == "DefaultDir").unwrap_or(2);

        for row in &dir_table.rows {
            let dir_name = row.values.get(dir_idx).map(|v| v.display()).unwrap_or_default();
            let default_val = row.values.get(name_idx).map(|v| v.display()).unwrap_or_default();

            // Only include settable directory properties
            if is_settable_directory(&dir_name) {
                directories.push(DirectoryInfo {
                    name: dir_name,
                    default_value: default_val,
                });
            }
        }
    }

    Ok(SilentInstallInfo {
        product_name,
        product_version,
        manufacturer,
        product_code,
        upgrade_code,
        public_properties,
        features,
        directories,
    })
}

fn is_internal_property(name: &str) -> bool {
    // Properties that are internal/not meant to be set by users
    matches!(name,
        "ProductCode" | "UpgradeCode" | "ProductVersion" | "ProductName" |
        "Manufacturer" | "ProductLanguage" | "ALLUSERS" |
        "ARPCONTACT" | "ARPCOMMENTS" | "ARPHELPLINK" | "ARPHELPTELEPHONE" |
        "ARPINSTALLLOCATION" | "ARPNOMODIFY" | "ARPNOREMOVE" | "ARPNOREPAIR" |
        "ARPPRODUCTICON" | "ARPREADME" | "ARPSIZE" | "ARPSYSTEMCOMPONENT" |
        "ARPURLINFOABOUT" | "ARPURLUPDATEINFO" |
        "MSIINSTALLPERUSER" | "MSIFASTINSTALL" | "MSIDISABLERMRESTART" |
        "REBOOT" | "REBOOTPROMPT" | "MSIRESTARTMANAGERCONTROL"
    )
}

fn is_settable_directory(name: &str) -> bool {
    // Directory properties that users commonly want to set
    matches!(name,
        "INSTALLDIR" | "INSTALLLOCATION" | "APPLICATIONFOLDER" | "APPDIR" |
        "TARGETDIR"
    ) || (name.chars().all(|c| c.is_uppercase() || c.is_numeric() || c == '_') &&
         (name.ends_with("DIR") || name.ends_with("FOLDER")))
}
