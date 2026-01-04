//! WiX XML formatter library
//!
//! Provides consistent formatting for WiX XML files with optional
//! data-driven element and attribute ordering based on wix-data.

mod config;
mod formatter;
mod loader;
mod ordering;
mod writer;

pub use config::{FormatConfig, IndentStyle};
pub use formatter::{format, format_file, Formatter};
pub use loader::{LoadError, WixData};
