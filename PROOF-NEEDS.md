<!--
SPDX-License-Identifier: MPL-2.0
Copyright (c) Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk>
-->
# PROOF-NEEDS.md

## Template ABI Cleanup (2026-03-29)

Template ABI removed -- was creating false impression of formal verification.
The removed files (Types.idr, Layout.idr, Foreign.idr) contained only RSR template
scaffolding with unresolved {{PROJECT}}/{{AUTHOR}} placeholders and no domain-specific proofs.

When this project needs formal ABI verification, create domain-specific Idris2 proofs
following the pattern in repos like `typed-wasm`, `proven`, `echidna`, or `boj-server`.

## Trusted-base posture: module-type interface axioms (2026-06-15)

The Trusted-base reduction policy scanner flags 15 `Axiom` declarations in
`proofs/verification/coq/solo-core/ResourceAlgebra.v`. **These are not proof
debt and not global trusted-base assumptions** — they are the abstract
*interface fields* of three Coq `Module Type`s:

- `SEMIRING` (10): `qadd_comm`, `qadd_assoc`, `qadd_zero_l`, `qadd_zero_r`,
  `qmul_assoc`, `qmul_one_l`, `qmul_zero_r`, `qmul_zero_l`,
  `qmul_distrib_l`, `qmul_distrib_r`.
- `ORDERED_SEMIRING` (3): `qle_refl`, `qle_trans`, `qle_zero`.
- `RESIDUE_MEASURE` (2): `measure_empty`, `measure_combine`.

A `Module Type` axiom is a *parameter* of the parametric soundness functor
`SoloCoreF`, **discharged by every concrete instance**: `Module Linear3 <:
ORDERED_SEMIRING` proves all of them by `destruct; reflexivity` (real `Qed`),
and `Module Tropical <: ORDERED_SEMIRING` likewise at the infinite carrier;
`EchoTraceTropical` discharges the `RESIDUE_MEASURE` pair. This is exactly why
`Print Assumptions progress` / `preservation` reports **"Closed under the
global context"** for the concrete development (`Include SoloCoreF Linear3`) —
asserted in `proofs.yml`. Each axiom additionally carries an inline `AXIOM:`
annotation per the policy. No genuine proof obligation is hidden here.
