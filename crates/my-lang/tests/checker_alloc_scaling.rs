// SPDX-License-Identifier: MPL-2.0
// Copyright (c) Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk>
//! Allocation-scaling harness for the type checker.
//!
//! Investigation support for hyperpolymath/my-lang#14 (follow-up to #1 / #12).
//!
//! The original report observed a 16-32 GiB allocation while type-checking a
//! ~330 LOC scaffold tool on `stable-x86_64-pc-windows-msvc`. The maintainer
//! could not reproduce a super-linear allocation on Linux and suspected the
//! checker's `check_expr` / `is_assignable_from` are structurally linear in
//! AST size. The original ~330 LOC repro file was never attached, so this
//! harness reconstructs the *shape* the issue describes and measures how the
//! checker's heap traffic actually scales:
//!
//!   * "aggregate complexity of nested string-building constructs" -> the
//!     `breadth` axis: many functions, each with many `str_concat` sites and
//!     a moderately nested `str_concat` chain.
//!   * raw nesting -> the `depth` axis: a single `str_concat` chain whose
//!     depth grows but stays strictly below `MAX_EXPR_DEPTH` (so the depth
//!     guard from #12 never fires and we measure the *real* cost).
//!
//! A counting global allocator records bytes requested during `check()` only
//! (parsing / AST construction happens outside the measured region). If the
//! checker were exponential / super-linear in the number of templating sites
//! -- as the original report hypothesised -- the per-unit byte cost would
//! climb as the input doubles. These tests assert it stays bounded, so they
//! double as a regression guard: a future change that reintroduces
//! super-linear checking will fail here with a concrete measurement instead
//! of an opaque OOM on someone's Windows box.
//!
//! Run with `cargo test -p my-lang --test checker_alloc_scaling -- --nocapture`
//! to see the per-size byte report.

use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Mutex;

use my_lang::checker::{CheckError, MAX_EXPR_DEPTH};
use my_lang::{
    check, parse, Block, Expr, FnDecl, Ident, Literal, Program, Span, Stmt, TopLevel,
};

/// Faithful reconstruction of the issue's ~330 LOC string-building scaffold
/// tool (the original was never attached to #14). See the file header.
const ISSUE_14_SCAFFOLD: &str = include_str!("fixtures/issue_14_scaffold.my");

/// Global allocator that tallies total bytes requested while `RECORDING` is
/// on. We sum allocation sizes (gross traffic) rather than tracking live heap:
/// the failure mode under investigation is runaway *allocation*, and gross
/// traffic is what a `heaptrack` / `dhat` "bytes allocated" figure would show.
struct CountingAlloc;

static RECORDING: AtomicBool = AtomicBool::new(false);
static BYTES: AtomicUsize = AtomicUsize::new(0);

unsafe impl GlobalAlloc for CountingAlloc {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        if RECORDING.load(Ordering::Relaxed) {
            BYTES.fetch_add(layout.size(), Ordering::Relaxed);
        }
        System.alloc(layout)
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        System.dealloc(ptr, layout)
    }
}

#[global_allocator]
static GLOBAL: CountingAlloc = CountingAlloc;

/// `RECORDING` / `BYTES` are process-global, but Cargo runs the tests in this
/// file on parallel threads. Without serialisation each test's measured
/// `check()` races the other's counter resets and produces nonsense (a
/// near-zero reading or a wildly low first sample). This lock makes every
/// measured region mutually exclusive so the numbers are trustworthy.
static MEASURE_LOCK: Mutex<()> = Mutex::new(());

/// Measure bytes allocated by `check(program)` in isolation.
fn bytes_to_check(program: &Program) -> usize {
    let _guard = MEASURE_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    BYTES.store(0, Ordering::Relaxed);
    RECORDING.store(true, Ordering::Relaxed);
    let _ = check(program);
    RECORDING.store(false, Ordering::Relaxed);
    BYTES.load(Ordering::Relaxed)
}

