<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->
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
      axiom-free)                                     E1 ☐ Buchholz ordinal global WF (11/13
 ──bridge──                                                → close or fence)
 R1 ✓ q_reassoc de-concretise (now SoloCore.v)        E2 ☐ (epi,mono) image factorisation —
 R2 ✓ SEMIRING functor; Include Linear3 =                 decide the 1 truncation postulate
      R0, axiom-free (consolidated)                  E3 ☐ proof-CI gate (agda --safe)
 R3 ✓ ORDERED_SEMIRING + subusage rule →             E4 ◑ SEAM: Measure : Residue E → R,
      Affine4 (affine_pres becomes DISTINCT,               Coq mirror ✓ (indep upstream-cited)
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
      S1.1 ✓ subject reduction sr_comm +
      config_subject_reduction; S1.2 ✓ session
      fidelity + progress/deadlock-freedom;
      S1.3 ✓ a:n-ary choice, c:congruence
      (wt_congr), b-core:μ type-layer
      (unfold_mu/dual_unfold/guarded); all
      axiom-free. S1.3b-meta μ-typing/SR = open)
 S2 ◑ duet = ensemble │2-party by projection
      (S2.0 ✓ gty/proj/two_party + witnesses;
      S2.1 ✓ projection_duality + projected_*
      {wf, subj.red., fidelity, progress};
      S2.2 ✓ choice GBra + μ GMu/GVar, PARTIAL
      proj + merge/merge_idem, option-form
      duality; S3a ✓ n≥3 projection_total
      (projectable_wf, first n-party theorem);
      S3b ✓ static n-party config (role_assignment,
      wf_assignment, conf_is_role_assignment2);
      S3c.0 ✓ full label-union merge (umerge,
      umerge_idem); S3c.1 ✓ union-projection
      (proj_u, projection_total_u, projectable_wf
      _implies_u); S3c.2 ✓ n-ary located opsem
      (nstep/gstep, ring run, adequacy-only);
      S3c.3-msg ✓ head-coupled message subject
      reduction (nstep_sr_msg_head, first EARNED
      n-party safety half); S3c.3-choice ✓
      head-coupled select/branch SR
      (nstep_sr_choice_head, uninvolved arm via
      proj_uninv_selected/merge_forces_eq); all
      axiom-free. S3c.3-perm + S3c.4 progress = open)
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
- **S2.0/S2.1 DONE — duet by projection, axiom-free 2026-06-14.** The duet thesis
  "duet = ensemble │ 2-party" rendered as a theorem, appended to `SessionPi.v`. Adds the
  multiparty-SHAPED global-type layer `gty` (message-passing + end), the three-case projection
  `proj G r` (sender `!` / receiver `?` / uninvolved passthrough — faithful to
  `formal-system.md:250-252`; `role := nat` so the uninvolved branch is *syntactically* reachable),
  and the two-party restriction `two_party p q G`. **`projection_duality`** (`Qed`, axiom-free):
  for a two-party choreography, `proj G p = dual (proj G q)` — projection yields DUAL binary local
  types (`p≠q` is load-bearing — at `p=q` it is FALSE, witnessed by a self-send). **`projected_config_wf`**
  + the corollaries `projected_config_subject_reduction` / `projected_session_fidelity` /
  `projected_config_progress` (+ the round-trip `global_projects_to_dual_channel`) transport the
  WHOLE S1.1b/S1.2 guarantee across projection: a config built from the two role-projections is
  well-formed BY CONSTRUCTION and inherits subject reduction, fidelity, and progress/deadlock-freedom.
  Witnesses `gpingpong`/`g3` project by `reflexivity` (the 3-party `g3` fires all three projection
  branches on one term and is provably *not* `two_party 0 1`); `projected_pingpong_deadlock_free` is
  a one-line instance. All axiom-free, CI-guarded (dedicated `proofs.yml` step). **Honest fence:**
  message-passing + end ONLY — the syntax is multiparty-shaped but every theorem's `two_party`
  hypothesis collapses to a single fixed pair, so this INSTANTIATES the thesis on two-party
  choreographies; it does NOT prove a general n-party result, and `proj` is total only because
  there is no choice (no merge / no coherence / no projectability predicate). OUT: choice global
  types (S2.2), μ-recursion (S2.2), n≥3 coherence/merge/projection-existence (S3). **Echo-types
  audit: NOT-RELEVANT** (axis-2 STRUCTURE; projection emits no obligation/residue — re-checked for
  the multiparty-syntax layer specifically).
