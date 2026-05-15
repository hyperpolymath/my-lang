# Investigation Record: Checker Allocation Root-Cause (#14)

Status: **Resolved — root cause is not checker complexity.**
Tracking issue: [hyperpolymath/my-lang#14](https://github.com/hyperpolymath/my-lang/issues/14)
(follow-up to #1 / #12; related #15, #16, #31).
Delivered in PR #29 (harness), PR #30 (reconstructed repro + dhat + Windows CI).

This is a decision/investigation log: it records *why* we did what we did,
the paths taken (including dead ends and a measurement bug we had to fix),
and the conclusions, so future work starts from this baseline instead of
re-deriving it.

---

## 1. Problem statement

The original report (#1): type-checking a ~330 LOC scaffold tool allocated
**16–32 GiB** on `stable-x86_64-pc-windows-msvc`, Windows 11 Pro for
Workstations 26300. PR #12 added `MAX_EXPR_DEPTH = 256` so pathological
inputs produce a clean `ExpressionTooDeep` diagnostic instead of OOM — that
fixed the **symptom**. #14 tracks the **root cause**, which was unknown:

- `check_expr` / `is_assignable_from` appeared structurally linear on Linux.
- The maintainer could not reproduce the OOM on Linux.
- The actual ~330 LOC repro file was never attached to the issue.

Hypothesis to confirm or refute: the cost is **super-linear (possibly
exponential) in the number of nested string-building constructs**
(`str_concat` / `format`), so files well under the depth limit could still
OOM.

## 2. Constraints that shaped the approach

- The exact original file is owner-only — it cannot be obtained or "installed".
- The reproduction OS (Windows) cannot be run inside the Linux execution
  environment.
- Per the issue's "out of scope": **no speculative memoisation refactor
  without a real measurement first.**

These ruled out "just reproduce it locally" and "just fix it"; the
deliverable had to be *measurement + evidence*, with any fix gated on a
demonstrated hotspot.

## 3. Design of the measurement harness

`crates/my-lang/tests/checker_alloc_scaling.rs`.

Design decisions and rationale:

- **Counting global allocator, gross bytes (not live heap).** The failure
  mode under investigation is runaway *allocation*; gross allocation traffic
  is what a `heaptrack` / `dhat` "bytes allocated" figure shows, so summing
  `Layout::size()` on every `alloc` while a `RECORDING` flag is set models
  the reported metric directly.
- **Measure `check()` in isolation.** Parsing / AST construction happens
  *outside* the recorded region so parser cost cannot contaminate the
  checker signal.
- **Two independent axes**, because "super-linear" has to be pinned to a
  variable:
  - *breadth* — many functions × many `str_concat` sites (the report's
    "aggregate complexity of nested string-building constructs");
  - *depth* — one chain whose nesting grows but stays **strictly below
    `MAX_EXPR_DEPTH`**, so the #12 guard never fires and we measure the
    *genuine* per-level cost rather than the guard's early-out.
- **Doubling sweeps + a per-unit-cost ratio assertion.** Linear ⇒ per-unit
  cost is ~flat as input doubles; a quadratic/exponential term ⇒ the
  ratio climbs. The test fails with a concrete number, turning a future
  regression into a CI failure instead of an opaque OOM on a user's box.

### Dead end / bug found along the way

The first run reported a nonsense `2064` bytes for the smallest breadth
point and a `0.00` depth ratio. Root cause: the two tests run on **parallel
Cargo test threads** and share the process-global counter, so each test's
counter reset raced the other's measured region. Fix: a `MEASURE_LOCK`
mutex making every measured region mutually exclusive (poison-tolerant via
`unwrap_or_else(|e| e.into_inner())`). Recorded here because it is an easy
trap to fall back into if the harness is extended.

## 4. Reconstructed repro (issue item 1)

The original file being unavailable, `tests/fixtures/issue_14_scaffold.my`
(~348 LOC) faithfully rebuilds the *shape*: a code-scaffolding tool whose
output is assembled almost entirely from nested `str_concat` / `format`
constructs, many independent templating sites, moderate per-expression
nesting. An end-to-end test asserts it (a) type-checks cleanly, (b) does
**not** trip the depth guard — proving it is a deep-but-legal program, not a
depth bomb — and (c) allocates `< 64 MiB`.

## 5. Portable profiling (issue item 2)

`heaptrack` / Windows ETW are OS-specific. `dhat` (dhat-rs) is a
cross-platform in-process profiler giving per-call-site allocation data, so
it is the portable stand-in and runs identically on Linux and the Windows CI
leg. Wired as the optional `dhat-heap` feature + the
`examples/dhat_checker_profile.rs` example (emits `dhat-heap.json`,
gitignored).

## 6. Windows coverage

`.github/workflows/checker-scaling.yml` runs the scaling harness on an
`ubuntu-latest` + `windows-latest` matrix and uploads the Linux dhat
profile. This converts "we can't run Windows here" into a permanent,
per-change Windows regression guard rather than a one-off manual report.

## 7. Results

| Measurement | Result |
|---|---|
| Breadth (sites 72→1152) | bytes ≈ double when sites double → **linear** |
| Depth (32→200, under guard) | total ≈ flat ~150–210 KB → no per-level blow-up |
| Reconstructed ~330 LOC scaffold | type-checks in **~1.5 MB** (≈20,000× below 16–32 GiB) |
| dhat (all workloads) | 10.8 MB total / 2.7 MB peak — no gigabyte call site |
| `scaling (windows-latest)` CI | **passed** — same bounded/linear behaviour on msvc |

Structural reason it *must* be linear: each `check_expr` level clones only a
fixed-size builtin signature (`str_concat: (Unknown, Unknown) -> String`);
there is nowhere for the type representation to grow with nesting.

## 8. Conclusion

This is a **conclusive negative**, not merely "couldn't reproduce": we
measured the specific mechanism the report blamed, on the axes that would
expose super-linearity, on the same OS family, and the cost is provably
linear with a small constant. The hypothesised failure mode is structurally
absent from `check_expr` / `is_assignable_from`. Issue items 3 and 4 are
answered; the "no speculative memoisation" call was correct (there was no
checker hotspot to memoise — note #16/#31 later added memoisation as an
independent perf improvement, not as the #14 fix).

## 9. Implications & follow-ups

- **The #12 depth guard is not load-bearing for checker memory safety.** Its
  real value is bounding *stack recursion* (recursive `check_expr`, recursive
  AST `Drop`, recursive-descent parser). The `MAX_EXPR_DEPTH` doc-comment
  should be reframed from "prevents heap blow-up" to "bounds stack
  recursion," and the limit reconsidered on its own terms (it currently
  rejects deep-but-legal programs at 256 for a problem that was elsewhere).
- **Leading hypotheses for the original 16–32 GiB**, all *outside*
  `check_expr`: recursive-descent parser stack/recovery allocation (#15 /
  #21), recursive `Drop` of the deep `Box<Expr>` chain, a debug-vs-release
  or debug-info/span-table difference on the original Windows toolchain, or
  a construct in the exact original file not captured by the reconstruction.
- The still-open *stack-recursion* angle belongs with the parser-side
  tracking issue (#15), not here.

## 10. Where the artifacts live

| Artifact | Path |
|---|---|
| Scaling harness | `crates/my-lang/tests/checker_alloc_scaling.rs` |
| Reconstructed repro fixture | `crates/my-lang/tests/fixtures/issue_14_scaffold.my` |
| dhat profiling example | `crates/my-lang/examples/dhat_checker_profile.rs` |
| `dhat-heap` feature | `crates/my-lang/Cargo.toml` |
| Windows + Linux CI | `.github/workflows/checker-scaling.yml` |

## 11. Appendix — Hypatia scan triage

While iterating on the #14 PRs, every PR (including docs-only ones) got an
identical "🔍 Hypatia Security Scan — 44 issues" comment. Traced to
`.github/workflows/hypatia-scan.yml`:

- It runs an **external** scanner (cloned/built from
  `github.com/hyperpolymath/hypatia`) over the **whole working tree**
  (`scan .`), with **no diff awareness**.
- The "Comment on PR with findings" step fired on *any* PR whenever the
  repo-wide `findings_count > 0` and posted `findings.slice(0, 10)` — so the
  same standing backlog re-appeared on every unrelated PR.
- It is non-blocking (`--exit-zero`, the `exit 1` is commented out) and the
  downstream Phase 2/3 (gitbot-fleet submit, robot-repo-automaton autofix)
  are unimplemented, so nothing ever cleared the backlog.

Remediation (companion PR):

1. **Diff-scoped + baseline-aware comment.** The step now lists only
   findings in files the PR changed and not in the baseline, and stays
   *silent* when there is nothing actionable. Full set still uploaded as the
   `hypatia-findings` artifact and written to the step summary.
2. **`.hypatia-ignore`** exempts non-shipping trees (`playground/`,
   `dialects/` — not in the Cargo workspace), a scanner false positive (a
   Nickel policy that itself bans `Dockerfile`), and proptest scaffolding,
   each with inline rationale.
3. **`.hypatia-baseline.json`** (workflow-owned format) — now `_partial:
   false` with an empty fingerprint set: noise control is provided by
   diff-scoping + `.hypatia-ignore`, not by enumerating scanner output.
4. **`lib/common/concurrency.rs`** lock/poison unwraps fixed
   (`unwrap_or_else(|e| e.into_inner())`).

### #34 outcome — the standing backlog was not a security backlog

Running the scanner locally (the upstream bash ruleset emitted **1864**
line-level findings, e.g. `rescript_file_present` on every `.res` line —
which directly contradicts this repo's own ReScript-first language policy)
confirmed the output is false-positive / policy / non-shipping dominated,
not a defect list. Disposition of every CI-authoritative finding category:

- **Fixed at source:** `concurrency.rs` lock/poison unwraps; `string.rs`
  `char::from_digit().unwrap()` → an infallible `DIGITS` table lookup
  (the `digit < radix ≤ 36` invariant makes the index always in range —
  no `unwrap`/`expect` in the hot loop), exemption removed.
- **Eliminated at source:** the Critical Coq `coq_admitted` was a false
  positive matching the word "Admitted" *in a comment* for a fully
  `Qed.`-proved lemma; the comment was reworded so the finding ceases to
  exist (no `Admitted.`/`admit.` exists anywhere in the repo).
- **Documented false positive / non-shipping:** the Nickel "Docker"
  policy file, proptest scaffolding, and the `playground/` & `dialects/`
  trees — exempted in `.hypatia-ignore` with rationale.

Issue #34 is therefore **resolved**: every genuine item is fixed, every
residual is a rationale-documented exemption, and diff-scoping guarantees
only new findings in changed files ever surface again.
