//! My Language CLI
//!
//! A programming language with first-class AI integration.

use std::env;
use std::fs;
use std::io::{self, BufRead, Write};
use std::process;

use my_lang::{Interpreter, Value};

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        print_usage();
        process::exit(1);
    }

    let command = &args[1];

    match command.as_str() {
        "run" => {
            if args.len() < 3 {
                eprintln!("Error: run command requires a file argument");
                process::exit(1);
            }
            run_file(&args[2]);
        }
        "parse" => {
            if args.len() < 3 {
                eprintln!("Error: parse command requires a file argument");
                process::exit(1);
            }
            parse_file(&args[2]);
        }
        "lex" => {
            if args.len() < 3 {
                eprintln!("Error: lex command requires a file argument");
                process::exit(1);
            }
            lex_file(&args[2]);
        }
        "check" => {
            if args.len() < 3 {
                eprintln!("Error: check command requires a file argument");
                process::exit(1);
            }
            check_file(&args[2]);
        }
        "typecheck" => {
            if args.len() < 3 {
                eprintln!("Error: typecheck command requires a file argument");
                process::exit(1);
            }
            typecheck_file(&args[2]);
        }
        "compile" => {
            if args.len() < 3 {
                eprintln!("Error: compile command requires a file argument");
                process::exit(1);
            }
            compile_file(&args[2]);
        }
        "repl" => {
            run_repl();
        }
        "help" | "--help" | "-h" => {
            print_usage();
        }
        "version" | "--version" | "-v" => {
            println!("My Language v0.1.0");
        }
        _ => {
            // Try to run as a file if it looks like a path
            if command.ends_with(".ml") || command.ends_with(".mylang") || args.len() == 2 {
                run_file(command);
            } else {
                eprintln!("Unknown command: {}", command);
                print_usage();
                process::exit(1);
            }
        }
    }
}

fn print_usage() {
    eprintln!("My Language - A programming language with first-class AI integration");
    eprintln!();
    eprintln!("Usage: my-lang <command> [file]");
    eprintln!();
    eprintln!("Commands:");
    eprintln!("  run <file>        Run a source file with the interpreter");
    eprintln!("  repl              Start interactive REPL");
    eprintln!("  parse <file>      Parse a source file and print the AST");
    eprintln!("  lex <file>        Tokenize a source file");
    eprintln!("  check <file>      Parse and validate syntax");
    eprintln!("  typecheck <file>  Parse and type-check a source file");
    eprintln!("  compile <file>    Full compilation (parse + typecheck)");
    eprintln!("  help              Show this help message");
    eprintln!("  version           Show version information");
    eprintln!();
    eprintln!("Examples:");
    eprintln!("  my-lang run example.ml");
    eprintln!("  my-lang repl");
    eprintln!("  my-lang typecheck example.ml");
}

fn run_file(path: &str) {
    let source = match fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error reading file '{}': {}", path, e);
            process::exit(1);
        }
    };

    match my_lang::eval(&source) {
        Ok(value) => {
            // Only print non-unit return values
            if !matches!(value, Value::Unit) {
                println!("{}", value);
            }
        }
        Err(e) => {
            eprintln!("{}", e);
            process::exit(1);
        }
    }
}

fn parse_file(path: &str) {
    let source = match fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error reading file '{}': {}", path, e);
            process::exit(1);
        }
    };

    match my_lang::parse(&source) {
        Ok(program) => {
            println!("Parsed {} top-level items:", program.items.len());
            for (i, item) in program.items.iter().enumerate() {
                println!("  {}. {:?}", i + 1, item_summary(item));
            }
        }
        Err(e) => {
            eprintln!("Parse error: {}", e);
            process::exit(1);
        }
    }
}

fn lex_file(path: &str) {
    let source = match fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error reading file '{}': {}", path, e);
            process::exit(1);
        }
    };

    let mut lexer = my_lang::Lexer::new(&source);
    let tokens = lexer.tokenize();

    println!("Tokens ({}):", tokens.len());
    for token in &tokens {
        println!(
            "  {:?} '{}' at {}:{}",
            token.kind, token.literal, token.span.line, token.span.column
        );
    }
}

