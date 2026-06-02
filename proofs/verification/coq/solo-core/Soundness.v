(* SPDX-License-Identifier: MPL-2.0 *)
(* SPDX-FileCopyrightText: 2026 Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk> *)

(* ========================================================== *)
(* my-lang Solo core: soundness STATEMENTS                    *)
(* (Coq twin of Soundness.idr)                                *)
(*                                                            *)
(* Statements-first: progress and preservation are committed  *)
(* as named PROPOSITIONS (Definition ... : Prop), not as      *)
(* incomplete Theorems. This is deliberate:                   *)
(*                                                            *)
(*   - A bare `Definition : Prop` asserts NOTHING — it only   *)
(*     records the proposition we intend to prove. It leaves  *)
(*     no proof hole and adds no unproved assumption to the   *)
(*     trusted base (cf. the standards Trusted-Base Reduction *)
(*     Policy, which forbids undocumented proof-debt escape   *)
(*     hatches in .v files).                                  *)
(*   - Each proposition is discharged later as                *)
(*       `Theorem progress : Progress. Proof. ... Qed.`       *)
(*     (Track F1.3 / F1.4 — see proofs/ALIGNMENT-PLAN.md).    *)
(*                                                            *)
(* Authoritative status: proofs/STATUS.md. These are          *)
(* *statement-only* until the discharging Theorems land.      *)
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
      deferred to Track F1.1 once the operational semantics is
      committed — call-by-value, left-to-right). *)

Inductive step : tm -> tm -> Prop := .

(** * Progress (proposition) *)

(** A closed, well-typed Solo term is a value or can step. *)
Definition Progress : Prop :=
  forall t a,
    has_type Empty t a ->
    value t \/ exists t', step t t'.

(** * Preservation (proposition) *)

(** Reduction preserves typing in the SAME context — the affine
    accounting content of the theorem. *)
Definition Preservation : Prop :=
  forall g t t' a,
    has_type g t a ->
    step t t' ->
    has_type g t' a.

(** * Affine preservation (proposition; same as Preservation for
      the Solo kernel, since the preserved context already carries
      the quantity accounting). *)
Definition AffinePreservation : Prop := Preservation.

(* Track F1.3: Theorem progress         : Progress.        Proof. ... Qed. *)
(* Track F1.4: Theorem preservation     : Preservation.    Proof. ... Qed. *)
(* Track F1.4: Theorem affine_pres      : AffinePreservation. Proof. ... Qed. *)
