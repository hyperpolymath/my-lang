<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->
<!-- SPDX-FileCopyrightText: 2026 Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk> -->

# Effects Implementation

This file captures the effects system content you delivered (originally `effects_implementation.ml`).

- **Source**: `proofs/shared/effect-system/soundness.md` and `proofs/shared/effect-system/algebra.md` contain the formalization, soundness proof, and algebraic laws for the effect rows and handlers. The operational counterpart lives in `proofs/shared/operational-semantics/small-step.md` and `src/checker.rs` (type-level representation).
- **Highlights**:
  * Effect rows track the set of primitives invoked by each expression.
  * Handlers are proven sound via case analyses (`Eff-IO-Print`, `Eff-Context`, etc.).
  * Algebra includes union, containment, and non-interference (see ``Row Containment`` section).

This repackages the core narrative so it is available under the historic filename.
