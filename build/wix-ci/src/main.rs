//! wix-ci CLI - CI/CD pipeline templates for WiX projects

use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;
use wix_ci::{CiGenerator, CiOptions, CiPlatform};

#[derive(Parser)]
#[command(name = "wix-ci")]
#[command(about = "Generate CI/CD pipeline templates for WiX projects")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate a CI/CD workflow file
    Generate {
        /// Target CI/CD platform
        #[arg(value_enum)]
        platform: Platform,

        /// Output directory (default: current directory)
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Project name
        #[arg(short = 'n', long, default_value = "MyInstaller")]
        project_name: String,

        /// WiX version
        #[arg(long, default_value = "5.0.2")]
        wix_version: String,

        /// .NET version
        #[arg(long, default_value = "8.0")]
        dotnet_version: String,

        /// Include code signing step
        #[arg(long)]
        sign: bool,

        /// Skip tests
        #[arg(long)]
        no_tests: bool,

        /// Skip artifact upload
        #[arg(long)]
        no_artifacts: bool,

        /// Include release publishing
        #[arg(long)]
        release: bool,

        /// Build branch
        #[arg(long, default_value = "main")]
        branch: String,

        /// Print to stdout instead of writing file
        #[arg(long)]
        stdout: bool,
    },

    /// List available platforms
    List,

    /// Show example workflow for a platform
    Example {
        /// Target CI/CD platform
        #[arg(value_enum)]
        platform: Platform,
    },
}

#[derive(Clone, Copy, ValueEnum)]
enum Platform {
    Github,
    Azure,
    Gitlab,
}

impl From<Platform> for CiPlatform {
    fn from(p: Platform) -> Self {
        match p {
            Platform::Github => CiPlatform::GitHubActions,
            Platform::Azure => CiPlatform::AzurePipelines,
            Platform::Gitlab => CiPlatform::GitLabCi,
        }
    }
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Generate {
            platform,
            output,
            project_name,
            wix_version,
            dotnet_version,
            sign,
            no_tests,
            no_artifacts,
            release,
            branch,
            stdout,
        } => {
            let options = CiOptions {
                project_name,
                wix_version,
                dotnet_version,
                code_signing: sign,
                run_tests: !no_tests,
                upload_artifacts: !no_artifacts,
                publish_release: release,
                build_branch: branch,
                ..Default::default()
            };

            let generator = CiGenerator::new(options);
            let ci_platform: CiPlatform = platform.into();

            if stdout {
                println!("{}", generator.generate(ci_platform));
            } else {
                let base_path = output.unwrap_or_else(|| PathBuf::from("."));
                match generator.write(ci_platform, &base_path) {
                    Ok(path) => {
                        println!("Generated {} workflow at: {}", ci_platform.as_str(), path);
                    }
                    Err(e) => {
                        eprintln!("Error: {}", e);
                        std::process::exit(1);
                    }
                }
            }
        }

        Commands::List => {
            println!("Available CI/CD platforms:");
            println!();
            for platform in CiPlatform::all() {
                println!("  {:20} {}", platform.as_str(), platform.file_path());
            }
            println!();
            println!("Use 'wix-ci generate <platform>' to create a workflow file.");
        }

        Commands::Example { platform } => {
            let ci_platform: CiPlatform = platform.into();
            let generator = CiGenerator::default();

            println!("# Example {} workflow", ci_platform.as_str());
            println!("# Output file: {}", ci_platform.file_path());
            println!();
            println!("{}", generator.generate(ci_platform));
        }
    }
}
