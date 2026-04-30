// SPDX-License-Identifier: PMPL-1.0-or-later
//! My Language CLI - Command-line interface for My Language compiler and interpreter

#![forbid(unsafe_code)]
use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Parser)]
#[command(name = "my")]
#[command(about = "My Language - AI-native programming language", long_about = None)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run a My Language file with the interpreter
    Run {
        file: PathBuf,
        /// Args passed to the program (visible to env_args()). Use `--` to separate.
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },

    /// Parse a file and show the AST
    Parse {
        file: PathBuf,

        /// Output format: debug (default), sexpr, json
        #[arg(long, default_value = "debug")]
        output: String,
    },

    /// Dump AST as S-expression (shorthand for parse --output sexpr)
    DumpSexpr { file: PathBuf },

    /// Tokenize a file and show tokens
    Lex { file: PathBuf },

    /// Type-check a file
    Check { file: PathBuf },

    /// Compile to MIR and run with MIR interpreter
    Interpret { file: PathBuf },

    /// Compile to a native binary via LLVM (requires --features llvm)
    Build {
        file: PathBuf,
        /// Output binary path (default: input filename without extension)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Start an interactive REPL
    Repl,
}

fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Run { file, args } => {
            my_lang::stdlib::set_program_args(args);
            run_file(&file)
        }
        Commands::Parse { file, output } => parse_file(&file, &output),
        Commands::DumpSexpr { file } => parse_file(&file, "sexpr"),
        Commands::Lex { file } => lex_file(&file),
        Commands::Check { file } => check_file(&file),
        Commands::Interpret { file } => interpret_file(&file),
        Commands::Build { file, output } => build_native(&file, output.as_deref()),
        Commands::Repl => run_repl(),
    }
}

fn run_file(path: &PathBuf) -> Result<()> {
    let source = fs::read_to_string(path)
        .with_context(|| format!("Failed to read file: {}", path.display()))?;

    // Parse
    let program = my_lang::parse(&source)
        .map_err(|e| anyhow::anyhow!("Parse error: {:?}", e))?;

    // Type check
    my_lang::check(&program)
        .map_err(|e| anyhow::anyhow!("Type check error: {:?}", e))?;

    // Interpret with AST interpreter
    let mut interpreter = my_lang::Interpreter::new();
    let result = interpreter.run(&program)
        .map_err(|e| anyhow::anyhow!("Runtime error: {:?}", e))?;

    println!("Result: {:?}", result);
    Ok(())
}

/// Parse a file and display the AST in the requested output format.
///
/// Supported formats:
///   - "debug" — Rust Debug pretty-print (default)
///   - "sexpr" — S-expression representation following JtV reference pattern
///   - "json"  — Pretty-printed JSON via serde
fn parse_file(path: &PathBuf, format: &str) -> Result<()> {
    let source = fs::read_to_string(path)
        .with_context(|| format!("Failed to read file: {}", path.display()))?;

    let program = my_lang::parse(&source)
        .map_err(|e| anyhow::anyhow!("Parse error: {:?}", e))?;

    match format {
        "json" => {
            let json_str = serde_json::to_string_pretty(&program)
                .with_context(|| "JSON serialization failed")?;
            println!("{}", json_str);
        }
        "sexpr" | "s-expr" | "sexp" => {
            println!("{}", program_to_sexpr(&program));
        }
        "debug" | _ => {
            println!("{:#?}", program);
        }
    }
    Ok(())
}

// ============================================================================
// S-expression AST dump
//
// Converts the My Language AST into a Lisp-like S-expression representation,
// following the Julia-the-Viper (JtV) reference pattern. Each AST node
// becomes a parenthesised list tagged by its variant name.
// ============================================================================

/// Convert an entire My Language program to S-expression format.
fn program_to_sexpr(program: &my_lang::Program) -> String {
    let mut out = String::new();
    out.push_str("(program");
    for item in &program.items {
        out.push_str("\n  ");
        top_level_to_sexpr(item, &mut out, 2);
    }
    out.push(')');
    out
}

