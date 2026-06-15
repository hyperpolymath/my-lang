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
--   4b  shape + shift               (shapeType/hvShift/htShift [VERIFIED];
--                                     htShift0/weakeningAppend — PENDING)
--   4c  substitution core           (reassoc algebra, hvSubst, htSubst,
--                                     substLemma0, subst2Lemma — PENDING)
--   4d  preservation                (consumes 4c; congruences recurse on Step
--                                     — PENDING; Soundness.preservation stays
--                                     the honest hole until 4d lands)
--
-- STATUS (Idris track, #108): HOLE-FREE for the layers it currently contains —
-- 4a (append algebra), the shape invariant, `hvShift` (has_var weakening),
-- the structural shift index `shiftIdx`/`shiftIdxCorrect`/`shiftVarLemma`, and
-- `htShift` (term weakening, all 14 constructor cases, total) on top of its
-- support lemmas (`unitInv`, `usplitLemma`, `uaddUshift`, `ushiftUscale`).
-- It does NOT yet prove preservation (htSubst/subst2Lemma/preservation pending).
--
-- ARCHITECTURAL BLOCKER on `htShift` — RESOLVED 2026-06-15 via Option A.
-- `htShift` shifts under binders, so its binder cases extend the type context
-- with the binder's type and recurse on the BODY. For `Lam q a body` the bound
-- type `a` is in the term and recoverable. But `Let`/`Case`/`LetPair` did NOT
-- annotate their bound types in the term — those types lived only as ERASED
-- indices of the typing derivation (`THLet`/`THCase`/`THLetPair`), so the
-- extended context `TSnoc i <bound-type>` for the body recursion could not be
-- constructed under Idris erasure. (Coq was unaffected — `Prop` indices erase
-- without blocking proofs; `progress` was unaffected — it never recurses into
-- those bodies.)
--
-- FIX (Option A): `THLet`/`THCase`/`THLetPair` now carry their bound types as
-- explicit, runtime-relevant fields — the ADR-003 pattern already used for the
-- usage splits (`THApp`/`THTensor`). `progress` was re-verified against the new
-- fields; the build stays green. With the bound types available, all 14
-- `htShift` cases typecheck and the lemma is total (below).

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
shapeType g (LetPair t1 t2)(THLetPair d1 d2 _ _ h1 hb prf)= trans (uaddLen d1 d2 _ prf) (shapeType g t1 h1)
shapeType g (Inl b t1)     (THInl h)                  = shapeType g t1 h
shapeType g (Inr a t1)     (THInr h)                  = shapeType g t1 h
shapeType g (Case t tL tR) (THCase d1 d2 _ _ h hL hR prf) = trans (uaddLen d1 d2 _ prf) (shapeType g t h)
shapeType g (Let q t1 t2)  (THLet d1 d2 _ _ h1 h2 prf)  =
  trans (uaddLen (uscale q d1) d2 _ prf) (trans (uscaleLen q d1) (shapeType g t1 h1))
shapeType g (MkEcho m a b t1) (THEcho h)              = shapeType g t1 h
shapeType g (Weaken t1)    (THWeaken h)               = shapeType g t1 h

||| `n < 0` is `False` (the shift base case needs this; `<` does not reduce
||| on an abstract `n` without casing it).
nLtZero : (n : Nat) -> (n < 0) = False
nLtZero Z     = Refl
nLtZero (S _) = Refl

-- HasVar inversion. Idris cannot pattern-match `HVHere` when the usage is a
-- stuck term (e.g. `uappend dg di`): `HVHere`'s usage index `USnoc (uzero g)
-- One` will not unify with it. The standard fix is an inversion lemma taking
-- the usage as a FLEXIBLE parameter and returning the constructor disjunction
-- with PROPOSITIONAL equalities — which the caller consumes without matching.
||| Inversion result for `HasVar (TSnoc g0 a0) dd nn aa`. Equations are
||| PROPOSITIONAL (not in the indices), so the caller matches `VIH`/`VIT`
||| without unifying its abstract `dd` against a constructor's usage shape;
||| the `HVThere` witnesses `d0`/`n0` are ERASED (the caller recovers a
||| relevant index from its own `nn`-pattern + the equations).
public export
data VarInvR : Tctx -> Ty -> Uvec -> Nat -> Ty -> Type where
  VIH : (dd = USnoc (uzero g0) One) -> (nn = Z) -> (aa = a0) -> VarInvR g0 a0 dd nn aa
  VIT : {0 d0 : Uvec} -> {0 n0 : Nat}
     -> (dd = USnoc d0 Zero) -> (nn = S n0) -> HasVar g0 d0 n0 aa
     -> VarInvR g0 a0 dd nn aa

public export
varInv : HasVar (TSnoc g0 a0) dd nn aa -> VarInvR g0 a0 dd nn aa
varInv HVHere       = VIH Refl Refl Refl
varInv (HVThere hv) = VIT Refl Refl hv

||| The de Bruijn shift index, defined STRUCTURALLY (not via `if n < c`):
||| inserting a binder at cutoff `c` keeps indices below `c` and bumps the
||| rest. `shiftIdx c n` reduces definitionally on its arguments — so the
||| shift-lemma index arithmetic needs no lazy-`if`/`Ord` rewriting. The
||| bridge to the kernel's `shift` (which uses `if`) is `shiftIdxCorrect`.
public export
shiftIdx : Nat -> Nat -> Nat
shiftIdx Z     n     = S n
shiftIdx (S c) Z     = Z
shiftIdx (S c) (S n) = S (shiftIdx c n)

||| Push `S` through the shift-index conditional (used only in the bridge).
sPushIf : (b : Bool) -> (x, y : Nat) -> S (ifThenElse b x y) = ifThenElse b (S x) (S y)
sPushIf True  x y = Refl
sPushIf False x y = Refl

||| `shiftIdx` agrees with the kernel `shift`'s `if n < c then n else S n`.
||| Proven once (the only place the lazy-`if` index reasoning is needed);
||| consumed by `htShift`'s `Var` case to connect `hvShift` to
||| `shift (tlen i) (Var n)`.
public export
shiftIdxCorrect : (c, n : Nat) -> shiftIdx c n = ifThenElse (n < c) n (S n)
shiftIdxCorrect Z     n     = rewrite nLtZero n in Refl
shiftIdxCorrect (S c) Z     = Refl
shiftIdxCorrect (S c) (S n) = rewrite shiftIdxCorrect c n in sPushIf (n < c) n (S n)

||| has_var weakening: insert a fresh Zero-usage binder `c` at cutoff `tlen i`,
||| shifting the looked-up index by `shiftIdx (tlen i)`. `i`,`g`,`c`,`dg`,`di`,
||| `n` are explicit/relevant (the `HVHere` reconstruction needs the contexts;
||| Idris erases the indices). With `shiftIdx`, every index step is definitional.
public export
hvShift : (i, g : Tctx) -> (c : Ty) -> (dg, di : Uvec) -> (n : Nat)
       -> ulen di = tlen i
       -> HasVar (tappend g i) (uappend dg di) n a
       -> HasVar (tappend (TSnoc g c) i) (uappend (USnoc dg Zero) di)
                 (shiftIdx (tlen i) n) a
hvShift TEmpty g c dg UEmpty n hlen hv = HVThere hv
hvShift TEmpty g c dg (USnoc _ _) n hlen hv = void (sNotZ' hlen)
hvShift (TSnoc i' a') g c dg UEmpty n hlen hv = void (zNotS' hlen)
hvShift (TSnoc i' a') g c dg (USnoc di' qd) Z hlen hv =
  -- shiftIdx (S (tlen i')) Z = Z : the head variable stays at 0 (HVHere).
  case varInv hv of
    VIH heq _ ha =>
      let (hdgdi, hqd) = usnocInj heq
          (hdg, hdi)   = uappendInj di' (uzero i') dg (uzero g)
                           (trans (predEq' hlen) (sym (uzeroLen i')))
                           (trans hdgdi (uzeroTappend g i'))
      in rewrite hqd in rewrite hdg in rewrite hdi in
         rewrite sym (uzeroTappend (TSnoc g c) i') in
         rewrite ha in HVHere
    VIT _ hn _ => void (zNotS' hn)
hvShift (TSnoc i' a') g c dg (USnoc di' qd) (S n0) hlen hv =
  -- shiftIdx (S (tlen i')) (S n0) = S (shiftIdx (tlen i') n0) : recurse (HVThere).
  case varInv hv of
    VIH _ hn _ => void (sNotZ' hn)
    VIT heq hn hv' =>
      let (hdgdi, hqd) = usnocInj heq
          hv'' : HasVar (tappend g i') (uappend dg di') n0 a
          hv'' = rewrite hdgdi in rewrite predEq' hn in hv'
          ih = hvShift i' g c dg di' n0 (predEq' hlen) hv''
      in rewrite hqd in HVThere ih

------------------------------------------------------------
-- 4b (cont). Term weakening: htShift
------------------------------------------------------------

trueNotFalse : True = False -> Void
trueNotFalse Refl impossible

||| `shiftIdx` vs the kernel `<`, split by the boolean (avoids the surface-form
||| mismatch `n < c` vs `compareNat n c == LT` that breaks `rewrite`).
shiftIdxTrue : (c, n : Nat) -> (n < c) = True -> shiftIdx c n = n
shiftIdxTrue Z     n     prf = void (trueNotFalse (trans (sym prf) (nLtZero n)))
shiftIdxTrue (S c) Z     prf = Refl
shiftIdxTrue (S c) (S n) prf = cong S (shiftIdxTrue c n prf)

shiftIdxFalse : (c, n : Nat) -> (n < c) = False -> shiftIdx c n = S n
shiftIdxFalse Z     n     prf = Refl
shiftIdxFalse (S c) Z     prf = void (trueNotFalse prf)
shiftIdxFalse (S c) (S n) prf = cong S (shiftIdxFalse c n prf)

||| The kernel `shift` on a variable equals `Var` of the structural `shiftIdx`.
public export
shiftVarLemma : (c, n : Nat) -> shift c (Var n) = Var (shiftIdx c n)
shiftVarLemma c n with (n < c) proof prf
  shiftVarLemma c n | True  = rewrite shiftIdxTrue c n prf in Refl
  shiftVarLemma c n | False = rewrite shiftIdxFalse c n prf in Refl

||| UnitT inversion (its usage `uzero g` is not a free field; direct `case`
||| would be stuck, same as `HVHere`). Also yields the type equation, since
||| the caller's result type is not yet refined to `TUnit`.
public export
unitInv : Has g dd UnitT aa -> (dd = uzero g, aa = TUnit)
unitInv THUnit = (Refl, Refl)

||| Split a derivation's usage along the `G`/`I` append boundary via the shape
||| invariant. `g`,`i`,`t` relevant (shapeType needs the context + term).
public export
usplitLemma : (g, i : Tctx) -> (dd : Uvec) -> (t : Tm) -> Has (tappend g i) dd t a
           -> (dg : Uvec ** di : Uvec ** (dd = uappend dg di, ulen di = tlen i))
usplitLemma g i dd t h =
  uappendSplit (tlen i) dd
    (rewrite trans (shapeType (tappend g i) t h) (tappendLen g i) in
     rewrite plusCommutative (tlen g) (tlen i) in lteAddRight (tlen i))

||| Boundary-insert: add two `(USnoc _ Zero)`-headed appends.
public export
uaddUshift : (ag, ai, bg, bi, cg, ci : Uvec)
          -> uadd ag bg = Just cg -> uadd ai bi = Just ci
          -> uadd (uappend (USnoc ag Zero) ai) (uappend (USnoc bg Zero) bi)
             = Just (uappend (USnoc cg Zero) ci)
uaddUshift ag ai bg bi cg ci hg hi =
  uaddUappend ai bi ci (USnoc ag Zero) (USnoc bg Zero) (USnoc cg Zero)
    hi (rewrite hg in Refl)

||| Scaling commutes with the boundary-insert.
public export
ushiftUscale : (q : Q) -> (dg, di : Uvec)
            -> uscale q (uappend (USnoc dg Zero) di)
               = uappend (USnoc (uscale q dg) Zero) (uscale q di)
ushiftUscale q dg di =
  rewrite uscaleUappend q (USnoc dg Zero) di in rewrite qMulZeroR q in Refl

------------------------------------------------------------
-- 4b (cont). Term weakening: htShift (open-context shift)
------------------------------------------------------------

||| Insert a fresh `Zero`-usage binder `c` at cutoff `tlen i` into a typing
||| derivation, shifting the term by `shift (tlen i)`. The general (prefix `i`)
||| form is what the under-binder substitution induction needs.
|||
||| The binder cases recurse on the body in a context extended by the bound
||| type. `Lam` reads its type from the term; `Let`/`Case`/`LetPair` read theirs
||| from the explicit type fields added to `THLet`/`THCase`/`THLetPair` (Option
||| A — without them the extended context `TSnoc i <bound>` is unconstructible
||| under Idris erasure). The usage-splitting rules decompose each premise's
||| usage along the `G`/`I` boundary (`usplitLemma`/`uappendSplit` + the
||| `(Refl, _)` let-pattern performs the substitution), split the `uadd` premise
||| with `uaddSplitBoundary`, and reassemble with the boundary insert
||| `uaddUshift` (composing `ushiftUscale` for the `q`-scaled legs of App/Let).
public export
htShift : (i, g : Tctx) -> (c : Ty) -> (dg, di : Uvec) -> (t : Tm)
       -> ulen di = tlen i
       -> Has (tappend g i) (uappend dg di) t a
       -> Has (tappend (TSnoc g c) i) (uappend (USnoc dg Zero) di)
                (shift (tlen i) t) a
htShift i g c dg di (Var n) hlen (THVar hv) =
  rewrite shiftVarLemma (tlen i) n in
  THVar (hvShift i g c dg di n hlen hv)
htShift i g c dg di UnitT hlen h =
  let (hd, ha) = unitInv h
      (hdg, hdi) = uappendInj di (uzero i) dg (uzero g)
                     (trans hlen (sym (uzeroLen i)))
                     (trans hd (uzeroTappend g i))
  in rewrite ha in rewrite hdg in rewrite hdi in
     rewrite sym (uzeroTappend (TSnoc g c) i) in THUnit
htShift i g c dg di (Lam q a' body) hlen (THLam bodyD) =
  THLam (htShift (TSnoc i a') g c dg (USnoc di q) body (cong S hlen) bodyD)
htShift i g c dg di (App t1 t2) hlen (THApp d1 d2 q h1 h2 prf) =
  let (d1g ** d1i ** (Refl, l1)) = usplitLemma g i d1 t1 h1
      (d2g ** d2i ** (Refl, l2)) = usplitLemma g i d2 t2 h2
      (hg, hi) = uaddSplitBoundary d1g d1i (uscale q d2g) (uscale q d2i) dg di
                   (trans l1 (sym (trans (uscaleLen q d2i) l2)))
                   (trans hlen (sym l1))
                   (rewrite sym (uscaleUappend q d2g d2i) in prf)
  in THApp (uappend (USnoc d1g Zero) d1i) (uappend (USnoc d2g Zero) d2i) q
           (htShift i g c d1g d1i t1 l1 h1)
           (htShift i g c d2g d2i t2 l2 h2)
           (rewrite ushiftUscale q d2g d2i in
            uaddUshift d1g d1i (uscale q d2g) (uscale q d2i) dg di hg hi)
htShift i g c dg di (With t1 t2) hlen (THWith h1 h2) =
  THWith (htShift i g c dg di t1 hlen h1) (htShift i g c dg di t2 hlen h2)
htShift i g c dg di (Fst t) hlen (THFst h) = THFst (htShift i g c dg di t hlen h)
htShift i g c dg di (Snd t) hlen (THSnd h) = THSnd (htShift i g c dg di t hlen h)
htShift i g c dg di (Tensor t1 t2) hlen (THTensor d1 d2 h1 h2 prf) =
  let (d1g ** d1i ** (Refl, l1)) = usplitLemma g i d1 t1 h1
      (d2g ** d2i ** (Refl, l2)) = usplitLemma g i d2 t2 h2
      (hg, hi) = uaddSplitBoundary d1g d1i d2g d2i dg di
                   (trans l1 (sym l2)) (trans hlen (sym l1)) prf
  in THTensor (uappend (USnoc d1g Zero) d1i) (uappend (USnoc d2g Zero) d2i)
              (htShift i g c d1g d1i t1 l1 h1)
              (htShift i g c d2g d2i t2 l2 h2)
              (uaddUshift d1g d1i d2g d2i dg di hg hi)
htShift i g c dg di (LetPair t1 t2) hlen (THLetPair d1 d2 a' b' h1 hb prf) =
  let (d1g ** d1i ** (Refl, l1)) = usplitLemma g i d1 t1 h1
      (d2g ** d2i ** (Refl, l2)) = uappendSplit (tlen i) d2
        (rewrite trans (predEq' (predEq' (shapeType (TSnoc (TSnoc (tappend g i) a') b') t2 hb))) (tappendLen g i) in
         rewrite plusCommutative (tlen g) (tlen i) in lteAddRight (tlen i))
      (hg, hi) = uaddSplitBoundary d1g d1i d2g d2i dg di
                   (trans l1 (sym l2)) (trans hlen (sym l1)) prf
  in THLetPair (uappend (USnoc d1g Zero) d1i) (uappend (USnoc d2g Zero) d2i) a' b'
               (htShift i g c d1g d1i t1 l1 h1)
               (htShift (TSnoc (TSnoc i a') b') g c d2g (USnoc (USnoc d2i One) One) t2
                  (cong S (cong S l2)) hb)
               (uaddUshift d1g d1i d2g d2i dg di hg hi)
htShift i g c dg di (Inl b0 t) hlen (THInl h) = THInl (htShift i g c dg di t hlen h)
htShift i g c dg di (Inr a0 t) hlen (THInr h) = THInr (htShift i g c dg di t hlen h)
htShift i g c dg di (Case t tL tR) hlen (THCase d1 d2 a' b' h hL hR prf) =
  let (d1g ** d1i ** (Refl, l1)) = usplitLemma g i d1 t h
      (d2g ** d2i ** (Refl, l2)) = uappendSplit (tlen i) d2
        (rewrite trans (predEq' (shapeType (TSnoc (tappend g i) a') tL hL)) (tappendLen g i) in
         rewrite plusCommutative (tlen g) (tlen i) in lteAddRight (tlen i))
      (hg, hi) = uaddSplitBoundary d1g d1i d2g d2i dg di
                   (trans l1 (sym l2)) (trans hlen (sym l1)) prf
  in THCase (uappend (USnoc d1g Zero) d1i) (uappend (USnoc d2g Zero) d2i) a' b'
            (htShift i g c d1g d1i t l1 h)
            (htShift (TSnoc i a') g c d2g (USnoc d2i One) tL (cong S l2) hL)
            (htShift (TSnoc i b') g c d2g (USnoc d2i One) tR (cong S l2) hR)
            (uaddUshift d1g d1i d2g d2i dg di hg hi)
htShift i g c dg di (Let q t1 t2) hlen (THLet d1 d2 _ a' h1 h2 prf) =
  let (d1g ** d1i ** (Refl, l1)) = usplitLemma g i d1 t1 h1
      (d2g ** d2i ** (Refl, l2)) = uappendSplit (tlen i) d2
        (rewrite trans (predEq' (shapeType (TSnoc (tappend g i) a') t2 h2)) (tappendLen g i) in
         rewrite plusCommutative (tlen g) (tlen i) in lteAddRight (tlen i))
      (hg, hi) = uaddSplitBoundary (uscale q d1g) (uscale q d1i) d2g d2i dg di
                   (trans (uscaleLen q d1i) (trans l1 (sym l2)))
                   (trans hlen (sym (trans (uscaleLen q d1i) l1)))
                   (rewrite sym (uscaleUappend q d1g d1i) in prf)
  in THLet (uappend (USnoc d1g Zero) d1i) (uappend (USnoc d2g Zero) d2i) q a'
           (htShift i g c d1g d1i t1 l1 h1)
           (htShift (TSnoc i a') g c d2g (USnoc d2i q) t2 (cong S l2) h2)
           (rewrite ushiftUscale q d1g d1i in
            uaddUshift (uscale q d1g) (uscale q d1i) d2g d2i dg di hg hi)
htShift i g c dg di (MkEcho m aE bE t) hlen (THEcho h) = THEcho (htShift i g c dg di t hlen h)
htShift i g c dg di (Weaken t) hlen (THWeaken h) = THWeaken (htShift i g c dg di t hlen h)

------------------------------------------------------------
-- 4c. Substitution core — accounting algebra
------------------------------------------------------------

||| Iterated weakening shift: `shiftn k = (shift 0)` iterated `k` times.
||| The cumulative shift the substituted term carries after descending `k`
||| binders (mirrors Coq `shiftn`).
public export
shiftn : Nat -> Tm -> Tm
shiftn Z     u = u
shiftn (S k) u = shift 0 (shiftn k u)

||| `ht_shift0`: weaken a derivation by one fresh `Zero`-usage binder — the
||| `htShift` instance at `i = TEmpty`.
public export
htShift0 : (g : Tctx) -> (c : Ty) -> (dg : Uvec) -> (t : Tm)
        -> Has g dg t a -> Has (TSnoc g c) (USnoc dg Zero) (shift 0 t) a
htShift0 g c dg t h = htShift TEmpty g c dg UEmpty t Refl h

||| Middle-four exchange for `qAdd` (pure semiring rearrangement): groups the
||| inner pair so the additive accounting can be re-associated.
qAddRearrange : (w, x, y, z : Q)
             -> qAdd (qAdd w x) (qAdd y z) = qAdd (qAdd w y) (qAdd x z)
qAddRearrange w x y z =
  trans (qAddAssoc w x (qAdd y z))
    (trans (cong (qAdd w) midSwap) (sym (qAddAssoc w y (qAdd x z))))
  where
    midSwap : qAdd x (qAdd y z) = qAdd y (qAdd x z)
    midSwap = trans (sym (qAddAssoc x y z))
                (trans (cong (\u => qAdd u z) (qAddComm x y)) (qAddAssoc y x z))

||| The Q-semiring identity behind substituting through an additive split
||| (mirrors Coq `q_reassoc`; derived from the named semiring laws, no
||| 3^6-case enumeration).
qReassoc : (dg1, dg2, du, q', q1, q2 : Q)
        -> qAdd (qAdd dg1 (qMul q' dg2)) (qMul (qAdd q1 (qMul q' q2)) du)
         = qAdd (qAdd dg1 (qMul q1 du)) (qMul q' (qAdd dg2 (qMul q2 du)))
qReassoc dg1 dg2 du q' q1 q2 =
  trans (cong (qAdd (qAdd dg1 (qMul q' dg2)))
              (trans (qMulDistribR q1 (qMul q' q2) du)
                     (cong (qAdd (qMul q1 du)) (qMulAssoc q' q2 du))))
    (trans (qAddRearrange dg1 (qMul q' dg2) (qMul q1 du) (qMul q' (qMul q2 du)))
           (cong (qAdd (qAdd dg1 (qMul q1 du)))
                 (sym (qMulDistribL q' dg2 (qMul q2 du)))))

||| `uadd D (Zero · E) = D` (lengths matched): the zero-scaled right summand
||| is the additive identity. (Mirrors Coq `uadd_uscaleZero_r`.)
public export
uaddUscaleZeroR : (d, e : Uvec) -> ulen d = ulen e -> uadd d (uscale Zero e) = Just d
uaddUscaleZeroR UEmpty UEmpty _ = Refl
uaddUscaleZeroR UEmpty (USnoc _ _) prf = void (zNotS' prf)
uaddUscaleZeroR (USnoc _ _) UEmpty prf = void (sNotZ' prf)
uaddUscaleZeroR (USnoc d qd) (USnoc e qe) prf =
  rewrite uaddUscaleZeroR d e (predEq' prf) in
  rewrite qMulZeroL qe in
  rewrite qAddZeroR qd in Refl
