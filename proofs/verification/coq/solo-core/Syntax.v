(* SPDX-License-Identifier: MPL-2.0 *)
(* SPDX-FileCopyrightText: 2026 Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk> *)

(* ========================================================== *)
(* my-lang Solo core: syntax (Coq twin of Syntax.idr)         *)
(*                                                            *)
(* Simply-typed lambda calculus + Unit + pairs + sums + let,  *)
(* every binder annotated by a QTT quantity. de Bruijn terms. *)
(* ========================================================== *)

Require Import Coq.Init.Nat.
Require Import Quantity.

(** * Types *)

Inductive ty : Type :=
  | TUnit : ty
  | TPair : ty -> ty -> ty
  | TSum  : ty -> ty -> ty
  | TArr  : Q -> ty -> ty -> ty.   (* (q x : a) -> b *)

(** * Terms (de Bruijn) *)

Inductive tm : Type :=
  | Var   : nat -> tm
  | UnitT : tm
  | Lam   : Q -> ty -> tm -> tm
  | App   : tm -> tm -> tm
  | Pair  : tm -> tm -> tm
  | Fst   : tm -> tm
  | Snd   : tm -> tm
  | Inl   : ty -> tm -> tm    (* annotation = the other summand *)
  | Inr   : ty -> tm -> tm
  | Case  : tm -> tm -> tm -> tm   (* scrutinee, left (binds 1), right (binds 1) *)
  | Let   : Q -> tm -> tm -> tm.   (* let (q x) = e1 in e2 *)
