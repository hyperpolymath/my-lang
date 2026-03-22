# Getting Started

Welcome to My Language! This guide will help you install the toolchain and write your first program.

## Installation

### Quick Install (Recommended)

**Unix (Linux/macOS):**
```bash
curl -sSf https://mylang.org/install.sh | sh
```

**Windows (PowerShell):**
```powershell
irm https://mylang.org/install.ps1 | iex
```

### Package Managers

**Homebrew (macOS/Linux):**
```bash
brew install mylang
```

**Cargo (Rust users):**
```bash
cargo install mylang
```

### Verify Installation

```bash
ml --version
# My Language v0.1.0
```

## Your First Program

### Create a Project

```bash
ml new hello-world
cd hello-world
```

This creates:
```
hello-world/
├── ml.toml
└── src/
    └── main.ml
```

### The Main File

Open `src/main.ml`:

```ml
fn main() {
    print("Hello, World!");
}
```

### Run Your Program

```bash
ml run
# Hello, World!
```

## Project Structure

### ml.toml

The project manifest:

```toml
[package]
name = "hello-world"
version = "0.1.0"
edition = "2025"

[dependencies]
# Add dependencies here

[ai]
# AI configuration (optional)
default_provider = "openai"
```

### Source Files

```
src/
├── main.ml      # Entry point (binary)
├── lib.ml       # Library root (optional)
└── utils/       # Submodules
    ├── mod.ml
    └── helpers.ml
```

## Basic Syntax

### Variables

```ml
// Immutable (default)
let name = "Alice";
let age = 30;

// Mutable
let mut counter = 0;
counter = counter + 1;

// Type annotations
let score: Int = 100;
```

### Functions

```ml
fn greet(name: String) -> String {
    "Hello, {name}!"
}

fn main() {
    let message = greet("World");
    print(message);
}
```

### Control Flow

```ml
fn check_age(age: Int) {
    if age >= 18 {
        print("Adult");
    } else {
        print("Minor");
    }
}

fn describe_number(n: Int) -> String {
    match n {
        0 => "zero",
        1..=9 => "single digit",
        _ => "large number",
    }
}
```

### Data Structures

```ml
// Struct
struct Point {
    x: Int,
    y: Int,
}

// Creating instances
let origin = Point { x: 0, y: 0 };
let point = Point { x: 10, y: 20 };

// Enum
enum Color {
    Red,
    Green,
    Blue,
}

let color = Color::Red;
```

## Using AI Features

My Language has built-in AI capabilities:

```ml
fn main() {
    // Quick AI query
    let answer = ai! { "What is 2 + 2?" };
    print("AI says: {answer}");

    // AI with more options
    let summary = ai query {
        prompt: "Summarize quantum computing"
        max_tokens: 100
    };
    print(summary);
}
```

### Configure AI Provider

In `ml.toml`:

```toml
[ai]
default_provider = "openai"

[ai.providers.openai]
model = "gpt-4"
# API key from environment: OPENAI_API_KEY
```

Or in code:

```ml
use std::ai::{set_default_provider, Provider};

fn main() {
    set_default_provider(Provider::OpenAI {
        api_key: env("OPENAI_API_KEY"),
        model: "gpt-4",
    });

    let result = ai! { "Hello!" };
}
```

## Common Commands

| Command | Description |
|---------|-------------|
| `ml new <name>` | Create new project |
| `ml build` | Build the project |
| `ml run` | Build and run |
| `ml check` | Type-check without building |
| `ml test` | Run tests |
| `ml fmt` | Format code |
| `ml doc` | Generate documentation |

## Build Modes

```bash
# Debug build (default)
ml build

# Release build (optimized)
ml build --release

# Run in release mode
ml run --release
```

## Adding Dependencies

Edit `ml.toml`:

```toml
[dependencies]
http = "1.0"
json = "2.0"
```

Then use them:

```ml
use http::Client;
use json;

async fn fetch_data() {
    let client = Client::new();
    let response = client.get("https://api.example.com/data").await?;
    let data = json::parse(response.body())?;
}
```

## Editor Setup

### VS Code

Install the "My Language" extension:
```bash
code --install-extension mylang.mylang-vscode
```

Features:
- Syntax highlighting
- Code completion
- Go to definition
- Error diagnostics
- Formatting

### Neovim

Add to your config:
```lua
require('lspconfig').mylang.setup{}
```

## Next Steps

1. **Learn the Language**: [Language Tour](../language/tour.md)
2. **Explore AI Features**: [AI Integration Guide](ai-integration.md)
3. **Build Something**: [Tutorials](../tutorials/basics.md)
4. **Read the Reference**: [Standard Library](../reference/stdlib.md)

## Getting Help

- **Documentation**: [docs.mylang.org](https://docs.mylang.org)
- **Community Discord**: [discord.gg/mylang](https://discord.gg/mylang)
- **GitHub Issues**: [github.com/mylang/mylang](https://github.com/mylang/mylang)
- **Stack Overflow**: Tag `[mylang]`

## Common Issues

### AI calls timeout

```ml
// Add explicit timeout
let result = timeout(Duration::seconds(30)) {
    ai query { prompt: "..." }
}.await?;
```

### Missing API key

Set the environment variable:
```bash
export OPENAI_API_KEY="your-key-here"
```

### Compile errors

Run with verbose output:
```bash
ml build --verbose
```

### Format issues

Run the formatter:
```bash
ml fmt
```
