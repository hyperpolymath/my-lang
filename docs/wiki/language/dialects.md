<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->
<!-- Owner: Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk> -->
# Dialects

My Language is a **progressive-disclosure language family**: complexity is
revealed in layers, and only one layer is authoritative in the current
foundation release (`f0`).

## The nested hierarchy

The dialects are **three, nested** — each is a conservative extension of the one
before:

```
Solo  ⊂  Duet  ⊂  Ensemble
```

| Dialect | Adds | Semantic core | f0 status |
|---------|------|---------------|-----------|
| **Solo** | affine types + arena allocation, single-agent | QTT (`{0,1,ω}`) λ-calculus; **mechanised** (Coq + Idris2) | **authoritative** |
| **Duet** | session types, two-party protocols | session-typed extension | planned |
| **Ensemble** | the agent / orchestration calculus, multi-agent | agent calculus | planned |

### Solo is the only authoritative dialect in f0

Per the repo's scope-arrest anchor
([`ANCHOR.scope-arrest.2026-01-01.Jewell.scm`](../../../ANCHOR.scope-arrest.2026-01-01.Jewell.scm)):
**only Solo is in scope in f0.** Duet and Ensemble are tagged *planned* and **must
not ship partial semantics** — their design lives in paper proofs
(`proofs/duet/`, `proofs/ensemble/`) and the [roadmap](../roadmap/overview.md),
not in shipping code. This containment is deliberate: make one dialect real and
correct before widening.

The Solo kernel's metatheory (progress proved; preservation in progress) is
documented under [Formal Verification](../internals/formal-verification.md).

## Me is *not* a fourth dialect

**Me** is an **on-the-fly, agent-generated projection** *over* the dialect
hierarchy — a runtime view tailored to a learner/operator, specified in the
ecosystem's `tentacles-agentic-syllabus` repo — **not** a static compiler or a
fourth dialect. Two earlier attempts to build Me as a static dialect were retired
(`hyperpolymath/me-dialect`, archived; the in-tree `my-lang/dialects/me/` sidelined
to `_exploratory/me-scaffolding/`). Do not treat Me as a static dialect.

## Not dialects

- **`frontier-practices/`** — the my-lang-specific *applied learner layer* that
  consumes the common curriculum. An applied layer, not a dialect.
- **`my-ssg/`, `playground/`** — tooling / scratch, outside the language.

## See also

- [Type System](types.md) — the QTT/affine core and Echo types
- [Formal Verification](../internals/formal-verification.md) — the mechanised Solo core
- [Roadmap](../roadmap/overview.md)
