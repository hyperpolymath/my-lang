# Getting Started with My Language

## Installation

### From Source

```bash
git clone https://github.com/Hyperpolymath/me-dialect-playground
cd me-dialect-playground
cargo build --release
cargo install --path .
```

### Using Cargo

```bash
cargo install my-lang
```

## Your First Program

Create `hello.solo`:

```solo
fn main() {
    println("Hello, World!");
}
```

Compile and run:

```bash
my-lang run hello.solo
```

## Basic Concepts

### Variables

```solo
let x = 42;           // Immutable
let mut counter = 0;  // Mutable
counter = counter + 1;
```

### Functions

```solo
fn add(a: i32, b: i32) -> i32 {
    a + b  // Last expression is return value
}

fn greet(name: str) {
    println("Hello, " + name + "!");
}
```

### Control Flow

```solo
// If expressions
let result = if x > 0 {
    "positive"
} else {
    "non-positive"
};

// Pattern matching
match value {
    0 => println("zero"),
    1..=9 => println("single digit"),
    _ => println("other")
}

// Loops
for i in 0..10 {
    println(i);
}

while counter < 100 {
    counter = counter + 1;
}
```

### Data Structures

```solo
// Struct
struct Point {
    x: i32,
    y: i32
}

let p = Point { x: 10, y: 20 };

// Enum
enum Status {
    Success,
    Error(str)
}

let s = Status::Success;
```

## Intermediate Topics

### Generics

```solo
fn first<T>(list: [T]) -> T {
    list[0]
}

struct Pair<T, U> {
    first: T,
    second: U
}
```

### Traits

```solo
trait Drawable {
    fn draw(&self);
}

impl Drawable for Circle {
    fn draw(&self) {
        println("Drawing circle");
    }
}
```

### Error Handling

```solo
enum Result<T, E> {
    Ok(T),
    Err(E)
}

fn divide(a: i32, b: i32) -> Result<i32, str> {
    if b == 0 {
        Result::Err("Division by zero")
    } else {
        Result::Ok(a / b)
    }
}

// Using ? operator
fn parse_and_divide(s: str) -> Result<i32, str> {
    let num = parse_int(s)?;  // Propagates error
    divide(100, num)
}
```

## Advanced Features

### Contracts

```solo
fn sqrt(x: f64) -> f64
where
    pre x >= 0.0,
    post result * result <= x + 0.001,
    post result * result >= x - 0.001
{
    // Implementation verified against contract
}
```

### Affine Types

```solo
struct File {
    handle: affine FileHandle
}

impl File {
    fn read(self) -> str {
        // Consumes self - can only read once
        self.handle.read_all()
    }
}
```

### Async/Await

```solo
async fn fetch_data(url: str) -> Result<str, Error> {
    let response = http::get(url).await?;
    Ok(response.body())
}

async fn main() {
    let data = fetch_data("https://api.example.com").await;
    println(data);
}
```

### Comptime

```solo
comptime fn factorial(n: u64) -> u64 {
    if n <= 1 { 1 } else { n * factorial(n-1) }
}

const FACT_10: u64 = comptime factorial(10);  // Computed at compile time
```

## Moving to Duet

Enable AI features:

```duet
// Synthesis
@synth(spec: |x| x * 2)
fn double(x: i32) -> i32;

// Verification
@verify(property: |arr| is_sorted(arr))
fn sort(arr: &mut [i32]) {
    arr.sort();
}

// Intent
fn main() {
    let result = intent("find prime numbers under 100", {
        outputs: [i32]
    });
}
```

## Next Steps

- Read the [Language Specification](../specs/LANGUAGE_SPECIFICATION.md)
- Explore [Examples](../../examples/)
- Try [Advanced Tutorial](ADVANCED_TUTORIAL.md)
- Learn about [Duet AI Features](DUET_TUTORIAL.md)
- Explore [Ensemble Multi-Agent](ENSEMBLE_TUTORIAL.md)
