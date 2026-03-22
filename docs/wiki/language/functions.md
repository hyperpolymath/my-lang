# Functions and Closures

Functions are first-class values in My Language, supporting generics, effects, contracts, and powerful closure semantics.

## Function Basics

### Defining Functions

```ml
// Basic function
fn greet(name: String) -> String {
    "Hello, {name}!"
}

// Multiple parameters
fn add(a: Int, b: Int) -> Int {
    a + b
}

// No return value (returns unit)
fn log_message(msg: String) {
    print("[LOG] {msg}");
}

// Explicit unit return
fn log_message(msg: String) -> () {
    print("[LOG] {msg}");
}
```

### Return Values

```ml
// Implicit return (last expression)
fn square(x: Int) -> Int {
    x * x
}

// Explicit return
fn abs(x: Int) -> Int {
    if x < 0 {
        return -x;
    }
    x
}

// Early return
fn find_first_even(numbers: List<Int>) -> Option<Int> {
    for n in numbers {
        if n % 2 == 0 {
            return Some(n);
        }
    }
    None
}
```

### Parameters

```ml
// Required parameters
fn divide(dividend: Int, divisor: Int) -> Float {
    dividend as Float / divisor as Float
}

// Default parameters
fn greet(name: String, greeting: String = "Hello") -> String {
    "{greeting}, {name}!"
}

greet("Alice")                    // "Hello, Alice!"
greet("Bob", greeting: "Hi")      // "Hi, Bob!"

// Named arguments (at call site)
fn create_user(name: String, age: Int, admin: Bool = false) -> User {
    User { name, age, admin }
}

let user = create_user(
    name: "Alice",
    age: 30,
    admin: true,
);
```

### Variadic Parameters (Rest)

```ml
fn sum(numbers: ..Int) -> Int {
    let mut total = 0;
    for n in numbers {
        total += n;
    }
    total
}

sum(1, 2, 3, 4, 5)  // 15

fn log(level: String, messages: ..String) {
    for msg in messages {
        print("[{level}] {msg}");
    }
}
```

## Generic Functions

### Type Parameters

```ml
fn identity<T>(value: T) -> T {
    value
}

fn swap<A, B>(pair: (A, B)) -> (B, A) {
    let (a, b) = pair;
    (b, a)
}

fn first<T>(list: List<T>) -> Option<T> {
    if list.is_empty() {
        None
    } else {
        Some(list[0].clone())
    }
}
```

### Trait Bounds

```ml
// Single bound
fn print_all<T: Display>(items: List<T>) {
    for item in items {
        print(item.to_string());
    }
}

// Multiple bounds
fn compare_and_display<T: Ord + Display>(a: T, b: T) -> String {
    if a < b {
        "{a} < {b}"
    } else if a > b {
        "{a} > {b}"
    } else {
        "{a} = {b}"
    }
}

// Where clause
fn complex_generic<T, U, V>(t: T, u: U, v: V) -> Result<V, Error>
where
    T: Clone + Debug,
    U: Into<T>,
    V: Default + FromStr,
{
    // ...
}
```

### Associated Type Bounds

```ml
fn collect_items<I>(iter: I) -> List<I::Item>
where
    I: Iterator,
    I::Item: Clone,
{
    iter.collect()
}
```

## Function Types

### Function Pointers

```ml
// Function type syntax
type IntToInt = fn(Int) -> Int;
type Predicate<T> = fn(T) -> Bool;
type BinaryOp = fn(Int, Int) -> Int;

// Using function types
fn apply(f: fn(Int) -> Int, x: Int) -> Int {
    f(x)
}

fn double(x: Int) -> Int { x * 2 }

let result = apply(double, 5);  // 10
```

### Higher-Order Functions

```ml
fn map<T, U>(list: List<T>, f: fn(T) -> U) -> List<U> {
    let mut result = List::new();
    for item in list {
        result.push(f(item));
    }
    result
}

fn filter<T>(list: List<T>, pred: fn(&T) -> Bool) -> List<T> {
    let mut result = List::new();
    for item in list {
        if pred(&item) {
            result.push(item);
        }
    }
    result
}

fn fold<T, U>(list: List<T>, init: U, f: fn(U, T) -> U) -> U {
    let mut acc = init;
    for item in list {
        acc = f(acc, item);
    }
    acc
}

// Usage
let numbers = [1, 2, 3, 4, 5];
let doubled = map(numbers, |x| x * 2);           // [2, 4, 6, 8, 10]
let evens = filter(numbers, |x| x % 2 == 0);     // [2, 4]
let sum = fold(numbers, 0, |acc, x| acc + x);    // 15
```

