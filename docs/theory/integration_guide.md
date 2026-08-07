<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->
<!-- SPDX-FileCopyrightText: 2026 Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk> -->

# Integration Guide

This mirrors the former `integration_guide.md`. It catalogs how Solo, Duet, and Ensemble components stitch together across the toolchain.

- **Solo → Parser → Checker**: The grammar and parser live in `docs/wiki/reference/grammar.md` and `crates/my-lang/src/parser.rs`; the type checker (`src/checker.rs`) consumes the AST and effect/contract metadata.
- **Duet extensions**: Described in `docs/dialects/duet.md` (in `my-newsroom`), the AI directives (`intent`, `@synth`, `#[ai_optimize]`, etc.) are layered over Solo by reusing the same parser and AST nodes while flagging contract/AI metadata.
- **Ensemble runtime**: `docs/dialects/ensemble.md` guides how comptime orchestration, agents, trust fusion (Julia core at `my-newsroom/src/dempster_shafer.jl`), and the epistemic ledger slot into the larger Newroom pipeline.
- **Toolchain flow**: Solo grammar → parser → AST → checker/type inference → backend (QBE planned) → runtime (Rust arena allocator / future Ensemble orchestrator). Duet and Ensemble reuse the Solo pieces, adding AI metadata and orchestration semantics without rewriting the core.

This guide points to the files that contain the concrete implementations, so the integration story is preserved in the repo.
