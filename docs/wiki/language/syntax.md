# Syntax Overview

Complete syntax reference for My Language.

## Lexical Elements

### Comments

```ml
// Line comment

/* Block comment */

/*
 * Multi-line
 * block comment
 */

/// Documentation comment for items
/// Supports **markdown** formatting
fn documented_function() { }

//! Module-level documentation comment
//! Describes the current module
```

### Identifiers

```ml
// Standard identifiers
let name = 1;
let camelCase = 2;
let snake_case = 3;
let PascalCase = 4;
let _private = 5;
let __dunder__ = 6;

// Unicode identifiers
let café = "coffee";
let 日本語 = "Japanese";
let émoji = "🎉";
```

### Keywords

Reserved keywords:
```
fn       let      mut      const    type
struct   enum     trait    impl     where
if       else     match    for      while
loop     break    continue return   in
async    await    spawn    go       yield
use      mod      pub      self     super
crate    as       true     false    _
```

AI keywords:
```
ai       query    verify   generate embed
classify optimize test     infer    constrain
validate prompt   ai_model comptime
```

### Literals

```ml
// Integers
let decimal = 42;
let hex = 0xFF;
let octal = 0o77;
let binary = 0b1010;
let with_separator = 1_000_000;

// Floats
let float = 3.14;
let scientific = 6.022e23;
let negative_exp = 1.0e-10;

// Strings
let simple = "Hello, World!";
let escaped = "Line 1\nLine 2\tTabbed";
let interpolated = "Hello, {name}!";
let raw = r#"Raw string with "quotes""#;
let multiline = """
    This is a
    multi-line string
    """;

// Characters
let char = 'a';
let unicode_char = '🎉';
let escaped_char = '\n';

// Booleans
let yes = true;
let no = false;
```

### Operators

```ml
// Arithmetic
+   -   *   /   %   **

// Comparison
==  !=  <   >   <=  >=

// Logical
&&  ||  !

// Bitwise
&   |   ^   ~   <<  >>

// Assignment
=   +=  -=  *=  /=  %=
&=  |=  ^=  <<= >>=

// Other
..      // Range (exclusive)
..=     // Range (inclusive)
?       // Error propagation
|>      // Pipe operator
=>      // Match arm / lambda
->      // Return type
::      // Path separator
```

## Expressions

### Arithmetic

```ml
let sum = a + b;
let product = x * y;
let power = base ** exponent;
let remainder = n % divisor;

// Operator precedence (highest to lowest)
// 1. ** (right associative)
// 2. * / %
// 3. + -
// 4. << >>
// 5. & | ^
// 6. == != < > <= >=
// 7. &&
// 8. ||
```

### Comparison and Logical

```ml
let equal = a == b;
let not_equal = a != b;
let greater = a > b;

let both = a && b;
let either = a || b;
let negated = !a;

// Short-circuit evaluation
let result = expensive() && cheap();  // cheap() only called if expensive() is true
```

### Control Flow Expressions

```ml
// If expression (returns a value)
let max = if a > b { a } else { b };

// Match expression
let description = match status {
    Status::Active => "active",
    Status::Inactive => "inactive",
    Status::Pending { since } => "pending since {since}",
};

// Block expression
let result = {
    let x = compute();
    let y = transform(x);
    y * 2  // Last expression is the value
};
```

### Function Calls

```ml
// Basic call
let result = function(arg1, arg2);

// Method call
let length = string.len();

// Chained calls
let result = data
    .filter(|x| x > 0)
    .map(|x| x * 2)
    .sum();

// Named arguments
let user = create_user(
    name: "Alice",
    age: 30,
    admin: false,
);

// Trailing lambda
let result = list.map { |item|
    item.process()
};
```

### Closures/Lambdas

```ml
// Basic lambda
let add = |a, b| a + b;

// With type annotations
let add: fn(Int, Int) -> Int = |a, b| a + b;

// Multi-line
let process = |item| {
    let validated = validate(item);
    transform(validated)
};

// Capturing environment
let multiplier = 3;
let triple = |x| x * multiplier;
```

### AI Expressions

```ml
// Quick query
ai! { "prompt text" }

// Full query
ai query {
    prompt: "text"
    context: variable
    model: "gpt-4"
    temperature: 0.7
    max_tokens: 100
}

// Other AI operations
ai verify { input: data, constraint: "description" }
ai generate { prompt: "text", options: opts }
ai embed(text)
ai classify { input: text, categories: list }

// Prompt invocation
let result = my_prompt!(arg1, arg2);
```

## Statements

### Let Bindings

```ml
// Immutable (default)
let x = 5;

// Mutable
let mut counter = 0;

// With type annotation
let name: String = "Alice";

// Destructuring
let (a, b) = tuple;
let Point { x, y } = point;
let [first, second, ..rest] = list;
```

### Assignment

```ml
// Simple assignment (mutable variables only)
counter = counter + 1;

// Compound assignment
counter += 1;
total *= 2;
flags |= NEW_FLAG;

// Destructuring assignment
(a, b) = (b, a);  // Swap
```

