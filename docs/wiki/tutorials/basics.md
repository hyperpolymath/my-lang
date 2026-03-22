# Language Basics Tutorial

Learn the fundamentals of My Language through hands-on examples.

## Lesson 1: Hello World

Create a new project:

```bash
ml new hello-tutorial
cd hello-tutorial
```

Edit `src/main.ml`:

```ml
fn main() {
    print("Hello, World!");
}
```

Run it:

```bash
ml run
# Output: Hello, World!
```

## Lesson 2: Variables and Types

```ml
fn main() {
    // Immutable variables (default)
    let name = "Alice";
    let age = 30;
    let height = 5.8;
    let is_student = false;

    // Type annotations (optional)
    let score: Int = 100;

    // Mutable variables
    let mut counter = 0;
    counter = counter + 1;
    counter += 1;  // Shorthand
    print("Counter: {counter}");

    // Constants (compile-time)
    const MAX_SIZE: Int = 1000;
}
```

### Type Inference

The compiler infers types automatically:

```ml
let x = 42;          // Int
let y = 3.14;        // Float
let s = "hello";     // String
let b = true;        // Bool
let list = [1, 2, 3]; // List<Int>
```

## Lesson 3: Functions

```ml
// Basic function
fn add(a: Int, b: Int) -> Int {
    a + b  // Implicit return (last expression)
}

// Function with explicit return
fn absolute(x: Int) -> Int {
    if x < 0 {
        return -x;
    }
    x
}

// Function with no return value
fn greet(name: String) {
    print("Hello, {name}!");
}

// Default parameters
fn power(base: Int, exp: Int = 2) -> Int {
    let mut result = 1;
    for _ in 0..exp {
        result *= base;
    }
    result
}

fn main() {
    let sum = add(5, 3);
    print("5 + 3 = {sum}");

    greet("World");

    print("2^3 = {power(2, exp: 3)}");
    print("5^2 = {power(5)}");  // Uses default exp=2
}
```

## Lesson 4: Control Flow

### If Expressions

```ml
fn classify_number(n: Int) -> String {
    if n > 0 {
        "positive"
    } else if n < 0 {
        "negative"
    } else {
        "zero"
    }
}

fn main() {
    let result = classify_number(-5);
    print(result);  // negative

    // If as expression
    let max = if 5 > 3 { 5 } else { 3 };
    print("max: {max}");
}
```

### Match Expressions

```ml
fn describe(n: Int) -> String {
    match n {
        0 => "zero",
        1 => "one",
        2 | 3 => "two or three",
        4..=10 => "small",
        n if n < 0 => "negative",
        _ => "large",
    }
}

fn main() {
    print(describe(0));   // zero
    print(describe(3));   // two or three
    print(describe(7));   // small
    print(describe(-5));  // negative
    print(describe(100)); // large
}
```

### Loops

```ml
fn main() {
    // For loop
    for i in 0..5 {
        print("i = {i}");
    }

    // For with collection
    let fruits = ["apple", "banana", "cherry"];
    for fruit in fruits {
        print("Fruit: {fruit}");
    }

    // While loop
    let mut count = 0;
    while count < 3 {
        print("count = {count}");
        count += 1;
    }

    // Loop with break
    let mut sum = 0;
    loop {
        sum += 1;
        if sum >= 10 {
            break;
        }
    }
    print("sum = {sum}");

    // Loop with value
    let result = loop {
        sum += 1;
        if sum >= 20 {
            break sum;  // Return value from loop
        }
    };
    print("result = {result}");
}
```

## Lesson 5: Collections

### Lists (Vectors)

```ml
fn main() {
    // Create a list
    let mut numbers = [1, 2, 3, 4, 5];

    // Access elements
    let first = numbers[0];
    print("First: {first}");

    // Modify elements
    numbers[0] = 10;

    // Add elements
    numbers.push(6);

    // List operations
    print("Length: {numbers.len()}");
    print("Contains 3: {numbers.contains(3)}");

    // Iterate
    for n in numbers {
        print("Number: {n}");
    }

    // Functional operations
    let doubled = numbers.map(|x| x * 2);
    let evens = numbers.filter(|x| x % 2 == 0);
    let sum = numbers.fold(0, |acc, x| acc + x);
}
```

### Maps

```ml
fn main() {
    let mut scores = Map::new();

    // Insert
    scores.insert("Alice", 100);
    scores.insert("Bob", 85);
    scores.insert("Charlie", 92);

    // Access
    match scores.get("Alice") {
        Some(score) => print("Alice's score: {score}"),
        None => print("Alice not found"),
    }

    // Iterate
    for (name, score) in scores {
        print("{name}: {score}");
    }
}
```

## Lesson 6: Structs and Enums

### Structs

