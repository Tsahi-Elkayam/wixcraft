//! Code actions for LSP quick fixes

use tower_lsp::lsp_types::*;
use crate::{Diagnostic as WixDiagnostic, Fix, FixAction};

/// Provides code actions (quick fixes) for diagnostics
pub struct CodeActionProvider {
    // Reserved for future configuration
}

impl CodeActionProvider {
    pub fn new() -> Self {
        Self {}
    }

    /// Get code actions for the given diagnostics
    pub fn get_actions(
        &self,
        uri: &Url,
        diagnostics: &[WixDiagnostic],
        content: &str,
    ) -> Vec<CodeActionOrCommand> {
        let mut actions = Vec::new();

        for diag in diagnostics {
            // Get quick fixes from the diagnostic's fix field
            if let Some(ref fix) = diag.fix {
                if let Some(action) = self.fix_to_code_action(uri, diag, fix) {
                    actions.push(CodeActionOrCommand::CodeAction(action));
                }
            }

            // Generate additional quick fixes based on rule ID
            actions.extend(self.generate_rule_specific_actions(uri, diag, content));
        }

        // Add "Disable rule" actions
        for diag in diagnostics {
            actions.push(CodeActionOrCommand::CodeAction(
                self.create_disable_rule_action(uri, diag, content),
            ));
        }

        actions
    }

    /// Convert a Fix to a CodeAction
    fn fix_to_code_action(
        &self,
        uri: &Url,
        diag: &WixDiagnostic,
        fix: &Fix,
    ) -> Option<CodeAction> {
        let text_edit = match &fix.action {
            FixAction::ReplaceText { range, new_text } => TextEdit {
                range: self.wix_range_to_lsp(range),
                new_text: new_text.clone(),
            },
            FixAction::AddAttribute { range, name, value } => {
                // Insert attribute at end of range (before >)
                let lsp_range = self.wix_range_to_lsp(range);
                TextEdit {
                    range: Range { start: lsp_range.end, end: lsp_range.end },
                    new_text: format!(" {}=\"{}\"", name, value),
                }
            }
            FixAction::RemoveAttribute { range, .. } => TextEdit {
                range: self.wix_range_to_lsp(range),
                new_text: String::new(),
            },
            FixAction::ReplaceAttribute { range, new_value, .. } => TextEdit {
                range: self.wix_range_to_lsp(range),
                new_text: new_value.clone(),
            },
            FixAction::AddElement { parent_range, element, .. } => {
                // Insert element after parent opening tag
                let lsp_range = self.wix_range_to_lsp(parent_range);
                TextEdit {
                    range: Range { start: lsp_range.end, end: lsp_range.end },
                    new_text: format!("\n  <{} />", element),
                }
            }
            FixAction::RemoveElement { range } => TextEdit {
                range: self.wix_range_to_lsp(range),
                new_text: String::new(),
            },
        };

        let mut changes = std::collections::HashMap::new();
        changes.insert(uri.clone(), vec![text_edit]);

        Some(CodeAction {
            title: fix.description.clone(),
            kind: Some(CodeActionKind::QUICKFIX),
            diagnostics: Some(vec![self.wix_diagnostic_to_lsp(diag)]),
            edit: Some(WorkspaceEdit {
                changes: Some(changes),
                document_changes: None,
                change_annotations: None,
            }),
            command: None,
            is_preferred: Some(true),
            disabled: None,
            data: None,
        })
    }

