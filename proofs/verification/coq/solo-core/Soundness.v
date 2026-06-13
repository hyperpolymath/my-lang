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
(*   * Phase F1.4 — [preservation] and [affine_pres] are now  *)
(*     DISCHARGED as real `Theorem ... Qed.` (no Admitted/     *)
(*     Axiom), via the QTT substitution lemma [ht_subst].      *)
(*     See the F1.4 section at the foot of this file.          *)
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
Require Import PeanoNat.
Require Import Lia.

(** * Values: canonical forms of closed terms *)

Inductive value : tm -> Prop :=
  | VUnit : value UnitT
  | VLam  : forall q a t, value (Lam q a t)
  | VWith : forall t1 t2, value t1 -> value t2 -> value (With t1 t2)
  | VInl  : forall b t, value t -> value (Inl b t)
  | VInr  : forall a t, value t -> value (Inr a t)
  (* an echo with a fully-evaluated residue is a value; [Weaken] is
     not (it is the elimination that drives the linear->affine step) *)
  | VEcho : forall m a b t, value t -> value (MkEcho m a b t)
  (* a multiplicative pair is a value once both components are *)
  | VTensor : forall t1 t2, value t1 -> value t2 -> value (Tensor t1 t2).

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
  | S_LetPair : forall v1 v2 body,
      value v1 -> value v2 ->
      step (LetPair (Tensor v1 v2) body) (subst2 v1 v2 body)
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
  | S_Tensor1 : forall t1 t1' t2,
      step t1 t1' -> step (Tensor t1 t2) (Tensor t1' t2)
  | S_Tensor2 : forall v1 t2 t2',
      value v1 -> step t2 t2' -> step (Tensor v1 t2) (Tensor v1 t2')
  | S_LetPair1 : forall t1 t1' t2,
      step t1 t1' -> step (LetPair t1 t2) (LetPair t1' t2)
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
                 | m0 ae be ue Hue | w1 w2 Hw1 Hw2 ].
  - inversion Ht.
  - exists q0, a0, tb. reflexivity.
  - inversion Ht.
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
                 | m0 ae be ue Hue | w1 w2 Hw1 Hw2 ].
  - inversion Ht.
  - inversion Ht.
  - exists u1, u2. split; [reflexivity | split; assumption].
  - inversion Ht.
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
                 | m0 ae be ue Hue | w1 w2 Hw1 Hw2 ].
  - inversion Ht.
  - inversion Ht.
  - inversion Ht.
  - left.  exists b0, u0. split; [reflexivity | assumption].
  - right. exists a0, u0. split; [reflexivity | assumption].
  - inversion Ht.
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
                 | m0 ae be ue Hue | w1 w2 Hw1 Hw2 ].
  - inversion Ht.
  - inversion Ht.
  - inversion Ht.
  - inversion Ht.
  - inversion Ht.
  - inversion Ht; subst. exists ue. split; [reflexivity | assumption].
  - inversion Ht.
Qed.

(** Canonical form for tensors: a closed value of multiplicative-product
    type is a [Tensor] of two values. Drives the [LetPair] case of
    progress. *)
Lemma canon_tensor : forall a b v,
  value v -> has_type TEmpty UEmpty v (TTensor a b) ->
  exists v1 v2, v = Tensor v1 v2 /\ value v1 /\ value v2.
