//! WiX Language Server implementation

use dashmap::DashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer};

use crate::core::{SymbolIndex, WixDocument, SymbolAtPosition, extract_from_source, symbol_at_position};
use crate::{analyze_with_source, Config, Diagnostic as WixDiagnostic, Severity};

use super::actions::CodeActionProvider;

/// Document state stored by the server
#[derive(Debug, Clone)]
struct DocumentState {
    /// The document content
    content: String,
    /// Current diagnostics
    diagnostics: Vec<WixDiagnostic>,
}

/// WiX Language Server
pub struct WixLanguageServer {
    /// LSP client for sending notifications
    client: Client,
    /// Open documents
    documents: DashMap<Url, DocumentState>,
    /// Symbol index for cross-file analysis
    index: Arc<tokio::sync::RwLock<SymbolIndex>>,
    /// Configuration
    config: Arc<tokio::sync::RwLock<Config>>,
    /// Code action provider
    action_provider: CodeActionProvider,
}

impl WixLanguageServer {
    pub fn new(client: Client) -> Self {
        Self {
            client,
            documents: DashMap::new(),
            index: Arc::new(tokio::sync::RwLock::new(SymbolIndex::new())),
            config: Arc::new(tokio::sync::RwLock::new(Config::default())),
            action_provider: CodeActionProvider::new(),
        }
    }

    /// Analyze a document and publish diagnostics
    async fn analyze_document(&self, uri: &Url, content: &str) {
        let path = uri_to_path(uri);

        // Update index
        {
            let mut index = self.index.write().await;
            let _ = index.index_source(content, &path);
        }

        // Parse and run analysis
        let diagnostics = match WixDocument::parse(content, &path) {
            Ok(doc) => {
                let index = self.index.read().await;
                let config = self.config.read().await;
                let result = analyze_with_source(&doc, &index, &config, Some(content));
                result.diagnostics
            }
            Err(_) => Vec::new(),
        };

        // Store state
        self.documents.insert(
            uri.clone(),
            DocumentState {
                content: content.to_string(),
                diagnostics: diagnostics.clone(),
            },
        );

        // Publish diagnostics
        let lsp_diagnostics: Vec<Diagnostic> = diagnostics
            .iter()
            .map(|d| wix_diagnostic_to_lsp(d))
            .collect();

        self.client
            .publish_diagnostics(uri.clone(), lsp_diagnostics, None)
            .await;
    }

