//! # wix-references
//!
//! Go to Definition and Find References for WiX files.
//!
//! This library provides cross-file symbol indexing and reference tracking
//! for WiX XML files. It's designed for use in LSP servers, IDE plugins,
//! and code analysis tools.
//!
//! ## Features
//!
//! - Cross-file symbol indexing
//! - Find definition by symbol name
//! - Find all references to a symbol
//! - Go-to-definition from cursor position
//! - Support for 14+ WiX symbol kinds
//! - Incremental file updates
//!
//! ## Example
//!
//! ```
//! use wix_references::{ReferenceIndex, Location};
//!
//! let mut index = ReferenceIndex::new();
//! index.add_file("product.wxs", r#"<Component Id="MainComp" />"#);
//! index.add_file("feature.wxs", r#"<ComponentRef Id="MainComp" />"#);
//!
//! // Find definition
//! if let Some(def) = index.find_definition("MainComp") {
//!     println!("Defined at {}:{}", def.location.file, def.location.line);
//! }
//!
//! // Find references
//! let refs = index.find_references("MainComp");
//! println!("Found {} references", refs.len());
//! ```

use roxmltree::{Document, Node};
use serde::Serialize;
use std::collections::HashMap;
use thiserror::Error;

/// Error types for reference operations.
#[derive(Error, Debug)]
pub enum ReferenceError {
    /// Failed to parse XML content.
    #[error("Failed to parse XML: {0}")]
    ParseError(String),

    /// File not found in the index.
    #[error("File not found in index: {0}")]
    FileNotFound(String),
}

/// Position in source code (1-based line and column).
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

/// Range in source code (start and end positions).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct Range {
    pub start: Position,
    pub end: Position,
}

impl Range {
    pub fn new(start: Position, end: Position) -> Self {
        Self { start, end }
    }

    /// Create a zero-width range at a single point.
    pub fn point(line: u32, character: u32) -> Self {
        let pos = Position::new(line, character);
        Self { start: pos, end: pos }
    }

    /// Check if a position is within this range.
    pub fn contains(&self, pos: Position) -> bool {
        if pos.line < self.start.line || pos.line > self.end.line {
            return false;
        }
        if pos.line == self.start.line && pos.character < self.start.character {
            return false;
        }
        if pos.line == self.end.line && pos.character > self.end.character {
            return false;
        }
        true
    }
}

/// Location of a symbol or reference in a file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Location {
    pub file: String,
    pub line: u32,
    pub column: u32,
    pub range: Range,
}

impl Location {
    pub fn new(file: impl Into<String>, line: u32, column: u32, range: Range) -> Self {
        Self {
            file: file.into(),
            line,
            column,
            range,
        }
    }
}

/// Type of symbol entry (definition vs reference).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum SymbolType {
    /// A definition (e.g., `<Component Id="X">`)
    Definition,
    /// A reference (e.g., `<ComponentRef Id="X">`)
    Reference,
}

/// Kind of WiX symbol.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
pub enum SymbolKind {
    Component,
    ComponentGroup,
    Feature,
    Directory,
    Property,
    CustomAction,
    Binary,
    File,
    Registry,
    Fragment,
    Package,
    UI,
    Dialog,
    Control,
    Other,
}

impl SymbolKind {
    /// Get symbol kind from element name.
    pub fn from_element(name: &str) -> Self {
        match name {
            "Component" | "ComponentRef" => SymbolKind::Component,
            "ComponentGroup" | "ComponentGroupRef" => SymbolKind::ComponentGroup,
            "Feature" | "FeatureRef" | "FeatureGroup" | "FeatureGroupRef" => SymbolKind::Feature,
            "Directory" | "DirectoryRef" | "StandardDirectory" => SymbolKind::Directory,
            "Property" | "PropertyRef" => SymbolKind::Property,
            "CustomAction" | "CustomActionRef" => SymbolKind::CustomAction,
            "Binary" | "BinaryRef" => SymbolKind::Binary,
            "File" | "FileRef" => SymbolKind::File,
            "RegistryKey" | "RegistryValue" | "RegistrySearch" => SymbolKind::Registry,
            "Fragment" | "FragmentRef" => SymbolKind::Fragment,
            "Package" => SymbolKind::Package,
            "UI" | "UIRef" => SymbolKind::UI,
            "Dialog" | "DialogRef" => SymbolKind::Dialog,
            "Control" | "ControlRef" => SymbolKind::Control,
            _ => SymbolKind::Other,
        }
    }

