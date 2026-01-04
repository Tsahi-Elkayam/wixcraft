//! WiX hover documentation provider
//!
//! Provides rich documentation when hovering over WiX elements and attributes.

mod context;
mod hover;
mod loader;
mod types;

pub use hover::HoverProvider;
pub use loader::{LoadError, WixData};
pub use types::{HoverInfo, HoverTarget, Range};
