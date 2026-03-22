# Operators Reference

Complete reference for all operators in My Language.

## Operator Precedence

From highest to lowest precedence:

| Level | Operators | Associativity | Description |
|-------|-----------|---------------|-------------|
| 1 | `()` `[]` `.` `?` | Left | Grouping, indexing, field access, try |
| 2 | `!` `-` `*` `&` | Right | Unary operators |
| 3 | `**` | Right | Exponentiation |
| 4 | `*` `/` `%` | Left | Multiplication, division, remainder |
| 5 | `+` `-` | Left | Addition, subtraction |
| 6 | `<<` `>>` | Left | Bit shifts |
| 7 | `&` | Left | Bitwise AND |
| 8 | `^` | Left | Bitwise XOR |
| 9 | `\|` | Left | Bitwise OR |
| 10 | `==` `!=` `<` `>` `<=` `>=` | Left | Comparisons |
| 11 | `&&` | Left | Logical AND |
| 12 | `\|\|` | Left | Logical OR |
| 13 | `..` `..=` | Left | Range |
| 14 | `\|>` | Left | Pipe |
| 15 | `=` `+=` `-=` etc. | Right | Assignment |

## Arithmetic Operators

### Binary Arithmetic

| Operator | Name | Example | Result |
|----------|------|---------|--------|
| `+` | Addition | `5 + 3` | `8` |
| `-` | Subtraction | `5 - 3` | `2` |
| `*` | Multiplication | `5 * 3` | `15` |
| `/` | Division | `7 / 2` | `3` (Int) or `3.5` (Float) |
| `%` | Remainder | `7 % 3` | `1` |
| `**` | Exponentiation | `2 ** 3` | `8` |

```ml
let sum = 10 + 5;        // 15
let diff = 10 - 5;       // 5
let product = 10 * 5;    // 50
let quotient = 10 / 3;   // 3 (integer division)
let remainder = 10 % 3;  // 1
let power = 2 ** 10;     // 1024

// Float division
let precise = 10.0 / 3.0;  // 3.333...
```

### Unary Arithmetic

| Operator | Name | Example | Result |
|----------|------|---------|--------|
| `-` | Negation | `-5` | `-5` |
| `+` | Identity | `+5` | `5` |

```ml
let x = 5;
let neg = -x;  // -5
let pos = +x;  // 5
```

## Comparison Operators

| Operator | Name | Example | Result |
|----------|------|---------|--------|
| `==` | Equal | `5 == 5` | `true` |
| `!=` | Not equal | `5 != 3` | `true` |
| `<` | Less than | `3 < 5` | `true` |
| `>` | Greater than | `5 > 3` | `true` |
| `<=` | Less or equal | `3 <= 3` | `true` |
| `>=` | Greater or equal | `5 >= 5` | `true` |

```ml
let equal = 5 == 5;      // true
let not_eq = 5 != 3;     // true
let less = 3 < 5;        // true
let greater = 5 > 3;     // true
let less_eq = 3 <= 3;    // true
let greater_eq = 5 >= 5; // true

// Chaining comparisons
let in_range = 0 < x && x < 10;

// String comparison (lexicographic)
let cmp = "apple" < "banana";  // true
```

## Logical Operators

| Operator | Name | Example | Result |
|----------|------|---------|--------|
| `&&` | Logical AND | `true && false` | `false` |
| `\|\|` | Logical OR | `true \|\| false` | `true` |
| `!` | Logical NOT | `!true` | `false` |

```ml
let both = true && false;   // false
let either = true || false; // true
let negate = !true;         // false

// Short-circuit evaluation
let result = false && expensive();  // expensive() not called
let result = true || expensive();   // expensive() not called
```

## Bitwise Operators

| Operator | Name | Example | Result |
|----------|------|---------|--------|
| `&` | Bitwise AND | `0b1100 & 0b1010` | `0b1000` |
| `\|` | Bitwise OR | `0b1100 \| 0b1010` | `0b1110` |
| `^` | Bitwise XOR | `0b1100 ^ 0b1010` | `0b0110` |
| `~` | Bitwise NOT | `~0b1100` | `...0011` |
| `<<` | Left shift | `1 << 4` | `16` |
| `>>` | Right shift | `16 >> 2` | `4` |

