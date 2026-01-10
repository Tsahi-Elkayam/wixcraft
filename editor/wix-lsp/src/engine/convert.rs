//! Type conversions from plugin types to LSP types

use crate::plugins::{
    Completion, CompletionKind, Diagnostic, DiagnosticSeverity, HoverInfo, Symbol, SymbolKind,
    SymbolRange,
};
use tower_lsp::lsp_types::{
    self, CompletionItem as LspCompletionItem, CompletionItemKind as LspCompletionItemKind,
    Diagnostic as LspDiagnostic, DiagnosticSeverity as LspDiagnosticSeverity,
    DocumentSymbol as LspDocumentSymbol, Hover, HoverContents, MarkupContent, MarkupKind,
    Position as LspPosition, Range as LspRange, SymbolKind as LspSymbolKind,
};

/// Convert plugin completion to LSP completion item
pub fn to_lsp_completion(item: &Completion) -> LspCompletionItem {
    LspCompletionItem {
        label: item.label.clone(),
        kind: Some(to_lsp_completion_kind(&item.kind)),
        detail: item.detail.clone(),
        documentation: item.documentation.as_ref().map(|doc| {
            lsp_types::Documentation::MarkupContent(MarkupContent {
                kind: MarkupKind::Markdown,
                value: doc.clone(),
            })
        }),
        insert_text: Some(item.insert_text.clone()),
        insert_text_format: Some(lsp_types::InsertTextFormat::SNIPPET),
        insert_text_mode: None,
        sort_text: Some(format!("{:05}", item.sort_priority)),
        filter_text: None,
        deprecated: Some(false),
        preselect: None,
        additional_text_edits: None,
        command: None,
        commit_characters: None,
        data: None,
        tags: None,
        label_details: None,
        text_edit: None,
    }
}

/// Convert plugin completion kind to LSP completion kind
fn to_lsp_completion_kind(kind: &CompletionKind) -> LspCompletionItemKind {
    match kind {
        CompletionKind::Element => LspCompletionItemKind::CLASS,
        CompletionKind::Attribute => LspCompletionItemKind::PROPERTY,
        CompletionKind::Value => LspCompletionItemKind::VALUE,
        CompletionKind::Snippet => LspCompletionItemKind::SNIPPET,
        CompletionKind::Directory => LspCompletionItemKind::FOLDER,
        CompletionKind::Property => LspCompletionItemKind::VARIABLE,
        CompletionKind::Keyword => LspCompletionItemKind::KEYWORD,
    }
}

/// Convert plugin symbol to LSP document symbol
pub fn to_lsp_document_symbol(symbol: &Symbol) -> LspDocumentSymbol {
    let range = to_lsp_range(&symbol.range);
    let selection_range = to_lsp_range(&symbol.selection_range);

    #[allow(deprecated)]
    LspDocumentSymbol {
        name: symbol.name.clone(),
        detail: symbol.detail.clone(),
        kind: to_lsp_symbol_kind(&symbol.kind),
        range,
        selection_range,
        children: if symbol.children.is_empty() {
            None
        } else {
            Some(symbol.children.iter().map(to_lsp_document_symbol).collect())
        },
        tags: None,
        deprecated: None,
    }
}

/// Convert plugin symbol kind to LSP symbol kind
fn to_lsp_symbol_kind(kind: &SymbolKind) -> LspSymbolKind {
    match kind {
        SymbolKind::File => LspSymbolKind::FILE,
        SymbolKind::Module => LspSymbolKind::MODULE,
        SymbolKind::Namespace => LspSymbolKind::NAMESPACE,
        SymbolKind::Class => LspSymbolKind::CLASS,
        SymbolKind::Function => LspSymbolKind::FUNCTION,
        SymbolKind::Variable => LspSymbolKind::VARIABLE,
        SymbolKind::Constant => LspSymbolKind::CONSTANT,
        SymbolKind::String => LspSymbolKind::STRING,
        SymbolKind::Property => LspSymbolKind::PROPERTY,
        SymbolKind::Key => LspSymbolKind::KEY,
        SymbolKind::Struct => LspSymbolKind::STRUCT,
        SymbolKind::Event => LspSymbolKind::EVENT,
        SymbolKind::Operator => LspSymbolKind::OPERATOR,
        SymbolKind::TypeParameter => LspSymbolKind::TYPE_PARAMETER,
    }
}

/// Convert plugin range to LSP range
pub fn to_lsp_range(range: &SymbolRange) -> LspRange {
    LspRange {
        start: LspPosition {
            line: range.start_line,
            character: range.start_col,
        },
        end: LspPosition {
            line: range.end_line,
            character: range.end_col,
        },
    }
}

