<!-- SPDX-License-Identifier: MPL-2.0 -->
<!-- SPDX-FileCopyrightText: 2026 Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk> -->

# my-lang Axis Architecture — the proof-factoring map

**Status:** active · **Created:** 2026-06-13 · **Owner:** @hyperpolymath

This is the conceptual parent of [`ALIGNMENT-PLAN.md`](./ALIGNMENT-PLAN.md) (the phased
roadmap) and [`STATUS.md`](./STATUS.md) (what is actually proved). It records *how the
proof effort factors* and the **one invariant that must not erode**.

> The mechanised solo-core (Coq, axiom-free `progress`/`preservation`) is **one rung on one
> axis**. The architecture below explains why the other work is not "more of the same."

---

## 1. Four orthogonal axes

my-lang factors into four axes that compose but **must not be collapsed into each other**.
Each factors *differently* — the design mistake would be a single strategy (all top-down or
all bottom-up) for what are genuinely different kinds of structure.

```
 AXIS              MEMBERS                          HOW IT FACTORS             DIRECTION
 ───────────────────────────────────────────────────────────────────────────────────────
 1 RESOURCE        linear {0,1,ω} · affine          CUT ACROSS                 parametric
   (quantity)        {0,?,1,ω} · tropical max-plus   prove Soundness ONCE       (the "bridge")
                     (cost) · min-max (bottleneck)   over a semiring →
                                                     the rest are INSTANCES
 2 STRUCTURE       solo (λ/QTT) ⊂ duet (sessions)   STACK, asymmetric          bottom-up solo;
   (judgements)      ⊂ ensemble (π-calculus)         duet ⊂ ensemble (project)  ensemble-first
                                                     but  solo ⊄ ensemble        on process side
 3 MODALITY        echo (loss-grading)              SEPARATE LAYER             own metatheory
   (graded poset)                                   thin poset; own theorems   (foundations repo)
                                                     (no-section, degrade-comp)
 4 SURFACE         me (visual / blocks)             FREE (elaboration)         desugar → solo,
                                                     adequacy lemma, not new      reuse soundness
                                                     progress/preservation
```

Key structural facts (verified 2026-06-13):
- **`solo ⊄ ensemble`** — the sequential QTT λ-calculus is *not* a fragment of the
  π-calculus; a top-down "pivot to ensemble" would **not** subsume solo. Keep solo as the
  proven foundation; do the *process* side top-down (ensemble-first, recover duet by
  projection).
- **`me` is surface, not metatheory** — covering it is an elaboration-correctness lemma over
  the already-proven solo core, not a fresh soundness proof.

---

## 2. Vocabulary — "echo-grade" is **deprecated**

The phrase "echo-grade" straddles two different things and invites the resource and modality
axes to re-collapse. Use exactly three terms:

| Term | Axis | Meaning |
|------|------|---------|
| **resource grade** | 1 | the `q` annotating a binder: usage / cost / latency — linear, affine, tropical |
| **echo index** | 3 | the thin-poset modality index: `keep ≤ residue ≤ forget` (and `linear ⊑ affine`) |
| **residue measure** | seam | a semiring-valued *observation* attached to a residue |

Directionality: resource algebras are **produced** by the semiring functor; the echo layer
**consumes** one as a measure. Echo never *produces* a `Soundness` instance.

---

## 3. The invariant — **Echo IS-NOT a resource instance**

> *Echo Types, as currently intended and mechanised, are not an instance of the my-lang
> resource/Soundness functor. Any semiring encoding would be a lossy measure or quotient,
> not the Echo foundation itself.*

This is load-bearing; treat it as an estate IS-NOT registry entry. **Why it holds:**

1. **Category.** `Soundness(S)` is QTT λ-calculus progress/preservation over a *usage*
   algebra `S`. Echo's grades are not usage counts; Echo's theorems (no-section,
   degrade-compose) need *thinness* (`≤g-prop`, a poset fact), not a semiring. An "echo
   instance of `Soundness`" is ill-typed against the interface, or proves the wrong theorem.
