//! WiX symbol references library
//!
//! Provides Go to Definition and Find References functionality for WiX XML files.
//!
//! # Example
//!
//! ```
//! use wix_references::{SymbolIndex, go_to_definition};
//! use std::path::Path;
//!
//! let mut index = SymbolIndex::new();
//! let source = r#"<Wix><Component Id="MainComp" /><ComponentRef Id="MainComp" /></Wix>"#;
//! index.index_file(Path::new("test.wxs"), source).unwrap();
//!
//! // Go to definition from ComponentRef
//! let result = go_to_definition(source, 1, 50, &index);
//! assert!(result.definition.is_some());
//! ```

mod extractor;
mod index;
mod resolver;
mod types;

pub use extractor::{extract_from_source, ExtractionResult};
pub use index::SymbolIndex;
pub use resolver::{
    detect_symbol_at, find_definition_by_id, find_references, find_references_by_id,
    go_to_definition,
};
pub use types::{
    DefinitionKind, DefinitionResult, Location, Position, Range, ReferenceKind, ReferencesResult,
    SymbolDefinition, SymbolReference, SymbolTarget,
};
