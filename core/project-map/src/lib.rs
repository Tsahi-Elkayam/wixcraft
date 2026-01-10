//! Project indexer with symbol graph and relevance detection
//!
//! Provides project structure analysis, symbol tracking, and relationship mapping.
//!
//! # Example
//!
//! ```
//! use project_map::{ProjectMap, Symbol, SymbolKind};
//!
//! let mut map = ProjectMap::new();
//! map.add_file("src/main.rs", "fn main() {}");
//!
//! // Query symbols
//! let symbols = map.symbols_in_file("src/main.rs");
//! ```

pub use code_detector::Language;

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

/// Kind of symbol (function, class, variable, etc.)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SymbolKind {
    /// Function or method
    Function,
    /// Class or struct
    Class,
    /// Interface or trait
    Interface,
    /// Variable or constant
    Variable,
    /// Type alias
    TypeAlias,
    /// Module or namespace
    Module,
    /// Enum type
    Enum,
    /// Enum variant
    EnumVariant,
    /// Property or field
    Property,
    /// Macro
    Macro,
    /// XML/HTML Element
    Element,
    /// XML/HTML Attribute
    Attribute,
    /// Import statement
    Import,
    /// Export statement
    Export,
}

/// Visibility of a symbol
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum Visibility {
    /// Public (accessible from anywhere)
    Public,
    /// Private (accessible only within the same scope)
    #[default]
    Private,
    /// Protected (accessible within inheritance hierarchy)
    Protected,
    /// Package/module-level visibility
    Internal,
}

/// Location in source code
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Location {
    /// File path
    pub file: PathBuf,
    /// Start line (1-indexed)
    pub start_line: u32,
    /// Start column (1-indexed)
    pub start_column: u32,
    /// End line (1-indexed)
    pub end_line: u32,
    /// End column (1-indexed)
    pub end_column: u32,
}

impl Location {
    pub fn new(file: impl Into<PathBuf>, start_line: u32, start_column: u32, end_line: u32, end_column: u32) -> Self {
        Self {
            file: file.into(),
            start_line,
            start_column,
            end_line,
            end_column,
        }
    }

    pub fn point(file: impl Into<PathBuf>, line: u32, column: u32) -> Self {
        Self::new(file, line, column, line, column)
    }
}

/// Unique identifier for a symbol
pub type SymbolId = u64;

/// A symbol in the project (function, class, variable, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Symbol {
    /// Unique identifier
    pub id: SymbolId,
    /// Symbol name
    pub name: String,
    /// Fully qualified name (e.g., "module::Class::method")
    pub qualified_name: String,
    /// Kind of symbol
    pub kind: SymbolKind,
    /// Location in source code
    pub location: Location,
    /// Visibility
    pub visibility: Visibility,
    /// Documentation string if available
    pub documentation: Option<String>,
    /// Type signature if available
    pub type_signature: Option<String>,
    /// Parent symbol ID (for nested symbols)
    pub parent: Option<SymbolId>,
    /// Detected language
    pub language: Language,
}

impl Symbol {
    pub fn new(
        id: SymbolId,
        name: impl Into<String>,
        kind: SymbolKind,
        location: Location,
    ) -> Self {
        let name = name.into();
        Self {
            id,
            qualified_name: name.clone(),
            name,
            kind,
            location,
            visibility: Visibility::default(),
            documentation: None,
            type_signature: None,
            parent: None,
            language: Language::Unknown,
        }
    }

    pub fn with_qualified_name(mut self, qualified_name: impl Into<String>) -> Self {
        self.qualified_name = qualified_name.into();
        self
    }

    pub fn with_visibility(mut self, visibility: Visibility) -> Self {
        self.visibility = visibility;
        self
    }

    pub fn with_documentation(mut self, doc: impl Into<String>) -> Self {
        self.documentation = Some(doc.into());
        self
    }

    pub fn with_type_signature(mut self, sig: impl Into<String>) -> Self {
        self.type_signature = Some(sig.into());
        self
    }

    pub fn with_parent(mut self, parent: SymbolId) -> Self {
        self.parent = Some(parent);
        self
    }

    pub fn with_language(mut self, language: Language) -> Self {
        self.language = language;
        self
    }
}

/// Type of relationship between symbols
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RelationKind {
    /// Symbol A calls/invokes symbol B
    Calls,
    /// Symbol A references symbol B
    References,
    /// Symbol A imports symbol B
    Imports,
    /// Symbol A extends/inherits from symbol B
    Extends,
    /// Symbol A implements interface B
    Implements,
    /// Symbol A contains symbol B (parent-child)
    Contains,
    /// Symbol A uses type B
    UsesType,
    /// Symbol A overrides symbol B
    Overrides,
}