2. **Mechanised separation (in echo-types).** `sep-degrade-compose-fails`: drop thinness and
   you get a *checked* `true ≢ false`. Plus the matched-negative that the entropy / numeric
   **shadow is blind where Echo distinguishes**. Encoding Echo as a semiring shadow is itself
   a lossy collapse — the shadow is a *residue* of Echo, not Echo. *Backing artifacts (echo-types):*
   `echo.index.thinposet`, `echo.modality.core`, and `echo.separation.notresourceinstance`.

**Interaction, not identification.** The axes couple at exactly one seam:

```
  ResourceAlgebra R          -- axis 1 (semiring)
  EchoModality   E           -- axis 3 (thin poset)
  Measure : Residue E → R     -- the ONLY coupling
```

Echo core stays **measure-independent**; `Cost` / `Prob` / `Tropical` are decorations *at the
seam*. (cf. `!` kept distinct from the multiplicatives in linear logic, related by precise
rules.) Tropical proves the quantity axis is real; echo proves the modality axis is real;
my-lang is parametric over **both**, and identifies **neither**.

---

## 4. The proof ladder (rungs + dependencies)

`✓` done · `◑` in flight · `☐` open. See `STATUS.md` for the authoritative per-artefact state.

```
 LANGUAGE THREAD (this repo)                        FOUNDATIONS THREAD(S) (sibling repos)
 ─────────────────────────────────────             ──────────────────────────────────────
 R0 ✓ solo progress + preservation (Coq,            ECHO  (echo-types, Agda)
      axiom-free)                                     E1 ☐ Buchholz ordinal global WF (10/13
 ──bridge──                                                → close or fence)
 R1 ✓ q_reassoc de-concretise (now SoloCore.v)        E2 ☐ (epi,mono) image factorisation —
 R2 ✓ SEMIRING functor; Include Linear3 =                 decide the 1 truncation postulate
      R0, axiom-free (consolidated)                  E3 ☐ proof-CI gate (agda --safe)
 R3 ✓ ORDERED_SEMIRING + subusage rule →             E4 ☐ SEAM: Measure : Residue E → R,
      Affine4 (affine_pres becomes DISTINCT,               echo core proven measure-INDEP
      not an alias) ── makes "affine" real           E5 ☐ Pillar E paper write-up
 R4 ✓ Tropical instance (cost) ── acceptance
      test: infinite carrier, analytic laws         TROPICAL  (tropical-resource-typing, Lean/Isabelle)
 R5 ✓ static-split ≡ usage-walk (F1.4 tail) ──       T1 ☐ independently re-verify Isabelle 2025-1
      VERIFIED [check] + aff_type_dec, axiom-free     T2 ☐ session ext: choice / recursion / multiparty
 ──parity / surface──                                T3 ☐ firm the Lean↔Isabelle cross-links
 P1 ☐ Idris twin of solo (close ?todo) +             T4 ☐ WithTop ∞ (drop tcZero=1e6 hack)
      mirror subst2 fix + parametric design
 M1 ✓ me elab→solo (M1.0/1.1/1.1b, whole me_tm)           SEAM CAPSTONE (joint): E4 needs R2 (a SEMIRING)
 ──structure climb (last, hardest)──                 + a solid echo core. It is where "interaction,
 S1 ◑ ensemble π metatheory (S1.0 ✓ core+witnesses;        not identification" becomes a theorem.
      S1.1a ✓ comm-redex SR sr_comm + value subst;
      S1.1b ✓ closed-system SR config_subject_reduction
      (fused (νc)(P∣Q)); S1.2 session fidelity = open)
 S2 ☐ duet = ensemble │2-party  (by projection,
      falls out of S1)
```

Dependency spine: `R0 → R1 → R2 → {R3, R4} ; R3 → R5` ; `R2 + echo-core → E4` ; `S1 → S2`.
`P1`, `M1`, and the foundations ladders run **in parallel** with the bridge.

### 4.1 Progress — 2026-06-13

- **R1 done.** `q_reassoc` (`Soundness.v`) is now derived from the named semiring laws
  (`qmul_distrib_l/r`, `qmul_assoc`, `qadd_assoc/comm`), not a 729-case carrier
  enumeration. `progress`/`preservation`/`affine_pres` re-verified axiom-free.
