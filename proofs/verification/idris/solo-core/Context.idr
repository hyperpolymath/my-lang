-- SPDX-License-Identifier: MPL-2.0
-- SPDX-FileCopyrightText: 2026 Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk>
--
-- Separated QTT context for the Solo core of my-lang (Coq twin: the
-- Usage layer of SoloCore.v).
--
-- The standard QTT presentation, adopted for the F1.4 preservation
-- track (design decision 2026-06-12, ADR-006): the TYPE context `Tctx`
-- (types only) is FIXED across a derivation's splits, while the USAGE
-- vector `Uvec` (quantities only) is what gets added/scaled. Because
-- addition and scaling now act purely on quantities, the algebra is
-- genuinely clean — `uadd` is commutative and associative with no
-- type-shape asymmetry (the wart of the old conflated `Ctx`, where
-- `ctxAdd` took the type from its first argument).

module Context

import Quantity
import Syntax

%default total

------------------------------------------------------------
-- Type context — types only
------------------------------------------------------------

||| A snoc-list of types. `TEmpty` at the base; `TSnoc G a` pushes a
||| binding of type `a`. Fixed across a derivation's context splits.
public export
data Tctx : Type where
  TEmpty : Tctx
  TSnoc  : Tctx -> Ty -> Tctx

%name Tctx g, g1, g2

public export
tlen : Tctx -> Nat
tlen TEmpty       = 0
tlen (TSnoc g _)  = S (tlen g)

------------------------------------------------------------
-- Usage vector — quantities only, shape-matched to a Tctx
------------------------------------------------------------

||| A snoc-list of quantities, shape-matched to a `Tctx`. This is what
||| the typing judgement adds and scales.
public export
data Uvec : Type where
  UEmpty : Uvec
  USnoc  : Uvec -> Q -> Uvec

%name Uvec d, d1, d2

public export
ulen : Uvec -> Nat
ulen UEmpty      = 0
ulen (USnoc d _) = S (ulen d)

||| The all-zero usage of a type context's shape ("nothing is used").
public export
uzero : Tctx -> Uvec
uzero TEmpty      = UEmpty
uzero (TSnoc g _) = USnoc (uzero g) Zero

||| Scalar multiplication of a usage vector.
public export
uscale : Q -> Uvec -> Uvec
uscale _ UEmpty       = UEmpty
uscale q (USnoc d qe) = USnoc (uscale q d) (qMul q qe)

||| Pointwise addition of usage vectors — partial (defined on equal
||| shapes; in a derivation both summands range over the same `Tctx`).
public export
uadd : Uvec -> Uvec -> Maybe Uvec
uadd UEmpty UEmpty = Just UEmpty
uadd (USnoc d1 q1) (USnoc d2 q2) =
  case uadd d1 d2 of
    Nothing => Nothing
    Just d  => Just (USnoc d (qAdd q1 q2))
uadd _ _ = Nothing

------------------------------------------------------------
-- Small disjointness / injectivity helpers
------------------------------------------------------------
--
-- Stated locally via `Refl impossible` (constructor disjointness) and a
-- one-line successor-injectivity, so the algebra does not depend on
-- which `Uninhabited`/`Injective` instances a given Prelude version
-- happens to export.

predEq : S n = S m -> n = m
predEq Refl = Refl

sNotZ : S n = Z -> Void
sNotZ Refl impossible

zNotS : Z = S n -> Void
zNotS Refl impossible

justInj : Just x = Just y -> x = y
justInj Refl = Refl

nothingNotJust : Nothing = Just x -> Void
nothingNotJust Refl impossible

usnocNotUEmpty : USnoc d q = UEmpty -> Void
usnocNotUEmpty Refl impossible

------------------------------------------------------------
-- The clean algebra (the payoff of separation)
------------------------------------------------------------

||| Length of the zero usage matches the type context.
public export
uzeroLen : (g : Tctx) -> ulen (uzero g) = tlen g
uzeroLen TEmpty      = Refl
uzeroLen (TSnoc g a) = cong S (uzeroLen g)