/// Emit a top-level declaration as an S-expression.
fn top_level_to_sexpr(tl: &my_lang::TopLevel, out: &mut String, indent: usize) {
    use my_lang::TopLevel;
    match tl {
        TopLevel::Function(f) => fn_decl_to_sexpr(f, out, indent),
        TopLevel::Struct(s) => {
            out.push_str(&format!("(struct \"{}\"", s.name.name));
            if !s.type_params.is_empty() {
                out.push_str(" (type-params");
                for p in &s.type_params {
                    out.push_str(&format!(" \"{}\"", p.name));
                }
                out.push(')');
            }
            for field in &s.fields {
                out.push_str("\n");
                out.push_str(&" ".repeat(indent + 2));
                out.push_str(&format!("(field \"{}\" ", field.name.name));
                type_to_sexpr(&field.ty, out);
                out.push(')');
            }
            out.push(')');
        }
        TopLevel::Effect(e) => {
            out.push_str(&format!("(effect \"{}\"", e.name.name));
            for op in &e.ops {
                out.push_str(&format!(" (op \"{}\" ", op.name.name));
                type_to_sexpr(&op.ty, out);
                out.push(')');
            }
            out.push(')');
        }
        TopLevel::Contract(c) => {
            out.push_str(&format!("(contract \"{}\"", c.name.name));
            contract_to_sexpr(&c.contract, out, indent + 2);
            out.push(')');
        }
        TopLevel::Import(i) => {
            let path_str: Vec<&str> = i.path.iter().map(|id| id.name.as_str()).collect();
            out.push_str(&format!("(import \"{}\"", path_str.join(".")));
            if let Some(items) = &i.items {
                out.push_str(" (items");
                for item in items {
                    out.push_str(&format!(" \"{}\"", item.name));
                }
                out.push(')');
            }
            out.push(')');
        }
        TopLevel::Comptime(c) => {
            out.push_str("(comptime");
            block_to_sexpr(&c.block, out, indent + 2);
            out.push(')');
        }
        TopLevel::Arena(a) => {
            out.push_str(&format!("(arena \"{}\")", a.name.name));
        }
        TopLevel::AiModel(m) => {
            out.push_str(&format!("(ai-model \"{}\"", m.name.name));
            for attr in &m.attributes {
                out.push_str(&format!(" {:?}", attr));
            }
            out.push(')');
        }
        TopLevel::Prompt(p) => {
            out.push_str(&format!("(prompt \"{}\" {:?})", p.name.name, p.template));
        }
        TopLevel::Agent(a) => {
            out.push_str(&format!("(agent \"{}\"", a.name.name));
            if !a.capabilities.is_empty() {
                out.push_str(" (capabilities");
                for c in &a.capabilities {
                    out.push_str(&format!(" \"{}\"", c.name));
                }
                out.push(')');
            }
            for f in &a.functions {
                out.push_str("\n");
                out.push_str(&" ".repeat(indent + 2));
                fn_decl_to_sexpr(f, out, indent + 2);
            }
            out.push(')');
        }
        TopLevel::Ensemble(e) => {
            out.push_str(&format!("(ensemble \"{}\"", e.name.name));
            out.push_str(" (agents");
            for a in &e.agents {
                out.push_str(&format!(" \"{}\"", a.name));
            }
            out.push(')');
            for w in &e.workflows {
                out.push_str("\n");
                out.push_str(&" ".repeat(indent + 2));
                out.push_str(&format!("(workflow \"{}\"", w.name.name));
                block_to_sexpr(&w.body, out, indent + 4);
                out.push(')');
            }
            out.push(')');
        }
        TopLevel::Orchestrate(o) => {
            out.push_str(&format!("(orchestrate \"{}\" {:?} {:?} {:?})",
                o.name.name, o.fusion, o.consensus, o.topology));
        }
    }
}

