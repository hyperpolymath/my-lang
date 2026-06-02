-- SPDX-License-Identifier: MPL-2.0
-- SPDX-FileCopyrightText: 2026 Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk>
--
-- QTT typing judgement for the Solo core of my-lang.
--
-- The judgement `Has g t a` means: term `t` has type `a` in
-- context `g`, with `g`'s quantities exactly accounting for the
-- uses of each bound variable inside `t`.
--
-- These are the RULES, stated as data constructors. The meta-
-- theorems (progress, preservation, affine preservation) are
-- stated in `Soundness.idr` and proved case-by-case — see
-- proofs/ALIGNMENT-PLAN.md, Track F1.
--
-- Source of truth for the rule shapes:
--   * proofs/solo/affine-types/formal-system.md
--   * proofs/shared/type-system/typing-rules.md
--
-- We take the standard QTT presentation with explicit context
-- splitting (ctxAdd / ctxScale) — the formulation for which
-- progress + preservation are provable, and which the eventual
-- `dialects/solo` usage-walk checker must agree with.

module Typing

import Quantity
import Syntax
import Context

%default total

------------------------------------------------------------
-- Variable lookup
------------------------------------------------------------

||| `HasVar g n a` says the de Bruijn variable `n` has type `a`
||| in `g`, where `g` assigns quantity `One` at position `n` and
||| `Zero` everywhere else. This is T-Var in QTT: "using a
||| variable once uses it exactly once, and uses nothing else".
public export
data HasVar : Ctx -> Nat -> Ty -> Type where
  HVHere  : HasVar (Snoc (ctxZero g) a One) Z a
  HVThere : HasVar g n a
         -> HasVar (Snoc g b Zero) (S n) a

------------------------------------------------------------
-- The typing judgement
------------------------------------------------------------

||| `Has g t a` — in QTT context `g`, term `t` has type `a`.
||| Context splitting is explicit at T-App, T-Pair, T-Let and
||| T-Case. T-Lam introduces a binder with a declared quantity.
public export
data Has : Ctx -> Tm -> Ty -> Type where

  ||| T-Var: a variable use consumes quantity One at its position.
  THVar : HasVar g n a
       -> Has g (Var n) a

  ||| T-Unit: the unit term types in the all-zero context.
  THUnit : Has (ctxZero g) UnitT TUnit

  ||| T-Lam: introduce a binding with quantity `q` and type `a`,
  ||| type the body in the extended context. The resulting
  ||| arrow `TArr q a b` records the quantity.
  THLam : Has (Snoc g a q) t b
       -> Has g (Lam q a t) (TArr q a b)

  ||| T-App: function typed in `g1`, argument in `g2` scaled by
  ||| the parameter quantity `q`, whole application in the
  ||| pointwise sum `g1 + q * g2`.
  THApp : Has g1 t1 (TArr q a b)
       -> Has g2 t2 a
       -> ctxAdd g1 (ctxScale q g2) = Just g
       -> Has g (App t1 t2) b

  ||| T-Pair: product introduction splits the context additively.
  THPair : Has g1 t1 a
        -> Has g2 t2 b
        -> ctxAdd g1 g2 = Just g
        -> Has g (Pair t1 t2) (TPair a b)

  ||| T-Fst / T-Snd: projection is a single subterm (no split).
  THFst : Has g t (TPair a b)
       -> Has g (Fst t) a
  THSnd : Has g t (TPair a b)
       -> Has g (Snd t) b

  ||| T-Inl / T-Inr: sum introduction carries the other summand
  ||| as the annotation, matching `Syntax.idr`.
  THInl : Has g t a
       -> Has g (Inl b t) (TSum a b)
  THInr : Has g t b
       -> Has g (Inr a t) (TSum a b)

  ||| T-Case: scrutinee typed in `g1`; each branch binds the
  ||| injected value with quantity One and is typed in `g2`
  ||| extended with that binder. Branches agree on `g2` and on
  ||| the result type. The whole `case` is typed in `g1 + g2`.
  THCase : Has g1 t (TSum a b)
        -> Has (Snoc g2 a One) tL c
        -> Has (Snoc g2 b One) tR c
        -> ctxAdd g1 g2 = Just g
        -> Has g (Case t tL tR) c

  ||| T-Let: bind `t1` with declared quantity `q`. The RHS is
  ||| typed in `g1`, scaled by `q`, then added to `g2` (the
  ||| body's context).
  THLet : Has g1 t1 a
       -> Has (Snoc g2 a q) t2 b
       -> ctxAdd (ctxScale q g1) g2 = Just g
       -> Has g (Let q t1 t2) b
