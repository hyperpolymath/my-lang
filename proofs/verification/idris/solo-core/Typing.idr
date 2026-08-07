-- SPDX-License-Identifier: MPL-2.0
-- SPDX-FileCopyrightText: 2026 Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk>

--
-- QTT typing judgement for the Solo core of my-lang, over the
-- SEPARATED context (ADR-006). Coq twin: `has_var` / `has_type` in
-- SoloCore.v.
--
-- The judgement `Has g d t a` means: term `t` has type `a` in TYPE
-- context `g` under USAGE vector `d`, with `d`'s quantities exactly
-- accounting for the uses of each bound variable inside `t`. Every
-- rule SHARES the type context `g`; the usage vector `d` is what adds
-- up (App/Tensor/LetPair/Case/Let) or scales (App/Let).
--
-- TWO PRODUCTS (ADR-007):
--   * additive `&` (THWith): both components typed under the SAME usage
--     `d` (NOT split) — only one survives elimination (Fst/Snd), so
--     reusing a linear resource across the two components is safe.
--   * multiplicative `(X)` (THTensor): introduction SPLITS usage
--     (uadd d1 d2); eliminated by THLetPair, whose body binds two
--     variables (x@1, y@0) each used linearly.
--
-- Source of truth for the rule shapes:
--   * proofs/solo/affine-types/formal-system.md
--   * proofs/shared/type-system/typing-rules.md
--   * the Coq twin SoloCore.v (canonical, ADR D1)

module Typing

import Quantity
import EchoMode
import Syntax
import Context

%default total

------------------------------------------------------------
-- Variable lookup
------------------------------------------------------------

||| `HasVar g d n a` says the de Bruijn variable `n` has type `a` in
||| type context `g`, where the usage `d` assigns quantity `One` at
||| position `n` and `Zero` everywhere else. This is T-Var in QTT:
||| "using a variable once uses it exactly once, and uses nothing else".
public export
data HasVar : Tctx -> Uvec -> Nat -> Ty -> Type where
  HVHere  : HasVar (TSnoc g a) (USnoc (uzero g) One) Z a
  HVThere : HasVar g d n a
         -> HasVar (TSnoc g b) (USnoc d Zero) (S n) a

------------------------------------------------------------
-- The typing judgement
------------------------------------------------------------

||| `Has g d t a` — in type context `g` under usage `d`, term `t` has
||| type `a`. Usage splitting is explicit at THApp, THTensor, THLetPair,
||| THCase and THLet. THWith shares usage (additive). THLam introduces a
||| binder with a declared quantity.
public export
data Has : Tctx -> Uvec -> Tm -> Ty -> Type where

  ||| T-Var: a variable use consumes quantity One at its position.
  THVar : HasVar g d n a
       -> Has g d (Var n) a

  ||| T-Unit: the unit term types under the all-zero usage.
  THUnit : Has g (uzero g) UnitT TUnit

  ||| T-Lam: introduce a binding with quantity `q` and type `a`, type
  ||| the body in the extended context. The arrow records the quantity.
  THLam : Has (TSnoc g a) (USnoc d q) t b
       -> Has g d (Lam q a t) (TArr q a b)

  ||| T-App: function typed under `d1`, argument under `d2` scaled by
  ||| the parameter quantity `q`, whole application under `d1 + q*d2`.
  |||
  ||| The split usages `d1`, `d2` and the scaling quantity `q` are
  ||| explicit fields: the derivation *records its usage split*. (In
  ||| QTT-as-Idris this keeps them runtime-relevant, which the
  ||| `progress`/`preservation` inversions need; the Coq twin leaves
  ||| them as `Prop` binders since `Prop` has no erasure — ADR-003.)
  THApp : (d1, d2 : Uvec) -> (q : Q)
       -> Has g d1 t1 (TArr q a b)
       -> Has g d2 t2 a
       -> uadd d1 (uscale q d2) = Just d
       -> Has g d (App t1 t2) b

  ||| T-With: ADDITIVE product introduction. Both components typed under
  ||| the SAME usage `d` — not split. The genuine `&`.
  THWith : Has g d t1 a
        -> Has g d t2 b
        -> Has g d (With t1 t2) (TWith a b)

  ||| T-Fst / T-Snd: projection from an additive pair (no split).
  THFst : Has g d t (TWith a b)
       -> Has g d (Fst t) a
  THSnd : Has g d t (TWith a b)
       -> Has g d (Snd t) b

  ||| T-Tensor: MULTIPLICATIVE product introduction SPLITS usage
  ||| (uadd d1 d2): both halves are paid for separately.
  THTensor : (d1, d2 : Uvec)
          -> Has g d1 t1 a
          -> Has g d2 t2 b
          -> uadd d1 d2 = Just d
          -> Has g d (Tensor t1 t2) (TTensor a b)

  ||| T-LetPair: eliminate a tensor. The body `t2` binds two variables —
  ||| x:a at de Bruijn index 1, y:b at index 0 — each used linearly (One).
  THLetPair : (d1, d2 : Uvec) -> (a, b : Ty)
           -> Has g d1 t1 (TTensor a b)
           -> Has (TSnoc (TSnoc g a) b) (USnoc (USnoc d2 One) One) t2 c
           -> uadd d1 d2 = Just d
           -> Has g d (LetPair t1 t2) c

  ||| T-Inl / T-Inr: sum introduction carries the other summand
  ||| as the annotation, matching `Syntax.idr`.
  THInl : Has g d t a
       -> Has g d (Inl b t) (TSum a b)
  THInr : Has g d t b
       -> Has g d (Inr a t) (TSum a b)

  ||| T-Case: scrutinee typed under `d1`; each branch binds the injected
  ||| value with quantity One and is typed under `d2` extended with that
  ||| binder. Branches agree on `d2` and on the result type. The whole
  ||| `case` is typed under `d1 + d2`.
  THCase : (d1, d2 : Uvec) -> (a, b : Ty)
        -> Has g d1 t (TSum a b)
        -> Has (TSnoc g a) (USnoc d2 One) tL c
        -> Has (TSnoc g b) (USnoc d2 One) tR c
        -> uadd d1 d2 = Just d
        -> Has g d (Case t tL tR) c

  ||| T-Let: bind `t1` with declared quantity `q`. The RHS is typed
  ||| under `d1`, scaled by `q`, then added to `d2` (the body's usage).
  THLet : (d1, d2 : Uvec) -> (q : Q) -> (a : Ty)
       -> Has g d1 t1 a
       -> Has (TSnoc g a) (USnoc d2 q) t2 b
       -> uadd (uscale q d1) d2 = Just d
       -> Has g d (Let q t1 t2) b

  ||| T-Echo: introduce an echo residue retaining a witness `t : a`
  ||| of an admissible collapse `a => b` at mode `m`. The codomain
  ||| `b` is a phantom annotation (the non-dependent approximation,
  ||| echo-types-integration.md §1). Does not split the usage — it
  ||| only records what was kept, so `d` is threaded unchanged.
  THEcho : Has g d t a
        -> Has g d (MkEcho m a b t) (TEcho m a b)

  ||| T-Weaken: `EchoLinear.weaken`. A Linear echo may be weakened to
  ||| an Affine one; one-way (the reverse is barred —
  ||| `EchoMode.noSectionWeaken`). Usage `d` unchanged: weakening
  ||| spends no resources, it only drops a distinction.
  THWeaken : Has g d t (TEcho Linear a b)
          -> Has g d (Weaken t) (TEcho Affine a b)
