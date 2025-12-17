# Effects and Capabilities

My Language uses an effect system to track computational effects in the type system, making side effects explicit and enabling powerful abstractions.

## What Are Effects?

Effects represent operations that go beyond pure computation:
- **IO**: Reading/writing files, console I/O
- **AI**: Making AI API calls
- **Network**: HTTP requests, database connections
- **State**: Mutable state modifications
- **Exception**: Operations that may fail
- **Async**: Asynchronous operations

## Basic Effect Annotations

### Declaring Effects

```ml
// Function with IO effect
fn read_config(path: String) -> Config with IO {
    let content = read_file(path);
    parse_config(content)
}

// Function with AI effect
fn analyze(text: String) -> String with AI {
    ai query { prompt: "Analyze: {text}" }
}

// Function with multiple effects
fn process_and_log(data: Data) -> Result<Output, Error> with IO, AI {
    log("Processing data");  // IO
    let enhanced = ai query { prompt: "Enhance: {data}" };  // AI
    log("Done");  // IO
    Ok(enhanced)
}

// Pure function (no effects)
fn add(a: Int, b: Int) -> Int {
    a + b  // No effects!
}
```

### Effect Inference

The compiler infers effects when not explicitly annotated:

```ml
// Effect inferred as: with IO
fn greet(name: String) {
    print("Hello, {name}!");  // print has IO effect
}

// Effect inferred as: with AI
fn summarize(text: String) -> String {
    ai generate { prompt: "Summarize: {text}" }
}

// Effect inferred as: with IO, AI
fn log_and_query(prompt: String) -> String {
    log("Querying...");
    ai query { prompt: prompt }
}
```

## Effect Polymorphism

Functions can be generic over effects:

```ml
// Effect-polymorphic map
fn map<T, U, E>(list: List<T>, f: fn(T) -> U with E) -> List<U> with E {
    let mut result = List::new();
    for item in list {
        result.push(f(item));
    }
    result
}

// Can be used with any effect
let doubled = map([1, 2, 3], |x| x * 2);  // Pure
let printed = map([1, 2, 3], |x| { print(x); x });  // with IO
let enhanced = map(texts, |t| ai query { prompt: t });  // with AI
```

### Effect Bounds

```ml
// Require specific effects
fn with_io<T, E: IO>(f: fn() -> T with E) -> T with E {
    log("Starting");
    let result = f();
    log("Done");
    result
}

// Exclude effects
fn pure_only<T>(f: fn() -> T with Pure) -> T {
    f()  // Guaranteed no effects
}
```

## Built-in Effects

### IO Effect

```ml
effect IO {
    fn print(msg: String);
    fn read_line() -> String;
    fn read_file(path: String) -> String;
    fn write_file(path: String, content: String);
}

fn example() with IO {
    let name = read_line();
    print("Hello, {name}!");
}
```

### AI Effect

```ml
effect AI {
    fn query(prompt: String) -> String;
    fn embed(text: String) -> List<Float>;
    fn generate(prompt: String, options: Options) -> String;
}

fn example() with AI {
    let answer = ai query { prompt: "What is 2+2?" };
}
```

### State Effect

```ml
effect State<S> {
    fn get() -> S;
    fn put(value: S);
    fn modify(f: fn(S) -> S);
}

fn counter() with State<Int> {
    let current = get();
    put(current + 1);
}
```

### Exception Effect

```ml
effect Exception<E> {
    fn throw(error: E) -> !;
}

fn divide(a: Int, b: Int) -> Int with Exception<String> {
    if b == 0 {
        throw("Division by zero")
    } else {
        a / b
    }
}
```

### Async Effect

```ml
effect Async {
    fn await<T>(future: Future<T>) -> T;
    fn spawn<T>(task: fn() -> T) -> Handle<T>;
}

fn fetch_data(url: String) -> Data with Async {
    let response = await(http::get(url));
    response.json()
}
```

## Effect Handlers

Effect handlers provide implementations for effects:

### Basic Handlers

```ml
// Handle State effect
fn run_state<S, T>(initial: S, f: fn() -> T with State<S>) -> (T, S) {
    let mut state = initial;

    handle f() {
        get() => resume(state),
        put(value) => {
            state = value;
            resume(())
        }
    }
}

// Usage
let (result, final_state) = run_state(0, || {
    let x = get();
    put(x + 1);
    let y = get();
    put(y * 2);
    get()
});
// result = 2, final_state = 2
```

### Exception Handlers

```ml
fn catch<T, E>(f: fn() -> T with Exception<E>) -> Result<T, E> {
    handle f() {
        throw(e) => Err(e),
        return(value) => Ok(value),
    }
}

// Usage
let result = catch(|| {
    let x = divide(10, 0);  // throws
    x * 2
});
// result = Err("Division by zero")
```

### Async Handlers

```ml
fn run_async<T>(f: fn() -> T with Async) -> T {
    let runtime = Runtime::new();
    handle f() {
        await(future) => {
            let value = runtime.block_on(future);
            resume(value)
        }
        spawn(task) => {
            let handle = runtime.spawn(task);
            resume(handle)
        }
    }
}
```

## Composing Effects

### Effect Rows

Multiple effects compose automatically:

```ml
fn complex_operation() with IO, AI, State<Config> {
    let config = get();  // State
    log("Using config: {config}");  // IO
    let result = ai query { prompt: config.prompt };  // AI
    put(config.with_result(result));  // State
}
```

### Effect Subtyping