/// Convert plugin diagnostic to LSP diagnostic
pub fn to_lsp_diagnostic(diag: &Diagnostic) -> LspDiagnostic {
    LspDiagnostic {
        range: LspRange {
            start: LspPosition {
                line: diag.line.saturating_sub(1),
                character: diag.column.saturating_sub(1),
            },
            end: LspPosition {
                line: diag.line.saturating_sub(1),
                character: diag.column.saturating_sub(1) + diag.length,
            },
        },
        severity: Some(to_lsp_severity(&diag.severity)),
        code: Some(lsp_types::NumberOrString::String(diag.rule_id.clone())),
        code_description: None,
        source: Some("wix-lsp".to_string()),
        message: diag.message.clone(),
        related_information: None,
        tags: None,
        data: None,
    }
}

/// Convert plugin severity to LSP severity
fn to_lsp_severity(severity: &DiagnosticSeverity) -> LspDiagnosticSeverity {
    match severity {
        DiagnosticSeverity::Error => LspDiagnosticSeverity::ERROR,
        DiagnosticSeverity::Warning => LspDiagnosticSeverity::WARNING,
        DiagnosticSeverity::Info => LspDiagnosticSeverity::INFORMATION,
        DiagnosticSeverity::Hint => LspDiagnosticSeverity::HINT,
    }
}

