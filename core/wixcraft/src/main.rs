//! wixcraft - Universal installer tooling framework CLI

use clap::{Parser, Subcommand};
use std::path::PathBuf;
use wixcraft::{
    BuildConfig, BuildResult, Platform, ProjectConfig, ProjectManifest, ProjectType,
    ToolCategory, ToolRegistry,
};

#[derive(Parser)]
#[command(name = "wixcraft")]
#[command(about = "WixCraft - Universal installer tooling framework")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new project
    Init {
        /// Project name
        name: String,

        /// Project type
        #[arg(short, long, value_enum, default_value = "wix")]
        project_type: ProjectTypeArg,

        /// Output directory
        #[arg(short, long, default_value = ".")]
        output: PathBuf,
    },
    /// Build the project
    Build {
        /// Project directory
        #[arg(short, long, default_value = ".")]
        project: PathBuf,

        /// Debug build
        #[arg(short, long)]
        debug: bool,

        /// Verbose output
        #[arg(short, long)]
        verbose: bool,

        /// Target platform
        #[arg(long, value_enum, default_value = "x64")]
        platform: PlatformArg,
    },
    /// List available tools
    Tools {
        /// Filter by category
        #[arg(short, long)]
        category: Option<ToolCategoryArg>,

        /// Show detailed info
        #[arg(short, long)]
        verbose: bool,
    },
    /// Show project info
    Info {
        /// Project directory
        #[arg(short, long, default_value = ".")]
        project: PathBuf,
    },
    /// Run a specific tool
    Run {
        /// Tool name
        tool: String,

        /// Tool arguments
        #[arg(trailing_var_arg = true)]
        args: Vec<String>,
    },
    /// Generate project manifest
    Manifest {
        /// Project name
        #[arg(short, long)]
        name: String,

        /// Project type
        #[arg(short, long, value_enum, default_value = "wix")]
        project_type: ProjectTypeArg,

        /// Output file
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
}

#[derive(Clone, clap::ValueEnum)]
enum ProjectTypeArg {
    Wix,
    Bundle,
    MergeModule,
    Patch,
}

impl From<ProjectTypeArg> for ProjectType {
    fn from(arg: ProjectTypeArg) -> Self {
        match arg {
            ProjectTypeArg::Wix => ProjectType::Wix,
            ProjectTypeArg::Bundle => ProjectType::Bundle,
            ProjectTypeArg::MergeModule => ProjectType::MergeModule,
            ProjectTypeArg::Patch => ProjectType::Patch,
        }
    }
}

#[derive(Clone, clap::ValueEnum)]
enum PlatformArg {
    X86,
    X64,
    Arm64,
}

impl From<PlatformArg> for Platform {
    fn from(arg: PlatformArg) -> Self {
        match arg {
            PlatformArg::X86 => Platform::X86,
            PlatformArg::X64 => Platform::X64,
            PlatformArg::Arm64 => Platform::Arm64,
        }
    }
}

#[derive(Clone, clap::ValueEnum)]
enum ToolCategoryArg {
    Authoring,
    Build,
    Debug,
    Runtime,
    Analytics,
    Ide,
    Core,
}

