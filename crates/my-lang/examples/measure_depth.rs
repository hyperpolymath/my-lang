// SPDX-License-Identifier: PMPL-1.0-or-later
//! Stack-budget probe for hyperpolymath/my-lang#37.
//!
//! Builds a deep, *non-`Call`-shaped* AST (a `Unary::Not` chain — the shape
//! the old test helper could not handle) and exercises one recursive walk on
//! a thread with a known, fixed stack size. The process aborts on stack
//! overflow, so a wrapper runs this at increasing depths and reads the exit
//! status to find the overflow cliff. From `cliff_depth` at a known
//! `stack_bytes` we get `bytes_per_level ≈ stack_bytes / cliff_depth`.
//!
//! Usage: measure_depth <check|drop_recursive|drop_iter> <depth> <stack_kib>

use my_lang::ast::*;
use my_lang::token::Span;

fn build_unary_chain(depth: usize) -> Expr {
    let span = Span::default();
    let mut e = Expr::Literal(Literal::Bool(true, span));
    for _ in 0..depth {
        e = Expr::Unary {
            op: UnaryOp::Not,
            operand: Box::new(e),
            span,
        };
    }
    e
}

fn wrap(value: Expr) -> Program {
    let span = Span::default();
    Program {
        items: vec![TopLevel::Function(FnDecl {
            modifiers: vec![],
            name: Ident::new("main", span),
            params: vec![],
            return_type: None,
            contract: None,
            body: Block {
                stmts: vec![Stmt::Let {
                    mutable: false,
                    name: Ident::new("s", span),
                    ty: None,
                    value,
                    span,
                }],
                span,
            },
            span,
        })],
    }
}

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let mode = a.get(1).cloned().unwrap_or_default();
    let depth: usize = a.get(2).and_then(|s| s.parse().ok()).unwrap_or(1000);
    let stack_kib: usize = a.get(3).and_then(|s| s.parse().ok()).unwrap_or(1024);

    let h = std::thread::Builder::new()
        .stack_size(stack_kib * 1024)
        .spawn(move || {
            let program = wrap(build_unary_chain(depth));
            match mode.as_str() {
                "check" => {
                    // check_expr recursion (the guard is what #37 re-derives).
                    let _ = my_lang::check(&program);
                    my_lang::ast::drop_program_iteratively(program);
                }
                "drop_recursive" => {
                    // The unguarded danger: auto-derived recursive Drop.
                    drop(program);
                }
                "drop_iter" => {
                    // The #37 fix: general iterative teardown.
                    my_lang::ast::drop_program_iteratively(program);
                }
                _ => eprintln!("unknown mode"),
            }
            println!("OK depth={depth} stack_kib={stack_kib} mode={mode}");
        })
        .unwrap();
    h.join().unwrap();
}
