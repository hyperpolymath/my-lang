# Installation Guide

Detailed instructions for installing My Language on various platforms.

## System Requirements

### Minimum Requirements

| Component | Requirement |
|-----------|-------------|
| OS | Linux, macOS 10.15+, Windows 10+ |
| RAM | 4 GB |
| Disk Space | 500 MB |
| Architecture | x86_64, ARM64 |

### Recommended

| Component | Recommendation |
|-----------|----------------|
| RAM | 8 GB+ |
| Disk Space | 2 GB+ |
| CPU | Multi-core |

## Installation Methods

### 1. Official Installer (Recommended)

The official installer handles everything automatically.

**Unix (Linux/macOS):**

```bash
curl -sSf https://mylang.org/install.sh | sh
```

This will:
- Download the latest stable release
- Install to `~/.mylang`
- Add to your PATH
- Install shell completions

**Windows (PowerShell as Administrator):**

```powershell
irm https://mylang.org/install.ps1 | iex
```

**Verify installation:**

```bash
ml --version
```

### 2. Package Managers

#### Homebrew (macOS/Linux)

```bash
brew install mylang
```

#### APT (Debian/Ubuntu)

```bash
# Add repository
curl -fsSL https://packages.mylang.org/gpg | sudo gpg --dearmor -o /usr/share/keyrings/mylang.gpg
echo "deb [signed-by=/usr/share/keyrings/mylang.gpg] https://packages.mylang.org/apt stable main" | sudo tee /etc/apt/sources.list.d/mylang.list

# Install
sudo apt update
sudo apt install mylang
```

#### DNF (Fedora/RHEL)

```bash
sudo dnf config-manager --add-repo https://packages.mylang.org/rpm/mylang.repo
sudo dnf install mylang
```

#### Pacman (Arch Linux)

```bash
# From AUR
yay -S mylang
```

#### Scoop (Windows)

```powershell
scoop bucket add mylang https://github.com/mylang/scoop-mylang
scoop install mylang
```

#### Chocolatey (Windows)

```powershell
choco install mylang
```

### 3. Cargo (Rust)

If you have Rust installed:

```bash
cargo install mylang
```

### 4. From Source

For development or custom builds:

```bash
# Clone repository
git clone https://github.com/mylang/mylang.git
cd mylang

# Build
cargo build --release

# Install
cargo install --path .
```

### 5. Docker

```bash
docker pull mylang/mylang:latest

# Run REPL
docker run -it mylang/mylang repl

# Compile a project
docker run -v $(pwd):/app mylang/mylang build
```

### 6. Pre-built Binaries

