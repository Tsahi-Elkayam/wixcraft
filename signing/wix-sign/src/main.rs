//! wix-sign CLI - Code signing helper for MSI packages

use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;
use std::process::Command;
use wix_sign::{HashAlgorithm, SignConfig, SignProvider, TimestampServer};

#[derive(Parser)]
#[command(name = "wix-sign")]
#[command(about = "Code signing helper for MSI packages")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Sign an MSI file
    Sign {
        /// File to sign
        file: PathBuf,

        /// Signing provider
        #[arg(short, long, value_enum, default_value = "signtool")]
        provider: Provider,

        /// Certificate file (PFX)
        #[arg(short = 'f', long)]
        certificate: Option<PathBuf>,

        /// Certificate password
        #[arg(short = 'p', long)]
        password: Option<String>,

        /// Certificate thumbprint (for store)
        #[arg(long)]
        thumbprint: Option<String>,

        /// Certificate store name
        #[arg(short = 's', long)]
        store: Option<String>,

        /// Certificate subject name
        #[arg(short = 'n', long)]
        subject: Option<String>,

        /// Timestamp server
        #[arg(short = 't', long, value_enum, default_value = "digicert")]
        timestamp: Timestamp,

        /// Custom timestamp URL
        #[arg(long)]
        timestamp_url: Option<String>,

        /// Hash algorithm
        #[arg(long, value_enum, default_value = "sha256")]
        algorithm: Algorithm,

        /// Description
        #[arg(short = 'd', long)]
        description: Option<String>,

        /// Description URL
        #[arg(long)]
        description_url: Option<String>,

        /// Append signature (dual signing)
        #[arg(long)]
        append: bool,

        /// Verbose output
        #[arg(short, long)]
        verbose: bool,

        /// Dry run - show command without executing
        #[arg(long)]
        dry_run: bool,

        /// Azure Key Vault URL
        #[arg(long)]
        azure_vault: Option<String>,

        /// Azure Client ID
        #[arg(long)]
        azure_client: Option<String>,

        /// Azure Tenant ID
        #[arg(long)]
        azure_tenant: Option<String>,
    },

    /// Show signing command without executing
    Show {
        /// File to sign
        file: PathBuf,

        /// Certificate file (PFX)
        #[arg(short = 'f', long)]
        certificate: Option<PathBuf>,

        /// Certificate password
        #[arg(short = 'p', long)]
        password: Option<String>,

        /// Signing provider
        #[arg(long, value_enum, default_value = "signtool")]
        provider: Provider,
    },

    /// List available timestamp servers
    Timestamps,

    /// Verify a signed file
    Verify {
        /// File to verify
        file: PathBuf,
    },
}

#[derive(Clone, Copy, ValueEnum)]
enum Provider {
    Signtool,
    Azure,
    Osslsigncode,
}

impl From<Provider> for SignProvider {
    fn from(p: Provider) -> Self {
        match p {
            Provider::Signtool => SignProvider::SignTool,
            Provider::Azure => SignProvider::AzureSignTool,
            Provider::Osslsigncode => SignProvider::Osslsigncode,
        }
    }
}

#[derive(Clone, Copy, ValueEnum)]
enum Timestamp {
    Digicert,
    Sectigo,
    Globalsign,
    None,
}

impl From<Timestamp> for TimestampServer {
    fn from(t: Timestamp) -> Self {
        match t {
            Timestamp::Digicert => TimestampServer::DigiCert,
            Timestamp::Sectigo => TimestampServer::Sectigo,
            Timestamp::Globalsign => TimestampServer::GlobalSign,
            Timestamp::None => TimestampServer::None,
        }
    }
}

#[derive(Clone, Copy, ValueEnum)]
enum Algorithm {
    Sha1,
    Sha256,
    Sha384,
    Sha512,
}

