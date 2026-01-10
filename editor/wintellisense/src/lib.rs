//! Wintellisense - Context-aware autocomplete engine for WiX XML files
//!
//! This library provides intelligent code completion, go-to-definition, hover
//! information, and project indexing for WiX installer files.
//!
//! # Architecture
//!
//! ```text
//! CLI/LSP -> Engine -> ProviderManager -> Provider -> Data
//!                  |
//!                  +-> ProjectIndex (cross-file symbols)
//! ```
//!
//! # Features
//!
//! - **Schema Autocomplete**: Elements, attributes, values from wixkb
//! - **Snippets**: Code templates with placeholders
//! - **Go-to-Definition**: Navigate to symbol definitions
//! - **Hover**: Documentation on hover
//! - **Project Index**: Cross-file symbol references
//!
//! # Plugin System
//!
//! Like Winter linter, Wintellisense supports language plugins via YAML manifests:
//!
//! ```yaml
//! plugin:
//!   id: wix
//!   version: "1.0.0"
//!   extensions: ["wxs", "wxi"]
//!
//! completions:
//!   elements: true
//!   attributes: true
//!   snippets: true
//!
//! definitions:
//!   - pattern: "ComponentRef"
//!     target: "Component"
//!     attribute: "Id"
//! ```

pub mod completions;
pub mod context;
pub mod index;
pub mod loader;
pub mod plugin;
pub mod providers;
pub mod types;

// Re-export main types
pub use context::parse_context;
pub use index::ProjectIndex;
pub use loader::SchemaData;
pub use types::{
    CompletionItem, CompletionKind, CompletionResult, CursorContext,
    Definition, DefinitionResult, HoverInfo, HoverResult, Location, Position,
};

use anyhow::Result;
use std::path::Path;
use std::sync::Arc;

/// Main autocomplete engine
pub struct Engine {
    /// Schema data from wixkb
    schema: Arc<SchemaData>,

    /// Project index for cross-file symbols
    index: ProjectIndex,

    /// Maximum completions to return
    max_completions: usize,
}

impl Engine {
    /// Create engine from wixkb database path
    pub fn new(wixkb_path: &Path) -> Result<Self> {
        let schema = Arc::new(SchemaData::load(wixkb_path)?);
        Ok(Self {
            schema,
            index: ProjectIndex::new(),
            max_completions: 50,
        })
    }

    /// Create engine with pre-loaded schema
    pub fn with_schema(schema: SchemaData) -> Self {
        Self {
            schema: Arc::new(schema),
            index: ProjectIndex::new(),
            max_completions: 50,
        }
    }

    /// Set maximum completions
    pub fn with_max_completions(mut self, max: usize) -> Self {
        self.max_completions = max;
        self
    }

    /// Index a project directory for cross-file symbols
    pub fn index_project(&mut self, root: &Path) -> Result<usize> {
        self.index.index_directory(root)
    }

    /// Index a single file
    pub fn index_file(&mut self, path: &Path) -> Result<()> {
        self.index.index_file(path)
    }

    /// Get completions at cursor position (1-based line/column)
    pub fn complete(&self, source: &str, line: u32, column: u32) -> CompletionResult {
        let ctx = parse_context(source, line, column);
        self.complete_with_context(&ctx, source)
    }

    /// Get completions with pre-parsed context
    pub fn complete_with_context(&self, ctx: &CursorContext, source: &str) -> CompletionResult {
        completions::get_completions(&self.schema, &self.index, ctx, source, self.max_completions)
    }

    /// Get definition for symbol at position
    pub fn go_to_definition(&self, source: &str, line: u32, column: u32) -> DefinitionResult {
        let ctx = parse_context(source, line, column);
        providers::definitions::find_definition(&self.schema, &self.index, &ctx, source)
    }

    /// Get hover information at position
    pub fn hover(&self, source: &str, line: u32, column: u32) -> HoverResult {
        let ctx = parse_context(source, line, column);
        providers::hover::get_hover_info(&self.schema, &ctx, source)
    }

    /// Get the schema data
    pub fn schema(&self) -> &SchemaData {
        &self.schema
    }

    /// Get the project index
    pub fn index(&self) -> &ProjectIndex {
        &self.index
    }

    /// Get statistics
    pub fn stats(&self) -> EngineStats {
        EngineStats {
            elements: self.schema.elements.len(),
            snippets: self.schema.snippets.len(),
            indexed_files: self.index.file_count(),
            indexed_symbols: self.index.symbol_count(),
        }
    }
}

/// Engine statistics
#[derive(Debug, Clone)]
pub struct EngineStats {
    pub elements: usize,
    pub snippets: usize,
    pub indexed_files: usize,
    pub indexed_symbols: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_engine_stats() {
        let schema = SchemaData::default();
        let engine = Engine::with_schema(schema);
        let stats = engine.stats();
        assert_eq!(stats.elements, 0);
        assert_eq!(stats.indexed_files, 0);
    }
}
