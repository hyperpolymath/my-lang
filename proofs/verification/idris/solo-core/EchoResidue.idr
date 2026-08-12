-- SPDX-License-Identifier: MPL-2.0
-- SPDX-FileCopyrightText: 2026 Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk>

--
-- Echo residue — proof-layer object for the my-lang Solo core.
--
-- "Slice 3" of docs/design/echo-types-integration.md: the proof-layer
-- residue object that the Rust checker (crates/my-lang/src/types.rs,
-- `Ty::Echo` + `EchoMode`) points at. It pins, against the *mechanised*
-- kernel, the facts the checker's Echo subtyping relies on:
--
--   * the linear->affine weakening is well-typed (kernel rule THWeaken)
--     and actually reduces (kernel rule SWeaken);
--   * Echo subtyping = "source mode weakens to target mode, domain &
--     codomain invariant" — exactly `Ty::is_assignable_from` for the
--     `Echo` case;
--   * weakening has *no section* (affine cannot be used where linear is
--     required) — EchoLinear.no-section.
--
-- Mirrors the echo-types Agda library module EchoLinear (`Mode`,
-- `_<=m_`, `weaken`, `no-section-weaken`), narrowed to the
-- R-2026-05-18 claim line: Echo is a loss-graded reindexing modality
-- over a thin poset (design note §6). Coq twin: EchoResidue.v.

module EchoResidue

import Quantity
import EchoMode
import Syntax
import Context
import Typing
import Soundness

%default total

------------------------------------------------------------
-- 1. The weakening is well-typed and reduces (kernel-backed)
------------------------------------------------------------

||| Typed weaken: a Linear echo over `a => b` weakens to an Affine one
||| in the SAME context — no resources are spent, only a distinction is
||| dropped. (= kernel rule `THWeaken`; `EchoLinear.weaken`.)
public export
echoWeakenTyped : Has g d t (TEcho Linear a b) -> Has g d (Weaken t) (TEcho Affine a b)
echoWeakenTyped = THWeaken

||| Operational weaken: once the residue is a value, `Weaken` takes the
||| Linear echo to the corresponding Affine echo. (= kernel rule
||| `SWeaken`.)
public export
echoWeakenStep : Value v -> Step (Weaken (MkEcho Linear a b v)) (MkEcho Affine a b v)
echoWeakenStep = SWeaken

||| The weaken preserves the retained residue — the affine echo carries
||| exactly the same witness `v`. Loss that is not total erasure.
public export
echoWeakenKeepsResidue : (a, b : Ty) -> (v : Tm) -> Value v
  -> (w : Tm ** (Step (Weaken (MkEcho Linear a b v)) (MkEcho Affine a b w), w = v))
echoWeakenKeepsResidue a b v pv = (v ** (SWeaken pv, Refl))

------------------------------------------------------------
-- 2. Echo subtyping — the formal backing of Ty::is_assignable_from
------------------------------------------------------------
--
-- The Rust checker accepts a source echo where a target echo is
-- expected exactly when `mle source_mode target_mode` holds and the
-- domain/codomain agree. We name that relation and prove the laws the
-- five Rust unit tests assert.

public export
EchoAssignable : (tgtMode, srcMode : Mode)
              -> (tgtDom, tgtCod, srcDom, srcCod : Ty) -> Type
EchoAssignable tgtMode srcMode tgtDom tgtCod srcDom srcCod =
  (mle srcMode tgtMode = True, tgtDom = srcDom, tgtCod = srcCod)

||| Reflexivity: any echo is assignable to one of the same shape.
public export
echoAssignableRefl : (m : Mode) -> (a, b : Ty) -> EchoAssignable m m a b a b
echoAssignableRefl m a b = (mleRefl m, Refl, Refl)

||| `EchoLinear.weaken`: a Linear echo is assignable where an Affine
||| echo is expected. (target = Affine, source = Linear.)
public export
echoAssignableWeaken : (a, b : Ty) -> EchoAssignable Affine Linear a b a b
echoAssignableWeaken a b = (weakenLinearAffine, Refl, Refl)

||| `EchoLinear.no-section-weaken`: an Affine echo is NOT assignable
||| where a Linear echo is required. Weakening is one-way; it has no
||| section. (target = Linear, source = Affine.)
public export
echoAssignableNoSection : (a, b : Ty) -> Not (EchoAssignable Linear Affine a b a b)
echoAssignableNoSection a b (Refl, _, _) impossible

||| Domain / codomain are invariant under echo subtyping.
public export
echoAssignableDomCodInvariant
  : EchoAssignable mt ms dt ct ds cs -> (dt = ds, ct = cs)
echoAssignableDomCodInvariant (_, hd, hc) = (hd, hc)

||| Transitivity — assignability composes (the thin poset is a
||| preorder), so a chain of weakenings is itself a weakening.
public export
echoAssignableTrans : (m1, m2, m3 : Mode) -> (a, b : Ty)
  -> EchoAssignable m2 m1 a b a b
  -> EchoAssignable m3 m2 a b a b
  -> EchoAssignable m3 m1 a b a b
echoAssignableTrans m1 m2 m3 a b (h12, _, _) (h23, _, _) =
  (mleTrans m1 m2 m3 h12 h23, Refl, Refl)

------------------------------------------------------------
-- 3. Coherence: the typed kernel weaken realises echo subtyping
------------------------------------------------------------

||| The kernel's `Weaken` term turns a `TEcho Linear a b` into a
||| `TEcho Affine a b` — i.e. it inhabits exactly the assignability
||| relation `EchoAssignable Affine Linear`. The proof-relevant residue
||| and the type-algebra subtyping agree.
public export
weakenRealisesAssignable : {a, b : Ty} -> Has g d t (TEcho Linear a b)
  -> (Has g d (Weaken t) (TEcho Affine a b), EchoAssignable Affine Linear a b a b)
weakenRealisesAssignable d = (echoWeakenTyped d, echoAssignableWeaken a b)