    /// Get diagnostics at a specific range
    fn get_diagnostics_in_range(&self, uri: &Url, range: &Range) -> Vec<WixDiagnostic> {
        if let Some(state) = self.documents.get(uri) {
            state
                .diagnostics
                .iter()
                .filter(|d| ranges_overlap(range, &wix_range_to_lsp(&d.location.range)))
                .cloned()
                .collect()
        } else {
            Vec::new()
        }
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for WixLanguageServer {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                diagnostic_provider: Some(DiagnosticServerCapabilities::Options(
                    DiagnosticOptions {
                        identifier: Some("wix-analyzer".to_string()),
                        inter_file_dependencies: true,
                        workspace_diagnostics: true,
                        work_done_progress_options: WorkDoneProgressOptions::default(),
                    },
                )),
                code_action_provider: Some(CodeActionProviderCapability::Options(
                    CodeActionOptions {
                        code_action_kinds: Some(vec![
                            CodeActionKind::QUICKFIX,
                            CodeActionKind::REFACTOR,
                            CodeActionKind::SOURCE,
                        ]),
                        work_done_progress_options: WorkDoneProgressOptions::default(),
                        resolve_provider: Some(false),
                    },
                )),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                completion_provider: Some(CompletionOptions {
                    trigger_characters: Some(vec!["<".to_string(), " ".to_string(), "\"".to_string()]),
                    ..Default::default()
                }),
                definition_provider: Some(OneOf::Left(true)),
                references_provider: Some(OneOf::Left(true)),
                document_symbol_provider: Some(OneOf::Left(true)),
                ..Default::default()
            },
            server_info: Some(ServerInfo {
                name: "wix-analyzer".to_string(),
                version: Some(env!("CARGO_PKG_VERSION").to_string()),
            }),
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "WiX Analyzer LSP initialized")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri;
        let content = params.text_document.text;
        self.analyze_document(&uri, &content).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri;
        if let Some(change) = params.content_changes.into_iter().next() {
            self.analyze_document(&uri, &change.text).await;
        }
    }

    async fn did_save(&self, params: DidSaveTextDocumentParams) {
        if let Some(content) = params.text {
            self.analyze_document(&params.text_document.uri, &content).await;
        }
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        self.documents.remove(&params.text_document.uri);
        // Clear diagnostics
        self.client
            .publish_diagnostics(params.text_document.uri, vec![], None)
            .await;
    }

    async fn code_action(&self, params: CodeActionParams) -> Result<Option<CodeActionResponse>> {
        let uri = &params.text_document.uri;

        // Get diagnostics in range
        let diagnostics = self.get_diagnostics_in_range(uri, &params.range);

        if diagnostics.is_empty() {
            return Ok(None);
        }

        let content = if let Some(state) = self.documents.get(uri) {
            state.content.clone()
        } else {
            return Ok(None);
        };

        let actions = self.action_provider.get_actions(uri, &diagnostics, &content);

        if actions.is_empty() {
            Ok(None)
        } else {
            Ok(Some(actions))
        }
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let uri = &params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;

        // Get diagnostics at position for hover info
        let range = Range {
            start: position,
            end: Position {
                line: position.line,
                character: position.character + 1,
            },
        };
        let diagnostics = self.get_diagnostics_in_range(uri, &range);

        if diagnostics.is_empty() {
            return Ok(None);
        }

        let mut contents = Vec::new();
        for diag in diagnostics {
            let severity_emoji = match diag.severity {
                Severity::Blocker => "🚫",
                Severity::High => "❌",
                Severity::Medium => "⚠️",
                Severity::Low => "💡",
                Severity::Info => "ℹ️",
            };

            let mut text = format!(
                "{} **{}**: {}\n\n",
                severity_emoji, diag.rule_id, diag.message
            );

            if let Some(ref help) = diag.help {
                text.push_str(&format!("**Help**: {}\n\n", help));
            }

            if let Some(ref url) = diag.doc_url {
                text.push_str(&format!("[Documentation]({})\n", url));
            }

            contents.push(MarkedString::String(text));
        }

        Ok(Some(Hover {
            contents: HoverContents::Array(contents),
            range: None,
        }))
    }

    async fn goto_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> Result<Option<GotoDefinitionResponse>> {
        let uri = &params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;

        let content = if let Some(state) = self.documents.get(uri) {
            state.content.clone()
        } else {
            return Ok(None);
        };

        let path = uri_to_path(uri);

        // Parse document
        let Ok(doc) = WixDocument::parse(&content, &path) else {
            return Ok(None);
        };

        // Find symbol at position (1-based line/column for our API)
        if let Some(symbol) = symbol_at_position(
            &doc,
            position.line as usize + 1,
            position.character as usize + 1,
        ) {
            // Only handle references - going to definition
            if let SymbolAtPosition::Reference { id, kind, .. } = symbol {
                let index = self.index.read().await;
                let element_type = kind.definition_element();
                if let Some(def) = index.get_definition(element_type, &id) {
                    return Ok(Some(GotoDefinitionResponse::Scalar(Location {
                        uri: path_to_uri(&def.location.file),
                        range: wix_range_to_lsp(&def.location.range),
                    })));
                }
            }
        }

        Ok(None)
    }

    async fn references(&self, params: ReferenceParams) -> Result<Option<Vec<Location>>> {
        let uri = &params.text_document_position.text_document.uri;
        let position = params.text_document_position.position;

        let content = if let Some(state) = self.documents.get(uri) {
            state.content.clone()
        } else {
            return Ok(None);
        };

        let path = uri_to_path(uri);

        // Parse document
        let Ok(doc) = WixDocument::parse(&content, &path) else {
            return Ok(None);
        };

        // Find symbol at position
        if let Some(symbol) = symbol_at_position(
            &doc,
            position.line as usize + 1,
            position.character as usize + 1,
        ) {
            let (id, element_type) = match &symbol {
                SymbolAtPosition::Definition { id, kind, .. } => {
                    (id.clone(), kind.canonical_type().to_string())
                }
                SymbolAtPosition::Reference { id, kind, .. } => {
                    (id.clone(), kind.definition_element().to_string())
                }
            };

            let index = self.index.read().await;

            // Get definition to find references
            if let Some(def) = index.get_definition(&element_type, &id) {
                let refs = index.find_references(def);

                if refs.is_empty() {
                    return Ok(None);
                }

                let locations: Vec<Location> = refs
                    .iter()
                    .map(|r| Location {
                        uri: path_to_uri(&r.location.file),
                        range: wix_range_to_lsp(&r.location.range),
                    })
                    .collect();

                return Ok(Some(locations));
            }
        }

        Ok(None)
    }

    async fn document_symbol(
        &self,
        params: DocumentSymbolParams,
    ) -> Result<Option<DocumentSymbolResponse>> {
        let uri = &params.text_document.uri;

        let content = if let Some(state) = self.documents.get(uri) {
            state.content.clone()
        } else {
            return Ok(None);
        };

        let path = uri_to_path(uri);

        // Extract symbols from document
        let extraction = match extract_from_source(&content, &path) {
            Ok(e) => e,
            Err(_) => return Ok(None),
        };

        let symbols: Vec<SymbolInformation> = extraction
            .definitions
            .iter()
            .map(|def| {
                #[allow(deprecated)]
                SymbolInformation {
                    name: def.id.clone(),
                    kind: def_kind_to_symbol_kind(&def.kind),
                    tags: None,
                    deprecated: None,
                    location: Location {
                        uri: uri.clone(),
                        range: wix_range_to_lsp(&def.location.range),
                    },
                    container_name: def.detail.clone(),
                }
            })
            .collect();

        if symbols.is_empty() {
            Ok(None)
        } else {
            Ok(Some(DocumentSymbolResponse::Flat(symbols)))
        }
    }
}

