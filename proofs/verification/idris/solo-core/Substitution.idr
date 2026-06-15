-- SPDX-License-Identifier: MPL-2.0
-- SPDX-FileCopyrightText: 2026 Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk>
--
-- The QTT substitution lemma + preservation for the my-lang Solo core
-- (Idris2 twin of the F1.4 section of SoloCore.v). Phase F1.4, Idris track.
--
-- This is the proof of `preservation` (Soundness.idr states it as the hole
-- `?todo_preservation`). It rests on the standard open-context substitution
-- lemma `htSubst`: substitution under binders shifts the substituted
-- variable's index, so the lemma is generalised over a type-context prefix
-- `I` (the binders crossed so far). The reduction rules only ever do
-- `subst0` (one variable) and `subst2` (two), but the under-binder induction
-- needs the general form.
--
-- Layers (mirroring the Coq twin):
--   4a  append-context algebra      (tappend/uappend + injectivity/split)  [VERIFIED]
--   4b  shape + shift               (shapeType [VERIFIED]; hvShift/htShift/
--                                     htShift0/weakeningAppend — PENDING)
--   4c  substitution core           (reassoc algebra, hvSubst, htSubst,
--                                     substLemma0, subst2Lemma — PENDING)
--   4d  preservation                (consumes 4c; congruences recurse on Step
--                                     — PENDING; Soundness.preservation stays
--                                     the honest hole until 4d lands)
--
-- STATUS (Idris track, #108): this module is HOLE-FREE for the layers it
-- currently contains (4a + the shape invariant). It does NOT yet prove
-- preservation. The remaining layers each thread the type context (and term)
-- RELEVANTLY because Idris erases the indices of `Has`/`HasVar` (ADR-003) —
-- the same wall the `progress` proof navigated with explicit-term recursion.
-- That makes the Idris port heavier than the Coq original (whose `Prop`
-- indices erase without blocking the proofs), but the technique is settled
-- (see `shapeType`/`shapeVar` below): pass `g`/`t` explicitly, recurse on the
-- non-binding premise so erased branch-binder types are never demanded.

module Substitution

import Quantity
import EchoMode
import Syntax
import Context
import Typing
import Soundness
import Data.Nat

%default total

------------------------------------------------------------
-- local disjointness / injectivity helpers
------------------------------------------------------------
-- (Context.idr's equivalents are not exported; restate locally.)

justInj' : Just x = Just y -> x = y
justInj' Refl = Refl

nothingNotJust' : Nothing = Just x -> Void
nothingNotJust' Refl impossible

usnocNotUEmpty' : USnoc d q = UEmpty -> Void
usnocNotUEmpty' Refl impossible

predEq' : S n = S m -> n = m
predEq' Refl = Refl

sNotZ' : S n = Z -> Void
sNotZ' Refl impossible

zNotS' : Z = S n -> Void
zNotS' Refl impossible

------------------------------------------------------------
-- 4a. Append-context algebra
------------------------------------------------------------

||| Append type contexts: `tappend G I` places `I`'s binders on top of `G`.
public export
tappend : Tctx -> Tctx -> Tctx
tappend g TEmpty       = g
tappend g (TSnoc i a)  = TSnoc (tappend g i) a

||| Append usage vectors, shape-matched to `tappend`.
public export
uappend : Uvec -> Uvec -> Uvec
uappend d UEmpty       = d
uappend d (USnoc e q)  = USnoc (uappend d e) q

public export
tappendLen : (g, i : Tctx) -> tlen (tappend g i) = tlen g + tlen i
tappendLen g TEmpty      = rewrite plusZeroRightNeutral (tlen g) in Refl
tappendLen g (TSnoc i a) =
  rewrite tappendLen g i in rewrite plusSuccRightSucc (tlen g) (tlen i) in Refl

public export
uappendLen : (d, e : Uvec) -> ulen (uappend d e) = ulen d + ulen e
uappendLen d UEmpty      = rewrite plusZeroRightNeutral (ulen d) in Refl
uappendLen d (USnoc e q) =
  rewrite uappendLen d e in rewrite plusSuccRightSucc (ulen d) (ulen e) in Refl

public export
uzeroTappend : (g, i : Tctx) -> uzero (tappend g i) = uappend (uzero g) (uzero i)
uzeroTappend g TEmpty      = Refl
uzeroTappend g (TSnoc i a) = rewrite uzeroTappend g i in Refl

public export
uscaleUappend : (q : Q) -> (d, e : Uvec)
             -> uscale q (uappend d e) = uappend (uscale q d) (uscale q e)
uscaleUappend q d UEmpty      = Refl
uscaleUappend q d (USnoc e qe) = rewrite uscaleUappend q d e in Refl

||| Add appended vectors componentwise (forward direction).
public export
uaddUappend : (ai, bi, ci, ag, bg, cg : Uvec)
           -> uadd ai bi = Just ci -> uadd ag bg = Just cg
           -> uadd (uappend ag ai) (uappend bg bi) = Just (uappend cg ci)
uaddUappend UEmpty UEmpty ci ag bg cg hi hg =
  rewrite sym (justInj' hi) in hg
uaddUappend UEmpty (USnoc _ _) _ _ _ _ hi _ = void (nothingNotJust' hi)
uaddUappend (USnoc _ _) UEmpty _ _ _ _ hi _ = void (nothingNotJust' hi)
uaddUappend (USnoc ai qa) (USnoc bi qb) ci ag bg cg hi hg with (uadd ai bi) proof p
  uaddUappend (USnoc ai qa) (USnoc bi qb) ci ag bg cg hi hg | Nothing =
    void (nothingNotJust' hi)
  uaddUappend (USnoc ai qa) (USnoc bi qb) ci ag bg cg hi hg | Just cii =
    rewrite uaddUappend ai bi cii ag bg cg p hg in
    rewrite sym (justInj' hi) in Refl

usnocInj : USnoc x p = USnoc y q -> (x = y, p = q)
usnocInj Refl = (Refl, Refl)

||| uappend injectivity, given the low parts have matching length.
public export
uappendInj : (e, f, d1, d2 : Uvec) -> ulen e = ulen f
          -> uappend d1 e = uappend d2 f -> (d1 = d2, e = f)
uappendInj UEmpty UEmpty d1 d2 _ heq = (heq, Refl)
uappendInj UEmpty (USnoc _ _) _ _ hlen _ = void (zNotS' hlen)
uappendInj (USnoc _ _) UEmpty _ _ hlen _ = void (sNotZ' hlen)
uappendInj (USnoc e qe) (USnoc f qf) d1 d2 hlen heq =
  let (hbody, hq) = usnocInj heq
      (hd, he)    = uappendInj e f d1 d2 (predEq' hlen) hbody
  in (hd, rewrite he in rewrite hq in Refl)

||| Split a usage vector by a target low-length `m`.
public export
uappendSplit : (m : Nat) -> (d : Uvec) -> LTE m (ulen d)
            -> (dhi : Uvec ** dlo : Uvec ** (d = uappend dhi dlo, ulen dlo = m))
uappendSplit Z d _ = (d ** UEmpty ** (Refl, Refl))
uappendSplit (S m) UEmpty lte = absurd lte
uappendSplit (S m) (USnoc d q) lte =
  let (dhi ** dlo ** (heq, hlen)) = uappendSplit m d (fromLteSucc lte) in
  (dhi ** USnoc dlo q ** (rewrite heq in Refl, cong S hlen))

||| Inverse of `uaddUappend`: a sum of appends (with matching low-lengths)
||| splits back into a sum of the highs and a sum of the lows.
public export
uaddUappendInv : (ai, bi, ag, bg, c : Uvec) -> ulen ai = ulen bi
              -> uadd (uappend ag ai) (uappend bg bi) = Just c
              -> (cg : Uvec ** ci : Uvec **
                    (c = uappend cg ci, uadd ag bg = Just cg, uadd ai bi = Just ci))
uaddUappendInv UEmpty UEmpty ag bg c _ h = (c ** UEmpty ** (Refl, h, Refl))
uaddUappendInv UEmpty (USnoc _ _) _ _ _ hlen _ = void (zNotS' hlen)
uaddUappendInv (USnoc _ _) UEmpty _ _ _ hlen _ = void (sNotZ' hlen)
uaddUappendInv (USnoc ai qa) (USnoc bi qb) ag bg c hlen h
    with (uadd (uappend ag ai) (uappend bg bi)) proof p
  uaddUappendInv (USnoc ai qa) (USnoc bi qb) ag bg c hlen h | Nothing =
    void (nothingNotJust' h)
  uaddUappendInv (USnoc ai qa) (USnoc bi qb) ag bg c hlen h | Just d =
    let (cg ** ci ** (heq, hg, hi)) = uaddUappendInv ai bi ag bg d (predEq' hlen) p in
    (cg ** USnoc ci (qAdd qa qb) **
       (rewrite sym (justInj' h) in rewrite heq in Refl,
        hg,
        rewrite hi in Refl))

||| Split a sum-of-appends at the boundary (low parts matched by length).
public export
uaddSplitBoundary : (dg1, di1, dg2, di2, dg, di : Uvec)
                 -> ulen di1 = ulen di2 -> ulen di = ulen di1
                 -> uadd (uappend dg1 di1) (uappend dg2 di2) = Just (uappend dg di)
                 -> (uadd dg1 dg2 = Just dg, uadd di1 di2 = Just di)
uaddSplitBoundary dg1 di1 dg2 di2 dg di h12 hd heq =
  let (cg ** ci ** (hap, hg, hi)) =
        uaddUappendInv di1 di2 dg1 dg2 (uappend dg di) h12 heq
      (hdg, hdi) = uappendInj di ci dg cg
                     (rewrite hd in sym (uaddLen di1 di2 ci hi)) hap
  in (rewrite hdg in hg, rewrite hdi in hi)

------------------------------------------------------------
-- 4b. Shape invariant + shift / weakening
------------------------------------------------------------

-- The type context `g` (and, for `shapeType`, the term `t`) are passed
-- EXPLICITLY/relevantly: Idris erases the indices of `HasVar`/`Has`, so the
-- `THUnit` case (which needs `uzeroLen g`) and the binder cases (which need
-- the Lam annotation for the extended context) cannot recover them from the
-- erased derivation. The term drives the recursion; for every BINDING rule
-- except `THLam` we recurse only on the non-binding premise, so the erased
-- branch-binder types are never needed.

||| A variable lookup's usage has the same length as its type context.
public export
shapeVar : (g : Tctx) -> HasVar g d n a -> ulen d = tlen g
shapeVar (TSnoc g a) HVHere       = cong S (uzeroLen g)
shapeVar (TSnoc g b) (HVThere hv) = cong S (shapeVar g hv)

||| Every derivation's usage has the length of its type context. The
||| separated-context invariant that the split/scale algebra relies on.
public export
shapeType : (g : Tctx) -> (t : Tm) -> Has g d t a -> ulen d = tlen g
shapeType g (Var n)        (THVar hv)                 = shapeVar g hv
shapeType g UnitT          THUnit                     = uzeroLen g
shapeType g (Lam q a body) (THLam bodyD)              = predEq' (shapeType (TSnoc g a) body bodyD)
shapeType g (App t1 t2)    (THApp d1 d2 q h1 h2 prf)  = trans (uaddLen d1 (uscale q d2) _ prf) (shapeType g t1 h1)
shapeType g (With t1 t2)   (THWith h1 h2)             = shapeType g t1 h1
shapeType g (Fst t1)       (THFst h)                  = shapeType g t1 h
shapeType g (Snd t1)       (THSnd h)                  = shapeType g t1 h
shapeType g (Tensor t1 t2) (THTensor d1 d2 h1 h2 prf) = trans (uaddLen d1 d2 _ prf) (shapeType g t1 h1)
shapeType g (LetPair t1 t2)(THLetPair d1 d2 h1 hb prf)= trans (uaddLen d1 d2 _ prf) (shapeType g t1 h1)
shapeType g (Inl b t1)     (THInl h)                  = shapeType g t1 h
shapeType g (Inr a t1)     (THInr h)                  = shapeType g t1 h
shapeType g (Case t tL tR) (THCase d1 d2 h hL hR prf) = trans (uaddLen d1 d2 _ prf) (shapeType g t h)
shapeType g (Let q t1 t2)  (THLet d1 d2 _ h1 h2 prf)  =
  trans (uaddLen (uscale q d1) d2 _ prf) (trans (uscaleLen q d1) (shapeType g t1 h1))
shapeType g (MkEcho m a b t1) (THEcho h)              = shapeType g t1 h
shapeType g (Weaken t1)    (THWeaken h)               = shapeType g t1 h
