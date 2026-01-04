//! wix-lint: A linter for WiX XML files
//!
//! This library provides linting capabilities for WiX (Windows Installer XML) files,
//! helping developers catch common mistakes and follow best practices.

pub mod config;
pub mod diagnostics;
pub mod engine;
pub mod loader;
pub mod output;
pub mod parser;
pub mod plugins;
pub mod rules;

pub use config::{CliOptions, Config, ConfigError};
pub use diagnostics::{Diagnostic, Severity};
pub use engine::{LintEngine, LintStatistics};
pub use loader::RuleLoader;
pub use parser::WixDocument;
pub use plugins::{PluginError, PluginManager};
pub use rules::Rule;
