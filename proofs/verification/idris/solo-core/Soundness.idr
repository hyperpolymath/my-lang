-- SPDX-License-Identifier: MPL-2.0
-- SPDX-FileCopyrightText: 2026 Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk>

--
-- Operational semantics + soundness for the my-lang Solo core, over
-- the SEPARATED QTT context (ADR-006) and the TWO products (ADR-007).
--
--   * Phase F1.1 — the call-by-value small-step relation `Step` is
--     COMMITTED (constructors below), including the echo residue rules,
--     the additive projections (Fst/Snd on `With`) and the
--     multiplicative let-pair eliminator (`LetPair` on `Tensor`).
--   * Phase F1.3 — `progress` is DISCHARGED as a real total function
--     (no `?todo` holes, no `believe_me`/`assert_total`/`postulate`).
--   * Phase F1.4 — `preservation` is DISCHARGED via the QTT substitution
--     lemma (Substitution.idr). See the F1.4 section at the foot.
--
-- Coq twin: the Soundness layer of SoloCore.v (where progress and
-- preservation are real `Qed` theorems).

module Soundness

import Quantity
import EchoMode
import Syntax
import Context
import Typing
import Substitution

%default total

-- NOTE (F1.4): this module re-proves `progress` over the separated context
-- + two products, and PROVES `preservation` over the correct same-usage
-- statement (DISCHARGED 2026-06-21, #108). The QTT substitution lemmas it
-- consumes live in the companion module Substitution.idr (`substLemma0`,
-- `subst2Lemma`); the proof is by induction on `Step`, total and hole-free
-- (`:total preservation`). The OLD design (split-intro Pair + project-elim
-- Fst/Snd) made the same-usage preservation literally UNSOUND (issue #93),
-- so this could not be honestly filled before the 2026-06-15 migration.

------------------------------------------------------------
-- Values
------------------------------------------------------------

||| Solo values: canonical forms of closed terms.
public export
data Value : Tm -> Type where
  VUnit : Value UnitT
  VLam  : Value (Lam q a t)
  -- additive pair: a value once both components are (eager, ADR-008)
  VWith : Value t1 -> Value t2 -> Value (With t1 t2)
  -- multiplicative pair: a value once both components are
  VTensor : Value t1 -> Value t2 -> Value (Tensor t1 t2)
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
-- rules thread evaluation left-to-right. Mirrors `step` in SoloCore.v.

public export
data Step : Tm -> Tm -> Type where
  -- computation
  SApp   : Value v -> Step (App (Lam q a t) v) (subst0 v t)
  -- additive projections fire on a `With` value (both components values)
  SFst   : Value v1 -> Value v2 -> Step (Fst (With v1 v2)) v1
  SSnd   : Value v1 -> Value v2 -> Step (Snd (With v1 v2)) v2
  SCaseL : Value v -> Step (Case (Inl b v) tL tR) (subst0 v tL)
  SCaseR : Value v -> Step (Case (Inr a v) tL tR) (subst0 v tR)
  SLet   : Value v -> Step (Let q v t2) (subst0 v t2)
  -- multiplicative let-pair fires on a `Tensor` value, substituting BOTH
  -- components into the two-binder body
  SLetPair : Value v1 -> Value v2 -> Step (LetPair (Tensor v1 v2) body) (subst2 v1 v2 body)
  -- congruence (left-to-right, CBV)
  SApp1   : Step t1 t1' -> Step (App t1 t2) (App t1' t2)
  SApp2   : Value v1 -> Step t2 t2' -> Step (App v1 t2) (App v1 t2')
  SWith1  : Step t1 t1' -> Step (With t1 t2) (With t1' t2)
  SWith2  : Value v1 -> Step t2 t2' -> Step (With v1 t2) (With v1 t2')
  SFst1   : Step t t' -> Step (Fst t) (Fst t')
  SSnd1   : Step t t' -> Step (Snd t) (Snd t')
  STensor1 : Step t1 t1' -> Step (Tensor t1 t2) (Tensor t1' t2)
  STensor2 : Value v1 -> Step t2 t2' -> Step (Tensor v1 t2) (Tensor v1 t2')
  SLetPair1 : Step t1 t1' -> Step (LetPair t1 t2) (LetPair t1' t2)
  SInl1   : Step t t' -> Step (Inl b t) (Inl b t')
  SInr1   : Step t t' -> Step (Inr a t) (Inr a t')
  SCase1  : Step t t' -> Step (Case t tL tR) (Case t' tL tR)
  SLet1   : Step t1 t1' -> Step (Let q t1 t2) (Let q t1' t2)
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
-- Closed-context lemmas
------------------------------------------------------------
--
-- "Closed" = typed in the empty TYPE context `TEmpty` under the empty
-- USAGE `UEmpty`. Because the separated context FIXES the type context
-- across splits, every premise of a closed derivation is also in
-- `TEmpty`; only the usage vector needs massaging back to `UEmpty`.
-- The split inversions `uaddEmpty` / `uscaleEmpty` (Context.idr) do
-- exactly that. `coeUse` retypes a sub-derivation along a usage
-- equality. (progress recurses on the explicit TERM, so a coerced —
-- non-subterm — derivation is fine for totality.)

