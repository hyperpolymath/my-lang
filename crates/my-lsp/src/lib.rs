// SPDX-License-Identifier: MIT
//! Language Server Protocol implementation for My Language
//!
//! Provides IDE features through LSP:
//! - Diagnostics (errors, warnings)
//! - Completions (context-aware)
//! - Hover information
//! - Go to definition
//! - Find references
//! - Rename symbol
//! - Code actions (quick fixes)
//! - Formatting
//! - Signature help

use my_lang::{parse, check, Program, CheckError, Span};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer};

/// Document state for the language server
#[derive(Debug)]
pub struct Document {
    pub uri: Url,
    pub text: String,
    pub version: i32,
    pub program: Option<Program>,
    pub diagnostics: Vec<Diagnostic>,
}

impl Document {
    pub fn new(uri: Url, text: String, version: i32) -> Self {
        let mut doc = Document {
            uri,
            text,
            version,
            program: None,
            diagnostics: Vec::new(),
        };
        doc.analyze();
        doc
    }

    pub fn update(&mut self, text: String, version: i32) {
        self.text = text;
        self.version = version;
        self.analyze();
    }

    fn analyze(&mut self) {
        self.diagnostics.clear();

        match parse(&self.text) {
            Ok(program) => {
                // Run type checker
                match check(&program) {
                    Ok(()) => {
                        self.program = Some(program);
                    }
                    Err(errors) => {
                        for error in errors {
                            self.diagnostics.push(check_error_to_diagnostic(&error));
                        }
                        self.program = Some(program);
                    }
                }
            }
            Err(parse_error) => {
                self.diagnostics.push(Diagnostic {
                    range: Range {
                        start: Position { line: 0, character: 0 },
                        end: Position { line: 0, character: 0 },
                    },
                    severity: Some(DiagnosticSeverity::ERROR),
                    source: Some("my-lang".to_string()),
                    message: format!("{}", parse_error),
                    ..Default::default()
                });
            }
        }
    }
}

fn check_error_to_diagnostic(error: &CheckError) -> Diagnostic {
    // TODO: Extract span from error
    Diagnostic {
        range: Range {
            start: Position { line: 0, character: 0 },
            end: Position { line: 0, character: 0 },
        },
        severity: Some(DiagnosticSeverity::ERROR),
        source: Some("my-lang".to_string()),
        message: format!("{}", error),
        ..Default::default()
    }
}

/// My Language Server implementation
pub struct MyLanguageServer {
    client: Client,
    documents: Arc<RwLock<HashMap<Url, Document>>>,
}

