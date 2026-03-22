# Tooling Overview

Development tools for My Language.

## Core Tools

| Tool | Command | Description |
|------|---------|-------------|
| Compiler | `ml build` | Compile source to executable |
| Runner | `ml run` | Build and run |
| Checker | `ml check` | Type-check without building |
| REPL | `ml repl` | Interactive environment |
| Formatter | `ml fmt` | Format source code |
| Linter | `ml lint` | Static analysis |
| Tester | `ml test` | Run tests |
| Documenter | `ml doc` | Generate documentation |

## CLI Reference

### Build Commands

```bash
# Build the project
ml build

# Build in release mode
ml build --release

# Build for specific target
ml build --target wasm32-wasi

# Build with verbose output
ml build --verbose
```

### Run Commands

```bash
# Build and run
ml run

# Run with arguments
ml run -- arg1 arg2

# Run example
ml run --example hello

# Run in release mode
ml run --release
```

### Check Commands

```bash
# Type-check only
ml check

# Parse only (show AST)
ml parse src/main.ml

# Type-check with verbose output
ml typecheck src/main.ml
```

### REPL

```bash
# Start REPL
ml repl

# REPL commands
:help     # Show help
:type e   # Show type of expression
:quit     # Exit
:clear    # Clear screen
:load f   # Load file
```

### Project Management

```bash
# Create new project
ml new project-name
ml new --lib library-name

# Initialize in current directory
ml init

# Clean build artifacts
ml clean
```

### Formatting

```bash
# Format all files
ml fmt

# Check formatting (for CI)
ml fmt --check

# Format specific file
ml fmt src/main.ml
```

### Testing

```bash
# Run all tests
ml test

# Run specific test
ml test test_name

# Run with pattern
ml test --filter "integration"

# Run with coverage
ml test --coverage
```

### Documentation

```bash
# Generate docs
ml doc

# Generate and open
ml doc --open

# Include private items
ml doc --document-private-items
```

## Configuration

### ml.toml

```toml
[package]
name = "myproject"
version = "0.1.0"
authors = ["Your Name <you@example.com>"]
edition = "2025"

[dependencies]
http = "1.0"
json = "2.0"

[dev-dependencies]
testing = "1.0"

[ai]
default_provider = "openai"

[build]
target = "native"

[profile.release]
opt_level = 3
lto = true
```

### Global Configuration

Located at `~/.config/mylang/config.toml`:

```toml
[defaults]
author = "Your Name <you@example.com>"
edition = "2025"

[ai]
default_provider = "openai"

[format]
max_width = 100
indent_size = 4
```

## IDE Support

### VS Code

Install the extension:
```bash
code --install-extension mylang.mylang-vscode
```

Features:
- Syntax highlighting
- Code completion
- Go to definition
- Find references
- Hover information
- Error diagnostics
- Formatting
- AI inline hints

### Neovim

Using `lazy.nvim`:
```lua
{
  "mylang/mylang.nvim",
  ft = "ml",
  config = function()
    require("lspconfig").mylang.setup{}
  end
}
```

### Other Editors

- **Vim**: `mylang/vim-mylang`
- **Emacs**: `mylang-mode`
- **Helix**: Built-in support
- **Zed**: Extension available

## Language Server

The Language Server Protocol (LSP) server provides:

- Real-time diagnostics
- Code completion
- Go to definition/references
- Hover documentation
- Code actions
- Formatting
- Rename symbol

Start manually:
```bash
ml lsp
```

## Debugger

### mldb (Planned)

```bash
# Start debugging
ml debug ./target/debug/myapp

# Common commands
(mldb) break src/main.ml:25   # Set breakpoint
(mldb) run                     # Start execution
(mldb) next                    # Step over
(mldb) step                    # Step into
(mldb) continue               # Continue
(mldb) print x                # Print variable
(mldb) backtrace              # Show stack trace
(mldb) ai-trace               # Show AI call history
```

## Package Manager

### mlpkg (Planned)

```bash
# Add dependency
mlpkg add http

# Remove dependency
mlpkg remove http

# Update dependencies
mlpkg update

# Search packages
mlpkg search json

# Publish package
mlpkg publish
```

## CI/CD Integration

### GitHub Actions

```yaml
name: CI
on: [push, pull_request]

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: mylang/setup-ml@v1

      - name: Check
        run: ml check

      - name: Test
        run: ml test

      - name: Build
        run: ml build --release
```

### Pre-commit Hook

```bash
#!/bin/sh
ml fmt --check && ml lint && ml test
```

## Environment Variables

| Variable | Description |
|----------|-------------|
| `ML_HOME` | Installation directory |
| `ML_CACHE` | Cache directory |
| `ML_LOG` | Log level |
| `OPENAI_API_KEY` | OpenAI API key |
| `ANTHROPIC_API_KEY` | Anthropic API key |

## Troubleshooting

### Common Issues

**Command not found:**
```bash
export PATH="$HOME/.mylang/bin:$PATH"
```

**Build errors:**
```bash
ml build --verbose 2>&1 | less
```

**AI not working:**
```bash
echo $OPENAI_API_KEY  # Verify key is set
```

**Format issues:**
```bash
ml fmt --check  # See what would change
```