    /// Generate rule-specific code actions
    fn generate_rule_specific_actions(
        &self,
        uri: &Url,
        diag: &WixDiagnostic,
        content: &str,
    ) -> Vec<CodeActionOrCommand> {
        let mut actions = Vec::new();

        match diag.rule_id.as_str() {
            // Missing GUID - offer to generate one
            "BP-IDIOM-002" | "VAL-ATTR-002" => {
                if let Some(action) = self.create_add_guid_action(uri, diag) {
                    actions.push(CodeActionOrCommand::CodeAction(action));
                }
            }

            // Missing Id - offer to generate one
            "VAL-REQ-001" | "VAL-REQ-002" => {
                if let Some(action) = self.create_add_id_action(uri, diag, content) {
                    actions.push(CodeActionOrCommand::CodeAction(action));
                }
            }

            // Missing MajorUpgrade - offer to add it
            "BP-IDIOM-001" => {
                if let Some(action) = self.create_add_major_upgrade_action(uri, diag) {
                    actions.push(CodeActionOrCommand::CodeAction(action));
                }
            }

            // Hardcoded path - offer to use property
            "BP-MAINT-001" => {
                if let Some(action) = self.create_use_property_action(uri, diag, content) {
                    actions.push(CodeActionOrCommand::CodeAction(action));
                }
            }

            // Dead code - offer to remove
            "DEAD-COMP-001" | "DEAD-PROP-001" | "DEAD-CA-001" | "DEAD-DIR-001" => {
                if let Some(action) = self.create_remove_element_action(uri, diag) {
                    actions.push(CodeActionOrCommand::CodeAction(action));
                }
            }

            _ => {}
        }

        actions
    }

    /// Create a "Disable rule" action
    fn create_disable_rule_action(&self, uri: &Url, diag: &WixDiagnostic, content: &str) -> CodeAction {
        let line = diag.location.range.start.line;
        let indent = self.get_indent(content, line);

        let comment = format!("{}<!-- wix-analyzer-disable {} -->\n", indent, diag.rule_id);

        let mut changes = std::collections::HashMap::new();
        changes.insert(
            uri.clone(),
            vec![TextEdit {
                range: Range {
                    start: Position {
                        line: line.saturating_sub(1) as u32,
                        character: 0,
                    },
                    end: Position {
                        line: line.saturating_sub(1) as u32,
                        character: 0,
                    },
                },
                new_text: comment,
            }],
        );

        CodeAction {
            title: format!("Disable rule {} for this line", diag.rule_id),
            kind: Some(CodeActionKind::QUICKFIX),
            diagnostics: Some(vec![self.wix_diagnostic_to_lsp(diag)]),
            edit: Some(WorkspaceEdit {
                changes: Some(changes),
                document_changes: None,
                change_annotations: None,
            }),
            command: None,
            is_preferred: Some(false),
            disabled: None,
            data: None,
        }
    }

    /// Create an action to add a GUID
    fn create_add_guid_action(&self, uri: &Url, diag: &WixDiagnostic) -> Option<CodeAction> {
        let guid = generate_guid();
        let line = diag.location.range.start.line;

        let mut changes = std::collections::HashMap::new();
        // Insert after the element name
        changes.insert(
            uri.clone(),
            vec![TextEdit {
                range: Range {
                    start: Position {
                        line: line.saturating_sub(1) as u32,
                        character: diag.location.range.end.character.saturating_sub(1) as u32,
                    },
                    end: Position {
                        line: line.saturating_sub(1) as u32,
                        character: diag.location.range.end.character.saturating_sub(1) as u32,
                    },
                },
                new_text: format!(" Guid=\"{}\"", guid),
            }],
        );

        Some(CodeAction {
            title: "Add generated GUID".to_string(),
            kind: Some(CodeActionKind::QUICKFIX),
            diagnostics: Some(vec![self.wix_diagnostic_to_lsp(diag)]),
            edit: Some(WorkspaceEdit {
                changes: Some(changes),
                document_changes: None,
                change_annotations: None,
            }),
            command: None,
            is_preferred: Some(true),
            disabled: None,
            data: None,
        })
    }

    /// Create an action to add an Id
    fn create_add_id_action(&self, uri: &Url, diag: &WixDiagnostic, content: &str) -> Option<CodeAction> {
        let line = diag.location.range.start.line;
        let element = self.get_element_name_at_line(content, line)?;
        let id = generate_id(&element);

        let mut changes = std::collections::HashMap::new();
        changes.insert(
            uri.clone(),
            vec![TextEdit {
                range: Range {
                    start: Position {
                        line: line.saturating_sub(1) as u32,
                        character: (diag.location.range.start.character + element.len()).saturating_sub(1) as u32,
                    },
                    end: Position {
                        line: line.saturating_sub(1) as u32,
                        character: (diag.location.range.start.character + element.len()).saturating_sub(1) as u32,
                    },
                },
                new_text: format!(" Id=\"{}\"", id),
            }],
        );

        Some(CodeAction {
            title: format!("Add Id=\"{}\"", id),
            kind: Some(CodeActionKind::QUICKFIX),
            diagnostics: Some(vec![self.wix_diagnostic_to_lsp(diag)]),
            edit: Some(WorkspaceEdit {
                changes: Some(changes),
                document_changes: None,
                change_annotations: None,
            }),
            command: None,
            is_preferred: Some(true),
            disabled: None,
            data: None,
        })
    }

