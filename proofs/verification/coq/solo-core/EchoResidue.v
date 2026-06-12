(* SPDX-License-Identifier: MPL-2.0 *)
(* SPDX-FileCopyrightText: 2026 Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk> *)

(* ========================================================== *)
(* my-lang Solo core: Echo residue — proof-layer object       *)
(* (Coq twin of EchoResidue.idr)                              *)
(*                                                            *)
(* This is "slice 3" of docs/design/echo-types-integration.md *)
(* — the proof-layer residue object that the Rust checker     *)
(* (crates/my-lang/src/types.rs, `Ty::Echo` + `EchoMode`)     *)
(* points at. It pins, against the *mechanised* kernel, the   *)
(* facts the checker's Echo subtyping relies on:              *)
(*                                                            *)
(*   * the linear->affine weakening is well-typed             *)
(*     (kernel rule T_Weaken) and actually reduces            *)
(*     (kernel rule S_Weaken);                                *)
(*   * Echo subtyping = source mode weakens to target mode    *)
(*     with domain & codomain invariant — exactly             *)
(*     Ty::is_assignable_from for the Echo case;              *)
(*   * weakening has *no section* (affine cannot be used      *)
(*     where linear is required) — EchoLinear.no-section.     *)
(*                                                            *)
(* Mirrors the echo-types Agda library module EchoLinear      *)
(* (`Mode`, `_<=m_`, `weaken`, `no-section-weaken`), narrowed *)
(* to the R-2026-05-18 claim line: Echo is a loss-graded      *)
(* reindexing modality over a thin poset (design note §6).    *)
(* ========================================================== *)

Require Import Quantity.
Require Import EchoMode.
Require Import Syntax.
Require Import Usage.
Require Import Typing.
Require Import Soundness.

(** * 1. The weakening is well-typed and reduces (kernel-backed).

    These re-export the kernel rules under residue-facing names, so a
    consumer can cite "the echo weaken is sound" without unfolding the
    typing judgement. [T_Weaken] / [S_Weaken] are the source of truth. *)

(** Typed weaken: a Linear echo over [a => b] weakens to an Affine one
    in the SAME context — no resources are spent, only a distinction is
    dropped. *)
Theorem echo_weaken_typed : forall G D a b t,
  has_type G D t (TEcho Linear a b) ->
  has_type G D (Weaken t) (TEcho Affine a b).
Proof. intros. apply T_Weaken. assumption. Qed.

(** Operational weaken: once the residue is a value, [Weaken] takes the
    Linear echo to the corresponding Affine echo. *)
Theorem echo_weaken_step : forall a b v,
  value v ->
  step (Weaken (MkEcho Linear a b v)) (MkEcho Affine a b v).
Proof. intros. apply S_Weaken. assumption. Qed.

(** The weaken preserves the retained residue — the affine echo carries
    exactly the same witness [v]. "Loss that is not total erasure." *)
Theorem echo_weaken_keeps_residue : forall a b v,
  value v ->
  exists v', step (Weaken (MkEcho Linear a b v)) (MkEcho Affine a b v')
             /\ v' = v.
Proof. intros a b v Hv. exists v. split; [ apply S_Weaken; assumption | reflexivity ]. Qed.

(** * 2. Echo subtyping — the formal backing of `Ty::is_assignable_from`.

    The Rust checker accepts a source echo where a target echo is
    expected exactly when [mle source_mode target_mode] holds and the
    domain/codomain agree. We name that relation and prove the laws the
    five Rust unit tests assert. *)

Definition echo_assignable
    (tgt_mode src_mode : Mode) (tgt_dom tgt_cod src_dom src_cod : ty) : Prop :=
  mle src_mode tgt_mode = true /\ tgt_dom = src_dom /\ tgt_cod = src_cod.

(** Reflexivity: any echo is assignable to one of the same shape. *)
Theorem echo_assignable_refl : forall m a b,
  echo_assignable m m a b a b.
Proof. intros. unfold echo_assignable. repeat split. apply mle_refl. Qed.

(** [EchoLinear.weaken]: a Linear echo is assignable where an Affine
    echo is expected. (target = Affine, source = Linear.) *)
Theorem echo_assignable_weaken : forall a b,
  echo_assignable Affine Linear a b a b.
Proof.
  intros a b. unfold echo_assignable.
  split; [ exact weaken_linear_affine | split; reflexivity ].
Qed.

(** [EchoLinear.no-section-weaken]: an Affine echo is NOT assignable
    where a Linear echo is required. Weakening is one-way; it has no
    section. (target = Linear, source = Affine.) *)
Theorem echo_assignable_no_section : forall a b,
  ~ echo_assignable Linear Affine a b a b.
Proof.
  intros a b [Hmle _]. discriminate.
Qed.

(** Domain / codomain are invariant under echo subtyping: assignability
    forces them equal (no covariance / contravariance in slice 1). *)
Theorem echo_assignable_dom_cod_invariant :
  forall mt ms dt ct ds cs,
  echo_assignable mt ms dt ct ds cs -> dt = ds /\ ct = cs.
Proof. intros mt ms dt ct ds cs [_ [Hd Hc]]. split; assumption. Qed.

(** Transitivity — assignability composes (the thin poset is a
    preorder), so a chain of weakenings is itself a weakening. *)
Theorem echo_assignable_trans :
  forall m1 m2 m3 a b,
  echo_assignable m2 m1 a b a b ->
  echo_assignable m3 m2 a b a b ->
  echo_assignable m3 m1 a b a b.
Proof.
  intros m1 m2 m3 a b [H12 _] [H23 _]. unfold echo_assignable.
  repeat split. apply (mle_trans m1 m2 m3); assumption.
Qed.

(** * 3. Coherence: the typed kernel weaken realises echo subtyping.

    The kernel's [Weaken] term turns a [TEcho Linear a b] into a
    [TEcho Affine a b] — i.e. it inhabits exactly the assignability
    relation [echo_assignable Affine Linear]. The proof-relevant
    residue and the type-algebra subtyping agree. *)

Theorem weaken_realises_assignable : forall G D a b t,
  has_type G D t (TEcho Linear a b) ->
  has_type G D (Weaken t) (TEcho Affine a b) /\
  echo_assignable Affine Linear a b a b.
Proof.
  intros G D a b t Ht. split.
  - apply echo_weaken_typed. assumption.
  - apply echo_assignable_weaken.
Qed.
