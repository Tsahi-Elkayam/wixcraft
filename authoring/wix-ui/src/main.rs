//! wix-ui - WiX UI sequence generator and customizer

use clap::{Parser, Subcommand, ValueEnum};
use std::fs;
use std::path::PathBuf;
use wix_ui::{UIConfig, UIGenerator, UITemplates, UIType};

#[derive(Parser)]
#[command(name = "wix-ui")]
#[command(about = "Generate WiX UI configurations")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate UI reference XML
    Generate {
        /// UI type to generate
        #[arg(short, long, value_enum, default_value = "install-dir")]
        ui_type: UITypeArg,

        /// Include license dialog
        #[arg(long)]
        license: bool,

        /// Banner bitmap path
        #[arg(long)]
        banner: Option<String>,

        /// Dialog bitmap path
        #[arg(long)]
        dialog_bitmap: Option<String>,

        /// Output file path
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    /// Use a template preset
    Template {
        /// Template name
        #[arg(value_enum)]
        name: TemplateArg,

        /// Output file path
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    /// List available UI types
    List,
    /// Generate a custom dialog
    Dialog {
        /// Dialog ID
        #[arg(short, long)]
        id: String,

        /// Dialog title
        #[arg(short, long)]
        title: String,

        /// Output file path
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
}

#[derive(Clone, ValueEnum)]
enum UITypeArg {
    Minimal,
    InstallDir,
    FeatureTree,
    Mondo,
    Advanced,
}

#[derive(Clone, ValueEnum)]
enum TemplateArg {
    ProgressOnly,
    StandardApp,
    FeatureInstaller,
    Enterprise,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Generate {
            ui_type,
            license,
            banner,
            dialog_bitmap,
            output,
        } => {
            let base_config = match ui_type {
                UITypeArg::Minimal => UIConfig::minimal(),
                UITypeArg::InstallDir => UIConfig::install_dir(),
                UITypeArg::FeatureTree => UIConfig::feature_tree(),
                UITypeArg::Mondo => UIConfig::mondo(),
                UITypeArg::Advanced => UIConfig {
                    ui_type: UIType::Advanced,
                    install_dir_dialog: true,
                    feature_dialog: true,
                    ..Default::default()
                },
            };

            let mut config = if license {
                base_config.with_license()
            } else {
                base_config
            };

            if let Some(b) = banner {
                config = config.with_banner(&b);
            }
            if let Some(d) = dialog_bitmap {
                config = config.with_dialog_bitmap(&d);
            }

            let xml = UIGenerator::generate_ui_ref(&config);

            if let Some(path) = output {
                fs::write(&path, &xml)?;
                println!("Generated UI configuration at: {}", path.display());
            } else {
                println!("{}", xml);
            }
        }

        Commands::Template { name, output } => {
            let config = match name {
                TemplateArg::ProgressOnly => UITemplates::progress_only(),
                TemplateArg::StandardApp => UITemplates::standard_app(),
                TemplateArg::FeatureInstaller => UITemplates::feature_installer(),
                TemplateArg::Enterprise => UITemplates::enterprise(),
            };

            let xml = UIGenerator::generate_ui_ref(&config);

            if let Some(path) = output {
                fs::write(&path, &xml)?;
                println!("Generated template at: {}", path.display());
            } else {
                println!("{}", xml);
            }
        }

        Commands::List => {
            println!("Available UI Types:\n");
            println!("  minimal      - {}", UIType::Minimal.description());
            println!("  install-dir  - {}", UIType::InstallDir.description());
            println!("  feature-tree - {}", UIType::FeatureTree.description());
            println!("  mondo        - {}", UIType::Mondo.description());
            println!("  advanced     - {}", UIType::Advanced.description());
            println!("\nTemplates:\n");
            println!("  progress-only     - Simple progress bar only");
            println!("  standard-app      - InstallDir with license");
            println!("  feature-installer - FeatureTree with license");
            println!("  enterprise        - Full Mondo UI");
        }

        Commands::Dialog { id, title, output } => {
            let dialog = wix_ui::Dialog {
                id,
                title,
                width: 370,
                height: 270,
                controls: vec![],
            };

            let xml = UIGenerator::generate_dialog(&dialog);

            if let Some(path) = output {
                fs::write(&path, &xml)?;
                println!("Generated dialog at: {}", path.display());
            } else {
                println!("{}", xml);
            }
        }
    }

    Ok(())
}