    /// Check if element name is a reference type (ends with "Ref").
    pub fn is_reference_element(name: &str) -> bool {
        name.ends_with("Ref")
    }
}

/// A symbol entry (definition or reference).
#[derive(Debug, Clone, Serialize)]
pub struct SymbolEntry {
    /// Symbol name (the Id attribute value).
    pub name: String,
    /// Kind of symbol.
    pub kind: SymbolKind,
    /// Whether this is a definition or reference.
    pub symbol_type: SymbolType,
    /// Location in source.
    pub location: Location,
    /// Element name (e.g., "Component", "ComponentRef").
    pub element: String,
}

/// Index of all symbols and references across files.
///
/// The index maintains mappings from symbol names to their definitions
/// and references, enabling fast lookups for go-to-definition and
/// find-references operations.
#[derive(Debug, Default)]
pub struct ReferenceIndex {
    /// All definitions indexed by name.
    definitions: HashMap<String, Vec<SymbolEntry>>,
    /// All references indexed by name.
    references: HashMap<String, Vec<SymbolEntry>>,
    /// File contents (for position lookups).
    files: HashMap<String, String>,
}

impl ReferenceIndex {
    /// Create a new empty index.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a file to the index.
    ///
    /// Parses the XML content and extracts all symbol definitions and references.
    pub fn add_file(&mut self, path: &str, content: &str) -> Result<(), ReferenceError> {
        // Store file content
        self.files.insert(path.to_string(), content.to_string());

        // Parse XML
        let doc = Document::parse(content)
            .map_err(|e| ReferenceError::ParseError(e.to_string()))?;

        // Extract symbols
        self.extract_symbols(path, content, doc.root_element());
        Ok(())
    }

    /// Remove a file from the index.
    ///
    /// Removes all definitions and references from the specified file.
    pub fn remove_file(&mut self, path: &str) {
        self.files.remove(path);

        // Remove all entries for this file
        for entries in self.definitions.values_mut() {
            entries.retain(|e| e.location.file != path);
        }
        for entries in self.references.values_mut() {
            entries.retain(|e| e.location.file != path);
        }

        // Clean up empty entries
        self.definitions.retain(|_, v| !v.is_empty());
        self.references.retain(|_, v| !v.is_empty());
    }

    /// Update a file in the index.
    ///
    /// Removes the old content and re-indexes the new content.
    pub fn update_file(&mut self, path: &str, content: &str) -> Result<(), ReferenceError> {
        self.remove_file(path);
        self.add_file(path, content)
    }

    /// Find the definition of a symbol by name.
    ///
    /// Returns the first definition found, or `None` if not found.
    pub fn find_definition(&self, name: &str) -> Option<&SymbolEntry> {
        self.definitions.get(name).and_then(|v| v.first())
    }

    /// Find all definitions of a symbol (in case of duplicates).
    pub fn find_all_definitions(&self, name: &str) -> Vec<&SymbolEntry> {
        self.definitions
            .get(name)
            .map(|v| v.iter().collect())
            .unwrap_or_default()
    }

    /// Find all references to a symbol.
    pub fn find_references(&self, name: &str) -> Vec<&SymbolEntry> {
        self.references
            .get(name)
            .map(|v| v.iter().collect())
            .unwrap_or_default()
    }