/// One function with `sites` independent `str_concat` lets plus one
/// `chain_depth`-deep `str_concat` chain. This mirrors the issue's
/// "nested string-building constructs" shape.
fn gen_function(idx: usize, sites: usize, chain_depth: usize) -> String {
    let mut src = format!("fn f{idx}() {{\n");
    for s in 0..sites {
        src.push_str(&format!(
            "    let v{s} = str_concat(\"a\", str_concat(\"b\", \"c\"));\n"
        ));
    }
    src.push_str("    let chained = ");
    for _ in 0..chain_depth {
        src.push_str("str_concat(\"x\", ");
    }
    src.push_str("\"end\"");
    for _ in 0..chain_depth {
        src.push(')');
    }
    src.push_str(";\n}\n");
    src
}

/// Breadth-scaled program: `funcs` functions, each with `sites` templating
/// sites and a fixed shallow chain.
fn gen_program(funcs: usize, sites: usize, chain_depth: usize) -> String {
    let mut src = String::new();
    for i in 0..funcs {
        src.push_str(&gen_function(i, sites, chain_depth));
    }
    src
}

/// The checker's heap traffic must stay (near-)linear in AST size along the
/// "many templating sites" axis the issue calls out. We double the input four
/// times; for linear behaviour the bytes-per-site figure is roughly flat. We
/// allow generous slack (constant per-run overhead, allocator rounding) but
/// fail hard if per-unit cost trends upward, which is exactly the
/// super-linear signature the original Windows report hypothesised.
#[test]
fn checker_allocation_is_linear_in_templating_breadth() {
    // (funcs, sites_per_fn) doubling pairs -> total sites = funcs * sites.
    let configs = [(8usize, 8usize), (16, 8), (32, 8), (64, 8), (128, 8)];

    let mut report = Vec::new();
    for &(funcs, sites) in &configs {
        let src = gen_program(funcs, sites, 16);
        let program = parse(&src).expect("generated program must parse");
        let total_sites = funcs * (sites + 1);
        let bytes = bytes_to_check(&program);
        let per_site = bytes as f64 / total_sites as f64;
        report.push((total_sites, bytes, per_site));
    }

    println!("\n=== checker allocation vs templating breadth (#14) ===");
    println!("{:>10}  {:>14}  {:>14}", "sites", "bytes", "bytes/site");
    for (sites, bytes, per) in &report {
        println!("{sites:>10}  {bytes:>14}  {per:>14.1}");
    }

    // Smallest vs largest per-site cost. Linear => ratio ~1. Super-linear
    // (e.g. quadratic/exponential in site count) => ratio grows with size.
    let first_per = report.first().unwrap().2;
    let last_per = report.last().unwrap().2;
    let ratio = last_per / first_per;
    println!("per-site cost ratio (largest / smallest) = {ratio:.2}\n");

    assert!(
        ratio < 4.0,
        "checker allocation is super-linear in templating breadth: \
         per-site cost grew {ratio:.2}x across a 16x input increase \
         (report: {report:?}). This is the signature the Windows OOM in \
         #1/#14 hypothesised -- investigate before relaxing this guard."
    );
}

/// Build `str_concat("x", str_concat("x", ... "end"))` `depth` levels deep
/// directly as an AST. We bypass the parser on purpose: recursive-descent
/// parsing would itself stack-overflow long before the checker runs, and the
/// issue explicitly scopes the parser-side guard out.
fn deep_chain_program(depth: usize) -> Program {
    let span = Span::default();
    let mut expr = Expr::Literal(Literal::String("end".to_string(), span));
    for _ in 0..depth {
        expr = Expr::Call {
            callee: Box::new(Expr::Ident(Ident::new("str_concat", span))),
            args: vec![Expr::Literal(Literal::String("x".to_string(), span)), expr],
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
                    value: expr,
                    span,
                }],
                span,
            },
            span,
        })],
    }
}

