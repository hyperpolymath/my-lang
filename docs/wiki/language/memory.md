# Memory Management

My Language uses ownership and borrowing for memory safety without garbage collection, similar to Rust but with ergonomic improvements.

## Ownership

### The Rules

1. Each value has exactly one owner
2. When the owner goes out of scope, the value is dropped
3. Ownership can be transferred (moved)

### Move Semantics

```ml
fn main() {
    let s1 = "hello".to_string();  // s1 owns the string
    let s2 = s1;                    // Ownership moves to s2
    // print(s1);                   // ERROR: s1 no longer valid

    print(s2);                      // OK: s2 owns the string
}  // s2 goes out of scope, string is dropped
```

### Copy Types

Some types are copied instead of moved:

```ml
// Copy types: Int, Float, Bool, Char, tuples of Copy types
let x = 5;
let y = x;  // Copy, not move
print(x);   // OK: x is still valid
print(y);   // OK: y has its own copy

// Non-copy types: String, List, custom structs
let s1 = "hello".to_string();
let s2 = s1;  // Move
// s1 is no longer valid
```

### Clone

Explicit deep copying:

```ml
let s1 = "hello".to_string();
let s2 = s1.clone();  // Deep copy
print(s1);  // OK: s1 still valid
print(s2);  // OK: s2 is independent copy
```

## Borrowing

### Immutable References

```ml
fn calculate_length(s: &String) -> Int {
    s.len()  // Can read, not modify
}

fn main() {
    let s = "hello".to_string();
    let len = calculate_length(&s);  // Borrow s
    print("Length of '{s}' is {len}");  // s still valid
}
```

### Multiple Immutable Borrows

```ml
fn main() {
    let s = "hello".to_string();

    let r1 = &s;  // OK
    let r2 = &s;  // OK: multiple immutable borrows allowed
    let r3 = &s;  // OK

    print("{r1}, {r2}, {r3}");
}
```

### Mutable References

```ml
fn append_world(s: &mut String) {
    s.push_str(", world!");
}

fn main() {
    let mut s = "hello".to_string();
    append_world(&mut s);
    print(s);  // "hello, world!"
}
```

### Exclusive Mutable Access

```ml
fn main() {
    let mut s = "hello".to_string();

    let r1 = &mut s;
    // let r2 = &mut s;     // ERROR: only one mutable borrow
    // let r3 = &s;         // ERROR: can't borrow immutably while mutably borrowed

    r1.push_str("!");
    // After r1's last use, can borrow again
    let r4 = &s;  // OK
}
```

### Borrow Rules

1. You can have either:
   - One mutable reference, OR
   - Any number of immutable references
2. References must always be valid (no dangling references)

## Lifetimes

### Implicit Lifetimes

Most lifetimes are inferred:

```ml
fn first_word(s: &String) -> &str {
    // Lifetime of return value tied to input
    &s[0..s.find(' ').unwrap_or(s.len())]
}
```

### Explicit Lifetimes

When needed, lifetimes can be explicit:

```ml
fn longest<'a>(x: &'a String, y: &'a String) -> &'a String {
    if x.len() > y.len() { x } else { y }
}

fn main() {
    let s1 = "hello".to_string();
    let s2 = "world!".to_string();
    let result = longest(&s1, &s2);
    print("Longest: {result}");
}
```

### Lifetime Elision

Common patterns don't need explicit lifetimes:

```ml
// Input lifetime → output lifetime
fn get_first(list: &List<Int>) -> &Int {
    &list[0]
}

// &self lifetime → output lifetime
impl List<T> {
    fn first(&self) -> &T {
        &self[0]
    }
}
```

### Struct Lifetimes

```ml
struct Excerpt<'a> {
    text: &'a str,
}

fn main() {
    let novel = "Call me Ishmael. Some years ago...".to_string();
    let first_sentence = novel.split('.').next().unwrap();

    let excerpt = Excerpt {
        text: first_sentence,
    };

    print("{}", excerpt.text);
}
```

### Static Lifetime

```ml
// Lives for the entire program
let s: &'static str = "I live forever";

// Constants are always 'static
const MESSAGE: &str = "Hello, World!";
```

## Smart Pointers

### Box (Heap Allocation)

```ml
// Allocate on heap
let b = Box::new(5);
print("b = {b}");

// Recursive types
enum List<T> {
    Cons(T, Box<List<T>>),
    Nil,
}

let list = Cons(1, Box::new(Cons(2, Box::new(Nil))));
```

### Rc (Reference Counting)

```ml
use std::rc::Rc;

// Shared ownership
let a = Rc::new([1, 2, 3]);
let b = Rc::clone(&a);  // Increment ref count
let c = Rc::clone(&a);

print("Count: {}", Rc::strong_count(&a));  // 3
```

### Arc (Atomic Reference Counting)

```ml
use std::sync::Arc;

// Thread-safe shared ownership
let data = Arc::new([1, 2, 3]);

let data_clone = Arc::clone(&data);
spawn {
    print("From thread: {data_clone}");
};
```

### RefCell (Interior Mutability)

```ml
use std::cell::RefCell;

let cell = RefCell::new(5);

// Borrow mutably at runtime
*cell.borrow_mut() += 1;

print("Value: {}", cell.borrow());  // 6
```

### Combining Rc and RefCell

```ml
use std::rc::Rc;
use std::cell::RefCell;

// Shared mutable state
let shared = Rc::new(RefCell::new(Vec::new()));

let a = Rc::clone(&shared);
let b = Rc::clone(&shared);

a.borrow_mut().push(1);
b.borrow_mut().push(2);

print("{:?}", shared.borrow());  // [1, 2]
```

