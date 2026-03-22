# Type System

My Language features a powerful, static type system with inference, generics, and AI-aware types.

## Primitive Types

### Numeric Types

| Type | Description | Range |
|------|-------------|-------|
| `Int` | Signed integer (64-bit) | -2^63 to 2^63-1 |
| `Int8` | Signed 8-bit integer | -128 to 127 |
| `Int16` | Signed 16-bit integer | -32768 to 32767 |
| `Int32` | Signed 32-bit integer | -2^31 to 2^31-1 |
| `Int64` | Signed 64-bit integer | -2^63 to 2^63-1 |
| `UInt` | Unsigned integer (64-bit) | 0 to 2^64-1 |
| `UInt8` | Unsigned 8-bit integer | 0 to 255 |
| `UInt16` | Unsigned 16-bit integer | 0 to 65535 |
| `UInt32` | Unsigned 32-bit integer | 0 to 2^32-1 |
| `UInt64` | Unsigned 64-bit integer | 0 to 2^64-1 |
| `Float` | 64-bit floating point | IEEE 754 double |
| `Float32` | 32-bit floating point | IEEE 754 single |

```ml
let integer: Int = 42;
let unsigned: UInt = 100;
let floating: Float = 3.14159;
let byte: UInt8 = 255;
```

### Text Types

| Type | Description |
|------|-------------|
| `String` | UTF-8 encoded string |
| `Char` | Unicode scalar value |

```ml
let greeting: String = "Hello, World!";
let letter: Char = 'A';
let emoji: Char = '🎉';
```

### Boolean

```ml
let active: Bool = true;
let disabled: Bool = false;
```

### Unit

The unit type `()` represents the absence of a meaningful value.

```ml
fn print_message(msg: String) -> () {
    print(msg);
    // Implicitly returns ()
}

// Can be omitted
fn print_message(msg: String) {
    print(msg);
}
```

## Composite Types

### Tuples

Fixed-size, heterogeneous collections.

```ml
// Tuple type
let point: (Int, Int) = (10, 20);
let mixed: (String, Int, Bool) = ("Alice", 30, true);

// Accessing elements
let x = point.0;
let y = point.1;

// Destructuring
let (name, age, active) = mixed;

// Unit tuple
let unit: () = ();
```

### Arrays

Fixed-size, homogeneous collections.

```ml
// Array type
let numbers: [Int; 5] = [1, 2, 3, 4, 5];
let zeros: [Int; 10] = [0; 10];  // 10 zeros

// Accessing elements
let first = numbers[0];
let last = numbers[4];

// Length
let len = numbers.len();  // 5
```

### Slices

Views into contiguous sequences.

```ml
let array = [1, 2, 3, 4, 5];
let slice: &[Int] = &array[1..4];  // [2, 3, 4]

fn sum(numbers: &[Int]) -> Int {
    numbers.iter().sum()
}
```

### Vectors (List)

Dynamic-size, homogeneous collections.

```ml
let mut items: List<String> = List::new();
items.push("apple");
items.push("banana");

let fruits: List<String> = ["apple", "banana", "cherry"];
let first = fruits[0];
let length = fruits.len();
```

### Maps

Key-value collections.

```ml
let mut scores: Map<String, Int> = Map::new();
scores.insert("Alice", 100);
scores.insert("Bob", 85);

let alice_score = scores.get("Alice");  // Some(100)
let unknown = scores.get("Charlie");    // None
```

### Sets

Unique value collections.

```ml
let mut tags: Set<String> = Set::new();
tags.insert("important");
tags.insert("urgent");

let has_urgent = tags.contains("urgent");  // true
```

## User-Defined Types

### Structs

```ml
// Named fields
struct User {
    name: String,
    email: String,
    age: Int,
}

// Usage
let user = User {
    name: "Alice",
    email: "alice@example.com",
    age: 30,
};

// Field access
let name = user.name;

// Update syntax
let older_user = User { age: 31, ..user };

// Tuple structs
struct Point(Int, Int);
let origin = Point(0, 0);

// Unit structs
struct Marker;
let m = Marker;
```

### Enums

