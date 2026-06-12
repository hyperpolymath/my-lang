(* SPDX-License-Identifier: MPL-2.0 *)
(* SPDX-FileCopyrightText: 2026 Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk> *)

(* ========================================================== *)
(* my-lang Solo core: QTT typing (Coq twin of Typing.idr)     *)
(*                                                            *)
(* [has_type G D t a]: term [t] has type [a] in type context  *)
(* [G] under usage vector [D], with [D]'s quantities          *)
(* accounting exactly for the variable uses in [t].           *)
(*                                                            *)
(* SEPARATED-CONTEXT presentation (Usage.v, design decision   *)
(* 2026-06-12): the TYPE context [G] is SHARED across every    *)
(* premise of a rule; only the USAGE vector [D] is split (via  *)
(* [uadd] at App/Pair/Let/Case) or scaled (via [uscale] at     *)
(* App/Let). Because splitting is now pure quantity algebra,   *)
(* [uadd] is genuinely commutative and associative — the clean *)
(* algebra the F1.4 substitution lemma / preservation depends  *)
(* on (the old conflated [ctx] took the type from [ctx_add]'s  *)
(* first argument, so it was not commutative in the types).    *)
(* ========================================================== *)

Require Import Coq.Init.Nat.
Require Import Quantity.
Require Import EchoMode.
Require Import Syntax.
Require Import Usage.

(** * Variable lookup: [One] at position [n], [Zero] elsewhere.

    [has_var G D n a]: in type context [G], the usage [D] spends [One]
    at de Bruijn index [n] (whose type is [a]) and [Zero] everywhere
    else. The type context is arbitrary; only the usage is pinned. *)

Inductive has_var : tctx -> uvec -> nat -> ty -> Prop :=
  | HVHere  : forall G a,
      has_var (TSnoc G a) (USnoc (uzero G) One) 0 a
  | HVThere : forall G D n a b,
      has_var G D n a ->
      has_var (TSnoc G b) (USnoc D Zero) (S n) a.

(** * The QTT typing judgement.

    Every rule shares the TYPE context [G]; the USAGE vector [D] adds
    up (App/Pair/Let/Case) or scales (App/Let) exactly as the affine
    accounting demands. *)

Inductive has_type : tctx -> uvec -> tm -> ty -> Prop :=

  | T_Var : forall G D n a,
      has_var G D n a ->
      has_type G D (Var n) a

  | T_Unit : forall G,
      has_type G (uzero G) UnitT TUnit

  | T_Lam : forall G D q a b t,
      has_type (TSnoc G a) (USnoc D q) t b ->
      has_type G D (Lam q a t) (TArr q a b)

  | T_App : forall G D D1 D2 q a b t1 t2,
      has_type G D1 t1 (TArr q a b) ->
      has_type G D2 t2 a ->
      uadd D1 (uscale q D2) = Some D ->
      has_type G D (App t1 t2) b

  | T_Pair : forall G D D1 D2 a b t1 t2,
      has_type G D1 t1 a ->
      has_type G D2 t2 b ->
      uadd D1 D2 = Some D ->
      has_type G D (Pair t1 t2) (TPair a b)

  | T_Fst : forall G D a b t,
      has_type G D t (TPair a b) ->
      has_type G D (Fst t) a

  | T_Snd : forall G D a b t,
      has_type G D t (TPair a b) ->
      has_type G D (Snd t) b

  | T_Inl : forall G D a b t,
      has_type G D t a ->
      has_type G D (Inl b t) (TSum a b)

  | T_Inr : forall G D a b t,
      has_type G D t b ->
      has_type G D (Inr a t) (TSum a b)

  | T_Case : forall G D D1 D2 a b c t tL tR,
      has_type G D1 t (TSum a b) ->
      has_type (TSnoc G a) (USnoc D2 One) tL c ->
      has_type (TSnoc G b) (USnoc D2 One) tR c ->
      uadd D1 D2 = Some D ->
      has_type G D (Case t tL tR) c

  | T_Let : forall G D D1 D2 q a b t1 t2,
      has_type G D1 t1 a ->
      has_type (TSnoc G a) (USnoc D2 q) t2 b ->
      uadd (uscale q D1) D2 = Some D ->
      has_type G D (Let q t1 t2) b

  (* echo-types residue (echo-types-integration.md slice 3).
     T_Echo introduces a residue retaining a witness [t : a] of an
     admissible collapse [a => b] at mode [m]; the codomain [b] is a
     phantom annotation (the non-dependent approximation, design §1).
     The residue does not split the usage — it just records what was
     kept, so [D] is threaded unchanged. *)
  | T_Echo : forall G D m a b t,
      has_type G D t a ->
      has_type G D (MkEcho m a b t) (TEcho m a b)

  (* T_Weaken is [EchoLinear.weaken]: a Linear echo may be weakened to
     an Affine one. One-way (the reverse is barred — no-section-weaken,
     EchoMode.no_section_weaken). Usage [D] is unchanged: weakening
     spends no resources, it only drops a distinction. *)
  | T_Weaken : forall G D a b t,
      has_type G D t (TEcho Linear a b) ->
      has_type G D (Weaken t) (TEcho Affine a b).
