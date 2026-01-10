//! WiX language plugin
//!
//! Implements all plugin traits for WiX installer development.

use crate::plugins::registry::FullPluginDyn;
use crate::plugins::traits::{
    Completion, CompletionProvider, Diagnostic, DiagnosticProvider, FormatProvider, HoverInfo,
    HoverProvider, HoverRange, LanguagePlugin, Symbol, SymbolKind, SymbolProvider, SymbolRange,
};
use std::path::Path;

// Re-export for convenience
pub use wix_fmt::{FormatConfig, Formatter};
pub use wix_hover::HoverProvider as WixHoverProvider;
pub use wix_symbols::extract_symbols;

/// WiX language plugin
pub struct WixPlugin {
    /// Plugin initialized flag
    initialized: bool,
    /// Hover provider
    hover: Option<wix_hover::HoverProvider>,
    /// Formatter
    formatter: Formatter,
}

impl WixPlugin {
    /// Create a new uninitialized WiX plugin
    pub fn new() -> Self {
        Self {
            initialized: false,
            hover: None,
            formatter: Formatter::new(FormatConfig::default()),
        }
    }

    /// Create and initialize with a data path
    pub fn with_data_path(data_path: &Path) -> Result<Self, String> {
        let mut plugin = Self::new();
        plugin.initialize(data_path)?;
        Ok(plugin)
    }
}

impl Default for WixPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl LanguagePlugin for WixPlugin {
    fn name(&self) -> &str {
        "wix"
    }

    fn file_extensions(&self) -> &[&str] {
        &[".wxs", ".wxi", ".wxl"]
    }

    fn trigger_characters(&self) -> &[char] {
        &['<', ' ', '"', '=']
    }

    fn initialize(&mut self, data_path: &Path) -> Result<(), String> {
        // Initialize hover
        match wix_hover::WixData::load(data_path) {
            Ok(data) => {
                tracing::info!("WiX hover provider initialized");
                self.hover = Some(wix_hover::HoverProvider::new(data));
            }
            Err(e) => {
                tracing::warn!("Failed to initialize hover: {}", e);
            }
        }

        // Initialize formatter with wix-data
        match wix_fmt::WixData::load(data_path) {
            Ok(data) => {
                self.formatter = Formatter::with_wix_data(FormatConfig::default(), data);
                tracing::info!("WiX formatter initialized with data ordering");
            }
            Err(e) => {
                tracing::warn!("Failed to load formatter data: {}", e);
            }
        }

        self.initialized = true;
        Ok(())
    }

    fn is_initialized(&self) -> bool {
        self.initialized
    }
}

impl CompletionProvider for WixPlugin {
    fn complete(&self, _source: &str, _line: u32, _column: u32) -> Vec<Completion> {
        // TODO: Integrate wintellisense for completion support
        Vec::new()
    }
}

impl HoverProvider for WixPlugin {
    fn hover(&self, source: &str, line: u32, column: u32) -> Option<HoverInfo> {
        let provider = self.hover.as_ref()?;
        let info = provider.hover(source, line, column)?;

        Some(HoverInfo {
            contents: info.contents,
            range: info.range.map(|r| HoverRange {
                start_line: r.start_line,
                start_col: r.start_col,
                end_line: r.end_line,
                end_col: r.end_col,
            }),
        })
    }
}

