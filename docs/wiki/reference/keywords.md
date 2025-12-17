# Keywords Reference

Complete reference for all keywords in My Language.

## Reserved Keywords

These keywords are reserved and cannot be used as identifiers.

### Declaration Keywords

| Keyword | Description | Example |
|---------|-------------|---------|
| `fn` | Function declaration | `fn add(a: Int, b: Int) -> Int` |
| `let` | Variable binding | `let x = 5;` |
| `mut` | Mutable binding modifier | `let mut x = 5;` |
| `const` | Compile-time constant | `const MAX: Int = 100;` |
| `type` | Type alias | `type UserId = Int;` |
| `struct` | Structure definition | `struct Point { x: Int, y: Int }` |
| `enum` | Enumeration definition | `enum Option<T> { Some(T), None }` |
| `trait` | Trait definition | `trait Display { fn display(&self) -> String; }` |
| `impl` | Implementation block | `impl Point { fn new() -> Point { ... } }` |
| `mod` | Module declaration | `mod utils;` |

### Control Flow Keywords

| Keyword | Description | Example |
|---------|-------------|---------|
| `if` | Conditional branch | `if x > 0 { "positive" }` |
| `else` | Alternative branch | `if x > 0 { ... } else { ... }` |
| `match` | Pattern matching | `match value { Some(x) => x, None => 0 }` |
| `for` | For loop | `for item in list { ... }` |
| `while` | While loop | `while condition { ... }` |
| `loop` | Infinite loop | `loop { if done { break; } }` |
| `break` | Exit loop | `break;` or `break value;` |
| `continue` | Skip iteration | `continue;` |
| `return` | Return from function | `return value;` |
| `in` | Iterator/range expression | `for i in 0..10` |

### Concurrency Keywords

| Keyword | Description | Example |
|---------|-------------|---------|
| `async` | Async function/block | `async fn fetch() -> Data` |
| `await` | Await future | `let data = fetch().await;` |
| `spawn` | Spawn task | `spawn { background_work() }` |
| `go` | Lightweight task | `go { concurrent_work() }` |
| `yield` | Yield execution | `yield;` |

### Module Keywords

| Keyword | Description | Example |
|---------|-------------|---------|
| `use` | Import items | `use std::collections::HashMap;` |
| `pub` | Public visibility | `pub fn public_api() { }` |
| `self` | Current module/instance | `self.field` |
| `super` | Parent module | `use super::parent_fn;` |
| `crate` | Crate root | `use crate::utils;` |
| `as` | Rename import/cast | `use HashMap as Map;` |

### Literal Keywords

| Keyword | Description | Example |
|---------|-------------|---------|
| `true` | Boolean true | `let active = true;` |
| `false` | Boolean false | `let disabled = false;` |

### Special Keywords

| Keyword | Description | Example |
|---------|-------------|---------|
| `_` | Wildcard pattern | `match x { _ => default() }` |
| `where` | Type constraints | `fn f<T>() where T: Clone` |

## AI Keywords

These keywords are used for AI-specific features.

| Keyword | Description | Example |
|---------|-------------|---------|
| `ai` | AI expression prefix | `ai query { prompt: "..." }` |
| `query` | AI query operation | `ai query { prompt: text }` |
| `verify` | AI verification | `ai verify { input: data }` |
| `generate` | AI generation | `ai generate { prompt: text }` |
| `embed` | AI embedding | `ai embed(text)` |
| `classify` | AI classification | `ai classify { input: text }` |
| `optimize` | AI optimization | `ai optimize { target: fn }` |
| `test` | AI test generation | `ai test { function: fn }` |
| `infer` | AI type inference | `ai infer { data: value }` |
| `constrain` | AI constraints | `ai constrain { ... }` |
| `validate` | AI validation | `ai validate { ... }` |
| `prompt` | Prompt template | `prompt name(args) { ... }` |
| `ai_model` | AI model declaration | `ai_model Name { ... }` |
| `comptime` | Compile-time AI | `comptime { ai query {...} }` |

## Contract Keywords

| Keyword | Description | Example |
|---------|-------------|---------|
| `requires` | Precondition | `fn f(x: Int) requires x > 0` |
| `ensures` | Postcondition | `fn f() -> Int ensures result > 0` |
| `invariant` | Type invariant | `struct X { } invariant self.valid()` |

## Effect Keywords

| Keyword | Description | Example |
|---------|-------------|---------|
| `with` | Effect annotation | `fn f() -> T with IO` |
| `effect` | Effect definition | `effect IO { fn read(); }` |
| `handle` | Effect handler | `handle f() { op() => ... }` |

## Contextual Keywords

These have special meaning only in certain contexts.

