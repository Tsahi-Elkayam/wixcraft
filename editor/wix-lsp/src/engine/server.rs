//! Generic LSP server implementation
//!
//! This server delegates to registered plugins based on file type.

use super::config::EngineConfig;
use super::convert;
use super::document::DocumentManager;
use crate::plugins::PluginRegistry;
use std::path::PathBuf;
use std::sync::Arc;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer};

/// Generic Language Server
pub struct LspServer {
    /// LSP client for sending notifications
    client: Client,
    /// Document manager
    documents: DocumentManager,
    /// Plugin registry
    plugins: Arc<PluginRegistry>,
    /// Engine configuration
    config: EngineConfig,
    /// Data path (discovered or configured)
    #[allow(dead_code)]
    data_path: Option<PathBuf>,
}

impl LspServer {
    /// Create a new LSP server with the given plugins
    pub fn new(client: Client, plugins: PluginRegistry) -> Self {
        Self {
            client,
            documents: DocumentManager::new(),
            plugins: Arc::new(plugins),
            config: EngineConfig::default(),
            data_path: None,
        }
    }

    /// Create with configuration
    pub fn with_config(client: Client, plugins: PluginRegistry, config: EngineConfig) -> Self {
        Self {
            client,
            documents: DocumentManager::new(),
            plugins: Arc::new(plugins),
            config,
            data_path: None,
        }
    }

    /// Get server capabilities based on registered plugins
    pub fn capabilities(&self) -> ServerCapabilities {
        let trigger_chars = self.plugins.all_trigger_characters();

        ServerCapabilities {
            text_document_sync: Some(TextDocumentSyncCapability::Options(
                TextDocumentSyncOptions {
                    open_close: Some(true),
                    change: Some(TextDocumentSyncKind::FULL),
                    will_save: None,
                    will_save_wait_until: None,
                    save: None,
                },
            )),

            completion_provider: Some(CompletionOptions {
                trigger_characters: if trigger_chars.is_empty() {
                    None
                } else {
                    Some(trigger_chars)
                },
                resolve_provider: Some(false),
                work_done_progress_options: WorkDoneProgressOptions::default(),
                all_commit_characters: None,
                completion_item: None,
            }),

            hover_provider: Some(HoverProviderCapability::Simple(true)),
            document_symbol_provider: Some(OneOf::Left(true)),
            document_formatting_provider: Some(OneOf::Left(true)),

            // Not yet implemented
            definition_provider: None,
            references_provider: None,
            signature_help_provider: None,
            declaration_provider: None,
            type_definition_provider: None,
            implementation_provider: None,
            code_action_provider: None,
            code_lens_provider: None,
            document_highlight_provider: None,
            workspace_symbol_provider: None,
            execute_command_provider: None,
            workspace: None,
            selection_range_provider: None,
            rename_provider: None,
            document_range_formatting_provider: None,
            document_on_type_formatting_provider: None,
            folding_range_provider: None,
            linked_editing_range_provider: None,
            call_hierarchy_provider: None,
            semantic_tokens_provider: None,
            moniker_provider: None,
            inlay_hint_provider: None,
            inline_value_provider: None,
            color_provider: None,
            document_link_provider: None,
            diagnostic_provider: None,
            experimental: None,
            position_encoding: None,
        }
    }

