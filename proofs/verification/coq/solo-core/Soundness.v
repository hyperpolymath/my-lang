(* SPDX-License-Identifier: MPL-2.0 *)
(* SPDX-FileCopyrightText: 2026 Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk> *)

(* ========================================================== *)
(* my-lang Solo core: operational semantics + soundness       *)
(* (Coq twin of Soundness.idr)                                *)
(*                                                            *)
(*   * Phase F1.1 — the call-by-value small-step relation     *)
(*     [step] is COMMITTED (constructors below).              *)
(*   * Phase F1.3 — [progress] is DISCHARGED as a real        *)
(*     `Theorem ... Qed.` (no Admitted/Axiom).                *)
(*   * Phase F1.4 — [Preservation] remains a named            *)
(*     proposition (its QTT substitution lemma is the         *)
(*     outstanding obligation). Recorded in proofs/STATUS.md. *)
(*                                                            *)
(* Stated over the SEPARATED context (Usage.v): a closed term *)
(* is typed in the empty type context [TEmpty] under the      *)
(* empty usage [UEmpty].                                       *)
(*                                                            *)
(* Discipline: a `Definition : Prop` asserts nothing and adds *)
(* no unproved assumption to the trusted base; a `Theorem     *)
(* ... Qed.` is a real result. We never describe a hole as    *)
(* proved (proofs/STATUS.md vocabulary).                      *)
(* ========================================================== *)

Require Import Coq.Init.Nat.
Require Import Quantity.
Require Import EchoMode.
Require Import Syntax.
Require Import Usage.
Require Import Typing.

(** * Values: canonical forms of closed terms *)

Inductive value : tm -> Prop :=
  | VUnit : value UnitT
  | VLam  : forall q a t, value (Lam q a t)
  | VWith : forall t1 t2, value t1 -> value t2 -> value (With t1 t2)
  | VInl  : forall b t, value t -> value (Inl b t)
  | VInr  : forall a t, value t -> value (Inr a t)
  (* an echo with a fully-evaluated residue is a value; [Weaken] is
     not (it is the elimination that drives the linear->affine step) *)
  | VEcho : forall m a b t, value t -> value (MkEcho m a b t).

(** * Small-step reduction — call-by-value, left-to-right (F1.1).

    Computation rules fire only once their arguments are values;
    congruence rules thread evaluation left-to-right. This is the
    reference semantics the solo dialect's interpreter must match,
    and the twin of [Step] in Soundness.idr. *)

Inductive step : tm -> tm -> Prop :=
  (* --- computation --- *)
  | S_App   : forall q a t v,
      value v -> step (App (Lam q a t) v) (subst0 v t)
  | S_Fst   : forall v1 v2,
      value v1 -> value v2 -> step (Fst (With v1 v2)) v1
  | S_Snd   : forall v1 v2,
      value v1 -> value v2 -> step (Snd (With v1 v2)) v2
  | S_CaseL : forall b v tL tR,
      value v -> step (Case (Inl b v) tL tR) (subst0 v tL)
  | S_CaseR : forall a v tL tR,
      value v -> step (Case (Inr a v) tL tR) (subst0 v tR)
  | S_Let   : forall q v t2,
      value v -> step (Let q v t2) (subst0 v t2)
  (* --- congruence (left-to-right, CBV) --- *)
  | S_App1  : forall t1 t1' t2,
      step t1 t1' -> step (App t1 t2) (App t1' t2)
  | S_App2  : forall v1 t2 t2',
      value v1 -> step t2 t2' -> step (App v1 t2) (App v1 t2')
  | S_With1 : forall t1 t1' t2,
      step t1 t1' -> step (With t1 t2) (With t1' t2)
  | S_With2 : forall v1 t2 t2',
      value v1 -> step t2 t2' -> step (With v1 t2) (With v1 t2')
  | S_Fst1  : forall t t',
      step t t' -> step (Fst t) (Fst t')
  | S_Snd1  : forall t t',
      step t t' -> step (Snd t) (Snd t')
  | S_Inl1  : forall b t t',
      step t t' -> step (Inl b t) (Inl b t')
  | S_Inr1  : forall a t t',
      step t t' -> step (Inr a t) (Inr a t')
  | S_Case1 : forall t t' tL tR,
      step t t' -> step (Case t tL tR) (Case t' tL tR)
  | S_Let1  : forall q t1 t1' t2,
      step t1 t1' -> step (Let q t1 t2) (Let q t1' t2)
  (* echo residue: evaluate inside, and the one-way linear->affine
     weakening fires once the residue is a value (EchoLinear.weaken). *)
  | S_Echo1   : forall m a b t t',
      step t t' -> step (MkEcho m a b t) (MkEcho m a b t')
  | S_Weaken1 : forall t t',
      step t t' -> step (Weaken t) (Weaken t')
  | S_Weaken  : forall a b v,
      value v -> step (Weaken (MkEcho Linear a b v)) (MkEcho Affine a b v).

