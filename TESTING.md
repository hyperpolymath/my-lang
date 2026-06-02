<!-- SPDX-License-Identifier: MPL-2.0 -->
<!-- Copyright (c) 2026 Jonathan D.A. Jewell (hyperpolymath) <j.d.a.jewell@open.ac.uk> -->

# Testing & Coverage Roadmap

This document records the current state of automated testing in the `my-lang`
workspace and the prioritised plan for improving it. It is a living document:
update it as gaps are closed.

## How to run the tests

```sh
# Whole workspace
cargo test --workspace

# Just the compiler crate
cargo test -p my-lang

# Conformance + example fixtures (parse-level)
cargo test -p my-lang --test conformance

# Coverage (requires llvm-tools-preview + cargo-llvm-cov)
rustup component add llvm-tools-preview
cargo install cargo-llvm-cov --locked
cargo llvm-cov --workspace --exclude my-llvm --summary-only
```

CI runs coverage on every push/PR via `.github/workflows/coverage.yml` and
enforces a conservative line-coverage **floor** (`COVERAGE_FLOOR`). The floor is
a ratchet: raise it as coverage improves; never lower it.

### Baseline (at introduction)

Whole-workspace coverage (excluding `my-llvm`, which needs a system LLVM
toolchain) was **~46.7% lines / 45.3% regions / 48.0% functions**. Notable
per-area gaps:

| Area | Line coverage |
|------|--------------:|
| `my-lang/src/lexer.rs` | ~95% |
| `my-lang/src/parser.rs` | ~79% |
| `my-lang/src/checker.rs` | ~61% |
| `my-lang/src/interpreter.rs` | ~58% |
| `my-lang/src/stdlib.rs` | ~42% |
| `my-lang/src/types.rs` | ~31% |
| `my-lang/src/token.rs` | ~21% |
| `my-mir/src/lib.rs` | ~21% |
| `my-pkg/src/lib.rs` | ~10% |
| `my-lsp/src/lib.rs` | ~16% |
| `my-lang/src/main.rs`, `src/visitor.rs`, `my-test`, `my-lsp/main.rs` | 0% |

## Current state (snapshot)

The active build is the `crates/*` Cargo workspace. Test density is very uneven:
the compiler front end (lexer/parser/checker/interpreter) is reasonably covered,
while the middle/back end, the CLI, and most support binaries are barely tested.

| Crate | ~LOC | Inline tests | Integration files | Status |
|-------|-----:|-------------:|------------------:|--------|
| `my-lang` (compiler) | 14,156 | 140+ | 3 | Front end good; `stdlib.rs`, `ast.rs`, `types.rs`, `token.rs`, `visitor.rs` have **no** unit tests |
| `my-cli` | 764 | 0 | 0 | The `my` binary — **untested** |
| `my-mir` | 1,596 | 1 | 0 | MIR lowering nearly untested |
| `my-llvm` | 1,077 | 1 | 0 | Codegen nearly untested |
| `my-lsp` | 648 | 1 | 0 | Language server nearly untested |
| `my-ai` | 778 | 5 | 0 | Light; no async end-to-end tests |
| `my-hir` | 666 | 1 | 0 | Thin |
| `my-fmt` | 529 | 2 | 0 | Thin |
| `my-lint` | 444 | 1 | 0 | Thin |
| `my-pkg` | 377 | 1 | 0 | Thin |
| `my-test` | 344 | 0 | 0 | **Untested** |
| `my-debug` | 266 | 0 | 0 | **Untested** |
| `my-dap` | 173 | 0 | 0 | **Untested** |
| `my-parser` | 41 | 1 | 1 | Stub (all methods `Ok(())` / TODO) |

### Structural issues found

1. **Partly-orphaned duplicate source tree.** The repository root contains a
   second `src/`, `lib/`, `tests/`, and `fuzz/` tree alongside the active
   `crates/*` workspace. The picture is mixed:
   - `tests/integration_test.rs` (20 tests) **is** run — `crates/my-lang/Cargo.toml`
     wires it in via `[[test]] path = "../../tests/integration_test.rs"`.
   - `tests/property_tests.rs` (34 tests) is **not referenced anywhere** and
     never runs.
   - The root `src/` and `lib/` are **not referenced** by any crate; the root
     `src/lib.rs` also references modules that do not exist there, so it cannot
     compile as a package.
   → **Action:** wire in (or port) `property_tests.rs`, and delete the orphaned
   root `src/`/`lib/` duplicates so the layout has a single source of truth.

