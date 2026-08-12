-- SPDX-License-Identifier: MPL-2.0
-- SPDX-FileCopyrightText: 2026 Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk>

--
-- Quantity semiring for the my-lang Solo core.
--
-- This is the QTT three-point semiring {0, 1, omega} that
-- underpins my-lang's *solo* dialect (affine types + arena
-- allocation; see `dialects/solo/` and the paper proof in
-- `proofs/solo/affine-types/formal-system.md`).
--
-- AT-MOST-ONCE (affine `?`) is a planned refinement of this
-- core — see proofs/ALIGNMENT-PLAN.md, Track F1.2. Affine
-- weakening is *currently* modelled through the `qLe` ordering
-- (`Zero` may substitute where `One` is expected), which lets a
-- linear resource go unused; the dedicated four-point affine
-- element is the next mechanisation step.
--
-- This module MUST mirror the eventual `dialects/solo` quantity
-- checker (Rust). Today that checker is `TODO(#typeck)`, so this
-- formal model LEADS the implementation: it is the spec the
-- checker must satisfy.
--
-- The semiring tables:
--
--    +  |  0   1   omega            *  |  0   1   omega
--   ----+----------------          ----+----------------
--    0  |  0   1   omega            0  |  0   0   0
--    1  |  1   omega omega          1  |  0   1   omega
--  omega| omega omega omega        omega|  0   omega omega
--
-- Ordering (qLe q1 q2 = True iff q1 can substitute where q2 is
-- expected): 0 <= q for all q; q <= omega for all q; 1 <= 1.

module Quantity

%default total

||| The three-point quantity semiring from Quantitative Type
||| Theory.
|||
||| * `Zero`  — erased, compile-time only (irrelevant `0`).
||| * `One`   — linear, used exactly once at runtime.
||| * `Omega` — unrestricted.
public export
data Q : Type where
  Zero  : Q
  One   : Q
  Omega : Q

%name Q q, q1, q2, q3

public export
Eq Q where
  Zero  == Zero  = True
  One   == One   = True
  Omega == Omega = True
  _     == _     = False

------------------------------------------------------------
-- Semiring operations
------------------------------------------------------------

||| Addition in the quantity semiring. Note `One + One = Omega`:
||| the domain is three-point, not the naturals.
public export
qAdd : Q -> Q -> Q
qAdd Zero  q     = q
qAdd One   Zero  = One
qAdd One   One   = Omega
qAdd One   Omega = Omega
qAdd Omega _     = Omega

||| Multiplication in the quantity semiring.
public export
qMul : Q -> Q -> Q
qMul Zero  _     = Zero
qMul One   q     = q
qMul Omega Zero  = Zero
qMul Omega One   = Omega
qMul Omega Omega = Omega

||| Subquantity ordering. `qLe q1 q2` is `True` when a value with
||| quantity `q1` can be used where `q2` is expected (more
||| restricted may substitute for less restricted). This is also
||| where affine *weakening* lives for the Solo core: `Zero`
||| substitutes for `One`, so a linear resource may be dropped.
public export
qLe : Q -> Q -> Bool
qLe Zero _       = True
qLe _    Omega   = True
qLe One  One     = True
qLe One  Zero    = False
qLe Omega Zero   = False
qLe Omega One    = False

------------------------------------------------------------
-- Semiring laws (exhaustive case split — no axioms)
------------------------------------------------------------

||| `Zero` is the left identity of addition.
public export
qAddZeroL : (q : Q) -> qAdd Zero q = q
qAddZeroL Zero  = Refl
qAddZeroL One   = Refl
qAddZeroL Omega = Refl

||| `Zero` is the right identity of addition.
public export
qAddZeroR : (q : Q) -> qAdd q Zero = q
qAddZeroR Zero  = Refl
qAddZeroR One   = Refl
qAddZeroR Omega = Refl

||| Addition is commutative.
public export
qAddComm : (q1, q2 : Q) -> qAdd q1 q2 = qAdd q2 q1
qAddComm Zero  Zero  = Refl
qAddComm Zero  One   = Refl
qAddComm Zero  Omega = Refl
qAddComm One   Zero  = Refl
qAddComm One   One   = Refl
qAddComm One   Omega = Refl
qAddComm Omega Zero  = Refl
qAddComm Omega One   = Refl
qAddComm Omega Omega = Refl