/// A relationship between two symbols
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relation {
    /// Source symbol
    pub from: SymbolId,
    /// Target symbol
    pub to: SymbolId,
    /// Type of relationship
    pub kind: RelationKind,
    /// Location where the relationship is established
    pub location: Option<Location>,
}

impl Relation {
    pub fn new(from: SymbolId, to: SymbolId, kind: RelationKind) -> Self {
        Self {
            from,
            to,
            kind,
            location: None,
        }
    }

    pub fn with_location(mut self, location: Location) -> Self {
        self.location = Some(location);
        self
    }
}

/// Information about an indexed file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileInfo {
    /// File path
    pub path: PathBuf,
    /// Detected language
    pub language: Language,
    /// Symbols defined in this file
    pub symbols: Vec<SymbolId>,
    /// Files this file imports/depends on
    pub imports: Vec<PathBuf>,
    /// Files that import/depend on this file
    pub imported_by: Vec<PathBuf>,
    /// Last modified time (if tracked)
    pub modified: Option<u64>,
    /// Content hash for change detection
    pub content_hash: Option<u64>,
}

impl FileInfo {
    pub fn new(path: impl Into<PathBuf>, language: Language) -> Self {
        Self {
            path: path.into(),
            language,
            symbols: Vec::new(),
            imports: Vec::new(),
            imported_by: Vec::new(),
            modified: None,
            content_hash: None,
        }
    }
}

/// Project map containing symbols, relationships, and file information
#[derive(Debug, Default)]
pub struct ProjectMap {
    /// All symbols by ID
    symbols: HashMap<SymbolId, Symbol>,
    /// All relations
    relations: Vec<Relation>,
    /// File information
    files: HashMap<PathBuf, FileInfo>,
    /// Symbol name to IDs index (for quick lookup)
    name_index: HashMap<String, Vec<SymbolId>>,
    /// Next symbol ID
    next_id: SymbolId,
}

impl ProjectMap {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a file to the project map
    pub fn add_file(&mut self, path: impl Into<PathBuf>, content: &str) -> &FileInfo {
        let path = path.into();
        let path_str = path.to_string_lossy();
        let language = code_detector::detect(&path_str, content);

        let file_info = FileInfo::new(path.clone(), language);
        self.files.insert(path.clone(), file_info);
        self.files.get(&path).unwrap()
    }

    /// Get file information
    pub fn get_file(&self, path: impl AsRef<Path>) -> Option<&FileInfo> {
        self.files.get(path.as_ref())
    }

    /// Get all indexed files
    pub fn files(&self) -> impl Iterator<Item = &FileInfo> {
        self.files.values()
    }

    /// Add a symbol to the project map
    pub fn add_symbol(&mut self, mut symbol: Symbol) -> SymbolId {
        let id = self.next_id;
        self.next_id += 1;
        symbol.id = id;

        // Update file info
        if let Some(file_info) = self.files.get_mut(&symbol.location.file) {
            file_info.symbols.push(id);
        }

        // Update name index
        self.name_index
            .entry(symbol.name.clone())
            .or_default()
            .push(id);

        self.symbols.insert(id, symbol);
        id
    }

    /// Get a symbol by ID
    pub fn get_symbol(&self, id: SymbolId) -> Option<&Symbol> {
        self.symbols.get(&id)
    }

    /// Get all symbols
    pub fn symbols(&self) -> impl Iterator<Item = &Symbol> {
        self.symbols.values()
    }

    /// Get symbols by name
    pub fn symbols_by_name(&self, name: &str) -> Vec<&Symbol> {
        self.name_index
            .get(name)
            .map(|ids| ids.iter().filter_map(|id| self.symbols.get(id)).collect())
            .unwrap_or_default()
    }