fn check_file(path: &str) {
    let source = match fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error reading file '{}': {}", path, e);
            process::exit(1);
        }
    };

    match my_lang::parse(&source) {
        Ok(program) => {
            println!("OK: {} parsed successfully", path);
            println!("    {} top-level items", program.items.len());
        }
        Err(e) => {
            eprintln!("FAIL: {}", e);
            process::exit(1);
        }
    }
}

fn typecheck_file(path: &str) {
    let source = match fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error reading file '{}': {}", path, e);
            process::exit(1);
        }
    };

    let program = match my_lang::parse(&source) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Parse error: {}", e);
            process::exit(1);
        }
    };

    match my_lang::check(&program) {
        Ok(()) => {
            println!("OK: {} type-checked successfully", path);
            println!("    {} top-level items", program.items.len());
        }
        Err(errors) => {
            eprintln!("Type errors in {}:", path);
            for error in &errors {
                eprintln!("  - {}", error);
            }
            process::exit(1);
        }
    }
}

fn compile_file(path: &str) {
    let source = match fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error reading file '{}': {}", path, e);
            process::exit(1);
        }
    };

    match my_lang::compile(&source) {
        Ok(program) => {
            println!("OK: {} compiled successfully", path);
            println!("    {} top-level items", program.items.len());
        }
        Err(e) => {
            eprintln!("Compilation failed: {}", e);
            process::exit(1);
        }
    }
}

fn run_repl() {
    println!("My Language REPL v0.1.0");
    println!("Type 'help' for commands, 'exit' to quit");
    println!();

    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let mut interpreter = Interpreter::new();
    let mut multiline_buffer = String::new();
    let mut in_multiline = false;

    loop {
        // Print prompt
        if in_multiline {
            print!("... ");
        } else {
            print!(">>> ");
        }
        stdout.flush().unwrap();

        // Read line
        let mut line = String::new();
        if stdin.lock().read_line(&mut line).is_err() {
            break;
        }

        let line = line.trim_end();

        // Handle multiline input
        if in_multiline {
            if line.is_empty() {
                // Empty line ends multiline input
                in_multiline = false;
                let input = std::mem::take(&mut multiline_buffer);
                eval_repl_input(&mut interpreter, &input);
                continue;
            }
            multiline_buffer.push_str(line);
            multiline_buffer.push('\n');
            continue;
        }

        // Handle special commands
        match line {
            "exit" | "quit" | ":q" => break,
            "help" | ":h" | ":help" => {
                print_repl_help();
                continue;
            }
            "clear" | ":c" => {
                // Clear screen (ANSI escape)
                print!("\x1B[2J\x1B[1;1H");
                stdout.flush().unwrap();
                continue;
            }
            "reset" | ":r" => {
                interpreter = Interpreter::new();
                println!("Interpreter reset.");
                continue;
            }
            "" => continue,
            _ => {}
        }

        // Check if this starts a multiline block
        if line.ends_with('{') && !line.contains('}') {
            in_multiline = true;
            multiline_buffer = line.to_string();
            multiline_buffer.push('\n');
            continue;
        }

        // Evaluate the input
        eval_repl_input(&mut interpreter, line);
    }

    println!("Goodbye!");
}