```ml
// Simple enum
enum Color {
    Red,
    Green,
    Blue,
}

let color = Color::Red;

// Enum with data
enum Shape {
    Circle { radius: Float },
    Rectangle { width: Float, height: Float },
    Point,
}

let circle = Shape::Circle { radius: 5.0 };

// Generic enums
enum Option<T> {
    Some(T),
    None,
}

enum Result<T, E> {
    Ok(T),
    Err(E),
}
```

### Type Aliases

```ml
type UserId = Int;
type Callback<T> = fn(T) -> Bool;
type StringMap<V> = Map<String, V>;

let id: UserId = 42;
let predicate: Callback<Int> = |x| x > 0;
```

## Generic Types

### Generic Functions

```ml
fn identity<T>(value: T) -> T {
    value
}

fn pair<A, B>(a: A, b: B) -> (A, B) {
    (a, b)
}

fn first<T>(list: List<T>) -> Option<T> {
    if list.is_empty() {
        None
    } else {
        Some(list[0])
    }
}
```

### Generic Structs

```ml
struct Container<T> {
    value: T,
}

struct Pair<A, B> {
    first: A,
    second: B,
}

let int_container: Container<Int> = Container { value: 42 };
let pair: Pair<String, Int> = Pair { first: "age", second: 30 };
```

### Generic Enums

```ml
enum Tree<T> {
    Leaf(T),
    Node {
        value: T,
        left: Box<Tree<T>>,
        right: Box<Tree<T>>,
    },
}
```

## Trait Bounds

```ml
// Single bound
fn print_all<T: Display>(items: List<T>) {
    for item in items {
        print(item.display());
    }
}

// Multiple bounds
fn compare_and_show<T: Ord + Display>(a: T, b: T) {
    if a < b {
        print("{a} is less than {b}");
    }
}

// Where clause (preferred for complex bounds)
fn complex<T, U>(t: T, u: U) -> String
where
    T: Display + Clone,
    U: Debug + Default,
{
    // ...
}
```

## AI Types

### AI Effect Type

The `AI<T>` type represents a computation that requires AI capabilities.

```ml
// AI query returns AI<String>
let response: AI<String> = ai query { prompt: "Hello" };

// Functions with AI return types
fn summarize(text: String) -> AI<String> {
    ai generate { prompt: "Summarize: {text}" }
}

// AI embedding returns vector
fn embed(text: String) -> AI<List<Float>> {
    ai embed(text)
}
```

### AI Constraints

```ml
// Constrain AI behavior with types
fn get_json(prompt: String) -> AI<JSON>
where AI: Temperature<0.0>, AI: ResponseFormat<JSON>
{
    ai generate { prompt: prompt }
}

// Token limits
fn brief_summary(text: String) -> AI<String>
where AI: MaxTokens<100>
{
    ai generate { prompt: "Brief summary: {text}" }
}
```

### Prompt Types

```ml
// Typed prompt returns
prompt get_user_info(name: String) -> UserInfo {
    """
    Extract information about {name} and return as JSON:
    - age: number
    - occupation: string
    - interests: string[]
    """
}

// Usage returns typed result
let info: AI<UserInfo> = get_user_info!("Alice");
```

## Effect Types

### Effect Annotations

```ml
// IO effect
fn read_file(path: String) -> String with IO {
    std::fs::read_to_string(path)
}

// AI effect
fn query_model(prompt: String) -> String with AI {
    ai query { prompt: prompt }
}

// Multiple effects
fn process() -> Result<Data, Error> with IO, AI, Network {
    let config = read_file("config.toml");  // IO
    let enhanced = ai query { prompt: config };  // AI
    send_to_server(enhanced)  // Network
}

// Pure functions (no effects)
fn add(a: Int, b: Int) -> Int {
    a + b  // Pure!
}
```

### Effect Polymorphism

```ml
fn map<T, U, E>(list: List<T>, f: fn(T) -> U with E) -> List<U> with E {
    let mut result = List::new();
    for item in list {
        result.push(f(item));
    }
    result
}
```

## Reference Types

### Immutable References

```ml
fn print_length(s: &String) {
    print("Length: {s.len()}");
}

let text = "Hello";
print_length(&text);  // Borrow
print(text);          // Still valid
```

### Mutable References

```ml
fn increment(n: &mut Int) {
    *n = *n + 1;
}

let mut counter = 0;
increment(&mut counter);
print(counter);  // 1
```

