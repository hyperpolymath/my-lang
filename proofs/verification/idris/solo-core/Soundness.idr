-- SPDX-License-Identifier: MPL-2.0
-- SPDX-FileCopyrightText: 2026 Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk>
--
-- Statements of the soundness theorems for the my-lang Solo core.
--
-- This file deliberately contains NO PROOFS yet. The Track F1
-- week-1-2 deliverable is the *statement* of progress and
-- preservation; the derivations are filled case-by-case in the
-- following weeks (see proofs/ALIGNMENT-PLAN.md). All theorems
-- are left as explicit `?todo_...` holes so the module still
-- typechecks as a declaration unit.
--
-- This mirrors the methodology AffineScript used for its own
-- solo-core soundness module, so the two flagship languages
-- track the same metatheory shape.

module Soundness

import Quantity
import Syntax
import Context
import Typing

%default total

------------------------------------------------------------
-- Values
------------------------------------------------------------

||| Solo values: canonical forms of closed terms.
public export
data Value : Tm -> Type where
  VUnit : Value UnitT
  VLam  : Value (Lam q a t)
  VPair : Value t1 -> Value t2 -> Value (Pair t1 t2)
  VInl  : Value t -> Value (Inl b t)
  VInr  : Value t -> Value (Inr a t)

------------------------------------------------------------
-- Small-step reduction (declaration only)
------------------------------------------------------------
--
-- We declare `Step t t'` as a data family but do NOT enumerate
-- its constructors yet. They will follow call-by-value,
-- left-to-right beta / projection / case reduction, mirroring
-- the reference interpreter the solo dialect will grow, and are
-- introduced alongside the progress / preservation proofs.

public export
data Step : Tm -> Tm -> Type where
  -- Constructors intentionally omitted until the operational
  -- semantics is committed (Track F1.3). Keeping them open lets
  -- the *statements* below typecheck without prematurely fixing
  -- the reduction strategy.

------------------------------------------------------------
-- Existential wrapper
------------------------------------------------------------

||| Simple Sigma to avoid importing `Data.DPair` right now.
public export
data StepsTo : Tm -> Type where
  MkStepsTo : (t' : Tm) -> Step t t' -> StepsTo t

------------------------------------------------------------
-- Progress
------------------------------------------------------------

||| Progress: a closed, well-typed Solo term is either a value
||| or can take a step.
|||
||| "Closed" means typed in the empty context — there are no free
||| de Bruijn indices because `Empty` has no `HVHere` / `HVThere`
||| inhabitants.
public export
progress : Has Empty t a -> Either (Value t) (StepsTo t)
progress _ = ?todo_progress

------------------------------------------------------------
-- Preservation
------------------------------------------------------------

||| Preservation: reduction preserves typing in the SAME context.
||| Preserving the context (not merely "some g'") is the affine-
||| accounting content of the theorem — a reduct that duplicated
||| a linear variable would require a *larger* context.
public export
preservation : Has g t a -> Step t t' -> Has g t' a
preservation _ _ = ?todo_preservation

------------------------------------------------------------
-- Affine preservation (corollary)
------------------------------------------------------------

||| Affine preservation: if a term is well-typed with every
||| binding at quantity `One` or `Zero` and it steps, the reduct
||| is still well-typed in the same context. For the Solo kernel
||| this is a direct corollary of `preservation` (the preserved
||| context already carries the quantity accounting); stated here
||| for documentation.
public export
affinePreservation : Has g t a -> Step t t' -> Has g t' a
affinePreservation = preservation
