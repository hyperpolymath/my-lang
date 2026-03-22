# Language Roadmap

This document details the evolution of My Language's core features and syntax.

## Current State (v0.1.0)

The language grammar and parser support the following features:

### Implemented Syntax
- [x] Functions with parameters and return types
- [x] Let bindings (mutable and immutable)
- [x] Control flow (if/else, match, while, for, loop)
- [x] Pattern matching with guards
- [x] Structs and enums
- [x] Type annotations and inference markers
- [x] AI expressions (query, verify, generate, embed, classify)
- [x] AI model and prompt declarations
- [x] Async/await syntax
- [x] Effect types and annotations
- [x] Attributes and decorators
- [x] Contracts (requires/ensures/invariant)
- [x] Modules and imports
- [x] Generic type syntax
- [x] Operator expressions

### Type Checker Implemented
- [x] Name resolution
- [x] Basic type checking
- [x] Function call validation
- [x] AI construct validation
- [x] Error reporting with locations

## Phase 1: Core Language Completion

### 1.1 Complete Type System
**Priority: High**

```ml
// Generic types with constraints
fn map<T, U>(list: List<T>, f: fn(T) -> U) -> List<U>
where T: Clone, U: Default {
    // ...
}

// Associated types
trait Iterator {
    type Item;
    fn next(&mut self) -> Option<Self::Item>;
}

// Higher-kinded types (future)
trait Functor<F<_>> {
    fn map<A, B>(fa: F<A>, f: fn(A) -> B) -> F<B>;
}
```

Tasks:
- [ ] Constraint solving for generics
- [ ] Trait bounds implementation
- [ ] Associated type resolution
- [ ] Variance inference
- [ ] Recursive type handling

### 1.2 Effect System
**Priority: High**

```ml
// Effect annotations
fn read_file(path: String) -> String with IO {
    // ...
}

// AI effect tracking
fn analyze(text: String) -> Analysis with AI {
    ai query { prompt: "Analyze: {text}" }
}

// Effect polymorphism
fn map<T, U, E>(list: List<T>, f: fn(T) -> U with E) -> List<U> with E {
    // ...
}
```

Tasks:
- [ ] Effect inference algorithm
- [ ] Effect subtyping rules
- [ ] Effect polymorphism
- [ ] Effect handlers (algebraic effects)

### 1.3 Pattern Matching Enhancements
**Priority: Medium**

```ml
// Or-patterns
match value {
    Some(1 | 2 | 3) => "small",
    Some(n) if n > 100 => "large",
    Some(n) => "medium",
    None => "empty",
}

// Pattern bindings
match point {
    Point { x, y } @ p if p.is_origin() => "origin",
    Point { x, y } => "point at ({x}, {y})",
}

// Slice patterns
match list {
    [] => "empty",
    [x] => "singleton",
    [first, .., last] => "multiple",
}
```

Tasks:
- [ ] Or-patterns
- [ ] @ bindings
- [ ] Slice/array patterns
- [ ] Exhaustiveness checking improvements

## Phase 2: AI Language Features

### 2.1 AI Type System
**Priority: High**

```ml
// AI result types with confidence
type AIResult<T> = {
    value: T,
    confidence: Float,
    model: String,
    tokens_used: Int,
}

// Validated AI types
type ValidatedJSON = AI<JSON> with Validated;

// AI type constraints
fn get_summary(text: String) -> AI<String>
where AI: MaxTokens<100>, AI: Temperature<0.3> {
    ai generate { prompt: "Summarize: {text}" }
}
```

### 2.2 Prompt Templates
**Priority: High**

```ml
// Typed prompt parameters
prompt summarize<T: Display>(content: T, max_words: Int = 100) -> String {
    """
    Summarize the following content in at most {max_words} words:

    {content}

    Provide a concise summary.
    """
}

// Prompt composition
prompt analyze_and_summarize<T>(content: T) -> Analysis {
    let analysis = analyze!(content);
    let summary = summarize!(content);
    combine!(analysis, summary)
}
```

### 2.3 AI Pipelines
**Priority: Medium**

```ml
// Pipeline syntax
let result = text
    |> ai embed
    |> find_similar(database, _)
    |> ai generate { context: _ }
    |> validate_output;

// Streaming AI
async fn stream_response(prompt: String) -> Stream<String> with AI {
    ai stream {
        prompt: prompt
        on_token: |token| yield token
    }
}
```

## Phase 3: Advanced Features

### 3.1 Metaprogramming
**Priority: Medium**

```ml
// Compile-time computation
const fn factorial(n: Int) -> Int {
    if n <= 1 { 1 } else { n * factorial(n - 1) }
}

// Macros
macro define_getter($field:ident, $type:ty) {
    fn get_$field(&self) -> $type {
        self.$field
    }
}

// Compile-time AI (experimental)
comptime {
    let schema = ai generate { prompt: "Generate JSON schema for User" };
    define_struct_from_schema!(User, schema)
}
```

### 3.2 Dependent Types (Experimental)
**Priority: Low**

```ml
// Length-indexed vectors
type Vec<T, const N: usize>;

fn concat<T, const M: usize, const N: usize>(
    a: Vec<T, M>,
    b: Vec<T, N>
) -> Vec<T, M + N> {
    // ...
}

// Refined types
type NonEmpty<T> = List<T> where len > 0;
type Positive = Int where self > 0;
```

### 3.3 Concurrency Enhancements
**Priority: Medium**

```ml
// Structured concurrency
async fn parallel_ai_queries(items: List<String>) -> List<Result> {
    scope {
        items.map(|item| spawn { ai query { prompt: item } })
    }.await_all()
}

// Actor model
actor Counter {
    state count: Int = 0;

    fn increment(&mut self) {
        self.count += 1;
    }

    fn get(&self) -> Int {
        self.count
    }
}
```

## Language Stability

### Stable Features (v1.0)
Features that will be stabilized for v1.0:
- Core syntax (functions, types, control flow)
- Basic AI expressions
- Pattern matching
- Module system
- Effect annotations

### Experimental Features
Features under active development:
- Compile-time AI
- Dependent types
- Effect handlers
- Advanced metaprogramming

### Future Considerations
Features being researched:
- Linear types
- Session types
- Gradual typing escape hatches
- Hot code reloading

## Breaking Changes Policy

Until v1.0:
- Breaking changes may occur between minor versions
- Migration guides will be provided
- Deprecation warnings will precede removals

After v1.0:
- Semantic versioning strictly followed
- Breaking changes only in major versions
- Long deprecation periods

## Syntax Evolution Examples

### Before/After Comparisons

**AI Expressions (v0.1 → v1.0)**
```ml
// v0.1 (current)
let result = ai query { prompt: "Hello" };

// v1.0 (planned)
let result = ai "Hello";  // Short form
let result = ai {         // Block form with options
    "Hello"
    model: "gpt-4"
    temperature: 0.7
};
```

**Error Handling**
```ml
// v0.1 (current)
match risky_operation() {
    Ok(value) => use(value),
    Err(e) => handle(e),
}

// v1.0 (planned - with ? operator)
let value = risky_operation()?;
use(value)
```

## Feedback

Language design is driven by community feedback. Please share your thoughts:
- GitHub Discussions for feature requests
- RFCs for major changes
- Discord for real-time discussion