/// Emit a function declaration as an S-expression.
fn fn_decl_to_sexpr(f: &my_lang::FnDecl, out: &mut String, indent: usize) {
    out.push_str(&format!("(fn \"{}\"", f.name.name));
    if !f.modifiers.is_empty() {
        out.push_str(&format!(" (modifiers {:?})", f.modifiers));
    }
    // Parameters
    out.push_str("\n");
    out.push_str(&" ".repeat(indent + 2));
    out.push_str("(params");
    for p in &f.params {
        out.push_str(&format!(" (\"{}\" ", p.name.name));
        type_to_sexpr(&p.ty, out);
        out.push(')');
    }
    out.push(')');
    // Return type
    if let Some(ret) = &f.return_type {
        out.push_str("\n");
        out.push_str(&" ".repeat(indent + 2));
        out.push_str("(returns ");
        type_to_sexpr(ret, out);
        out.push(')');
    }
    // Contract
    if let Some(contract) = &f.contract {
        out.push_str("\n");
        out.push_str(&" ".repeat(indent + 2));
        contract_to_sexpr(contract, out, indent + 2);
    }
    // Body
    block_to_sexpr(&f.body, out, indent + 2);
    out.push(')');
}

/// Emit a contract as an S-expression.
fn contract_to_sexpr(contract: &my_lang::Contract, out: &mut String, indent: usize) {
    out.push_str("(contract");
    for clause in &contract.clauses {
        out.push_str("\n");
        out.push_str(&" ".repeat(indent + 2));
        match clause {
            my_lang::ContractClause::Pre(e) => {
                out.push_str("(pre ");
                expr_to_sexpr(e, out, indent + 4);
                out.push(')');
            }
            my_lang::ContractClause::Post(e) => {
                out.push_str("(post ");
                expr_to_sexpr(e, out, indent + 4);
                out.push(')');
            }
            my_lang::ContractClause::Invariant(e) => {
                out.push_str("(invariant ");
                expr_to_sexpr(e, out, indent + 4);
                out.push(')');
            }
            my_lang::ContractClause::AiCheck(s) => {
                out.push_str(&format!("(ai-check {:?})", s));
            }
            my_lang::ContractClause::AiEnsure(s) => {
                out.push_str(&format!("(ai-ensure {:?})", s));
            }
        }
    }
    out.push(')');
}

/// Emit a block of statements as S-expressions.
fn block_to_sexpr(block: &my_lang::Block, out: &mut String, indent: usize) {
    for stmt in &block.stmts {
        out.push_str("\n");
        out.push_str(&" ".repeat(indent));
        stmt_to_sexpr(stmt, out, indent);
    }
}

/// Emit a statement as an S-expression.
fn stmt_to_sexpr(stmt: &my_lang::Stmt, out: &mut String, indent: usize) {
    use my_lang::Stmt;
    match stmt {
        Stmt::Expr(e) => expr_to_sexpr(e, out, indent),
        Stmt::Let { mutable, name, ty, value, .. } => {
            let mut_tag = if *mutable { "let-mut" } else { "let" };
            out.push_str(&format!("({} \"{}\"", mut_tag, name.name));
            if let Some(t) = ty {
                out.push_str(" (: ");
                type_to_sexpr(t, out);
                out.push(')');
            }
            out.push(' ');
            expr_to_sexpr(value, out, indent + 2);
            out.push(')');
        }
        Stmt::If { condition, then_block, else_block, .. } => {
            out.push_str("(if ");
            expr_to_sexpr(condition, out, indent + 2);
            out.push_str("\n");
            out.push_str(&" ".repeat(indent + 2));
            out.push_str("(then");
            block_to_sexpr(then_block, out, indent + 4);
            out.push(')');
            if let Some(eb) = else_block {
                out.push_str("\n");
                out.push_str(&" ".repeat(indent + 2));
                out.push_str("(else");
                block_to_sexpr(eb, out, indent + 4);
                out.push(')');
            }
            out.push(')');
        }
        Stmt::Go { block, .. } => {
            out.push_str("(go");
            block_to_sexpr(block, out, indent + 2);
            out.push(')');
        }
        Stmt::Return { value, .. } => {
            out.push_str("(return");
            if let Some(v) = value {
                out.push(' ');
                expr_to_sexpr(v, out, indent + 2);
            }
            out.push(')');
        }
        Stmt::Await { value, .. } => {
            out.push_str("(await ");
            expr_to_sexpr(value, out, indent + 2);
            out.push(')');
        }
        Stmt::Try { value, propagate, .. } => {
            let tag = if *propagate { "try?" } else { "try" };
            out.push_str(&format!("({} ", tag));
            expr_to_sexpr(value, out, indent + 2);
            out.push(')');
        }
        Stmt::Comptime { block, .. } => {
            out.push_str("(comptime");
            block_to_sexpr(block, out, indent + 2);
            out.push(')');
        }
        Stmt::Ai(ai) => {
            out.push_str(&format!("(ai {:?})", ai.keyword));
        }
        Stmt::Belief { name, ty, confidence, .. } => {
            out.push_str(&format!("(belief \"{}\" ", name.name));
            type_to_sexpr(ty, out);
            out.push_str(&format!(" :confidence {})", confidence));
        }
    }
}

