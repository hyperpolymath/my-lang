# My Language Wiki

Welcome to the official wiki for **My Language** — a **progressive-disclosure,
multi-dialect programming language** with a **Quantitative Type Theory (QTT)
affine core**, built so that **mechanised formal verification is a first-class
deliverable** alongside the Rust implementation.

> **Identity in one sentence:** a language *family* with progressive complexity
> levels (`Solo ⊂ Duet ⊂ Ensemble`, plus the agent-generated *Me* projection)
> whose authoritative `f0` layer — **Solo** (affine + arena allocation) — is
> mechanised in Coq *and* Idris2, and integrates
> [`echo-types`](https://github.com/hyperpolymath/echo-types) for *structured
> loss* directly in the type system.

## Quick links

- [Dialects](language/dialects.md) — the `Solo ⊂ Duet ⊂ Ensemble` hierarchy (+ Me)
- [Type System](language/types.md) — QTT/affine core and Echo types
- [Formal Verification](internals/formal-verification.md) — the mechanised Solo core
- [Language Tour](language/tour.md)
- [Roadmap](roadmap/overview.md)

## What is My Language?

My Language is a statically-typed, multi-dialect language. Its authoritative
foundation dialect, **Solo**, is an **affine** language (resources used at most
once) over a **QTT `{0, 1, ω}`** core, compiled by a Rust toolchain. Higher
dialects (Duet, Ensemble) extend Solo conservatively; **Me** is an on-the-fly,
agent-generated *projection* over the hierarchy, not a separate compiler. AI
integration is one capability among many — not the defining feature.

### Key characteristics

| Aspect | Description |
|--------|-------------|
| **Affine / QTT core** | Every binder carries a quantity `0` (erased), `1` (linear), or `ω` (unrestricted) |
| **Formally verified** | Solo-core mechanised in **Coq + Idris2**; *progress* proved, *preservation* in progress |
| **Echo types** | `echo-types` integrated into the type system — *loss that is not total erasure* (a loss-graded reindexing modality) |
| **Progressive disclosure** | `Solo ⊂ Duet ⊂ Ensemble`; only Solo is authoritative in `f0` |
| **Rust implementation** | The compiler/interpreter is implemented in Rust |
| **Effects & contracts** | Effect tracking and pre/post conditions (design + paper proofs) |

## Documentation sections

### Language

- [Dialects](language/dialects.md)
- [Type System](language/types.md)
- [Syntax Overview](language/syntax.md)
- [Functions & Closures](language/functions.md)
- [Effects & Capabilities](language/effects.md)
- [Memory Management](language/memory.md)
- [Modules & Imports](language/modules.md)
- [Concurrency](language/concurrency.md)
- [AI Features](language/ai-features.md)
- [Language Tour](language/tour.md)

### Reference

- [Keywords](reference/keywords.md)
- [Operators](reference/operators.md)
- [Grammar (EBNF)](reference/grammar.md)
- [Standard Library](reference/stdlib.md)

### Internals (for compiler developers)

- [Architecture Overview](internals/architecture.md)
- [Formal Verification](internals/formal-verification.md)
- [Lexer](internals/lexer.md)
- [Parser](internals/parser.md)
- [Type Checker](internals/type-checker.md)
- [Standard Library Design](internals/stdlib-design.md)
- [Checker Allocation Investigation](internals/checker-allocation-investigation.md)
- [Contributing](internals/contributing.md)

### Guides

- [Getting Started](guides/getting-started.md)
- [Installation](guides/installation.md)

### Tutorials

- [Language Basics](tutorials/basics.md)

### Tooling

- [Tooling Overview](tooling/overview.md)

### Roadmap

- [Overview](roadmap/overview.md)
- [Language](roadmap/language.md)
- [Compiler](roadmap/compiler.md)
- [Tooling](roadmap/tooling.md)
- [Ecosystem](roadmap/ecosystem.md)

> **Note:** pages not yet written (e.g. additional tutorials, per-tool guides,
> the Duet/Ensemble dialect references) are tracked in the
> [roadmap](roadmap/overview.md) rather than linked as stubs here, so the wiki
> stays free of dead links.

## Project status

- **Version:** `0.1.0` (early-alpha) — see
  [`.machine_readable/6a2/STATE.a2ml`](../../.machine_readable/6a2/STATE.a2ml)
  for the authoritative state, and [`proofs/STATUS.md`](../../proofs/STATUS.md)
  for the proof-status registry.
- **Scope:** Solo dialect only in `f0` (per the
  [scope-arrest anchor](../../ANCHOR.scope-arrest.2026-01-01.Jewell.scm)).

## Contributing

See [`CONTRIBUTING.md`](../../CONTRIBUTING.md) and
[internals/contributing.md](internals/contributing.md).

## License

My Language is open source under the **Mozilla Public License 2.0 (MPL-2.0)** —
see [`LICENSE`](../../LICENSE). Every source file carries an
`SPDX-License-Identifier: MPL-2.0` header.
