//! WiX document symbols extraction library
//!
//! Extracts Components, Directories, Features, Properties, and other named
//! elements from WiX XML files for outline views and "Go to Symbol" features.
//!
//! # Example
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

mod symbols;
mod types;

pub use symbols::{extract_symbols, filter_symbols, flatten_symbols};
pub use types::{Position, Range, Symbol, SymbolKind};
