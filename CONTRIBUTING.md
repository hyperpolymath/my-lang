<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->
<!-- SPDX-FileCopyrightText: 2026 Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk> -->

# Contributing to my-lang

Thanks for your interest. my-lang is an early-alpha research language: an
affine / Quantitative-Type-Theory core whose metatheory is mechanised in Coq
and Idris2, with a Rust compiler that is being brought into correspondence
with those proofs.

That last part shapes everything below. **Changes to typing rules are proof
obligations, not just code changes.** See [Proof-affecting changes](#proof-affecting-changes).

## Getting set up

```sh
# Clone
git clone https://github.com/hyperpolymath/my-lang.git
cd my-lang

# Initialise submodules (proof tracks and vendored specs)
just init

# Verify the toolchain works
just check      # fmt-check + lint + test
just test       # unit + conformance tests (excludes my-llvm)
```

You need a stable Rust toolchain. The LLVM back end (`my-llvm`) additionally
needs **system LLVM 21**, which is why the default `build`/`test` recipes
exclude it — use `just build-all` / `just test-all` when you have it.

Proof work additionally needs **Coq/Rocq** and/or **Idris2**:

```sh
just proofs        # both tracks
just proofs-coq    # the authoritative track
just proofs-idris  # the Idris2 twin
```

Run `just --list` for the full recipe set.

## Repository structure

```
my-lang/
├── crates/            # the Cargo workspace — all live compiler code
│   ├── my-lang/       #   core: lexer, parser, checker, interpreter, stdlib
│   ├── my-parser/     #   standalone parser
│   ├── my-qtt/        #   QTT kernel — the port of the verified Coq checker
│   ├── my-hir/  my-mir/  my-llvm/    # lowering pipeline → native
│   ├── my-cli/        #   the `my` binary
│   ├── my-fmt/ my-lint/ my-lsp/ my-dap/ my-debug/ my-pkg/ my-test/
│   └── my-ai/         #   AI-assist surface (mock operations — see DEBT.md)
├── proofs/            # mechanised metatheory (Coq + Idris2) + STATUS.md
├── dialects/          # dialect definitions (solo, duet)
├── docs/              # design notes, ADRs, and the in-tree wiki
├── spec/  conformance/  tests/  fuzz/    # specification and test surfaces
├── examples/          # sample programs
├── .machine_readable/ # machine-facing state, contractiles, service metadata
└── Justfile           # task runner — the golden path
```

Note: the root `src/`, `lib/` and `tests/` trees are **orphaned** — not
referenced by any crate. They are tracked as debt in [`DEBT.md`](DEBT.md);
do not add to them.

## How to contribute

### Reporting bugs

Search existing issues first, then use the
[bug report template](.github/ISSUE_TEMPLATE/bug_report.md). Include the
environment, steps to reproduce, expected vs actual behaviour, and a minimal
reproduction where possible.

For a miscompilation or a type-checker soundness bug, say so explicitly — those
are triaged ahead of everything else, and may indicate a divergence between the
implementation and the mechanised specification.

### Suggesting features

Check [`ROADMAP.adoc`](ROADMAP.adoc) and
[`proofs/STATUS.md`](proofs/STATUS.md) first — a surprising amount of "missing"
functionality is deliberately fenced pending a proof. Then use the
[feature request template](.github/ISSUE_TEMPLATE/feature_request.md).

### Good first contributions

- [`good first issue`](https://github.com/hyperpolymath/my-lang/labels/good%20first%20issue)
- [`help wanted`](https://github.com/hyperpolymath/my-lang/labels/help%20wanted)
- [`documentation`](https://github.com/hyperpolymath/my-lang/labels/documentation)

Documentation accuracy fixes are especially welcome — see [`DEBT.md`](DEBT.md)
for a catalogue of known-stale pages.

## Development workflow

### Branches

```
docs/short-description        # documentation
test/what-added               # test additions
feat/short-description        # new features
fix/issue-number-description  # bug fixes
refactor/what-changed         # internal improvements
security/what-fixed           # security fixes
proof/what-proved             # mechanised proof work
```

### Commits

We follow [Conventional Commits](https://www.conventionalcommits.org/):

```
<type>(<scope>): <description>

[optional body]

[optional footer]
```

**Commits must be cryptographically signed.** The `main` branch ruleset
enforces `required_signatures`, so unsigned commits are rejected at push time:

```sh
git config --global commit.gpgsign true
git config --global user.signingkey <your-key-id>
```

SSH signing works too (`gpg.format = ssh`). A linear history is also enforced —
rebase rather than merge.

### Pull requests

1. Branch from `main`.
2. Keep the change focused; unrelated cleanups belong in their own PR.
3. Run `just check` before pushing.
4. Update `CHANGELOG.md` under `[Unreleased]`.
5. If you touched documentation claims, make sure they are still true — this
   repository has an explicit honesty discipline (below).

CI gates include governance/licence checks, the Hypatia neurosymbolic scanner,
secret scanning, CodeQL, and both proof tracks. A red proof gate blocks merge.

## Proof-affecting changes

If your change touches the typing rules, usage/quantity discipline, the QTT
kernel (`crates/my-qtt/`), or anything in `proofs/`:

- [`proofs/STATUS.md`](proofs/STATUS.md) is **the single authoritative record**
  of what is proved. If your change makes it inaccurate, update it in the same
  PR.
- Use its status vocabulary precisely: *machine-checked*, *locally-checked*,
  *conformance-checked*, *proved-on-paper*, *statement-only*,
  *definitions-only*, *absent*.
- **No proof hole is ever described as proved.** A `statement-only` theorem is
  an obligation, not a result.
- The Coq trusted base carries no `Admitted`/`Axiom`; soundness results are
  asserted axiom-free via `Print Assumptions` in CI. Keep it that way.

## Documentation standards

- Prose is `CC-BY-SA-4.0`; code is `MPL-2.0`. Every file carries **exactly
  one** `SPDX-License-Identifier` on its first line.
- Don't document unimplemented behaviour as if it ships. If you must describe
  intended design, mark it explicitly as planned and record it in
  [`DEBT.md`](DEBT.md).
- Prefer fixing a stale claim to adding a new page.

## Code of conduct

Participation is governed by [`CODE_OF_CONDUCT.md`](CODE_OF_CONDUCT.md).

## Licence

By contributing you agree that your contributions are licensed under
**MPL-2.0** (code) or **CC-BY-SA-4.0** (documentation), matching the file you
are editing. See [`LICENSE`](LICENSE).
