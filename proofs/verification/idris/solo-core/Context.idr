-- SPDX-License-Identifier: MPL-2.0
-- SPDX-FileCopyrightText: 2026 Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk>
--
-- QTT-style typing contexts for the Solo core of my-lang.
--
-- A context `Ctx` is a snoc-list of `(Ty, Q)` entries: each
-- bound variable carries both its type and its current available
-- quantity. Context addition `ctxAdd` adds quantities pointwise;
-- context scaling `ctxScale q g` multiplies every entry's
-- quantity by `q`.
--
-- The typing judgement in Typing.idr uses these operations to
-- SPLIT the input context across subterms in T-App, T-Pair,
-- T-Let and T-Case — the standard QTT context discipline that
-- makes affine/linear accounting work. This is the static view
-- the future `dialects/solo` checker (the post-hoc usage walk)
-- must agree with; the equivalence lemma is Track F1.4.

module Context

import Quantity
import Syntax

%default total

------------------------------------------------------------
-- Context representation
------------------------------------------------------------

||| A snoc-list context: `Empty` at the base, `Snoc g a q`
||| pushes a binding with type `a` and quantity `q` on top.
public export
data Ctx : Type where
  Empty : Ctx
  Snoc  : Ctx -> Ty -> Q -> Ctx

%name Ctx g, g1, g2, d

------------------------------------------------------------
-- Length (for well-scopedness conditions)
------------------------------------------------------------

public export
ctxLen : Ctx -> Nat
ctxLen Empty        = 0
ctxLen (Snoc g _ _) = S (ctxLen g)

------------------------------------------------------------
-- Pointwise quantity addition on contexts
------------------------------------------------------------

||| Pointwise-add two contexts. Defined only when the two
||| contexts have the same shape; on a mismatch we return
||| `Nothing`. In a well-typed derivation the shapes always
||| agree by construction.
public export
ctxAdd : Ctx -> Ctx -> Maybe Ctx
ctxAdd Empty Empty = Just Empty
ctxAdd (Snoc g1 a1 q1) (Snoc g2 a2 q2) =
  case ctxAdd g1 g2 of
    Nothing => Nothing
    Just g  =>
      -- Type equality is not checked here; Typing.idr only ever
      -- combines contexts whose shapes match by construction. A
      -- decidable `Eq Ty` would let us tighten this (Track F1.4).
      Just (Snoc g a1 (qAdd q1 q2))
ctxAdd _ _ = Nothing

------------------------------------------------------------
-- Scalar multiplication of contexts
------------------------------------------------------------

||| Multiply every quantity in the context by `q`. Used in
||| T-App (argument scaled by the parameter quantity) and T-Let.
public export
ctxScale : Q -> Ctx -> Ctx
ctxScale _ Empty             = Empty
ctxScale q (Snoc g a qEntry) = Snoc (ctxScale q g) a (qMul q qEntry)

------------------------------------------------------------
-- The all-zero context derived from a shape
------------------------------------------------------------

||| Given any context, produce the same-shape context with every
||| quantity replaced by `Zero` — the "nothing is used" context
||| that appears in T-Unit and as the non-consuming siblings of
||| T-Var.
public export
ctxZero : Ctx -> Ctx
ctxZero Empty        = Empty
ctxZero (Snoc g a _) = Snoc (ctxZero g) a Zero
