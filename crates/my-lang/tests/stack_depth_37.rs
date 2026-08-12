// SPDX-License-Identifier: MPL-2.0
// SPDX-FileCopyrightText: 2026 Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk>

//! Regression guard for hyperpolymath/my-lang#37.
//!
//! Subtlety 3 of #37: `MAX_EXPR_DEPTH` must be safe on the *smallest* target
//! main-thread stack. Windows defaults the process main thread to 1 MiB (Linux
//! is ~8 MiB), so 1 MiB is the binding budget. This test runs the guarded
//! checker walk and the AST teardown at `MAX_EXPR_DEPTH` on a thread pinned to
//! exactly that 1 MiB budget. If the value is ever raised past what fits, the
//! thread overflows and aborts the test process — a hard CI failure — so the
//! "Windows-CI leg" of the #37 measurement is produced automatically on every
//! OS the test matrix covers.
//!
//! For the *quantitative* per-level measurement (bytes/level, the overflow
//! cliff), see `examples/measure_depth.rs`, which is self-driving and prints a
//! per-platform report.

use my_lang::ast::*;
use my_lang::checker::MAX_EXPR_DEPTH;
use my_lang::token::Span;

/// The Windows process main-thread default, and thus the binding stack budget.
const WINDOWS_MAIN_THREAD_STACK: usize = 1024 * 1024;

/// A left-nested `!!!…true` chain (non-`Call` shape) wrapped in a minimal program.
fn deep_unary_program(depth: usize) -> Program {
    let span = Span::default();
    let mut value = Expr::Literal(Literal::Bool(true, span));
    for _ in 0..depth {
        value = Expr::Unary {
            op: UnaryOp::Not,
            operand: Box::new(value),
            span,
        };
    }
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

/// The guarded `check_expr` recursion at `MAX_EXPR_DEPTH` must not overflow a
/// 1 MiB stack. (The walk returns the depth-limit error; we only assert it
/// returns at all — an overflow would abort the process instead.)
#[test]
fn checker_depth_fits_windows_main_thread_stack() {
    let handle = std::thread::Builder::new()
        .stack_size(WINDOWS_MAIN_THREAD_STACK)
        .spawn(|| {
            let program = deep_unary_program(MAX_EXPR_DEPTH);
            let _ = my_lang::check(&program);
            my_lang::ast::drop_program_iteratively(program);
        })
        .expect("spawn probe thread");
    handle
        .join()
        .expect("checker walk at MAX_EXPR_DEPTH overflowed the 1 MiB Windows stack budget");
}

/// The general iterative teardown (subtlety 1's fix) must tear down an AST far
/// deeper than any recursive `Drop` could survive, on the same 1 MiB budget —
/// proving the overflow vector is structurally gone, not just bounded by the
/// guard.
#[test]
fn iterative_teardown_survives_far_past_recursive_cliff() {
    let handle = std::thread::Builder::new()
        .stack_size(WINDOWS_MAIN_THREAD_STACK)
        .spawn(|| {
            // ~1e6 levels: orders of magnitude past where auto-derived recursive
            // Drop cliffs (a few thousand levels on this budget).
            let program = deep_unary_program(1_000_000);
            my_lang::ast::drop_program_iteratively(program);
        })
        .expect("spawn teardown thread");
    handle
        .join()
        .expect("iterative teardown overflowed — #37 subtlety 1 regressed");
}