- **S2.2 DONE — choice + recursion by projection, axiom-free 2026-06-14.** Design-panel-validated
  (the panel empirically compiled the proposed fixpoints on Coq 8.18 and caught a real **None-erasure
  soundness bug** in `proj_uninv` + two faithfulness overclaims BEFORE any code was written). Global
  types gain labelled **choice** `GBra p→q:{lᵢ:Gᵢ}` (dedicated mutual `gbranch`, NOT `list`) and
  equi-recursive **`GMu`/`GVar`** (de Bruijn aligned with the local `SMu`/`SVar`); projection becomes
  **PARTIAL** (`proj : gty → role → option sty`, three-way mutual `proj`/`proj_br`/`proj_uninv`) —
  sender→`SSelect`, receiver→`SBranch`, uninvolved→**merge** of the branches. **`merge`** + keystone
  **`merge_idem : merge s s = Some s`** (mutual `sty_mut` scheme) make an uninvolved role's projection
  DEFINED exactly when the branches AGREE. **`projection_duality`** reproved in **option-map form**
  `proj G p = option_map dual (proj G q)` via the mutual `two_party_mut` scheme (the `GBra` case needs
  the per-branch motive `proj_br p = option_map dual_br (proj_br q)`); the whole S2.0/S2.1 bridge +
  four corollaries rethreaded through `proj … = Some _` (`proj_q_dual_p_some`). Witnesses: choice
  (`gchoice2`) + recursive (`grec = μX.0→1:Nat.X`) duality, partial projection in BOTH directions, the
  **tail-conflict regression** `proj_tail_conflict_none` locking the panel's None-erasure fix, and
  merge's two failure modes. All axiom-free, dedicated `proofs.yml` S2.2 gate. **Honest fences:**
  (1) `merge` is the **plain/identity-MEET** merge (= equality on the message fragment), NOT the full
  label-set-UNION merge — deferred S3; (2) `GMu` projection is **UNPRUNED** (a non-participating role
  yields the non-contractive `μX.X`, rejected by `guarded`, only OUTSIDE the `two_party {p,q}` collapse
  — non-theorematic, never shown as a role's type); (3) **NO global-level metatheory** (SR/fidelity/
  progress not re-proved for `GBra`/`GMu`; the bridge transports binary metatheory only where both
  projections are `Some`); (4) `dual_unfold` NOT consumed (the `GMu` case uses only structural
  `dual(SMu x)=SMu(dual x)`). OUT → S3: n≥3 coherence / full-UNION merge / projection-existence.
  **Echo-types audit: NOT-RELEVANT** (axis-2 STRUCTURE).
- **S3a DONE — n-party projection totality, axiom-free 2026-06-14.** The **FIRST theorem quantifying
  over a genuine n≥3 role space** — it breaks the standing fence that no prior theorem quantified over
  a real n-party system. Design-panel-validated (4 lenses, empirically compiled on Coq 8.18; the panel
  forced the honest scope). Keystone **`projection_total : projectable_wf G → ∀ r:role, ∃ s,
  proj G r = Some s`** (via mutual `projectable_wf_mut`): every role of a well-branched global type
  projects. **`projectable_wf`** (mutual `projectable_wf_br`) = the restricted projectability predicate
  — at every `GBra`: `p≠q`, non-empty choice, inductive body coherence, and the **merge-existence
  clause** (`∀ r∉{p,q}, ∃ s, proj_uninv bs r = Some s`); `μ` permitted, handled by the keystone.
  Witnesses pin **non-vacuity at n=3**: a 3-party **ring** (`g_ring`) projects on all 3 roles and is
  not `two_party`; an **agreeing** 3-party choice (`g_choice3`) projects its uninvolved role via a
  non-trivial `merge`; and the plain-merge **boundary** is witnessed — a same-direction different-payload
  choice (`g_excluded`) is a SAFE protocol full-union admits but plain merge **rejects** (`~ projectable_wf`).
  **Honest scope (panel-mandated renames + fences):** `projectable_wf` asserts projection EXISTENCE,
  **NOT** safety (the name is deliberately not "coherent"); no n-party SR / progress / fidelity.
  **Fences:** (1) projection-existence only, no safety; (2) plain / identity-meet merge — divergent-
  uninvolved choices EXCLUDED (full-UNION = **S3c**); (3) `μ` projection unpruned (guardedness not
  imposed); (4) the general `two_party → projectable_wf` is **NOT proved and is FALSE** (`two_party`
  admits empty choices + μ-divergent branches that break the absent-role merge) — n=2 collapse
  witnessed BY EXAMPLE (`projectable_gpingpong`); (5) no n-party config / operational semantics —
  static role-assignment tuple (**S3b**) and full-union merge + n-party metatheory (**S3c**) deferred.
  **Echo-types audit: NOT-RELEVANT** (axis-2 STRUCTURE).
