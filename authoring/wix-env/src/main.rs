//! wix-env CLI - Environment variable and PATH configuration helper
//!
//! Usage:
//!   wix-env generate --var "MY_HOME=[INSTALLDIR]" --path "[INSTALLDIR]bin"
//!   wix-env template java-home --name JAVA_HOME --scope system
//!   wix-env template add-to-path --path "[INSTALLDIR]bin"

use anyhow::Result;
use clap::{Parser, Subcommand};
use wix_env::*;

#[derive(Parser)]
#[command(name = "wix-env")]
#[command(about = "Environment variable and PATH configuration helper for WiX")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate WiX XML for environment variables
    Generate {
        /// Environment variables (format: NAME=value[@scope][:action])
        #[arg(short, long)]
        var: Vec<String>,

        /// PATH entries to add
        #[arg(short, long)]
        path: Vec<String>,

        /// Prepend PATH entries (default: append)
        #[arg(long)]
        prepend_path: bool,

        /// Scope for PATH entries (user, system)
        #[arg(long, default_value = "user")]
        path_scope: String,

        /// Component ID
        #[arg(long, default_value = "EnvVars")]
        component_id: String,

        /// WiX version (3, 4)
        #[arg(long, default_value = "4")]
        wix_version: u8,

        /// Output format (xml, json)
        #[arg(short, long, default_value = "xml")]
        format: String,
    },

    /// Generate from common templates
    Template {
        #[command(subcommand)]
        template: TemplateCommand,
    },

    /// Show examples
    Examples,
}

#[derive(Subcommand)]
enum TemplateCommand {
    /// Generate JAVA_HOME style configuration (HOME var + bin in PATH)
    JavaHome {
        /// Variable name (e.g., JAVA_HOME, PYTHON_HOME)
        #[arg(short, long)]
        name: String,

        /// Scope (user, system)
        #[arg(short, long, default_value = "user")]
        scope: String,

        /// Make permanent (survive uninstall)
        #[arg(long)]
        permanent: bool,
    },

    /// Add installation directory to PATH
    AddToPath {
        /// Path to add (default: [INSTALLDIR])
        #[arg(short, long, default_value = "[INSTALLDIR]")]
        path: String,

        /// Scope (user, system)
        #[arg(short, long, default_value = "user")]
        scope: String,

        /// Prepend instead of append
        #[arg(long)]
        prepend: bool,

        /// Make permanent
        #[arg(long)]
        permanent: bool,
    },

    /// Set application home directory variable
    AppHome {
        /// Variable name
        #[arg(short, long)]
        name: String,

        /// Path value (default: [INSTALLDIR])
        #[arg(short, long, default_value = "[INSTALLDIR]")]
        path: String,

        /// Scope (user, system)
        #[arg(short, long, default_value = "user")]
        scope: String,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Generate {
            var,
            path,
            prepend_path,
            path_scope,
            component_id,
            wix_version,
            format,
        } => {
            let mut gen = EnvGenerator::new().with_component_id(&component_id);

            // Parse and add variables
            for v in &var {
                let env_var = parse_env_definition(v)?;
                gen.add_variable(env_var);
            }

            // Parse and add PATH entries
            let scope = parse_scope(&path_scope)?;
            for p in &path {
                let mut entry = PathEntry::new(p);
                if scope == EnvScope::System {
                    entry = entry.system();
                }
                if prepend_path {
                    entry = entry.prepend();
                }
                gen.add_path(entry);
            }

            // Generate output
            match format.as_str() {
                "json" => {
                    let output = serde_json::json!({
                        "variables": var,
                        "paths": path,
                        "component_id": component_id,
                        "wix_version": wix_version,
                    });
                    println!("{}", serde_json::to_string_pretty(&output)?);
                }
                _ => {
                    let xml = if wix_version == 3 {
                        gen.generate_wix3()
                    } else {
                        gen.generate_wix4()
                    };
                    println!("{}", xml);
                }
            }
        }

        Commands::Template { template } => match template {
            TemplateCommand::JavaHome {
                name,
                scope,
                permanent,
            } => {
                let scope = parse_scope(&scope)?;
                let (mut home_var, mut path_entry) = EnvTemplates::java_home_style(&name, scope);

                if permanent {
                    home_var = home_var.permanent();
                    path_entry = path_entry.permanent();
                }

                let mut gen = EnvGenerator::new()
                    .with_component_id(&format!("{}Env", name.replace('_', "")));
                gen.add_variable(home_var);
                gen.add_path(path_entry);

                println!("{}", gen.generate_wix4());
                println!();
                println!("<!-- Usage: Add ComponentRef to your Feature -->");
                println!(
                    "<!-- <ComponentRef Id=\"{}Env\" /> -->",
                    name.replace('_', "")
                );
            }

            TemplateCommand::AddToPath {
                path,
                scope,
                prepend,
                permanent,
            } => {
                let scope = parse_scope(&scope)?;
                let mut entry = PathEntry::new(&path);
                if scope == EnvScope::System {
                    entry = entry.system();
                }
                if prepend {
                    entry = entry.prepend();
                }
                if permanent {
                    entry = entry.permanent();
                }

                let mut gen = EnvGenerator::new().with_component_id("PathConfig");
                gen.add_path(entry);

                println!("{}", gen.generate_wix4());
            }

            TemplateCommand::AppHome { name, path, scope } => {
                let scope = parse_scope(&scope)?;
                let mut var = EnvVariable::new(&name, &path);
                if scope == EnvScope::System {
                    var = var.system();
                }

                let mut gen = EnvGenerator::new()
                    .with_component_id(&format!("{}Home", name.replace('_', "")));
                gen.add_variable(var);

                println!("{}", gen.generate_wix4());
            }
        },

        Commands::Examples => {
            println!("wix-env Examples");
            println!("{}", "=".repeat(60));
            println!();
            println!("1. Add installation directory to user PATH:");
            println!("   wix-env template add-to-path --path \"[INSTALLDIR]bin\"");
            println!();
            println!("2. Set JAVA_HOME style configuration:");
            println!("   wix-env template java-home --name JAVA_HOME --scope system");
            println!();
            println!("3. Set custom environment variable:");
            println!("   wix-env generate --var \"MY_APP_HOME=[INSTALLDIR]@system\"");
            println!();
            println!("4. Multiple variables and PATH entries:");
            println!("   wix-env generate \\");
            println!("     --var \"APP_HOME=[INSTALLDIR]\" \\");
            println!("     --var \"APP_CONFIG=[LocalAppDataFolder]MyApp\" \\");
            println!("     --path \"[INSTALLDIR]bin\" \\");
            println!("     --path \"[INSTALLDIR]scripts\"");
            println!();
            println!("5. Append to existing variable:");
            println!("   wix-env generate --var \"PYTHONPATH=[INSTALLDIR]lib:append\"");
            println!();
            println!("Variable format: NAME=value[@scope][:action]");
            println!("  @scope: user (default), system");
            println!("  :action: set (default), append, prepend, create");
        }
    }

    Ok(())
}

fn parse_scope(s: &str) -> Result<EnvScope> {
    match s.to_lowercase().as_str() {
        "user" => Ok(EnvScope::User),
        "system" | "machine" => Ok(EnvScope::System),
        _ => Err(anyhow::anyhow!("Invalid scope: {}. Use: user, system", s)),
    }
}
