//! wix-autocomplete: Context-aware autocomplete for WiX XML files
//!
//! This library provides intelligent autocomplete suggestions for WiX installer
//! files by analyzing cursor context and providing relevant elements, attributes,
//! values, and snippets.
//!
//! # Example
//!
//! ```no_run
//! use wix_autocomplete::{AutocompleteEngine, CursorContext};
//! use std::path::Path;
//!
//! // Load the engine from wix-data
//! let engine = AutocompleteEngine::from_wix_data(Path::new("path/to/wix-data")).unwrap();
//!
//! // Get completions at a cursor position
//! let source = "<Package>\n  <";
//! let completions = engine.complete(source, 2, 4);
//!
//! for item in completions {
//!     println!("{}: {}", item.label, item.detail.unwrap_or_default());
//! }
//! ```

mod completions;
mod context;
mod loader;
mod types;

pub use context::parse_context;
pub use loader::{LoadError, WixData};
pub use types::{
    AttributeDef, CompletionItem, CompletionKind, CursorContext, ElementDef, Keywords, Snippet,
};

use std::path::Path;

/// Main autocomplete engine
pub struct AutocompleteEngine {
    data: WixData,
    max_completions: usize,
}

impl AutocompleteEngine {
    /// Load autocomplete engine from wix-data directory
    pub fn from_wix_data(path: &Path) -> Result<Self, LoadError> {
        let data = WixData::load(path)?;
        Ok(Self {
            data,
            max_completions: 50,
        })
    }

    /// Create engine from pre-loaded WixData
    pub fn new(data: WixData) -> Self {
        Self {
            data,
            max_completions: 50,
        }
    }

    /// Set maximum number of completions to return
    pub fn with_max_completions(mut self, max: usize) -> Self {
        self.max_completions = max;
        self
    }

    /// Get completions at cursor position (line and column are 1-based)
    pub fn complete(&self, source: &str, line: u32, column: u32) -> Vec<CompletionItem> {
        let context = parse_context(source, line, column);
        self.complete_context(&context, source)
    }

    /// Get completions for a pre-parsed context
    pub fn complete_context(&self, context: &CursorContext, source: &str) -> Vec<CompletionItem> {
        completions::get_completions(&self.data, context, source, self.max_completions)
    }

    /// Get the underlying WixData
    pub fn data(&self) -> &WixData {
        &self.data
    }

    /// Get number of loaded elements
    pub fn element_count(&self) -> usize {
        self.data.elements.len()
    }

    /// Get number of loaded snippets
    pub fn snippet_count(&self) -> usize {
        self.data.snippets.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_wix_data() -> TempDir {
        let temp = TempDir::new().unwrap();

        let elements_dir = temp.path().join("elements");
        fs::create_dir(&elements_dir).unwrap();

        let wix = r#"{"name": "Wix", "description": "Root", "parents": [], "children": ["Package"], "attributes": {}}"#;
        fs::write(elements_dir.join("wix.json"), wix).unwrap();

        let package = r#"{"name": "Package", "description": "Package", "parents": ["Wix"], "children": ["Component", "Directory"], "attributes": {"Name": {"type": "string", "required": true}}}"#;
        fs::write(elements_dir.join("package.json"), package).unwrap();

        let component = r#"{"name": "Component", "description": "Component", "parents": ["Package", "Directory"], "children": ["File"], "attributes": {"Guid": {"type": "guid", "required": true}}}"#;
        fs::write(elements_dir.join("component.json"), component).unwrap();

        let keywords_dir = temp.path().join("keywords");
        fs::create_dir(&keywords_dir).unwrap();
        fs::write(
            keywords_dir.join("keywords.json"),
            r#"{"standardDirectories": ["ProgramFilesFolder"], "builtinProperties": [], "elements": [], "preprocessorDirectives": []}"#,
        )
        .unwrap();

        let snippets_dir = temp.path().join("snippets");
        fs::create_dir(&snippets_dir).unwrap();
        fs::write(snippets_dir.join("snippets.json"), r#"{"snippets": []}"#).unwrap();

        temp
    }

    #[test]
    fn test_engine_creation() {
        let temp = create_test_wix_data();
        let engine = AutocompleteEngine::from_wix_data(temp.path()).unwrap();

        assert!(engine.element_count() > 0);
    }

    #[test]
    fn test_complete_element() {
        let temp = create_test_wix_data();
        let engine = AutocompleteEngine::from_wix_data(temp.path()).unwrap();

        let source = "<Package>\n  <";
        let completions = engine.complete(source, 2, 4);

        assert!(!completions.is_empty());
        assert!(completions.iter().any(|c| c.label == "Component"));
    }

    #[test]
    fn test_complete_attribute() {
        let temp = create_test_wix_data();
        let engine = AutocompleteEngine::from_wix_data(temp.path()).unwrap();

        let source = "<Component ";
        let completions = engine.complete(source, 1, 12);

        assert!(!completions.is_empty());
        assert!(completions.iter().any(|c| c.label == "Guid"));
    }

    #[test]
    fn test_max_completions() {
        let temp = create_test_wix_data();
        let engine = AutocompleteEngine::from_wix_data(temp.path())
            .unwrap()
            .with_max_completions(1);

        let source = "<Package>\n  <";
        let completions = engine.complete(source, 2, 4);

        assert!(completions.len() <= 1);
    }
}
