# My Language

A programming language with **first-class AI integration**.

```ml
// Define an AI model
ai_model claude {
    provider: "anthropic"
    model: "claude-3-opus"
    temperature: 0.7
}

// Quick AI query
let answer = ai! { "What is the meaning of life?" };

// Typed AI operations
fn summarize(text: String) -> AI<String> {
    return ai query {
        prompt: "Summarize this text"
        context: text
        model: claude
    };
}

fn main() {
    let summary = summarize("Long article content...");
    println(summary);
}
```

## Features

| Feature | Description |
|---------|-------------|
| **First-Class AI** | Native `ai!` expressions, AI models, prompt templates |
| **Type Safety** | Static typing with inference and AI-aware types (`AI<T>`) |
| **Effect System** | Track AI calls, I/O, and side effects in the type system |
| **Pattern Matching** | Exhaustive matching with guards |
| **Concurrency** | `go` blocks for lightweight concurrent execution |
| **Memory Safety** | Ownership and borrowing (Rust-inspired) |
| **Contracts** | Pre/post conditions with `where` clauses |

## Quick Start

```bash
# Build the compiler
cargo build --release

# Run a program
cargo run -- run examples/hello.ml

# Start the REPL
cargo run -- repl

# Parse and type-check
cargo run -- check examples/demo.ml
```

## Project Structure

```
my-lang/
├── src/                    # Core language implementation
│   ├── lexer.rs           # Tokenizer
│   ├── parser.rs          # Recursive descent parser
│   ├── ast.rs             # Abstract syntax tree
│   ├── checker.rs         # Type checker
│   ├── interpreter.rs     # Tree-walking interpreter
│   └── stdlib.rs          # Standard library (60+ functions)
├── lib/                    # Extended library modules
│   ├── common/            # I/O, strings, math, arrays
│   └── mylang/            # AI runtime, prompts, tools
├── my-ssg/                # Static Site Generator (ecosystem tool)
├── examples/              # Example programs
├── docs/wiki/             # Full documentation
└── grammar.ebnf           # Formal grammar specification
```

## Language Examples

### Variables and Functions

```ml
fn factorial(n: Int) -> Int {
    if n <= 1 {
        return 1;
    } else {
        return n * factorial(n - 1);
    }
}

fn main() {
    let result = factorial(5);
    println(str_concat("5! = ", to_string(result)));
}
```

### AI Operations

```ml
// AI model configuration
ai_model gpt4 {
    provider: "openai"
    model: "gpt-4-turbo"
    temperature: 0.5
}

// Prompt templates
prompt summarize { "Summarize in 3 sentences: {text}" }

// AI expressions
fn analyze(code: String) -> AI<String> {
    let bugs = ai verify {
        input: code
        constraint: "syntactically valid code"
    };

    let review = ai query {
        prompt: "Review this code"
        context: code
        model: gpt4
    };

    return review;
}
```

### Pattern Matching

```ml
fn describe(x: Int) -> String {
    match x {
        0 => "zero",
        1 => "one",
        n if n < 0 => "negative",
        _ => "other",
    }
}
```

### Concurrency

```ml
fn fetch_all() {
    go {
        let a = ai query { prompt: "Task A" };
        println(a);
    }

    go {
        let b = ai query { prompt: "Task B" };
        println(b);
    }
}
```

## Standard Library

The interpreter includes 60+ built-in functions:

- **I/O**: `print`, `println`, `debug`, `input`
- **Strings**: `len`, `str_concat`, `str_split`, `str_join`, `str_trim`, `str_upper`, `str_lower`, `str_contains`, `str_replace`
- **Math**: `abs`, `min`, `max`, `floor`, `ceil`, `round`, `sqrt`, `pow`, `sin`, `cos`, `tan`, `log`, `PI`, `E`
- **Arrays**: `push`, `pop`, `first`, `last`, `get`, `set`, `concat`, `slice`, `reverse`, `range`, `contains`
- **Types**: `type_of`, `to_string`, `to_int`, `to_float`, `to_bool`, `is_int`, `is_string`, `is_array`
- **Utilities**: `assert`, `panic`, `time`, `sleep`, `random`, `env`

## Ecosystem

### My SSG

Static site generator powered by My Language templates:

```bash
cd my-ssg
cargo build
cargo run -- new my-blog
cargo run -- build
```

Features:
- Markdown with YAML frontmatter
- Template expressions (`{{ page.title }}`)
- Control flow (`{% if %}`, `{% for %}`)
- Filters (`{{ name | uppercase }}`)

## Documentation

- [Getting Started](docs/wiki/guides/getting-started.md)
- [Language Tour](docs/wiki/language/tour.md)
- [AI Features](docs/wiki/language/ai-features.md)
- [Full Roadmap](docs/wiki/roadmap/overview.md)
- [Grammar Specification](grammar.ebnf)

## Status

**Version 0.2.0** - Interpreter & Runtime Complete

| Component | Status |
|-----------|--------|
| Lexer | ✅ Complete |
| Parser | ✅ Complete |
| Type Checker | ✅ Complete |
| Interpreter | ✅ Complete |
| REPL | ✅ Complete |
| Standard Library | ✅ Complete |
| AI Runtime (Mock) | ✅ Complete |
| My SSG | ✅ Complete |
| Native Compiler | 🔄 Planned |

## License

MIT License - See [LICENSE](LICENSE) for details.

## Contributing

Contributions welcome! See the [roadmap](docs/wiki/roadmap/overview.md) for planned features.
