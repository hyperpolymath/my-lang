(* SPDX-License-Identifier: MPL-2.0 *)
(* SPDX-FileCopyrightText: 2026 Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk> *)

(* ========================================================== *)
(* my-lang Solo core: soundness STATEMENTS                    *)
(* (Coq twin of Soundness.idr)                                *)
(*                                                            *)
(* Statements-first: progress and preservation are committed  *)
(* as theorems with [Admitted] placeholders. The case-by-case *)
(* derivations land incrementally (Track F1; ALIGNMENT-PLAN). *)
(* Each [Admitted] is a tracked proof obligation, NOT a claim  *)
(* of a completed proof — see proofs/STATUS.md.               *)
(* ========================================================== *)

Require Import Coq.Init.Nat.
Require Import Quantity.
Require Import Syntax.
Require Import Context.
Require Import Typing.

(** * Values: canonical forms of closed terms *)

Inductive value : tm -> Prop :=
  | VUnit : value UnitT
  | VLam  : forall q a t, value (Lam q a t)
  | VPair : forall t1 t2, value t1 -> value t2 -> value (Pair t1 t2)
  | VInl  : forall b t, value t -> value (Inl b t)
  | VInr  : forall a t, value t -> value (Inr a t).

(** * Small-step reduction (relation declared; constructors
      deferred to Track F1.3 once the operational semantics is
      committed — call-by-value, left-to-right). *)

Inductive step : tm -> tm -> Prop := .

(** * Progress *)

(** A closed, well-typed Solo term is a value or can step. *)
Theorem progress : forall t a,
  has_type Empty t a ->
  value t \/ exists t', step t t'.
Proof.
  (* Track F1: by induction on the typing derivation, using the
     canonical-forms lemmas. *)
Admitted.

(** * Preservation *)

(** Reduction preserves typing in the SAME context — the affine
    accounting content of the theorem. *)
Theorem preservation : forall g t t' a,
  has_type g t a ->
  step t t' ->
  has_type g t' a.
Proof.
  (* Track F1: by induction on the step relation, using a QTT
     substitution lemma that respects context splitting. *)
Admitted.

(** * Affine preservation (corollary) *)

Theorem affine_preservation : forall g t t' a,
  has_type g t a ->
  step t t' ->
  has_type g t' a.
Proof. exact preservation. Qed.