/// Emit an expression as an S-expression.
///
/// Core of the dump: every `Expr` variant maps to a named S-expression list.
fn expr_to_sexpr(expr: &my_lang::Expr, out: &mut String, indent: usize) {
    use my_lang::Expr;
    match expr {
        Expr::Literal(lit) => literal_to_sexpr(lit, out),
        Expr::Ident(id) => out.push_str(&format!("(ident \"{}\")", id.name)),
        Expr::Call { callee, args, .. } => {
            out.push_str("(call ");
            expr_to_sexpr(callee, out, indent + 2);
            for a in args {
                out.push(' ');
                expr_to_sexpr(a, out, indent + 2);
            }
            out.push(')');
        }
        Expr::Field { object, field, .. } => {
            out.push_str("(field-access ");
            expr_to_sexpr(object, out, indent + 2);
            out.push_str(&format!(" \"{}\")", field.name));
        }
        Expr::Binary { left, op, right, .. } => {
            out.push_str(&format!("({:?} ", op));
            expr_to_sexpr(left, out, indent + 2);
            out.push(' ');
            expr_to_sexpr(right, out, indent + 2);
            out.push(')');
        }
        Expr::Unary { op, operand, .. } => {
            out.push_str(&format!("({:?} ", op));
            expr_to_sexpr(operand, out, indent + 2);
            out.push(')');
        }
        Expr::Try { operand, .. } => {
            out.push_str("(try ");
            expr_to_sexpr(operand, out, indent + 2);
            out.push(')');
        }
        Expr::Block(block) => {
            out.push_str("(block");
            block_to_sexpr(block, out, indent + 2);
            out.push(')');
        }
        Expr::Restrict { operand, .. } => {
            out.push_str("(restrict ");
            expr_to_sexpr(operand, out, indent + 2);
            out.push(')');
        }
        Expr::Ai(ai) => {
            out.push_str(&format!("(ai-expr {:?})", ai));
        }
        Expr::Lambda { params, body, .. } => {
            out.push_str("(lambda (params");
            for p in params {
                out.push_str(&format!(" (\"{}\" ", p.name.name));
                type_to_sexpr(&p.ty, out);
                out.push(')');
            }
            out.push(')');
            match body {
                my_lang::LambdaBody::Expr(e) => {
                    out.push(' ');
                    expr_to_sexpr(e, out, indent + 2);
                }
                my_lang::LambdaBody::Block(b) => {
                    block_to_sexpr(b, out, indent + 2);
                }
            }
            out.push(')');
        }
        Expr::Match { scrutinee, arms, .. } => {
            out.push_str("(match ");
            expr_to_sexpr(scrutinee, out, indent + 2);
            for arm in arms {
                out.push_str("\n");
                out.push_str(&" ".repeat(indent + 2));
                out.push_str("(arm ");
                pattern_to_sexpr(&arm.pattern, out);
                out.push(' ');
                expr_to_sexpr(&arm.body, out, indent + 4);
                out.push(')');
            }
            out.push(')');
        }
        Expr::Array { elements, .. } => {
            out.push_str("(array");
            for e in elements {
                out.push(' ');
                expr_to_sexpr(e, out, indent + 2);
            }
            out.push(')');
        }
        Expr::Record { fields, .. } => {
            out.push_str("(record");
            for f in fields {
                out.push_str(&format!(" (\"{}\" ", f.name.name));
                expr_to_sexpr(&f.value, out, indent + 2);
                out.push(')');
            }
            out.push(')');
        }
    }
}