    /// Publish diagnostics for a document
    async fn publish_diagnostics(&self, uri: &Url, content: &str) {
        if let Some(plugin) = self.plugins.plugin_for_uri(uri.as_str()) {
            let path = uri
                .to_file_path()
                .unwrap_or_else(|_| PathBuf::from("document.wxs"));

            let diagnostics = plugin.as_diagnostic().diagnose(content, &path);
            let lsp_diagnostics: Vec<_> = diagnostics
                .iter()
                .map(convert::to_lsp_diagnostic)
                .collect();

            self.client
                .publish_diagnostics(uri.clone(), lsp_diagnostics, None)
                .await;
        }
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for LspServer {
    async fn initialize(&self, params: InitializeParams) -> Result<InitializeResult> {
        tracing::info!("{} initializing", self.config.engine.name);

        // Try to find data path from workspace
        if self.config.engine.workspace_discovery {
            if let Some(root_uri) = params.root_uri {
                if let Ok(root_path) = root_uri.to_file_path() {
                    if let Some(data_path) = self.config.find_data_path(&root_path) {
                        tracing::info!("Found data at: {}", data_path.display());
                    }
                }
            }
        }

        Ok(InitializeResult {
            capabilities: self.capabilities(),
            server_info: Some(ServerInfo {
                name: self.config.engine.name.clone(),
                version: Some(env!("CARGO_PKG_VERSION").to_string()),
            }),
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        tracing::info!("{} initialized", self.config.engine.name);
        self.client
            .log_message(
                MessageType::INFO,
                format!("{} ready", self.config.engine.name),
            )
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        tracing::info!("{} shutting down", self.config.engine.name);
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri;
        let content = params.text_document.text;
        let version = params.text_document.version;

        tracing::debug!("Document opened: {}", uri);
        self.documents.open(uri.clone(), content.clone(), version);
        self.publish_diagnostics(&uri, &content).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri;
        let version = params.text_document.version;

        if let Some(change) = params.content_changes.into_iter().next() {
            self.documents.update(&uri, change.text.clone(), version);
            self.publish_diagnostics(&uri, &change.text).await;
        }
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        let uri = params.text_document.uri;
        tracing::debug!("Document closed: {}", uri);
        self.documents.close(&uri);
        self.client.publish_diagnostics(uri, vec![], None).await;
    }

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        let uri = &params.text_document_position.text_document.uri;
        let position = params.text_document_position.position;

        if let Some(plugin) = self.plugins.plugin_for_uri(uri.as_str()) {
            if let Some(content) = self.documents.get_content(uri) {
                let completions = plugin.as_completion().complete(
                    &content,
                    position.line + 1,
                    position.character + 1,
                );

                let items: Vec<_> = completions
                    .iter()
                    .map(convert::to_lsp_completion)
                    .collect();

                return Ok(Some(CompletionResponse::Array(items)));
            }
        }

        Ok(None)
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let uri = &params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;

        if let Some(plugin) = self.plugins.plugin_for_uri(uri.as_str()) {
            if let Some(content) = self.documents.get_content(uri) {
                if let Some(info) = plugin
                    .as_hover()
                    .hover(&content, position.line + 1, position.character + 1)
                {
                    return Ok(Some(convert::to_lsp_hover(&info)));
                }
            }
        }

        Ok(None)
    }

    async fn document_symbol(
        &self,
        params: DocumentSymbolParams,
    ) -> Result<Option<DocumentSymbolResponse>> {
        let uri = &params.text_document.uri;

        if let Some(plugin) = self.plugins.plugin_for_uri(uri.as_str()) {
            if let Some(content) = self.documents.get_content(uri) {
                match plugin.as_symbol().symbols(&content) {
                    Ok(symbols) => {
                        let lsp_symbols: Vec<_> = symbols
                            .iter()
                            .map(convert::to_lsp_document_symbol)
                            .collect();

                        return Ok(Some(DocumentSymbolResponse::Nested(lsp_symbols)));
                    }
                    Err(e) => {
                        tracing::warn!("Failed to extract symbols: {}", e);
                    }
                }
            }
        }

        Ok(None)
    }

    async fn formatting(&self, params: DocumentFormattingParams) -> Result<Option<Vec<TextEdit>>> {
        let uri = &params.text_document.uri;

        if let Some(plugin) = self.plugins.plugin_for_uri(uri.as_str()) {
            if let Some(content) = self.documents.get_content(uri) {
                match plugin.as_format().format(&content) {
                    Ok(formatted) => {
                        let lines: Vec<_> = content.lines().collect();
                        let last_line = lines.len().saturating_sub(1);
                        let last_col = lines.last().map(|l| l.len()).unwrap_or(0);

                        let edit = TextEdit {
                            range: Range {
                                start: Position {
                                    line: 0,
                                    character: 0,
                                },
                                end: Position {
                                    line: last_line as u32,
                                    character: last_col as u32,
                                },
                            },
                            new_text: formatted,
                        };

                        return Ok(Some(vec![edit]));
                    }
                    Err(e) => {
                        tracing::warn!("Failed to format document: {}", e);
                    }
                }
            }
        }

        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_capabilities_empty_plugins() {
        // Can't easily test without a Client, but we can test the config
        let config = EngineConfig::default();
        assert_eq!(config.engine.name, "wix-lsp");
    }
}
