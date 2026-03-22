# My Language Complete Specification

## Table of Contents

1. [Introduction](#introduction)
2. [Dialect Overview](#dialect-overview)
3. [Solo Specification](#solo-specification)
4. [Duet Specification](#duet-specification)
5. [Ensemble Specification](#ensemble-specification)
6. [Type System](#type-system)
7. [Affine Types](#affine-types)
8. [Contract System](#contract-system)
9. [Concurrency Model](#concurrency-model)
10. [Standard Library](#standard-library)

## Introduction

My Language is a family of four programming language dialects designed for progressive complexity:

- **Me**: Educational HTML-like language for children
- **Solo**: Core systems language with affine types and safety features
- **Duet**: Solo + neurosymbolic AI integration  
- **Ensemble**: Multi-agent orchestration and collaboration

### Design Principles

1. **Progressive Complexity**: Start simple, add features as needed
2. **Safety First**: Strong type system, affine types, contracts
3. **AI Integration**: First-class support for AI-assisted development
4. **Performance**: Zero-cost abstractions, compile-time execution
5. **Ergonomics**: Clear syntax, helpful error messages

## Dialect Overview

### Comparison Matrix

| Feature | Me | Solo | Duet | Ensemble |
|---------|----|----|------|----------|
| Target Audience | Children | Systems programmers | AI developers | Multi-agent systems |
| Complexity | Low | Medium | High | Very High |
| Type System | Simple | Full | Full + AI | Full + Agents |
| Concurrency | None | M:N Threading | M:N + AI | Distributed |
| AI Features | None | None | Full | Full + Orchestration |

## Solo Specification

### Syntax Overview

```solo
fn main() {
    println("Hello, World!");
}
```

### Core Features

#### 1. Variables and Types

```solo
let x: i32 = 42;
let mut counter = 0;
let tuple: (i32, str) = (1, "hello");
```

#### 2. Functions

```solo
fn add(a: i32, b: i32) -> i32 {
    a + b
}

fn generic<T>(value: T) -> T {
    value
}
```

#### 3. Control Flow

```solo
if condition {
    // ...
} else {
    // ...
}

match value {
    1 => "one",
    2 => "two",
    _ => "other"
}

for i in 0..10 {
    println(i);
}
```

#### 4. Structs and Enums

```solo
struct Point {
    x: i32,
    y: i32
}

enum Option<T> {
    Some(T),
    None
}
```

#### 5. Traits

```solo
trait Printable {
    fn print(&self);
}

impl Printable for Point {
    fn print(&self) {
        println("({}, {})", self.x, self.y);
    }
}
```

#### 6. Contracts

```solo
fn divide(a: i32, b: i32) -> i32
where
    pre b != 0,
    post result * b == a
{
    a / b
}
```

#### 7. Affine Types

```solo
struct FileHandle {
    handle: affine File
}

impl FileHandle {
    fn read(self) -> str {
        // Consumes self, ensuring single use
        self.handle.read_all()
    }
}
```

#### 8. Async/Await

```solo
async fn fetch_data(url: str) -> str {
    let response = http::get(url).await;
    response.body()
}
```

#### 9. Comptime

```solo
comptime fn factorial(n: u64) -> u64 {
    if n <= 1 { 1 } else { n * factorial(n-1) }
}

const FACT_10: u64 = comptime factorial(10);
```

## Duet Specification

Duet extends Solo with neurosymbolic AI integration.

### AI Synthesis

```duet
@synth(spec: |x| x * 2)
fn double(x: i32) -> i32;

@synth(examples: [(0,0), (1,1), (2,4), (3,9)])
fn square(x: i32) -> i32;
```

### AI Verification

```duet
@verify(property: |x, result| result >= 0)
fn abs(x: i32) -> i32 {
    if x < 0 { -x } else { x }
}
```

### Intent Programming

```duet
fn main() {
    let sorted = intent("sort this list", {
        inputs: [5, 2, 8, 1],
        outputs: [i32]
    });
}
```

### AI Optimization

```duet
#[ai_optimize(strategy: "minimize_latency")]
fn fibonacci(n: u64) -> u64 {
    if n <= 1 { n } else { fibonacci(n-1) + fibonacci(n-2) }
}
```

### Hybrid Reasoning

```duet
fn classify(data: Data) -> Label {
    hybrid {
        symbolic: rule_based_classifier(data),
        neural: ml_classifier(data),
        fusion: weighted(0.3, 0.7)
    }
}
```

## Ensemble Specification

Two variants exist:

### Variant A: Agent as Construct

```ensemble
agent ReporterAgent {
    state {
        persistent topic: str;
        ephemeral sources: Vec<Source>;
    }

    capabilities {
        research(topic: str) -> Article: action;
    }

    goals {
        high investigate: find_stories();
    }

    communication {
        on ArticleRequest {
            let article = research(topic);
            send article to sender;
        }
    }
}

workflow Newroom {
    stages {
        ReporterAgent -> EditorAgent -> FactCheckerAgent -> PublisherAgent
    }

    coordination {
        consensus(threshold: 0.85) on verification;
    }
}
```

### Variant B: Mode-Based

Regular Duet code with attributes:

```duet
#![ensemble(mode: "collaborative", agents: ["reporter", "editor"])]

#[agent("reporter")]
fn research(topic: str) -> Article {
    // Implementation
}

#[workflow({
    "research": { agent: "reporter" },
    "edit": { agent: "editor", depends_on: ["research"] }
})]
fn pipeline(topic: str) -> Article {
    // Pipeline logic
}
```

Configuration in `my.toml`:

```toml
[ensemble]
mode = "collaborative"

[[ensemble.agents]]
name = "reporter"
role = "research"
model = "claude-3-opus"
```

## Type System

### Primitive Types

- Integers: `i8`, `i16`, `i32`, `i64`, `i128`, `isize`
- Unsigned: `u8`, `u16`, `u32`, `u64`, `u128`, `usize`
- Floats: `f32`, `f64`
- Boolean: `bool`
- Character: `char`
- String: `str`
- Unit: `()`
- Never: `!`

### Compound Types

- Tuples: `(T1, T2, ...)`
- Arrays: `[T; N]`
- Slices: `[T]`
- References: `&T`, `&mut T`
- Functions: `fn(T1, T2) -> R`

### Generic Types

```solo
struct Container<T> {
    value: T
}

fn identity<T>(x: T) -> T { x }
```

### Type Inference

The compiler infers types where possible:

```solo
let x = 42;  // i32
let y = 3.14;  // f64
```

## Affine Types

Affine types ensure resources are used at most once.

### Declaration

```solo
struct Resource {
    data: affine Handle
}
```

### Rules

1. Affine values can be used at most once
2. No implicit copying
3. Explicit consumption required
4. Compiler enforces linearity

### Example

```solo
fn use_resource(r: Resource) {
    r.data.consume();  // OK: consumed once
    // r.data.consume();  // ERROR: already consumed
}
```

## Contract System

### Preconditions

```solo
fn sqrt(x: f64) -> f64
where
    pre x >= 0.0
{
    // Implementation
}
```

### Postconditions

```solo
fn abs(x: i32) -> i32
where
    post result >= 0
{
    if x < 0 { -x } else { x }
}
```

### Invariants

```solo
impl BankAccount
where
    invariant self.balance >= 0
{
    fn withdraw(&mut self, amount: i32) -> bool {
        if self.balance >= amount {
            self.balance -= amount;
            true
        } else {
            false
        }
    }
}
```

## Concurrency Model

### M:N Threading

Green threads multiplexed over OS threads:

```solo
async fn task() {
    // Runs on M:N scheduler
}

fn main() {
    spawn_task(task());
}
```

### Thread Safety

- `Send`: Can be sent across threads
- `Sync`: Can be shared across threads
- Compiler enforces thread safety

### Synchronization

```solo
let mutex = Mutex::new(0);
let guard = mutex.lock();
*guard += 1;
```

## Standard Library

### Core Modules

- `std::io`: Input/output
- `std::fs`: File system
- `std::net`: Networking
- `std::collections`: Data structures
- `std::thread`: Threading
- `std::sync`: Synchronization
- `std::async`: Async runtime

### Duet Modules

- `std::duet::synth`: Synthesis
- `std::duet::verify`: Verification
- `std::duet::intent`: Intent API
- `std::duet::hybrid`: Hybrid reasoning

### Ensemble Modules

- `std::ensemble::agent`: Agent runtime
- `std::ensemble::workflow`: Workflow orchestration
- `std::ensemble::coordination`: Coordination primitives
