<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->
<!-- SPDX-FileCopyrightText: 2026 Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk> -->

# Contracts Implementation

This document preserves the contracts system proof and implementation notes (matching the former `contracts_implementation.ml`).

- **Core references**: `proofs/shared/type-system/soundness.md` (contract clauses), `proofs/shared/operational-semantics/small-step.md` (contract-triggered reduction rules), and the AST/`src/checker.rs` where `Contract` nodes are represented.
- **Summary**:
  * Contracts (`pre`, `post`, `invariant`) only admit boolean expressions; the type checker enforces this constraint.
  * The proof shows that the verifier can generate verification conditions, and that satisfying them preserves the semantics of compiled code.
  * Contracts are integrated with the effect system to track AI checks (`ai_check`, `ai_ensure`).

This file links the high-level textual narrative with the active implementation artifacts.
