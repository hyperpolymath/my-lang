-- SPDX-License-Identifier: MPL-2.0
-- SPDX-FileCopyrightText: 2026 Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk>
--
-- Operational semantics + soundness for the my-lang Solo core.
--
--   * Phase F1.1 — the call-by-value small-step relation `Step` is
--     COMMITTED (constructors below), including the echo residue
--     rules.
--   * Phase F1.3 — `progress` is DISCHARGED as a real total function
--     (no `?todo` holes, no `believe_me`/`assert_total`/`postulate`).
--   * Phase F1.4 — `preservation` remains a typed hole
--     (`?todo_preservation`): its QTT substitution lemma is the
--     outstanding obligation, recorded in proofs/STATUS.md. We never
--     describe a hole as proved.
--
-- Coq twin: Soundness.v (where progress is a `Qed` theorem and
-- preservation a named `Prop`).

module Soundness

import Quantity
import EchoMode
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
  -- an echo with a fully-evaluated residue is a value; `Weaken` is
  -- not (it is the elimination that drives the linear->affine step)
  VEcho : Value t -> Value (MkEcho m a b t)

------------------------------------------------------------
-- Small-step reduction — call-by-value, left-to-right (F1.1)
------------------------------------------------------------
--
-- Computation rules fire once their arguments are values; congruence
-- rules thread evaluation left-to-right. Mirrors `step` in Soundness.v
-- and the reference interpreter the solo dialect will grow.

public export
data Step : Tm -> Tm -> Type where
  -- computation
  SApp   : Value v -> Step (App (Lam q a t) v) (subst0 v t)
  SFst   : Value v1 -> Value v2 -> Step (Fst (Pair v1 v2)) v1
  SSnd   : Value v1 -> Value v2 -> Step (Snd (Pair v1 v2)) v2
  SCaseL : Value v -> Step (Case (Inl b v) tL tR) (subst0 v tL)
  SCaseR : Value v -> Step (Case (Inr a v) tL tR) (subst0 v tR)
  SLet   : Value v -> Step (Let q v t2) (subst0 v t2)
  -- congruence (left-to-right, CBV)
  SApp1  : Step t1 t1' -> Step (App t1 t2) (App t1' t2)
  SApp2  : Value v1 -> Step t2 t2' -> Step (App v1 t2) (App v1 t2')
  SPair1 : Step t1 t1' -> Step (Pair t1 t2) (Pair t1' t2)
  SPair2 : Value v1 -> Step t2 t2' -> Step (Pair v1 t2) (Pair v1 t2')
  SFst1  : Step t t' -> Step (Fst t) (Fst t')
  SSnd1  : Step t t' -> Step (Snd t) (Snd t')
  SInl1  : Step t t' -> Step (Inl b t) (Inl b t')
  SInr1  : Step t t' -> Step (Inr a t) (Inr a t')
  SCase1 : Step t t' -> Step (Case t tL tR) (Case t' tL tR)
  SLet1  : Step t1 t1' -> Step (Let q t1 t2) (Let q t1' t2)
  -- echo residue: evaluate inside, and the one-way linear->affine
  -- weakening fires once the residue is a value (EchoLinear.weaken)
  SEcho1   : Step t t' -> Step (MkEcho m a b t) (MkEcho m a b t')
  SWeaken1 : Step t t' -> Step (Weaken t) (Weaken t')
  SWeaken  : Value v -> Step (Weaken (MkEcho Linear a b v)) (MkEcho Affine a b v)

------------------------------------------------------------
-- Existential wrapper
------------------------------------------------------------