    /// Find all usages of a symbol (definition + references).
    pub fn find_all_usages(&self, name: &str) -> Vec<&SymbolEntry> {
        let mut result = Vec::new();
        if let Some(defs) = self.definitions.get(name) {
            result.extend(defs.iter());
        }
        if let Some(refs) = self.references.get(name) {
            result.extend(refs.iter());
        }
        result
    }

    /// Get the symbol at a specific position in a file.
    ///
    /// Used for hover and go-to-definition from cursor position.
    pub fn symbol_at_position(&self, file: &str, line: u32, column: u32) -> Option<&SymbolEntry> {
        let pos = Position::new(line, column);

        // Check definitions
        for entries in self.definitions.values() {
            for entry in entries {
                if entry.location.file == file && entry.location.range.contains(pos) {
                    return Some(entry);
                }
            }
        }

        // Check references
        for entries in self.references.values() {
            for entry in entries {
                if entry.location.file == file && entry.location.range.contains(pos) {
                    return Some(entry);
                }
            }
        }

        None
    }

    /// Go to definition from a position.
    ///
    /// Finds the symbol at the given position, then returns its definition.
    pub fn go_to_definition(&self, file: &str, line: u32, column: u32) -> Option<&SymbolEntry> {
        let symbol = self.symbol_at_position(file, line, column)?;
        self.find_definition(&symbol.name)
    }

    /// Get all definitions in the index.
    pub fn all_definitions(&self) -> Vec<&SymbolEntry> {
        self.definitions.values().flatten().collect()
    }

    /// Get all references in the index.
    pub fn all_references(&self) -> Vec<&SymbolEntry> {
        self.references.values().flatten().collect()
    }

    /// Get index statistics.
    pub fn stats(&self) -> IndexStats {
        IndexStats {
            file_count: self.files.len(),
            definition_count: self.definitions.values().map(|v| v.len()).sum(),
            reference_count: self.references.values().map(|v| v.len()).sum(),
            unique_symbols: self.definitions.len(),
        }
    }

    /// Extract symbols from an XML node recursively.
    fn extract_symbols(&mut self, path: &str, content: &str, node: Node) {
        let element_name = node.tag_name().name();

        // Check for Id attribute (the main identifier)
        if let Some(id) = node.attribute("Id") {
            let kind = SymbolKind::from_element(element_name);
            let is_ref = SymbolKind::is_reference_element(element_name);

            let pos = self.offset_to_position(content, node.range().start);
            let end_pos = self.offset_to_position(content, node.range().end);
            let range = Range::new(pos, end_pos);

            let entry = SymbolEntry {
                name: id.to_string(),
                kind,
                symbol_type: if is_ref {
                    SymbolType::Reference
                } else {
                    SymbolType::Definition
                },
                location: Location::new(path, pos.line, pos.character, range),
                element: element_name.to_string(),
            };

            if is_ref {
                self.references
                    .entry(id.to_string())
                    .or_default()
                    .push(entry);
            } else {
                self.definitions
                    .entry(id.to_string())
                    .or_default()
                    .push(entry);
            }
        }

        // Recurse into children
        for child in node.children().filter(|n| n.is_element()) {
            self.extract_symbols(path, content, child);
        }
    }