/// Convert a URI to a PathBuf
fn uri_to_path(uri: &Url) -> PathBuf {
    uri.to_file_path().unwrap_or_else(|_| PathBuf::from(uri.path()))
}

/// Convert a PathBuf to a URI
fn path_to_uri(path: &std::path::Path) -> Url {
    Url::from_file_path(path).unwrap_or_else(|_| Url::parse(&format!("file://{}", path.display())).unwrap())
}

/// Convert WiX diagnostic to LSP diagnostic
fn wix_diagnostic_to_lsp(diag: &WixDiagnostic) -> Diagnostic {
    Diagnostic {
        range: wix_range_to_lsp(&diag.location.range),
        severity: Some(wix_severity_to_lsp(diag.severity)),
        code: Some(NumberOrString::String(diag.rule_id.clone())),
        code_description: diag.doc_url.as_ref().map(|url| CodeDescription {
            href: Url::parse(url).unwrap_or_else(|_| Url::parse("https://example.com").unwrap()),
        }),
        source: Some("wix-analyzer".to_string()),
        message: diag.message.clone(),
        related_information: if diag.related.is_empty() {
            None
        } else {
            Some(
                diag.related
                    .iter()
                    .map(|r| DiagnosticRelatedInformation {
                        location: Location {
                            uri: path_to_uri(&r.location.file),
                            range: wix_range_to_lsp(&r.location.range),
                        },
                        message: r.message.clone(),
                    })
                    .collect(),
            )
        },
        tags: None,
        data: None,
    }
}