    /// Create an action to add MajorUpgrade
    fn create_add_major_upgrade_action(&self, uri: &Url, diag: &WixDiagnostic) -> Option<CodeAction> {
        let line = diag.location.range.end.line;

        let mut changes = std::collections::HashMap::new();
        changes.insert(
            uri.clone(),
            vec![TextEdit {
                range: Range {
                    start: Position {
                        line: line as u32,
                        character: 0,
                    },
                    end: Position {
                        line: line as u32,
                        character: 0,
                    },
                },
                new_text: "    <MajorUpgrade DowngradeErrorMessage=\"A newer version is already installed.\" />\n".to_string(),
            }],
        );

        Some(CodeAction {
            title: "Add MajorUpgrade element".to_string(),
            kind: Some(CodeActionKind::QUICKFIX),
            diagnostics: Some(vec![self.wix_diagnostic_to_lsp(diag)]),
            edit: Some(WorkspaceEdit {
                changes: Some(changes),
                document_changes: None,
                change_annotations: None,
            }),
            command: None,
            is_preferred: Some(true),
            disabled: None,
            data: None,
        })
    }

    /// Create an action to use a property instead of hardcoded path
    fn create_use_property_action(&self, uri: &Url, diag: &WixDiagnostic, content: &str) -> Option<CodeAction> {
        let line = diag.location.range.start.line;
        let line_content = content.lines().nth(line.saturating_sub(1))?;

        // Find hardcoded path and suggest replacement
        if let Some(start) = line_content.find("C:\\") {
            let end = line_content[start..].find('"').map(|i| start + i)?;
            let path = &line_content[start..end];

            // Suggest using [ProgramFilesFolder]
            let new_path = path.replacen("C:\\Program Files\\", "[ProgramFilesFolder]", 1);

            let mut changes = std::collections::HashMap::new();
            changes.insert(
                uri.clone(),
                vec![TextEdit {
                    range: Range {
                        start: Position {
                            line: line.saturating_sub(1) as u32,
                            character: start as u32,
                        },
                        end: Position {
                            line: line.saturating_sub(1) as u32,
                            character: end as u32,
                        },
                    },
                    new_text: new_path,
                }],
            );

            return Some(CodeAction {
                title: "Use standard directory property".to_string(),
                kind: Some(CodeActionKind::QUICKFIX),
                diagnostics: Some(vec![self.wix_diagnostic_to_lsp(diag)]),
                edit: Some(WorkspaceEdit {
                    changes: Some(changes),
                    document_changes: None,
                    change_annotations: None,
                }),
                command: None,
                is_preferred: Some(false),
                disabled: None,
                data: None,
            });
        }

        None
    }

    /// Create an action to remove an element
    fn create_remove_element_action(&self, uri: &Url, diag: &WixDiagnostic) -> Option<CodeAction> {
        let mut changes = std::collections::HashMap::new();
        changes.insert(
            uri.clone(),
            vec![TextEdit {
                range: self.wix_range_to_lsp(&diag.location.range),
                new_text: String::new(),
            }],
        );

        Some(CodeAction {
            title: "Remove unused element".to_string(),
            kind: Some(CodeActionKind::QUICKFIX),
            diagnostics: Some(vec![self.wix_diagnostic_to_lsp(diag)]),
            edit: Some(WorkspaceEdit {
                changes: Some(changes),
                document_changes: None,
                change_annotations: None,
            }),
            command: None,
            is_preferred: Some(false),
            disabled: None,
            data: None,
        })
    }

    // Helper methods

    fn wix_range_to_lsp(&self, range: &crate::core::Range) -> Range {
        Range {
            start: Position {
                line: range.start.line.saturating_sub(1) as u32,
                character: range.start.character.saturating_sub(1) as u32,
            },
            end: Position {
                line: range.end.line.saturating_sub(1) as u32,
                character: range.end.character.saturating_sub(1) as u32,
            },
        }
    }

