(* SPDX-License-Identifier: MPL-2.0 *)
(* SPDX-FileCopyrightText: 2026 Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk> *)

(* ========================================================== *)
(* my-lang Solo core: syntax (Coq twin of Syntax.idr)         *)
(*                                                            *)
(* Simply-typed lambda calculus + Unit + additive pairs (&) + *)
(* sums + let,                                                *)
(* every binder annotated by a QTT quantity. de Bruijn terms. *)
(* ========================================================== *)

Require Import Coq.Init.Nat.
Require Import Quantity.
Require Import EchoMode.

(** * Types *)

Inductive ty : Type :=
  | TUnit : ty
  | TWith : ty -> ty -> ty          (* additive product  a & b  (shared usage,
                                       projected by Fst/Snd). The multiplicative
                                       product  a (X) b  is deferred to a
                                       follow-up (TTensor + let-pair). *)
  | TSum  : ty -> ty -> ty
  | TArr  : Q -> ty -> ty -> ty     (* (q x : a) -> b *)
  | TEcho : Mode -> ty -> ty -> ty. (* echo residue: [m] Echo<a => b> *)

(** * Terms (de Bruijn) *)

Inductive tm : Type :=
  | Var   : nat -> tm
  | UnitT : tm
  | Lam   : Q -> ty -> tm -> tm
  | App   : tm -> tm -> tm
  | With  : tm -> tm -> tm    (* additive pair  <t1, t2> : a & b *)
  | Fst   : tm -> tm
  | Snd   : tm -> tm
  | Inl   : ty -> tm -> tm    (* annotation = the other summand *)
  | Inr   : ty -> tm -> tm
  | Case  : tm -> tm -> tm -> tm   (* scrutinee, left (binds 1), right (binds 1) *)
  | Let   : Q -> tm -> tm -> tm    (* let (q x) = e1 in e2 *)
  (* echo-types residue (echo-types-integration.md slice 3):
     [MkEcho m a b t] retains witness [t : a] as the proof-relevant
     residue of an admissible collapse [a => b] at linearity mode [m];
     [Weaken t] weakens a linear echo to an affine one (one-way). *)
  | MkEcho : Mode -> ty -> ty -> tm -> tm
  | Weaken : tm -> tm.

(** * de Bruijn substitution

    The operational semantics (Soundness.v) reduces redexes by
    substituting a value for the bound variable. We give the standard
    capture-avoiding de Bruijn substitution: [shift] renumbers free
    variables when we push under a binder, [subst_at j u t] replaces
    de Bruijn index [j] in [t] by [u] (renumbering the rest), and
    [subst0] is the single-variable substitution for index 0 used by
    the reduction rules. These are pure syntactic operations; the
    *typing* content (the substitution lemma) lives in Soundness.v. *)

Require Import PeanoNat.

(** [shift c t]: increment every free variable [>= c] by one. The
    cutoff [c] grows by one under each binder so bound occurrences are
    left untouched. *)
Fixpoint shift (c : nat) (t : tm) : tm :=
  match t with
  | Var k        => if Nat.ltb k c then Var k else Var (S k)
  | UnitT        => UnitT
  | Lam q a t1   => Lam q a (shift (S c) t1)
  | App t1 t2    => App (shift c t1) (shift c t2)
  | With t1 t2   => With (shift c t1) (shift c t2)
  | Fst t1       => Fst (shift c t1)
  | Snd t1       => Snd (shift c t1)
  | Inl b t1     => Inl b (shift c t1)
  | Inr a t1     => Inr a (shift c t1)
  | Case t1 tL tR => Case (shift c t1) (shift (S c) tL) (shift (S c) tR)
  | Let q t1 t2  => Let q (shift c t1) (shift (S c) t2)
  | MkEcho m a b t1 => MkEcho m a b (shift c t1)
  | Weaken t1    => Weaken (shift c t1)
  end.

(** [subst_at j u t]: replace de Bruijn index [j] in [t] by [u],
    decrementing every free variable [> j] (the binder [j] disappears).
    Under each binder [j] increments and [u] is shifted to keep its
    free variables pointing at the same things. *)
Fixpoint subst_at (j : nat) (u : tm) (t : tm) : tm :=
  match t with
  | Var k =>
      match Nat.compare k j with
      | Lt => Var k
      | Eq => u
      | Gt => Var (Nat.pred k)
      end
  | UnitT        => UnitT
  | Lam q a t1   => Lam q a (subst_at (S j) (shift 0 u) t1)
  | App t1 t2    => App (subst_at j u t1) (subst_at j u t2)
  | With t1 t2   => With (subst_at j u t1) (subst_at j u t2)
  | Fst t1       => Fst (subst_at j u t1)
  | Snd t1       => Snd (subst_at j u t1)
  | Inl b t1     => Inl b (subst_at j u t1)
  | Inr a t1     => Inr a (subst_at j u t1)
  | Case t1 tL tR =>
      Case (subst_at j u t1)
           (subst_at (S j) (shift 0 u) tL)
           (subst_at (S j) (shift 0 u) tR)
  | Let q t1 t2  => Let q (subst_at j u t1) (subst_at (S j) (shift 0 u) t2)
  | MkEcho m a b t1 => MkEcho m a b (subst_at j u t1)
  | Weaken t1    => Weaken (subst_at j u t1)
  end.

(** Single-variable substitution for index 0 — the [(\x.t) v -> t[v/x]]
    workhorse of the reduction rules. *)
Definition subst0 (u : tm) (t : tm) : tm := subst_at 0 u t.
