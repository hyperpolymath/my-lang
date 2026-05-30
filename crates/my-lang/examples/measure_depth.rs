// SPDX-License-Identifier: MPL-2.0
//! Stack-budget probe & self-driving measurement for hyperpolymath/my-lang#37.
//!
//! Issue #37 needs the per-recursion-level stack cost of the checker walk and
//! of AST teardown *measured on each target platform* (Linux + the Windows CI
//! toolchain) so that `MAX_EXPR_DEPTH` is justified by a real budget rather
//! than the disproven OOM argument, and so the recursive-`Drop` overflow
//! (subtlety 1) provably stays fixed.
//!
//! A stack overflow aborts the whole process, so it cannot be observed in
//! process. This example therefore has two roles, selected by argv:
//!
//!   * **worker** — `measure_depth <check|drop_recursive|drop_iter> <depth> <stack_kib>`
//!     runs exactly one recursive walk on a thread with the given stack size,
//!     printing an `OK …` line and exiting `0` on survival. A stack overflow
//!     aborts it with a non-zero status; that status is the measurement signal.
//!
//!   * **driver** — no args (or `report`) re-executes itself as worker
//!     subprocesses, binary-searching the overflow cliff for each walk, prints
//!     the measured bytes/level, and **asserts** that the live `MAX_EXPR_DEPTH`
//!     fits inside the smallest target main-thread stack (Windows = 1 MiB) with
//!     headroom. It exits non-zero if the budget is unsafe, so it doubles as a
//!     one-command CI guard that *produces* the per-platform datapoint and
//!     *locks it in* against regressions.
//!
//! Usage:
//!   cargo run --release -p my-lang --example measure_depth            # driver
//!   cargo run --release -p my-lang --example measure_depth -- check 128 1024

use my_lang::ast::*;
use my_lang::checker::MAX_EXPR_DEPTH;
use my_lang::token::Span;
use std::process::Command;

/// Smallest main-thread stack across our target platforms. Windows defaults the
/// *process main thread* to 1 MiB (Linux is typically 8 MiB). The checker and
/// the AST `Drop` run on whatever thread the embedder calls them on, so 1 MiB is
/// the binding budget that `MAX_EXPR_DEPTH` must respect.
const MIN_MAIN_THREAD_STACK_KIB: usize = 1024;

/// The depth-`MAX_EXPR_DEPTH` checker walk must fit within this fraction of the
/// 1 MiB floor, leaving headroom for the embedder's own frames above the walk.
const SAFETY_NUMER: usize = 4;
const SAFETY_DENOM: usize = 5; // 80 % — comfortably above the ~55% a depth-128
                               // walk uses, well below the ~110% a 256 walk would.

/// Small reference stack the recursive-`Drop` cliff is measured against. Kept
/// well under 1 MiB so the cliff (and the iterative fix surviving past it) is
/// reached quickly without huge allocations.
const DROP_REF_KIB: usize = 256;

/// Build a left-nested `!!!…true` chain of the given depth. `Unary` (not `Call`)
/// is used so the walk exercises the simple single-child recursive edge — the
/// cleanest per-level cost signal, and the exact shape the old `Call`-only test
/// helper could not tear down.
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

/// Wrap an expression in a minimal well-formed `Program` (`fn main() { let s = … }`).
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

/// One recursive walk on a thread with `stack_kib` of stack. Returns normally on
/// survival; a stack overflow aborts the process (non-zero exit), which is the
/// signal [`survives`] reads.
fn worker(mode: &str, depth: usize, stack_kib: usize) {
    let mode_owned = mode.to_string();
    let handle = std::thread::Builder::new()
        .stack_size(stack_kib * 1024)
        .spawn(move || {
            let program = wrap(build_unary_chain(depth));
            match mode_owned.as_str() {
                // Guarded checker recursion — the walk MAX_EXPR_DEPTH bounds.
                "check" => {
                    let _ = my_lang::check(&program);
                    drop_program_iteratively(program); // iterative teardown, never overflows
                }
                // The unguarded danger (subtlety 1): auto-derived recursive Drop.
                "drop_recursive" => drop(program),
                // The #37 fix: the general iterative teardown must never overflow.
                "drop_iter" => drop_program_iteratively(program),
                other => {
                    eprintln!("unknown mode: {other}");
                    std::process::exit(2);
                }
            }
            // Only reached if the walk did not overflow.
            println!("OK mode={mode_owned} depth={depth} stack_kib={stack_kib}");
        })
        .expect("spawn probe thread");

    // Overflow already aborted the process; a non-overflow panic → non-zero exit.
    if handle.join().is_err() {
        std::process::exit(1);
    }
}