||| Addition is associative.
public export
qAddAssoc : (q1, q2, q3 : Q)
         -> qAdd (qAdd q1 q2) q3 = qAdd q1 (qAdd q2 q3)
qAddAssoc Zero  Zero  Zero  = Refl
qAddAssoc Zero  Zero  One   = Refl
qAddAssoc Zero  Zero  Omega = Refl
qAddAssoc Zero  One   Zero  = Refl
qAddAssoc Zero  One   One   = Refl
qAddAssoc Zero  One   Omega = Refl
qAddAssoc Zero  Omega Zero  = Refl
qAddAssoc Zero  Omega One   = Refl
qAddAssoc Zero  Omega Omega = Refl
qAddAssoc One   Zero  Zero  = Refl
qAddAssoc One   Zero  One   = Refl
qAddAssoc One   Zero  Omega = Refl
qAddAssoc One   One   Zero  = Refl
qAddAssoc One   One   One   = Refl
qAddAssoc One   One   Omega = Refl
qAddAssoc One   Omega Zero  = Refl
qAddAssoc One   Omega One   = Refl
qAddAssoc One   Omega Omega = Refl
qAddAssoc Omega Zero  Zero  = Refl
qAddAssoc Omega Zero  One   = Refl
qAddAssoc Omega Zero  Omega = Refl
qAddAssoc Omega One   Zero  = Refl
qAddAssoc Omega One   One   = Refl
qAddAssoc Omega One   Omega = Refl
qAddAssoc Omega Omega Zero  = Refl
qAddAssoc Omega Omega One   = Refl
qAddAssoc Omega Omega Omega = Refl

||| `Zero` is the left absorbing element of multiplication.
public export
qMulZeroL : (q : Q) -> qMul Zero q = Zero
qMulZeroL Zero  = Refl
qMulZeroL One   = Refl
qMulZeroL Omega = Refl

||| `Zero` is the right absorbing element of multiplication.
public export
qMulZeroR : (q : Q) -> qMul q Zero = Zero
qMulZeroR Zero  = Refl
qMulZeroR One   = Refl
qMulZeroR Omega = Refl

||| `One` is the left identity of multiplication.
public export
qMulOneL : (q : Q) -> qMul One q = q
qMulOneL Zero  = Refl
qMulOneL One   = Refl
qMulOneL Omega = Refl

||| `One` is the right identity of multiplication.
public export
qMulOneR : (q : Q) -> qMul q One = q
qMulOneR Zero  = Refl
qMulOneR One   = Refl
qMulOneR Omega = Refl

||| Multiplication is commutative.
public export
qMulComm : (q1, q2 : Q) -> qMul q1 q2 = qMul q2 q1
qMulComm Zero  Zero  = Refl
qMulComm Zero  One   = Refl
qMulComm Zero  Omega = Refl
qMulComm One   Zero  = Refl
qMulComm One   One   = Refl
qMulComm One   Omega = Refl
qMulComm Omega Zero  = Refl
qMulComm Omega One   = Refl
qMulComm Omega Omega = Refl

||| Multiplication is associative.
public export
qMulAssoc : (q1, q2, q3 : Q)
         -> qMul (qMul q1 q2) q3 = qMul q1 (qMul q2 q3)
qMulAssoc Zero  Zero  Zero  = Refl
qMulAssoc Zero  Zero  One   = Refl
qMulAssoc Zero  Zero  Omega = Refl
qMulAssoc Zero  One   Zero  = Refl
qMulAssoc Zero  One   One   = Refl
qMulAssoc Zero  One   Omega = Refl
qMulAssoc Zero  Omega Zero  = Refl
qMulAssoc Zero  Omega One   = Refl
qMulAssoc Zero  Omega Omega = Refl
qMulAssoc One   Zero  Zero  = Refl
qMulAssoc One   Zero  One   = Refl
qMulAssoc One   Zero  Omega = Refl
qMulAssoc One   One   Zero  = Refl
qMulAssoc One   One   One   = Refl
qMulAssoc One   One   Omega = Refl
qMulAssoc One   Omega Zero  = Refl
qMulAssoc One   Omega One   = Refl
qMulAssoc One   Omega Omega = Refl
qMulAssoc Omega Zero  Zero  = Refl
qMulAssoc Omega Zero  One   = Refl
qMulAssoc Omega Zero  Omega = Refl
qMulAssoc Omega One   Zero  = Refl
qMulAssoc Omega One   One   = Refl
qMulAssoc Omega One   Omega = Refl
qMulAssoc Omega Omega Zero  = Refl
qMulAssoc Omega Omega One   = Refl
qMulAssoc Omega Omega Omega = Refl