impl From<Algorithm> for HashAlgorithm {
    fn from(a: Algorithm) -> Self {
        match a {
            Algorithm::Sha1 => HashAlgorithm::SHA1,
            Algorithm::Sha256 => HashAlgorithm::SHA256,
            Algorithm::Sha384 => HashAlgorithm::SHA384,
            Algorithm::Sha512 => HashAlgorithm::SHA512,
        }
    }
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Sign {
            file,
            provider,
            certificate,
            password,
            thumbprint,
            store,
            subject,
            timestamp,
            timestamp_url,
            algorithm,
            description,
            description_url,
            append,
            verbose,
            dry_run,
            azure_vault,
            azure_client,
            azure_tenant,
        } => {
            let ts = if let Some(url) = timestamp_url {
                TimestampServer::Custom(url)
            } else {
                timestamp.into()
            };

            let config = SignConfig {
                provider: provider.into(),
                certificate_path: certificate,
                password,
                thumbprint,
                store_name: store,
                subject,
                timestamp_server: ts,
                hash_algorithm: algorithm.into(),
                description,
                description_url,
                append_signature: append,
                verbose,
                azure_vault_url: azure_vault,
                azure_client_id: azure_client,
                azure_tenant_id: azure_tenant,
            };

            if let Err(e) = config.validate() {
                eprintln!("Configuration error: {}", e);
                std::process::exit(1);
            }

            let cmd = config.build_command(&file.display().to_string());

            if dry_run || verbose {
                println!("Command: {}", cmd);
                if dry_run {
                    return;
                }
            }

            // Execute the signing command
            let sign_provider: SignProvider = provider.into();
            let args = config.build_args(&file.display().to_string());

            let result = Command::new(sign_provider.executable())
                .args(&args[1..]) // Skip the "sign" verb which is args[0]
                .output();

            match result {
                Ok(output) => {
                    if output.status.success() {
                        println!("Successfully signed: {}", file.display());
                        if verbose {
                            println!("{}", String::from_utf8_lossy(&output.stdout));
                        }
                    } else {
                        eprintln!("Signing failed");
                        eprintln!("{}", String::from_utf8_lossy(&output.stderr));
                        std::process::exit(1);
                    }
                }
                Err(e) => {
                    eprintln!(
                        "Failed to execute {}: {}",
                        sign_provider.executable(),
                        e
                    );
                    eprintln!("Make sure {} is installed and in PATH", sign_provider.executable());
                    std::process::exit(1);
                }
            }
        }

        Commands::Show {
            file,
            certificate,
            password,
            provider,
        } => {
            let config = SignConfig {
                provider: provider.into(),
                certificate_path: certificate,
                password,
                ..Default::default()
            };

            println!("{}", config.build_command(&file.display().to_string()));
        }

        Commands::Timestamps => {
            println!("Available timestamp servers:");
            println!();
            println!("  DigiCert:    http://timestamp.digicert.com");
            println!("  Sectigo:     http://timestamp.sectigo.com");
            println!("  GlobalSign:  http://timestamp.globalsign.com/tsa/r6advanced1");
            println!();
            println!("Use --timestamp-url to specify a custom timestamp server.");
        }

        Commands::Verify { file } => {
            if !file.exists() {
                eprintln!("File not found: {}", file.display());
                std::process::exit(1);
            }

            // Try to verify using signtool
            let result = Command::new("signtool")
                .args(["verify", "/pa", "/v", &file.display().to_string()])
                .output();

            match result {
                Ok(output) => {
                    if output.status.success() {
                        println!("✓ File is signed: {}", file.display());
                        println!();
                        println!("{}", String::from_utf8_lossy(&output.stdout));
                    } else {
                        println!("✗ File is not signed or signature invalid: {}", file.display());
                        if !output.stderr.is_empty() {
                            println!("{}", String::from_utf8_lossy(&output.stderr));
                        }
                        std::process::exit(1);
                    }
                }
                Err(e) => {
                    eprintln!("Failed to run signtool verify: {}", e);
                    eprintln!("Make sure signtool is installed and in PATH");
                    std::process::exit(1);
                }
            }
        }
    }
}