```ml
struct Point {
    x: Int,
    y: Int,
}

impl Point {
    // Constructor
    fn new(x: Int, y: Int) -> Point {
        Point { x, y }
    }

    // Method
    fn distance_from_origin(&self) -> Float {
        let sum = (self.x * self.x + self.y * self.y) as Float;
        sum.sqrt()
    }

    // Mutable method
    fn translate(&mut self, dx: Int, dy: Int) {
        self.x += dx;
        self.y += dy;
    }
}

fn main() {
    let mut p = Point::new(3, 4);
    print("Distance: {p.distance_from_origin()}");  // 5.0

    p.translate(1, 1);
    print("New position: ({p.x}, {p.y})");  // (4, 5)
}
```

### Enums

```ml
enum Shape {
    Circle { radius: Float },
    Rectangle { width: Float, height: Float },
    Square { side: Float },
}

impl Shape {
    fn area(&self) -> Float {
        match self {
            Shape::Circle { radius } => 3.14159 * radius * radius,
            Shape::Rectangle { width, height } => width * height,
            Shape::Square { side } => side * side,
        }
    }
}

fn main() {
    let shapes = [
        Shape::Circle { radius: 5.0 },
        Shape::Rectangle { width: 4.0, height: 3.0 },
        Shape::Square { side: 2.0 },
    ];

    for shape in shapes {
        print("Area: {shape.area()}");
    }
}
```

## Lesson 7: Error Handling

### Option Type

```ml
fn find_index(list: List<Int>, target: Int) -> Option<Int> {
    for (i, item) in list.enumerate() {
        if item == target {
            return Some(i);
        }
    }
    None
}

fn main() {
    let numbers = [1, 2, 3, 4, 5];

    match find_index(numbers, 3) {
        Some(i) => print("Found at index {i}"),
        None => print("Not found"),
    }

    // Using unwrap_or
    let index = find_index(numbers, 10).unwrap_or(-1);
    print("Index: {index}");
}
```

### Result Type

```ml
fn divide(a: Int, b: Int) -> Result<Int, String> {
    if b == 0 {
        Err("Division by zero")
    } else {
        Ok(a / b)
    }
}

fn main() {
    match divide(10, 2) {
        Ok(result) => print("10 / 2 = {result}"),
        Err(msg) => print("Error: {msg}"),
    }

    // Using ? for propagation
    fn calculate() -> Result<Int, String> {
        let x = divide(100, 5)?;  // Returns early on error
        let y = divide(x, 2)?;
        Ok(y)
    }
}
```

## Lesson 8: AI Basics

```ml
fn main() {
    // Quick AI query
    let answer = ai! { "What is the capital of France?" };
    print("Answer: {answer}");

    // AI with options
    let summary = ai query {
        prompt: "Summarize photosynthesis in one sentence"
        max_tokens: 50
    };
    print("Summary: {summary}");

    // AI verification
    let is_valid = ai verify {
        input: "user@example.com"
        constraint: "valid email address"
    };
    print("Valid email: {is_valid}");

    // AI classification
    let sentiment = ai classify {
        input: "I love this product!"
        categories: ["positive", "negative", "neutral"]
    };
    print("Sentiment: {sentiment}");
}
```

## Exercises

### Exercise 1: FizzBuzz

Write a function that prints numbers 1-100, but:
- For multiples of 3, print "Fizz"
- For multiples of 5, print "Buzz"
- For multiples of both, print "FizzBuzz"

<details>
<summary>Solution</summary>

```ml
fn fizzbuzz() {
    for i in 1..=100 {
        let output = match (i % 3, i % 5) {
            (0, 0) => "FizzBuzz".to_string(),
            (0, _) => "Fizz".to_string(),
            (_, 0) => "Buzz".to_string(),
            _ => i.to_string(),
        };
        print(output);
    }
}
```
</details>

### Exercise 2: Word Counter

Write a function that counts words in a string.

<details>
<summary>Solution</summary>

```ml
fn count_words(text: String) -> Int {
    text.split_whitespace().count()
}

fn main() {
    let text = "Hello world this is a test";
    print("Word count: {count_words(text)}");  // 6
}
```
</details>

### Exercise 3: Simple Calculator

Write a calculator that uses AI to interpret natural language.

<details>
<summary>Solution</summary>

```ml
fn calculate(query: String) -> String {
    ai! { "Calculate and return only the number: {query}" }
}

fn main() {
    print(calculate("What is 25 plus 17?"));
    print(calculate("What is 144 divided by 12?"));
}
```
</details>

## Next Steps

- [Working with AI](ai-queries.md) - Deep dive into AI features
- [Building a Web App](web-app.md) - Create a web application
- [Building a CLI Tool](cli-tool.md) - Create a command-line tool
