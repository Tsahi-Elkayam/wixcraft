//! Types for document symbols

use serde::Serialize;

/// Symbol kind (maps to LSP SymbolKind values)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "PascalCase")]
pub enum SymbolKind {
    /// File (Binary, File elements)
    File = 1,
    /// Module (Fragment, Package)
    Module = 2,
    /// Namespace (Wix root)
    Namespace = 3,
    /// Class (unused)
    Class = 5,
    /// Function (CustomAction)
    Function = 12,
    /// Variable (unused)
    Variable = 13,
    /// Constant (unused)
    Constant = 14,
    /// String (unused)
    String = 15,
    /// Property (Property element)
    Property = 7,
    /// Key (RegistryKey, RegistryValue)
    Key = 20,
    /// Struct (Component, ComponentGroup)
    Struct = 23,
    /// Event (unused)
    Event = 24,
    /// Operator (unused)
    Operator = 25,
    /// TypeParameter (Feature)
    TypeParameter = 26,
}

impl SymbolKind {
    /// Get display name for the symbol kind
    pub fn display_name(&self) -> &'static str {
        match self {
            SymbolKind::File => "File",
            SymbolKind::Module => "Module",
            SymbolKind::Namespace => "Namespace",
            SymbolKind::Class => "Class",
            SymbolKind::Function => "CustomAction",
            SymbolKind::Variable => "Variable",
            SymbolKind::Constant => "Constant",
            SymbolKind::String => "String",
            SymbolKind::Property => "Property",
            SymbolKind::Key => "Registry",
            SymbolKind::Struct => "Component",
            SymbolKind::Event => "Event",
            SymbolKind::Operator => "Operator",
            SymbolKind::TypeParameter => "Feature",
        }
    }

    /// Get LSP numeric value
    pub fn lsp_value(&self) -> u32 {
        *self as u32
    }
}

/// Position in source (1-based line and column)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct Position {
    pub line: u32,
    pub character: u32,
}

impl Position {
    pub fn new(line: u32, character: u32) -> Self {
        Self { line, character }
    }
}

/// Range in source document
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct Range {
    pub start: Position,
    pub end: Position,
}

impl Range {
    pub fn new(start_line: u32, start_col: u32, end_line: u32, end_col: u32) -> Self {
        Self {
            start: Position::new(start_line, start_col),
            end: Position::new(end_line, end_col),
        }
    }

    /// Create from roxmltree text position
    pub fn from_text_pos(source: &str, start_offset: usize, end_offset: usize) -> Self {
        let start = offset_to_position(source, start_offset);
        let end = offset_to_position(source, end_offset);
        Self { start, end }
    }
}

/// Convert byte offset to line/column position (1-based)
fn offset_to_position(source: &str, offset: usize) -> Position {
    let mut line = 1u32;
    let mut col = 1u32;

    for (i, ch) in source.char_indices() {
        if i >= offset {
            break;
        }
        if ch == '\n' {
            line += 1;
            col = 1;
        } else {
            col += 1;
        }
    }

    Position::new(line, col)
}

/// A document symbol
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Symbol {
    /// Display name (typically the Id attribute)
    pub name: String,

    /// Symbol kind
    pub kind: SymbolKind,

    /// Additional detail (e.g., Name attribute, type info)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,

    /// Full range of the element
    pub range: Range,

    /// Range of the name/identifier (for highlighting)
    pub selection_range: Range,

    /// Child symbols (for hierarchical view)
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub children: Vec<Symbol>,
}

impl Symbol {
    /// Create a new symbol
    pub fn new(name: String, kind: SymbolKind, range: Range, selection_range: Range) -> Self {
        Self {
            name,
            kind,
            detail: None,
            range,
            selection_range,
            children: Vec::new(),
        }
    }

    /// Set detail
    pub fn with_detail(mut self, detail: String) -> Self {
        self.detail = Some(detail);
        self
    }

    /// Add a child symbol
    pub fn add_child(&mut self, child: Symbol) {
        self.children.push(child);
    }

    /// Format for text output
    pub fn format_text(&self, indent: usize) -> String {
        let prefix = "  ".repeat(indent);
        let detail_str = self
            .detail
            .as_ref()
            .map(|d| format!(" ({})", d))
            .unwrap_or_default();

        let range_str = format!(
            "[{}:{}-{}:{}]",
            self.range.start.line,
            self.range.start.character,
            self.range.end.line,
            self.range.end.character
        );

        let mut result = format!(
            "{}{}: {}{} {}\n",
            prefix,
            self.kind.display_name(),
            self.name,
            detail_str,
            range_str
        );

        for child in &self.children {
            result.push_str(&child.format_text(indent + 1));
        }

        result
    }

