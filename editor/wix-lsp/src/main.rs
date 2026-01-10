//! WiX Language Server executable
//!
//! This is the entry point for the WiX Language Server. It can be run as:
//! - stdio mode (default): For editor integration
//! - TCP mode: For debugging (not yet implemented)

use tower_lsp::{LspService, Server};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};
use wix_lsp::{LspServer, PluginRegistry, WixPlugin};

#[tokio::main]
async fn main() {
    // Initialize logging
    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .with(tracing_subscriber::fmt::layer().with_writer(std::io::stderr))
        .init();

    tracing::info!("Starting WiX Language Server");

    // Create plugin registry and register WiX plugin
    let mut registry = PluginRegistry::new();
    registry.register(WixPlugin::new());

    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(|client| LspServer::new(client, registry));
    Server::new(stdin, stdout, socket).serve(service).await;
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_version() {
        // Verify package version is accessible
        let version = env!("CARGO_PKG_VERSION");
        assert!(!version.is_empty());
    }
}