/// Run `self <mode> <depth> <stack_kib>` as a subprocess; `true` == survived.
fn survives(exe: &std::path::Path, mode: &str, depth: usize, stack_kib: usize) -> bool {
    Command::new(exe)
        .arg(mode)
        .arg(depth.to_string())
        .arg(stack_kib.to_string())
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Largest `depth` that survives `mode` on a fixed `stack_kib`: exponential
/// probe to bracket the cliff, then binary search. `ceiling` caps both the
/// search and the allocation. Returns 0 if even depth 1 overflows, or `ceiling`
/// if it never cliffs within the cap.
fn max_surviving_depth(exe: &std::path::Path, mode: &str, stack_kib: usize, ceiling: usize) -> usize {
    if !survives(exe, mode, 1, stack_kib) {
        return 0;
    }
    let (mut lo, mut hi) = (1usize, 2usize);
    while hi < ceiling && survives(exe, mode, hi, stack_kib) {
        lo = hi;
        hi = (hi * 2).min(ceiling);
    }
    if survives(exe, mode, hi, stack_kib) {
        return hi; // never cliffed within the ceiling
    }
    while hi - lo > 1 {
        let mid = lo + (hi - lo) / 2;
        if survives(exe, mode, mid, stack_kib) {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    lo
}

/// Smallest `stack_kib` in `[1, ceiling_kib]` on which `mode` survives at the
/// fixed `depth`. Returns `ceiling_kib + 1` (sentinel) if it never fits.
fn min_surviving_stack(exe: &std::path::Path, mode: &str, depth: usize, ceiling_kib: usize) -> usize {
    if survives(exe, mode, depth, 1) {
        return 1;
    }
    let (mut lo, mut hi) = (1usize, 2usize); // lo known-too-small
    while hi < ceiling_kib && !survives(exe, mode, depth, hi) {
        lo = hi;
        hi = (hi * 2).min(ceiling_kib);
    }
    if !survives(exe, mode, depth, hi) {
        return ceiling_kib + 1;
    }
    while hi - lo > 1 {
        let mid = lo + (hi - lo) / 2;
        if survives(exe, mode, depth, mid) {
            hi = mid;
        } else {
            lo = mid;
        }
    }
    hi
}

/// The driver: measure, report, and assert the budget. Returns the process exit code.
fn driver() -> i32 {
    let exe = std::env::current_exe().expect("current_exe");
    let os = std::env::consts::OS;
    println!("== my-lang #37 stack-depth measurement ==");
    println!("platform           : {os} / {}", std::env::consts::ARCH);
    println!("MAX_EXPR_DEPTH      : {MAX_EXPR_DEPTH} (live)");
    println!("main-thread floor   : {MIN_MAIN_THREAD_STACK_KIB} KiB (Windows)\n");

    // 1) Recursive-`Drop` cliff on a small stack → bytes/level for the Drop walk.
    let drop_cliff = max_surviving_depth(&exe, "drop_recursive", DROP_REF_KIB, 2_000_000);
    let drop_bpl = if drop_cliff > 0 {
        (DROP_REF_KIB * 1024) / drop_cliff
    } else {
        0
    };
    println!("recursive Drop      : cliff ~{drop_cliff} levels @ {DROP_REF_KIB} KiB  (~{drop_bpl} B/level)");

    // 2) Checker cost: smallest stack a MAX_EXPR_DEPTH-deep guarded walk fits in.
    let chk_min_kib = min_surviving_stack(&exe, "check", MAX_EXPR_DEPTH, MIN_MAIN_THREAD_STACK_KIB * 8);
    let chk_bpl = (chk_min_kib * 1024) / MAX_EXPR_DEPTH.max(1);
    println!("checker @ depth {MAX_EXPR_DEPTH} : fits in >= {chk_min_kib} KiB  (~{chk_bpl} B/level)");

    // 3) The fix must survive far past where recursive Drop cliffs.
    let iter_target = drop_cliff.saturating_mul(8).max(1_000_000);
    let iter_ok = survives(&exe, "drop_iter", iter_target, DROP_REF_KIB);
    println!("iterative teardown  : survives {iter_target} levels @ {DROP_REF_KIB} KiB: {iter_ok}\n");

    // ---- assertions (these are what make this a CI guard) ----
    let budget_kib = MIN_MAIN_THREAD_STACK_KIB * SAFETY_NUMER / SAFETY_DENOM;
    let mut ok = true;

    if chk_min_kib > budget_kib {
        eprintln!(
            "FAIL: depth-{MAX_EXPR_DEPTH} checker needs >= {chk_min_kib} KiB but the budget is \
             {budget_kib} KiB ({SAFETY_NUMER}/{SAFETY_DENOM} of {MIN_MAIN_THREAD_STACK_KIB} KiB). \
             Lower MAX_EXPR_DEPTH."
        );
        ok = false;
    }
    if !iter_ok {
        eprintln!(
            "FAIL: iterative teardown overflowed at {iter_target} levels where recursive Drop \
             cliffs at ~{drop_cliff} — subtlety 1 (the #37 fix) has regressed."
        );
        ok = false;
    }
    if drop_cliff != 0 && drop_cliff < MAX_EXPR_DEPTH {
        println!(
            "note: recursive Drop cliffs ({drop_cliff}) below MAX_EXPR_DEPTH ({MAX_EXPR_DEPTH}); \
             teardown MUST stay iterative (it is)."
        );
    }

    if ok {
        println!(
            "PASS: MAX_EXPR_DEPTH={MAX_EXPR_DEPTH} is safe within {budget_kib} KiB of the \
             {MIN_MAIN_THREAD_STACK_KIB} KiB floor on {os}."
        );
        0
    } else {
        1
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    match args.get(1).map(String::as_str) {
        None | Some("report") | Some("measure") | Some("auto") => std::process::exit(driver()),
        Some(mode) => {
            let depth = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(1000);
            let stack_kib = args.get(3).and_then(|s| s.parse().ok()).unwrap_or(1024);
            worker(mode, depth, stack_kib);
        }
    }
}
