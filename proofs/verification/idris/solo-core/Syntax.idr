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

%name Tm t, t1, t2, u
