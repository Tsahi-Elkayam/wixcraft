//! Winter - Universal XML Linter Framework
//!
//! A fast, modular linter for XML-based files with plugin support.
//! While it ships with WiX support, it can be extended to lint any XML format
//! through YAML/JSON plugin manifests.
//!
//! # Architecture
//!
//! ```text
//! CLI/API -> Engine -> PluginManager -> Plugin -> File
//! ```
//!
//! The engine loads configuration, discovers plugins (both built-in and dynamic),
//! routes files to appropriate plugins based on extension, and collects diagnostics.
//!
//! # Creating Custom Plugins
//!
//! Create a YAML manifest file (e.g., `jenkins.yaml`):
//!
//! ```yaml
//! plugin:
//!   id: jenkins
//!   version: "1.0.0"
//!   description: "Jenkins pipeline linter"
//!   extensions: ["Jenkinsfile"]
//!   base_parser: xml
//!
//! rules:
//!   - id: jenkins-no-hardcoded-credentials
//!     condition: "name == 'sh' && attributes.script =~ /password/i"
//!     message: "Avoid hardcoding credentials"
//!     severity: error
//! ```

pub mod baseline;
pub mod cache;
pub mod complexity;
pub mod config;
pub mod cross_file;
pub mod diagnostic;
pub mod engine;
pub mod fixer;
pub mod lsp;
pub mod output;
pub mod plugin;
pub mod plugin_manager;
pub mod rule;
pub mod watch;

// Re-export main types
pub use baseline::Baseline;
pub use cache::LintCache;
pub use complexity::{ComplexityAnalyzer, ComplexityMetrics, ComplexityRating};
pub use config::Config;
pub use cross_file::CrossFileValidator;
pub use diagnostic::{Diagnostic, Fix as DiagnosticFix, FixSafety, Location, Severity};
pub use engine::{Engine, LintResult, RuleTiming};
pub use fixer::{Fix, FixMode, FixResult, Fixer};
pub use lsp::{
    to_code_action, to_lsp_diagnostics, to_publish_diagnostics, CodeAction, LspDiagnostic,
    LspSeverity, Position as LspPosition, PublishDiagnosticsParams, Range as LspRange,
    ServerCapabilities, TextEdit, WorkspaceEdit,
};
pub use output::{
    AzureFormatter, CompactFormatter, GithubFormatter, GitlabFormatter, GroupedFormatter,
    JUnitFormatter, OutputFormatter,
};
pub use plugin::{Document, Node, Plugin};
pub use plugin_manager::{DynamicPlugin, EmbeddedLanguage, PluginManager, PluginManifest};
pub use rule::{Rule, RuleCategory, RuleStability};
pub use watch::Watcher;

// Built-in plugins
pub mod plugins {
    pub mod wix;
    pub mod xml;
}