## Drop Trait

### Custom Cleanup

```ml
struct FileHandle {
    fd: Int,
}

impl Drop for FileHandle {
    fn drop(&mut self) {
        close_file(self.fd);
        print("File closed");
    }
}

fn main() {
    let file = FileHandle { fd: open_file("test.txt") };
    // Use file...
}  // file.drop() called automatically
```

### Early Drop

```ml
fn main() {
    let file = FileHandle { fd: 0 };

    // ... use file ...

    drop(file);  // Explicitly drop early

    // file no longer accessible
}
```

## Slice Types

### String Slices

```ml
let s = "Hello, World!".to_string();

let hello: &str = &s[0..5];    // "Hello"
let world: &str = &s[7..12];   // "World"

// Ranges
let full = &s[..];        // Full string
let start = &s[..5];      // "Hello"
let end = &s[7..];        // "World!"
```

### Array Slices

```ml
let arr = [1, 2, 3, 4, 5];

let slice: &[Int] = &arr[1..4];  // [2, 3, 4]

fn sum(numbers: &[Int]) -> Int {
    numbers.iter().sum()
}

let total = sum(&arr);      // Sum entire array
let partial = sum(slice);   // Sum slice
```

## Memory Layout

### Stack vs Heap

```ml
// Stack allocated (known size)
let x: Int = 5;
let arr: [Int; 3] = [1, 2, 3];
let point: Point = Point { x: 0, y: 0 };

// Heap allocated (dynamic size)
let s: String = "hello".to_string();
let v: Vec<Int> = vec![1, 2, 3];
let b: Box<Int> = Box::new(5);
```

### Size and Alignment

```ml
// Get size
let size = std::mem::size_of::<Point>();

// Get alignment
let align = std::mem::align_of::<Point>();

// Zero-sized types
struct Empty;
assert_eq(std::mem::size_of::<Empty>(), 0);
```

## Unsafe Code

### Unsafe Blocks

```ml
unsafe fn dangerous() {
    // Can do unsafe operations
}

fn main() {
    // Must explicitly opt in
    unsafe {
        dangerous();
    }
}
```

### Raw Pointers

```ml
let x = 5;
let raw_ptr: *const Int = &x;

unsafe {
    print("Value: {}", *raw_ptr);
}

let mut y = 10;
let raw_mut: *mut Int = &mut y;

unsafe {
    *raw_mut = 20;
}
```

### When Unsafe is Needed

1. Dereferencing raw pointers
2. Calling unsafe functions
3. Accessing mutable statics
4. Implementing unsafe traits
5. FFI (Foreign Function Interface)

## Common Patterns

### Builder Pattern

```ml
struct RequestBuilder {
    url: String,
    method: Method,
    headers: Map<String, String>,
}

impl RequestBuilder {
    fn new(url: String) -> Self {
        RequestBuilder {
            url,
            method: Method::Get,
            headers: Map::new(),
        }
    }

    fn method(mut self, method: Method) -> Self {
        self.method = method;
        self
    }

    fn header(mut self, key: String, value: String) -> Self {
        self.headers.insert(key, value);
        self
    }

    fn build(self) -> Request {
        Request { ... }
    }
}

// Usage
let request = RequestBuilder::new("https://api.example.com")
    .method(Method::Post)
    .header("Content-Type", "application/json")
    .build();
```

### RAII (Resource Acquisition Is Initialization)

```ml
struct MutexGuard<'a, T> {
    mutex: &'a Mutex<T>,
}

impl<T> Drop for MutexGuard<'_, T> {
    fn drop(&mut self) {
        self.mutex.unlock();
    }
}

fn with_lock<T, R>(mutex: &Mutex<T>, f: fn(&mut T) -> R) -> R {
    let guard = mutex.lock();  // Lock acquired
    f(&mut guard.data)
}  // guard dropped, lock released
```

### Newtype Pattern

```ml
// Wrapper with zero runtime cost
struct UserId(Int);
struct Email(String);

fn send_email(to: Email, from: UserId) {
    // Type safety: can't mix up Email and String
}

// Usage
let email = Email("user@example.com".to_string());
let user_id = UserId(42);
send_email(email, user_id);
```

## Best Practices

### 1. Prefer Borrowing Over Owning

```ml
// Good: Borrow when you just need to read
fn print_length(s: &String) {
    print("Length: {}", s.len());
}

// Avoid: Taking ownership unnecessarily
fn print_length(s: String) {
    print("Length: {}", s.len());
}
```

### 2. Use Clone Sparingly

```ml
// Good: Restructure to avoid clone
fn process(data: &Data) -> Result {
    // Work with reference
}

// Avoid: Unnecessary cloning
fn process(data: Data) -> Result {
    let copy = data.clone();  // Expensive!
}
```

### 3. Return Owned Values from Functions

```ml
// Good: Return owned value
fn create_greeting(name: &str) -> String {
    format!("Hello, {name}!")
}

// Awkward: Returning reference requires lifetime
fn create_greeting<'a>(name: &'a str) -> &'a str {
    // Limited to returning parts of input
}
```

### 4. Use Interior Mutability Carefully

```ml
// Good: Clear need for shared mutation
struct Counter {
    count: RefCell<Int>,
}

// Avoid: Overuse of RefCell
struct Data {
    field1: RefCell<T>,  // Why not just mut?
    field2: RefCell<U>,
}
```