### Control Flow Statements

```ml
// If statement
if condition {
    do_something();
} else if other_condition {
    do_other();
} else {
    do_default();
}

// Match statement
match value {
    Pattern1 => action1(),
    Pattern2 => action2(),
    _ => default_action(),
}

// While loop
while condition {
    process();
}

// For loop
for item in collection {
    process(item);
}

for i in 0..10 {
    print(i);
}

// Loop (infinite)
loop {
    if done() {
        break;
    }
}

// Loop with value
let result = loop {
    if found() {
        break value;
    }
};

// Break and continue
for i in 0..100 {
    if skip(i) { continue; }
    if done(i) { break; }
    process(i);
}
```

### Return

```ml
fn early_return(x: Int) -> Int {
    if x < 0 {
        return 0;  // Early return
    }
    x * 2  // Implicit return (last expression)
}
```

## Declarations

### Functions

```ml
// Basic function
fn name(param: Type) -> ReturnType {
    body
}

// With generics
fn generic<T: Trait>(value: T) -> T {
    value
}

// With effects
fn effectful() -> Int with IO, AI {
    // ...
}

// With contracts
fn constrained(x: Int) -> Int
requires x > 0
ensures result > x
{
    x * 2
}

// Async function
async fn fetch(url: String) -> Response {
    // ...
}
```

### Structs

```ml
// Basic struct
struct Point {
    x: Int,
    y: Int,
}

// With generics
struct Container<T> {
    value: T,
    count: Int,
}

// With visibility
struct User {
    pub name: String,
    pub email: String,
    password_hash: String,  // Private
}

// Tuple struct
struct Color(Int, Int, Int);

// Unit struct
struct Marker;
```

### Enums

```ml
// Simple enum
enum Direction {
    North,
    South,
    East,
    West,
}

// With associated data
enum Option<T> {
    Some(T),
    None,
}

enum Result<T, E> {
    Ok(T),
    Err(E),
}

// With named fields
enum Message {
    Text { content: String, sender: User },
    Image { url: String, width: Int, height: Int },
    Quit,
}
```

### Type Aliases

```ml
type UserId = Int;
type Callback = fn(Int) -> Bool;
type StringMap<V> = HashMap<String, V>;
```

### Traits

```ml
trait Display {
    fn display(&self) -> String;
}

trait Iterator {
    type Item;
    fn next(&mut self) -> Option<Self::Item>;
}

trait Default {
    fn default() -> Self;
}
```

### Implementations

```ml
impl Point {
    fn new(x: Int, y: Int) -> Point {
        Point { x, y }
    }

    fn distance(&self, other: &Point) -> Float {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        ((dx * dx + dy * dy) as Float).sqrt()
    }
}

impl Display for Point {
    fn display(&self) -> String {
        "({self.x}, {self.y})"
    }
}
```

### AI Declarations

```ml
// AI model
ai_model ModelName {
    provider: "openai"
    model: "gpt-4"
    temperature: 0.7
    max_tokens: 4096
}

// Prompt template
prompt template_name(param: Type) -> ReturnType {
    """
    Prompt text with {param} interpolation
    """
}
```

### Modules

```ml
// Inline module
mod utils {
    pub fn helper() { }
    fn private_helper() { }
}

// External module (in utils.ml or utils/mod.ml)
mod utils;

// Re-export
pub use utils::helper;
```

## Patterns

```ml
// Literal patterns
match x {
    0 => "zero",
    1 => "one",
    _ => "other",
}

// Variable patterns
match opt {
    Some(value) => use(value),
    None => default(),
}

// Struct patterns
match user {
    User { name, age: 18 } => "adult named {name}",
    User { name, .. } => "user {name}",
}

// Tuple patterns
match point {
    (0, 0) => "origin",
    (x, 0) => "on x-axis",
    (0, y) => "on y-axis",
    (x, y) => "at ({x}, {y})",
}

// Enum patterns
match result {
    Ok(value) => value,
    Err(e) => panic(e),
}

// Guards
match n {
    x if x < 0 => "negative",
    x if x > 0 => "positive",
    _ => "zero",
}

// Or patterns
match c {
    'a' | 'e' | 'i' | 'o' | 'u' => "vowel",
    _ => "consonant",
}

// Range patterns
match n {
    0..=9 => "single digit",
    10..=99 => "double digit",
    _ => "large",
}
```

## Attributes

```ml
// Item attributes
#[test]
fn test_something() { }

#[derive(Debug, Clone)]
struct Data { }

#[deprecated(note: "use new_fn")]
fn old_fn() { }

// Expression attributes
let x = #[allow(overflow)] risky_operation();

// Built-in attributes
#[inline]
#[cold]
#[must_use]
#[cfg(target = "wasm")]
```

## Complete Grammar Reference

See [Grammar (EBNF)](../reference/grammar.md) for the complete formal grammar specification.
