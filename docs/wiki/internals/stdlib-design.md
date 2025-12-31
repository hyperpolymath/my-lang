# Standard Library Design

This document describes the design principles and patterns used in the My Language standard library, inspired by the [hyperpolymath/aggregate-library](https://github.com/hyperpolymath/aggregate-library) specification.

## Design Philosophy

### Separation of Common and Standard Library

The standard library is organized into two tiers:

1. **Common Library** (20 Core Operations) - Universal operations that exist across all languages
2. **Extended Library** - Language-specific features (AI, async, effects, etc.)

### Core Operations

These 20 operations form the minimal vocabulary of the language:

| Category | Operations |
|----------|-----------|
| **Arithmetic** | `add`, `subtract`, `multiply`, `divide`, `modulo` |
| **Comparison** | `less_than`, `greater_than`, `equal`, `not_equal`, `less_equal`, `greater_equal` |
| **Logical** | `and`, `or`, `not` |
| **String** | `concat`, `length`, `substring` |
| **Collection** | `map`, `filter`, `fold`, `contains` |
| **Conditional** | `if_then_else` |

## API Design Conventions

### Three-Component Specification Format

Each operation is documented with:

1. **Interface Signature** - Abstract type notation
2. **Behavioral Semantics** - Purpose, parameters, return values, edge cases
3. **Executable Test Cases** - Concrete examples with expected outputs

Example:

```markdown
## abs(number) -> number

**Signature**: `abs: Number -> Number`

**Purpose**: Returns the absolute value of a number.

**Parameters**:
- `x: Number` - The input number (Int or Float)

**Returns**: `Number` - The absolute value (same type as input)

**Properties**:
- `abs(x) >= 0` for all x
- `abs(-x) == abs(x)`
- `abs(0) == 0`

**Edge Cases**:
- Int::MIN: Returns Int::MIN (overflow behavior)
- NaN (Float): Returns NaN
- Infinity: Returns positive Infinity

**Examples**:
```ml
abs(-5)      // => 5
abs(3.14)    // => 3.14
abs(0)       // => 0
```

**Test Cases**:
```yaml
- input: -42
  output: 42
  description: "Negative integer"

- input: 3.14159
  output: 3.14159
  description: "Positive float"
```
```

### Immutability-First Design

Operations return **new values** rather than modifying state:

```ml
// Good: Returns new array
fn push(arr: [T], value: T) -> [T]

// Avoid: Mutates in place (unless explicitly needed)
fn push_mut(arr: &mut [T], value: T)
```

### Error Handling

Error handling is a Standard Library concern, not Common Library:

- Use `Result<T, E>` for operations that can fail
- Use `Option<T>` for optional values
- Provide context in error messages
- Include source location information

```ml
// Good: Returns Result with specific error
fn parse(s: &str) -> Result<Int, ParseError>

// Good: Error includes context
ParseError::InvalidDigit {
    found: char,
    position: usize,
    input: String
}
```

### Type Safety with Minimal Vocabulary

Use these five fundamental types:

| Type | Description |
|------|-------------|
| `Number` | Numeric values (Int, Float) |
| `String` | Text and character sequences |
| `Bool` | True/false values |
| `Collection[T]` | Ordered sequences with generic element type |
| `Function[A -> B]` | Function types with input/output signatures |

## Module Organization

```
std/
├── prelude/          # Auto-imported essentials
│   ├── types.ml      # Option, Result, primitives
│   ├── traits.ml     # Clone, Debug, Display, etc.
│   └── functions.ml  # print, assert, panic
├── core/             # Core operations (Common Library)
│   ├── arithmetic.ml # add, subtract, multiply, divide, modulo
│   ├── comparison.ml # <, >, ==, !=, <=, >=
│   ├── logical.ml    # and, or, not
│   ├── string.ml     # concat, length, substring
│   └── collections.ml# map, filter, fold, contains
├── string/           # Extended string operations
├── collections/      # Extended collections
├── io/               # Input/output
├── fs/               # Filesystem
├── net/              # Networking
├── sync/             # Synchronization
├── async/            # Async runtime
├── time/             # Date and time
├── math/             # Mathematics
├── ai/               # AI integration
└── error/            # Error types and handling
```

## Implementation Guidelines

### SPDX Headers

All files must include SPDX license headers:

```rust
// SPDX-License-Identifier: MIT
// SPDX-FileCopyrightText: 2025 Hyperpolymath
```

### Documentation

Every public item must have:
- Brief description
- Parameters (with types)
- Return value
- Examples
- Edge cases (if applicable)

### Testing

Each operation must have:
- Unit tests for normal cases
- Edge case tests
- Property-based tests where applicable

### Versioning

- Pre-v1.0.0: 6-month deprecation windows
- Post-v1.0.0: 2 releases or 12 months
- Always provide migration guides for breaking changes

## Security Considerations

- No network dependencies in core library
- API keys handled with secure memory (zeroize)
- Input validation at system boundaries
- No hardcoded credentials

## Performance Guidelines

- Prefer owned types over references where possible
- Use `Cow<T>` for conditional ownership
- Avoid unnecessary cloning in hot paths
- Document algorithmic complexity