### Ownership Rules

1. Each value has exactly one owner
2. When the owner goes out of scope, the value is dropped
3. Values can be borrowed immutably (multiple) or mutably (exclusive)

```ml
fn take_ownership(s: String) {
    // s is owned here
}  // s is dropped

fn borrow(s: &String) {
    // s is borrowed, not owned
}  // s is NOT dropped

let text = "Hello".to_string();
borrow(&text);        // Borrow
print(text);          // Still valid
take_ownership(text); // Moved
// print(text);       // ERROR: text was moved
```

## Optional and Result Types

### Option<T>

Represents an optional value.

```ml
enum Option<T> {
    Some(T),
    None,
}

fn find(list: List<Int>, target: Int) -> Option<Int> {
    for (i, item) in list.enumerate() {
        if item == target {
            return Some(i);
        }
    }
    None
}

// Usage
match find(numbers, 42) {
    Some(index) => print("Found at {index}"),
    None => print("Not found"),
}

// Methods
let value = opt.unwrap();           // Panics if None
let value = opt.unwrap_or(default); // Default if None
let value = opt.expect("message");  // Panics with message if None
```

### Result<T, E>

Represents success or failure.

```ml
enum Result<T, E> {
    Ok(T),
    Err(E),
}

fn parse_int(s: String) -> Result<Int, ParseError> {
    // ...
}

// Error propagation with ?
fn process(input: String) -> Result<Int, Error> {
    let parsed = parse_int(input)?;  // Returns early on error
    Ok(parsed * 2)
}

// Match on result
match parse_int("42") {
    Ok(n) => print("Parsed: {n}"),
    Err(e) => print("Error: {e}"),
}
```

## Type Inference

My Language uses bidirectional type inference:

```ml
// Types inferred from literals
let x = 42;        // Int
let y = 3.14;      // Float
let z = "hello";   // String
let b = true;      // Bool

// Types inferred from usage
let list = List::new();  // List<?>
list.push(1);            // Now List<Int>

// Types inferred from return
fn double(x: Int) -> _ {  // Return type inferred
    x * 2                  // Int * Int = Int
}

// Turbofish for ambiguous cases
let numbers = List::<Int>::new();
let parsed = "42".parse::<Int>()?;
```

## Subtyping

### Variance

```ml
// Covariance: List<Cat> is subtype of List<Animal>
// if Cat is subtype of Animal (for reading)

// Contravariance: fn(Animal) is subtype of fn(Cat)
// (for function parameters)

// Invariance: &mut T is invariant in T
```

### Never Type

The `!` (never) type represents computations that never complete.

```ml
fn diverge() -> ! {
    loop { }
}

fn panic(msg: String) -> ! {
    print("PANIC: {msg}");
    std::process::exit(1)
}

// Never can coerce to any type
let x: Int = if condition {
    42
} else {
    panic("unreachable")  // ! coerces to Int
};
```

## Type Coercion

Automatic coercions in specific contexts:

```ml
// Deref coercion
let s: String = "hello".to_string();
let len = s.len();  // String derefs to str

// Reference coercion
let x: &Int = &42;
let y: &Int = x;  // &T to &T

// Never coercion
let x: Int = panic("!");  // ! to any type
```

## Advanced Type Features

### Associated Types

```ml
trait Iterator {
    type Item;

    fn next(&mut self) -> Option<Self::Item>;
}

impl Iterator for Counter {
    type Item = Int;

    fn next(&mut self) -> Option<Int> {
        // ...
    }
}
```

### Higher-Kinded Types (Future)

```ml
trait Functor<F<_>> {
    fn map<A, B>(fa: F<A>, f: fn(A) -> B) -> F<B>;
}

impl Functor<Option> {
    fn map<A, B>(fa: Option<A>, f: fn(A) -> B) -> Option<B> {
        match fa {
            Some(a) => Some(f(a)),
            None => None,
        }
    }
}
```

### Dependent Types (Future)

```ml
// Length-indexed vectors
type Vec<T, const N: USize>;

fn concat<T, const M: USize, const N: USize>(
    a: Vec<T, M>,
    b: Vec<T, N>
) -> Vec<T, M + N>;

// Refinement types
type Positive = Int where self > 0;
type NonEmpty<T> = List<T> where len > 0;
```