/// Emit a pattern as an S-expression.
fn pattern_to_sexpr(pat: &my_lang::Pattern, out: &mut String) {
    use my_lang::Pattern;
    match pat {
        Pattern::Literal(lit) => literal_to_sexpr(lit, out),
        Pattern::Ident(id) => out.push_str(&format!("\"{}\"", id.name)),
        Pattern::Wildcard(_) => out.push('_'),
        Pattern::Constructor { name, args, .. } => {
            out.push_str(&format!("(ctor \"{}\"", name.name));
            for a in args {
                out.push(' ');
                pattern_to_sexpr(a, out);
            }
            out.push(')');
        }
    }
}

/// Emit a literal as an S-expression.
fn literal_to_sexpr(lit: &my_lang::Literal, out: &mut String) {
    use my_lang::Literal;
    match lit {
        Literal::Int(v, _) => out.push_str(&format!("(int {})", v)),
        Literal::Float(v, _) => out.push_str(&format!("(float {})", v)),
        Literal::String(s, _) => out.push_str(&format!("(string {:?})", s)),
        Literal::Bool(b, _) => out.push_str(&format!("(bool {})", b)),
    }
}

/// Emit a type as an S-expression.
fn type_to_sexpr(ty: &my_lang::Type, out: &mut String) {
    use my_lang::Type;
    match ty {
        Type::Primitive(p) => out.push_str(&format!("{:?}", p)),
        Type::Named(id) => out.push_str(&format!("(type \"{}\")", id.name)),
        Type::Function { param, result, .. } => {
            out.push_str("(-> ");
            type_to_sexpr(param, out);
            out.push(' ');
            type_to_sexpr(result, out);
            out.push(')');
        }
        Type::Effect { inner, .. } => {
            out.push_str("(Effect ");
            type_to_sexpr(inner, out);
            out.push(')');
        }
        Type::Ai { inner, .. } => {
            out.push_str("(AI ");
            type_to_sexpr(inner, out);
            out.push(')');
        }
        Type::Reference { mutable, inner, .. } => {
            let tag = if *mutable { "&mut" } else { "&" };
            out.push_str(&format!("({} ", tag));
            type_to_sexpr(inner, out);
            out.push(')');
        }
        Type::Array { element, .. } => {
            out.push_str("(array-type ");
            type_to_sexpr(element, out);
            out.push(')');
        }
        Type::Record { fields, .. } => {
            out.push_str("(record-type");
            for f in fields {
                out.push_str(&format!(" (\"{}\" ", f.name.name));
                type_to_sexpr(&f.ty, out);
                out.push(')');
            }
            out.push(')');
        }
        Type::Tuple { elements, .. } => {
            out.push_str("(tuple-type");
            for e in elements {
                out.push(' ');
                type_to_sexpr(e, out);
            }
            out.push(')');
        }
        Type::Constrained { base, constraints, .. } => {
            out.push_str("(constrained ");
            type_to_sexpr(base, out);
            out.push_str(&format!(" {:?})", constraints));
        }
    }
}

fn lex_file(path: &PathBuf) -> Result<()> {
    let source = fs::read_to_string(path)
        .with_context(|| format!("Failed to read file: {}", path.display()))?;

    let mut lexer = my_lang::Lexer::new(&source);
    let tokens = lexer.tokenize();

    for token in tokens {
        println!("{:?}", token);
    }
    Ok(())
}

fn check_file(path: &PathBuf) -> Result<()> {
    let source = fs::read_to_string(path)
        .with_context(|| format!("Failed to read file: {}", path.display()))?;

    let program = my_lang::parse(&source)
        .map_err(|e| anyhow::anyhow!("Parse error: {:?}", e))?;

    my_lang::check(&program)
        .map_err(|e| anyhow::anyhow!("Type check error: {:?}", e))?;

    println!("✓ Type check passed");
    Ok(())
}