impl MyLanguageServer {
    pub fn new(client: Client) -> Self {
        MyLanguageServer {
            client,
            documents: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    async fn publish_diagnostics(&self, uri: Url, diagnostics: Vec<Diagnostic>, version: Option<i32>) {
        self.client.publish_diagnostics(uri, diagnostics, version).await;
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for MyLanguageServer {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                completion_provider: Some(CompletionOptions {
                    trigger_characters: Some(vec![".".to_string(), ":".to_string()]),
                    ..Default::default()
                }),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                definition_provider: Some(OneOf::Left(true)),
                references_provider: Some(OneOf::Left(true)),
                rename_provider: Some(OneOf::Left(true)),
                document_formatting_provider: Some(OneOf::Left(true)),
                code_action_provider: Some(CodeActionProviderCapability::Simple(true)),
                signature_help_provider: Some(SignatureHelpOptions {
                    trigger_characters: Some(vec!["(".to_string(), ",".to_string()]),
                    ..Default::default()
                }),
                ..Default::default()
            },
            ..Default::default()
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "My Language Server initialized")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri.clone();
        let doc = Document::new(
            uri.clone(),
            params.text_document.text,
            params.text_document.version,
        );
        let diagnostics = doc.diagnostics.clone();
        let version = doc.version;

        self.documents.write().await.insert(uri.clone(), doc);
        self.publish_diagnostics(uri, diagnostics, Some(version)).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri.clone();
        let mut docs = self.documents.write().await;

        if let Some(doc) = docs.get_mut(&uri) {
            if let Some(change) = params.content_changes.into_iter().next() {
                doc.update(change.text, params.text_document.version);
                let diagnostics = doc.diagnostics.clone();
                let version = doc.version;
                drop(docs);
                self.publish_diagnostics(uri, diagnostics, Some(version)).await;
            }
        }
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        self.documents.write().await.remove(&params.text_document.uri);
    }

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        let uri = &params.text_document_position.text_document.uri;
        let docs = self.documents.read().await;

        if let Some(_doc) = docs.get(uri) {
            // TODO: Implement context-aware completions
            let items = vec![
                CompletionItem::new_simple("fn".to_string(), "Function declaration".to_string()),
                CompletionItem::new_simple("let".to_string(), "Variable binding".to_string()),
                CompletionItem::new_simple("if".to_string(), "Conditional expression".to_string()),
                CompletionItem::new_simple("match".to_string(), "Pattern matching".to_string()),
                CompletionItem::new_simple("struct".to_string(), "Struct definition".to_string()),
                CompletionItem::new_simple("effect".to_string(), "Effect declaration".to_string()),
                CompletionItem::new_simple("ai".to_string(), "AI expression".to_string()),
                CompletionItem::new_simple("ai_model".to_string(), "AI model configuration".to_string()),
                CompletionItem::new_simple("prompt".to_string(), "Prompt template".to_string()),
            ];
            return Ok(Some(CompletionResponse::Array(items)));
        }

        Ok(None)
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let uri = &params.text_document_position_params.text_document.uri;
        let _position = params.text_document_position_params.position;
        let docs = self.documents.read().await;

        if let Some(_doc) = docs.get(uri) {
            // TODO: Implement hover information
        }

        Ok(None)
    }

    async fn goto_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> Result<Option<GotoDefinitionResponse>> {
        let uri = &params.text_document_position_params.text_document.uri;
        let _position = params.text_document_position_params.position;
        let docs = self.documents.read().await;

        if let Some(_doc) = docs.get(uri) {
            // TODO: Implement go to definition
        }

        Ok(None)
    }

    async fn references(&self, params: ReferenceParams) -> Result<Option<Vec<Location>>> {
        let uri = &params.text_document_position.text_document.uri;
        let _position = params.text_document_position.position;
        let docs = self.documents.read().await;

        if let Some(_doc) = docs.get(uri) {
            // TODO: Implement find references
        }

        Ok(None)
    }

    async fn rename(&self, params: RenameParams) -> Result<Option<WorkspaceEdit>> {
        let uri = &params.text_document_position.text_document.uri;
        let _position = params.text_document_position.position;
        let _new_name = &params.new_name;
        let docs = self.documents.read().await;

        if let Some(_doc) = docs.get(uri) {
            // TODO: Implement rename
        }

        Ok(None)
    }

    async fn formatting(&self, params: DocumentFormattingParams) -> Result<Option<Vec<TextEdit>>> {
        let uri = &params.text_document.uri;
        let docs = self.documents.read().await;

        if let Some(doc) = docs.get(uri) {
            // TODO: Use my-fmt for formatting
            let _text = &doc.text;
        }

        Ok(None)
    }

    async fn code_action(&self, params: CodeActionParams) -> Result<Option<CodeActionResponse>> {
        let uri = &params.text_document.uri;
        let _range = params.range;
        let docs = self.documents.read().await;

        if let Some(_doc) = docs.get(uri) {
            // TODO: Implement code actions
        }

        Ok(None)
    }

    async fn signature_help(&self, params: SignatureHelpParams) -> Result<Option<SignatureHelp>> {
        let uri = &params.text_document_position_params.text_document.uri;
        let _position = params.text_document_position_params.position;
        let docs = self.documents.read().await;

        if let Some(_doc) = docs.get(uri) {
            // TODO: Implement signature help
        }

        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_document_creation() {
        let doc = Document::new(
            Url::parse("file:///test.my").unwrap(),
            "fn main() {}".to_string(),
            1,
        );
        assert!(doc.program.is_some());
    }
}
