// SPDX-License-Identifier: MIT
//! My Language Server executable

use my_lsp::MyLanguageServer;
use tower_lsp::{LspService, Server};

#[tokio::main]
async fn main() {
    // Set up logging
    tracing_subscriber::fmt()
        .with_ansi(false)
        .with_writer(std::io::stderr)
        .init();

    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(|client| MyLanguageServer::new(client));
    Server::new(stdin, stdout, socket).serve(service).await;
}