    /// Convert byte offset to line/column position.
    fn offset_to_position(&self, content: &str, offset: usize) -> Position {
        let mut line = 1u32;
        let mut col = 1u32;

        for (i, ch) in content.char_indices() {
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
}

/// Index statistics.
#[derive(Debug, Clone, Serialize)]
pub struct IndexStats {
    pub file_count: usize,
    pub definition_count: usize,
    pub reference_count: usize,
    pub unique_symbols: usize,
}

/// Result of a go-to-definition or find-references query.
#[derive(Debug, Clone, Serialize)]
pub struct QueryResult {
    /// The symbol name queried.
    pub symbol: String,
    /// The definition location (if found).
    pub definition: Option<Location>,
    /// All reference locations.
    pub references: Vec<Location>,
}

impl QueryResult {
    pub fn new(symbol: impl Into<String>) -> Self {
        Self {
            symbol: symbol.into(),
            definition: None,
            references: Vec::new(),
        }
    }

    pub fn with_definition(mut self, location: Location) -> Self {
        self.definition = Some(location);
        self
    }

    pub fn with_references(mut self, references: Vec<Location>) -> Self {
        self.references = references;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_file() {
        let mut index = ReferenceIndex::new();
        let content = r#"<Wix><Component Id="MainComp" Guid="*" /></Wix>"#;
        index.add_file("test.wxs", content).unwrap();

        assert_eq!(index.stats().file_count, 1);
        assert_eq!(index.stats().definition_count, 1);
    }

    #[test]
    fn test_find_definition() {
        let mut index = ReferenceIndex::new();
        let content = r#"<Wix><Component Id="MainComp" Guid="*" /></Wix>"#;
        index.add_file("test.wxs", content).unwrap();

        let def = index.find_definition("MainComp").unwrap();
        assert_eq!(def.name, "MainComp");
        assert_eq!(def.kind, SymbolKind::Component);
        assert_eq!(def.symbol_type, SymbolType::Definition);
    }

    #[test]
    fn test_find_references() {
        let mut index = ReferenceIndex::new();
        index
            .add_file("product.wxs", r#"<Wix><Component Id="MainComp" /></Wix>"#)
            .unwrap();
        index
            .add_file(
                "feature.wxs",
                r#"<Wix><Feature Id="Main"><ComponentRef Id="MainComp" /></Feature></Wix>"#,
            )
            .unwrap();

        let refs = index.find_references("MainComp");
        assert_eq!(refs.len(), 1);
        assert_eq!(refs[0].location.file, "feature.wxs");
    }

    #[test]
    fn test_find_all_usages() {
        let mut index = ReferenceIndex::new();
        index
            .add_file("product.wxs", r#"<Wix><Component Id="MainComp" /></Wix>"#)
            .unwrap();
        index
            .add_file(
                "feature.wxs",
                r#"<Wix><ComponentRef Id="MainComp" /></Wix>"#,
            )
            .unwrap();

        let usages = index.find_all_usages("MainComp");
        assert_eq!(usages.len(), 2);
    }

    #[test]
    fn test_go_to_definition() {
        let mut index = ReferenceIndex::new();
        index
            .add_file("product.wxs", r#"<Wix><Component Id="MainComp" /></Wix>"#)
            .unwrap();
        index
            .add_file(
                "feature.wxs",
                r#"<Wix><ComponentRef Id="MainComp" /></Wix>"#,
            )
            .unwrap();

        // The ComponentRef is at line 1, column 6 (approximately)
        let def = index.go_to_definition("feature.wxs", 1, 20);
        assert!(def.is_some());
        assert_eq!(def.unwrap().location.file, "product.wxs");
    }

    #[test]
    fn test_remove_file() {
        let mut index = ReferenceIndex::new();
        index
            .add_file("test.wxs", r#"<Wix><Component Id="Comp1" /></Wix>"#)
            .unwrap();
        assert_eq!(index.stats().definition_count, 1);

        index.remove_file("test.wxs");
        assert_eq!(index.stats().definition_count, 0);
        assert_eq!(index.stats().file_count, 0);
    }

    #[test]
    fn test_update_file() {
        let mut index = ReferenceIndex::new();
        index
            .add_file("test.wxs", r#"<Wix><Component Id="Comp1" /></Wix>"#)
            .unwrap();

        assert!(index.find_definition("Comp1").is_some());
        assert!(index.find_definition("Comp2").is_none());

        index
            .update_file("test.wxs", r#"<Wix><Component Id="Comp2" /></Wix>"#)
            .unwrap();

        assert!(index.find_definition("Comp1").is_none());
        assert!(index.find_definition("Comp2").is_some());
    }

    #[test]
    fn test_multiple_definitions() {
        let mut index = ReferenceIndex::new();
        index
            .add_file("file1.wxs", r#"<Wix><Component Id="Shared" /></Wix>"#)
            .unwrap();
        index
            .add_file("file2.wxs", r#"<Wix><Component Id="Shared" /></Wix>"#)
            .unwrap();

        let defs = index.find_all_definitions("Shared");
        assert_eq!(defs.len(), 2);
    }

    #[test]
    fn test_symbol_kinds() {
        let mut index = ReferenceIndex::new();
        index
            .add_file(
                "test.wxs",
                r#"<Wix>
                <Directory Id="INSTALLFOLDER" />
                <Feature Id="MainFeature" />
                <Property Id="MYPROP" />
                <CustomAction Id="DoSomething" />
            </Wix>"#,
            )
            .unwrap();

        assert_eq!(
            index.find_definition("INSTALLFOLDER").unwrap().kind,
            SymbolKind::Directory
        );
        assert_eq!(
            index.find_definition("MainFeature").unwrap().kind,
            SymbolKind::Feature
        );
        assert_eq!(
            index.find_definition("MYPROP").unwrap().kind,
            SymbolKind::Property
        );
        assert_eq!(
            index.find_definition("DoSomething").unwrap().kind,
            SymbolKind::CustomAction
        );
    }

    #[test]
    fn test_reference_types() {
        assert!(!SymbolKind::is_reference_element("Component"));
        assert!(SymbolKind::is_reference_element("ComponentRef"));
        assert!(!SymbolKind::is_reference_element("Feature"));
        assert!(SymbolKind::is_reference_element("FeatureRef"));
    }

    #[test]
    fn test_position_contains() {
        let range = Range::new(Position::new(5, 10), Position::new(5, 20));

        assert!(range.contains(Position::new(5, 10)));
        assert!(range.contains(Position::new(5, 15)));
        assert!(range.contains(Position::new(5, 20)));
        assert!(!range.contains(Position::new(5, 9)));
        assert!(!range.contains(Position::new(5, 21)));
        assert!(!range.contains(Position::new(4, 15)));
        assert!(!range.contains(Position::new(6, 15)));
    }

    #[test]
    fn test_multiline_range() {
        let range = Range::new(Position::new(5, 10), Position::new(8, 20));

        assert!(range.contains(Position::new(5, 10)));
        assert!(range.contains(Position::new(6, 1)));
        assert!(range.contains(Position::new(7, 50)));
        assert!(range.contains(Position::new(8, 20)));
        assert!(!range.contains(Position::new(5, 9)));
        assert!(!range.contains(Position::new(8, 21)));
    }

    #[test]
    fn test_all_definitions() {
        let mut index = ReferenceIndex::new();
        index
            .add_file(
                "test.wxs",
                r#"<Wix><Component Id="C1" /><Component Id="C2" /></Wix>"#,
            )
            .unwrap();

        let defs = index.all_definitions();
        assert_eq!(defs.len(), 2);
    }

    #[test]
    fn test_stats() {
        let mut index = ReferenceIndex::new();
        index
            .add_file("a.wxs", r#"<Wix><Component Id="C1" /><Component Id="C2" /></Wix>"#)
            .unwrap();
        index
            .add_file("b.wxs", r#"<Wix><ComponentRef Id="C1" /></Wix>"#)
            .unwrap();

        let stats = index.stats();
        assert_eq!(stats.file_count, 2);
        assert_eq!(stats.definition_count, 2);
        assert_eq!(stats.reference_count, 1);
        assert_eq!(stats.unique_symbols, 2);
    }

    #[test]
    fn test_query_result() {
        let loc = Location::new("test.wxs", 1, 1, Range::point(1, 1));
        let result = QueryResult::new("TestSymbol")
            .with_definition(loc.clone())
            .with_references(vec![loc]);

        assert_eq!(result.symbol, "TestSymbol");
        assert!(result.definition.is_some());
        assert_eq!(result.references.len(), 1);
    }

    #[test]
    fn test_parse_error() {
        let mut index = ReferenceIndex::new();
        let result = index.add_file("bad.wxs", "not valid xml <>");
        assert!(result.is_err());
    }
}
