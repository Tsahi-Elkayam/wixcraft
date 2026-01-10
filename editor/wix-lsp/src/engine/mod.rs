//! LSP engine core
//!
//! Generic LSP server implementation that delegates to registered plugins.

pub mod config;
pub mod convert;
pub mod document;
pub mod server;

pub use config::EngineConfig;
pub use document::DocumentManager;
pub use server::LspServer;
