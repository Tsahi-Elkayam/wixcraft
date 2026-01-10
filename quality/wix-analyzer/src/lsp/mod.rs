//! Language Server Protocol implementation for WiX Analyzer
//!
//! Provides IDE integration via the Language Server Protocol (LSP).

mod actions;
mod server;

pub use actions::CodeActionProvider;
pub use server::{run_server, WixLanguageServer};
