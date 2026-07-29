-- SPDX-License-Identifier: MPL-2.0
-- SPDX-FileCopyrightText: 2026 Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk>
--
-- Echo linearity mode for the my-lang Solo core.
--
-- The two-point thin poset `Linear <= Affine` that decorates echo
-- residues. Mirrors `Mode` and `_<=m_` in the echo-types Agda library
-- (EchoLinear.agda) and the Rust `EchoMode` in
-- crates/my-lang/src/types.rs. The headline facts — reflexivity,
-- transitivity, the weaken direction `Linear <= anything`, and the
-- *no-section* fact `not (Affine <= Linear)` — are the type-algebra
-- content behind the checker's Echo subtyping. Coq twin: EchoMode.v.

module EchoMode

%default total

||| Linearity mode of an echo residue.
|||
||| * `Linear` — full residue; distinctions retained.
||| * `Affine` — collapsed residue; distinctions weakened away.
public export
data Mode : Type where
  Linear : Mode
  Affine : Mode

%name Mode m, m1, m2, m3

||| The mode ordering `mle m1 m2`: an echo at mode `m1` may be
||| *weakened* to mode `m2`. `Linear` weakens to anything; `Affine`
||| weakens only to itself. This is `EchoLinear._<=m_`.
public export
mle : Mode -> Mode -> Bool
mle Linear _      = True
mle Affine Affine = True
mle Affine Linear = False

------------------------------------------------------------
-- Poset laws (exhaustive case split — no axioms)
------------------------------------------------------------

||| Reflexivity.
public export
mleRefl : (m : Mode) -> mle m m = True
mleRefl Linear = Refl
mleRefl Affine = Refl

||| Transitivity.
public export
mleTrans : (m1, m2, m3 : Mode)
        -> mle m1 m2 = True -> mle m2 m3 = True -> mle m1 m3 = True
mleTrans Linear _      _      _    _    = Refl
mleTrans Affine Affine Affine _    _    = Refl
mleTrans Affine Affine Linear _    Refl impossible
mleTrans Affine Linear _      Refl _    impossible

||| Antisymmetry — the poset is *thin* (two-point).
public export
mleAntisym : (m1, m2 : Mode)
          -> mle m1 m2 = True -> mle m2 m1 = True -> m1 = m2
mleAntisym Linear Linear _    _    = Refl
mleAntisym Affine Affine _    _    = Refl
mleAntisym Linear Affine _    Refl impossible
mleAntisym Affine Linear Refl _    impossible

||| `EchoLinear.weaken` direction: `Linear` is the bottom — a linear
||| residue weakens to any mode.
public export
mleLinearBot : (m : Mode) -> mle Linear m = True
mleLinearBot _ = Refl

||| Concretely, `Linear` weakens to `Affine`.
public export
weakenLinearAffine : mle Linear Affine = True
weakenLinearAffine = Refl

||| `EchoLinear.no-section-weaken`: weakening is irreversible. An
||| `Affine` residue can NOT be used where a `Linear` one is required —
||| the weakening map has no section.
public export
noSectionWeaken : mle Affine Linear = False
noSectionWeaken = Refl