||| Left distributivity: q1 * (q2 + q3) = q1*q2 + q1*q3.
public export
qMulDistribL : (q1, q2, q3 : Q)
            -> qMul q1 (qAdd q2 q3) = qAdd (qMul q1 q2) (qMul q1 q3)
qMulDistribL Zero  Zero  Zero  = Refl
qMulDistribL Zero  Zero  One   = Refl
qMulDistribL Zero  Zero  Omega = Refl
qMulDistribL Zero  One   Zero  = Refl
qMulDistribL Zero  One   One   = Refl
qMulDistribL Zero  One   Omega = Refl
qMulDistribL Zero  Omega Zero  = Refl
qMulDistribL Zero  Omega One   = Refl
qMulDistribL Zero  Omega Omega = Refl
qMulDistribL One   Zero  Zero  = Refl
qMulDistribL One   Zero  One   = Refl
qMulDistribL One   Zero  Omega = Refl
qMulDistribL One   One   Zero  = Refl
qMulDistribL One   One   One   = Refl
qMulDistribL One   One   Omega = Refl
qMulDistribL One   Omega Zero  = Refl
qMulDistribL One   Omega One   = Refl
qMulDistribL One   Omega Omega = Refl
qMulDistribL Omega Zero  Zero  = Refl
qMulDistribL Omega Zero  One   = Refl
qMulDistribL Omega Zero  Omega = Refl
qMulDistribL Omega One   Zero  = Refl
qMulDistribL Omega One   One   = Refl
qMulDistribL Omega One   Omega = Refl
qMulDistribL Omega Omega Zero  = Refl
qMulDistribL Omega Omega One   = Refl
qMulDistribL Omega Omega Omega = Refl

||| Right distributivity: (q1 + q2) * q3 = q1*q3 + q2*q3.
public export
qMulDistribR : (q1, q2, q3 : Q)
            -> qMul (qAdd q1 q2) q3 = qAdd (qMul q1 q3) (qMul q2 q3)
qMulDistribR Zero  Zero  Zero  = Refl
qMulDistribR Zero  Zero  One   = Refl
qMulDistribR Zero  Zero  Omega = Refl
qMulDistribR Zero  One   Zero  = Refl
qMulDistribR Zero  One   One   = Refl
qMulDistribR Zero  One   Omega = Refl
qMulDistribR Zero  Omega Zero  = Refl
qMulDistribR Zero  Omega One   = Refl
qMulDistribR Zero  Omega Omega = Refl
qMulDistribR One   Zero  Zero  = Refl
qMulDistribR One   Zero  One   = Refl
qMulDistribR One   Zero  Omega = Refl
qMulDistribR One   One   Zero  = Refl
qMulDistribR One   One   One   = Refl
qMulDistribR One   One   Omega = Refl
qMulDistribR One   Omega Zero  = Refl
qMulDistribR One   Omega One   = Refl
qMulDistribR One   Omega Omega = Refl
qMulDistribR Omega Zero  Zero  = Refl
qMulDistribR Omega Zero  One   = Refl
qMulDistribR Omega Zero  Omega = Refl
qMulDistribR Omega One   Zero  = Refl
qMulDistribR Omega One   One   = Refl
qMulDistribR Omega One   Omega = Refl
qMulDistribR Omega Omega Zero  = Refl
qMulDistribR Omega Omega One   = Refl
qMulDistribR Omega Omega Omega = Refl

------------------------------------------------------------
-- Ordering sanity
------------------------------------------------------------

||| Zero is below everything.
public export
qLeZero : (q : Q) -> qLe Zero q = True
qLeZero Zero  = Refl
qLeZero One   = Refl
qLeZero Omega = Refl

||| Omega is above everything.
public export
qLeOmega : (q : Q) -> qLe q Omega = True
qLeOmega Zero  = Refl
qLeOmega One   = Refl
qLeOmega Omega = Refl

||| Reflexivity of the ordering.
public export
qLeRefl : (q : Q) -> qLe q q = True
qLeRefl Zero  = Refl
qLeRefl One   = Refl
qLeRefl Omega = Refl