||| Simple Sigma to avoid importing `Data.DPair` right now.
public export
data StepsTo : Tm -> Type where
  MkStepsTo : (t' : Tm) -> Step t t' -> StepsTo t

------------------------------------------------------------
-- Context lemmas for closed terms
------------------------------------------------------------
--
-- The empty context splits only trivially: a sum is `Empty` only when
-- both summands are, and a scaling is `Empty` only when its argument
-- is. These recover `Empty` sub-contexts so the recursive calls (which
-- speak about `Empty`) apply. Matching the returned `Refl` refines the
-- sub-derivation's context in place — it stays a structural subterm,
-- so the `progress` recursion is accepted as total.

-- Helpers take the relevant data EXPLICITLY (the context for the
-- splitting lemmas, the term for the canonical-forms lemmas). The QTT
-- multiplicities make the context / term indices of `Has` and `Value`
-- erased; pattern-matching on them or returning their pieces needs a
-- runtime-relevant copy, which the explicit argument provides.

noVarEmpty : HasVar Empty n a -> Void
noVarEmpty HVHere impossible
noVarEmpty (HVThere _) impossible

ctxScaleEmpty : (g : Ctx) -> ctxScale q g = Empty -> g = Empty
ctxScaleEmpty Empty _ = Refl
ctxScaleEmpty (Snoc _ _ _) Refl impossible

ctxAddEmpty : (g1, g2 : Ctx) -> ctxAdd g1 g2 = Just Empty -> (g1 = Empty, g2 = Empty)
ctxAddEmpty Empty Empty _ = (Refl, Refl)
ctxAddEmpty Empty (Snoc _ _ _) Refl impossible
ctxAddEmpty (Snoc _ _ _) Empty Refl impossible
ctxAddEmpty (Snoc x y z) (Snoc w v u) prf with (ctxAdd x w)
  ctxAddEmpty (Snoc x y z) (Snoc w v u) Refl | Nothing impossible
  ctxAddEmpty (Snoc x y z) (Snoc w v u) Refl | (Just gg) impossible

||| Coerce a derivation along a context equality. Used to retype the
||| immediate subterms in `Empty` once the splitting lemmas have shown
||| their contexts are `Empty`. (`progress` recurses on the explicit
||| term, so a coerced — non-subterm — derivation is fine for totality.)
coeCtx : g = g' -> Has g t a -> Has g' t a
coeCtx Refl d = d

------------------------------------------------------------
-- Canonical forms: a closed value's shape is fixed by its type
------------------------------------------------------------

canonArr : (v : Tm) -> Value v -> Has Empty v (TArr q a b)
        -> (q' : Q ** a' : Ty ** body : Tm ** v = Lam q' a' body)
canonArr (Lam q' a' body) VLam _ = (q' ** a' ** body ** Refl)
canonArr UnitT VUnit THUnit impossible
canonArr (Pair _ _) (VPair _ _) (THPair _ _ _ _ _) impossible
canonArr (Inl _ _) (VInl _) (THInl _) impossible
canonArr (Inr _ _) (VInr _) (THInr _) impossible
canonArr (MkEcho _ _ _ _) (VEcho _) (THEcho _) impossible

canonPair : (v : Tm) -> Value v -> Has Empty v (TPair a b)
         -> (t1 : Tm ** t2 : Tm ** (v = Pair t1 t2, Value t1, Value t2))
canonPair (Pair t1 t2) (VPair p1 p2) _ = (t1 ** t2 ** (Refl, p1, p2))
canonPair UnitT VUnit THUnit impossible
canonPair (Lam _ _ _) VLam (THLam _) impossible
canonPair (Inl _ _) (VInl _) (THInl _) impossible
canonPair (Inr _ _) (VInr _) (THInr _) impossible
canonPair (MkEcho _ _ _ _) (VEcho _) (THEcho _) impossible

canonSum : (v : Tm) -> Value v -> Has Empty v (TSum a b)
        -> Either (bb : Ty ** v' : Tm ** (v = Inl bb v', Value v'))
                  (aa : Ty ** v' : Tm ** (v = Inr aa v', Value v'))
canonSum (Inl bb v') (VInl p) _ = Left (bb ** v' ** (Refl, p))
canonSum (Inr aa v') (VInr p) _ = Right (aa ** v' ** (Refl, p))
canonSum UnitT VUnit THUnit impossible
canonSum (Lam _ _ _) VLam (THLam _) impossible
canonSum (Pair _ _) (VPair _ _) (THPair _ _ _ _ _) impossible
canonSum (MkEcho _ _ _ _) (VEcho _) (THEcho _) impossible

-- Returns the echo's domain/codomain annotations (`a'`, `b'`) and
-- residue (`v'`) from the *term* (relevant), not the erased type index,
-- so the `SWeaken` reduct `MkEcho Affine a' b' v'` is constructible.
canonEcho : (v : Tm) -> Value v -> Has Empty v (TEcho m a b)
         -> (a' : Ty ** b' : Ty ** v' : Tm ** (v = MkEcho m a' b' v', Value v'))
canonEcho (MkEcho m a b v') (VEcho p) (THEcho _) = (a ** b ** v' ** (Refl, p))
canonEcho UnitT VUnit THUnit impossible
canonEcho (Lam _ _ _) VLam (THLam _) impossible
canonEcho (Pair _ _) (VPair _ _) (THPair _ _ _ _ _) impossible
canonEcho (Inl _ _) (VInl _) (THInl _) impossible
canonEcho (Inr _ _) (VInr _) (THInr _) impossible

------------------------------------------------------------
-- Progress (F1.3)
------------------------------------------------------------

||| Progress: a closed, well-typed Solo term is either a value or can
||| take a step.
|||
||| "Closed" means typed in the empty context — there are no free de
||| Bruijn indices because `Empty` has no `HasVar` inhabitants. The
||| term is taken explicitly so its subterms (the reducts) are
||| runtime-relevant; matching a context-splitting `Refl` forces the
||| sub-derivation's context to `Empty` in place, keeping it a
||| structural subterm so the recursion is total.
public export
progress : (t : Tm) -> Has Empty t a -> Either (Value t) (StepsTo t)
progress (Var n) (THVar v) = absurd (noVarEmpty v)
progress UnitT _ = Left VUnit
progress (Lam q a body) (THLam _) = Left VLam
progress (App t1 t2) (THApp g1 g2 q d1 d2 prf) =
  let (e1, e2) = ctxAddEmpty g1 (ctxScale q g2) prf
      d1' = coeCtx e1 d1
      d2' = coeCtx (ctxScaleEmpty g2 e2) d2
  in case progress t1 d1' of
       Right (MkStepsTo _ s1) => Right (MkStepsTo _ (SApp1 s1))
       Left v1 => case progress t2 d2' of
         Right (MkStepsTo _ s2) => Right (MkStepsTo _ (SApp2 v1 s2))
         Left v2 =>
           let (_ ** _ ** _ ** eq) = canonArr t1 v1 d1' in
           rewrite eq in Right (MkStepsTo _ (SApp v2))
progress (Pair t1 t2) (THPair g1 g2 d1 d2 prf) =
  let (e1, e2) = ctxAddEmpty g1 g2 prf
      d1' = coeCtx e1 d1
      d2' = coeCtx e2 d2
  in case progress t1 d1' of
       Right (MkStepsTo _ s1) => Right (MkStepsTo _ (SPair1 s1))
       Left v1 => case progress t2 d2' of
         Right (MkStepsTo _ s2) => Right (MkStepsTo _ (SPair2 v1 s2))
         Left v2 => Left (VPair v1 v2)
progress (Fst t) (THFst d) =
  case progress t d of
    Right (MkStepsTo _ s) => Right (MkStepsTo _ (SFst1 s))
    Left v =>
      let (_ ** _ ** (eq, v1, v2)) = canonPair t v d in
      rewrite eq in Right (MkStepsTo _ (SFst v1 v2))
progress (Snd t) (THSnd d) =
  case progress t d of
    Right (MkStepsTo _ s) => Right (MkStepsTo _ (SSnd1 s))
    Left v =>
      let (_ ** _ ** (eq, v1, v2)) = canonPair t v d in
      rewrite eq in Right (MkStepsTo _ (SSnd v1 v2))
progress (Inl b t) (THInl d) =
  case progress t d of
    Left v => Left (VInl v)
    Right (MkStepsTo _ s) => Right (MkStepsTo _ (SInl1 s))
progress (Inr a t) (THInr d) =
  case progress t d of
    Left v => Left (VInr v)
    Right (MkStepsTo _ s) => Right (MkStepsTo _ (SInr1 s))
progress (Case t tL tR) (THCase g1 g2 d dL dR prf) =
  let (e1, e2) = ctxAddEmpty g1 g2 prf
      d' = coeCtx e1 d
  in case progress t d' of
       Right (MkStepsTo _ s) => Right (MkStepsTo _ (SCase1 s))
       Left v => case canonSum t v d' of
         Left l =>
           let (_ ** _ ** (eq, v')) = l in
           rewrite eq in Right (MkStepsTo _ (SCaseL v'))
         Right r =>
           let (_ ** _ ** (eq, v')) = r in
           rewrite eq in Right (MkStepsTo _ (SCaseR v'))
progress (Let q t1 t2) (THLet g1 g2 _ d1 d2 prf) =
  let (e1, e2) = ctxAddEmpty (ctxScale q g1) g2 prf
      d1' = coeCtx (ctxScaleEmpty g1 e1) d1
  in case progress t1 d1' of
       Right (MkStepsTo _ s1) => Right (MkStepsTo _ (SLet1 s1))
       Left v1 => Right (MkStepsTo _ (SLet v1))
progress (MkEcho m a b t) (THEcho d) =
  case progress t d of
    Left v => Left (VEcho v)
    Right (MkStepsTo _ s) => Right (MkStepsTo _ (SEcho1 s))
progress (Weaken t) (THWeaken d) =
  case progress t d of
    Right (MkStepsTo _ s) => Right (MkStepsTo _ (SWeaken1 s))
    Left v =>
      let (_ ** _ ** _ ** (eq, v')) = canonEcho t v d in
      rewrite eq in Right (MkStepsTo _ (SWeaken v'))

------------------------------------------------------------
-- Preservation (F1.4 — outstanding obligation)
------------------------------------------------------------

||| Preservation: reduction preserves typing in the SAME context.
||| Preserving the context (not merely "some g'") is the affine-
||| accounting content of the theorem — a reduct that duplicated a
||| linear variable would require a *larger* context.
|||
||| OUTSTANDING (Phase F1.4): the proof needs a QTT substitution lemma
||| that respects context splitting (the App/Let/Case computation
||| cases substitute a value for the bound variable). Left as a typed
||| hole; tracked in proofs/STATUS.md. NOT described as proved.
public export
preservation : Has g t a -> Step t t' -> Has g t' a
preservation _ _ = ?todo_preservation

||| Affine preservation: a direct corollary of `preservation` for the
||| Solo kernel (the preserved context already carries the accounting).
public export
affinePreservation : Has g t a -> Step t t' -> Has g t' a
affinePreservation = preservation
