-- SPDX-License-Identifier: MPL-2.0
-- SPDX-FileCopyrightText: 2026 Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk>
--
-- Syntax of the Solo core of my-lang.
--
-- Solo = simply-typed lambda calculus + Unit + the TWO genuine linear
-- products + sums + let, with every binder annotated by a QTT quantity
-- q ∈ {0, 1, omega}. Untyped de Bruijn indices here; Typing.idr layers
-- the QTT typing judgement on top.
--
-- TWO PRODUCTS (ADR-007, mirrored from the Coq twin SoloCore.v):
--   * additive    `a & b`   (TWith)   — intro `With t1 t2`, shared usage,
--                                        eliminated by projections Fst/Snd;
--   * multiplicative `a (X) b` (TTensor) — intro `Tensor t1 t2`, split usage,
--                                        eliminated by `LetPair` (binds 2).
-- The earlier single conflated `TPair` (split-at-intro + project-at-elim)
-- was neither `&` nor `(X)` and strictly weaker than both; it is gone.
--
-- This is the affine/linear *kernel* of the solo dialect. The surface
-- dialect (`dialects/solo/`) adds arena allocation, references
-- (`&`/`&mut`), structs, and contracts; layered on this kernel later.
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
  ||| Additive product `a & b`: shared usage, projected by Fst/Snd.
  TWith : Ty -> Ty -> Ty
  ||| Multiplicative product `a (X) b`: split usage, eliminated by let-pair.
  TTensor : Ty -> Ty -> Ty
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
||| `LetPair` binds TWO variables in its body (x at index 1, y at 0).
public export
data Tm : Type where
  Var   : Nat -> Tm
  UnitT : Tm
  Lam   : Q -> Ty -> Tm -> Tm
  App   : Tm -> Tm -> Tm
  ||| Additive pair `<t1, t2> : a & b`.
  With  : Tm -> Tm -> Tm
  Fst   : Tm -> Tm
  Snd   : Tm -> Tm
  ||| Multiplicative pair `(t1, t2) : a (X) b`.
  Tensor  : Tm -> Tm -> Tm
  ||| `let (x, y) = e1 in e2` — e2 binds two vars (x@1, y@0).
  LetPair : Tm -> Tm -> Tm
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
-- reduction rules use. Coq twin: the same operations in SoloCore.v.

||| `shift c t`: increment every free variable `>= c` by one. The
||| cutoff grows by one under each binder (by two under LetPair's body).
public export
shift : Nat -> Tm -> Tm
shift c (Var k)        = if k < c then Var k else Var (S k)
shift c UnitT          = UnitT
shift c (Lam q a t)    = Lam q a (shift (S c) t)
shift c (App t1 t2)    = App (shift c t1) (shift c t2)
shift c (With t1 t2)   = With (shift c t1) (shift c t2)
shift c (Fst t)        = Fst (shift c t)
shift c (Snd t)        = Snd (shift c t)
shift c (Tensor t1 t2) = Tensor (shift c t1) (shift c t2)
shift c (LetPair t1 t2) = LetPair (shift c t1) (shift (S (S c)) t2)
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
substAt j u (With t1 t2)   = With (substAt j u t1) (substAt j u t2)
substAt j u (Fst t)        = Fst (substAt j u t)
substAt j u (Snd t)        = Snd (substAt j u t)
substAt j u (Tensor t1 t2) = Tensor (substAt j u t1) (substAt j u t2)
substAt j u (LetPair t1 t2) =
  LetPair (substAt j u t1) (substAt (S (S j)) (shift 0 (shift 0 u)) t2)
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

||| Two-variable substitution for the let-pair eliminator: replace de
||| Bruijn index 1 by `u1` and index 0 by `u2`, as two sequential single
||| substitutions. The inner pass substitutes index 0 by `u2`; because the
||| index-1 binder is still present during that pass, `u2`'s free variables
||| must skip it, so `u2` is pre-shifted with `shift 0`. The outer pass then
||| substitutes `u1` for the (now index-0) former index-1 binder. Correct for
||| OPEN `u1` `u2` (needed by preservation, where the tensor components are
||| typed in a non-empty context); for CLOSED `u1` `u2` the `shift 0` is the
||| identity. Mirrors `subst2` in the Coq twin.
public export
subst2 : Tm -> Tm -> Tm -> Tm
subst2 u1 u2 t = subst0 u1 (subst0 (shift 0 u2) t)
