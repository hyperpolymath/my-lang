(* SPDX-License-Identifier: MPL-2.0 *)
(* SPDX-FileCopyrightText: 2026 Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk> *)

(* ========================================================== *)
(* my-lang Solo core: QTT typing (Coq twin of Typing.idr)     *)
(*                                                            *)
(* [has_type g t a]: term t has type a in QTT context g, with *)
(* g's quantities accounting exactly for the variable uses in *)
(* t. Context splitting is explicit at App/Pair/Let/Case.     *)
(* ========================================================== *)

Require Import Coq.Init.Nat.
Require Import Quantity.
Require Import Syntax.
Require Import Context.

(** * Variable lookup: One at position n, Zero elsewhere *)

Inductive has_var : ctx -> nat -> ty -> Prop :=
  | HVHere  : forall g a,
      has_var (Snoc (ctx_zero g) a One) 0 a
  | HVThere : forall g n a b,
      has_var g n a ->
      has_var (Snoc g b Zero) (S n) a.

(** * The QTT typing judgement *)

Inductive has_type : ctx -> tm -> ty -> Prop :=

  | T_Var : forall g n a,
      has_var g n a ->
      has_type g (Var n) a

  | T_Unit : forall g,
      has_type (ctx_zero g) UnitT TUnit

  | T_Lam : forall g q a b t,
      has_type (Snoc g a q) t b ->
      has_type g (Lam q a t) (TArr q a b)

  | T_App : forall g g1 g2 q a b t1 t2,
      has_type g1 t1 (TArr q a b) ->
      has_type g2 t2 a ->
      ctx_add g1 (ctx_scale q g2) = Some g ->
      has_type g (App t1 t2) b

  | T_Pair : forall g g1 g2 a b t1 t2,
      has_type g1 t1 a ->
      has_type g2 t2 b ->
      ctx_add g1 g2 = Some g ->
      has_type g (Pair t1 t2) (TPair a b)

  | T_Fst : forall g a b t,
      has_type g t (TPair a b) ->
      has_type g (Fst t) a

  | T_Snd : forall g a b t,
      has_type g t (TPair a b) ->
      has_type g (Snd t) b

  | T_Inl : forall g a b t,
      has_type g t a ->
      has_type g (Inl b t) (TSum a b)

  | T_Inr : forall g a b t,
      has_type g t b ->
      has_type g (Inr a t) (TSum a b)

  | T_Case : forall g g1 g2 a b c t tL tR,
      has_type g1 t (TSum a b) ->
      has_type (Snoc g2 a One) tL c ->
      has_type (Snoc g2 b One) tR c ->
      ctx_add g1 g2 = Some g ->
      has_type g (Case t tL tR) c

  | T_Let : forall g g1 g2 q a b t1 t2,
      has_type g1 t1 a ->
      has_type (Snoc g2 a q) t2 b ->
      ctx_add (ctx_scale q g1) g2 = Some g ->
      has_type g (Let q t1 t2) b.
