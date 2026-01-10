//! # wix-hover
//!
//! Shared hover documentation provider for WiX XML files.
//!
//! This library provides rich hover information for WiX elements, attributes,
//! and values. It's designed to be used by editor integrations (VS Code, Sublime,
//! LSP servers, etc.) and CLI tools.
//!
//! ## Features
//!
//! - Element hover: description, parents, children, documentation links
//! - Attribute hover: type, required, default value, enum values
//! - Value hover: standard directories, builtin properties, auto-GUID
//! - Range tracking for precise highlighting
//!
//! ## Usage
//!
//! ```rust,ignore
//! use wix_hover::{HoverProvider, WixData};
//!
//! // Load WiX schema data
//! let data = WixData::load("path/to/wix-data")?;
//!
//! // Create provider
//! let provider = HoverProvider::new(data);
//!
//! // Get hover at position
//! let source = "<Component Guid=\"*\" />";
//! if let Some(info) = provider.hover(source, 1, 3) {
//!     println!("{}", info.contents);
//! }
//! ```
//!
//! ## CLI Usage
//!
//! ```bash
//! wix-hover file.wxs 10 15 --wix-data ./wix-data --format markdown
//! ```

mod context;
mod hover;
mod loader;
mod types;

pub use context::detect_hover_target;
pub use hover::HoverProvider;
pub use loader::{LoadError, WixData};
pub use types::{AttributeDef, ElementDef, HoverInfo, HoverTarget, Keywords, Range};