/// Convert WiX range to LSP range
fn wix_range_to_lsp(range: &crate::core::Range) -> Range {
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

/// Convert WiX severity to LSP severity
fn wix_severity_to_lsp(severity: Severity) -> DiagnosticSeverity {
    match severity {
        Severity::Blocker | Severity::High => DiagnosticSeverity::ERROR,
        Severity::Medium => DiagnosticSeverity::WARNING,
        Severity::Low => DiagnosticSeverity::INFORMATION,
        Severity::Info => DiagnosticSeverity::HINT,
    }
}

/// Convert definition kind to LSP symbol kind
fn def_kind_to_symbol_kind(kind: &crate::core::DefinitionKind) -> SymbolKind {
    use crate::core::DefinitionKind;
    match kind {
        DefinitionKind::Component | DefinitionKind::ComponentGroup => SymbolKind::CLASS,
        DefinitionKind::Directory | DefinitionKind::StandardDirectory => SymbolKind::NAMESPACE,
        DefinitionKind::Feature | DefinitionKind::FeatureGroup => SymbolKind::MODULE,
        DefinitionKind::Property => SymbolKind::PROPERTY,
        DefinitionKind::CustomAction => SymbolKind::FUNCTION,
        DefinitionKind::Binary => SymbolKind::FILE,
        DefinitionKind::Fragment => SymbolKind::PACKAGE,
        DefinitionKind::Package | DefinitionKind::Module | DefinitionKind::Bundle => SymbolKind::PACKAGE,
    }
}

/// Check if two ranges overlap
fn ranges_overlap(a: &Range, b: &Range) -> bool {
    !(a.end.line < b.start.line
        || (a.end.line == b.start.line && a.end.character < b.start.character)
        || b.end.line < a.start.line
        || (b.end.line == a.start.line && b.end.character < a.start.character))
}

/// Run the language server
pub async fn run_server() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = tower_lsp::LspService::new(|client| WixLanguageServer::new(client));
    tower_lsp::Server::new(stdin, stdout, socket)
        .serve(service)
        .await;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wix_range_to_lsp() {
        let wix_range = crate::core::Range::new(
            crate::core::Position::new(1, 1),
            crate::core::Position::new(1, 10),
        );
        let lsp_range = wix_range_to_lsp(&wix_range);

        assert_eq!(lsp_range.start.line, 0);
        assert_eq!(lsp_range.start.character, 0);
        assert_eq!(lsp_range.end.line, 0);
        assert_eq!(lsp_range.end.character, 9);
    }

    #[test]
    fn test_wix_severity_to_lsp() {
        assert_eq!(wix_severity_to_lsp(Severity::Blocker), DiagnosticSeverity::ERROR);
        assert_eq!(wix_severity_to_lsp(Severity::High), DiagnosticSeverity::ERROR);
        assert_eq!(wix_severity_to_lsp(Severity::Medium), DiagnosticSeverity::WARNING);
        assert_eq!(wix_severity_to_lsp(Severity::Low), DiagnosticSeverity::INFORMATION);
        assert_eq!(wix_severity_to_lsp(Severity::Info), DiagnosticSeverity::HINT);
    }

    #[test]
    fn test_ranges_overlap() {
        let a = Range {
            start: Position { line: 1, character: 0 },
            end: Position { line: 1, character: 10 },
        };
        let b = Range {
            start: Position { line: 1, character: 5 },
            end: Position { line: 1, character: 15 },
        };
        assert!(ranges_overlap(&a, &b));

        let c = Range {
            start: Position { line: 2, character: 0 },
            end: Position { line: 2, character: 10 },
        };
        assert!(!ranges_overlap(&a, &c));
    }

    #[test]
    fn test_uri_to_path() {
        let uri = Url::parse("file:///home/user/test.wxs").unwrap();
        let path = uri_to_path(&uri);
        assert!(path.to_string_lossy().contains("test.wxs"));
    }

    #[test]
    fn test_def_kind_to_symbol_kind() {
        use crate::core::DefinitionKind;
        assert_eq!(def_kind_to_symbol_kind(&DefinitionKind::Component), SymbolKind::CLASS);
        assert_eq!(def_kind_to_symbol_kind(&DefinitionKind::Directory), SymbolKind::NAMESPACE);
        assert_eq!(def_kind_to_symbol_kind(&DefinitionKind::Feature), SymbolKind::MODULE);
        assert_eq!(def_kind_to_symbol_kind(&DefinitionKind::Property), SymbolKind::PROPERTY);
        assert_eq!(def_kind_to_symbol_kind(&DefinitionKind::CustomAction), SymbolKind::FUNCTION);
    }
}
