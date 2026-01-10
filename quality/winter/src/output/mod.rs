//! Output formatters for lint results

mod azure;
mod compact;
mod github;
mod gitlab;
mod grouped;
mod json;
mod junit;
mod sarif;
mod text;

pub use azure::AzureFormatter;
pub use compact::CompactFormatter;
pub use github::GithubFormatter;
pub use gitlab::GitlabFormatter;
pub use grouped::GroupedFormatter;
pub use json::JsonFormatter;
pub use junit::JUnitFormatter;
pub use sarif::SarifFormatter;
pub use text::TextFormatter;

use crate::diagnostic::Diagnostic;
use crate::engine::LintResult;

/// Output formatter trait
pub trait OutputFormatter: Send + Sync {
    /// Format the entire lint result
    fn format(&self, result: &LintResult) -> String;

    /// Format a single diagnostic
    fn format_diagnostic(&self, diagnostic: &Diagnostic) -> String;
}
