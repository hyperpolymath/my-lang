# My Language VSCode Extension

Language support for My Language - an AI-native programming language with four dialects (Solo, Duet, Ensemble, Me).

## Features

- Syntax highlighting for `.my` files
- Language Server Protocol (LSP) integration with `my-lsp`
- Commands for running, building, and testing
- IntelliSense support
- Error diagnostics
- Code completion
- Hover information
- Go to definition

## Requirements

- My Language toolchain installed (`my`, `my-lsp`)
- VSCode 1.80.0 or higher

## Extension Settings

This extension contributes the following settings:

* `my-lang.lsp.path`: Path to my-lsp executable (default: "my-lsp")
* `my-lang.trace.server`: Trace LSP communication (off/messages/verbose)

## Commands

* `My: Run File` - Run the current `.my` file
* `My: Build Binary` - Compile to native binary
* `My: Run Tests` - Run test suite

## Dialects

My Language has four dialects:

- **Solo**: Base systems programming (no AI features)
- **Duet**: AI-assisted development with verification
- **Ensemble**: AI as first-class native component
- **Me**: Personal AI agent dialect

## Installation

1. Install My Language toolchain
2. Install this extension from VSCode marketplace or `.vsix`
3. Open a `.my` file to activate the extension

## Building from Source

The runtime entry the host loads is `src/index.cjs` (a vendored adapter +
wrapper); `package.json` `main` points there, and `out/extension.cjs` is the
committed last-good build. Recompiling from the AffineScript source requires:

- the **AffineScript compiler** on `PATH` (`affinescript`), and
- `AFFINESCRIPT_STDLIB` pointing at the AffineScript `stdlib/` directory
  (the compiler does not yet bundle stdlib bindings).

```bash
cd vscode-extension
npm install
export AFFINESCRIPT_STDLIB=/path/to/affinescript/stdlib
npm run compile           # affinescript compile --vscode-extension …
npm run package           # vsce package  -> my-lang-<version>.vsix (no PAT)
code --install-extension my-lang-0.3.0.vsix
```

> **Status:** the `src/extension.affine` → `out/extension.cjs` recompile is
> gated on upstream **affinescript#35 Phase 2** (vscode-API stdlib bindings;
> `stdlib/Vscode.affine` does not yet type-check). Until that lands, the
> committed `out/extension.cjs` + `src/index.cjs` are the working artifacts;
> `vsce package` produces an installable `.vsix` from them and needs **no**
> Azure Marketplace PAT (only `vsce publish` does).

## License

PMPL-1.0-or-later

## Author

Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk>