impl SymbolProvider for WixPlugin {
    fn symbols(&self, source: &str) -> Result<Vec<Symbol>, String> {
        let symbols = extract_symbols(source)?;

        fn convert_symbol(s: &wix_symbols::Symbol) -> Symbol {
            Symbol {
                name: s.name.clone(),
                kind: match s.kind {
                    wix_symbols::SymbolKind::File => SymbolKind::File,
                    wix_symbols::SymbolKind::Module => SymbolKind::Module,
                    wix_symbols::SymbolKind::Namespace => SymbolKind::Namespace,
                    wix_symbols::SymbolKind::Class => SymbolKind::Class,
                    wix_symbols::SymbolKind::Function => SymbolKind::Function,
                    wix_symbols::SymbolKind::Variable => SymbolKind::Variable,
                    wix_symbols::SymbolKind::Constant => SymbolKind::Constant,
                    wix_symbols::SymbolKind::String => SymbolKind::String,
                    wix_symbols::SymbolKind::Property => SymbolKind::Property,
                    wix_symbols::SymbolKind::Key => SymbolKind::Key,
                    wix_symbols::SymbolKind::Struct => SymbolKind::Struct,
                    wix_symbols::SymbolKind::Event => SymbolKind::Event,
                    wix_symbols::SymbolKind::Operator => SymbolKind::Operator,
                    wix_symbols::SymbolKind::TypeParameter => SymbolKind::TypeParameter,
                },
                detail: s.detail.clone(),
                range: SymbolRange {
                    start_line: s.range.start.line,
                    start_col: s.range.start.character,
                    end_line: s.range.end.line,
                    end_col: s.range.end.character,
                },
                selection_range: SymbolRange {
                    start_line: s.selection_range.start.line,
                    start_col: s.selection_range.start.character,
                    end_line: s.selection_range.end.line,
                    end_col: s.selection_range.end.character,
                },
                children: s.children.iter().map(convert_symbol).collect(),
            }
        }

        Ok(symbols.iter().map(convert_symbol).collect())
    }
}

impl DiagnosticProvider for WixPlugin {
    fn diagnose(&self, _source: &str, _path: &Path) -> Vec<Diagnostic> {
        // TODO: Integrate wix-analyzer for diagnostics
        Vec::new()
    }
}

impl FormatProvider for WixPlugin {
    fn format(&self, source: &str) -> Result<String, String> {
        self.formatter.format(source).map_err(|e| e.to_string())
    }
}

// Implement FullPluginDyn for dynamic dispatch
impl FullPluginDyn for WixPlugin {
    fn as_language(&self) -> &dyn LanguagePlugin {
        self
    }

    fn as_completion(&self) -> &dyn CompletionProvider {
        self
    }

    fn as_hover(&self) -> &dyn HoverProvider {
        self
    }

    fn as_symbol(&self) -> &dyn SymbolProvider {
        self
    }

    fn as_diagnostic(&self) -> &dyn DiagnosticProvider {
        self
    }

    fn as_format(&self) -> &dyn FormatProvider {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wix_plugin_new() {
        let plugin = WixPlugin::new();
        assert!(!plugin.is_initialized());
        assert_eq!(plugin.name(), "wix");
    }

    #[test]
    fn test_wix_plugin_file_extensions() {
        let plugin = WixPlugin::new();
        let exts = plugin.file_extensions();
        assert!(exts.contains(&".wxs"));
        assert!(exts.contains(&".wxi"));
        assert!(exts.contains(&".wxl"));
    }

    #[test]
    fn test_wix_plugin_trigger_characters() {
        let plugin = WixPlugin::new();
        let chars = plugin.trigger_characters();
        assert!(chars.contains(&'<'));
        assert!(chars.contains(&' '));
        assert!(chars.contains(&'"'));
        assert!(chars.contains(&'='));
    }

    #[test]
    fn test_symbols_without_init() {
        let plugin = WixPlugin::new();
        let result = plugin.symbols("<Wix><Component Id=\"Test\" /></Wix>");
        assert!(result.is_ok());
        let symbols = result.unwrap();
        assert_eq!(symbols.len(), 1);
        assert_eq!(symbols[0].name, "Test");
    }

    #[test]
    fn test_completion_without_init() {
        let plugin = WixPlugin::new();
        let completions = plugin.complete("<Wix>", 1, 5);
        assert!(completions.is_empty());
    }

    #[test]
    fn test_hover_without_init() {
        let plugin = WixPlugin::new();
        let hover = plugin.hover("<Component Id=\"Test\" />", 1, 3);
        assert!(hover.is_none());
    }

    #[test]
    fn test_format_without_init() {
        let plugin = WixPlugin::new();
        let result = plugin.format("<Wix><Component Id=\"Test\"/></Wix>");
        assert!(result.is_ok());
    }

    #[test]
    fn test_diagnose_without_init() {
        let plugin = WixPlugin::new();
        let diagnostics = plugin.diagnose("<Wix />", Path::new("test.wxs"));
        assert!(diagnostics.is_empty());
    }
}