| Keyword | Context | Example |
|---------|---------|---------|
| `result` | In ensures clause | `ensures result > 0` |
| `self` | In methods | `fn method(&self)` |
| `Self` | In impl blocks | `fn new() -> Self` |

## Keyword Usage Examples

### fn

```ml
// Basic function
fn greet(name: String) -> String {
    "Hello, {name}!"
}

// Generic function
fn identity<T>(value: T) -> T {
    value
}

// Async function
async fn fetch(url: String) -> Data {
    http::get(url).await
}

// Function with effects
fn read_file(path: String) -> String with IO {
    std::fs::read_to_string(path)
}

// Function with contracts
fn sqrt(x: Float) -> Float
requires x >= 0.0
ensures result >= 0.0
{
    x.sqrt()
}
```

### let

```ml
// Immutable binding
let x = 5;
let name = "Alice";

// Mutable binding
let mut counter = 0;
counter += 1;

// With type annotation
let age: Int = 30;

// Destructuring
let (a, b) = (1, 2);
let Point { x, y } = point;
let [first, ..rest] = list;

// Pattern with guard
let Some(value) = optional else { return; };
```

### match

```ml
// Basic matching
let result = match value {
    0 => "zero",
    1 => "one",
    n if n < 0 => "negative",
    _ => "other",
};

// Enum matching
match option {
    Some(x) => use(x),
    None => default(),
}

// Struct matching
match user {
    User { name, age: 18.. } => adult(name),
    User { name, .. } => minor(name),
}

// Multiple patterns
match char {
    'a' | 'e' | 'i' | 'o' | 'u' => "vowel",
    'A'..='Z' => "uppercase",
    _ => "other",
}
```

### struct

```ml
// Basic struct
struct Point {
    x: Int,
    y: Int,
}

// Generic struct
struct Container<T> {
    value: T,
}

// Tuple struct
struct Color(u8, u8, u8);

// Unit struct
struct Marker;

// With visibility
struct User {
    pub name: String,
    pub email: String,
    password_hash: String,  // private
}

// With derive
#[derive(Debug, Clone, PartialEq)]
struct Data {
    field: Int,
}
```

### enum

```ml
// Simple enum
enum Direction {
    North,
    South,
    East,
    West,
}

// With data
enum Message {
    Text(String),
    Image { url: String, width: Int },
    Quit,
}

// Generic enum
enum Result<T, E> {
    Ok(T),
    Err(E),
}

// Recursive enum
enum List<T> {
    Cons(T, Box<List<T>>),
    Nil,
}
```

### trait and impl

```ml
// Trait definition
trait Display {
    fn display(&self) -> String;
}

// Trait with default method
trait Greet {
    fn name(&self) -> String;

    fn greet(&self) -> String {
        "Hello, {self.name()}!"
    }
}

// Trait with associated type
trait Iterator {
    type Item;
    fn next(&mut self) -> Option<Self::Item>;
}

// Implementation
impl Display for Point {
    fn display(&self) -> String {
        "({self.x}, {self.y})"
    }
}

// Inherent impl
impl Point {
    fn new(x: Int, y: Int) -> Point {
        Point { x, y }
    }

    fn origin() -> Point {
        Point { x: 0, y: 0 }
    }
}
```

### async/await

```ml
// Async function
async fn fetch_user(id: Int) -> User {
    let response = http::get("/users/{id}").await;
    response.json().await
}

// Await in expression
let data = fetch_data().await?;

// Concurrent awaits
let (a, b, c) = join!(
    fetch_a(),
    fetch_b(),
    fetch_c(),
).await;

// Select first
let result = select! {
    x = channel1.recv() => handle_x(x),
    y = channel2.recv() => handle_y(y),
    _ = timeout(1000) => Err(Timeout),
};
```

### use

```ml
// Simple import
use std::collections::HashMap;

// Multiple imports
use std::collections::{HashMap, HashSet, BTreeMap};

// Glob import
use std::io::*;

// Aliased import
use std::collections::HashMap as Map;

// Nested imports
use std::{
    io::{Read, Write},
    collections::HashMap,
};

// Re-export
pub use internal::Type;
```

## Soft Keywords

Words that have special meaning in certain contexts but can be used as identifiers elsewhere.

| Word | Context |
|------|---------|
| `union` | Reserved for future union types |
| `macro` | Reserved for future macro system |
| `move` | Capture mode in closures |
| `ref` | Reference patterns |
| `box` | Box patterns (future) |
| `dyn` | Dynamic trait objects |
| `static` | Static lifetime / items |
| `extern` | External functions |

## Future Reserved Keywords

These are reserved for potential future use:

- `abstract`
- `become`
- `do`
- `final`
- `macro`
- `override`
- `priv`
- `try`
- `typeof`
- `unsized`
- `virtual`
