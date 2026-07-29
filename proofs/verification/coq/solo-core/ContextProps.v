(* SPDX-License-Identifier: MPL-2.0 *)
(* SPDX-FileCopyrightText: 2026 Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk> *)

(* ========================================================== *)
(* my-lang Solo core: QTT context algebra                     *)
(*                                                            *)
(* The structural lemmas about ctx_scale / ctx_add / ctx_zero *)
(* that the substitution lemma (Phase F1.4 preservation) needs:*)
(* scaling is a monoid action of the quantity semiring on the *)
(* context's quantity vector, and it distributes over the     *)
(* additive (context-splitting) structure.                    *)
(*                                                            *)
(* These are decision-independent (they do not depend on the  *)
(* product/elimination choice of issue #93).                  *)
(* ========================================================== *)

Require Import Quantity.
Require Import SoloCore.  (* was Syntax — merged into the consolidated functor *)
Require Import Context.

(** ** Scaling is a monoid action of (Q, qmul, One) on contexts. *)

(** Scaling by [One] is the identity. *)
Lemma ctx_scale_one : forall g, ctx_scale One g = g.
Proof.
  induction g as [| g IH a qe]; simpl.
  - reflexivity.
  - (* simpl already reduces `qmul One qe` to `qe` (One is a constructor head) *)
    rewrite IH. reflexivity.
Qed.

(** Scaling composes by quantity multiplication. *)
Lemma ctx_scale_compose : forall q1 q2 g,
  ctx_scale q1 (ctx_scale q2 g) = ctx_scale (qmul q1 q2) g.
Proof.
  intros q1 q2. induction g as [| g IH a qe]; simpl.
  - reflexivity.
  - rewrite IH, qmul_assoc. reflexivity.
Qed.

(** ** Interaction of scaling and zeroing. *)

(** The all-zero context absorbs scaling. *)
Lemma ctx_scale_zero : forall q g, ctx_scale q (ctx_zero g) = ctx_zero g.
Proof.
  intros q. induction g as [| g IH a qe]; simpl.
  - reflexivity.
  - rewrite IH, qmul_zero_r. reflexivity.
Qed.

(** Zeroing forgets a prior scaling (it forgets all quantities). *)
Lemma ctx_zero_scale : forall q g, ctx_zero (ctx_scale q g) = ctx_zero g.
Proof.
  intros q. induction g as [| g IH a qe]; simpl.
  - reflexivity.
  - rewrite IH. reflexivity.
Qed.

(** Zeroing is idempotent. *)
Lemma ctx_zero_idem : forall g, ctx_zero (ctx_zero g) = ctx_zero g.
Proof.
  induction g as [| g IH a qe]; simpl.
  - reflexivity.
  - rewrite IH. reflexivity.
Qed.

(** ** Shape preservation. *)

Lemma ctx_scale_len : forall q g, ctx_len (ctx_scale q g) = ctx_len g.
Proof.
  intros q. induction g as [| g IH a qe]; simpl.
  - reflexivity.
  - rewrite IH. reflexivity.
Qed.

Lemma ctx_zero_len : forall g, ctx_len (ctx_zero g) = ctx_len g.
Proof.
  induction g as [| g IH a qe]; simpl.
  - reflexivity.
  - rewrite IH. reflexivity.
Qed.

(** ** Scaling distributes over the additive (splitting) structure.

    This is the load-bearing lemma for the application/let cases of the
    substitution lemma: [q · (Γ₁ + Γ₂) = q·Γ₁ + q·Γ₂]. Stated on the
    partial [ctx_add] (defined when the shapes agree): whenever the sum
    exists, the scaled sum exists and equals the sum of the scalings. *)
Lemma ctx_scale_add : forall q g1 g2 g,
  ctx_add g1 g2 = Some g ->
  ctx_add (ctx_scale q g1) (ctx_scale q g2) = Some (ctx_scale q g).
Proof.
  intros q g1. induction g1 as [| g1 IHg1 a1 q1]; intros g2 g H.
  - (* g1 = Empty *)
    destruct g2; simpl in H; try discriminate.
    injection H as <-. reflexivity.
  - (* g1 = Snoc g1 a1 q1 *)
    destruct g2 as [| g2 a2 q2]; simpl in H; try discriminate.
    destruct (ctx_add g1 g2) as [c |] eqn:E; try discriminate.
    injection H as <-. simpl.
    rewrite (IHg1 g2 c E), qmul_distrib_l. reflexivity.
Qed.