noVarEmpty : HasVar TEmpty d n a -> Void
noVarEmpty HVHere impossible
noVarEmpty (HVThere _) impossible

||| Coerce a derivation along a USAGE equality. Used to retype the
||| immediate subterms back to `UEmpty` once the split inversions have
||| shown their usages are `UEmpty`.
coeUse : d = d' -> Has g d t a -> Has g d' t a
coeUse Refl x = x

------------------------------------------------------------
-- Canonical forms: a closed value's shape is fixed by its type
------------------------------------------------------------

canonArr : (v : Tm) -> Value v -> Has TEmpty d v (TArr q a b)
        -> (q' : Q ** a' : Ty ** body : Tm ** v = Lam q' a' body)
canonArr (Lam q' a' body) VLam _ = (q' ** a' ** body ** Refl)
canonArr UnitT VUnit THUnit impossible
canonArr (With _ _) (VWith _ _) (THWith _ _) impossible
canonArr (Tensor _ _) (VTensor _ _) (THTensor _ _ _ _ _) impossible
canonArr (Inl _ _) (VInl _) (THInl _) impossible
canonArr (Inr _ _) (VInr _) (THInr _) impossible
canonArr (MkEcho _ _ _ _) (VEcho _) (THEcho _) impossible

||| Additive product: a closed value of type `TWith a b` is `With v1 v2`.
canonWith : (v : Tm) -> Value v -> Has TEmpty d v (TWith a b)
         -> (t1 : Tm ** t2 : Tm ** (v = With t1 t2, Value t1, Value t2))
canonWith (With t1 t2) (VWith p1 p2) _ = (t1 ** t2 ** (Refl, p1, p2))
canonWith UnitT VUnit THUnit impossible
canonWith (Lam _ _ _) VLam (THLam _) impossible
canonWith (Tensor _ _) (VTensor _ _) (THTensor _ _ _ _ _) impossible
canonWith (Inl _ _) (VInl _) (THInl _) impossible
canonWith (Inr _ _) (VInr _) (THInr _) impossible
canonWith (MkEcho _ _ _ _) (VEcho _) (THEcho _) impossible

||| Multiplicative product: a closed value of type `TTensor a b` is
||| `Tensor v1 v2`.
canonTensor : (v : Tm) -> Value v -> Has TEmpty d v (TTensor a b)
           -> (t1 : Tm ** t2 : Tm ** (v = Tensor t1 t2, Value t1, Value t2))
canonTensor (Tensor t1 t2) (VTensor p1 p2) _ = (t1 ** t2 ** (Refl, p1, p2))
canonTensor UnitT VUnit THUnit impossible
canonTensor (Lam _ _ _) VLam (THLam _) impossible
canonTensor (With _ _) (VWith _ _) (THWith _ _) impossible
canonTensor (Inl _ _) (VInl _) (THInl _) impossible
canonTensor (Inr _ _) (VInr _) (THInr _) impossible
canonTensor (MkEcho _ _ _ _) (VEcho _) (THEcho _) impossible