- **S3b DONE — static n-party configuration, axiom-free 2026-06-14.** The FIRST n-party
  *configuration* form: a STATIC role→endpoint assignment with no operational semantics (S3a gave
  projection totality on the type side; S3b gives the container of endpoint *processes*).
  `role_assignment := list (role * party)`; **`wf_assignment G ra`** — a plain `Definition` over a
  `Prop`, In-based (NOT nth/length, NOT a Fixpoint/Inductive, so the S1.3a guard/positivity wall is
  sidestepped) — asserts ONLY that every listed `(r,P)` is typed at `G`'s projection on `r`. Keystone
  **`conf_is_role_assignment2`**: the binary `Conf`/`wf_config` is the n=2 INSTANCE (from
  `two_party p q G` + `p≠q` + the two entries, via `projected_config_wf`), with the converse embed
  `role_assignment2_of_conf` and the lookup bridge `ra_get`/`ra_get_in`/`wf_assignment_get`.
  Non-vacuity at n=3 by `wf_ra_ring` over `g_ring` (concrete closed endpoints; NOT a single `Conf` —
  `length = 3` + `g_ring_not_two_party`, BY EXAMPLE). Honesty witnesses pin the boundary: duplicate-key
  tolerance (no functionality theorem) and `projectable_but_uncoverable` (`GVar 3` is `projectable_wf`
  yet `proj … = Some (SVar 3)` is uninhabited by any closed party via `svar_uninhabited` → NO general
  "projectable ⇒ coverable"). **Honest scope:** `wf_assignment` = typed-at-projection, **NOT** n-party
  safety (no SR/progress/fidelity/coherence); the n=2 recovery is a SLICE not an n→2 collapse; STATIC
  only; plain (not union) merge + unpruned μ inherited; `role_assignment` is an association list, not a
  verified map (duplicate-tolerant, vacuous on `[]`). Deferred to **S3c**: n-party operational
  semantics, n-party SR/progress/fidelity, full-union merge + coherence.
  **Echo-types audit: NOT-RELEVANT** (axis-2 STRUCTURE).
- **S3c.0 DONE — full label-union merge, axiom-free 2026-06-14.** The first S3c sub-rung,
  design-panel-validated (4 lenses, empirically compiled on Coq 8.18; the panel adjudicated the
  keystone empirically — `umerge_idem` is **unconditional**, the `bremove`-threaded design beating a
  `branch_free`-fenced alternative). The Honda-Yoshida-Carbone label-**UNION** merge `umerge`/`umerge_br`
  is **added ALONGSIDE** the plain identity-meet `merge` (touches nothing S2/S3a/S3b use — baseline
  rebuild byte-identical). At an **external** choice (`SBranch &`) a merged uninvolved role offers the
  UNION of the branches' labels (shared continuations recursively `umerge`d, unshared carried over); the
  message fragment + the **internal** choice (`SSelect ⊕`) require EQUALITY (`sty_eqb`) — unioning a
  *sender's* choice is HYC-unsound. Keystone **`umerge_idem : ∀ s, umerge s s = Some s`**. The WIN is
  witnessed: a different-**label** `&` choice plain `merge` rejects, `umerge` unions
  (`umerge_widens_strictly`). **Fences:** TYPE-ALGEBRA only — NOT wired into `proj`/`projectable_wf`/`cstep`
  (= S3c.1); NO coherence/safety claim (the name is deliberately not `coherent_merge`); `SSelect ⊕` is NOT
  unioned (the ⊕/& asymmetry is load-bearing); it does **NOT** unlock the `g_excluded` (different-PAYLOAD)
  class — `umerge` unions *labels*, not payloads (`umerge_still_rejects_payload_divergence`); only
  idempotence proved (not commutative/associative — later rungs reason up to branch-SET equality).
  **Remaining S3c (solo):** S3c.1 union-projection + coherence predicate; S3c.2 n-ary located config +
  `nstep` (operational adequacy); S3c.3 message-fragment n-party subject reduction (the first *earned*
  n-party safety half); S3c.4 n-party progress (research-hard — fence, witness binary collapse +
  3-cycle deadlock by example). **Echo-types audit: NOT-RELEVANT** (axis-2 STRUCTURE).