More specific effects can be used where less specific ones are expected:

```ml
// IO is a subset of IO + AI
fn use_io(f: fn() with IO) {
    f();
}

fn has_io_and_ai() with IO, AI {
    print("hello");
}

// This works because IO ⊆ (IO, AI)
// The AI effect is simply not used
```

## Capability-Based Security

Effects enable capability-based security patterns:

```ml
// Capability tokens
struct FileReadCap { path: String }
struct FileWriteCap { path: String }
struct NetworkCap { allowed_hosts: List<String> }

// Functions require capabilities
fn read_file(cap: FileReadCap) -> String with IO {
    std::fs::read_to_string(cap.path)
}

fn http_get(cap: NetworkCap, url: String) -> Response with Network {
    if !cap.allowed_hosts.contains(url.host()) {
        panic("Not allowed: {url.host()}");
    }
    http::get(url)
}

// Main grants capabilities
fn main() with IO, Network {
    let read_cap = FileReadCap { path: "./config.toml" };
    let config = read_file(read_cap);

    let net_cap = NetworkCap {
        allowed_hosts: ["api.example.com"]
    };
    let data = http_get(net_cap, "https://api.example.com/data");
}
```

## Effect Inference Details

### Local Inference

```ml
fn example() {
    // Each statement's effects are inferred
    let x = 5;                           // Pure
    let y = read_line();                 // IO
    let z = ai query { prompt: "hi" };   // AI

    // Function's effect is union: IO, AI
}
```

### Propagation

```ml
fn inner() with AI {
    ai query { prompt: "test" }
}

fn outer() {
    inner()  // outer inherits AI effect
}

// outer's inferred type: fn() with AI
```

### Effect Masking

```ml
// Handle effect to remove it
fn without_exceptions<T>(f: fn() -> T with Exception<E>) -> Option<T> {
    handle f() {
        throw(_) => None,
        return(v) => Some(v),
    }
}

// f has Exception effect, result does not
let safe = without_exceptions(risky_operation);
// safe: Option<T> (no Exception effect)
```

## Practical Examples

### Logging Middleware

```ml
fn with_logging<T, E>(name: String, f: fn() -> T with E) -> T with E, IO {
    log("[{name}] Starting");
    let start = Time::now();

    let result = f();

    let elapsed = Time::now() - start;
    log("[{name}] Completed in {elapsed}ms");

    result
}

// Usage
let data = with_logging("fetch_users", || {
    http::get("/users").json()
});
```

### Transaction Effect

```ml
effect Transaction {
    fn begin();
    fn commit();
    fn rollback();
    fn query(sql: String) -> Rows;
}

fn transfer(from: Int, to: Int, amount: Int) with Transaction {
    begin();

    let from_balance = query("SELECT balance FROM accounts WHERE id = {from}");

    if from_balance < amount {
        rollback();
        panic("Insufficient funds");
    }

    query("UPDATE accounts SET balance = balance - {amount} WHERE id = {from}");
    query("UPDATE accounts SET balance = balance + {amount} WHERE id = {to}");

    commit();
}

fn run_transaction<T>(f: fn() -> T with Transaction) -> Result<T, DbError> {
    let conn = db::connect();
    handle f() {
        begin() => { conn.begin(); resume(()) }
        commit() => { conn.commit(); resume(()) }
        rollback() => { conn.rollback(); resume(()) }
        query(sql) => { resume(conn.query(sql)) }
    }
}
```

### Testing with Effect Mocking

```ml
// Production: real AI
fn real_ai_handler<T>(f: fn() -> T with AI) -> T {
    handle f() {
        query(prompt) => {
            let response = openai::complete(prompt);
            resume(response)
        }
    }
}

// Testing: mock AI
fn mock_ai_handler<T>(responses: List<String>, f: fn() -> T with AI) -> T {
    let mut idx = 0;
    handle f() {
        query(_) => {
            let response = responses[idx];
            idx += 1;
            resume(response)
        }
    }
}

#[test]
fn test_ai_function() {
    let result = mock_ai_handler(
        ["mocked response"],
        || my_ai_function()
    );
    assert_eq(result, expected);
}
```

## Best Practices

### 1. Annotate Public APIs

```ml
// Good: Explicit effects in public API
pub fn fetch_user(id: Int) -> User with IO, Exception<ApiError> {
    // ...
}

// Avoid: Relying on inference for public APIs
pub fn fetch_user(id: Int) -> User {  // Effects hidden
    // ...
}
```

### 2. Minimize Effect Scope

```ml
// Good: Isolate effects
fn process(data: Data) -> Result {
    let validated = validate(data);  // Pure
    let transformed = transform(validated);  // Pure
    save(transformed)  // IO only here
}

// Avoid: Effects throughout
fn process(data: Data) with IO {
    log("Validating");  // Unnecessary IO
    let validated = validate(data);
    log("Transforming");  // Unnecessary IO
    let transformed = transform(validated);
    save(transformed)
}
```

### 3. Use Effect Handlers for Testing

```ml
// Production code uses effects
fn business_logic() with AI, IO {
    let data = read_input();
    let result = ai query { prompt: data };
    write_output(result);
}

// Test provides mock handlers
#[test]
fn test_business_logic() {
    mock_io(input: "test", || {
        mock_ai(responses: ["response"], || {
            business_logic();
        })
    });
}
```

## Related Documentation

- [Type System](types.md)
- [AI Features](ai-features.md)
- [Error Handling](../guides/error-handling.md)
