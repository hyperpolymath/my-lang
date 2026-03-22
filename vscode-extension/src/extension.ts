// SPDX-License-Identifier: PMPL-1.0-or-later
import * as vscode from 'vscode';
import * as path from 'path';
import { LanguageClient, LanguageClientOptions, ServerOptions, TransportKind } from 'vscode-languageclient/node';

let client: LanguageClient;

export function activate(context: vscode.ExtensionContext) {
    console.log('My Language extension activated');

    // LSP client setup
    const config = vscode.workspace.getConfiguration('my-lang');
    const lspPath = config.get<string>('lsp.path', 'my-lsp');

    const serverOptions: ServerOptions = {
        run: { command: lspPath },
        debug: { command: lspPath }
    };

    const clientOptions: LanguageClientOptions = {
        documentSelector: [{ scheme: 'file', language: 'my' }],
        synchronize: {
            fileEvents: vscode.workspace.createFileSystemWatcher('**/*.my')
        }
    };

    client = new LanguageClient(
        'my-lang',
        'My Language Server',
        serverOptions,
        clientOptions
    );

    client.start();

    // Register commands
    context.subscriptions.push(
        vscode.commands.registerCommand('my-lang.run', runFile),
        vscode.commands.registerCommand('my-lang.build', buildBinary),
        vscode.commands.registerCommand('my-lang.test', runTests)
    );
}

export function deactivate(): Thenable<void> | undefined {
    if (!client) {
        return undefined;
    }
    return client.stop();
}

async function runFile() {
    const editor = vscode.window.activeTextEditor;
    if (!editor) {
        vscode.window.showErrorMessage('No active editor');
        return;
    }

    const filePath = editor.document.uri.fsPath;
    const terminal = vscode.window.createTerminal('My Language');
    terminal.show();
    terminal.sendText(`my run "${filePath}"`);
}

async function buildBinary() {
    const editor = vscode.window.activeTextEditor;
    if (!editor) {
        vscode.window.showErrorMessage('No active editor');
        return;
    }

    const filePath = editor.document.uri.fsPath;
    const outputPath = filePath.replace(/\.my$/, '');
    const terminal = vscode.window.createTerminal('My Language');
    terminal.show();
    terminal.sendText(`my build "${filePath}" -o "${outputPath}"`);
}

async function runTests() {
    const workspaceFolder = vscode.workspace.workspaceFolders?.[0];
    if (!workspaceFolder) {
        vscode.window.showErrorMessage('No workspace folder');
        return;
    }

    const terminal = vscode.window.createTerminal('My Language Tests');
    terminal.show();
    terminal.sendText(`cd "${workspaceFolder.uri.fsPath}" && my-test`);
}