## Closures

### Basic Closures

```ml
// Lambda syntax
let add = |a, b| a + b;
let square = |x| x * x;
let greet = |name| print("Hello, {name}!");

// With type annotations
let add: fn(Int, Int) -> Int = |a, b| a + b;
let square: fn(Int) -> Int = |x: Int| -> Int { x * x };

// Multi-statement closure
let process = |x| {
    let doubled = x * 2;
    let squared = doubled * doubled;
    squared + 1
};
```

### Capturing Variables

```ml
fn make_adder(n: Int) -> fn(Int) -> Int {
    |x| x + n  // Captures n
}

let add_5 = make_adder(5);
let add_10 = make_adder(10);

add_5(3)   // 8
add_10(3)  // 13
```

### Capture Modes

```ml
// Capture by reference (default for reads)
let list = [1, 2, 3];
let get_length = || list.len();  // Borrows list

// Capture by mutable reference
let mut counter = 0;
let increment = || { counter += 1; };
increment();
increment();
print(counter);  // 2

// Move capture (takes ownership)
let data = expensive_data();
let processor = move || {
    consume(data)  // data is moved into closure
};
// data no longer accessible here
```

### Closure Traits

```ml
// Fn: Can be called multiple times, borrows captures
fn call_multiple<F: Fn()>(f: F) {
    f();
    f();
    f();
}

// FnMut: Can be called multiple times, may mutate captures
fn call_and_mutate<F: FnMut()>(mut f: F) {
    f();
    f();
}

// FnOnce: Can only be called once, may consume captures
fn call_once<F: FnOnce() -> T>(f: F) -> T {
    f()
}
```

## Async Functions

```ml
async fn fetch_data(url: String) -> Result<Data, Error> {
    let response = http::get(url).await?;
    let data = response.json().await?;
    Ok(data)
}

async fn fetch_all(urls: List<String>) -> List<Data> {
    let futures = urls.map(|url| fetch_data(url));
    join_all(futures).await
}

// Async closures
let fetch = async |url| {
    http::get(url).await?.json().await
};
```

## Functions with Effects

```ml
fn read_config() -> Config with IO {
    let content = read_file("config.toml");
    parse_toml(content)
}

fn analyze(text: String) -> Analysis with AI {
    ai query { prompt: "Analyze: {text}" }
}

fn complex_operation() with IO, AI, Network {
    let config = read_config();
    let data = fetch_remote(config.url);
    analyze(data)
}

// Effect-polymorphic functions
fn retry<T, E>(times: Int, f: fn() -> T with E) -> Result<T, Error> with E {
    for _ in 0..times {
        match catch(f) {
            Ok(result) => return Ok(result),
            Err(_) => continue,
        }
    }
    Err(Error::MaxRetriesExceeded)
}
```

## Function Contracts

### Preconditions

```ml
fn sqrt(x: Float) -> Float
requires x >= 0.0
{
    x.sqrt()
}

fn divide(a: Int, b: Int) -> Int
requires b != 0
{
    a / b
}

fn get(list: List<T>, index: Int) -> T
requires index >= 0
requires index < list.len()
{
    list[index]
}
```

### Postconditions

```ml
fn abs(x: Int) -> Int
ensures result >= 0
{
    if x < 0 { -x } else { x }
}

fn factorial(n: Int) -> Int
requires n >= 0
ensures result >= 1
{
    if n <= 1 { 1 } else { n * factorial(n - 1) }
}
```

### Combined Contracts

```ml
fn binary_search(list: List<Int>, target: Int) -> Option<Int>
requires is_sorted(list)
ensures match result {
    Some(i) => list[i] == target,
    None => !list.contains(target),
}
{
    // Implementation
}
```

## Method Syntax