Download from [releases page](https://github.com/mylang/mylang/releases):

| Platform | File |
|----------|------|
| Linux x64 | `mylang-linux-x86_64.tar.gz` |
| Linux ARM64 | `mylang-linux-aarch64.tar.gz` |
| macOS x64 | `mylang-darwin-x86_64.tar.gz` |
| macOS ARM64 | `mylang-darwin-aarch64.tar.gz` |
| Windows x64 | `mylang-windows-x86_64.zip` |

Extract and add to PATH:

```bash
# Linux/macOS
tar xzf mylang-linux-x86_64.tar.gz
sudo mv mylang /usr/local/bin/

# Windows (PowerShell)
Expand-Archive mylang-windows-x86_64.zip -DestinationPath C:\mylang
# Add C:\mylang to PATH
```

## Version Management

### Multiple Versions

Use `mlup` for version management:

```bash
# Install mlup
curl -sSf https://mylang.org/mlup.sh | sh

# Install specific version
mlup install 0.2.0
mlup install 0.1.0

# Switch versions
mlup use 0.2.0

# List installed versions
mlup list
```

### Nightly Builds

For latest development features:

```bash
# With installer
curl -sSf https://mylang.org/install.sh | sh -s -- --channel nightly

# With mlup
mlup install nightly
mlup use nightly
```

## Post-Installation Setup

### 1. Verify Installation

```bash
# Check version
ml --version

# Check help
ml --help

# Run a quick test
echo 'fn main() { print("Hello!"); }' > test.ml
ml run test.ml
rm test.ml
```

### 2. Configure AI Provider

Set your AI API key:

```bash
# OpenAI
export OPENAI_API_KEY="sk-..."

# Anthropic
export ANTHROPIC_API_KEY="sk-ant-..."

# Add to shell profile
echo 'export OPENAI_API_KEY="sk-..."' >> ~/.bashrc  # or ~/.zshrc
```

### 3. Editor Integration

#### VS Code

```bash
code --install-extension mylang.mylang-vscode
```

#### Neovim

With `lazy.nvim`:
```lua
{
  "mylang/mylang.nvim",
  config = function()
    require("lspconfig").mylang.setup{}
  end
}
```

#### Vim

```vim
Plug 'mylang/vim-mylang'
```

### 4. Shell Completions

**Bash:**
```bash
ml completions bash > ~/.local/share/bash-completion/completions/ml
```

**Zsh:**
```bash
ml completions zsh > ~/.zfunc/_ml
```

**Fish:**
```bash
ml completions fish > ~/.config/fish/completions/ml.fish
```

**PowerShell:**
```powershell
ml completions powershell >> $PROFILE
```

## Configuration

### Global Configuration

Create `~/.config/mylang/config.toml`:

```toml
[defaults]
edition = "2025"
author = "Your Name <your@email.com>"

[ai]
default_provider = "openai"
cache_enabled = true
cache_ttl = 3600

[ai.providers.openai]
model = "gpt-4"
temperature = 0.7

[build]
jobs = 4  # Parallel jobs
color = "auto"

[format]
max_width = 100
indent_style = "space"
indent_size = 4
```

### Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `ML_HOME` | Installation directory | `~/.mylang` |
| `ML_CACHE` | Cache directory | `~/.cache/mylang` |
| `OPENAI_API_KEY` | OpenAI API key | - |
| `ANTHROPIC_API_KEY` | Anthropic API key | - |
| `ML_LOG` | Log level (error, warn, info, debug, trace) | `error` |

## Updating

### Official Installer

```bash
ml self update
```

### Package Managers

```bash
# Homebrew
brew upgrade mylang

# APT
sudo apt update && sudo apt upgrade mylang

# Cargo
cargo install mylang --force
```

### mlup

```bash
mlup update
```

## Uninstalling

### Official Installer

```bash
ml self uninstall
```

Or manually:
```bash
rm -rf ~/.mylang
# Remove from PATH in ~/.bashrc or ~/.zshrc
```

### Package Managers

```bash
# Homebrew
brew uninstall mylang

# APT
sudo apt remove mylang

# Cargo
cargo uninstall mylang
```

## Troubleshooting

### PATH Issues

If `ml` command not found:

```bash
# Add to PATH manually
export PATH="$HOME/.mylang/bin:$PATH"

# Make permanent
echo 'export PATH="$HOME/.mylang/bin:$PATH"' >> ~/.bashrc
source ~/.bashrc
```

### Permission Errors

```bash
# Fix permissions
chmod -R u+rwX ~/.mylang
```

### SSL/TLS Errors

Update CA certificates:

```bash
# Linux
sudo apt install ca-certificates

# macOS
brew install ca-certificates
```

### Proxy Configuration

```bash
export HTTP_PROXY="http://proxy:port"
export HTTPS_PROXY="http://proxy:port"
```

### Firewall Issues

Ensure these domains are accessible:
- `mylang.org`
- `packages.mylang.org`
- `api.openai.com` (for AI features)

## Getting Help

If you encounter issues:

1. Check the [FAQ](https://mylang.org/faq)
2. Search [GitHub Issues](https://github.com/mylang/mylang/issues)
3. Join [Discord](https://discord.gg/mylang)
4. Open a new issue with:
   - OS and version
   - Installation method
   - Error message
   - Steps to reproduce
