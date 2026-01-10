//! Language Server Protocol implementation for WiX Analyzer
//!
//! Provides IDE integration via the Language Server Protocol (LSP).

mod server;
mod actions;

pub use server::{WixLanguageServer, run_server};
pub use actions::CodeActionProvider;