    fn wix_diagnostic_to_lsp(&self, diag: &WixDiagnostic) -> Diagnostic {
        Diagnostic {
            range: self.wix_range_to_lsp(&diag.location.range),
            severity: Some(match diag.severity {
                crate::Severity::Blocker | crate::Severity::High => DiagnosticSeverity::ERROR,
                crate::Severity::Medium => DiagnosticSeverity::WARNING,
                crate::Severity::Low => DiagnosticSeverity::INFORMATION,
                crate::Severity::Info => DiagnosticSeverity::HINT,
            }),
            code: Some(NumberOrString::String(diag.rule_id.clone())),
            source: Some("wix-analyzer".to_string()),
            message: diag.message.clone(),
            ..Default::default()
        }
    }

    fn get_indent(&self, content: &str, line: usize) -> String {
        let line_content = content.lines().nth(line.saturating_sub(1)).unwrap_or("");
        let trimmed = line_content.trim_start();
        line_content[..line_content.len() - trimmed.len()].to_string()
    }

    fn get_element_name_at_line(&self, content: &str, line: usize) -> Option<String> {
        let line_content = content.lines().nth(line.saturating_sub(1))?;
        let start = line_content.find('<')? + 1;
        let rest = &line_content[start..];
        let end = rest.find(|c: char| c.is_whitespace() || c == '>' || c == '/')?;
        Some(rest[..end].to_string())
    }
}

impl Default for CodeActionProvider {
    fn default() -> Self {
        Self::new()
    }
}

/// Generate a new GUID
fn generate_guid() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let random = time ^ (time >> 32);

    format!(
        "{:08X}-{:04X}-{:04X}-{:04X}-{:012X}",
        (random & 0xFFFFFFFF) as u32,
        ((random >> 32) & 0xFFFF) as u16,
        ((random >> 48) & 0x0FFF) as u16 | 0x4000, // Version 4
        ((random >> 60) & 0x3FFF) as u16 | 0x8000, // Variant 1
        (random >> 64) as u64 & 0xFFFFFFFFFFFF
    )
}

/// Generate an Id based on element type
fn generate_id(element: &str) -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis();

    let prefix = match element {
        "Component" => "Cmp",
        "Directory" => "Dir",
        "Feature" => "Feat",
        "Property" => "PROP",
        "CustomAction" => "CA",
        "File" => "File",
        _ => element,
    };

    format!("{}_{:X}", prefix, (time & 0xFFFFFF) as u32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_guid() {
        let guid = generate_guid();
        assert_eq!(guid.len(), 36);
        assert!(guid.contains('-'));

        // Check version (4) and variant (8, 9, A, B)
        let parts: Vec<&str> = guid.split('-').collect();
        assert_eq!(parts.len(), 5);
        assert!(parts[2].starts_with('4'));
    }

    #[test]
    fn test_generate_id() {
        let id = generate_id("Component");
        assert!(id.starts_with("Cmp_"));

        let id = generate_id("Directory");
        assert!(id.starts_with("Dir_"));

        let id = generate_id("Feature");
        assert!(id.starts_with("Feat_"));
    }

    #[test]
    fn test_code_action_provider_new() {
        let _provider = CodeActionProvider::new();
        let _default_provider = CodeActionProvider::default();
        // Just verify they can be created
    }

    #[test]
    fn test_get_element_name_at_line() {
        let provider = CodeActionProvider::new();
        let content = "<Wix>\n  <Component Id=\"Test\" />\n</Wix>";

        assert_eq!(provider.get_element_name_at_line(content, 1), Some("Wix".to_string()));
        assert_eq!(provider.get_element_name_at_line(content, 2), Some("Component".to_string()));
    }

    #[test]
    fn test_get_indent() {
        let provider = CodeActionProvider::new();
        let content = "<Wix>\n    <Component Id=\"Test\" />\n</Wix>";

        assert_eq!(provider.get_indent(content, 1), "");
        assert_eq!(provider.get_indent(content, 2), "    ");
    }
}
