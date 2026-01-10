//! Language Server Protocol support
//!
//! Provides LSP server for real-time linting in IDEs.
//!
//! Features:
//! - Real-time diagnostics on file changes
//! - Code actions for auto-fix suggestions
//! - Hover information for rules

use crate::diagnostic::{Diagnostic, Severity};
use crate::engine::LintResult;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// LSP diagnostic severity (matches LSP spec)
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum LspSeverity {
    Error = 1,
    Warning = 2,
    Information = 3,
    Hint = 4,
}

impl From<Severity> for LspSeverity {
    fn from(severity: Severity) -> Self {
        match severity {
            Severity::Error => LspSeverity::Error,
            Severity::Warning => LspSeverity::Warning,
            Severity::Info => LspSeverity::Information,
        }
    }
}

/// LSP position (0-indexed)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub line: u32,
    pub character: u32,
}

/// LSP range
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Range {
    pub start: Position,
    pub end: Position,
}

/// LSP diagnostic
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LspDiagnostic {
    pub range: Range,
    pub severity: Option<u32>,
    pub code: Option<String>,
    pub source: Option<String>,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

/// LSP code action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeAction {
    pub title: String,
    pub kind: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub diagnostics: Option<Vec<LspDiagnostic>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub edit: Option<WorkspaceEdit>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_preferred: Option<bool>,
}

/// LSP workspace edit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceEdit {
    pub changes: Option<HashMap<String, Vec<TextEdit>>>,
}

/// LSP text edit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextEdit {
    pub range: Range,
    #[serde(rename = "newText")]
    pub new_text: String,
}

/// Convert Winter diagnostics to LSP diagnostics
pub fn to_lsp_diagnostics(diagnostics: &[Diagnostic]) -> Vec<LspDiagnostic> {
    diagnostics
        .iter()
        .map(|d| LspDiagnostic {
            range: Range {
                start: Position {
                    line: d.location.line.saturating_sub(1) as u32,
                    character: d.location.column.saturating_sub(1) as u32,
                },
                end: Position {
                    line: d.location.line.saturating_sub(1) as u32,
                    character: (d.location.column + d.location.length).saturating_sub(1) as u32,
                },
            },
            severity: Some(LspSeverity::from(d.severity) as u32),
            code: Some(d.rule_id.clone()),
            source: Some("winter".to_string()),
            message: d.message.clone(),
            data: d.fix.as_ref().map(|f| {
                serde_json::json!({
                    "fix": {
                        "description": f.description,
                        "replacement": f.replacement,
                        "safety": f.safety.to_string(),
                    }
                })
            }),
        })
        .collect()
}

/// Convert Winter diagnostic to LSP code action
pub fn to_code_action(diagnostic: &Diagnostic, uri: &str) -> Option<CodeAction> {
    let fix = diagnostic.fix.as_ref()?;

    let text_edit = TextEdit {
        range: Range {
            start: Position {
                line: diagnostic.location.line.saturating_sub(1) as u32,
                character: 0,
            },
            end: Position {
                line: diagnostic.location.line.saturating_sub(1) as u32,
                character: u32::MAX, // End of line
            },
        },
        new_text: fix.replacement.clone(),
    };

    let mut changes = HashMap::new();
    changes.insert(uri.to_string(), vec![text_edit]);

    let kind = if fix.is_safe() {
        "quickfix"
    } else {
        "quickfix.unsafe"
    };

    let title = if fix.is_safe() {
        format!("{} (safe)", fix.description)
    } else {
        format!("{} (unsafe)", fix.description)
    };

    Some(CodeAction {
        title,
        kind: Some(kind.to_string()),
        diagnostics: None,
        edit: Some(WorkspaceEdit {
            changes: Some(changes),
        }),
        is_preferred: Some(fix.is_safe()),
    })
}

/// Publish diagnostics notification parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublishDiagnosticsParams {
    pub uri: String,
    pub diagnostics: Vec<LspDiagnostic>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<i32>,
}

/// Create publish diagnostics params from lint result
pub fn to_publish_diagnostics(file: &PathBuf, result: &LintResult) -> PublishDiagnosticsParams {
    let file_diagnostics: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| &d.location.file == file)
        .cloned()
        .collect();

    PublishDiagnosticsParams {
        uri: format!("file://{}", file.display()),
        diagnostics: to_lsp_diagnostics(&file_diagnostics),
        version: None,
    }
}

/// Server capabilities for LSP initialization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerCapabilities {
    /// Sync mode: 1 = Full, 2 = Incremental
    #[serde(rename = "textDocumentSync")]
    pub text_document_sync: u8,
    /// Support for code actions
    #[serde(rename = "codeActionProvider")]
    pub code_action_provider: bool,
    /// Support for hover
    #[serde(rename = "hoverProvider")]
    pub hover_provider: bool,
}

impl Default for ServerCapabilities {
    fn default() -> Self {
        Self {
            text_document_sync: 1, // Full sync
            code_action_provider: true,
            hover_provider: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagnostic::Location;

    #[test]
    fn test_to_lsp_diagnostics() {
        let diag = Diagnostic::new(
            "test-rule",
            Severity::Error,
            "Test error",
            Location::new(PathBuf::from("test.wxs"), 10, 5),
        );

        let lsp_diags = to_lsp_diagnostics(&[diag]);
        assert_eq!(lsp_diags.len(), 1);
        assert_eq!(lsp_diags[0].range.start.line, 9); // 0-indexed
        assert_eq!(lsp_diags[0].severity, Some(1)); // Error
        assert_eq!(lsp_diags[0].code, Some("test-rule".to_string()));
    }

    #[test]
    fn test_to_code_action() {
        let diag = Diagnostic::new(
            "test-rule",
            Severity::Warning,
            "Test warning",
            Location::new(PathBuf::from("test.wxs"), 5, 1),
        )
        .with_fix("Fix it", "fixed content");

        let action = to_code_action(&diag, "file:///test.wxs").unwrap();
        assert!(action.title.contains("Fix it"));
        assert!(action.title.contains("safe"));
        assert_eq!(action.kind, Some("quickfix".to_string()));
        assert!(action.is_preferred.unwrap());
    }

    #[test]
    fn test_lsp_severity_conversion() {
        assert_eq!(LspSeverity::from(Severity::Error) as u32, 1);
        assert_eq!(LspSeverity::from(Severity::Warning) as u32, 2);
        assert_eq!(LspSeverity::from(Severity::Info) as u32, 3);
    }
}
