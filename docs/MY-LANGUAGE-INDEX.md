<!--
SPDX-License-Identifier: MPL-2.0
Copyright (c) Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk>
-->
= My Language Master Index & Roadmap

*Source material: October 26, 2025 master index (My-Newsroom project summary).* This document anchors that vision inside the repo and links each highlighted piece to current artifacts.

== Dialect Completion Matrix

| Dialect | Purpose | Key Status | Artifacts (path) |
|---------|---------|------------|------------------|
| Me | Epistemic types, belief states | ✅ Complete | `my-newsroom/examples/me/playground.html`, `docs/dialects/me.md` |
| Solo | Systems, affine arena programming | ⚠️ Parser/Backend pending | `home/.../My grammar (solo).enbf`, `docs/reference/grammar.md`, `crates/my-lang/src/parser.rs`, `my-newsroom/solo-compiler/src` |
| Duet | Human-AI synthesis (intent/@synth) | ⚠️ Spec only | `my-newsroom/docs/dialects/duet.md`, `docs/theory/duet_ensemble_grammar.md` |
| Ensemble | Agentic orchestration + fusion | ⚠️ Spec only | `my-newsroom/docs/dialects/ensemble.md`, `docs/theory/duet_ensemble_grammar.md` |

== Toolchain Dependencies

* **Parser + AST**: `crates/my-lang/src/parser.rs`, Solo grammar + hive docs. Completion is critical for Solo → Duet pipeline (Menhir/OCaml parser is stub).
* **Type Checker**: `src/checker.rs` integrates affine/effect/contract metadata; AI directives reuse existing `Contract` nodes.
* **Backend**: QBE recommended (`my-newsroom/solo-compiler/src/main.rs`), with future LLVM option.
* **Duet/Ensemble runtime**: Duet adds AI metadata; Ensemble treats `comptime orchestrate` specs as configuration loaded by Rust/Elixir orchestrator. The Julia Dempster-Shafer core (`my-newsroom/src/dempster_shafer.jl`) provides belief fusion logic.
* **Newroom** proof-of-concept: `docs/NEWROOM-ROADMAP.md`, `examples/julia/*`, `src/dempster_shafer.jl`, `docs/research/*`.

== Next Immediate Focus (per repo status)

1. **Solo parser & backend (weeks 1-3)**
   - Finish Menhir parser using Solo grammar (target per master index). Reference: `docs/reference/grammar.md`, `crates/my-lang/src/parser.rs`.
   - Implement QBE backend (Rust) in `my-newsroom/solo-compiler/src` and connect to the interpreter runtime.
   - Add runtime library (arena allocation, GC hooks) in Rust.
2. **Duet specification & examples**
   - Derive Duet grammar (see `docs/theory/duet_ensemble_grammar.md`).
   - Capture `intent`, `@synth`, `#[ai_*]` constructs in parser/AST.
   - Draft verification/linting guidance per roadmap.
3. **Ensemble orchestration & Newroom**
   - Build `comptime orchestrate` interpreter that materializes agent ring + fusion.
   - Hook into `my-newsroom/src/dempster_shafer.jl` for belief fusion.
   - Prepare Newroom demos (agent definitions in `docs/dialects/ensemble.md`, examples under `examples/`).

== Historical Context

- The research conversation names (cited in `README.adoc` and `docs/README`) are the same as the handover. All theory proofs live in `proofs/shared` and are now echoed under `docs/theory/` for archival safety.
- This roadmap document links repo files to the statuses and dependencies outlined in the original master index, so nothing ever disappears from the recorded history.
