// Hello World in My Language
// Run with: cargo run -- run examples/hello.ml

fn main() {
    println("Hello, World!");

    // Variables
    let x = 42;
    let name = "My Language";
    println(str_concat("Welcome to ", name));

    // Arithmetic
    let result = factorial(5);
    println(str_concat("5! = ", to_string(result)));

    // Arrays
    let numbers = [1, 2, 3, 4, 5];
    println(str_concat("Array length: ", to_string(len(numbers))));

    // Conditionals
    if x > 40 {
        println("x is greater than 40");
    } else {
        println("x is 40 or less");
    }

    // String operations
    let greeting = str_upper("hello");
    println(greeting);

    // Math functions
    let pi = PI;
    println(str_concat("PI = ", to_string(pi)));
    println(str_concat("sqrt(2) = ", to_string(sqrt(2.0))));
}

fn factorial(n: Int) -> Int {
    if n <= 1 {
        return 1;
    } else {
        return n * factorial(n - 1);
    }
}
