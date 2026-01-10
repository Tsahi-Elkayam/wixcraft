//! WiX Language Server Protocol implementation
//!
//! A generic LSP engine with a plugin architecture. The WiX plugin provides
//! support for WiX installer development with:
//!
//! - Hover documentation (wix-hover)
//! - Document symbols (wix-symbols)
//! - Diagnostics/linting (wix-lint)
//! - Formatting (wix-fmt)
//!
//! # Architecture
//!
//! The LSP is built with a plugin system:
//!
//! - **Engine**: Generic LSP server that delegates to plugins
//! - **Plugins**: Language-specific implementations (e.g., WiX)
//! - **Config**: YAML-based configuration
//!
//! # Usage
//!
//! Run the language server via stdio:
//!
//! ```bash
//! wix-lsp
//! ```
//!
//! The server will automatically discover wix-data from the workspace root.
//!
//! # Configuration
//!
//! Create a `.wix-lsp.yaml` in your workspace root:
//!
//! ```yaml
//! engine:
//!   name: wix-lsp
//!   log_level: info
//!
//! plugins:
//!   data_search_paths:
//!     - wix-data
//!     - .wix-data
//! ```

pub mod engine;
pub mod plugins;

pub use engine::{DocumentManager, EngineConfig, LspServer};
pub use plugins::wix::WixPlugin;
pub use plugins::PluginRegistry;