||| Scaling preserves length.
public export
uscaleLen : (q : Q) -> (d : Uvec) -> ulen (uscale q d) = ulen d
uscaleLen q UEmpty       = Refl
uscaleLen q (USnoc d qe) = cong S (uscaleLen q d)

||| Scaling by One is the identity (monoid action unit).
public export
uscaleOne : (d : Uvec) -> uscale One d = d
uscaleOne UEmpty       = Refl
uscaleOne (USnoc d qe) = rewrite uscaleOne d in rewrite qMulOneL qe in Refl

||| Scaling composes (monoid action of (Q, qMul, One)).
public export
uscaleCompose : (q1, q2 : Q) -> (d : Uvec)
             -> uscale q1 (uscale q2 d) = uscale (qMul q1 q2) d
uscaleCompose q1 q2 UEmpty       = Refl
uscaleCompose q1 q2 (USnoc d qe) =
  rewrite uscaleCompose q1 q2 d in rewrite qMulAssoc q1 q2 qe in Refl

||| Scaling absorbs the zero usage.
public export
uscaleZero : (q : Q) -> (g : Tctx) -> uscale q (uzero g) = uzero g
uscaleZero q TEmpty      = Refl
uscaleZero q (TSnoc g a) = rewrite uscaleZero q g in rewrite qMulZeroR q in Refl

||| Addition is COMMUTATIVE — clean, no type-shape caveat (the win).
public export
uaddComm : (d1, d2 : Uvec) -> uadd d1 d2 = uadd d2 d1
uaddComm UEmpty UEmpty             = Refl
uaddComm UEmpty (USnoc d2 q2)      = Refl
uaddComm (USnoc d1 q1) UEmpty      = Refl
uaddComm (USnoc d1 q1) (USnoc d2 q2) with (uadd d1 d2) proof p
  uaddComm (USnoc d1 q1) (USnoc d2 q2) | Nothing =
    rewrite uaddComm d2 d1 in rewrite p in Refl
  uaddComm (USnoc d1 q1) (USnoc d2 q2) | Just d =
    rewrite uaddComm d2 d1 in rewrite p in rewrite qAddComm q1 q2 in Refl

||| `uzero` is a LEFT identity for addition (on matching shapes).
public export
uaddZeroL : (g : Tctx) -> (d : Uvec) -> ulen d = tlen g -> uadd (uzero g) d = Just d
uaddZeroL TEmpty UEmpty _ = Refl
uaddZeroL TEmpty (USnoc d q) prf = void (sNotZ prf)
uaddZeroL (TSnoc g a) UEmpty prf = void (zNotS prf)
uaddZeroL (TSnoc g a) (USnoc d q) prf =
  rewrite uaddZeroL g d (predEq prf) in rewrite qAddZeroL q in Refl

||| `uzero` is a RIGHT identity for addition (on matching shapes).
public export
uaddZeroR : (g : Tctx) -> (d : Uvec) -> ulen d = tlen g -> uadd d (uzero g) = Just d
uaddZeroR g d prf = rewrite uaddComm d (uzero g) in uaddZeroL g d prf

||| If `uadd d1 d2 = Just d`, the two summands have equal length.
public export
uaddLenEq : (d1, d2, d : Uvec) -> uadd d1 d2 = Just d -> ulen d1 = ulen d2
uaddLenEq UEmpty UEmpty _ _ = Refl
uaddLenEq UEmpty (USnoc _ _) _ prf = void (nothingNotJust prf)
uaddLenEq (USnoc _ _) UEmpty _ prf = void (nothingNotJust prf)
uaddLenEq (USnoc d1 q1) (USnoc d2 q2) d prf with (uadd d1 d2) proof p
  uaddLenEq (USnoc d1 q1) (USnoc d2 q2) d prf | Nothing = void (nothingNotJust prf)
  uaddLenEq (USnoc d1 q1) (USnoc d2 q2) d prf | Just dd =
    cong S (uaddLenEq d1 d2 dd p)