- **R2 boundary drafted.** `ResourceAlgebra.v` defines `Module Type SEMIRING` — the **10
  equational laws the functorised proof consumes** — plus an inert `ORDERED_SEMIRING`
  extension, and validates them via `Module Linear3 <: SEMIRING` (reusing `Quantity`) and
  a sealed `Linear3_Sealed : SEMIRING`. The contract is machine-pinned: **`qle`,
  `qmul_comm` and `qmul_one_r` are provably absent** — grade *addition* must be commutative
  (the separated-context payoff), but grade *multiplication need not be*, and the order is
  inert until a subusage rule exists. *Remaining (R2 proper):* functorize `Soundness` over
  `SEMIRING`.
- **The 10th law (`qmul_zero_l`) — functorisation audit, 2026-06-13.** The first boundary
  draft listed nine laws and excluded `qmul_zero_l`. An adversarial abstract-carrier probe
  of `Soundness.v:1022-1028` (`uadd_uscaleZero_r`, on the live preservation path via
  `hv_subst`/`ht_subst`) showed `uscale Zero` leaves `qmul zero qe` **irreducible** without
  it: the concrete proof only closed because `simpl` computes it on `|Q|=3`. So
  `qmul_zero_l` is required *for the abstraction* (it joins `qadd_zero_l`, already named for
  the same simpl-reduction reason) and is now the 10th `SEMIRING` law — discharged
  axiom-free by `Quantity.qmul_zero_l`. `qmul_comm`/`qmul_one_r` stay out (reached by
  neither citation nor reduction).
- **Diffusion finding (adversarial audit).** `|Q|=3` is **not** isolated at `q_reassoc` —
  the `Quantity.v` laws are themselves carrier enumerations reached independently. So
  carrier-abstraction requires the Module-Type lift (each instance supplies its own laws);
  there is no single-seam shortcut.
- **Functorisation strategy — Coq generativity, probe-verified 2026-06-13.** Coq module
  functors are **generative for inductives**: re-applying `SyntaxF M` inside a downstream
  `UsageF` yields a *different* `ty` (verified — `probe1` fails). So the naïve "each file is
  `Module F (M:SEMIRING)`, chained by re-application" is broken. Two paths share inductives
  (both probe-verified): **consolidated** (one functor spanning Syntax+Usage+Typing+Soundness,
  instantiated once) or **signature-threaded** (each layer's instance passed as a module
  parameter via a per-layer `Module Type`).
- **R2 DONE — consolidated, executed 2026-06-13.** The four chain files are merged into
  `SoloCore.v` as `Module SoloCoreF (M : SEMIRING)`; `Include SoloCoreF Linear3` recovers
  `progress`/`preservation`/`affine_pres` **axiom-free** (`Print Assumptions` closed; clean
  rebuild + CI updated). What made it near-mechanical: `Import M` resolves every operator/law
  name to `M`'s fields, and `Local Notation Zero/One` re-aliases the two literal tokens
  (single-token identifiers like `uadd_uscaleZero_r` untouched). Only **five** sites needed
  real fixes, each where the abstract carrier no longer computes `qmul`/`qadd` on literals:
  `uscale_one` (qmul_one_l), `uadd_zero_l` (qadd_zero_l), `uadd_ushift` (qadd_zero_r), the
  left-annihilation `qmul zero qe` in `uadd_uscaleZero_r` (qmul_zero_l — the 10th law), and a
  `qadd_zero_r`-before-`congruence` in `subst2_lemma`. `Context.v`/`ContextProps.v` (dead
  branch, kept concrete on `Quantity`) and `EchoResidue.v` were repointed to `SoloCore`; the
  *transparent* `Linear3` (never `Linear3_Sealed`) keeps the concrete re-export compatible.
  `SoloCoreF` is now ready to instantiate at tropical/affine algebras (R4).