- **S3c.1 DONE — union-projection, axiom-free 2026-06-14.** The second S3c sub-rung,
  design-panel-validated (4 lenses, empirically compiled on Coq 8.18; the panel found the keystone
  and corrected the agreement-lemma structure before any code landed). A **separate** union-projection
  `proj_u`/`proj_br_u`/`proj_uninv_u` (the plain `proj` with the uninvolved fold's `merge` replaced by
  `umerge`) is **added ALONGSIDE** `proj` — `proj`/`merge`/`projectable_wf`/`projection_total` and all
  binary metatheory stay **byte-identical** (`proj` is **NOT** re-pointed at `umerge` — S3c.0 fence 5).
  Keystone existence theorem **`projection_total_u : projectable_u_wf G → ∀ r, ∃ s, proj_u G r = Some s`**
  (the union analogue of `projection_total`, same mutual-scheme proof; `umerge`'s partiality never
  inspected — the merge-existence is a `PWu_Bra` hypothesis). The **monotonicity bridge**
  **`projectable_wf_implies_u : projectable_wf G → projectable_u_wf G`** (one-directional DOMAIN-INCLUSION;
  with `proj_agrees_u` the two projections coincide on the plain-projectable subset) rides on the keystone
  **`merge_forces_eq : merge s₁ s₂ = Some s → s₁ = s₂`** (plain merge is the identity-meet — exactly the
  invariant already asserted in the `merge` fence comment). **Non-vacuity** `g_union3` (uninvolved role 2
  sees `&{3:end}` vs `&{4:end}` — different **labels**, same direction) is genuinely in
  *union-projectable minus plain-projectable*: `proj g_union3 2 = None` ∧ `~projectable_wf g_union3` but
  `proj_u g_union3 2 = Some &{3:end,4:end}` ∧ `projectable_u_wf g_union3`. **Fences:** EXISTENCE only —
  NO n-party SR/progress/safety (= S3c.3+); strictly ADDITIVE; `umerge` unions **labels not payloads** —
  the different-PAYLOAD class is STILL rejected (`proj_u g_excluded 2 = None`, `g_excluded_u`); monotonicity
  is DOMAIN-INCLUSION not behavioural refinement (converse FALSE via `g_union3`; provable-cheap ONLY because
  this `merge` is positional-same-label — a regression pin on `merge_forces_eq`); `SSelect ⊕` asymmetry +
  order-dependence + unpruned `μ` inherited from S3c.0. **Remaining S3c (solo):** S3c.2 n-ary located
  config + `nstep`; S3c.3 message-fragment n-party subject reduction (first *earned* n-party safety half);
  S3c.4 n-party progress (research-hard — fence). **Echo-types audit: NOT-RELEVANT** (axis-2 STRUCTURE).
