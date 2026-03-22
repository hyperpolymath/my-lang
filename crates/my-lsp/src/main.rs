// SPDX-License-Identifier: PMPL-1.0-or-later
//! My Language Language Server Protocol Implementation

use lsp_types::{
    Diagnostic, DiagnosticSeverity, DidChangeTextDocumentParams, DidOpenTextDocumentParams,
    InitializeParams, InitializeResult, MessageType, Position, Range, ServerCapabilities,
    TextDocumentSyncCapability, TextDocumentSyncKind, Url,
};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::ServerInfo;
use tower_lsp::{Client, LanguageServer, LspService, Server};

#[derive(Debug)]
struct Backend {
    client: Client,
    documents: Arc<RwLock<HashMap<Url, String>>>,
}

impl Backend {
    fn new(client: Client) -> Self {
        Backend {
            client,
            documents: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    async fn diagnose(&self, uri: Url, text: &str) {
        let diagnostics = match my_lang::parse(text) {
            Ok(program) => {
                // Type check
                if let Err(e) = my_lang::check(&program) {
                    vec![Diagnostic {
                        range: Range::new(Position::new(0, 0), Position::new(0, 0)),
                        severity: Some(DiagnosticSeverity::ERROR),
                        code: Some(lsp_types::NumberOrString::String("type-error".to_string())),
                        message: format!("Type error: {:?}", e),
                        ..Default::default()
                    }]
                } else {
                    // Run linter
                    my_lint::lint(&program)
                        .into_iter()
                        .map(|d| Diagnostic {
                            range: Range::new(
                                Position::new(d.line as u32, d.column as u32),
                                Position::new(d.line as u32, d.column as u32 + 1),
                            ),
                            severity: Some(match d.severity {
                                my_lint::Severity::Error => DiagnosticSeverity::ERROR,
                                my_lint::Severity::Warning => DiagnosticSeverity::WARNING,
                                my_lint::Severity::Info => DiagnosticSeverity::INFORMATION,
                                my_lint::Severity::Hint => DiagnosticSeverity::HINT,
                            }),
                            code: Some(lsp_types::NumberOrString::String(d.code)),
                            message: d.message,
                            ..Default::default()
                        })
                        .collect()
                }
            }
            Err(e) => {
                vec![Diagnostic {
                    range: Range::new(Position::new(0, 0), Position::new(0, 0)),
                    severity: Some(DiagnosticSeverity::ERROR),
                    code: Some(lsp_types::NumberOrString::String("parse-error".to_string())),
                    message: format!("Parse error: {:?}", e),
                    ..Default::default()
                }]
            }
        };

        self.client.publish_diagnostics(uri, diagnostics, None).await;
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            server_info: Some(ServerInfo {
                name: "my-lsp".to_string(),
                version: Some(env!("CARGO_PKG_VERSION").to_string()),
            }),
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                ..Default::default()
            },
        })
    }

    async fn initialized(&self, _: lsp_types::InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "My Language Server initialized")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri.clone();
        let text = params.text_document.text.clone();

        self.documents.write().await.insert(uri.clone(), text.clone());
        self.diagnose(uri, &text).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri.clone();
        if let Some(change) = params.content_changes.first() {
            let text = change.text.clone();
            self.documents.write().await.insert(uri.clone(), text.clone());
            self.diagnose(uri, &text).await;
        }
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(Backend::new);
    Server::new(stdin, stdout, socket).serve(service).await;
}