```ml
let and = 0b1100 & 0b1010;  // 0b1000 = 8
let or = 0b1100 | 0b1010;   // 0b1110 = 14
let xor = 0b1100 ^ 0b1010;  // 0b0110 = 6
let not = ~0b1100;          // ...11110011

let left = 1 << 4;   // 16
let right = 16 >> 2; // 4

// Flags
const READ: Int = 1 << 0;   // 1
const WRITE: Int = 1 << 1;  // 2
const EXEC: Int = 1 << 2;   // 4

let permissions = READ | WRITE;
let can_read = (permissions & READ) != 0;
```

## Assignment Operators

| Operator | Name | Equivalent |
|----------|------|------------|
| `=` | Assignment | `x = value` |
| `+=` | Add assign | `x = x + value` |
| `-=` | Subtract assign | `x = x - value` |
| `*=` | Multiply assign | `x = x * value` |
| `/=` | Divide assign | `x = x / value` |
| `%=` | Remainder assign | `x = x % value` |
| `&=` | Bitwise AND assign | `x = x & value` |
| `\|=` | Bitwise OR assign | `x = x \| value` |
| `^=` | Bitwise XOR assign | `x = x ^ value` |
| `<<=` | Left shift assign | `x = x << value` |
| `>>=` | Right shift assign | `x = x >> value` |

```ml
let mut x = 10;
x += 5;   // x = 15
x -= 3;   // x = 12
x *= 2;   // x = 24
x /= 4;   // x = 6
x %= 4;   // x = 2

let mut flags = 0;
flags |= READ;   // Add READ flag
flags &= ~WRITE; // Remove WRITE flag
```

## Range Operators

| Operator | Name | Example | Description |
|----------|------|---------|-------------|
| `..` | Exclusive range | `0..5` | 0, 1, 2, 3, 4 |
| `..=` | Inclusive range | `0..=5` | 0, 1, 2, 3, 4, 5 |

```ml
// In for loops
for i in 0..5 {
    print(i);  // 0, 1, 2, 3, 4
}

for i in 0..=5 {
    print(i);  // 0, 1, 2, 3, 4, 5
}

// In patterns
match n {
    0..=9 => "single digit",
    10..=99 => "double digit",
    _ => "large",
}

// Slice indexing
let slice = &array[1..4];   // Elements 1, 2, 3
let slice = &array[..3];    // Elements 0, 1, 2
let slice = &array[2..];    // From 2 to end
let slice = &array[..];     // All elements
```

## Access Operators

| Operator | Name | Example | Description |
|----------|------|---------|-------------|
| `.` | Field access | `point.x` | Access struct field |
| `::` | Path separator | `std::io::Read` | Module/type path |
| `[]` | Index | `array[0]` | Access by index |

```ml
// Field access
let x = point.x;
let name = user.name;

// Method call
let len = string.len();

// Chained access
let city = user.address.city;

// Path access
use std::collections::HashMap;
let map = HashMap::new();

// Indexing
let first = array[0];
let char = string[3];
let value = map["key"];

// Chained indexing
let nested = matrix[0][1];
```

## Reference Operators

| Operator | Name | Example | Description |
|----------|------|---------|-------------|
| `&` | Borrow | `&value` | Immutable reference |
| `&mut` | Mutable borrow | `&mut value` | Mutable reference |
| `*` | Dereference | `*ref` | Access referenced value |

```ml
let x = 5;
let ref_x = &x;      // Immutable reference
let val = *ref_x;    // Dereference: 5

let mut y = 10;
let ref_y = &mut y;  // Mutable reference
*ref_y = 20;         // Modify through reference
```

## Error Handling Operators

| Operator | Name | Example | Description |
|----------|------|---------|-------------|
| `?` | Try | `result?` | Propagate error |