(** * Progress (proposition) *)

(** A closed, well-typed Solo term is a value or can step. "Closed" =
    typed in the empty type context under the empty usage. *)
Definition Progress : Prop :=
  forall t a,
    has_type TEmpty UEmpty t a ->
    value t \/ exists t', step t t'.

(** * Preservation (proposition) — outstanding obligation, F1.4 *)

(** Reduction preserves typing in the SAME context — the affine
    accounting content of the theorem. *)
Definition Preservation : Prop :=
  forall G D t t' a,
    has_type G D t a ->
    step t t' ->
    has_type G D t' a.

(** Affine preservation: identical to [Preservation] for the Solo
    kernel (the preserved usage already carries the accounting). *)
Definition AffinePreservation : Prop := Preservation.

(* ========================================================== *)
(* Progress, discharged (F1.3).                               *)
(* ========================================================== *)

(** ** Usage lemmas for closed terms.

    The empty usage splits only trivially: a sum is [UEmpty] only when
    both summands are, and a scaling is [UEmpty] only when its argument
    is. These let the progress induction recover [UEmpty] sub-usages,
    so the inductive hypotheses (stated for [UEmpty]) apply to the
    immediate subterms. The TYPE context is [TEmpty] throughout and is
    shared verbatim, so no analogous type-context lemma is needed. *)

Lemma uscale_empty : forall q D, uscale q D = UEmpty -> D = UEmpty.
Proof. intros q [| D qe] H; simpl in H; [reflexivity | discriminate]. Qed.

Lemma uadd_empty :
  forall D1 D2, uadd D1 D2 = Some UEmpty -> D1 = UEmpty /\ D2 = UEmpty.
Proof.
  intros [| D1 q1] [| D2 q2] H; simpl in H; try discriminate.
  - split; reflexivity.
  - destruct (uadd D1 D2); discriminate.
Qed.

Ltac empty_uvec :=
  repeat match goal with
  | [ H : uadd _ _ = Some UEmpty |- _ ] =>
      apply uadd_empty in H; destruct H as [? ?]
  | [ H : uscale _ _ = UEmpty |- _ ] => apply uscale_empty in H
  end; subst.