/// Along the pure-nesting axis (below the #12 depth guard) allocation must be
/// linear in depth: `check_expr` recurses once per level and clones only a
/// fixed-size builtin signature (`str_concat: (Unknown, Unknown) -> String`),
/// so there is no place for a super-linear term. This pins that conclusion
/// with a measurement and guards against a regression that would make
/// deep-but-legal programs (well under 256) allocate pathologically.
#[test]
fn checker_allocation_is_linear_in_chain_depth() {
    // Derived from MAX_EXPR_DEPTH (not hardcoded) so this stays strictly below
    // the #12 guard regardless of how the constant is tuned — it was
    // re-derived 256 -> 128 from a measured stack budget in
    // hyperpolymath/my-lang#37. The ladder still spans a ~6x depth range, more
    // than enough to expose any super-linear per-level term.
    let m = MAX_EXPR_DEPTH;
    let depths = [m / 8, m / 4, m / 2, (m * 3) / 4];
    assert!(*depths.last().unwrap() < MAX_EXPR_DEPTH);

    let mut report = Vec::new();
    for &d in &depths {
        let program = deep_chain_program(d);
        let bytes = bytes_to_check(&program);
        report.push((d, bytes, bytes as f64 / d as f64));
    }

    println!("\n=== checker allocation vs chain depth (#14) ===");
    println!("{:>8}  {:>14}  {:>14}", "depth", "bytes", "bytes/level");
    for (d, bytes, per) in &report {
        println!("{d:>8}  {bytes:>14}  {per:>14.1}");
    }

    let first_per = report.first().unwrap().2;
    let last_per = report.last().unwrap().2;
    let ratio = last_per / first_per;
    println!("per-level cost ratio (deepest / shallowest) = {ratio:.2}\n");

    assert!(
        ratio < 3.0,
        "checker allocation is super-linear in chain depth below the depth \
         guard: per-level cost grew {ratio:.2}x (report: {report:?}). Deep \
         but legal programs under MAX_EXPR_DEPTH would allocate \
         pathologically -- this is exactly the #14 concern."
    );
}

/// End-to-end check of the reconstructed ~330 LOC scaffold from #14. This is
/// the closest we can get to the original repro (which was never attached):
/// a legal, string-building-heavy program of the size and shape the report
/// describes. It must (a) type-check without error, (b) NOT trip the #12
/// depth guard -- proving the report's file is a deep-but-legal program, not
/// a depth bomb -- and (c) allocate a sane, bounded amount (kilobytes, not
/// gigabytes). A regression that reintroduces the OOM fails here loudly.
#[test]
fn issue_14_scaffold_checks_cheaply_without_oom() {
    let program = parse(ISSUE_14_SCAFFOLD).expect("reconstructed scaffold must parse");

    let bytes = bytes_to_check(&program);
    println!("\n=== #14 reconstructed scaffold ({} items) ===", program.items.len());
    println!("checker allocated {bytes} bytes");

    let errors = match check(&program) {
        Ok(()) => Vec::new(),
        Err(e) => e,
    };
    assert!(
        !errors
            .iter()
            .any(|e| matches!(e, CheckError::ExpressionTooDeep { .. })),
        "reconstructed scaffold tripped the depth guard -- it is meant to be \
         a deep-but-legal program, tune the fixture nesting down"
    );
    assert!(
        errors.is_empty(),
        "reconstructed scaffold should type-check cleanly, got: {errors:?}"
    );

    // The original report saw 16-32 GiB. A faithful Linux run of the same
    // shape stays in the low-MB range; 64 MiB is a wildly generous ceiling
    // that still catches any regression back toward the pathological regime.
    assert!(
        bytes < 64 * 1024 * 1024,
        "reconstructed #14 scaffold allocated {bytes} bytes (> 64 MiB) -- \
         the pathological allocation regime may have returned"
    );
}