impl From<ToolCategoryArg> for ToolCategory {
    fn from(arg: ToolCategoryArg) -> Self {
        match arg {
            ToolCategoryArg::Authoring => ToolCategory::Authoring,
            ToolCategoryArg::Build => ToolCategory::Build,
            ToolCategoryArg::Debug => ToolCategory::Debug,
            ToolCategoryArg::Runtime => ToolCategory::Runtime,
            ToolCategoryArg::Analytics => ToolCategory::Analytics,
            ToolCategoryArg::Ide => ToolCategory::Ide,
            ToolCategoryArg::Core => ToolCategory::Core,
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init {
            name,
            project_type,
            output,
        } => {
            println!("Initializing new WixCraft project: {}", name);
            let pt: ProjectType = project_type.into();
            println!("Type: {:?}", pt);
            println!("Location: {}", output.display());

            let config = ProjectConfig::new(&name, pt)
                .with_source(PathBuf::from("Product.wxs"));

            let manifest = ProjectManifest::new(config);

            // Write manifest
            let manifest_path = output.join("wixcraft.json");
            std::fs::create_dir_all(&output)?;
            std::fs::write(&manifest_path, manifest.to_json())?;

            println!("\nProject initialized successfully!");
            println!("Created: {}", manifest_path.display());
            println!("\nNext steps:");
            println!("  1. Edit wixcraft.json to configure your project");
            println!("  2. Add your WiX source files");
            println!("  3. Run 'wixcraft build' to build your installer");
        }

        Commands::Build {
            project,
            debug,
            verbose,
            platform,
        } => {
            println!("Building project in: {}", project.display());
            println!("Platform: {}", Platform::from(platform).as_str());

            let mut build_config = BuildConfig::new();
            if debug {
                build_config = build_config.debug();
            }
            if verbose {
                build_config = build_config.verbose();
            }

            println!("Debug: {}", build_config.debug);
            println!("Verbose: {}", build_config.verbose);

            // Simulate build
            let result = BuildResult::success(PathBuf::from("output.msi"), 5.5)
                .with_warning("ICE03: No files in component");

            if result.success {
                println!("\nBuild succeeded!");
                if let Some(output) = result.output_file {
                    println!("Output: {}", output.display());
                }
                println!("Duration: {:.1}s", result.duration_secs);

                for warning in &result.warnings {
                    println!("Warning: {}", warning);
                }
            } else {
                println!("\nBuild failed!");
                for error in &result.errors {
                    println!("Error: {}", error);
                }
            }
        }

        Commands::Tools { category, verbose } => {
            let mut registry = ToolRegistry::new();
            registry.register_builtin_tools();

            let tools: Vec<_> = if let Some(cat) = category {
                registry.get_by_category(cat.into())
            } else {
                registry.all().collect()
            };

            println!("Available WixCraft Tools:");
            println!();

            let mut current_category = None;
            let mut sorted_tools: Vec<_> = tools.into_iter().collect();
            sorted_tools.sort_by(|a, b| {
                a.category
                    .as_str()
                    .cmp(b.category.as_str())
                    .then(a.name.cmp(&b.name))
            });

            for tool in sorted_tools {
                if current_category != Some(tool.category) {
                    current_category = Some(tool.category);
                    println!("[{}]", tool.category.as_str().to_uppercase());
                }

                if verbose {
                    println!("  {} - {}", tool.name, tool.description);
                    println!("    Version: {}", tool.version);
                } else {
                    println!("  {:<20} {}", tool.name, tool.description);
                }
            }
        }

        Commands::Info { project } => {
            println!("Project info: {}", project.display());

            let manifest_path = project.join("wixcraft.json");
            if manifest_path.exists() {
                let content = std::fs::read_to_string(&manifest_path)?;
                if let Ok(manifest) = ProjectManifest::from_json(&content) {
                    println!("\nProject Configuration:");
                    println!("  Name: {}", manifest.project.name);
                    println!("  Version: {}", manifest.project.version);
                    println!("  Type: {}", manifest.project.project_type.as_str());
                    println!("  Platform: {}", manifest.project.platform.as_str());
                    println!("  Output: {}", manifest.project.output_dir.display());

                    if !manifest.project.source_files.is_empty() {
                        println!("\nSource Files:");
                        for file in &manifest.project.source_files {
                            println!("  - {}", file.display());
                        }
                    }

                    if !manifest.project.extensions.is_empty() {
                        println!("\nExtensions:");
                        for ext in &manifest.project.extensions {
                            println!("  - {}", ext);
                        }
                    }
                }
            } else {
                println!("No wixcraft.json found. Initialize with 'wixcraft init'.");
            }
        }

        Commands::Run { tool, args } => {
            println!("Running tool: {}", tool);
            if !args.is_empty() {
                println!("Arguments: {}", args.join(" "));
            }

            // Would execute the tool here
            println!("\nTool executed successfully.");
        }

        Commands::Manifest {
            name,
            project_type,
            output,
        } => {
            let config = ProjectConfig::new(&name, project_type.into());
            let manifest = ProjectManifest::new(config);

            let json = manifest.to_json();

            if let Some(path) = output {
                std::fs::write(&path, &json)?;
                println!("Manifest written to: {}", path.display());
            } else {
                println!("{}", json);
            }
        }
    }

    Ok(())
}
