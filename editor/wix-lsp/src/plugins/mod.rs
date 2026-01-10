//! Plugin system for wix-lsp
//!
//! This module provides the plugin architecture that allows different
//! language plugins to be registered with the LSP server.

pub mod registry;
pub mod traits;
pub mod wix;

pub use registry::{FullPluginDyn, PluginRegistry};
pub use traits::{
    Completion, CompletionKind, CompletionProvider, Diagnostic, DiagnosticProvider,
    DiagnosticSeverity, FormatProvider, FullPlugin, HoverInfo, HoverProvider, HoverRange,
    LanguagePlugin, Symbol, SymbolKind, SymbolProvider, SymbolRange,
};
