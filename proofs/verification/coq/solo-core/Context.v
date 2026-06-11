(* SPDX-License-Identifier: MPL-2.0 *)
(* SPDX-FileCopyrightText: 2026 Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk> *)

(* ========================================================== *)
(* my-lang Solo core: QTT contexts (Coq twin of Context.idr)  *)
(*                                                            *)
(* Snoc-list contexts of (ty, Q). Pointwise addition, scalar  *)
(* multiplication, and zeroing — the operations the typing    *)
(* judgement uses to split contexts across subterms.          *)
(* ========================================================== *)

Require Import Quantity.
Require Import Syntax.

(** * Context representation *)

Inductive ctx : Type :=
  | Empty : ctx
  | Snoc  : ctx -> ty -> Q -> ctx.

Fixpoint ctx_len (g : ctx) : nat :=
  match g with
  | Empty       => 0
  | Snoc g' _ _ => S (ctx_len g')
  end.

(** * Pointwise quantity addition (partial: shapes must match) *)

Fixpoint ctx_add (g1 g2 : ctx) : option ctx :=
  match g1, g2 with
  | Empty, Empty => Some Empty
  | Snoc g1' a1 q1, Snoc g2' _ q2 =>
      match ctx_add g1' g2' with
      | None    => None
      | Some g  => Some (Snoc g a1 (qadd q1 q2))
      end
  | _, _ => None
  end.

(** * Scalar multiplication of a context *)

Fixpoint ctx_scale (q : Q) (g : ctx) : ctx :=
  match g with
  | Empty         => Empty
  | Snoc g' a qe  => Snoc (ctx_scale q g') a (qmul q qe)
  end.

(** * The all-zero context of a given shape *)

Fixpoint ctx_zero (g : ctx) : ctx :=
  match g with
  | Empty        => Empty
  | Snoc g' a _  => Snoc (ctx_zero g') a Zero
  end.
