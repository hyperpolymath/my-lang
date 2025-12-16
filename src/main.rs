//! My Language CLI
//!
//! A programming language with first-class AI integration.

use std::env;
use std::fs;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: my-lang <command> [file]");
        eprintln!();
        eprintln!("Commands:");
        eprintln!("  parse <file>      Parse a source file and print the AST");
        eprintln!("  lex <file>        Tokenize a source file");
        eprintln!("  check <file>      Parse and validate syntax");
        eprintln!("  typecheck <file>  Parse and type-check a source file");
        eprintln!("  compile <file>    Full compilation (parse + typecheck)");
        eprintln!("  repl              Interactive REPL");
        eprintln!();
        eprintln!("Examples:");
        eprintln!("  my-lang parse example.ml");
        eprintln!("  my-lang typecheck example.ml");
        process::exit(1);
    }

    let command = &args[1];

    match command.as_str() {
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
        _ => {
            eprintln!("Unknown command: {}", command);
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
    use std::io::{self, BufRead, Write};

    println!("My Language REPL (type 'exit' to quit)");
    println!();

    let stdin = io::stdin();
    let mut stdout = io::stdout();

    loop {
        print!("> ");
        stdout.flush().unwrap();

        let mut line = String::new();
        if stdin.lock().read_line(&mut line).is_err() {
            break;
        }

        let line = line.trim();
        if line == "exit" || line == "quit" {
            break;
        }

        if line.is_empty() {
            continue;
        }

        // Try to parse as an expression wrapped in a function
        let wrapped = format!("fn __repl__() {{ {}; }}", line);
        match my_lang::parse(&wrapped) {
            Ok(program) => {
                if let Some(my_lang::TopLevel::Function(f)) = program.items.first() {
                    for stmt in &f.body.stmts {
                        println!("{:#?}", stmt);
                    }
                }
            }
            Err(_) => {
                // Try parsing as top-level
                match my_lang::parse(line) {
                    Ok(program) => {
                        for item in &program.items {
                            println!("{:#?}", item);
                        }
                    }
                    Err(e) => {
                        eprintln!("Error: {}", e);
                    }
                }
            }
        }
    }
}

fn item_summary(item: &my_lang::TopLevel) -> String {
    match item {
        my_lang::TopLevel::Function(f) => format!("fn {}", f.name.name),
        my_lang::TopLevel::Struct(s) => format!("struct {}", s.name.name),
        my_lang::TopLevel::Effect(e) => format!("effect {}", e.name.name),
        my_lang::TopLevel::AiModel(m) => format!("ai_model {}", m.name.name),
        my_lang::TopLevel::Prompt(p) => format!("prompt {}", p.name.name),
        my_lang::TopLevel::Import(i) => {
            format!("use {}", i.path.iter().map(|p| p.name.as_str()).collect::<Vec<_>>().join("::"))
        }
        my_lang::TopLevel::Comptime(_) => "comptime { ... }".to_string(),
        my_lang::TopLevel::Arena(a) => format!("arena {}", a.name.name),
        my_lang::TopLevel::Contract(c) => format!("contract {:?}", c),
    }
}