fn interpret_file(path: &PathBuf) -> Result<()> {
    let source = fs::read_to_string(path)
        .with_context(|| format!("Failed to read file: {}", path.display()))?;

    println!("=== Parsing ===");
    let program = my_lang::parse(&source)
        .map_err(|e| anyhow::anyhow!("Parse error: {:?}", e))?;
    println!("✓ Parse successful");

    println!("\n=== Type Checking ===");
    my_lang::check(&program)
        .map_err(|e| anyhow::anyhow!("Type check error: {:?}", e))?;
    println!("✓ Type check passed");

    println!("\n=== Lowering to HIR ===");
    let hir = my_hir::lower(&program)
        .map_err(|e| anyhow::anyhow!("HIR lowering error: {:?}", e))?;
    println!("✓ HIR lowering successful ({} items)", hir.items.len());

    println!("\n=== Lowering to MIR ===");
    let mir = my_mir::lower(&hir)
        .map_err(|e| anyhow::anyhow!("MIR lowering error: {:?}", e))?;
    println!("✓ MIR lowering successful ({} functions)", mir.functions.len());

    println!("\n=== Running MIR Interpreter ===");
    let mut interpreter = my_mir::interpreter::Interpreter::new(mir);
    let result = interpreter.run()
        .map_err(|e| anyhow::anyhow!("MIR runtime error: {:?}", e))?;

    println!("\n=== Execution Complete ===");
    println!("Result: {:?}", result);
    Ok(())
}

fn build_native(path: &PathBuf, output: Option<&Path>) -> Result<()> {
    #[cfg(not(feature = "llvm"))]
    {
        // Suppress unused variable warnings when llvm feature is off
        let _ = (path, output);
        anyhow::bail!(
            "LLVM backend not enabled. Rebuild with: cargo build --features llvm"
        );
    }

    #[cfg(feature = "llvm")]
    {
        let source = fs::read_to_string(path)
            .with_context(|| format!("Failed to read file: {}", path.display()))?;

        println!("=== Parsing ===");
        let program = my_lang::parse(&source)
            .map_err(|e| anyhow::anyhow!("Parse error: {:?}", e))?;
        println!("  Parsed {} top-level items", program.items.len());

        println!("=== Type Checking ===");
        my_lang::check(&program)
            .map_err(|e| anyhow::anyhow!("Type check error: {:?}", e))?;
        println!("  Type check passed");

        println!("=== Lowering to HIR ===");
        let hir = my_hir::lower(&program)
            .map_err(|e| anyhow::anyhow!("HIR lowering error: {:?}", e))?;
        println!("  HIR: {} items", hir.items.len());

        println!("=== Lowering to MIR ===");
        let mir = my_mir::lower(&hir)
            .map_err(|e| anyhow::anyhow!("MIR lowering error: {:?}", e))?;
        println!("  MIR: {} functions", mir.functions.len());

        // Determine output path
        let default_output = path.with_extension("");
        let output_path = output.unwrap_or(&default_output);

        println!("=== LLVM Code Generation ===");
        my_llvm::compile_program(
            &mir,
            &path.display().to_string(),
            output_path,
            my_llvm::OptLevel::Default,
        )
        .map_err(|e| anyhow::anyhow!("LLVM compilation failed: {}", e))?;

        println!("=== Build Complete ===");
        println!("  Output: {}", output_path.display());
        Ok(())
    }
}

fn run_repl() -> Result<()> {
    use std::io::{self, BufRead, Write};

    println!("My Language REPL v0.2.0");
    println!("Type 'exit' or 'quit' to exit\n");

    let stdin = io::stdin();
    let mut interpreter = my_lang::Interpreter::new();

    loop {
        print!(">>> ");
        io::stdout().flush()?;

        let mut line = String::new();
        stdin.lock().read_line(&mut line)?;
        let line = line.trim();

        if line.is_empty() {
            continue;
        }

        if line == "exit" || line == "quit" {
            break;
        }

        // Try to parse and evaluate the expression
        match my_lang::parse(&format!("fn main() {{ {} }}", line)) {
            Ok(program) => {
                if let Err(e) = my_lang::check(&program) {
                    eprintln!("Type error: {:?}", e);
                    continue;
                }

                match interpreter.run(&program) {
                    Ok(result) => println!("{:?}", result),
                    Err(e) => eprintln!("Runtime error: {:?}", e),
                }
            }
            Err(e) => {
                eprintln!("Parse error: {:?}", e);
            }
        }
    }

    println!("Goodbye!");
    Ok(())
}