- **R3 DONE — affine layer, executed 2026-06-13.** `affine_pres` is no longer an alias of
  `preservation`. Realisation (the **budget-wrapper**, chosen over the Affine4-instance and the
  in-judgement subusage-rule options): the functor parameter widened to `ORDERED_SEMIRING` —
  the linear proofs still cite only `SEMIRING`, so they are untouched and stay axiom-free.
  `ule` = pointwise `qle`; `aff_type G D t a := ∃ D0, has_type G D0 t a ∧ ule D0 D` ("uses at
  most budget `D`"). `has_type_aff` embeds linear ⊆ affine; `aff_weaken` relaxes the budget
  upward (the affine *discard* a strictly-linear system forbids); `AffinePreservation`
  (budget-stable preservation) is proved axiom-free by riding the linear `preservation` at the
  realised usage. `Quantity.qle_trans` was added and `ORDERED_SEMIRING` is now a live preorder
  (`qle_refl`/`qle_trans`/`qle_zero`). The Affine4 4-point semiring stays available through the
  same functor as a future R4 instance.
- **R5 DONE — verified usage-walk checker, executed 2026-06-13.** The static
  context-splitting judgement `has_type` is proved equivalent to an EXECUTABLE
  one-pass synthesiser `check : tctx → tm → option (ty × uvec)`: `check_sound`
  (only well-typed terms accepted), `check_complete` (every derivation recovered
  with EXACTLY its usage — usage determinacy internalised, corollary
  `typing_unique`), and `check_correct : has_type G D t a ↔ check G t = Some (a, D)`
  — all real `Qed`, axiom-free (`Print Assumptions` closed, CI-guarded). The
  calculus is synthesis-directed (every former carries its annotations), so a single
  bottom-up walk decides typing; it needs only `Q_eq_dec` (added to
  `ORDERED_SEMIRING`, discharged by both instances), which the soundness proofs do
  not. `Module SoloTropical := SoloCoreF Tropical` inherits the verified checker at
  the INFINITE min-plus carrier for free (R4-style acceptance test). This is the F1.4
  tail and **overtakes AffineScript**, whose solo-core states the same
  static-split≡usage-walk equivalence only as prose ("an explicit equivalence lemma
  is future work"). The checker decides the strictly-LINEAR `has_type`; the affine
  *discard* (`aff_type`/`ule`, R3) is a separate budget layer, now also decided —
  **R5b DONE:** `ule_dec` (the budget order), `aff_type_iff` (the `check`-characterisation
  `aff_type G D t a ↔ ∃ D0, check G t = Some (a, D0) ∧ ule D0 D`) and `aff_type_dec`
  (decidability of the affine judgement) are axiom-free and inherited free at the
  tropical carrier too. Several `Compute`-backed examples pin non-vacuity: a linear
  binder used once is accepted; one dropped (usage `Zero`) or duplicated
  multiplicatively (usage `Omega`) is rejected — the latter accepted under an `Omega`
  binder, so the rejection is exactly the linearity check. The affine *discard* is the
  precise separator: the SAME `UnitT` at budget `One` is `aff_type`-accepted (realises
  `Zero ≤ One`) yet `has_type`-rejected.
- **M1 DONE (partial) — me→solo elaboration adequacy, axiom-free 2026-06-14.** The visual /
  block `me` surface (`proofs/me/`, no settled AST in code — only the paper block grammar in
  `visual-semantics/formal-model.md`) is pinned as a Coq `me_tm` (the affine/token fragment)
  with `elab : me_tm → tm` landing in the de Bruijn solo core — the mechanised analogue of the
  paper `translate` (which targets Rust-ish surface syntax and is never mechanised). **M1.0:**
  `Example`s execute `elab` + the R5 `check` on the concrete carrier (linear token
  create-and-consume accepted; the dropped token rejected linearly but accepted via sequencing
  and via the R3 `aff_type` layer; the echo linear→affine bridge end-to-end). **M1.1:** the
  UNIVERSAL adequacy theorem `elab_data_check : ∀ e G, me_data e = true → ∃ a, check G (elab e)
  = Some (a, uzero G)` (formal-model.md Theorem 1, Visual Soundness) for the no-linear-use
  fragment (data + discard-sequencing + injections), with corollaries `elab_data_typed`
  (`has_type` via `check_correct`) and `elab_data_aff_budget` (fits any affine budget ≥
  `uzero G`). All real `Qed`, `Print Assumptions` closed, CI-guarded. The first MECHANISED
  surface→core elaboration-correctness result in either sibling — a clean **overtake of
  AffineScript** (solo-only, no `me`-like dialect, nothing analogous as surveyed (AS@main 2026-06-02)).
  Echo-types audit: NOT-RELEVANT (axis-4 surface vs axis-3 modality). **M1.1b ✓ (2026-06-14):**
  a me-level typing judgement `me_wt` with `me_wt_sound : me_wt G D e a → has_type G D (elab e) a`
  (axiom-free) makes the linear-USE constructs (`MeVar`/`MeLet`/`MeUsePair`) UNIVERSAL, plus the
  faithful conditional `MeIf → Case` (the paper IfBlock = sum ELIM); corollaries `me_wt_check` /
  `me_wt_aff`. **MeSeq closed too** (`MW_Seq`, reusing the F1.4 `ht_shift0` weakening lemma — e1
  erased at multiplicity Zero, e2 weakened under the discarded binder), so `me_wt` spans the
  WHOLE `me_tm` (all 12 constructors): **M1 is complete.**
- **S1.0 DONE — ensemble session-π core, axiom-free 2026-06-14.** The first step of the
  structure climb (axis 2, the PROCESS side, done ensemble-first per `solo ⊄ ensemble`). A NEW
  standalone Coq development `SessionPi.v` (module `SoloCore.SessionPi`) — deliberately NOT an
  extension of `SoloCore.v`: processes are a different term language, so nothing carrier-bearing
  is imported (reuse audit confirmed). It mechanises a synchronous binary session-typed π core
  after `proofs/duet/session-types/` + `proofs/ensemble/agent-calculus/`: base-value payloads,
  session types `sty` (send/recv/end) with a computed duality + `dual_involutive` (`Qed`),
  polarised endpoints (`Pos`/`Neg`, `co_involutive`), processes `proc` (nil/send/recv/par/ν), a
  **linear** channel-typing judgement `wt` with context splitting `csplit`, and small-step `step`
  for the communication redex under par/ν contexts. Executable witnesses are real `Qed`: a
  well-typed ping-pong (`wt_pingpong`) that reduces (`step_pingpong`/`step_sys`). All axiom-free
  (`Print Assumptions` closed) and CI-guarded (`proofs.yml` dedicated assertion step). **Subject
  reduction is the S1.1 obligation — NOT claimed at S1.0.** Honest OUT (deferred): name-passing /
  mobility (payloads are base values ⇒ capture-safe, no scope extrusion), structural congruence
  + choice (S1.2), replication, mismatch, μ-recursion, bisimulation, the AI primitives, and
  multiparty `G`/projection (the S2 hook). **Echo-types audit: NOT-RELEVANT** — echo-types is
  axis-3 MODALITY, this is axis-2 STRUCTURE; an echo index would re-collapse the axes §3 forbids
  (so no `EchoMode`/`EchoResidue` artefact enters the file). Greenfield **overtake** vs
  AffineScript: AS@main (surveyed 2026-06-02) records no concurrency / session-types / π-calculus
  / multiparty metatheory in any form — a category AS does not enter.
- **Echo side (echo-types):** `echo.index.thinposet`, `echo.modality.core`, and
  `echo.separation.notresourceinstance` are done. The Coq `EchoMode.v` / `EchoResidue.v` /
  `TEcho` are already **`Quantity`-independent**, so the measure seam can attach without
  disturbing the resource layer. *Seam prerequisite:* `EchoResidue.v` currently exposes
  residues *operationally* (via the `Weaken` rule), not as an algebra with a composition
  operator — naming that algebra is the gating step before `Measure : Residue → R`.

---

## 5. Pointers
- Phased roadmap: [`ALIGNMENT-PLAN.md`](./ALIGNMENT-PLAN.md) (AffineScript parity).
- Authoritative status: [`STATUS.md`](./STATUS.md).
- Working brief / handoff seed (off-repo): `~/developer/dev-notes/my-lang/MY-LANG-FOUNDATIONS-BRIEF.md`.
- Foundations: `hyperpolymath/echo-types` (axis 3), `hyperpolymath/tropical-resource-typing` (axis 1 witness).
