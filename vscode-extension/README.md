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
npm install               # needs a reachable npm registry (NOT WSL here)
export AFFINESCRIPT_STDLIB=/path/to/affinescript/stdlib
npm run compile           # affinescript compile --vscode-extension …
npm run package           # @vscode/vsce package -> my-lang-<version>.vsix
code --install-extension my-lang-0.3.0.vsix
```

> **Status:** `src/extension.affine` **compiles** (`affinescript compile
> --vscode-extension`) and the upstream `stdlib/Vscode.affine` /
> `VscodeLanguageClient.affine` bindings type-check on affinescript `main`
> (#35 Phase 2 effectively complete). `src/index.cjs` (the `main`) +
> `out/extension.cjs` are the runtime artifacts.
>
> **Packaging** needs `@vscode/vsce` (the old deprecated `vsce` 2.x cannot
> handle a `.cjs` entrypoint — it looks for `index.cjs.js`) **and** a
> registry-reachable environment for `npm install` (the WSL dev box's npm
> registry is unreachable, so package/publish must run elsewhere — CI or a
> non-WSL host). `vsce package` → `.vsix` needs **no** Azure Marketplace
> PAT; only `vsce publish` (Marketplace upload) does.

## License

PMPL-1.0-or-later

## Author

Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk>
