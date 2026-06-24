<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->
<!-- Owner: Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk> -->
# Formal Verification

My Language treats **mechanised formal verification as a first-class
deliverable**, alongside the Rust implementation. The semantic core of the
**solo** dialect (affine / Quantitative Type Theory) is mechanised *twice* — in
**Coq** and **Idris2** — so the two flagship metatheories track the same shape
and cross-check each other.

> The single authoritative status registry is
> [`proofs/STATUS.md`](../../../proofs/STATUS.md). **No proof is described as
> "proved" until a proof assistant accepts it.** This page is a readable
> companion, not the source of truth.

## Where the proofs live

| Path | Contents |
|------|----------|
| `proofs/verification/coq/solo-core/` | Coq mechanisation (`*.v`) |
| `proofs/verification/idris/solo-core/` | Idris2 mechanisation (`*.idr`, `solo-core.ipkg`) |
| `proofs/STATUS.md` | Authoritative proof-status registry |
| `proofs/ALIGNMENT-PLAN.md` | Phased roadmap (F1.0 … F5) toward AffineScript parity |
| `proofs/**.md` | Paper proofs (~6.3k lines) — type system, effects, memory, dialects |

## The solo-core kernel

The mechanised kernel is a simply-typed λ-calculus + Unit + pairs + sums + `let`,
where **every binder carries a QTT quantity** `q ∈ {0, 1, ω}`:

- `0` — erased (compile-time only),
- `1` — linear (used exactly once),
- `ω` — unrestricted.

Typing (`Has`/`has_type`) is the standard QTT presentation with **explicit
context splitting** (`ctxAdd` / `ctxScale`) at application, pairing, `let`, and
`case`. Affine weakening lives in the quantity ordering (`0 ≤ 1`), so a linear
resource may go unused.

## Status (2026-06-05)

| Phase | What | Status |
|-------|------|--------|
| F1.0 | QTT semiring + laws (both tracks) | ✅ **proved** (exhaustive case analysis) |
| F1.1 | CBV small-step operational semantics (`Step`/`step`) | ✅ **committed** (both tracks) |
| F1.3 | **Progress** | ✅ **proved** — Coq `Theorem … Qed.` (axiom-free); Idris total, hole-free |
| F1.4 | **Preservation** + QTT substitution lemma | ⏳ statement-only — **gated** on the product/elimination decision ([#93](https://github.com/hyperpolymath/my-lang/issues/93)) |
| F1.2 | Four-point affine semiring (`0 ⊏ ? ⊏ 1 ⊏ ω`) | ⏳ planned |
| F5 | **Proof CI** | ✅ **live** — `.github/workflows/proofs.yml` |

**Progress** says: a closed, well-typed solo term is either a value or can take a
step. The Coq proof is closed under the global context (no `Admitted`/`Axiom`);
the Idris proof is a total function with no holes (`%default total` makes a green
`idris2 --build` the totality certificate).

### Open question gating preservation

The kernel currently pairs a **multiplicative** product introduction (`THPair`
splits the context) with **projective** eliminators (`Fst`/`Snd`). That
combination is unsound for *preservation* on open terms — projecting discards the
other component's linear resources. Resolving it (additive `&`, multiplicative
`⊗` with `let`-pair elimination, or both) is tracked in
[#93](https://github.com/hyperpolymath/my-lang/issues/93) and is the next step on
the verification track.

## Echo-types in the type system

[`echo-types`](https://github.com/hyperpolymath/echo-types) — a formal account of
*loss that is not total erasure* — is integrated as a **first-class type former
in the formal kernel**, not merely the Rust checker:

- `TEcho mode dom cod` — the echo residue type former (`mode Echo<dom => cod>`);
- `MkEcho` / `Weaken` terms; `THEcho` / `THWeaken` typing rules; echo reduction
  rules (progress covers them);
- `EchoMode` — the `Linear ⊑ Affine` thin poset with the **weaken** and
  **no-section** laws;
- `EchoResidue` — the proof-layer object that pins exactly the laws behind the
  Rust checker's `Ty::is_assignable_from` Echo case (its 5 unit tests *are* these
  laws).

The claim line is held deliberately narrow (per echo-types' R-2026-05-18): Echo is
a **loss-graded reindexing modality over a thin poset** — not a graded comonad,
universal property, or adjunction. See the
[echo-types integration design note](../../design/echo-types-integration.md) and
[Type System](../language/types.md#echo-types).

## Reproducing the proofs

```bash
# Coq (8.18)
cd proofs/verification/coq/solo-core
coq_makefile -f _CoqProject -o CoqMakefile && make -f CoqMakefile

# Idris2 (0.7.0)
cd proofs/verification/idris/solo-core
idris2 --build solo-core.ipkg
```

CI runs both on every PR that touches `proofs/` (`.github/workflows/proofs.yml`)
and asserts `progress` is axiom-free.

## Discipline

- No `Admitted` / `Axiom` / `postulate` / `sorry` / `believe_me` / `assert_*`.
- **Statements-first**: theorems are committed as statements, then discharged;
  a typed hole (`?todo_*`) or a bare `Definition : Prop` is an *obligation*, never
  a result.
- Architecture decisions are recorded as ADRs in
  [`.machine_readable/6a2/META.a2ml`](../../../.machine_readable/6a2/META.a2ml).