Proof.
  intros a b v Hv Ht.
  destruct Hv as [ | q0 a0 tb | u1 u2 Hu1 Hu2 | b0 u0 Hu0 | a0 u0 Hu0
                 | m0 ae be ue Hue | w1 w2 Hw1 Hw2 ].
  - inversion Ht.
  - inversion Ht.
  - inversion Ht.
  - inversion Ht.
  - inversion Ht.
  - inversion Ht.
  - exists w1, w2. split; [reflexivity | split; assumption].
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
    | t1 IHt1 t2 IHt2 (* Tensor  *)
    | t1 IHt1 t2 IHt2 (* LetPair *)
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

  - (* Tensor t1 t2 : multiplicative pair — a value once both components are *)
    inversion Ht; subst; empty_uvec.
    match goal with HT1 : has_type TEmpty UEmpty t1 _ |- _ =>
      match goal with HT2 : has_type TEmpty UEmpty t2 _ |- _ =>
        destruct (IHt1 _ HT1) as [Hv1 | [t1' Hs1]];
        [ destruct (IHt2 _ HT2) as [Hv2 | [t2' Hs2]];
          [ left; apply VTensor; assumption
          | right; exists (Tensor t1 t2'); apply S_Tensor2; [exact Hv1 | exact Hs2] ]
        | right; exists (Tensor t1' t2); apply S_Tensor1; exact Hs1 ]
      end
    end.

  - (* LetPair t1 t2 : steps once the scrutinee is a Tensor value *)
    inversion Ht; subst; empty_uvec.
    match goal with HT : has_type TEmpty UEmpty t1 (TTensor _ _) |- _ =>
      destruct (IHt1 _ HT) as [Hv | [t1' Hs]];
      [ destruct (canon_tensor _ _ _ Hv HT) as [v1 [v2 [Heqv [Hv1 Hv2]]]]; subst t1;
        right; exists (subst2 v1 v2 t2); apply S_LetPair; [exact Hv1 | exact Hv2]
      | right; exists (LetPair t1' t2); apply S_LetPair1; exact Hs ]
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

(* ========================================================== *)
(* F1.4 Phase 4: the QTT substitution lemma + preservation,   *)
(* DISCHARGED as real `Theorem ... Qed.` (no Admitted/Axiom). *)
(*                                                            *)
(* Architecture (standard de Bruijn type-soundness, adapted   *)
(* to the separated QTT context Usage.v):                     *)
(*   1. shape invariant   has_type -> ulen D = tlen G         *)
(*   2. weakening (ht_shift): insert a fresh Zero-usage var    *)
(*      at any depth; structural over the term.               *)
(*   3. substitution (ht_subst): substitute a value for the   *)
(*      variable at a context boundary; the affine accounting *)
(*      is pure usage-vector algebra (subst_reassoc_add and    *)
(*      subst_reassoc_mult), which is exactly where            *)
(*      uadd_comm/uadd_assoc/uscale_add pay                    *)
(*      off (the separated context's promised payload).       *)
(*   4. preservation: induction on the step relation; redexes  *)
(*      use the substitution corollaries, congruences the IH.  *)
(*                                                            *)
(* tappend/uappend = the inner-context split (low de Bruijn    *)
(* indices on top) that lets substitution recurse under        *)
(* binders. See proofs/STATUS.md / proofs/ALIGNMENT-PLAN.md.   *)
(* ========================================================== *)

(* ===== uadd length / totality ===== *)

Lemma uadd_len_eq : forall D1 D2 D, uadd D1 D2 = Some D -> ulen D1 = ulen D2.
Proof.
  induction D1 as [|D1 IH q1]; intros [|D2 q2] D H; simpl in *; try discriminate.
  - reflexivity.
  - destruct (uadd D1 D2) as [d|] eqn:E; try discriminate.
    simpl. f_equal. apply (IH D2 d E).
Qed.

Lemma uadd_len : forall D1 D2 D, uadd D1 D2 = Some D -> ulen D = ulen D1.
Proof.
  induction D1 as [|D1 IH q1]; intros [|D2 q2] D H; simpl in *; try discriminate.
  - injection H as <-. reflexivity.
  - destruct (uadd D1 D2) as [d|] eqn:E; try discriminate.
    injection H as <-. simpl. f_equal. apply (IH D2 d E).
Qed.

Lemma uadd_total : forall D1 D2, ulen D1 = ulen D2 -> exists D, uadd D1 D2 = Some D.
Proof.
  induction D1 as [|D1 IH q1]; intros [|D2 q2] H; simpl in *; try discriminate.
  - exists UEmpty. reflexivity.
  - injection H as H. destruct (IH D2 H) as [d Ed].
    exists (USnoc d (qadd q1 q2)). rewrite Ed. reflexivity.
Qed.

(** [uzero] is also a right identity for addition (on matching shapes). *)
Lemma uadd_zero_r : forall G D, ulen D = tlen G -> uadd D (uzero G) = Some D.
Proof.
  intros G D H. rewrite uadd_comm. apply uadd_zero_l. exact H.
Qed.

(* ===== context concatenation (inner part on top / low indices) ===== *)

Fixpoint tappend (G I : tctx) : tctx :=
  match I with TEmpty => G | TSnoc I' a => TSnoc (tappend G I') a end.

Fixpoint uappend (D E : uvec) : uvec :=
  match E with UEmpty => D | USnoc E' q => USnoc (uappend D E') q end.

Lemma tappend_len : forall G I, tlen (tappend G I) = tlen G + tlen I.
Proof.
  induction I as [|I IH a]; simpl.
  - rewrite Nat.add_0_r. reflexivity.
  - rewrite IH, Nat.add_succ_r. reflexivity.
Qed.

Lemma uappend_len : forall D E, ulen (uappend D E) = ulen D + ulen E.
Proof.
  induction E as [|E IH q]; simpl.
  - rewrite Nat.add_0_r. reflexivity.
  - rewrite IH, Nat.add_succ_r. reflexivity.
Qed.

Lemma uzero_tappend : forall G I, uzero (tappend G I) = uappend (uzero G) (uzero I).
Proof.
  induction I as [|I IH a]; simpl.
  - reflexivity.
  - rewrite IH. reflexivity.
Qed.

Lemma uscale_uappend : forall q D E,
  uscale q (uappend D E) = uappend (uscale q D) (uscale q E).
Proof.
  induction E as [|E IH qe]; simpl.
  - reflexivity.
  - rewrite IH. reflexivity.
Qed.

Lemma uadd_uappend : forall AI BI CI Ag Bg Cg,
  uadd AI BI = Some CI -> uadd Ag Bg = Some Cg ->
  uadd (uappend Ag AI) (uappend Bg BI) = Some (uappend Cg CI).
Proof.
  induction AI as [|AI IH qa]; intros [|BI qb] CI Ag Bg Cg HI HG;
    simpl in *; try discriminate.
  - injection HI as <-. simpl. exact HG.
  - destruct (uadd AI BI) as [ci|] eqn:E; try discriminate.
    injection HI as <-. simpl.
    rewrite (IH BI ci Ag Bg Cg E HG). reflexivity.
Qed.

Lemma uadd_uappend_inv : forall AI BI Ag Bg C,
  ulen AI = ulen BI ->
  uadd (uappend Ag AI) (uappend Bg BI) = Some C ->
  exists Cg CI, C = uappend Cg CI /\ uadd Ag Bg = Some Cg /\ uadd AI BI = Some CI.
Proof.
  induction AI as [|AI IH qa]; intros [|BI qb] Ag Bg C Hlen H;
    simpl in *; try discriminate.
  - exists C, UEmpty. split; [reflexivity | split; [exact H | reflexivity]].
  - destruct (uadd (uappend Ag AI) (uappend Bg BI)) as [d|] eqn:E; try discriminate.
    injection H as <-. injection Hlen as Hlen.
    destruct (IH BI Ag Bg d Hlen E) as [Cg [ci [Heq [HG HI]]]].
    exists Cg, (USnoc ci (qadd qa qb)). split.
    + simpl. rewrite Heq. reflexivity.
    + split; [exact HG | simpl; rewrite HI; reflexivity].
Qed.

Lemma uappend_split : forall m D,
  m <= ulen D -> exists Dhi Dlo, D = uappend Dhi Dlo /\ ulen Dlo = m.
Proof.
  induction m as [|m IH]; intros D Hle.
  - exists D, UEmpty. split; reflexivity.
  - destruct D as [|D q]; simpl in Hle.
    + inversion Hle.
    + assert (Hm : m <= ulen D) by (apply le_S_n; exact Hle).
      destruct (IH D Hm) as [Dhi [Dlo [Heq Hlen]]].
      exists Dhi, (USnoc Dlo q). split.
      * simpl. rewrite Heq. reflexivity.
      * simpl. rewrite Hlen. reflexivity.
Qed.

(* ===== uappend injectivity (given matching low-length) ===== *)

Lemma uappend_inj : forall E F D1 D2,
  ulen E = ulen F -> uappend D1 E = uappend D2 F -> D1 = D2 /\ E = F.
Proof.
  induction E as [|E IH qe]; intros [|F qf] D1 D2 Hlen Heq; simpl in *; try discriminate.
  - split; [exact Heq | reflexivity].
  - injection Heq as Heq Hq. injection Hlen as Hlen.
    destruct (IH F D1 D2 Hlen Heq) as [HD HE]. subst.
    split; reflexivity.
Qed.

(* ===== reassembly helpers for "insert Zero at the boundary" ===== *)

Lemma ushift_uscale : forall q Dg DI,
  uscale q (uappend (USnoc Dg Zero) DI)
    = uappend (USnoc (uscale q Dg) Zero) (uscale q DI).
Proof. intros. rewrite uscale_uappend. simpl. rewrite qmul_zero_r. reflexivity. Qed.

Lemma uadd_ushift : forall Ag AI Bg BI Cg CI,
  uadd Ag Bg = Some Cg -> uadd AI BI = Some CI ->
  uadd (uappend (USnoc Ag Zero) AI) (uappend (USnoc Bg Zero) BI)
     = Some (uappend (USnoc Cg Zero) CI).
Proof.
  intros Ag AI Bg BI Cg CI HG HI.
  apply uadd_uappend; [exact HI | simpl; rewrite HG; reflexivity].
Qed.

Lemma uadd_split_boundary : forall Dg1 DI1 Dg2 DI2 Dg DI,
  ulen DI1 = ulen DI2 -> ulen DI = ulen DI1 ->
  uadd (uappend Dg1 DI1) (uappend Dg2 DI2) = Some (uappend Dg DI) ->
  uadd Dg1 Dg2 = Some Dg /\ uadd DI1 DI2 = Some DI.
Proof.
  intros Dg1 DI1 Dg2 DI2 Dg DI H12 HD Heq.
  destruct (uadd_uappend_inv DI1 DI2 Dg1 Dg2 (uappend Dg DI) H12 Heq)
    as [Cg [CI [Hap [HG HI]]]].
  assert (Hl : ulen DI = ulen CI)
    by (rewrite HD; symmetry; exact (uadd_len DI1 DI2 CI HI)).
  destruct (uappend_inj DI CI Dg Cg Hl Hap) as [HDg HDI]. subst.
  split; assumption.
Qed.

(* ===== shape invariant ===== *)

Lemma shape_var : forall G D n a, has_var G D n a -> ulen D = tlen G.
Proof.
  induction 1; simpl.
  - rewrite uzero_len. reflexivity.
  - rewrite IHhas_var. reflexivity.
Qed.

Lemma ltb_SS : forall n m, Nat.ltb (S n) (S m) = Nat.ltb n m.
Proof. intros; reflexivity. Qed.

(* ===== has_var under insertion of one fresh (Zero-usage) variable ===== *)

Lemma hv_shift : forall I G c Dg DI n a,
  ulen DI = tlen I ->
  has_var (tappend G I) (uappend Dg DI) n a ->
  has_var (tappend (TSnoc G c) I) (uappend (USnoc Dg Zero) DI)
          (if n <? tlen I then n else S n) a.
Proof.
  induction I as [|I IH a']; intros G c Dg DI n a Hlen H.
  - destruct DI as [|DI qd]; simpl in Hlen; [|discriminate].
    simpl in *. apply HVThere. exact H.
  - destruct DI as [|DI qd]; simpl in Hlen; [discriminate|].
    injection Hlen as Hlen.
    simpl in H. inversion H; subst.
    + (* HVHere *)
      simpl.
      match goal with
      | [ Hz : uappend Dg DI = uzero (tappend G I) |- _ ] => idtac
      | [ Hz : uzero (tappend G I) = uappend Dg DI |- _ ] => symmetry in Hz
      end.
      match goal with
      | [ Hz : uappend Dg DI = uzero (tappend G I) |- _ ] =>
          rewrite uzero_tappend in Hz;
          destruct (uappend_inj DI (uzero I) Dg (uzero G)
                     ltac:(rewrite uzero_len; exact Hlen) Hz) as [HDg HDI]
      end.
      subst.
      replace (uappend (USnoc (uzero G) Zero) (uzero I))
        with (uzero (tappend (TSnoc G c) I))
        by (rewrite uzero_tappend; reflexivity).
      apply HVHere.
    + (* HVThere *)
      simpl. rewrite ltb_SS.
      match goal with
      | [ H' : has_var (tappend G I) (uappend Dg DI) ?m ?aa |- _ ] =>
          destruct (m <? tlen I) eqn:E;
          apply HVThere;
          specialize (IH G c Dg DI m aa Hlen H');
          rewrite E in IH; exact IH
      end.
Qed.

Lemma shape_type : forall G D t a, has_type G D t a -> ulen D = tlen G.
Proof.
  intros G D t a H.
  induction H as
    [ G D n a Hv
    | G
    | G D q a b t Ht IH
    | G D D1 D2 q a b t1 t2 Ht1 IH1 Ht2 IH2 Hadd
    | G D a b t1 t2 Ht1 IH1 Ht2 IH2
    | G D a b t Ht IH
    | G D a b t Ht IH
    | G D D1 D2 a b t1 t2 Ht1 IH1 Ht2 IH2 Hadd
    | G D D1 D2 a b c t1 t2 Ht1 IH1 Ht2 IH2 Hadd
    | G D a b t Ht IH
    | G D a b t Ht IH
    | G D D1 D2 a b c t tL tR Ht IH HtL IHL HtR IHR Hadd
    | G D D1 D2 q a b t1 t2 Ht1 IH1 Ht2 IH2 Hadd
    | G D m a b t Ht IH
    | G D a b t Ht IH ].
  - exact (shape_var _ _ _ _ Hv).
  - apply uzero_len.
  - simpl in IH. injection IH as IH. exact IH.
  - apply uadd_len in Hadd. rewrite Hadd. exact IH1.
  - exact IH1.
  - exact IH.
  - exact IH.
  - apply uadd_len in Hadd. rewrite Hadd. exact IH1.
  - apply uadd_len in Hadd. rewrite Hadd. exact IH1.
  - exact IH.
  - exact IH.
  - apply uadd_len in Hadd. rewrite Hadd. exact IH.
  - apply uadd_len in Hadd. rewrite Hadd, uscale_len. exact IH1.
  - exact IH.
  - exact IH.
Qed.

(* ===== general shift / weakening lemma ===== *)

(* split a sub-derivation's usage along the G/I boundary *)
Lemma usplit_lemma : forall G I D t a,
  has_type (tappend G I) D t a ->
  exists Dg DI, D = uappend Dg DI /\ ulen DI = tlen I.
Proof.
  intros G I D t a H.
  assert (HL : ulen D = tlen G + tlen I)
    by (rewrite (shape_type _ _ _ _ H); apply tappend_len).
  destruct (uappend_split (tlen I) D ltac:(rewrite HL; lia)) as [Dg [DI [Heq Hl]]].
  exists Dg, DI. split; assumption.
Qed.

Lemma ht_shift : forall t I G Dg DI c a,
  ulen DI = tlen I ->
  has_type (tappend G I) (uappend Dg DI) t a ->
  has_type (tappend (TSnoc G c) I) (uappend (USnoc Dg Zero) DI) (shift (tlen I) t) a.
Proof.
  induction t as
    [ n | | q tyL t1 IHt1 | t1 IHt1 t2 IHt2 | t1 IHt1 t2 IHt2
    | t1 IHt1 | t1 IHt1 | t1 IHt1 t2 IHt2 | t1 IHt1 t2 IHt2
    | bAnn t1 IHt1 | aAnn t1 IHt1 | t1 IHt1 tL IHtL tR IHtR
    | q t1 IHt1 t2 IHt2 | mE aE bE t1 IHt1 | t1 IHt1 ];
    intros I G Dg DI c a Hlen H.

  - (* Var n *)
    inversion H; subst.
    match goal with [ Hv : has_var (tappend G I) (uappend Dg DI) n a |- _ ] =>
      pose proof (hv_shift I G c Dg DI n a Hlen Hv) as Hsv end.
    simpl.
    replace (if n <? tlen I then Var n else Var (S n))
      with (Var (if n <? tlen I then n else S n))
      by (destruct (n <? tlen I); reflexivity).
    apply T_Var. exact Hsv.

  - (* UnitT *)
    inversion H; subst. simpl.
    match goal with
    | [ Hz : uappend Dg DI = uzero (tappend G I) |- _ ] => idtac
    | [ Hz : uzero (tappend G I) = uappend Dg DI |- _ ] => symmetry in Hz
    end.
    match goal with
    | [ Hz : uappend Dg DI = uzero (tappend G I) |- _ ] =>
        rewrite uzero_tappend in Hz;
        destruct (uappend_inj DI (uzero I) Dg (uzero G)
                   ltac:(rewrite uzero_len; exact Hlen) Hz) as [HDg HDI]
    end.
    subst.
    replace (uappend (USnoc (uzero G) Zero) (uzero I))
      with (uzero (tappend (TSnoc G c) I))
      by (rewrite uzero_tappend; reflexivity).
    apply T_Unit.

  - (* Lam q tyL t1 *)
    inversion H; subst. simpl. apply T_Lam.
    match goal with
    | [ Hb : has_type (TSnoc (tappend G I) tyL) (USnoc (uappend Dg DI) q) t1 ?b |- _ ] =>
        specialize (IHt1 (TSnoc I tyL) G Dg (USnoc DI q) c b);
        simpl in IHt1; apply IHt1;
        [ simpl; rewrite Hlen; reflexivity | exact Hb ]
    end.

  - (* App t1 t2 *)
    inversion H; subst. simpl.
    match goal with
    | [ Ht1 : has_type (tappend G I) ?D1 t1 (TArr ?q ?aa ?bb),
        Ht2 : has_type (tappend G I) ?D2 t2 ?aa,
        Hadd : uadd ?D1 (uscale ?q ?D2) = Some (uappend Dg DI) |- _ ] =>
        destruct (usplit_lemma G I D1 t1 _ Ht1) as [Dgx [DIx [Heqx HlIx]]];
        destruct (usplit_lemma G I D2 t2 _ Ht2) as [Dgy [DIy [Heqy HlIy]]];
        rewrite Heqx in Ht1, Hadd; rewrite Heqy in Ht2, Hadd;
        rewrite uscale_uappend in Hadd;
        destruct (uadd_split_boundary Dgx DIx (uscale q Dgy) (uscale q DIy) Dg DI
                   ltac:(rewrite uscale_len; lia) ltac:(lia) Hadd) as [HG HI];
        eapply T_App;
        [ exact (IHt1 I G Dgx DIx c (TArr q aa bb) HlIx Ht1)
        | exact (IHt2 I G Dgy DIy c aa HlIy Ht2)
        | rewrite ushift_uscale; apply uadd_ushift; assumption ]
    end.

  - (* With t1 t2 *)
    inversion H; subst. simpl.
    match goal with
    | [ Ht1 : has_type (tappend G I) (uappend Dg DI) t1 ?aa,
        Ht2 : has_type (tappend G I) (uappend Dg DI) t2 ?bb |- _ ] =>
        apply T_With;
        [ exact (IHt1 I G Dg DI c aa Hlen Ht1)
        | exact (IHt2 I G Dg DI c bb Hlen Ht2) ]
    end.

  - (* Fst t1 *)
    inversion H; subst. simpl.
    match goal with
    | [ Hb : has_type (tappend G I) (uappend Dg DI) t1 (TWith ?aa ?bb) |- _ ] =>
        exact (T_Fst _ _ aa bb _ (IHt1 I G Dg DI c (TWith aa bb) Hlen Hb))
    end.

  - (* Snd t1 *)
    inversion H; subst. simpl.
    match goal with
    | [ Hb : has_type (tappend G I) (uappend Dg DI) t1 (TWith ?aa ?bb) |- _ ] =>
        exact (T_Snd _ _ aa bb _ (IHt1 I G Dg DI c (TWith aa bb) Hlen Hb))
    end.

  - (* Tensor t1 t2 *)
    inversion H; subst. simpl.
    match goal with
    | [ Ht1 : has_type (tappend G I) ?D1 t1 ?aa,
        Ht2 : has_type (tappend G I) ?D2 t2 ?bb,
        Hadd : uadd ?D1 ?D2 = Some (uappend Dg DI) |- _ ] =>
        destruct (usplit_lemma G I D1 t1 _ Ht1) as [Dgx [DIx [Heqx HlIx]]];
        destruct (usplit_lemma G I D2 t2 _ Ht2) as [Dgy [DIy [Heqy HlIy]]];
        rewrite Heqx in Ht1, Hadd; rewrite Heqy in Ht2, Hadd;
        destruct (uadd_split_boundary Dgx DIx Dgy DIy Dg DI
                   ltac:(lia) ltac:(lia) Hadd) as [HG HI];
        eapply T_Tensor;
        [ exact (IHt1 I G Dgx DIx c aa HlIx Ht1)
        | exact (IHt2 I G Dgy DIy c bb HlIy Ht2)
        | apply uadd_ushift; assumption ]
    end.

  - (* LetPair t1 t2 *)
    inversion H; subst. simpl.
    match goal with
    | [ Ht1 : has_type (tappend G I) ?D1 t1 (TTensor ?aa ?bb),
        Ht2 : has_type (TSnoc (TSnoc (tappend G I) ?aa) ?bb)
                       (USnoc (USnoc ?D2 One) One) t2 ?cc,
        Hadd : uadd ?D1 ?D2 = Some (uappend Dg DI) |- _ ] =>
        destruct (usplit_lemma G I D1 t1 _ Ht1) as [Dgx [DIx [Heqx HlIx]]];
        assert (HL2 : tlen I <= ulen D2) by
          (pose proof (shape_type _ _ _ _ Ht2) as Hs; simpl in Hs;
           rewrite tappend_len in Hs; lia);
        destruct (uappend_split (tlen I) D2 HL2) as [Dgy [DIy [Heqy HlIy]]];
        rewrite Heqx in Ht1, Hadd; rewrite Heqy in Ht2, Hadd;
        destruct (uadd_split_boundary Dgx DIx Dgy DIy Dg DI
                   ltac:(lia) ltac:(lia) Hadd) as [HG HI];
        eapply T_LetPair;
        [ exact (IHt1 I G Dgx DIx c (TTensor aa bb) HlIx Ht1)
        | exact (IHt2 (TSnoc (TSnoc I aa) bb) G Dgy (USnoc (USnoc DIy One) One) c cc
                   ltac:(simpl; rewrite HlIy; reflexivity) Ht2)
        | apply uadd_ushift; assumption ]
    end.

  - (* Inl bAnn t1 *)
    inversion H; subst. simpl.
    match goal with
    | [ Hb : has_type (tappend G I) (uappend Dg DI) t1 ?aa |- _ ] =>
        exact (T_Inl _ _ aa bAnn _ (IHt1 I G Dg DI c aa Hlen Hb))
    end.

  - (* Inr aAnn t1 *)
    inversion H; subst. simpl.
    match goal with
    | [ Hb : has_type (tappend G I) (uappend Dg DI) t1 ?bb |- _ ] =>
        exact (T_Inr _ _ aAnn bb _ (IHt1 I G Dg DI c bb Hlen Hb))
    end.

  - (* Case t1 tL tR *)
    inversion H; subst. simpl.
    match goal with
    | [ Ht1 : has_type (tappend G I) ?D1 t1 (TSum ?aa ?bb),
        HtL : has_type (TSnoc (tappend G I) ?aa) (USnoc ?D2 One) tL ?cc,
        HtR : has_type (TSnoc (tappend G I) ?bb) (USnoc ?D2 One) tR ?cc,
        Hadd : uadd ?D1 ?D2 = Some (uappend Dg DI) |- _ ] =>
        destruct (usplit_lemma G I D1 t1 _ Ht1) as [Dgx [DIx [Heqx HlIx]]];
        assert (HL2 : tlen I <= ulen D2) by
          (pose proof (shape_type _ _ _ _ HtL) as Hs; simpl in Hs;
           rewrite tappend_len in Hs; lia);
        destruct (uappend_split (tlen I) D2 HL2) as [Dgy [DIy [Heqy HlIy]]];
        rewrite Heqx in Ht1, Hadd; rewrite Heqy in HtL, HtR, Hadd;
        destruct (uadd_split_boundary Dgx DIx Dgy DIy Dg DI
                   ltac:(lia) ltac:(lia) Hadd) as [HG HI];
        eapply T_Case;
        [ exact (IHt1 I G Dgx DIx c (TSum aa bb) HlIx Ht1)
        | exact (IHtL (TSnoc I aa) G Dgy (USnoc DIy One) c cc
                   ltac:(simpl; rewrite HlIy; reflexivity) HtL)
        | exact (IHtR (TSnoc I bb) G Dgy (USnoc DIy One) c cc
                   ltac:(simpl; rewrite HlIy; reflexivity) HtR)
        | apply uadd_ushift; assumption ]
    end.

  - (* Let q t1 t2 *)
    inversion H; subst. simpl.
    match goal with
    | [ Ht1 : has_type (tappend G I) ?D1 t1 ?aa,
        Ht2 : has_type (TSnoc (tappend G I) ?aa) (USnoc ?D2 ?q) t2 ?bb,
        Hadd : uadd (uscale ?q ?D1) ?D2 = Some (uappend Dg DI) |- _ ] =>
        destruct (usplit_lemma G I D1 t1 _ Ht1) as [Dgx [DIx [Heqx HlIx]]];
        assert (HL2 : tlen I <= ulen D2) by
          (pose proof (shape_type _ _ _ _ Ht2) as Hs; simpl in Hs;
           rewrite tappend_len in Hs; lia);
        destruct (uappend_split (tlen I) D2 HL2) as [Dgy [DIy [Heqy HlIy]]];
        rewrite Heqx in Ht1, Hadd; rewrite Heqy in Ht2, Hadd;
        rewrite uscale_uappend in Hadd;
        destruct (uadd_split_boundary (uscale q Dgx) (uscale q DIx) Dgy DIy Dg DI
                   ltac:(rewrite uscale_len; lia) ltac:(rewrite uscale_len; lia) Hadd)
          as [HG HI];
        eapply T_Let;
        [ exact (IHt1 I G Dgx DIx c aa HlIx Ht1)
        | exact (IHt2 (TSnoc I aa) G Dgy (USnoc DIy q) c bb
                   ltac:(simpl; rewrite HlIy; reflexivity) Ht2)
        | rewrite ushift_uscale; apply uadd_ushift; assumption ]
    end.

  - (* MkEcho mE aE bE t1 *)
    inversion H; subst. simpl.
    match goal with
    | [ Hb : has_type (tappend G I) (uappend Dg DI) t1 ?aa |- _ ] =>
        exact (T_Echo _ _ _ _ _ _ (IHt1 I G Dg DI c aa Hlen Hb))
    end.

  - (* Weaken t1 *)
    inversion H; subst. simpl.
    match goal with
    | [ Hb : has_type (tappend G I) (uappend Dg DI) t1 (TEcho Linear ?aa ?bb) |- _ ] =>
        exact (T_Weaken _ _ aa bb _ (IHt1 I G Dg DI c (TEcho Linear aa bb) Hlen Hb))
    end.
Qed.

(* ===== weakening: insert k fresh (Zero-usage) variables at index 0 ===== *)

Lemma ht_shift0 : forall t G Dg c a,
  has_type G Dg t a ->
  has_type (TSnoc G c) (USnoc Dg Zero) (shift 0 t) a.
Proof.
  intros t G Dg c a H.
  pose proof (ht_shift t TEmpty G Dg UEmpty c a eq_refl) as Hs.
  simpl in Hs. apply Hs. exact H.
Qed.

Fixpoint shiftn (k : nat) (u : tm) : tm :=
  match k with O => u | S k' => shift 0 (shiftn k' u) end.

Lemma weakening_append : forall I G Du u a,
  has_type G Du u a ->
  has_type (tappend G I) (uappend Du (uzero I)) (shiftn (tlen I) u) a.
Proof.
  induction I as [|I IH c]; intros G Du u a H.
  - simpl. exact H.
  - simpl. apply ht_shift0. apply IH. exact H.
Qed.

(* ===== algebraic core of substitution (the affine accounting) ===== *)

Lemma uscale_qadd : forall a b D,
  uadd (uscale a D) (uscale b D) = Some (uscale (qadd a b) D).
Proof.
  induction D as [|D IH qe]; simpl.
  - reflexivity.
  - rewrite IH, qmul_distrib_r. reflexivity.
Qed.

(* the pure Q-semiring identity behind substituting through an additive split *)
Lemma q_reassoc : forall dg1 dg2 du q' q1 q2,
  qadd (qadd dg1 (qmul q' dg2)) (qmul (qadd q1 (qmul q' q2)) du)
  = qadd (qadd dg1 (qmul q1 du)) (qmul q' (qadd dg2 (qmul q2 du))).
Proof. intros; destruct dg1, dg2, du, q', q1, q2; reflexivity. Qed.

(* its lifting to usage vectors: the two reassociations agree as options *)
Lemma vec_reassoc : forall Du Dg1 Dg2 q' q1 q2 Dgr1 Dgr2 Dg,
  uadd Dg1 (uscale q1 Du) = Some Dgr1 ->
  uadd Dg2 (uscale q2 Du) = Some Dgr2 ->
  uadd Dg1 (uscale q' Dg2) = Some Dg ->
  uadd Dgr1 (uscale q' Dgr2) = uadd Dg (uscale (qadd q1 (qmul q' q2)) Du).
Proof.
  induction Du as [|Du IH du];
    intros Dg1 Dg2 q' q1 q2 Dgr1 Dgr2 Dg H1 H2 H3.
  - simpl in *.
    destruct Dg1 as [|Dg1 dg1]; [|discriminate].
    destruct Dg2 as [|Dg2 dg2]; [|discriminate].
    injection H1 as <-. injection H2 as <-. injection H3 as <-. reflexivity.
  - destruct Dg1 as [|Dg1 dg1]; simpl in H1; [discriminate|].
    destruct (uadd Dg1 (uscale q1 Du)) as [r1|] eqn:E1; [|discriminate].
    injection H1 as <-.
    destruct Dg2 as [|Dg2 dg2]; simpl in H2; [discriminate|].
    destruct (uadd Dg2 (uscale q2 Du)) as [r2|] eqn:E2; [|discriminate].
    injection H2 as <-.
    simpl in H3.
    destruct (uadd Dg1 (uscale q' Dg2)) as [dgm|] eqn:E3; [|discriminate].
    injection H3 as <-.
    simpl. rewrite (IH Dg1 Dg2 q' q1 q2 r1 r2 dgm E1 E2 E3).
    destruct (uadd dgm (uscale (qadd q1 (qmul q' q2)) Du)) as [d|] eqn:E4.
    + f_equal. f_equal. symmetry. apply q_reassoc.
    + reflexivity.
Qed.

Lemma subst_reassoc_add : forall Dg1 Dgr1 Dg2 Dgr2 Dg Du q' q1 q2,
  uadd Dg1 (uscale q1 Du) = Some Dgr1 ->
  uadd Dg2 (uscale q2 Du) = Some Dgr2 ->
  uadd Dg1 (uscale q' Dg2) = Some Dg ->
  exists Dgr,
    uadd Dg (uscale (qadd q1 (qmul q' q2)) Du) = Some Dgr /\
    uadd Dgr1 (uscale q' Dgr2) = Some Dgr.
Proof.
  intros Dg1 Dgr1 Dg2 Dgr2 Dg Du q' q1 q2 H1 H2 H3.
  assert (HL : ulen Dg = ulen (uscale (qadd q1 (qmul q' q2)) Du)).
  { rewrite uscale_len, (uadd_len _ _ _ H3), (uadd_len_eq _ _ _ H1), uscale_len.
    reflexivity. }
  destruct (uadd_total _ _ HL) as [Dgr HDgr].
  exists Dgr. split; [exact HDgr|].
  rewrite (vec_reassoc Du Dg1 Dg2 q' q1 q2 Dgr1 Dgr2 Dg H1 H2 H3). exact HDgr.
Qed.

(* ===== boundary splitters for substitution (USnoc-headed boundary) ===== *)

Lemma usplit_lemma2 : forall G a I D t b,
  has_type (tappend (TSnoc G a) I) D t b ->
  exists Dg q DI, D = uappend (USnoc Dg q) DI /\ ulen DI = tlen I.
Proof.
  intros G a I D t b H.
  assert (HL : ulen D = tlen (TSnoc G a) + tlen I)
    by (rewrite (shape_type _ _ _ _ H); apply tappend_len).
  destruct (uappend_split (tlen I) D ltac:(rewrite HL; simpl; lia)) as [Hi [DI [Heq Hl]]].
  destruct Hi as [|Dg q].
  - exfalso. rewrite Heq, uappend_len in HL. simpl in HL. lia.
  - exists Dg, q, DI. split; [exact Heq | exact Hl].
Qed.

Lemma uadd_split_boundary2 : forall Dg1 q1 DI1 Dg2 q2 DI2 Dg q DI,
  ulen DI1 = ulen DI2 -> ulen DI = ulen DI1 ->
  uadd (uappend (USnoc Dg1 q1) DI1) (uappend (USnoc Dg2 q2) DI2)
     = Some (uappend (USnoc Dg q) DI) ->
  uadd Dg1 Dg2 = Some Dg /\ q = qadd q1 q2 /\ uadd DI1 DI2 = Some DI.
Proof.
  intros Dg1 q1 DI1 Dg2 q2 DI2 Dg q DI H12 HD Heq.
  destruct (uadd_split_boundary (USnoc Dg1 q1) DI1 (USnoc Dg2 q2) DI2 (USnoc Dg q) DI
             H12 HD Heq) as [Hhead Htail].
  simpl in Hhead. destruct (uadd Dg1 Dg2) as [d|] eqn:E; [|discriminate].
  injection Hhead as Hd Hq.
  split; [f_equal; exact Hd | split; [symmetry; exact Hq | exact Htail]].
Qed.

(* ===== Var-substitution support ===== *)

Lemma uadd_uscaleZero_r : forall D E,
  ulen D = ulen E -> uadd D (uscale Zero E) = Some D.
Proof.
  induction D as [|D IH qd]; intros [|E qe] H; simpl in *; try discriminate.
  - reflexivity.
  - injection H as H. rewrite (IH E H), qadd_zero_r. reflexivity.
Qed.

(* substituting at a deeper index commutes with one extra weakening shift *)
Lemma subst_var_succ : forall k u n,
  subst_at (S k) (shiftn (S k) u) (Var (S n))
  = shift 0 (subst_at k (shiftn k u) (Var n)).
Proof.
  intros k u n. simpl.
  destruct (n ?= k) eqn:E.
  - reflexivity.
  - reflexivity.
  - destruct n as [|n0]; [destruct k; discriminate E | reflexivity].
Qed.

Lemma hv_subst : forall I G a Dg q DI n b Du u,
  ulen DI = tlen I ->
  has_var (tappend (TSnoc G a) I) (uappend (USnoc Dg q) DI) n b ->
  has_type G Du u a ->
  exists Dgr, uadd Dg (uscale q Du) = Some Dgr /\
    has_type (tappend G I) (uappend Dgr DI)
             (subst_at (tlen I) (shiftn (tlen I) u) (Var n)) b.
Proof.
  induction I as [|I IH c]; intros G a Dg q DI n b Du u Hlen Hv Hu.
  - (* I = TEmpty *)
    destruct DI as [|DI qd]; simpl in Hlen; [|discriminate].
    simpl in Hv. inversion Hv; subst.
    + (* HVHere: substituted variable itself *)
      exists Du. split.
      * rewrite uscale_one. apply uadd_zero_l. exact (shape_type _ _ _ _ Hu).
      * simpl. exact Hu.
    + (* HVThere: a deeper G-variable, unused by the binder *)
      match goal with [ Hpv : has_var G Dg ?m b |- _ ] =>
        exists Dg; split;
        [ apply uadd_uscaleZero_r;
          rewrite (shape_var _ _ _ _ Hpv), (shape_type _ _ _ _ Hu); reflexivity
        | simpl; apply T_Var; exact Hpv ]
      end.
  - (* I = TSnoc I c *)
    destruct DI as [|DI qd]; simpl in Hlen; [discriminate|].
    injection Hlen as Hlen. simpl in Hv. inversion Hv; subst.
    + (* HVHere: the inner binder c (index 0 < tlen I), kept as Var 0 *)
      match goal with
      | [ Hz : uappend (USnoc Dg q) DI = uzero (tappend (TSnoc G a) I) |- _ ] => idtac
      | [ Hz : uzero (tappend (TSnoc G a) I) = uappend (USnoc Dg q) DI |- _ ] =>
          symmetry in Hz
      end.
      match goal with
      | [ Hz : uappend (USnoc Dg q) DI = uzero (tappend (TSnoc G a) I) |- _ ] =>
          rewrite uzero_tappend in Hz;
          destruct (uappend_inj DI (uzero I) (USnoc Dg q) (uzero (TSnoc G a))
                     ltac:(rewrite uzero_len; exact Hlen) Hz) as [Hhd HDI]
      end.
      simpl in Hhd. injection Hhd as HDg Hq. subst.
      exists (uzero G). split.
      * apply uadd_uscaleZero_r.
        rewrite uzero_len, (shape_type _ _ _ _ Hu). reflexivity.
      * simpl. apply T_Var.
        replace (uappend (uzero G) (uzero I)) with (uzero (tappend G I))
          by (rewrite uzero_tappend; reflexivity).
        apply HVHere.
    + (* HVThere: index S n', recurse and re-weaken *)
      match goal with
      | [ Hpv : has_var (tappend (TSnoc G a) I) (uappend (USnoc Dg q) DI) ?m b |- _ ] =>
          destruct (IH G a Dg q DI m b Du u Hlen Hpv Hu) as [Dgr [Hadd Hht]]
      end.
      exists Dgr. split; [exact Hadd|].
      replace (tlen (TSnoc I c)) with (S (tlen I)) by reflexivity.
      rewrite subst_var_succ.
      change (tappend G (TSnoc I c)) with (TSnoc (tappend G I) c).
      replace (uappend Dgr (USnoc DI Zero))
        with (USnoc (uappend Dgr DI) Zero) by reflexivity.
      apply ht_shift0. exact Hht.
Qed.

(* uniform USnoc-headed split by length *)
Lemma usplit3 : forall D k m,
  ulen D = S k + m ->
  exists Dg q DI, D = uappend (USnoc Dg q) DI /\ ulen DI = m /\ ulen Dg = k.
Proof.
  intros D k m HL.
  destruct (uappend_split m D ltac:(lia)) as [Hi [DI [Heq Hl]]].
  assert (HHi : ulen Hi = S k).
  { assert (ulen D = ulen Hi + ulen DI) by (rewrite Heq; apply uappend_len). lia. }
  destruct Hi as [|Dg q]; simpl in HHi; [lia|].
  exists Dg, q, DI. split; [exact Heq | split; [exact Hl | lia]].
Qed.

(* multiplicative-split variant of the accounting algebra (q' = One) *)
Lemma subst_reassoc_mult : forall Dg1 Dgr1 Dg2 Dgr2 Dg Du q1 q2,
  uadd Dg1 (uscale q1 Du) = Some Dgr1 ->
  uadd Dg2 (uscale q2 Du) = Some Dgr2 ->
  uadd Dg1 Dg2 = Some Dg ->
  exists Dgr,
    uadd Dg (uscale (qadd q1 q2) Du) = Some Dgr /\ uadd Dgr1 Dgr2 = Some Dgr.
Proof.
  intros Dg1 Dgr1 Dg2 Dgr2 Dg Du q1 q2 H1 H2 H3.
  destruct (subst_reassoc_add Dg1 Dgr1 Dg2 Dgr2 Dg Du One q1 q2 H1 H2
             ltac:(rewrite uscale_one; exact H3)) as [Dgr [HDgr Hcomb]].
  exists Dgr. rewrite uscale_one in Hcomb. rewrite (qmul_one_l q2) in HDgr.
  split; assumption.
Qed.

(* ===== the QTT substitution lemma ===== *)

Lemma ht_subst : forall t I G a Dg q DI Du u b,
  ulen DI = tlen I ->
  has_type (tappend (TSnoc G a) I) (uappend (USnoc Dg q) DI) t b ->
  has_type G Du u a ->
  exists Dgr, uadd Dg (uscale q Du) = Some Dgr /\
    has_type (tappend G I) (uappend Dgr DI)
             (subst_at (tlen I) (shiftn (tlen I) u) t) b.
Proof.
  induction t as
    [ n | | q' tyL t1 IHt1 | t1 IHt1 t2 IHt2 | t1 IHt1 t2 IHt2
    | t1 IHt1 | t1 IHt1 | t1 IHt1 t2 IHt2 | t1 IHt1 t2 IHt2
    | bAnn t1 IHt1 | aAnn t1 IHt1 | t1 IHt1 tL IHtL tR IHtR
    | q' t1 IHt1 t2 IHt2 | mE aE bE t1 IHt1 | t1 IHt1 ];
    intros I G a Dg q DI Du u b Hlen H Hu.

  - (* Var n *)
    inversion H; subst.
    match goal with
    | [ Hv : has_var (tappend (TSnoc G a) I) (uappend (USnoc Dg q) DI) n b |- _ ] =>
        exact (hv_subst I G a Dg q DI n b Du u Hlen Hv Hu)
    end.

  - (* UnitT *)
    inversion H; subst.
    match goal with
    | [ Hz : uappend (USnoc Dg q) DI = uzero (tappend (TSnoc G a) I) |- _ ] => idtac
    | [ Hz : uzero (tappend (TSnoc G a) I) = uappend (USnoc Dg q) DI |- _ ] =>
        symmetry in Hz
    end.
    match goal with
    | [ Hz : uappend (USnoc Dg q) DI = uzero (tappend (TSnoc G a) I) |- _ ] =>
        rewrite uzero_tappend in Hz;
        destruct (uappend_inj DI (uzero I) (USnoc Dg q) (uzero (TSnoc G a))
                   ltac:(rewrite uzero_len; exact Hlen) Hz) as [Hhd HDI]
    end.
    simpl in Hhd. injection Hhd as HDg Hq. subst.
    exists (uzero G). split.
    + apply uadd_uscaleZero_r. rewrite uzero_len, (shape_type _ _ _ _ Hu). reflexivity.
    + simpl.
      replace (uappend (uzero G) (uzero I)) with (uzero (tappend G I))
        by (rewrite uzero_tappend; reflexivity).
      apply T_Unit.

  - (* Lam q' tyL t1 *)
    inversion H; subst.
    match goal with
    | [ Hbody : has_type (TSnoc (tappend (TSnoc G a) I) tyL)
                         (USnoc (uappend (USnoc Dg q) DI) q') t1 ?b' |- _ ] =>
        destruct (IHt1 (TSnoc I tyL) G a Dg q (USnoc DI q') Du u b'
                   ltac:(simpl; rewrite Hlen; reflexivity) Hbody Hu) as [Dgr [Hadd Hht]];
        exists Dgr; split; [exact Hadd | simpl; apply T_Lam; exact Hht]
    end.

  - (* App t1 t2 *)
    inversion H; subst.
    match goal with
    | [ Ht1 : has_type (tappend (TSnoc G a) I) ?D1 t1 (TArr ?qf ?aa ?bb),
        Ht2 : has_type (tappend (TSnoc G a) I) ?D2 t2 ?aa,
        Hadd : uadd ?D1 (uscale ?qf ?D2) = Some (uappend (USnoc Dg q) DI) |- _ ] =>
        destruct (usplit3 D1 (tlen G) (tlen I)
                   ltac:(rewrite (shape_type _ _ _ _ Ht1), tappend_len; simpl; lia))
          as [Dg1 [q1 [DI1 [Heq1 [Hl1 _]]]]];
        destruct (usplit3 D2 (tlen G) (tlen I)
                   ltac:(rewrite (shape_type _ _ _ _ Ht2), tappend_len; simpl; lia))
          as [Dg2 [q2 [DI2 [Heq2 [Hl2 _]]]]];
        rewrite Heq1 in Ht1, Hadd; rewrite Heq2 in Ht2, Hadd;
        rewrite uscale_uappend in Hadd; simpl in Hadd;
        destruct (uadd_split_boundary2 Dg1 q1 DI1 (uscale qf Dg2) (qmul qf q2)
                   (uscale qf DI2) Dg q DI
                   ltac:(rewrite uscale_len; lia) ltac:(lia) Hadd)
          as [HGadd [Hqeq HIadd]];
        destruct (IHt1 I G a Dg1 q1 DI1 Du u (TArr qf aa bb) Hl1 Ht1 Hu)
          as [Dgr1 [Hr1 Hht1]];
        destruct (IHt2 I G a Dg2 q2 DI2 Du u aa Hl2 Ht2 Hu) as [Dgr2 [Hr2 Hht2]];
        destruct (subst_reassoc_add Dg1 Dgr1 Dg2 Dgr2 Dg Du qf q1 q2 Hr1 Hr2 HGadd)
          as [Dgr [HDgr Hcomb]];
        exists Dgr; split;
        [ rewrite Hqeq; exact HDgr
        | simpl; eapply T_App;
          [ exact Hht1 | exact Hht2 |
            rewrite uscale_uappend; apply uadd_uappend; [exact HIadd | exact Hcomb] ] ]
    end.

  - (* With t1 t2 *)
    inversion H; subst.
    match goal with
    | [ Ht1 : has_type (tappend (TSnoc G a) I) (uappend (USnoc Dg q) DI) t1 ?aa,
        Ht2 : has_type (tappend (TSnoc G a) I) (uappend (USnoc Dg q) DI) t2 ?bb |- _ ] =>
        destruct (IHt1 I G a Dg q DI Du u aa Hlen Ht1 Hu) as [Dgr [Hr Hht1]];
        destruct (IHt2 I G a Dg q DI Du u bb Hlen Ht2 Hu) as [Dgr' [Hr' Hht2]];
        assert (Dgr' = Dgr) by (rewrite Hr in Hr'; injection Hr'; auto);
        subst Dgr';
        exists Dgr; split; [exact Hr | simpl; apply T_With; [exact Hht1 | exact Hht2]]
    end.

  - (* Fst t1 *)
    inversion H; subst.
    match goal with
    | [ Hb : has_type (tappend (TSnoc G a) I) (uappend (USnoc Dg q) DI) t1
                      (TWith ?aa ?bb) |- _ ] =>
        destruct (IHt1 I G a Dg q DI Du u (TWith aa bb) Hlen Hb Hu) as [Dgr [Hr Hht]];
        exists Dgr; split; [exact Hr | simpl; exact (T_Fst _ _ aa bb _ Hht)]
    end.

  - (* Snd t1 *)
    inversion H; subst.
    match goal with
    | [ Hb : has_type (tappend (TSnoc G a) I) (uappend (USnoc Dg q) DI) t1
                      (TWith ?aa ?bb) |- _ ] =>
        destruct (IHt1 I G a Dg q DI Du u (TWith aa bb) Hlen Hb Hu) as [Dgr [Hr Hht]];
        exists Dgr; split; [exact Hr | simpl; exact (T_Snd _ _ aa bb _ Hht)]
    end.

  - (* Tensor t1 t2 *)
    inversion H; subst.
    match goal with
    | [ Ht1 : has_type (tappend (TSnoc G a) I) ?D1 t1 ?aa,
        Ht2 : has_type (tappend (TSnoc G a) I) ?D2 t2 ?bb,
        Hadd : uadd ?D1 ?D2 = Some (uappend (USnoc Dg q) DI) |- _ ] =>
        destruct (usplit3 D1 (tlen G) (tlen I)
                   ltac:(rewrite (shape_type _ _ _ _ Ht1), tappend_len; simpl; lia))
          as [Dg1 [q1 [DI1 [Heq1 [Hl1 _]]]]];
        destruct (usplit3 D2 (tlen G) (tlen I)
                   ltac:(rewrite (shape_type _ _ _ _ Ht2), tappend_len; simpl; lia))
          as [Dg2 [q2 [DI2 [Heq2 [Hl2 _]]]]];
        rewrite Heq1 in Ht1, Hadd; rewrite Heq2 in Ht2, Hadd;
        destruct (uadd_split_boundary2 Dg1 q1 DI1 Dg2 q2 DI2 Dg q DI
                   ltac:(lia) ltac:(lia) Hadd) as [HGadd [Hqeq HIadd]];
        destruct (IHt1 I G a Dg1 q1 DI1 Du u aa Hl1 Ht1 Hu) as [Dgr1 [Hr1 Hht1]];
        destruct (IHt2 I G a Dg2 q2 DI2 Du u bb Hl2 Ht2 Hu) as [Dgr2 [Hr2 Hht2]];
        destruct (subst_reassoc_mult Dg1 Dgr1 Dg2 Dgr2 Dg Du q1 q2 Hr1 Hr2 HGadd)
          as [Dgr [HDgr Hcomb]];
        exists Dgr; split;
        [ rewrite Hqeq; exact HDgr
        | simpl; eapply T_Tensor;
          [ exact Hht1 | exact Hht2 | apply uadd_uappend; [exact HIadd | exact Hcomb] ] ]
    end.

  - (* LetPair t1 t2 *)
    inversion H; subst.
    match goal with
    | [ Ht1 : has_type (tappend (TSnoc G a) I) ?D1 t1 (TTensor ?aa ?bb),
        Ht2 : has_type (TSnoc (TSnoc (tappend (TSnoc G a) I) ?aa) ?bb)
                       (USnoc (USnoc ?D2 One) One) t2 ?cc,
        Hadd : uadd ?D1 ?D2 = Some (uappend (USnoc Dg q) DI) |- _ ] =>
        destruct (usplit3 D1 (tlen G) (tlen I)
                   ltac:(rewrite (shape_type _ _ _ _ Ht1), tappend_len; simpl; lia))
          as [Dg1 [q1 [DI1 [Heq1 [Hl1 _]]]]];
        destruct (usplit3 D2 (tlen G) (tlen I)
                   ltac:(pose proof (uadd_len_eq _ _ _ Hadd) as He;
                         pose proof (shape_type _ _ _ _ Ht1) as Hs;
                         rewrite tappend_len in Hs; simpl in Hs; lia))
          as [Dg2 [q2 [DI2 [Heq2 [Hl2 _]]]]];
        rewrite Heq1 in Ht1, Hadd; rewrite Heq2 in Ht2, Hadd;
        destruct (uadd_split_boundary2 Dg1 q1 DI1 Dg2 q2 DI2 Dg q DI
                   ltac:(lia) ltac:(lia) Hadd) as [HGadd [Hqeq HIadd]];
        destruct (IHt1 I G a Dg1 q1 DI1 Du u (TTensor aa bb) Hl1 Ht1 Hu)
          as [Dgr1 [Hr1 Hht1]];
        destruct (IHt2 (TSnoc (TSnoc I aa) bb) G a Dg2 q2
                   (USnoc (USnoc DI2 One) One) Du u cc
                   ltac:(simpl; rewrite Hl2; reflexivity) Ht2 Hu) as [Dgr2 [Hr2 Hht2]];
        destruct (subst_reassoc_mult Dg1 Dgr1 Dg2 Dgr2 Dg Du q1 q2 Hr1 Hr2 HGadd)
          as [Dgr [HDgr Hcomb]];
        exists Dgr; split;
        [ rewrite Hqeq; exact HDgr
        | simpl; eapply T_LetPair;
          [ exact Hht1 | exact Hht2
          | apply uadd_uappend; [exact HIadd | exact Hcomb] ] ]
    end.

  - (* Inl bAnn t1 *)
    inversion H; subst.
    match goal with
    | [ Hb : has_type (tappend (TSnoc G a) I) (uappend (USnoc Dg q) DI) t1 ?aa |- _ ] =>
        destruct (IHt1 I G a Dg q DI Du u aa Hlen Hb Hu) as [Dgr [Hr Hht]];
        exists Dgr; split; [exact Hr | simpl; exact (T_Inl _ _ aa bAnn _ Hht)]
    end.

  - (* Inr aAnn t1 *)
    inversion H; subst.
    match goal with
    | [ Hb : has_type (tappend (TSnoc G a) I) (uappend (USnoc Dg q) DI) t1 ?bb |- _ ] =>
        destruct (IHt1 I G a Dg q DI Du u bb Hlen Hb Hu) as [Dgr [Hr Hht]];
        exists Dgr; split; [exact Hr | simpl; exact (T_Inr _ _ aAnn bb _ Hht)]
    end.

  - (* Case t1 tL tR *)
    inversion H; subst.
    match goal with
    | [ Ht1 : has_type (tappend (TSnoc G a) I) ?D1 t1 (TSum ?aa ?bb),
        HtL : has_type (TSnoc (tappend (TSnoc G a) I) ?aa) (USnoc ?D2 One) tL ?cc,
        HtR : has_type (TSnoc (tappend (TSnoc G a) I) ?bb) (USnoc ?D2 One) tR ?cc,
        Hadd : uadd ?D1 ?D2 = Some (uappend (USnoc Dg q) DI) |- _ ] =>
        destruct (usplit3 D1 (tlen G) (tlen I)
                   ltac:(rewrite (shape_type _ _ _ _ Ht1), tappend_len; simpl; lia))
          as [Dg1 [q1 [DI1 [Heq1 [Hl1 _]]]]];
        destruct (usplit3 D2 (tlen G) (tlen I)
                   ltac:(pose proof (uadd_len_eq _ _ _ Hadd) as He;
                         pose proof (shape_type _ _ _ _ Ht1) as Hs;
                         rewrite tappend_len in Hs; simpl in Hs; lia))
          as [Dg2 [q2 [DI2 [Heq2 [Hl2 _]]]]];
        rewrite Heq1 in Ht1, Hadd; rewrite Heq2 in HtL, HtR, Hadd;
        destruct (uadd_split_boundary2 Dg1 q1 DI1 Dg2 q2 DI2 Dg q DI
                   ltac:(lia) ltac:(lia) Hadd) as [HGadd [Hqeq HIadd]];
        destruct (IHt1 I G a Dg1 q1 DI1 Du u (TSum aa bb) Hl1 Ht1 Hu)
          as [Dgr1 [Hr1 Hht1]];
        destruct (IHtL (TSnoc I aa) G a Dg2 q2 (USnoc DI2 One) Du u cc
                   ltac:(simpl; rewrite Hl2; reflexivity) HtL Hu) as [Dgr2 [Hr2 HhtL]];
        destruct (IHtR (TSnoc I bb) G a Dg2 q2 (USnoc DI2 One) Du u cc
                   ltac:(simpl; rewrite Hl2; reflexivity) HtR Hu) as [Dgr2' [Hr2' HhtR]];
        assert (Dgr2' = Dgr2) by (rewrite Hr2 in Hr2'; injection Hr2'; auto);
        subst Dgr2';
        destruct (subst_reassoc_mult Dg1 Dgr1 Dg2 Dgr2 Dg Du q1 q2 Hr1 Hr2 HGadd)
          as [Dgr [HDgr Hcomb]];
        exists Dgr; split;
        [ rewrite Hqeq; exact HDgr
        | simpl; eapply T_Case;
          [ exact Hht1 | exact HhtL | exact HhtR
          | apply uadd_uappend; [exact HIadd | exact Hcomb] ] ]
    end.

  - (* Let q' t1 t2 *)
    inversion H; subst.
    match goal with
    | [ Ht1 : has_type (tappend (TSnoc G a) I) ?D1 t1 ?aa,
        Ht2 : has_type (TSnoc (tappend (TSnoc G a) I) ?aa) (USnoc ?D2 ?qf) t2 ?cc,
        Hadd : uadd (uscale ?qf ?D1) ?D2 = Some (uappend (USnoc Dg q) DI) |- _ ] =>
        destruct (usplit3 D1 (tlen G) (tlen I)
                   ltac:(rewrite (shape_type _ _ _ _ Ht1), tappend_len; simpl; lia))
          as [Dg1 [qq1 [DI1 [Heq1 [Hl1 _]]]]];
        destruct (usplit3 D2 (tlen G) (tlen I)
                   ltac:(pose proof (uadd_len_eq _ _ _ Hadd) as He;
                         rewrite uscale_len in He;
                         pose proof (shape_type _ _ _ _ Ht1) as Hs;
                         rewrite tappend_len in Hs; simpl in Hs; lia))
          as [Dg2 [qq2 [DI2 [Heq2 [Hl2 _]]]]];
        rewrite Heq1 in Ht1, Hadd; rewrite Heq2 in Ht2, Hadd;
        rewrite uscale_uappend in Hadd; simpl in Hadd;
        destruct (uadd_split_boundary2 (uscale qf Dg1) (qmul qf qq1) (uscale qf DI1)
                   Dg2 qq2 DI2 Dg q DI
                   ltac:(rewrite uscale_len; lia) ltac:(rewrite uscale_len; lia) Hadd)
          as [HGadd [Hqeq HIadd]];
        destruct (IHt1 I G a Dg1 qq1 DI1 Du u aa Hl1 Ht1 Hu) as [Dgr1 [Hr1 Hht1]];
        destruct (IHt2 (TSnoc I aa) G a Dg2 qq2 (USnoc DI2 qf) Du u cc
                   ltac:(simpl; rewrite Hl2; reflexivity) Ht2 Hu) as [Dgr2 [Hr2 Hht2]];
        destruct (subst_reassoc_add Dg2 Dgr2 Dg1 Dgr1 Dg Du qf qq2 qq1 Hr2 Hr1
                   ltac:(rewrite uadd_comm; exact HGadd)) as [Dgr [HDgr Hcomb]];
        exists Dgr; split;
        [ rewrite Hqeq, (qadd_comm (qmul qf qq1) qq2); exact HDgr
        | simpl; eapply T_Let;
          [ exact Hht1 | exact Hht2
          | rewrite uscale_uappend; apply uadd_uappend;
            [ exact HIadd | rewrite uadd_comm; exact Hcomb ] ] ]
    end.

  - (* MkEcho mE aE bE t1 *)
    inversion H; subst.
    match goal with
    | [ Hb : has_type (tappend (TSnoc G a) I) (uappend (USnoc Dg q) DI) t1 ?aa |- _ ] =>
        destruct (IHt1 I G a Dg q DI Du u aa Hlen Hb Hu) as [Dgr [Hr Hht]];
        exists Dgr; split; [exact Hr | simpl; exact (T_Echo _ _ _ _ _ _ Hht)]
    end.

  - (* Weaken t1 *)
    inversion H; subst.
    match goal with
    | [ Hb : has_type (tappend (TSnoc G a) I) (uappend (USnoc Dg q) DI) t1
                      (TEcho Linear ?aa ?bb) |- _ ] =>
        destruct (IHt1 I G a Dg q DI Du u (TEcho Linear aa bb) Hlen Hb Hu) as [Dgr [Hr Hht]];
        exists Dgr; split; [exact Hr | simpl; exact (T_Weaken _ _ aa bb _ Hht)]
    end.
Qed.

(* ===== operational substitution corollaries ===== *)

Lemma subst_lemma0 : forall G a Dg q t Du u b,
  has_type (TSnoc G a) (USnoc Dg q) t b ->
  has_type G Du u a ->
  exists Dgr, uadd Dg (uscale q Du) = Some Dgr /\ has_type G Dgr (subst0 u t) b.
Proof.
  intros G a Dg q t Du u b Ht Hu.
  destruct (ht_subst t TEmpty G a Dg q UEmpty Du u b eq_refl Ht Hu)
    as [Dgr [Hadd Hht]].
  simpl in Hht. exists Dgr. split; [exact Hadd | exact Hht].
Qed.

Lemma subst2_lemma : forall G a b D D1 D2 Dv1 Dv2 t u1 u2 c,
  has_type (TSnoc (TSnoc G a) b) (USnoc (USnoc D2 One) One) t c ->
  has_type G Dv1 u1 a ->
  has_type G Dv2 u2 b ->
  uadd Dv1 Dv2 = Some D1 ->
  uadd D1 D2 = Some D ->
  has_type G D (subst2 u1 u2 t) c.
Proof.
  intros G a b D D1 D2 Dv1 Dv2 t u1 u2 c Ht Hu1 Hu2 Hd1 Hd.
  pose proof (shape_type _ _ _ _ Hu1) as Lv1.
  pose proof (shape_type _ _ _ _ Hu2) as Lv2.
  assert (LD2 : ulen D2 = tlen G).
  { pose proof (shape_type _ _ _ _ Ht) as Lt. simpl in Lt. lia. }
  pose proof (ht_shift0 u2 G Dv2 a b Hu2) as Hu2w.
  destruct (subst_lemma0 (TSnoc G a) b (USnoc D2 One) One t (USnoc Dv2 Zero)
             (shift 0 u2) c Ht Hu2w) as [Dr1 [Hadd1 Hht1]].
  rewrite uscale_one in Hadd1.
  destruct (uadd_total D2 Dv2 ltac:(rewrite LD2, Lv2; reflexivity)) as [Dr1g HDr1g].
  assert (Dr1 = USnoc Dr1g One)
    by (simpl in Hadd1; rewrite HDr1g in Hadd1; congruence).
  subst Dr1.
  destruct (subst_lemma0 G a Dr1g One (subst0 (shift 0 u2) t) Dv1 u1 c Hht1 Hu1)
    as [Dr [Hadd2 Hht2]].
  rewrite uscale_one in Hadd2.
  assert (HDrD : Dr = D).
  { destruct (uadd_assoc Dv1 Dv2 D2 D1 D Hd1 Hd) as [YZ [Hyz Hx]].
    rewrite (uadd_comm Dv2 D2) in Hyz.
    assert (YZ = Dr1g) by congruence. subst YZ.
    rewrite (uadd_comm Dr1g Dv1) in Hadd2.
    congruence. }
  subst Dr. exact Hht2.
Qed.

(* ===== PRESERVATION ===== *)

Theorem preservation : Preservation.
Proof.
  unfold Preservation. intros G D t t' a Ht Hstep. revert G D a Ht.
  induction Hstep; intros G D Tg Ht.

  - (* S_App: (\x.tb) v --> tb[v/x] *)
    inversion Ht; subst.
    match goal with [ HL : has_type G _ (Lam _ _ _) _ |- _ ] => inversion HL; subst end.
    match goal with
    | [ HB : has_type (TSnoc G ?af) (USnoc ?D1 ?q0) ?tb ?c,
        HV : has_type G ?D2 v ?af,
        HU : uadd ?D1 (uscale ?q0 ?D2) = Some D |- _ ] =>
        destruct (subst_lemma0 G af D1 q0 tb D2 v c HB HV) as [Dgr [Hadd Hht]];
        assert (Dgr = D) by congruence; subst Dgr; exact Hht
    end.

  - (* S_Fst: fst <v1,v2> --> v1 *)
    inversion Ht; subst.
    match goal with [ HW : has_type G D (With v1 v2) _ |- _ ] => inversion HW; subst end.
    assumption.

  - (* S_Snd: snd <v1,v2> --> v2 *)
    inversion Ht; subst.
    match goal with [ HW : has_type G D (With v1 v2) _ |- _ ] => inversion HW; subst end.
    assumption.

  - (* S_CaseL: case (inl v) of ... --> tL[v] *)
    inversion Ht; subst.
    match goal with [ HI : has_type G _ (Inl _ v) _ |- _ ] => inversion HI; subst end.
    match goal with
    | [ HL : has_type (TSnoc G ?af) (USnoc ?D2 One) tL ?c,
        HV : has_type G ?D1 v ?af,
        HU : uadd ?D1 ?D2 = Some D |- _ ] =>
        destruct (subst_lemma0 G af D2 One tL D1 v c HL HV) as [Dgr [Hadd Hht]];
        rewrite uscale_one in Hadd; rewrite (uadd_comm D2 D1) in Hadd;
        assert (Dgr = D) by congruence; subst Dgr; exact Hht
    end.

  - (* S_CaseR: case (inr v) of ... --> tR[v] *)
    inversion Ht; subst.
    match goal with [ HI : has_type G _ (Inr _ v) _ |- _ ] => inversion HI; subst end.
    match goal with
    | [ HR : has_type (TSnoc G ?bf) (USnoc ?D2 One) tR ?c,
        HV : has_type G ?D1 v ?bf,
        HU : uadd ?D1 ?D2 = Some D |- _ ] =>
        destruct (subst_lemma0 G bf D2 One tR D1 v c HR HV) as [Dgr [Hadd Hht]];
        rewrite uscale_one in Hadd; rewrite (uadd_comm D2 D1) in Hadd;
        assert (Dgr = D) by congruence; subst Dgr; exact Hht
    end.

  - (* S_Let: let q x = v in t2 --> t2[v/x] *)
    inversion Ht; subst.
    match goal with
    | [ HV : has_type G ?D1 v ?af,
        H2 : has_type (TSnoc G ?af) (USnoc ?D2 ?q0) t2 ?c,
        HU : uadd (uscale ?q0 ?D1) ?D2 = Some D |- _ ] =>
        destruct (subst_lemma0 G af D2 q0 t2 D1 v c H2 HV) as [Dgr [Hadd Hht]];
        rewrite (uadd_comm D2 (uscale q0 D1)) in Hadd;
        assert (Dgr = D) by congruence; subst Dgr; exact Hht
    end.

  - (* S_LetPair: let (x,y) = (v1,v2) in body --> body[v1,v2] *)
    inversion Ht; subst.
    match goal with [ HT : has_type G _ (Tensor v1 v2) _ |- _ ] => inversion HT; subst end.
    match goal with
    | [ HB  : has_type (TSnoc (TSnoc G ?af) ?bf) (USnoc (USnoc ?D2 One) One) body ?cc,
        HV1 : has_type G ?Dv1 v1 ?af,
        HV2 : has_type G ?Dv2 v2 ?bf,
        HUt : uadd ?Dv1 ?Dv2 = Some ?D1,
        HU  : uadd ?D1 ?D2 = Some D |- _ ] =>
        exact (subst2_lemma G af bf D D1 D2 Dv1 Dv2 body v1 v2 cc HB HV1 HV2 HUt HU)
    end.

  (* ---- congruence cases ---- *)
  - (* S_App1   *) inversion Ht; subst; econstructor; eauto.
  - (* S_App2   *) inversion Ht; subst; econstructor; eauto.
  - (* S_With1  *) inversion Ht; subst; econstructor; eauto.
  - (* S_With2  *) inversion Ht; subst; econstructor; eauto.
  - (* S_Fst1   *) inversion Ht; subst; econstructor; eauto.
  - (* S_Snd1   *) inversion Ht; subst; econstructor; eauto.
  - (* S_Inl1   *) inversion Ht; subst; econstructor; eauto.
  - (* S_Inr1   *) inversion Ht; subst; econstructor; eauto.
  - (* S_Case1  *) inversion Ht; subst; econstructor; eauto.
  - (* S_Let1   *) inversion Ht; subst; econstructor; eauto.
  - (* S_Tensor1 *) inversion Ht; subst; econstructor; eauto.
  - (* S_Tensor2 *) inversion Ht; subst; econstructor; eauto.
  - (* S_LetPair1 *) inversion Ht; subst; econstructor; eauto.
  - (* S_Echo1   *) inversion Ht; subst; econstructor; eauto.
  - (* S_Weaken1 *) inversion Ht; subst; econstructor; eauto.

  - (* S_Weaken: weaken (echo_L a b v) --> echo_A a b v *)
    inversion Ht; subst.
    match goal with [ HE : has_type G D (MkEcho Linear _ _ v) _ |- _ ] =>
      inversion HE; subst end.
    econstructor; eassumption.
Qed.

Theorem affine_pres : AffinePreservation.
Proof. exact preservation. Qed.