- **S3c.2 DONE — n-ary located operational semantics, axiom-free 2026-06-14.** The third S3c sub-rung
  and the FIRST operational metatheory over an n-party config (S3a/S3b/S3c.1 were all type-side / static).
  A located reduction **`nstep : role_assignment → role_assignment → Prop`** — the located mirror of the
  fused binary `cstep`: one synchronous communication between two distinct roles `ra_set`-updates exactly
  those two endpoints (`NStep_Comm` message + `NStep_Sel` select), every other role untouched (`p`
  universally quantified, so cstep's fixed-left/right `CStepR`/`CSelR` are subsumed). Plus the functional
  update `ra_set` (+ `ra_set_get_eq`/`ra_set_get_neq`/`ra_set_length`), a global-type reduction **`gstep`**
  (message fragment), the reflexive-transitive closures `nstar`/`gstar`, and the functional well-formedness
  **`wf_assignment_f`** (the ra_get form S3c.3's SR will use; `wf_assignment → wf_assignment_f`, one
  direction, so S3b's `wf_ra_ring` transfers). **Adequacy witnessed on both sides:** the 3-party ring runs
  `0→1→2→0` to all-`QEnd` — `ring_runs_to_end : nstar ra_ring ra_ring3` (three `NStep_Comm`) and
  `g_ring_gsteps : gstar g_ring GEnd`; `nstep_length`/`nstar_length` give the structural-adequacy fact that
  communication preserves the role count. **The load-bearing honesty witness `nstep_breaks_wf_at_fixed_G`**
  PROVES `nstep` does **not** preserve `wf_assignment_f` at a *fixed* G (after one step role 0 is a receiver
  while `proj g_ring 0` is a send type) — exactly *why* n-party subject reduction (S3c.3) must be stated
  against a *stepping* G, and why `wf_assignment ra → deadlock-free` is false. **Fences:** ADEQUACY ONLY —
  no SR (the refutation above), no progress / deadlock-freedom (the ring run is *by example*, = S3c.4),
  `nstep`/`gstep` not yet coupled (no simulation/fidelity, = S3c.3); message+select fragments only; `ra_set`
  first-match (duplicate-tolerant); uses plain `proj` (independent of the S3c.1 widening). **Remaining S3c
  (solo):** S3c.3 message-fragment n-party SR (coupled `gstep`/`nstep` vs a stepping G — the first *earned*
  n-party safety half); S3c.3-choice; S3c.4 n-party progress (research-hard — fence). **Echo-types audit:
  NOT-RELEVANT** (axis-2 STRUCTURE).
- **S3c.3-msg DONE — head-coupled message subject reduction, axiom-free 2026-06-14.** The fourth S3c
  sub-rung and **the first EARNED n-party safety half** — design-panel-validated + **independently
  adversary-verified** (4 design lenses on Coq 8.18 → coq-mechanic build → a separate adversary that
  re-instantiated the premises, re-derived the payload coupling, and compiled its own run-ahead refutation;
  verdict *sound-and-honest, land as-is*). S3c.2 proved `nstep` does **not** preserve `wf_assignment_f` at
  a *fixed* `G`; S3c.3-msg proves the TRUE statement — the **head** communication carries wf to the
  continuation: **`nstep_sr_msg_head : p≠q → wf_assignment_f (GMsg p q t G') ra → ra_get ra p = Some
  (QSend v P) → ra_get ra q = Some (QRecv Q) → wf_assignment_f G' (ra_set (ra_set ra p P) q (open_party v
  Q))`**. Proved by a 3-way located role case-split (sender `p` / receiver `q` / uninvolved `r`), the
  **coupling** being that the SAME value `v` at the SAME payload type `t` the sender ships (`pty_send_inv`)
  is what the receiver substitutes (`pty_subst0`); uninvolved roles transfer because `proj (GMsg p q t G')
  r = proj G' r` is *definitional* for `r∉{p,q}`. The firing pair is **structurally pinned** to `G`'s head
  by stating wf at `GMsg p q t G'` — no separate hypothesis needed. A coupled corollary
  **`nstep_gstep_sr_msg_head`** puts `gstep`+`nstep`+`wf'` in one conclusion for the head step. **Earned-safety
  headline:** the SAME `ra_ring1` that S3c.2's `nstep_breaks_wf_at_fixed_G` refuted at the fixed `g_ring`
  **is** wf at the *stepped* `g_ring` — `ra_ring1_wf_at_stepped_g` (proved **through** the core SR), sealed
  by `sr_earns_safety_across_step` (¬wf-at-`g_ring` ∧ wf-at-stepped ∧ the paying `gstep`). The
  **run-ahead fence is self-witnessed**: `runahead_breaks_head_coupling` proves a non-head `nstep` fires
  from a wf config yet is **not** wf at the head continuation — so head-coupled SR genuinely *cannot* cover
  it (boundary-by-example, cf. `g_excluded`). **Honest fences:** HEAD communication only (run-ahead /
  permutation = a bigger swap theory, S3c.3-perm); MESSAGE fragment only (select = S3c.3-choice); the
  coupled corollary is a head *wrapper*, NOT general SR / bisimulation; functional wf (`wf_assignment_f`);
  PRESERVATION only — NO progress / deadlock-freedom / fidelity (S3c.4+). The naming deliberately avoids the
  M1/S3a overclaim trap (`_msg_head`, not `n_party_safety`). **Remaining S3c (solo):** S3c.3-choice
  (select/branch SR, reusing `pty_sel_inv`/`pty_bra_inv`/`pget`); S3c.3-perm (run-ahead, needs a
  gstep-with-swap relation); S3c.4 n-party progress (research-hard — fence). **Echo-types audit:
  NOT-RELEVANT** (axis-2 STRUCTURE).
- **S3c.3-choice DONE — head-coupled select/branch subject reduction, axiom-free 2026-06-15.** The fifth S3c
  sub-rung — the CHOICE analogue of S3c.3-msg. Design-panel-validated (3 lenses) + **independently
  adversary-verified** (a SEPARATE adversary recompiled all 11 modules clean, re-instantiated at a fresh
  4-role label-1 choreography exercising the recursive arms of `gbget`/`proj_br_selected`/`proj_uninv_selected`,
  and ran run-ahead + disagreeing-branch break attempts; verdict *sound-and-honest, land as-is*). The HEAD
  select/branch sanctioned by a head `GBra p q {lᵢ:Gᵢ}` carries wf to its SELECTED branch continuation
  `Gl = gbget l bs`: **`nstep_sr_choice_head`** (same 3-way located role split as the message head — selector
  `p` via `pty_sel_inv`, offerer `q` via `pty_bra_inv` + the per-label coverage, uninvolved `r`). The label `l`
  drives BOTH the new global step **`GStep_Bra`** (on the new functional gbranch lookup **`gbget`** — no such
  lookup pre-existed; only `proj_br`/`proj_uninv` consumed `gbranch`) and the located **`NStep_Sel`** onto the
  SAME `Gl`. **The genuinely-new part:** the uninvolved arm is no longer DEFINITIONAL (as in the message head)
  — it rides `proj_uninv` (the plain-merge fold) and is discharged by the new **`proj_uninv_selected`** via
  **`merge_forces_eq`** (plain merge = identity-meet ⇒ every branch projects an uninvolved `r` to the SAME type,
  so the selected branch does too; the ≥2-branch case needs an explicit one-level unfold to avoid `cbn` over-
  unfolding). Coupled corollary `nstep_gstep_sr_choice_head`; non-vacuity `choice3_head_fires_end_to_end` on the
  agreeing 3-party `g_choice3`. Strictly ADDITIVE (bytes 1-2868 SHA-identical to baseline; sole prior-region
  edit = `gbget` + the `GStep_Bra` constructor). **Fences:** HEAD choice only (run-ahead = S3c.3-perm); PLAIN
  `proj` (label-UNION merge = S3, with a **regression pin** — widening `merge` breaks `proj_uninv_selected` and
  the theorem becomes false-as-stated); coupled corollary is a head wrapper, NOT general SR; PRESERVATION only
  (NO progress = S3c.4). Naming `_choice_head`, not `n_party_safety`. **Remaining S3c (solo):** S3c.3-perm
  (run-ahead), S3c.4 n-party progress (research-hard — fence). **Echo-types audit: NOT-RELEVANT** (axis-2 STRUCTURE).
- **S1.3 DONE — choice / congruence / μ type-layer, axiom-free 2026-06-14.** The last/hardest S1
  sub-rungs, design-panel-validated. **S1.3a (choice):** n-ary LABELLED select/branch (`⊕/&{lᵢ:Sᵢ}`)
  in the fused form; `sty`/`party` gain `SSelect`/`SBranch`/`QSel`/`QBra` over dedicated mutual
  inductives `sbranch`/`pbranch` (NOT `list`, so `dual`/`psubst_party` are well-guarded mutual
  fixpoints — Coq rejects the `List.map` form); the three fused theorems
  (`config_subject_reduction`/`session_fidelity`/`config_progress`) extended with the choice cases,
  label-mismatch safety expressible via the functional lookups `bget`/`pget`. **n-ary, not binary**
  — required so S2.2's `merge` (over label sets) is statable. **S1.3c (congruence):** `wt_congr`
  (typing preserved under structural congruence on the open `proc`), the par laws `P|0≡P`/comm/assoc,
  via new `csplit_comm`/`csplit_assoc` and `wt_absorb_ended`; fenced as typing-PRESERVATION (NOT
  SR-up-to-≡), with ν-extrusion/ν-swap/replication named OUT. **S1.3b-core (μ type-layer):**
  `SMu`/`SVar`, `tlift`/`tsubst`/`unfold_mu` (one structural pass, terminates on `μa.a`),
  `dual_unfold` (future S1.3b-meta infra — S2.2's `projection_duality` consumed only the structural
  `dual(SMu x)=SMu(dual x)`, NOT `dual_unfold`), `guarded` contractiveness.
  **DEFERRED — S1.3b-meta:** μ typing/subject-reduction up-to-unfolding (needs a `PT_Unfold` rule +
  its soundness + up-to-unfolding inversions); S2.2 (now DONE) consumed only the type layer.
  **Echo-types audit: NOT-RELEVANT** (axis-2 STRUCTURE).
- **Echo side (echo-types):** `echo.index.thinposet`, `echo.modality.core`, and
  `echo.separation.notresourceinstance` are done. The Coq `EchoMode.v` / `EchoResidue.v` /
  `TEcho` are already **`Quantity`-independent**, so the measure seam attaches without
  disturbing the resource layer.
- **E4 SEAM — Coq mirror DONE, axiom-free 2026-06-14.** The gating step ("name the residue
  algebra with a composition operator") is satisfied **upstream**: the echo-types audit found
  the seam already mechanised on both foundations — tropical-resource-typing
  `Resource/EchoBridge.lean` (`structure ResidueMeasure` with `combine`/`empty` +
  `measure_empty`/`measure_combine` monoid-hom laws) and echo-types
  `Echo/Measure/Interface.agda` (`ResidueMeasure` record) + `Echo/Separation/NotResourceInstance.agda`
  (the `equal-measure-does-not-imply-equal-echo` measure-INDEPENDENCE theorems, `--safe --without-K`).
  Per the echo-types-audit directive the move is **reuse, not re-invent**: the former PENDING prose
  sketch in `ResourceAlgebra.v` is now the real `Module Type RESIDUE_MEASURE (S : SEMIRING)`
  (`measure : Residue → S.Q`, `measure_empty`/`measure_combine` — a monoid homomorphism, needs only
  the DONE R2 SEMIRING), and `EchoMeasure.v` **inhabits** it at the DONE R4 Tropical cost carrier
  (`EchoTraceTropical`: residue = echo reindexing trace, measure = accumulated Affine-collapse cost)
  + witnesses measure-independence on the Coq side (`echo_measure_not_injective`). **Fences:** the
  Coq mirror carries the measure HOMOMORPHISM + residue-level non-injectivity; the full
  measure-independence over the degrade-compose modality (b: `ECHO_MODALITY` `act`) is upstream-CITED,
  not re-derived — hence E4 is `◑` (Coq mirror ✓, full seam upstream). **Hard IS-NOT invariant kept:**
  Echo is NOT a resource/SEMIRING instance; `measure` is a one-directional E→R decoration; the echo
  modules stay `Quantity`-independent. (E3 `agda --safe` gate is already green upstream + mirrored by
  my-lang's Coq `proofs.yml` — the ladder `☐` is stale.) **Echo-types audit: RELEVANT (axis-3 ECHO).**

---

## 5. Pointers
- Phased roadmap: [`ALIGNMENT-PLAN.md`](./ALIGNMENT-PLAN.md) (AffineScript parity).
- Authoritative status: [`STATUS.md`](./STATUS.md).
- Working brief / handoff seed (off-repo): `~/developer/dev-notes/my-lang/MY-LANG-FOUNDATIONS-BRIEF.md`.
- Foundations: `hyperpolymath/echo-types` (axis 3), `hyperpolymath/tropical-resource-typing` (axis 1 witness).
