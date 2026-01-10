//! Plugin trait definitions
//!
//! These traits define the interface that language plugins must implement
//! to provide IDE features like completion, hover, symbols, etc.

use std::path::Path;

/// Completion item returned by plugins
#[derive(Debug, Clone)]
pub struct Completion {
    pub label: String,
    pub kind: CompletionKind,
    pub detail: Option<String>,
    pub documentation: Option<String>,
    pub insert_text: String,
    pub sort_priority: u32,
}

/// Kind of completion item
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompletionKind {
    Element,
    Attribute,
    Value,
    Snippet,
    Directory,
    Property,
    Keyword,
}

/// Hover information returned by plugins
#[derive(Debug, Clone)]
pub struct HoverInfo {
    pub contents: String,
    pub range: Option<HoverRange>,
}

/// Range for hover highlighting
#[derive(Debug, Clone, Copy)]
pub struct HoverRange {
    pub start_line: u32,
    pub start_col: u32,
    pub end_line: u32,
    pub end_col: u32,
}

/// Document symbol returned by plugins
#[derive(Debug, Clone)]
pub struct Symbol {
    pub name: String,
    pub kind: SymbolKind,
    pub detail: Option<String>,
    pub range: SymbolRange,
    pub selection_range: SymbolRange,
    pub children: Vec<Symbol>,
}

/// Symbol kind (maps to LSP SymbolKind)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SymbolKind {
    File,
    Module,
    Namespace,
    Class,
    Function,
    Variable,
    Constant,
    String,
    Property,
    Key,
    Struct,
    Event,
    Operator,
    TypeParameter,
}

/// Range in document
#[derive(Debug, Clone, Copy)]
pub struct SymbolRange {
    pub start_line: u32,
    pub start_col: u32,
    pub end_line: u32,
    pub end_col: u32,
}

/// Diagnostic returned by plugins
#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub rule_id: String,
    pub message: String,
    pub severity: DiagnosticSeverity,
    pub line: u32,
    pub column: u32,
    pub length: u32,
    pub help: Option<String>,
}

/// Diagnostic severity
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticSeverity {
    Error,
    Warning,
    Info,
    Hint,
}

/// Core plugin trait - provides metadata about the plugin
pub trait LanguagePlugin: Send + Sync {
    /// Plugin name
    fn name(&self) -> &str;

    /// File extensions this plugin handles (e.g., [".wxs", ".wxi"])
    fn file_extensions(&self) -> &[&str];

    /// Characters that trigger completion
    fn trigger_characters(&self) -> &[char];

    /// Initialize the plugin with a data path
    fn initialize(&mut self, data_path: &Path) -> Result<(), String>;

    /// Check if plugin is initialized
    fn is_initialized(&self) -> bool;
}

/// Provides code completion
pub trait CompletionProvider: Send + Sync {
    fn complete(&self, source: &str, line: u32, column: u32) -> Vec<Completion>;
}

/// Provides hover information
pub trait HoverProvider: Send + Sync {
    fn hover(&self, source: &str, line: u32, column: u32) -> Option<HoverInfo>;
}

/// Provides document symbols for outline view
pub trait SymbolProvider: Send + Sync {
    fn symbols(&self, source: &str) -> Result<Vec<Symbol>, String>;
}

/// Provides diagnostics (linting)
pub trait DiagnosticProvider: Send + Sync {
    fn diagnose(&self, source: &str, path: &Path) -> Vec<Diagnostic>;
}

/// Provides document formatting
pub trait FormatProvider: Send + Sync {
    fn format(&self, source: &str) -> Result<String, String>;
}

/// Combined trait for a full-featured plugin
pub trait FullPlugin:
    LanguagePlugin + CompletionProvider + HoverProvider + SymbolProvider + DiagnosticProvider + FormatProvider
{
}

// Auto-implement FullPlugin for any type that implements all traits
impl<T> FullPlugin for T where
    T: LanguagePlugin
        + CompletionProvider
        + HoverProvider
        + SymbolProvider
        + DiagnosticProvider
        + FormatProvider
{
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_completion_kind_equality() {
        assert_eq!(CompletionKind::Element, CompletionKind::Element);
        assert_ne!(CompletionKind::Element, CompletionKind::Attribute);
    }

    #[test]
    fn test_diagnostic_severity() {
        assert_eq!(DiagnosticSeverity::Error, DiagnosticSeverity::Error);
        assert_ne!(DiagnosticSeverity::Error, DiagnosticSeverity::Warning);
    }

    #[test]
    fn test_symbol_kind() {
        assert_eq!(SymbolKind::File, SymbolKind::File);
        assert_ne!(SymbolKind::File, SymbolKind::Module);
    }
}