||| If `uadd d1 d2 = Just d`, the result has the length of the summands.
public export
uaddLen : (d1, d2, d : Uvec) -> uadd d1 d2 = Just d -> ulen d = ulen d1
uaddLen UEmpty UEmpty d prf = rewrite sym (justInj prf) in Refl
uaddLen UEmpty (USnoc _ _) _ prf = void (nothingNotJust prf)
uaddLen (USnoc _ _) UEmpty _ prf = void (nothingNotJust prf)
uaddLen (USnoc d1 q1) (USnoc d2 q2) d prf with (uadd d1 d2) proof p
  uaddLen (USnoc d1 q1) (USnoc d2 q2) d prf | Nothing = void (nothingNotJust prf)
  uaddLen (USnoc d1 q1) (USnoc d2 q2) d prf | Just dd =
    rewrite sym (justInj prf) in cong S (uaddLen d1 d2 dd p)

||| Addition is TOTAL on equal-length usages.
public export
uaddTotal : (d1, d2 : Uvec) -> ulen d1 = ulen d2 -> (d : Uvec ** uadd d1 d2 = Just d)
uaddTotal UEmpty UEmpty _ = (UEmpty ** Refl)
uaddTotal UEmpty (USnoc _ _) prf = void (zNotS prf)
uaddTotal (USnoc _ _) UEmpty prf = void (sNotZ prf)
uaddTotal (USnoc d1 q1) (USnoc d2 q2) prf =
  let (d ** eq) = uaddTotal d1 d2 (predEq prf) in
  (USnoc d (qAdd q1 q2) ** rewrite eq in Refl)

||| Distributivity of scaling over addition (right form): if
||| `uadd d1 d2 = Just d` then `uadd (uscale q d1) (uscale q d2) = Just (uscale q d)`.
public export
uscaleAdd : (q : Q) -> (d1, d2, d : Uvec)
         -> uadd d1 d2 = Just d
         -> uadd (uscale q d1) (uscale q d2) = Just (uscale q d)
uscaleAdd q UEmpty UEmpty d prf = rewrite sym (justInj prf) in Refl
uscaleAdd q UEmpty (USnoc _ _) _ prf = void (nothingNotJust prf)
uscaleAdd q (USnoc _ _) UEmpty _ prf = void (nothingNotJust prf)
uscaleAdd q (USnoc d1 q1) (USnoc d2 q2) d prf with (uadd d1 d2) proof p
  uscaleAdd q (USnoc d1 q1) (USnoc d2 q2) d prf | Nothing = void (nothingNotJust prf)
  uscaleAdd q (USnoc d1 q1) (USnoc d2 q2) d prf | Just dd =
    rewrite uscaleAdd q d1 d2 dd p in
    rewrite sym (justInj prf) in
    rewrite qMulDistribL q q1 q2 in Refl

------------------------------------------------------------
-- Inversions for the closed-context (Empty) splits
------------------------------------------------------------
--
-- The empty usage splits only trivially: a sum is `UEmpty` only when
-- both summands are, and a scaling is `UEmpty` only when its argument
-- is. These recover `UEmpty`/`uzero` sub-usages so the closed-term
-- recursive calls (which speak about the empty context) apply.

||| Scaling yields the empty usage only from the empty usage.
public export
uscaleEmpty : (q : Q) -> (d : Uvec) -> uscale q d = UEmpty -> d = UEmpty
uscaleEmpty q UEmpty _ = Refl
uscaleEmpty q (USnoc _ _) prf = void (usnocNotUEmpty prf)

||| Addition yields the empty usage only from two empty usages.
public export
uaddEmpty : (d1, d2 : Uvec) -> uadd d1 d2 = Just UEmpty -> (d1 = UEmpty, d2 = UEmpty)
uaddEmpty UEmpty UEmpty _ = (Refl, Refl)
uaddEmpty UEmpty (USnoc _ _) prf = void (nothingNotJust prf)
uaddEmpty (USnoc _ _) UEmpty prf = void (nothingNotJust prf)
uaddEmpty (USnoc d1 q1) (USnoc d2 q2) prf with (uadd d1 d2)
  uaddEmpty (USnoc d1 q1) (USnoc d2 q2) prf | Nothing = void (nothingNotJust prf)
  uaddEmpty (USnoc d1 q1) (USnoc d2 q2) prf | Just dd = void (usnocNotUEmpty (justInj prf))
