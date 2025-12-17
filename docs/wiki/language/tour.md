# My Language Tour

A quick introduction to My Language's features and syntax.

## Hello World

```ml
fn main() {
    print("Hello, World!");
}
```

## Variables

```ml
// Immutable by default
let name = "Alice";
let age = 30;
let pi = 3.14159;

// Mutable variables
let mut counter = 0;
counter = counter + 1;

// Type annotations (optional with inference)
let score: Int = 100;
let ratio: Float = 0.5;
let active: Bool = true;
```

## Functions

```ml
// Basic function
fn greet(name: String) -> String {
    "Hello, {name}!"
}

// Multiple parameters
fn add(a: Int, b: Int) -> Int {
    a + b
}

// Default parameters
fn repeat(text: String, times: Int = 1) -> String {
    // ...
}

// Generic function
fn first<T>(list: List<T>) -> Option<T> {
    list.get(0)
}
```

## Control Flow

```ml
// If-else
fn classify(n: Int) -> String {
    if n > 0 {
        "positive"
    } else if n < 0 {
        "negative"
    } else {
        "zero"
    }
}

// Match expression
fn describe(value: Option<Int>) -> String {
    match value {
        Some(n) if n > 100 => "large number",
        Some(n) => "number: {n}",
        None => "no value",
    }
}

// Loops
fn sum_to(n: Int) -> Int {
    let mut total = 0;
    for i in 1..=n {
        total = total + i;
    }
    total
}
```

## Data Types

```ml
// Structs
struct User {
    name: String,
    email: String,
    age: Int,
}

let user = User {
    name: "Alice",
    email: "alice@example.com",
    age: 30,
};

// Enums
enum Status {
    Active,
    Inactive,
    Pending { since: Time },
}

let status = Status::Pending { since: Time::now() };

// Tuples
let point = (10, 20);
let (x, y) = point;
```

## Pattern Matching

```ml
fn process(result: Result<User, Error>) {
    match result {
        Ok(User { name, age, .. }) if age >= 18 => {
            print("Adult user: {name}");
        }
        Ok(User { name, .. }) => {
            print("Minor user: {name}");
        }
        Err(Error::NotFound) => {
            print("User not found");
        }
        Err(e) => {
            print("Error: {e}");
        }
    }
}
```

## AI Expressions

This is what makes My Language special - first-class AI integration.

```ml
// Quick AI query
let answer = ai! { "What is the capital of France?" };

// AI with options
let summary = ai query {
    prompt: "Summarize this article"
    context: article_text
    model: "gpt-4"
};

// AI verification
let is_valid = ai verify {
    input: user_data
    constraint: "must be valid JSON"
};

// AI generation
let story = ai generate {
    prompt: "Write a short story about {topic}"
    temperature: 0.8
};

// AI embedding
let embedding = ai embed(document);

// AI classification
let sentiment = ai classify {
    input: review_text
    categories: ["positive", "negative", "neutral"]
};
```

## AI Models and Prompts

```ml
// Declare an AI model
ai_model Summarizer {
    provider: "openai"
    model: "gpt-4"
    temperature: 0.3
    max_tokens: 500
}

// Define a reusable prompt
prompt summarize(text: String, max_words: Int = 100) -> String {
    """
    Summarize the following text in at most {max_words} words:

    {text}

    Provide a clear, concise summary.
    """
}

// Use the prompt
fn create_summary(article: String) -> AI<String> {
    summarize!(article, max_words: 50)
}
```

## Async/Await

```ml
async fn fetch_user(id: Int) -> Result<User, Error> {
    let response = http::get("https://api.example.com/users/{id}").await?;
    let user = response.json::<User>().await?;
    Ok(user)
}

async fn fetch_all_users(ids: List<Int>) -> List<User> {
    // Concurrent fetching
    let futures = ids.map(|id| fetch_user(id));
    join_all(futures).await
}
```

## Error Handling

```ml
// Result type
fn divide(a: Int, b: Int) -> Result<Int, String> {
    if b == 0 {
        Err("Division by zero")
    } else {
        Ok(a / b)
    }
}

// Error propagation with ?
fn calculate(x: Int, y: Int) -> Result<Int, String> {
    let quotient = divide(x, y)?;
    let result = quotient * 2;
    Ok(result)
}

// Match on errors
match divide(10, 0) {
    Ok(value) => print("Result: {value}"),
    Err(msg) => print("Error: {msg}"),
}
```

## Modules and Imports

```ml
// Import from standard library
use std::collections::{Vec, HashMap};
use std::io::{read_file, write_file};

// Import from local module
use crate::utils::helpers;
use super::parent_module;

// Qualified import
use http::Client as HttpClient;

// Module definition
mod utils {
    pub fn helper_function() {
        // ...
    }
}
```

## Effects

```ml
// Functions can declare their effects
fn read_config() -> Config with IO {
    read_file("config.toml")
}

fn query_ai(prompt: String) -> String with AI {
    ai query { prompt: prompt }
}

// Pure functions have no effects
fn add(a: Int, b: Int) -> Int {
    a + b  // No effects!
}

// Effects are tracked in the type system
fn process() with IO, AI {
    let config = read_config();      // IO effect
    let result = query_ai("test");   // AI effect
}
```

## Contracts

```ml
fn sqrt(n: Float) -> Float
requires n >= 0.0
ensures result >= 0.0
ensures result * result == n
{
    // Implementation
}

struct PositiveInt {
    value: Int,
}
invariant self.value > 0
```

## Attributes

```ml
#[test]
fn test_addition() {
    assert_eq(2 + 2, 4);
}

#[derive(Debug, Clone, PartialEq)]
struct Point {
    x: Int,
    y: Int,
}

#[deprecated(since: "0.2.0", note: "use new_function instead")]
fn old_function() {
    // ...
}

#[ai_cached(ttl: 3600)]
fn expensive_ai_call(input: String) -> AI<String> {
    ai generate { prompt: input }
}
```

## Next Steps

- [Syntax Details](syntax.md) - Complete syntax reference
- [Type System](types.md) - Deep dive into types
- [AI Features](ai-features.md) - Full AI integration guide
- [Standard Library](../reference/stdlib.md) - Available modules and functions
