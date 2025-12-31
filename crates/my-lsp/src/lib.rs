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

use my_lang::{parse, check, Program, CheckError};
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

/// Extract location from CheckError and convert to LSP Diagnostic
fn check_error_to_diagnostic(error: &CheckError) -> Diagnostic {
    let (line, column) = extract_error_location(error);
    // LSP uses 0-based line numbers
    let lsp_line = if line > 0 { line as u32 - 1 } else { 0 };
    let lsp_col = if column > 0 { column as u32 - 1 } else { 0 };

    Diagnostic {
        range: Range {
            start: Position { line: lsp_line, character: lsp_col },
            end: Position { line: lsp_line, character: lsp_col + 1 },
        },
        severity: Some(DiagnosticSeverity::ERROR),
        source: Some("my-lang".to_string()),
        message: format!("{}", error),
        ..Default::default()
    }
}

/// Extract line and column from a CheckError
fn extract_error_location(error: &CheckError) -> (usize, usize) {
    match error {
        CheckError::UndefinedVariable { line, column, .. } => (*line, *column),
        CheckError::UndefinedType { line, column, .. } => (*line, *column),
        CheckError::UndefinedFunction { line, column, .. } => (*line, *column),
        CheckError::UndefinedAiModel { line, column, .. } => (*line, *column),
        CheckError::UndefinedPrompt { line, column, .. } => (*line, *column),
        CheckError::TypeMismatch { line, column, .. } => (*line, *column),
        CheckError::DuplicateDefinition { line, column, .. } => (*line, *column),
        CheckError::ImmutableAssignment { line, column, .. } => (*line, *column),
        CheckError::WrongArgCount { line, column, .. } => (*line, *column),
        CheckError::InvalidBinaryOp { line, column, .. } => (*line, *column),
        CheckError::NonBoolCondition { line, column, .. } => (*line, *column),
        CheckError::Other { line, column, .. } => (*line, *column),
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
        let position = params.text_document_position_params.position;
        let docs = self.documents.read().await;

        if let Some(doc) = docs.get(uri) {
            if let Some(program) = &doc.program {
                // Find word at position
                let word = get_word_at_position(&doc.text, position);

                // Look up in definitions
                for item in &program.items {
                    match item {
                        my_lang::TopLevel::Function(f) => {
                            if f.name.name == word {
                                let params_str: Vec<String> = f.params.iter()
                                    .map(|p| format!("{}: {:?}", p.name.name, p.ty))
                                    .collect();
                                let ret_str = f.return_type.as_ref()
                                    .map(|t| format!(" -> {:?}", t))
                                    .unwrap_or_default();

                                return Ok(Some(Hover {
                                    contents: HoverContents::Markup(MarkupContent {
                                        kind: MarkupKind::Markdown,
                                        value: format!("```my\nfn {}({}){}\n```", f.name.name, params_str.join(", "), ret_str),
                                    }),
                                    range: None,
                                }));
                            }
                        }
                        my_lang::TopLevel::Struct(s) => {
                            if s.name.name == word {
                                let fields_str: Vec<String> = s.fields.iter()
                                    .map(|f| format!("    {}: {:?}", f.name.name, f.ty))
                                    .collect();

                                return Ok(Some(Hover {
                                    contents: HoverContents::Markup(MarkupContent {
                                        kind: MarkupKind::Markdown,
                                        value: format!("```my\nstruct {} {{\n{}\n}}\n```", s.name.name, fields_str.join(",\n")),
                                    }),
                                    range: None,
                                }));
                            }
                        }
                        my_lang::TopLevel::Effect(e) => {
                            if e.name.name == word {
                                return Ok(Some(Hover {
                                    contents: HoverContents::Markup(MarkupContent {
                                        kind: MarkupKind::Markdown,
                                        value: format!("```my\neffect {} {{ ... }}\n```", e.name.name),
                                    }),
                                    range: None,
                                }));
                            }
                        }
                        my_lang::TopLevel::AiModel(m) => {
                            if m.name.name == word {
                                return Ok(Some(Hover {
                                    contents: HoverContents::Markup(MarkupContent {
                                        kind: MarkupKind::Markdown,
                                        value: format!("```my\nai_model {} {{ ... }}\n```\nAI model configuration", m.name.name),
                                    }),
                                    range: None,
                                }));
                            }
                        }
                        _ => {}
                    }
                }

                // Check for keywords
                let keyword_docs = match word.as_str() {
                    "fn" => Some("Function declaration keyword"),
                    "let" => Some("Variable binding keyword"),
                    "if" => Some("Conditional expression"),
                    "else" => Some("Alternative branch for if"),
                    "match" => Some("Pattern matching expression"),
                    "struct" => Some("Structure type definition"),
                    "effect" => Some("Effect type declaration"),
                    "ai" => Some("AI expression - invoke AI capabilities"),
                    "ai_model" => Some("AI model configuration block"),
                    "return" => Some("Return from function"),
                    "go" => Some("Spawn concurrent task"),
                    "await" => Some("Wait for async result"),
                    "try" => Some("Error handling expression"),
                    "comptime" => Some("Compile-time evaluation block"),
                    _ => None,
                };

                if let Some(doc_text) = keyword_docs {
                    return Ok(Some(Hover {
                        contents: HoverContents::Markup(MarkupContent {
                            kind: MarkupKind::Markdown,
                            value: format!("**{}**\n\n{}", word, doc_text),
                        }),
                        range: None,
                    }));
                }
            }
        }

        Ok(None)
    }

    async fn goto_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> Result<Option<GotoDefinitionResponse>> {
        let uri = &params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;
        let docs = self.documents.read().await;

        if let Some(doc) = docs.get(uri) {
            if let Some(program) = &doc.program {
                let word = get_word_at_position(&doc.text, position);

                // Find definition in program
                for item in &program.items {
                    let (name, span) = match item {
                        my_lang::TopLevel::Function(f) => (&f.name.name, f.span),
                        my_lang::TopLevel::Struct(s) => (&s.name.name, s.span),
                        my_lang::TopLevel::Effect(e) => (&e.name.name, e.span),
                        my_lang::TopLevel::AiModel(m) => (&m.name.name, m.span),
                        _ => continue,
                    };

                    if *name == word {
                        let (line, col) = offset_to_position(&doc.text, span.start);
                        return Ok(Some(GotoDefinitionResponse::Scalar(Location {
                            uri: uri.clone(),
                            range: Range {
                                start: Position { line, character: col },
                                end: Position { line, character: col + name.len() as u32 },
                            },
                        })));
                    }
                }
            }
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

/// Get the word at a given position in the text
fn get_word_at_position(text: &str, position: Position) -> String {
    let lines: Vec<&str> = text.lines().collect();
    if (position.line as usize) >= lines.len() {
        return String::new();
    }

    let line = lines[position.line as usize];
    let col = position.character as usize;

    if col >= line.len() {
        return String::new();
    }

    // Find word boundaries
    let chars: Vec<char> = line.chars().collect();
    let mut start = col;
    let mut end = col;

    // Go backwards to find start
    while start > 0 && is_word_char(chars[start - 1]) {
        start -= 1;
    }

    // Go forwards to find end
    while end < chars.len() && is_word_char(chars[end]) {
        end += 1;
    }

    chars[start..end].iter().collect()
}

fn is_word_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

/// Convert byte offset to line/column position
fn offset_to_position(text: &str, offset: usize) -> (u32, u32) {
    let mut line = 0u32;
    let mut col = 0u32;
    let mut current_offset = 0;

    for ch in text.chars() {
        if current_offset >= offset {
            break;
        }
        if ch == '\n' {
            line += 1;
            col = 0;
        } else {
            col += 1;
        }
        current_offset += ch.len_utf8();
    }

    (line, col)
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