fn eval_repl_input(interpreter: &mut Interpreter, input: &str) {
    // Try different parsing strategies

    // 1. Try as a complete program (for fn, struct, etc.)
    if let Ok(program) = my_lang::parse(input) {
        // Register any top-level declarations
        for item in &program.items {
            match item {
                my_lang::TopLevel::Function(func) => {
                    let fn_value = Value::Function(std::rc::Rc::new(
                        my_lang::interpreter::FunctionValue {
                            name: func.name.name.clone(),
                            params: func.params.iter().map(|p| p.name.name.clone()).collect(),
                            body: func.body.clone(),
                            closure: interpreter.env.clone(),
                        },
                    ));
                    interpreter
                        .env
                        .borrow_mut()
                        .define(func.name.name.clone(), fn_value);
                    println!("Defined function: {}", func.name.name);
                }
                my_lang::TopLevel::Struct(s) => {
                    interpreter.structs.insert(s.name.name.clone(), s.clone());
                    println!("Defined struct: {}", s.name.name);
                }
                my_lang::TopLevel::AiModel(m) => {
                    interpreter.ai_models.insert(m.name.name.clone(), m.clone());
                    println!("Defined ai_model: {}", m.name.name);
                }
                my_lang::TopLevel::Prompt(p) => {
                    interpreter.prompts.insert(p.name.name.clone(), p.clone());
                    println!("Defined prompt: {}", p.name.name);
                }
                _ => {
                    println!("{:#?}", item);
                }
            }
        }
        return;
    }

    // 2. Try as a statement wrapped in a function
    let wrapped_stmt = format!("fn __repl__() {{ {} }}", input);
    if let Ok(program) = my_lang::parse(&wrapped_stmt) {
        if let Some(my_lang::TopLevel::Function(f)) = program.items.first() {
            for stmt in &f.body.stmts {
                match interpreter.exec(stmt) {
                    Ok(value) => {
                        if !matches!(value, Value::Unit) {
                            println!("{}", value);
                        }
                    }
                    Err(my_lang::RuntimeError::Return(value)) => {
                        println!("{}", value);
                    }
                    Err(e) => {
                        eprintln!("Runtime error: {}", e);
                    }
                }
            }
        }
        return;
    }

    // 3. Try as an expression wrapped in a function
    let wrapped_expr = format!("fn __repl__() {{ {}; }}", input);
    if let Ok(program) = my_lang::parse(&wrapped_expr) {
        if let Some(my_lang::TopLevel::Function(f)) = program.items.first() {
            for stmt in &f.body.stmts {
                if let my_lang::Stmt::Expr(expr) = stmt {
                    match interpreter.eval(expr) {
                        Ok(value) => {
                            println!("{}", value);
                        }
                        Err(e) => {
                            eprintln!("Runtime error: {}", e);
                        }
                    }
                }
            }
        }
        return;
    }

    // If all parsing fails, show error
    eprintln!("Parse error: Could not parse input");
}

fn print_repl_help() {
    println!("REPL Commands:");
    println!("  help, :h     Show this help");
    println!("  exit, :q     Exit the REPL");
    println!("  clear, :c    Clear the screen");
    println!("  reset, :r    Reset the interpreter state");
    println!();
    println!("You can enter:");
    println!("  - Expressions: 1 + 2, \"hello\" + \" world\"");
    println!("  - Statements: let x = 5; println(x);");
    println!("  - Definitions: fn add(a: Int, b: Int) -> Int {{ return a + b; }}");
    println!();
    println!("Multiline input:");
    println!("  Start a block with '{{' and end with an empty line");
}

fn item_summary(item: &my_lang::TopLevel) -> String {
    match item {
        my_lang::TopLevel::Function(f) => format!("fn {}", f.name.name),
        my_lang::TopLevel::Struct(s) => format!("struct {}", s.name.name),
        my_lang::TopLevel::Effect(e) => format!("effect {}", e.name.name),
        my_lang::TopLevel::AiModel(m) => format!("ai_model {}", m.name.name),
        my_lang::TopLevel::Prompt(p) => format!("prompt {}", p.name.name),
        my_lang::TopLevel::Import(i) => {
            format!(
                "use {}",
                i.path
                    .iter()
                    .map(|p| p.name.as_str())
                    .collect::<Vec<_>>()
                    .join("::")
            )
        }
        my_lang::TopLevel::Comptime(_) => "comptime { ... }".to_string(),
        my_lang::TopLevel::Arena(a) => format!("arena {}", a.name.name),
        my_lang::TopLevel::Contract(c) => format!("contract {:?}", c),
    }
}
