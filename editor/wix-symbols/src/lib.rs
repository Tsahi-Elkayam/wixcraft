//! # wix-symbols
//!
//! WiX document symbols extraction library.
//!
//! Extracts Components, Directories, Features, Properties, and other named
//! elements from WiX XML files for outline views and "Go to Symbol" features.
//!
//! ## Features
//!
//! - Extract symbols from 26+ WiX element types
//! - Hierarchical symbol tree with children
//! - Flatten tree to list for workspace symbol search
//! - Filter/query symbols by name
//! - LSP-compatible SymbolKind values
//! - Selection range for precise identifier highlighting
//!
//! ## Example
//!
//! ```
//! use wix_symbols::{extract_symbols, Symbol, SymbolKind};
//!
//! let source = r#"<Wix><Component Id="MainComp" Guid="*" /></Wix>"#;
//! let symbols = extract_symbols(source).unwrap();
//!
//! assert_eq!(symbols.len(), 1);
//! assert_eq!(symbols[0].name, "MainComp");
//! ```
//!
//! ## CLI Usage
//!
//! ```bash
//! # Extract symbols (hierarchical)
//! wix-symbols file.wxs
//!
//! # Flat list
//! wix-symbols file.wxs --flat
//!
//! # Search symbols
//! wix-symbols file.wxs --query "Main"
//!
//! # JSON output
//! wix-symbols file.wxs --format json
//! ```

mod symbols;
mod types;

pub use symbols::{extract_symbols, filter_symbols, flatten_symbols};
pub use types::{Position, Range, Symbol, SymbolKind};