(** ** Canonical forms: a closed value's shape is fixed by its type. *)

Lemma canon_arr : forall q a b v,
  value v -> has_type TEmpty UEmpty v (TArr q a b) ->
  exists q' a' t, v = Lam q' a' t.
Proof.
  intros q a b v Hv Ht.
  destruct Hv as [ | q0 a0 tb | u1 u2 Hu1 Hu2 | b0 u0 Hu0 | a0 u0 Hu0
                 | m0 ae be ue Hue ].
  - inversion Ht.
  - exists q0, a0, tb. reflexivity.
  - inversion Ht.
  - inversion Ht.
  - inversion Ht.
  - inversion Ht.
Qed.

Lemma canon_with : forall a b v,
  value v -> has_type TEmpty UEmpty v (TWith a b) ->
  exists v1 v2, v = With v1 v2 /\ value v1 /\ value v2.
Proof.
  intros a b v Hv Ht.
  destruct Hv as [ | q0 a0 tb | u1 u2 Hu1 Hu2 | b0 u0 Hu0 | a0 u0 Hu0
                 | m0 ae be ue Hue ].
  - inversion Ht.
  - inversion Ht.
  - exists u1, u2. split; [reflexivity | split; assumption].
  - inversion Ht.
  - inversion Ht.
  - inversion Ht.
Qed.

Lemma canon_sum : forall a b v,
  value v -> has_type TEmpty UEmpty v (TSum a b) ->
  (exists b' v', v = Inl b' v' /\ value v') \/
  (exists a' v', v = Inr a' v' /\ value v').
Proof.
  intros a b v Hv Ht.
  destruct Hv as [ | q0 a0 tb | u1 u2 Hu1 Hu2 | b0 u0 Hu0 | a0 u0 Hu0
                 | m0 ae be ue Hue ].
  - inversion Ht.
  - inversion Ht.
  - inversion Ht.
  - left.  exists b0, u0. split; [reflexivity | assumption].
  - right. exists a0, u0. split; [reflexivity | assumption].
  - inversion Ht.
Qed.

(** Canonical form for echoes: a closed value of echo type is an
    [MkEcho] with a value residue (and matching mode / domain /
    codomain). Drives the [Weaken] case of progress. *)
Lemma canon_echo : forall m a b v,
  value v -> has_type TEmpty UEmpty v (TEcho m a b) ->
  exists v', v = MkEcho m a b v' /\ value v'.
Proof.
  intros m a b v Hv Ht.
  destruct Hv as [ | q0 a0 tb | u1 u2 Hu1 Hu2 | b0 u0 Hu0 | a0 u0 Hu0
                 | m0 ae be ue Hue ].
  - inversion Ht.
  - inversion Ht.
  - inversion Ht.
  - inversion Ht.
  - inversion Ht.
  - inversion Ht; subst. exists ue. split; [reflexivity | assumption].
Qed.

(** ** Progress. *)

Theorem progress : Progress.
Proof.
  unfold Progress. intros t.
  induction t as
    [ n               (* Var  *)
    |                 (* UnitT *)
    | q tyL t1 IHt1   (* Lam  *)
    | t1 IHt1 t2 IHt2 (* App  *)
    | t1 IHt1 t2 IHt2 (* With *)
    | t1 IHt1         (* Fst  *)
    | t1 IHt1         (* Snd  *)
    | bAnn t1 IHt1    (* Inl  *)
    | aAnn t1 IHt1    (* Inr  *)
    | t1 IHt1 tL IHtL tR IHtR  (* Case *)
    | q t1 IHt1 t2 IHt2        (* Let  *)
    | mE aE bE t1 IHt1         (* MkEcho *)
    | t1 IHt1 ];               (* Weaken *)
    intros a Ht.

  - (* Var n : no closed variable is well-typed *)
    inversion Ht; subst.
    match goal with H : has_var TEmpty _ _ _ |- _ => inversion H end.

  - (* UnitT : a value *)
    left. constructor.

  - (* Lam : a value *)
    left. constructor.

  - (* App t1 t2 *)
    inversion Ht; subst; empty_uvec.
    match goal with HT1 : has_type TEmpty UEmpty t1 _ |- _ =>
      match goal with HT2 : has_type TEmpty UEmpty t2 _ |- _ =>
        destruct (IHt1 _ HT1) as [Hv1 | [t1' Hs1]];
        [ destruct (IHt2 _ HT2) as [Hv2 | [t2' Hs2]];
          [ destruct (canon_arr _ _ _ _ Hv1 HT1) as [q' [a' [tb Heqv]]]; subst t1;
            right; exists (subst0 t2 tb); apply S_App; exact Hv2
          | right; exists (App t1 t2'); apply S_App2; [exact Hv1 | exact Hs2] ]
        | right; exists (App t1' t2); apply S_App1; exact Hs1 ]
      end
    end.

  - (* With t1 t2 : additive pair — a value once both components are.
       T_With shares usage (no split), so nothing to clear here. *)
    inversion Ht; subst.
    match goal with HT1 : has_type TEmpty UEmpty t1 _ |- _ =>
      match goal with HT2 : has_type TEmpty UEmpty t2 _ |- _ =>
        destruct (IHt1 _ HT1) as [Hv1 | [t1' Hs1]];
        [ destruct (IHt2 _ HT2) as [Hv2 | [t2' Hs2]];
          [ left; apply VWith; assumption
          | right; exists (With t1 t2'); apply S_With2; [exact Hv1 | exact Hs2] ]
        | right; exists (With t1' t2); apply S_With1; exact Hs1 ]
      end
    end.

  - (* Fst t1 *)
    inversion Ht; subst.
    match goal with HT : has_type TEmpty UEmpty t1 (TWith _ _) |- _ =>
      destruct (IHt1 _ HT) as [Hv | [t1' Hs]];
      [ destruct (canon_with _ _ _ Hv HT) as [v1 [v2 [Heqv [Hv1 Hv2]]]]; subst t1;
        right; exists v1; apply S_Fst; [exact Hv1 | exact Hv2]
      | right; exists (Fst t1'); apply S_Fst1; exact Hs ]
    end.

  - (* Snd t1 *)
    inversion Ht; subst.
    match goal with HT : has_type TEmpty UEmpty t1 (TWith _ _) |- _ =>
      destruct (IHt1 _ HT) as [Hv | [t1' Hs]];
      [ destruct (canon_with _ _ _ Hv HT) as [v1 [v2 [Heqv [Hv1 Hv2]]]]; subst t1;
        right; exists v2; apply S_Snd; [exact Hv1 | exact Hv2]
      | right; exists (Snd t1'); apply S_Snd1; exact Hs ]
    end.

  - (* Inl bAnn t1 *)
    inversion Ht; subst; empty_uvec.
    match goal with HT : has_type TEmpty UEmpty t1 _ |- _ =>
      destruct (IHt1 _ HT) as [Hv | [t1' Hs]];
      [ left; apply VInl; exact Hv
      | right; exists (Inl bAnn t1'); apply S_Inl1; exact Hs ]
    end.

  - (* Inr aAnn t1 *)
    inversion Ht; subst; empty_uvec.
    match goal with HT : has_type TEmpty UEmpty t1 _ |- _ =>
      destruct (IHt1 _ HT) as [Hv | [t1' Hs]];
      [ left; apply VInr; exact Hv
      | right; exists (Inr aAnn t1'); apply S_Inr1; exact Hs ]
    end.

  - (* Case t1 tL tR *)
    inversion Ht; subst; empty_uvec.
    match goal with HT : has_type TEmpty UEmpty t1 (TSum _ _) |- _ =>
      destruct (IHt1 _ HT) as [Hv | [t1' Hs]];
      [ destruct (canon_sum _ _ _ Hv HT)
          as [[bb [v [Heqv Hv']]] | [aa [v [Heqv Hv']]]]; subst t1;
        [ right; exists (subst0 v tL); apply S_CaseL; exact Hv'
        | right; exists (subst0 v tR); apply S_CaseR; exact Hv' ]
      | right; exists (Case t1' tL tR); apply S_Case1; exact Hs ]
    end.

  - (* Let q t1 t2 *)
    inversion Ht; subst; empty_uvec.
    match goal with HT : has_type TEmpty UEmpty t1 _ |- _ =>
      destruct (IHt1 _ HT) as [Hv | [t1' Hs]];
      [ right; exists (subst0 t1 t2); apply S_Let; exact Hv
      | right; exists (Let q t1' t2); apply S_Let1; exact Hs ]
    end.

  - (* MkEcho mE aE bE t1 : a value once its residue is a value *)
    inversion Ht; subst.
    match goal with HT : has_type TEmpty UEmpty t1 _ |- _ =>
      destruct (IHt1 _ HT) as [Hv | [t1' Hs]];
      [ left; apply VEcho; exact Hv
      | right; exists (MkEcho mE aE bE t1'); apply S_Echo1; exact Hs ]
    end.

  - (* Weaken t1 : steps to the affine residue once t1 is a value *)
    inversion Ht; subst.
    match goal with HT : has_type TEmpty UEmpty t1 (TEcho Linear _ _) |- _ =>
      destruct (IHt1 _ HT) as [Hv | [t1' Hs]];
      [ destruct (canon_echo _ _ _ _ Hv HT) as [v' [Heqv Hv']]; subst t1;
        right; eexists; apply S_Weaken; exact Hv'
      | right; exists (Weaken t1'); apply S_Weaken1; exact Hs ]
    end.
Qed.

(* Track F1.4: Theorem preservation : Preservation.    Proof. ... Qed. *)
(*             Theorem affine_pres  : AffinePreservation. Proof. ... Qed. *)
(* Outstanding obligation: the QTT substitution lemma respecting       *)
(* usage splitting (uadd/uscale). See proofs/STATUS.md and             *)
(* proofs/ALIGNMENT-PLAN.md.                                            *)