    /// Get symbols in a specific file
    pub fn symbols_in_file(&self, path: impl AsRef<Path>) -> Vec<&Symbol> {
        self.files
            .get(path.as_ref())
            .map(|info| {
                info.symbols
                    .iter()
                    .filter_map(|id| self.symbols.get(id))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get symbols of a specific kind
    pub fn symbols_by_kind(&self, kind: SymbolKind) -> Vec<&Symbol> {
        self.symbols
            .values()
            .filter(|s| s.kind == kind)
            .collect()
    }

    /// Add a relationship between symbols
    pub fn add_relation(&mut self, relation: Relation) {
        self.relations.push(relation);
    }

    /// Get all relations
    pub fn relations(&self) -> &[Relation] {
        &self.relations
    }

    /// Get relations from a symbol
    pub fn relations_from(&self, id: SymbolId) -> Vec<&Relation> {
        self.relations.iter().filter(|r| r.from == id).collect()
    }

    /// Get relations to a symbol
    pub fn relations_to(&self, id: SymbolId) -> Vec<&Relation> {
        self.relations.iter().filter(|r| r.to == id).collect()
    }

    /// Find symbols matching a query
    pub fn search(&self, query: &str) -> Vec<&Symbol> {
        let query_lower = query.to_lowercase();
        self.symbols
            .values()
            .filter(|s| {
                s.name.to_lowercase().contains(&query_lower)
                    || s.qualified_name.to_lowercase().contains(&query_lower)
            })
            .collect()
    }

    /// Get children of a symbol
    pub fn children(&self, parent_id: SymbolId) -> Vec<&Symbol> {
        self.symbols
            .values()
            .filter(|s| s.parent == Some(parent_id))
            .collect()
    }

    /// Get call graph for a function (what it calls)
    pub fn calls(&self, id: SymbolId) -> Vec<&Symbol> {
        self.relations
            .iter()
            .filter(|r| r.from == id && r.kind == RelationKind::Calls)
            .filter_map(|r| self.symbols.get(&r.to))
            .collect()
    }

    /// Get callers of a function (what calls it)
    pub fn callers(&self, id: SymbolId) -> Vec<&Symbol> {
        self.relations
            .iter()
            .filter(|r| r.to == id && r.kind == RelationKind::Calls)
            .filter_map(|r| self.symbols.get(&r.from))
            .collect()
    }

    /// Get symbols that use/reference a symbol
    pub fn references(&self, id: SymbolId) -> Vec<&Symbol> {
        self.relations
            .iter()
            .filter(|r| r.to == id && r.kind == RelationKind::References)
            .filter_map(|r| self.symbols.get(&r.from))
            .collect()
    }

    /// Find files relevant to a query (file path, symbol name, etc.)
    pub fn relevant_files(&self, query: &str) -> Vec<&FileInfo> {
        let query_lower = query.to_lowercase();

        // First check file paths
        let mut relevant: Vec<&FileInfo> = self.files
            .values()
            .filter(|f| f.path.to_string_lossy().to_lowercase().contains(&query_lower))
            .collect();

        // Then check symbols
        let symbol_files: HashSet<&PathBuf> = self.symbols
            .values()
            .filter(|s| {
                s.name.to_lowercase().contains(&query_lower)
                    || s.qualified_name.to_lowercase().contains(&query_lower)
            })
            .map(|s| &s.location.file)
            .collect();

        for path in symbol_files {
            if let Some(info) = self.files.get(path) {
                if !relevant.iter().any(|r| r.path == info.path) {
                    relevant.push(info);
                }
            }
        }

        relevant
    }

    /// Get statistics about the project
    pub fn stats(&self) -> ProjectStats {
        let mut kind_counts: HashMap<SymbolKind, usize> = HashMap::new();
        let mut language_counts: HashMap<Language, usize> = HashMap::new();
        let mut relation_counts: HashMap<RelationKind, usize> = HashMap::new();

        for symbol in self.symbols.values() {
            *kind_counts.entry(symbol.kind).or_default() += 1;
        }

        for file in self.files.values() {
            *language_counts.entry(file.language).or_default() += 1;
        }

        for relation in &self.relations {
            *relation_counts.entry(relation.kind).or_default() += 1;
        }

        ProjectStats {
            total_files: self.files.len(),
            total_symbols: self.symbols.len(),
            total_relations: self.relations.len(),
            symbols_by_kind: kind_counts,
            files_by_language: language_counts,
            relations_by_kind: relation_counts,
        }
    }

    /// Clear all data
    pub fn clear(&mut self) {
        self.symbols.clear();
        self.relations.clear();
        self.files.clear();
        self.name_index.clear();
        self.next_id = 0;
    }
}

/// Statistics about the project
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectStats {
    pub total_files: usize,
    pub total_symbols: usize,
    pub total_relations: usize,
    pub symbols_by_kind: HashMap<SymbolKind, usize>,
    pub files_by_language: HashMap<Language, usize>,
    pub relations_by_kind: HashMap<RelationKind, usize>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_file() {
        let mut map = ProjectMap::new();
        let info = map.add_file("src/main.rs", "fn main() {}");
        assert_eq!(info.path, PathBuf::from("src/main.rs"));
        assert_eq!(info.language, Language::Rust);
    }

    #[test]
    fn test_add_symbol() {
        let mut map = ProjectMap::new();
        map.add_file("src/main.rs", "fn main() {}");

        let loc = Location::new("src/main.rs", 1, 1, 1, 15);
        let symbol = Symbol::new(0, "main", SymbolKind::Function, loc)
            .with_visibility(Visibility::Public)
            .with_language(Language::Rust);

        let id = map.add_symbol(symbol);

        let retrieved = map.get_symbol(id).unwrap();
        assert_eq!(retrieved.name, "main");
        assert_eq!(retrieved.kind, SymbolKind::Function);
        assert_eq!(retrieved.visibility, Visibility::Public);
    }

    #[test]
    fn test_symbol_builder() {
        let loc = Location::new("test.rs", 1, 1, 1, 10);
        let symbol = Symbol::new(1, "test_fn", SymbolKind::Function, loc)
            .with_qualified_name("module::test_fn")
            .with_visibility(Visibility::Private)
            .with_documentation("A test function")
            .with_type_signature("fn() -> i32")
            .with_parent(0)
            .with_language(Language::Rust);

        assert_eq!(symbol.name, "test_fn");
        assert_eq!(symbol.qualified_name, "module::test_fn");
        assert_eq!(symbol.visibility, Visibility::Private);
        assert_eq!(symbol.documentation, Some("A test function".to_string()));
        assert_eq!(symbol.type_signature, Some("fn() -> i32".to_string()));
        assert_eq!(symbol.parent, Some(0));
        assert_eq!(symbol.language, Language::Rust);
    }

    #[test]
    fn test_symbols_by_name() {
        let mut map = ProjectMap::new();
        map.add_file("src/a.rs", "");
        map.add_file("src/b.rs", "");

        let loc1 = Location::new("src/a.rs", 1, 1, 1, 10);
        let loc2 = Location::new("src/b.rs", 1, 1, 1, 10);

        map.add_symbol(Symbol::new(0, "process", SymbolKind::Function, loc1));
        map.add_symbol(Symbol::new(0, "process", SymbolKind::Function, loc2));

        let found = map.symbols_by_name("process");
        assert_eq!(found.len(), 2);
    }

    #[test]
    fn test_symbols_in_file() {
        let mut map = ProjectMap::new();
        map.add_file("src/lib.rs", "");

        let loc1 = Location::new("src/lib.rs", 1, 1, 1, 10);
        let loc2 = Location::new("src/lib.rs", 5, 1, 5, 10);

        map.add_symbol(Symbol::new(0, "foo", SymbolKind::Function, loc1));
        map.add_symbol(Symbol::new(0, "bar", SymbolKind::Function, loc2));

        let symbols = map.symbols_in_file("src/lib.rs");
        assert_eq!(symbols.len(), 2);
    }

    #[test]
    fn test_symbols_by_kind() {
        let mut map = ProjectMap::new();
        map.add_file("src/lib.rs", "");

        let loc = Location::new("src/lib.rs", 1, 1, 1, 10);

        map.add_symbol(Symbol::new(0, "MyClass", SymbolKind::Class, loc.clone()));
        map.add_symbol(Symbol::new(0, "my_func", SymbolKind::Function, loc.clone()));
        map.add_symbol(Symbol::new(0, "other_func", SymbolKind::Function, loc));

        let classes = map.symbols_by_kind(SymbolKind::Class);
        let functions = map.symbols_by_kind(SymbolKind::Function);

        assert_eq!(classes.len(), 1);
        assert_eq!(functions.len(), 2);
    }

    #[test]
    fn test_relations() {
        let mut map = ProjectMap::new();
        map.add_file("src/lib.rs", "");

        let loc = Location::new("src/lib.rs", 1, 1, 1, 10);

        let caller_id = map.add_symbol(Symbol::new(0, "caller", SymbolKind::Function, loc.clone()));
        let callee_id = map.add_symbol(Symbol::new(0, "callee", SymbolKind::Function, loc));

        map.add_relation(Relation::new(caller_id, callee_id, RelationKind::Calls));

        let calls = map.calls(caller_id);
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].name, "callee");

        let callers = map.callers(callee_id);
        assert_eq!(callers.len(), 1);
        assert_eq!(callers[0].name, "caller");
    }

    #[test]
    fn test_search() {
        let mut map = ProjectMap::new();
        map.add_file("src/lib.rs", "");

        let loc = Location::new("src/lib.rs", 1, 1, 1, 10);

        map.add_symbol(Symbol::new(0, "process_data", SymbolKind::Function, loc.clone()));
        map.add_symbol(Symbol::new(0, "handle_request", SymbolKind::Function, loc.clone()));
        map.add_symbol(Symbol::new(0, "DataProcessor", SymbolKind::Class, loc));

        let results = map.search("data");
        assert_eq!(results.len(), 2);

        let results = map.search("request");
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_children() {
        let mut map = ProjectMap::new();
        map.add_file("src/lib.rs", "");

        let loc = Location::new("src/lib.rs", 1, 1, 1, 10);

        let parent_id = map.add_symbol(Symbol::new(0, "MyClass", SymbolKind::Class, loc.clone()));
        map.add_symbol(Symbol::new(0, "method1", SymbolKind::Function, loc.clone()).with_parent(parent_id));
        map.add_symbol(Symbol::new(0, "method2", SymbolKind::Function, loc).with_parent(parent_id));

        let children = map.children(parent_id);
        assert_eq!(children.len(), 2);
    }

    #[test]
    fn test_relevant_files() {
        let mut map = ProjectMap::new();
        map.add_file("src/user/auth.rs", "");
        map.add_file("src/user/profile.rs", "");
        map.add_file("src/data/store.rs", "");

        let loc1 = Location::new("src/data/store.rs", 1, 1, 1, 10);
        map.add_symbol(Symbol::new(0, "UserStore", SymbolKind::Class, loc1));

        let relevant = map.relevant_files("user");
        assert_eq!(relevant.len(), 3); // Two files with "user" in path + one with "UserStore" symbol
    }

    #[test]
    fn test_stats() {
        let mut map = ProjectMap::new();
        map.add_file("src/main.rs", "fn main() {}");
        map.add_file("src/lib.py", "def foo(): pass");

        let loc1 = Location::new("src/main.rs", 1, 1, 1, 10);
        let loc2 = Location::new("src/lib.py", 1, 1, 1, 10);

        let id1 = map.add_symbol(Symbol::new(0, "main", SymbolKind::Function, loc1));
        let id2 = map.add_symbol(Symbol::new(0, "foo", SymbolKind::Function, loc2));

        map.add_relation(Relation::new(id1, id2, RelationKind::Calls));

        let stats = map.stats();
        assert_eq!(stats.total_files, 2);
        assert_eq!(stats.total_symbols, 2);
        assert_eq!(stats.total_relations, 1);
        assert_eq!(stats.symbols_by_kind.get(&SymbolKind::Function), Some(&2));
    }

    #[test]
    fn test_location() {
        let loc = Location::new("test.rs", 10, 5, 15, 20);
        assert_eq!(loc.start_line, 10);
        assert_eq!(loc.start_column, 5);
        assert_eq!(loc.end_line, 15);
        assert_eq!(loc.end_column, 20);

        let point = Location::point("test.rs", 10, 5);
        assert_eq!(point.start_line, point.end_line);
        assert_eq!(point.start_column, point.end_column);
    }

    #[test]
    fn test_relation_with_location() {
        let rel_loc = Location::point("src/main.rs", 5, 10);
        let relation = Relation::new(0, 1, RelationKind::Calls)
            .with_location(rel_loc);

        assert!(relation.location.is_some());
        assert_eq!(relation.location.unwrap().start_line, 5);
    }

    #[test]
    fn test_clear() {
        let mut map = ProjectMap::new();
        map.add_file("src/main.rs", "");

        let loc = Location::new("src/main.rs", 1, 1, 1, 10);
        map.add_symbol(Symbol::new(0, "test", SymbolKind::Function, loc));

        assert_eq!(map.files().count(), 1);
        assert_eq!(map.symbols().count(), 1);

        map.clear();

        assert_eq!(map.files().count(), 0);
        assert_eq!(map.symbols().count(), 0);
    }

    #[test]
    fn test_file_info() {
        let info = FileInfo::new("src/test.rs", Language::Rust);
        assert_eq!(info.path, PathBuf::from("src/test.rs"));
        assert_eq!(info.language, Language::Rust);
        assert!(info.symbols.is_empty());
        assert!(info.imports.is_empty());
    }
}