canonSum : (v : Tm) -> Value v -> Has TEmpty d v (TSum a b)
        -> Either (bb : Ty ** v' : Tm ** (v = Inl bb v', Value v'))
                  (aa : Ty ** v' : Tm ** (v = Inr aa v', Value v'))
canonSum (Inl bb v') (VInl p) _ = Left (bb ** v' ** (Refl, p))
canonSum (Inr aa v') (VInr p) _ = Right (aa ** v' ** (Refl, p))
canonSum UnitT VUnit THUnit impossible
canonSum (Lam _ _ _) VLam (THLam _) impossible
canonSum (With _ _) (VWith _ _) (THWith _ _) impossible
canonSum (Tensor _ _) (VTensor _ _) (THTensor _ _ _ _ _) impossible
canonSum (MkEcho _ _ _ _) (VEcho _) (THEcho _) impossible

-- Returns the echo's domain/codomain annotations (`a'`, `b'`) and
-- residue (`v'`) from the *term* (relevant), not the erased type index,
-- so the `SWeaken` reduct `MkEcho Affine a' b' v'` is constructible.
canonEcho : (v : Tm) -> Value v -> Has TEmpty d v (TEcho m a b)
         -> (a' : Ty ** b' : Ty ** v' : Tm ** (v = MkEcho m a' b' v', Value v'))
canonEcho (MkEcho m a b v') (VEcho p) (THEcho _) = (a ** b ** v' ** (Refl, p))
canonEcho UnitT VUnit THUnit impossible
canonEcho (Lam _ _ _) VLam (THLam _) impossible
canonEcho (With _ _) (VWith _ _) (THWith _ _) impossible
canonEcho (Tensor _ _) (VTensor _ _) (THTensor _ _ _ _ _) impossible
canonEcho (Inl _ _) (VInl _) (THInl _) impossible
canonEcho (Inr _ _) (VInr _) (THInr _) impossible

------------------------------------------------------------
-- Progress (F1.3)
------------------------------------------------------------

||| Progress: a closed, well-typed Solo term is either a value or can
||| take a step.
|||
||| "Closed" means typed in the empty type context under the empty usage
||| — there are no free de Bruijn indices because `TEmpty` has no
||| `HasVar` inhabitants. The term is taken explicitly so its subterms
||| (the reducts) are runtime-relevant; the split inversions force the
||| sub-derivations' usages to `UEmpty`, which `coeUse` retypes.
public export
progress : (t : Tm) -> Has TEmpty UEmpty t a -> Either (Value t) (StepsTo t)
progress (Var n) (THVar v) = absurd (noVarEmpty v)
progress UnitT _ = Left VUnit
progress (Lam q a body) (THLam _) = Left VLam
progress (App t1 t2) (THApp d1 d2 q d1d d2d prf) =
  let (e1, e2) = uaddEmpty d1 (uscale q d2) prf
      d1' = coeUse e1 d1d
      d2' = coeUse (uscaleEmpty q d2 e2) d2d
  in case progress t1 d1' of
       Right (MkStepsTo _ s1) => Right (MkStepsTo _ (SApp1 s1))
       Left v1 => case progress t2 d2' of
         Right (MkStepsTo _ s2) => Right (MkStepsTo _ (SApp2 v1 s2))
         Left v2 =>
           let (_ ** _ ** _ ** eq) = canonArr t1 v1 d1' in
           rewrite eq in Right (MkStepsTo _ (SApp v2))
progress (With t1 t2) (THWith d1d d2d) =
  -- additive: usage is SHARED, both premises already in (TEmpty, UEmpty)
  case progress t1 d1d of
    Right (MkStepsTo _ s1) => Right (MkStepsTo _ (SWith1 s1))
    Left v1 => case progress t2 d2d of
      Right (MkStepsTo _ s2) => Right (MkStepsTo _ (SWith2 v1 s2))
      Left v2 => Left (VWith v1 v2)
progress (Fst t) (THFst d) =
  case progress t d of
    Right (MkStepsTo _ s) => Right (MkStepsTo _ (SFst1 s))
    Left v =>
      let (_ ** _ ** (eq, v1, v2)) = canonWith t v d in
      rewrite eq in Right (MkStepsTo _ (SFst v1 v2))
progress (Snd t) (THSnd d) =
  case progress t d of
    Right (MkStepsTo _ s) => Right (MkStepsTo _ (SSnd1 s))
    Left v =>
      let (_ ** _ ** (eq, v1, v2)) = canonWith t v d in
      rewrite eq in Right (MkStepsTo _ (SSnd v1 v2))
progress (Tensor t1 t2) (THTensor d1 d2 d1d d2d prf) =
  let (e1, e2) = uaddEmpty d1 d2 prf
      d1' = coeUse e1 d1d
      d2' = coeUse e2 d2d
  in case progress t1 d1' of
       Right (MkStepsTo _ s1) => Right (MkStepsTo _ (STensor1 s1))
       Left v1 => case progress t2 d2' of
         Right (MkStepsTo _ s2) => Right (MkStepsTo _ (STensor2 v1 s2))
         Left v2 => Left (VTensor v1 v2)
progress (LetPair t1 t2) (THLetPair d1 d2 _ _ d1d _ prf) =
  let (e1, _) = uaddEmpty d1 d2 prf
      d1' = coeUse e1 d1d
  in case progress t1 d1' of
       Right (MkStepsTo _ s1) => Right (MkStepsTo _ (SLetPair1 s1))
       Left v1 =>
         let (_ ** _ ** (eq, w1, w2)) = canonTensor t1 v1 d1' in
         rewrite eq in Right (MkStepsTo _ (SLetPair w1 w2))
progress (Inl b t) (THInl d) =
  case progress t d of
    Left v => Left (VInl v)
    Right (MkStepsTo _ s) => Right (MkStepsTo _ (SInl1 s))
progress (Inr a t) (THInr d) =
  case progress t d of
    Left v => Left (VInr v)
    Right (MkStepsTo _ s) => Right (MkStepsTo _ (SInr1 s))
progress (Case t tL tR) (THCase d1 d2 _ _ d dL dR prf) =
  let (e1, _) = uaddEmpty d1 d2 prf
      d' = coeUse e1 d
  in case progress t d' of
       Right (MkStepsTo _ s) => Right (MkStepsTo _ (SCase1 s))
       Left v => case canonSum t v d' of
         Left l =>
           let (_ ** _ ** (eq, v')) = l in
           rewrite eq in Right (MkStepsTo _ (SCaseL v'))
         Right r =>
           let (_ ** _ ** (eq, v')) = r in
           rewrite eq in Right (MkStepsTo _ (SCaseR v'))
progress (Let q t1 t2) (THLet d1 d2 _ _ d1d _ prf) =
  let (es, _) = uaddEmpty (uscale q d1) d2 prf
      d1' = coeUse (uscaleEmpty q d1 es) d1d
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
-- Preservation (F1.4)
------------------------------------------------------------

||| Preservation: reduction preserves typing in the SAME context AND the
||| SAME usage. Preserving the usage (not merely "some d'") is the
||| affine-accounting content of the theorem — a reduct that duplicated a
||| linear variable would require a *larger* usage.
|||
||| DISCHARGED (Phase F1.4) by induction on `Step` (mirrors the Coq twin
||| `preservation` in SoloCore.v). The β/projection/elimination cases
||| consume the QTT substitution lemmas from `Substitution.idr`:
||| `substLemma0` for the single-variable reductions (App/Case/Let), and
||| `subst2Lemma` for the two-variable `LetPair` reduction; each realigns
||| the residual usage with the original `d` via the `uadd` accounting
||| (commutativity / associativity / `uscale One`). The congruence cases
||| recurse structurally on the sub-step and rebuild the derivation.
|||
||| Stated over the CORRECT design (separated context + genuine products):
||| with the old split-intro/project-elim `Pair`, this same-usage statement
||| was unsound (issue #93); the migration in this file is what makes it
||| provable.
|||
||| The reduct term `t` is taken EXPLICITLY (relevant), exactly as
||| `progress` does: Idris erases the derivation's term/type indices, so the
||| substitution lemmas — which compute on the bound variable's body and the
||| substituted value — need those sub-terms supplied relevantly from the
||| matched term; `g`/`d` are relevant for the same reason. This is a
||| presentation choice; the logical statement is the Coq twin's.
public export
preservation : {g : Tctx} -> {d : Uvec} -> (t : Tm)
            -> Has g d t a -> Step t t' -> Has g d t' a
preservation (App (Lam q' a0 tb) v) (THApp d1 d2 q' (THLam hb) h2 prf) (SApp vval) =
  let (dgr ** (hadd, hht)) = substLemma0 g a0 d1 q' tb d2 v hb h2
  in coeUse (justInj' (trans (sym hadd) prf)) hht
-- additive projections fire on a With value
preservation (Fst (With v1 v2)) (THFst (THWith h1 h2)) (SFst v1v v2v) = h1
preservation (Snd (With v1 v2)) (THSnd (THWith h1 h2)) (SSnd v1v v2v) = h2
-- sum elimination: substitute the injected value into the taken branch
preservation (Case (Inl bb v) tL tR) (THCase d1 d2 aa bb (THInl hv) hL hR prf)
             (SCaseL vval) =
  let (dgr ** (hadd, hht)) = substLemma0 g aa d2 One tL d1 v hL hv
  in coeUse
       (justInj' (trans (sym (trans (uaddComm d1 d2)
                       (trans (sym (cong (uadd d2) (uscaleOne d1))) hadd))) prf))
       hht
preservation (Case (Inr aa v) tL tR) (THCase d1 d2 aa bb (THInr hv) hL hR prf)
             (SCaseR vval) =
  let (dgr ** (hadd, hht)) = substLemma0 g bb d2 One tR d1 v hR hv
  in coeUse
       (justInj' (trans (sym (trans (uaddComm d1 d2)
                       (trans (sym (cong (uadd d2) (uscaleOne d1))) hadd))) prf))
       hht
-- let binding: substitute the bound value
preservation (Let q1 v t2) (THLet d1 d2 q1 aa h1 h2 prf) (SLet vval) =
  let (dgr ** (hadd, hht)) = substLemma0 g aa d2 q1 t2 d1 v h2 h1
  in coeUse
       (justInj' (trans (sym (trans (uaddComm (uscale q1 d1) d2) hadd)) prf))
       hht
-- multiplicative let-pair: two-variable substitution of the tensor components
preservation (LetPair (Tensor v1 v2) body)
             (THLetPair d1 d2 aa bb (THTensor dv1 dv2 hv1 hv2 prft) hb prf)
             (SLetPair v1v v2v) =
  subst2Lemma g aa bb d d1 d2 dv1 dv2 body v1 v2 hb hv1 hv2 prft prf
-- congruence: thread the sub-step and rebuild
preservation (App t1 t2) (THApp d1 d2 q h1 h2 prf) (SApp1 s1) =
  THApp d1 d2 q (preservation t1 h1 s1) h2 prf
preservation (App v1 t2) (THApp d1 d2 q h1 h2 prf) (SApp2 v1v s2) =
  THApp d1 d2 q h1 (preservation t2 h2 s2) prf
preservation (With t1 t2) (THWith h1 h2) (SWith1 s1) =
  THWith (preservation t1 h1 s1) h2
preservation (With v1 t2) (THWith h1 h2) (SWith2 v1v s2) =
  THWith h1 (preservation t2 h2 s2)
preservation (Fst tt) (THFst h) (SFst1 s) = THFst (preservation tt h s)
preservation (Snd tt) (THSnd h) (SSnd1 s) = THSnd (preservation tt h s)
preservation (Tensor t1 t2) (THTensor d1 d2 h1 h2 prf) (STensor1 s1) =
  THTensor d1 d2 (preservation t1 h1 s1) h2 prf
preservation (Tensor v1 t2) (THTensor d1 d2 h1 h2 prf) (STensor2 v1v s2) =
  THTensor d1 d2 h1 (preservation t2 h2 s2) prf
preservation (LetPair t1 t2) (THLetPair d1 d2 aa bb h1 hb prf) (SLetPair1 s1) =
  THLetPair d1 d2 aa bb (preservation t1 h1 s1) hb prf
preservation (Inl b0 tt) (THInl h) (SInl1 s) = THInl (preservation tt h s)
preservation (Inr a0 tt) (THInr h) (SInr1 s) = THInr (preservation tt h s)
preservation (Case tc tL tR) (THCase d1 d2 aa bb h hL hR prf) (SCase1 s) =
  THCase d1 d2 aa bb (preservation tc h s) hL hR prf
preservation (Let q1 t1 t2) (THLet d1 d2 q1 aa h1 h2 prf) (SLet1 s1) =
  THLet d1 d2 q1 aa (preservation t1 h1 s1) h2 prf
preservation (MkEcho m a0 b0 tt) (THEcho h) (SEcho1 s) = THEcho (preservation tt h s)
preservation (Weaken tt) (THWeaken h) (SWeaken1 s) = THWeaken (preservation tt h s)
-- echo weaken: a Linear echo value weakens to an Affine one
preservation (Weaken (MkEcho Linear a0 b0 v)) (THWeaken h) (SWeaken vval) =
  case h of
    THEcho hv => THEcho hv

||| Affine preservation: a direct corollary of `preservation` for the
||| Solo kernel (the preserved usage already carries the accounting).
public export
affinePreservation : {g : Tctx} -> {d : Uvec} -> (t : Tm)
                  -> Has g d t a -> Step t t' -> Has g d t' a
affinePreservation t = preservation t
