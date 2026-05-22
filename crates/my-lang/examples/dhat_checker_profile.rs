// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Jonathan D.A. Jewell (hyperpolymath) <j.d.a.jewell@open.ac.uk>

//! Heap-profiling harness for the type checker (hyperpolymath/my-lang#14).
//!
//! The issue asked for `heaptrack` / `dhat` / ETW localisation of the
//! allocator hotspot behind the 16-32 GiB Windows OOM. `heaptrack` and ETW
//! are OS-specific; `dhat` (the dhat-rs crate) is a cross-platform, in-process
//! profiler that works identically on this Linux container and on the Windows
//! CI job, so it is the portable way to get per-call-site allocation data.
//!
//! Run:
//!
//! ```text
//! cargo run -p my-lang --example dhat_checker_profile --features dhat-heap --release
//! ```
//!
//! It writes `dhat-heap.json` in the working directory. Open it with the DHAT
//! viewer (https://nnethercote.github.io/dh_view/dh_view.html) to see exactly
//! which `check_expr` / `is_assignable_from` call sites allocate, ranked by
//! total bytes -- the localisation step the issue requested. If a real
//! super-linear hotspot exists it shows up here as a dominant call site whose
//! cost grows with the workloads below.

use my_lang::{check, parse};

#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

const SCAFFOLD: &str = include_str!("../tests/fixtures/issue_14_scaffold.my");

/// `funcs` functions, each with `sites` independent `str_concat` lets -- the
/// "many nested string-building constructs" breadth axis from the issue.
fn breadth_src(funcs: usize, sites: usize) -> String {
    let mut src = String::new();
    for i in 0..funcs {
        src.push_str(&format!("fn g{i}() {{\n"));
        for s in 0..sites {
            src.push_str(&format!(
                "    let v{s} = str_concat(\"a\", str_concat(\"b\", \"c\"));\n"
            ));
        }
        src.push_str("}\n");
    }
    src
}

fn profile(label: &str, src: &str) {
    match parse(src) {
        Ok(program) => {
            let _ = check(&program);
            eprintln!("profiled: {label} ({} items)", program.items.len());
        }
        Err(e) => eprintln!("skipped {label}: parse error: {e:?}"),
    }
}

fn main() {
    // `_profiler` writes dhat-heap.json when it is dropped at end of main.
    let _profiler = dhat::Profiler::new_heap();

    // Same shape and rough size as the original ~330 LOC report.
    profile("issue_14_scaffold", SCAFFOLD);

    // Breadth sweep: if checking is super-linear in templating-site count,
    // the dominant call site's share climbs across these.
    profile("breadth_32x8", &breadth_src(32, 8));
    profile("breadth_64x8", &breadth_src(64, 8));
    profile("breadth_128x8", &breadth_src(128, 8));

    eprintln!("wrote dhat-heap.json -- open with the DHAT viewer to localise hotspots");
}
