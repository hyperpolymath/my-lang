(* SPDX-License-Identifier: MPL-2.0 *)
(* SPDX-FileCopyrightText: 2026 Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk> *)
(*
 * Extraction of the VERIFIED usage-walk checker `check` (R5) to OCaml, so the
 * Rust port in `crates/my-qtt` can be DIFFERENTIALLY CONFORMANCE-TESTED against
 * the machine-checked algorithm itself (not just against a handful of hand-
 * picked reflexivity vectors).
 *
 * This file is NOT part of the proof suite (`_CoqProject` does not list it, so
 * `make` ignores it). It is consumed only by `conformance/run.sh`, which runs
 *   coqc -R <solo-core> SoloCore Extract.v
 * from a scratch build directory; Coq 8.18 writes the extracted *.ml/*.mli to
 * the current working directory.
 *
 * `check` is a real `Fixpoint : tctx -> tm -> option (ty * uvec)` (SoloCore.v
 * ~2128), so it extracts to a total OCaml function. `q` extracts to the bare
 * `Zero | One | Omega` enum — constructor-for-constructor identical to the Rust
 * `my_qtt::Q`, which is what makes the line-by-line result comparison exact.
 *)
From SoloCore Require Import SoloCore.
From SoloCore Require Import Eval.
From SoloCore Require Import SessionEval.
Require Coq.extraction.Extraction.
Extraction Language OCaml.

(* Use native OCaml ints for the de Bruijn `nat` so the oracle and the Rust
   `usize`-indexed terms share one numeric vocabulary in the S-expression
   bridge. The checker only ever pattern-matches O/S and never does unbounded
   arithmetic on indices, so int is faithful for the corpus sizes used. *)
Require Import Coq.extraction.ExtrOcamlNatInt.

(* The verified functions the conformance harnesses test the Rust port against:
   `check` (R5, checker coupling #1), `step1` (Eval.v, interpreter coupling #2 —
   sound+complete vs the `step` relation), and the session steppers (SessionEval.v,
   coupling #4): `cstep1` (binary, sound+complete vs `cstep`), `gstep1` (global,
   sound+progress vs `gstep`), and `nstep1` (n-ary located, sound+progress vs
   `nstep`). *)
Separate Extraction check step1 cstep1 gstep1 nstep1.