```ml
fn process() -> Result<Data, Error> {
    let file = open_file("data.txt")?;  // Return early on error
    let content = read_all(file)?;
    let data = parse(content)?;
    Ok(data)
}

// Equivalent to
fn process() -> Result<Data, Error> {
    let file = match open_file("data.txt") {
        Ok(f) => f,
        Err(e) => return Err(e),
    };
    // ...
}
```

## Pipe Operator

| Operator | Name | Example | Description |
|----------|------|---------|-------------|
| `\|>` | Pipe | `x \|> f` | Pass value to function |

```ml
// Without pipe
let result = process(transform(validate(parse(input))));

// With pipe
let result = input
    |> parse
    |> validate
    |> transform
    |> process;

// With arguments
let result = data
    |> filter(_, is_valid)
    |> map(_, transform)
    |> reduce(_, 0, add);
```

## Pattern Operators

| Operator | Name | Example | Description |
|----------|------|---------|-------------|
| `\|` | Or pattern | `Some(1) \| Some(2)` | Match either |
| `@` | Binding | `x @ Some(_)` | Bind while matching |
| `..` | Rest pattern | `[first, ..]` | Match rest |

```ml
// Or patterns
match value {
    1 | 2 | 3 => "small",
    _ => "other",
}

// Binding
match option {
    x @ Some(_) => print("Got {x:?}"),
    None => print("Nothing"),
}

// Rest patterns
let [first, second, ..rest] = array;

match list {
    [] => "empty",
    [x] => "one element",
    [first, .., last] => "multiple",
}
```

## Type Operators

| Operator | Name | Example | Description |
|----------|------|---------|-------------|
| `as` | Type cast | `x as Float` | Convert type |
| `:` | Type annotation | `let x: Int = 5` | Specify type |
| `->` | Return type | `fn f() -> Int` | Function return type |

```ml
// Type casting
let x: Int = 5;
let y: Float = x as Float;
let c: Char = 65 as Char;

// Type annotations
let list: List<Int> = List::new();
let map: Map<String, Int> = Map::new();

// Function types
fn process(data: Data) -> Result<Output, Error> {
    // ...
}
```

## Operator Traits

Operators are implemented via traits:

```ml
// Arithmetic
trait Add<Rhs = Self> {
    type Output;
    fn add(self, rhs: Rhs) -> Self::Output;
}

trait Sub<Rhs = Self> {
    type Output;
    fn sub(self, rhs: Rhs) -> Self::Output;
}

// Comparison
trait PartialEq<Rhs = Self> {
    fn eq(&self, other: &Rhs) -> Bool;
    fn ne(&self, other: &Rhs) -> Bool {
        !self.eq(other)
    }
}

trait PartialOrd<Rhs = Self>: PartialEq<Rhs> {
    fn partial_cmp(&self, other: &Rhs) -> Option<Ordering>;
    fn lt(&self, other: &Rhs) -> Bool;
    fn le(&self, other: &Rhs) -> Bool;
    fn gt(&self, other: &Rhs) -> Bool;
    fn ge(&self, other: &Rhs) -> Bool;
}

// Index
trait Index<Idx> {
    type Output;
    fn index(&self, index: Idx) -> &Self::Output;
}
```

## Custom Operators

Implement operators for custom types:

```ml
struct Vector2 {
    x: Float,
    y: Float,
}

impl Add for Vector2 {
    type Output = Vector2;

    fn add(self, other: Vector2) -> Vector2 {
        Vector2 {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl Neg for Vector2 {
    type Output = Vector2;

    fn neg(self) -> Vector2 {
        Vector2 { x: -self.x, y: -self.y }
    }
}

// Usage
let v1 = Vector2 { x: 1.0, y: 2.0 };
let v2 = Vector2 { x: 3.0, y: 4.0 };
let v3 = v1 + v2;  // Vector2 { x: 4.0, y: 6.0 }
let v4 = -v3;      // Vector2 { x: -4.0, y: -6.0 }
```
