-- SPDX-License-Identifier: MPL-2.0
-- SPDX-FileCopyrightText: 2026 Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk>
--
-- Syntax of the Solo core of my-lang.
--
-- Solo = simply-typed lambda calculus + Unit + pairs + sums + let,
-- with every binder annotated by a QTT quantity q ∈ {0, 1, omega}.
-- We use untyped de Bruijn indices here; Typing.idr layers the
-- QTT typing judgement on top.
--
-- This is the affine/linear *kernel* of the solo dialect. The
-- surface dialect (`dialects/solo/`) adds arena allocation,
-- references (`&`/`&mut`), structs, and contracts; those are
-- layered on this kernel in later tracks (see ALIGNMENT-PLAN.md).
--
-- Deliberately EXCLUDED from the Solo kernel (deferred to the
-- duet / ensemble / me dialects and to later tracks):
--   * effects and handlers              (shared/effect-system)
--   * session types                     (duet)
--   * agent / orchestration calculus    (ensemble)
--   * references, ownership, borrowing  (memory-model; Track F3)
--   * AI constructs                     (shared/ai-semantics)
--   * records / row polymorphism, refinement / dependent types

module Syntax

import Quantity
import EchoMode

%default total

------------------------------------------------------------
-- Types
------------------------------------------------------------

||| Solo kernel types. The function arrow `TArr q a b` carries
||| the quantity with which its argument is consumed — the
||| `(q x : τ₁) ⊸ τ₂` shape from
||| `proofs/solo/affine-types/formal-system.md`.
public export
data Ty : Type where
  TUnit : Ty
  TPair : Ty -> Ty -> Ty
  TSum  : Ty -> Ty -> Ty
  TArr  : Q -> Ty -> Ty -> Ty
  ||| Echo residue type former: `m Echo<a => b>`. The proof-relevant
  ||| residue of an admissible collapse `a => b` at linearity mode `m`
  ||| (echo-types-integration.md). Mirrors `Ty::Echo` in the Rust
  ||| checker and `TEcho` in the Coq twin.
  TEcho : Mode -> Ty -> Ty -> Ty

%name Ty a, b, c

------------------------------------------------------------
-- Terms (de Bruijn)
------------------------------------------------------------

||| Solo kernel terms, using de Bruijn indices as `Nat`.
|||
||| `Lam` carries the quantity annotation and domain type of the
||| bound variable. `Case` binds one variable in each branch.
public export
data Tm : Type where
  Var   : Nat -> Tm
  UnitT : Tm
  Lam   : Q -> Ty -> Tm -> Tm
  App   : Tm -> Tm -> Tm
  Pair  : Tm -> Tm -> Tm
  Fst   : Tm -> Tm
  Snd   : Tm -> Tm
  Inl   : Ty -> Tm -> Tm  -- annotation = the *other* summand
  Inr   : Ty -> Tm -> Tm
  Case  : Tm -> Tm -> Tm -> Tm
          -- scrutinee, left branch (binds 1), right branch (binds 1)
  Let   : Q -> Tm -> Tm -> Tm
          -- let (q x) = e1 in e2
  -- echo-types residue (echo-types-integration.md slice 3):
  -- `MkEcho m a b t` retains witness `t : a` as the proof-relevant
  -- residue of an admissible collapse `a => b` at mode `m`; `Weaken t`
  -- weakens a linear echo to an affine one (one-way).
  MkEcho : Mode -> Ty -> Ty -> Tm -> Tm
  Weaken : Tm -> Tm

%name Tm t, t1, t2, u

------------------------------------------------------------
-- de Bruijn substitution
------------------------------------------------------------
--
-- The operational semantics (Soundness.idr) reduces redexes by
-- substituting a value for the bound variable. Standard
-- capture-avoiding de Bruijn substitution: `shift` renumbers free
-- variables when pushing under a binder, `substAt j u t` replaces
-- de Bruijn index `j` by `u`, and `subst0` is the index-0 case the
-- reduction rules use. Coq twin: the same operations in Syntax.v.

||| `shift c t`: increment every free variable `>= c` by one.
public export
shift : Nat -> Tm -> Tm
shift c (Var k)        = if k < c then Var k else Var (S k)
shift c UnitT          = UnitT
shift c (Lam q a t)    = Lam q a (shift (S c) t)
shift c (App t1 t2)    = App (shift c t1) (shift c t2)
shift c (Pair t1 t2)   = Pair (shift c t1) (shift c t2)
shift c (Fst t)        = Fst (shift c t)
shift c (Snd t)        = Snd (shift c t)
shift c (Inl b t)      = Inl b (shift c t)
shift c (Inr a t)      = Inr a (shift c t)
shift c (Case t tL tR) = Case (shift c t) (shift (S c) tL) (shift (S c) tR)
shift c (Let q t1 t2)  = Let q (shift c t1) (shift (S c) t2)
shift c (MkEcho m a b t) = MkEcho m a b (shift c t)
shift c (Weaken t)     = Weaken (shift c t)

||| `substAt j u t`: replace de Bruijn index `j` by `u`, decrementing
||| every free variable `> j`. Under a binder, `j` grows and `u` is
||| shifted so its free variables keep pointing at the same things.
public export
substAt : Nat -> Tm -> Tm -> Tm
substAt j u (Var k) = case compare k j of
                        LT => Var k
                        EQ => u
                        GT => Var (minus k 1)
substAt j u UnitT          = UnitT
substAt j u (Lam q a t)    = Lam q a (substAt (S j) (shift 0 u) t)
substAt j u (App t1 t2)    = App (substAt j u t1) (substAt j u t2)
substAt j u (Pair t1 t2)   = Pair (substAt j u t1) (substAt j u t2)
substAt j u (Fst t)        = Fst (substAt j u t)
substAt j u (Snd t)        = Snd (substAt j u t)
substAt j u (Inl b t)      = Inl b (substAt j u t)
substAt j u (Inr a t)      = Inr a (substAt j u t)
substAt j u (Case t tL tR) =
  Case (substAt j u t) (substAt (S j) (shift 0 u) tL) (substAt (S j) (shift 0 u) tR)
substAt j u (Let q t1 t2)  = Let q (substAt j u t1) (substAt (S j) (shift 0 u) t2)
substAt j u (MkEcho m a b t) = MkEcho m a b (substAt j u t)
substAt j u (Weaken t)     = Weaken (substAt j u t)

||| Single-variable substitution for index 0 — the `(\x.t) v -> t[v/x]`
||| workhorse of the reduction rules.
public export
subst0 : Tm -> Tm -> Tm
subst0 u t = substAt 0 u t