    /// Flatten symbol tree to list
    pub fn flatten(&self) -> Vec<&Symbol> {
        let mut result = vec![self];
        for child in &self.children {
            result.extend(child.flatten());
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_symbol_kind_display() {
        // Test all SymbolKind variants for complete coverage
        assert_eq!(SymbolKind::File.display_name(), "File");
        assert_eq!(SymbolKind::Module.display_name(), "Module");
        assert_eq!(SymbolKind::Namespace.display_name(), "Namespace");
        assert_eq!(SymbolKind::Class.display_name(), "Class");
        assert_eq!(SymbolKind::Function.display_name(), "CustomAction");
        assert_eq!(SymbolKind::Variable.display_name(), "Variable");
        assert_eq!(SymbolKind::Constant.display_name(), "Constant");
        assert_eq!(SymbolKind::String.display_name(), "String");
        assert_eq!(SymbolKind::Property.display_name(), "Property");
        assert_eq!(SymbolKind::Key.display_name(), "Registry");
        assert_eq!(SymbolKind::Struct.display_name(), "Component");
        assert_eq!(SymbolKind::Event.display_name(), "Event");
        assert_eq!(SymbolKind::Operator.display_name(), "Operator");
        assert_eq!(SymbolKind::TypeParameter.display_name(), "Feature");
    }

    #[test]
    fn test_symbol_kind_lsp_value() {
        // Test lsp_value returns the enum variant as u32
        assert_eq!(SymbolKind::File.lsp_value(), 1);
        assert_eq!(SymbolKind::Module.lsp_value(), 2);
        assert_eq!(SymbolKind::Struct.lsp_value(), 23);
    }

    #[test]
    fn test_position_creation() {
        let pos = Position::new(5, 10);
        assert_eq!(pos.line, 5);
        assert_eq!(pos.character, 10);
    }

    #[test]
    fn test_range_creation() {
        let range = Range::new(1, 1, 5, 10);
        assert_eq!(range.start.line, 1);
        assert_eq!(range.end.character, 10);
    }

    #[test]
    fn test_offset_to_position() {
        let source = "abc\ndef\nghi";
        assert_eq!(offset_to_position(source, 0), Position::new(1, 1));
        assert_eq!(offset_to_position(source, 2), Position::new(1, 3));
        assert_eq!(offset_to_position(source, 4), Position::new(2, 1));
        assert_eq!(offset_to_position(source, 8), Position::new(3, 1));
    }

    #[test]
    fn test_symbol_with_detail() {
        let range = Range::new(1, 1, 1, 10);
        let symbol = Symbol::new("Test".to_string(), SymbolKind::Struct, range, range)
            .with_detail("Detail".to_string());

        assert_eq!(symbol.detail, Some("Detail".to_string()));
    }

    #[test]
    fn test_symbol_add_child() {
        let range = Range::new(1, 1, 10, 1);
        let mut parent = Symbol::new("Parent".to_string(), SymbolKind::Module, range, range);

        let child_range = Range::new(2, 1, 5, 1);
        let child = Symbol::new("Child".to_string(), SymbolKind::Struct, child_range, child_range);

        parent.add_child(child);
        assert_eq!(parent.children.len(), 1);
    }

    #[test]
    fn test_symbol_flatten() {
        let range = Range::new(1, 1, 10, 1);
        let mut parent = Symbol::new("Parent".to_string(), SymbolKind::Module, range, range);

        let child_range = Range::new(2, 1, 5, 1);
        let child = Symbol::new("Child".to_string(), SymbolKind::Struct, child_range, child_range);

        parent.add_child(child);

        let flat = parent.flatten();
        assert_eq!(flat.len(), 2);
    }

    #[test]
    fn test_symbol_format_text() {
        let range = Range::new(1, 1, 1, 20);
        let symbol = Symbol::new("MainComponent".to_string(), SymbolKind::Struct, range, range);

        let text = symbol.format_text(0);
        assert!(text.contains("Component: MainComponent"));
        assert!(text.contains("[1:1-1:20]"));
    }

    #[test]
    fn test_symbol_format_text_with_detail() {
        let range = Range::new(1, 1, 1, 30);
        let symbol =
            Symbol::new("INSTALLFOLDER".to_string(), SymbolKind::Namespace, range, range)
                .with_detail("MyApp".to_string());

        let text = symbol.format_text(0);
        assert!(text.contains("(MyApp)"));
    }

    #[test]
    fn test_range_from_text_pos() {
        let source = "line1\nline2\nline3";
        let range = Range::from_text_pos(source, 0, 5);
        assert_eq!(range.start.line, 1);
        assert_eq!(range.start.character, 1);
        assert_eq!(range.end.line, 1);
        assert_eq!(range.end.character, 6);
    }

    #[test]
    fn test_symbol_json_serialization() {
        let range = Range::new(1, 1, 1, 10);
        let symbol = Symbol::new("Test".to_string(), SymbolKind::Struct, range, range);

        let json = serde_json::to_string(&symbol).unwrap();
        assert!(json.contains("\"name\":\"Test\""));
        assert!(json.contains("\"kind\":\"Struct\""));
    }
}