/// Convert plugin hover info to LSP hover
pub fn to_lsp_hover(info: &HoverInfo) -> Hover {
    let range = info.range.as_ref().map(|r| LspRange {
        start: LspPosition {
            line: r.start_line,
            character: r.start_col,
        },
        end: LspPosition {
            line: r.end_line,
            character: r.end_col,
        },
    });

    Hover {
        contents: HoverContents::Markup(MarkupContent {
            kind: MarkupKind::Markdown,
            value: info.contents.clone(),
        }),
        range,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugins::HoverRange;

    #[test]
    fn test_to_lsp_completion() {
        let item = Completion {
            label: "Component".to_string(),
            kind: CompletionKind::Element,
            detail: Some("WiX element".to_string()),
            documentation: Some("A component is...".to_string()),
            insert_text: "<Component />".to_string(),
            sort_priority: 0,
        };

        let lsp_item = to_lsp_completion(&item);
        assert_eq!(lsp_item.label, "Component");
        assert_eq!(lsp_item.kind, Some(LspCompletionItemKind::CLASS));
    }

    #[test]
    fn test_to_lsp_completion_kinds() {
        assert_eq!(
            to_lsp_completion_kind(&CompletionKind::Element),
            LspCompletionItemKind::CLASS
        );
        assert_eq!(
            to_lsp_completion_kind(&CompletionKind::Attribute),
            LspCompletionItemKind::PROPERTY
        );
        assert_eq!(
            to_lsp_completion_kind(&CompletionKind::Value),
            LspCompletionItemKind::VALUE
        );
        assert_eq!(
            to_lsp_completion_kind(&CompletionKind::Snippet),
            LspCompletionItemKind::SNIPPET
        );
        assert_eq!(
            to_lsp_completion_kind(&CompletionKind::Directory),
            LspCompletionItemKind::FOLDER
        );
        assert_eq!(
            to_lsp_completion_kind(&CompletionKind::Property),
            LspCompletionItemKind::VARIABLE
        );
        assert_eq!(
            to_lsp_completion_kind(&CompletionKind::Keyword),
            LspCompletionItemKind::KEYWORD
        );
    }

    #[test]
    fn test_to_lsp_symbol_kinds() {
        assert_eq!(to_lsp_symbol_kind(&SymbolKind::File), LspSymbolKind::FILE);
        assert_eq!(
            to_lsp_symbol_kind(&SymbolKind::Module),
            LspSymbolKind::MODULE
        );
        assert_eq!(
            to_lsp_symbol_kind(&SymbolKind::Namespace),
            LspSymbolKind::NAMESPACE
        );
        assert_eq!(to_lsp_symbol_kind(&SymbolKind::Class), LspSymbolKind::CLASS);
        assert_eq!(
            to_lsp_symbol_kind(&SymbolKind::Function),
            LspSymbolKind::FUNCTION
        );
        assert_eq!(
            to_lsp_symbol_kind(&SymbolKind::Variable),
            LspSymbolKind::VARIABLE
        );
        assert_eq!(
            to_lsp_symbol_kind(&SymbolKind::Constant),
            LspSymbolKind::CONSTANT
        );
        assert_eq!(
            to_lsp_symbol_kind(&SymbolKind::String),
            LspSymbolKind::STRING
        );
        assert_eq!(
            to_lsp_symbol_kind(&SymbolKind::Property),
            LspSymbolKind::PROPERTY
        );
        assert_eq!(to_lsp_symbol_kind(&SymbolKind::Key), LspSymbolKind::KEY);
        assert_eq!(
            to_lsp_symbol_kind(&SymbolKind::Struct),
            LspSymbolKind::STRUCT
        );
        assert_eq!(to_lsp_symbol_kind(&SymbolKind::Event), LspSymbolKind::EVENT);
        assert_eq!(
            to_lsp_symbol_kind(&SymbolKind::Operator),
            LspSymbolKind::OPERATOR
        );
        assert_eq!(
            to_lsp_symbol_kind(&SymbolKind::TypeParameter),
            LspSymbolKind::TYPE_PARAMETER
        );
    }

    #[test]
    fn test_to_lsp_range() {
        let range = SymbolRange {
            start_line: 10,
            start_col: 5,
            end_line: 15,
            end_col: 20,
        };

        let lsp_range = to_lsp_range(&range);
        assert_eq!(lsp_range.start.line, 10);
        assert_eq!(lsp_range.start.character, 5);
        assert_eq!(lsp_range.end.line, 15);
        assert_eq!(lsp_range.end.character, 20);
    }

    #[test]
    fn test_to_lsp_severity() {
        assert_eq!(
            to_lsp_severity(&DiagnosticSeverity::Error),
            LspDiagnosticSeverity::ERROR
        );
        assert_eq!(
            to_lsp_severity(&DiagnosticSeverity::Warning),
            LspDiagnosticSeverity::WARNING
        );
        assert_eq!(
            to_lsp_severity(&DiagnosticSeverity::Info),
            LspDiagnosticSeverity::INFORMATION
        );
        assert_eq!(
            to_lsp_severity(&DiagnosticSeverity::Hint),
            LspDiagnosticSeverity::HINT
        );
    }

    #[test]
    fn test_to_lsp_diagnostic() {
        let diag = Diagnostic {
            rule_id: "TEST-001".to_string(),
            message: "Test message".to_string(),
            severity: DiagnosticSeverity::Error,
            line: 10,
            column: 5,
            length: 15,
            help: None,
        };

        let lsp_diag = to_lsp_diagnostic(&diag);
        assert_eq!(lsp_diag.message, "Test message");
        assert_eq!(lsp_diag.severity, Some(LspDiagnosticSeverity::ERROR));
        assert_eq!(lsp_diag.range.start.line, 9); // 1-indexed to 0-indexed
        assert_eq!(lsp_diag.range.start.character, 4);
    }

    #[test]
    fn test_to_lsp_hover() {
        let info = HoverInfo {
            contents: "# Component\n\nDescription".to_string(),
            range: Some(HoverRange {
                start_line: 5,
                start_col: 10,
                end_line: 5,
                end_col: 19,
            }),
        };

        let hover = to_lsp_hover(&info);
        assert!(hover.range.is_some());
        if let HoverContents::Markup(markup) = &hover.contents {
            assert_eq!(markup.kind, MarkupKind::Markdown);
            assert!(markup.value.contains("Component"));
        }
    }

    #[test]
    fn test_to_lsp_hover_no_range() {
        let info = HoverInfo {
            contents: "Simple text".to_string(),
            range: None,
        };

        let hover = to_lsp_hover(&info);
        assert!(hover.range.is_none());
    }

    #[test]
    fn test_to_lsp_document_symbol() {
        let symbol = Symbol {
            name: "MainComponent".to_string(),
            kind: SymbolKind::Struct,
            detail: Some("Main files".to_string()),
            range: SymbolRange {
                start_line: 10,
                start_col: 5,
                end_line: 15,
                end_col: 20,
            },
            selection_range: SymbolRange {
                start_line: 10,
                start_col: 15,
                end_line: 10,
                end_col: 28,
            },
            children: vec![],
        };

        let lsp_symbol = to_lsp_document_symbol(&symbol);
        assert_eq!(lsp_symbol.name, "MainComponent");
        assert_eq!(lsp_symbol.kind, LspSymbolKind::STRUCT);
        assert!(lsp_symbol.children.is_none());
    }

    #[test]
    fn test_to_lsp_document_symbol_with_children() {
        let child = Symbol {
            name: "File1".to_string(),
            kind: SymbolKind::File,
            detail: None,
            range: SymbolRange {
                start_line: 11,
                start_col: 10,
                end_line: 11,
                end_col: 30,
            },
            selection_range: SymbolRange {
                start_line: 11,
                start_col: 15,
                end_line: 11,
                end_col: 20,
            },
            children: vec![],
        };

        let symbol = Symbol {
            name: "MainComponent".to_string(),
            kind: SymbolKind::Struct,
            detail: None,
            range: SymbolRange {
                start_line: 10,
                start_col: 5,
                end_line: 15,
                end_col: 20,
            },
            selection_range: SymbolRange {
                start_line: 10,
                start_col: 15,
                end_line: 10,
                end_col: 28,
            },
            children: vec![child],
        };

        let lsp_symbol = to_lsp_document_symbol(&symbol);
        assert!(lsp_symbol.children.is_some());
        assert_eq!(lsp_symbol.children.as_ref().unwrap().len(), 1);
    }
}