### Impl Blocks

```ml
struct Point {
    x: Int,
    y: Int,
}

impl Point {
    // Constructor (associated function)
    fn new(x: Int, y: Int) -> Point {
        Point { x, y }
    }

    // Method (takes self)
    fn distance_from_origin(&self) -> Float {
        ((self.x * self.x + self.y * self.y) as Float).sqrt()
    }

    // Mutable method
    fn translate(&mut self, dx: Int, dy: Int) {
        self.x += dx;
        self.y += dy;
    }

    // Consuming method
    fn into_tuple(self) -> (Int, Int) {
        (self.x, self.y)
    }
}

// Usage
let mut p = Point::new(3, 4);
let dist = p.distance_from_origin();  // 5.0
p.translate(1, 1);
let tuple = p.into_tuple();  // (4, 5)
```

### Trait Methods

```ml
trait Drawable {
    fn draw(&self);
    fn bounds(&self) -> Rect;

    // Default implementation
    fn draw_bounded(&self) {
        let rect = self.bounds();
        draw_rect(rect);
        self.draw();
    }
}

impl Drawable for Point {
    fn draw(&self) {
        draw_pixel(self.x, self.y);
    }

    fn bounds(&self) -> Rect {
        Rect::new(self.x, self.y, 1, 1)
    }
}
```

## Operators as Functions

```ml
// Operators are syntactic sugar for trait methods
trait Add<Rhs = Self> {
    type Output;
    fn add(self, rhs: Rhs) -> Self::Output;
}

impl Add for Int {
    type Output = Int;
    fn add(self, rhs: Int) -> Int {
        // Built-in addition
    }
}

// a + b is equivalent to Add::add(a, b)
```

## Recursion

```ml
// Direct recursion
fn factorial(n: Int) -> Int {
    if n <= 1 { 1 } else { n * factorial(n - 1) }
}

// Tail recursion (optimized)
fn factorial_tail(n: Int, acc: Int = 1) -> Int {
    if n <= 1 { acc } else { factorial_tail(n - 1, n * acc) }
}

// Mutual recursion
fn is_even(n: Int) -> Bool {
    if n == 0 { true } else { is_odd(n - 1) }
}

fn is_odd(n: Int) -> Bool {
    if n == 0 { false } else { is_even(n - 1) }
}
```

## Function Attributes

```ml
#[inline]
fn small_helper(x: Int) -> Int {
    x + 1
}

#[inline(never)]
fn large_function() {
    // ...
}

#[cold]
fn error_path() {
    // Unlikely to be called
}

#[must_use]
fn important_result() -> Result<T, E> {
    // Caller must use the result
}

#[deprecated(since = "0.2.0", note = "use new_function instead")]
fn old_function() {
    // ...
}
```

## Best Practices

### 1. Prefer Small, Focused Functions

```ml
// Good: Single responsibility
fn validate_email(email: String) -> Bool { ... }
fn send_email(to: String, body: String) -> Result { ... }

// Avoid: Multiple responsibilities
fn validate_and_send_email(...) { ... }
```

### 2. Use Descriptive Names

```ml
// Good
fn calculate_monthly_payment(principal: Float, rate: Float, months: Int) -> Float

// Avoid
fn calc(p: Float, r: Float, m: Int) -> Float
```

### 3. Limit Parameters

```ml
// Good: Use a struct for many parameters
struct EmailOptions {
    to: String,
    subject: String,
    body: String,
    cc: List<String>,
    bcc: List<String>,
}

fn send_email(options: EmailOptions) -> Result

// Avoid: Too many parameters
fn send_email(to: String, subject: String, body: String, cc: List<String>, ...) -> Result
```

### 4. Document Complex Functions

```ml
/// Calculates the optimal path between two nodes using A* algorithm.
///
/// # Arguments
/// * `graph` - The graph to search
/// * `start` - Starting node ID
/// * `goal` - Goal node ID
///
/// # Returns
/// The shortest path as a list of node IDs, or None if no path exists
///
/// # Complexity
/// Time: O(E log V), Space: O(V)
fn find_path(graph: &Graph, start: NodeId, goal: NodeId) -> Option<List<NodeId>> {
    // ...
}
```