2. **Conformance suite was shell-only and broken.** `conformance/run_conformance.sh`
   drove `my-cli` through a `--parse-only` flag that no longer exists (the CLI
   uses subcommands such as `parse`/`check`). It was therefore easy to skip and
   silently non-functional. → **Done:** added `crates/my-lang/tests/conformance.rs`,
   which exercises every fixture against the library API under `cargo test` (and
   thus under coverage). See "Known parse gaps" below.

3. **No coverage measurement.** → **Done:** added `.github/workflows/coverage.yml`
   (`cargo-llvm-cov`) with an LCOV + HTML artifact and a ratcheting floor.

4. **Several crates did not compile.** Enabling a workspace-wide build revealed
   that `my-fmt`, `my-lsp`, and `my-lint` did not compile at all (a borrow-after-move,
   a non-exhaustive `match` missing the `ExpressionTooDeep` checker variant, and
   invalid multi-codepoint `char` literals, respectively). This strongly implies
   current CI only builds `my-lang`. → **Done:** all three are fixed in this
   change, and the new coverage job now builds and tests the whole workspace, so
   such breakage cannot recur silently. `my-lint`'s only unit test had also never
   run and asserted the wrong count; it now matches actual behaviour.

5. **Thin negative-path testing.** Outside `checker.rs`, very few tests assert
   on *which* error/span is produced. The `conformance/invalid/*.my` fixtures
   now assert rejection, but unit tests should also pin specific
   `ParseError`/`CheckError` variants.

## Known parse gaps (surfaced by the new conformance test)

Wiring up the conformance fixtures immediately found real defects/feature gaps.
One was a genuine bug and is **fixed**; the rest are tracked as a fail-closed
allowlist in `crates/my-lang/tests/conformance.rs` (`KNOWN_PARSE_GAPS`). When a
gap is closed, the test forces removal of its allowlist entry.

| Fixture | Gap | Status |
|---------|-----|--------|
| `v03_ai_model.my` | `ai_model` blocks rejected comma-separated / trailing-comma attributes (inconsistent with `struct`) | **Fixed** in `parser.rs` (`parse_ai_model_attr`) + regression test |
| `v04_let.my` | assignment statements (`y = y + x;`) not parsed | Tracked (allowlisted) |
| `v06_match.my` | `match` not accepted as a function tail expression | Tracked (allowlisted) |
| `v07_agent.my` | unit type `()` in effect op signatures not parsed | Tracked (allowlisted) |
| `v08_effect.my` | lambda syntax (`\|n\| => expr`) not parsed | Tracked (allowlisted) |

## Prioritised improvement plan

1. **Resolve the duplicate tree** (item 1 above). Highest leverage — until then
   coverage numbers are misleading.
2. **Close the known parse gaps**, removing each `KNOWN_PARSE_GAPS` entry as its
   feature lands (assignment statements, match tail expressions, unit type in
   effect ops, lambdas).
3. **Test the middle/back end:** golden tests for `my-mir` lowering and `my-llvm`
   codegen (snapshot IR for small programs).
4. **Test `my-cli`:** drive the built `my` binary (or `assert_cmd`) for
   argument parsing, exit codes, and each subcommand (`parse`, `check`, `lex`,
   `run`).
5. **Cover `stdlib.rs`** — the largest untested unit (~1,900 LOC) and likely
   high-churn (JSON, maps, strings, fs, dates).
6. **Add async tests for the AI paths** (`my-ai`, `library::mylang::*`) using the
   existing `MockAiClient` to exercise streaming and tool-call flows end-to-end.
7. **Strengthen negative-path unit tests** — assert specific error variants and
   spans, especially for the `conformance/invalid` cases.
8. **Cover the remaining untested binaries:** `my-test`, `my-debug`, `my-dap`.
9. **Ratchet `COVERAGE_FLOOR`** upward as the above lands.
