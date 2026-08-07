<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->
<!-- Owner: Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk> -->
<!-- Copyright (c) Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk> -->
# my-lang — Debt Register

**Measured 2026-08-07** against commit `e68bb3d`. One index of known debt across
seven domains. Every item records the command that produced its evidence, so it
can be re-measured rather than re-argued.

This register **links to** the existing specialised registers rather than
duplicating them:

- [`proofs/STATUS.md`](proofs/STATUS.md) — authoritative proof-status registry
- [`PROOF-NEEDS.md`](PROOF-NEEDS.md) — proof-side cleanup notes
- [`TESTING.md`](TESTING.md) — per-crate coverage measurements
- [`.machine_readable/6a2/STATE.a2ml`](.machine_readable/6a2/STATE.a2ml) — authoritative machine state

Severity: **HIGH** = correctness, licensing or security exposure ·
**MEDIUM** = misleads users or blocks work · **LOW** = tidiness.

Items marked **DIAGNOSIS (unconfirmed)** are hypotheses, not established facts.

---

## Summary

| Domain | Items | High | Medium | Low |
|---|---|---|---|---|
| Licence | 6 | 3 | 2 | 1 |
| Documentation | 8 | 2 | 5 | 1 |
| Code | 6 | 1 | 4 | 1 |
| Proof | 3 | 1 | 2 | 0 |
| Test | 3 | 0 | 2 | 1 |
| CI/CD | 3 | 1 | 2 | 0 |
| Metadata | 5 | 0 | 4 | 1 |
| **Total** | **34** | **8** | **21** | **5** |

---

## Licence (L)

### L-1 — Undefined SPDX identifier `Palimpsest-0.8` · HIGH
`dialects/solo/compiler/Cargo.toml:6` declares `license = "MIT OR Palimpsest-0.8"`.
`Palimpsest-0.8` is **not a registered SPDX identifier** and there is no
`LICENSES/Palimpsest-0.8.txt`. Any REUSE/SPDX validator rejects this, and the
expression is legally ambiguous.

```sh
grep -rn '^license' --include=Cargo.toml . | grep -v 'workspace = true'
```
**Next:** decide the licence for this crate and use a valid expression
(`MPL-2.0` to match the workspace, most likely).

### L-2 — MIT declared with no MIT licence text · HIGH
`my-ssg/Cargo.toml:6` and `playground/hives/me/Cargo.toml:7` declare
`license = "MIT"`, but `LICENSES/` contains only `AGPL-3.0-or-later.txt`,
`CC-BY-SA-4.0.txt` and `MPL-2.0.txt`. README and the workspace both state
MPL-2.0.
**Next:** either relicense these to MPL-2.0 or add `LICENSES/MIT.txt` and record
the exception deliberately.

### L-3 — SPDX tag contradicts licence body · HIGH
`frontier-practices/LICENSE` and `playground/LICENSE` both open with
`SPDX-License-Identifier: MPL-2.0`, but their body is the **Palimpsest-MPL
Licence 1.0** text, copyright *"Palimpsest Stewardship Council"* — not the repo
author. A machine reads MPL-2.0; a human reads modified terms.

```sh
head -3 frontier-practices/LICENSE playground/LICENSE
```
**Next:** this repo migrated off PMPL on 2026-05-26 (standards#196) — these are
leftovers. Replace with the MPL-2.0 text or delete the subtree licences.

### L-4 — AGPL text committed while policy bans AGPL · MEDIUM
`LICENSES/AGPL-3.0-or-later.txt` (34 KB) is present; nothing in the tree declares
AGPL, and `.machine_readable/6a2/AGENTIC.a2ml` states *"Never use AGPL licence"*.
Licence scanners will report AGPL for this repository.
**Next:** delete unless a dependency genuinely requires the text.

### L-5 — 24 manifests carry no `license` key · MEDIUM
Includes `fuzz/Cargo.toml`, `dialects/duet/compiler/Cargo.toml`, and all
`_exploratory/me-scaffolding` + `playground/hives/me` sub-crates.
**Next:** add `license.workspace = true`.

### L-6 — SPDX header gaps · LOW
`.rs`, `.idr` and `.adoc` are **100% covered**. Missing: **31 of 48 `.toml`**
(including the root `Cargo.toml`), 7 `.yml`, 2 `.v`
(`proofs/verification/coq/{Syntax,Typing}.v`), 1 `.md` (`GOVERNANCE.md`).
**Next:** sweep. Note the estate trap — the header must be **line 1**; a header
on line 3 reads as missing to `head -1` linters.

---

## Documentation (D)

### D-1 — Wiki documents a toolchain that does not exist · HIGH
`docs/wiki/guides/installation.md` (432 lines) and `getting-started.md` document
`curl https://mylang.org/install.sh`, `brew install mylang`, `cargo install mylang`,
an `mlup` updater, apt/dnf repos at `packages.mylang.org`, and `ml --version`.

None of it exists. The real binary is **`my`** (`crates/my-cli`), there is no
crates.io publication, and `mylang.org` is not this project's domain. A new user
following these pages cannot succeed.
**Next:** replace with the actual golden path (`git clone` → `just init` →
`just check`). Highest-value doc fix in the repo.

### D-2 — Standard-library reference describes a retracted design · HIGH
`docs/wiki/reference/stdlib.md` (724 lines) documents a 17-module `std::` source
tree (`std::net`, `std::async`, `std::sync`…). `IMPLEMENTATION.md` explicitly
retracts exactly this: *"That was never built and was actively misleading."* The
real stdlib is ~60 flat Rust builtins in `crates/my-lang/src/stdlib.rs`.
**Next:** regenerate from the actual builtin list, or mark the page as a design
sketch.

### D-3 — Concurrency and AI pages document unimplemented features · MEDIUM
`docs/wiki/language/concurrency.md` (611 lines) documents `async fn`/`.await`/
channels; `ai-features.md` (527 lines) documents an `ai!` macro and live model
queries. Concurrency exists as *metatheory only*; the AI runtime performs **mock**
operations. `vscode-extension/README.md` separately calls my-lang "AI-native",
which `docs/wiki/README.md` contradicts.
**Next:** banner both as planned-not-implemented (started — see below), reconcile
the vscode blurb.

### D-4 — Wiki understates proof status · MEDIUM
Three wiki pages say preservation is *"in progress"* / *"statement-only, gated on
the product-elimination decision"*. It is **machine-checked, axiom-free and
CI-gated** (F1.4, resolved 2026-06-14) per `proofs/STATUS.md`.
**Fixed in this pass** for `docs/wiki/README.md`, `language/dialects.md`,
`internals/formal-verification.md`. Recheck when rungs land.

### D-5 — GitHub wiki was a 29-byte stub · MEDIUM
The published wiki contained one page (*"Welcome to the my-lang wiki!"*) while 26
substantial pages sat unpublished in `docs/wiki/`.
**Addressed in this pass** — see [Wiki](#wiki-publication) below.

### D-6 — Orphaned index and theory stubs point at a dead project · MEDIUM
`docs/MY-LANGUAGE-INDEX.md` is anchored to an *"October 26, 2025 master index
(My-Newsroom project summary)"*; every path in it is dead (`my-newsroom/`,
`docs/dialects/*.md`, `src/checker.rs`, `docs/NEWROOM-ROADMAP.md`).
`docs/theory/*` (5 files) are self-described pointer stubs referencing the same
dead tree.
**Next:** delete or rewrite. They are pure navigational traps.

### D-7 — `docs/` is published by nothing · MEDIUM
`ARCHITECTURE.md` says `docs/` is *"published via Ddraig SSG"*, and README links
readers to GitHub Pages for documentation. But `pages.yml` builds from the
orphaned root `src/`, while `config.yaml` declares `input: site` — which holds a
single 12-line `index.md`. The 13,000-line `docs/wiki/` tree reaches no reader.
Three different domains are cited across docs (`mylang.org`, `my-lang.net`,
`hyperpolymath.github.io/my-lang`).
**Next:** point the SSG at `docs/`, or stop claiming `docs/` is published.
Publishing the wiki (D-5) mitigates but does not close this.

### D-8 — `.adoc`/`.md` duplicate pairs · LOW
`README`, `CONTRIBUTING`, `GOVERNANCE`, `MAINTAINERS`, `FOUNDATIONS_BRIDGE` each
exist twice. The `.adoc` copies drifted badly; they cannot be deleted because
`.machine_readable/contractiles/{Mustfile,Trustfile}.a2ml` reference them.
**Partly addressed:** `README.adoc` and `PALIMPSEST.adoc` rewritten as short
accurate pointers rather than competing duplicates.
**Next:** update the contractiles to reference the `.md` files, then retire the
`.adoc` twins. Also: `MAINTAINERS` names *"Metadatastician / @metadatastician"*,
not the `hyperpolymath` identity used everywhere else.

---

## Code (C)

### C-1 — 7,317 LOC of orphaned duplicate source · HIGH
Root `src/` (8 files, 2,685 LOC) and root `lib/` (17 files, 4,632 LOC) are stale
divergent forks of `crates/my-lang/src/` and `crates/my-lang/lib/`. Neither is
referenced by any manifest, so neither compiles. Root `lib/` contains files that
exist nowhere else (`collections.rs`, `concurrency.rs`, `fs.rs`).

```sh
find src lib -name '*.rs' | xargs wc -l | tail -1   # 7317 total
```
This is actively dangerous: `docs/wiki/internals/architecture.md` documents these
orphans as the compiler, and `pages.yml` builds the site from `src/`.
**Next:** salvage the unique files, then delete both trees. `TESTING.md` already
flags them.

### C-2 — `crates/my-parser` is a published-shape stub · MEDIUM
58 LOC total; `parse_program`/`parse_top_level`/`parse_fn_decl` all return
`Ok(())` unconditionally with `// TODO: Hook the Solo v1.0 grammar into this
method.` It is a full workspace member that parses nothing — the real parser is
`crates/my-lang/src/parser.rs` (100 KB).
**Next:** either wire it up or remove it from the workspace; a crate that always
succeeds is worse than an absent one.

### C-3 — `dialects/solo/compiler` is all TODO and unbuilt · MEDIUM
`TODO(#parser)`, `TODO(#typeck)`, `TODO(#codegen)`, `TODO(#runtime)`. This is the
artefact the Coq `check_correct` spec exists *for* — README names it as the thing
that must meet the executable spec — yet it is outside the workspace, so nothing
builds or tests it. Corresponds to `STATE.a2ml`'s high-severity `#typeck`.
**Next:** the project's headline correctness goal. Track explicitly.

### C-4 — Unimplemented handlers in shipped-looking crates · MEDIUM
22 TODOs in `crates/`: `my-lsp` 6 (completions, find-references, rename,
formatting, code actions, signature help), `my-pkg` 5 (registry query, tarball
extraction — i.e. it cannot resolve or install), `my-mir` 4 (closure conversion,
match decision trees), `my-hir` 3.
**Next:** these crates are presented as tooling in the docs; either scope them
down in prose or fill them in.

### C-5 — Panic surface baselined, not fixed · MEDIUM
7 entries in `.hypatia-baseline.json`, **expiring 2026-10-27** (issue #145). The
gate re-reds at expiry. Counts have drifted from the recorded baseline (`my-fmt`
26→27). The `my-lang` `.expect(` bulk is the documented scanner false positive
(the parser's own `self.expect(TokenKind)` method), but `my-qtt` (12 unwrap +
5 expect) and `my-fmt` (27 unwrap) are not covered by that explanation.
**Next:** discharge before expiry. ~11 weeks.

### C-6 — Untracked in-tree worktree · LOW
`.claude/worktrees/` is a full second checkout of the repo, **untracked and
absent from `.gitignore`** (`grep -c claude .gitignore` → 0). It doubles every
grep/scan result and inflates the tree.
**Next:** add `.claude/` to `.gitignore`.

---

## Proof (P)

### P-1 — Proof workflow does not run when the implementation changes · HIGH
`.github/workflows/proofs.yml` triggers only on `paths: proofs/verification/**`.
A change to the checker, the QTT bridge, or `Cargo.toml` never re-runs the proof
gate — precisely the changes most likely to break the correspondence between the
verified spec and the implementation.
**Next:** add `crates/my-qtt/**` and `crates/my-lang/src/{checker,qtt_bridge}.rs`
to the trigger paths.

### P-2 — Idris track has no hole assertion · MEDIUM
The Coq job runs ~10 `Print Assumptions` gates asserting *"Closed under the
global context"* — a genuine axiom-freedom gate. The Idris job runs
`idris2 --build` only. **A typed hole type-checks**, so a reintroduced
`?todo_*` would pass. `proofs/STATUS.md` correctly grades the Idris results
`locally-checked` rather than `machine-checked`; the gate should match that
honesty.
**Next:** add an explicit hole grep to the Idris job.

### P-3 — Proof registry is 8 weeks stale · MEDIUM
`proofs/STATUS.md` says *"Last verified: 2026-06-14"*, and it outranks every
other document by its own terms. Meanwhile `proofs.yml` already gates
S3c.3-choice, which `STATE.a2ml` still lists as `pending`.
**Next:** re-verify and re-date. The workflow is ahead of both registries.

---

## Test (T)

### T-1 — 34 property tests never run · MEDIUM
`tests/property_tests.rs` (23 KB, 34 `#[test]`/`proptest!` blocks) is not
referenced by any `[[test]]` target in any manifest, so it never compiles. Only
`tests/integration_test.rs` is wired (via `crates/my-lang/Cargo.toml:53-55`).
**Next:** wire it or delete it. Unrun tests read as coverage that does not exist.

### T-2 — Coverage floor is 40% against a 46.7% baseline · MEDIUM
`.github/workflows/coverage.yml` sets `COVERAGE_FLOOR: "40"`, and `my-llvm` is
excluded from measurement entirely. The floor sits *below* the current baseline,
so coverage can regress ~7 points without the gate noticing.
**Next:** raise the floor to just under the measured baseline and ratchet.

### T-3 — Test count claim reproduces · LOW *(verified, no action)*
Recorded here because it was checked rather than assumed. README's
**"221 tests pass, 0 failures"** reproduces exactly:

```sh
cargo test --workspace --exclude my-llvm   # passed=221 failed=0 ignored=0
```
Re-measured 2026-08-07. Contrast the estate pattern where published counts did
not reproduce.

---

## CI/CD (I)

### I-1 — Lockfile regeneration is a recurring manual step · HIGH
GitHub's Actions lockfile enforcement rejects any workflow whose `actions.lock`
is stale, with a 0-second `startup_failure` whose reason appears **only on the
run's HTML page**. Dependabot bumps action SHAs *without* regenerating the
lockfile, so every bump re-breaks the board — this happened on 2026-08-07 via
#155/#157 (7 of 15 workflows stale; CodeQL, Scorecard and Governance red).

Additionally, `gh actions-lock` **omits reusable-workflow callers entirely**, so
the six standards wrappers need hand-authored entries plus a transitive `uses:`
sub-list every time.

```sh
gh actions-lock --verify-local
```
**Next:** automate. Either a dependabot post-update hook or a scheduled job
running the full recipe (regenerate → restore SHA pins → `relock-sha-keys.sh` →
restore caller entries). Fixed for now in PR #158.

### I-2 — SPARK gate is advisory and has nothing to check · MEDIUM
`spark-theatre-gate.yml` calls an external reusable with
`enforce_zero_contract: false` — advisory only — and there is **no SPARK/Ada code
in the repository** for it to analyse. It reports green regardless.
**Next:** remove, or make it enforcing if Ada code arrives.

### I-3 — Governance linter is a shared-fate dependency · MEDIUM
The governance job runs `gh actions-lock --verify-local` from the standards
reusable, so lockfile staleness surfaces as a *governance* failure rather than a
lockfile one — correct behaviour, but it means a standards-side linter change can
red this repo without any local change. Observed 2026-08-07.
**Next:** none required; documented so the failure mode is recognisable.

---

## Metadata (M)

### M-1 — `ECOSYSTEM.a2ml` is two months stale · MEDIUM
`last-updated = "2026-06-02"`. Lists gaps G1/G2/G3/G5 as open — **all closed**
per `STATE.a2ml`. Still describes *"Four dialects: solo, duet, ensemble, me"*,
which `STATE.a2ml` explicitly retracts (`dialect-model = "nested-subsets"`; `me`
is a projector).
**Next:** regenerate from `STATE.a2ml`.

### M-2 — `STATE.a2ml` head-commit and proof claims drifted · MEDIUM
`main-commit = "8636e15"` — actual HEAD is `e68bb3d` (8 commits later).
The note *"Idris `?todo_preservation` deliberately open"* is contradicted by
`proofs/STATUS.md` (*discharged 2026-06-21, #108*) and by the tree — zero `?todo`
holes remain. `main-board = "FULLY GREEN"` was false on 2026-08-07.
**Next:** refresh; it is otherwise the best-maintained file in the repo.

### M-3 — K9 self-validation contradicts the tree · MEDIUM
`.machine_readable/self-validating/my-lang-metadata.k9.ncl` declares
`required_dialects = ["me","solo","duet","ensemble"]` (me is not a dialect) and
`forbidden_patterns = ["unwrap()", "expect()", …]` — violated 85 + 183 times in
`crates/`. MSRV disagrees three ways: `msrv = "1.75"` vs `mise.toml` `1.97.0` vs
`.tool-versions` `stable`, and **no `rust-version` key exists in any
`Cargo.toml`**.
**Next:** a self-validation file that cannot pass is worse than none — reconcile
or scope the patterns to new code.

### M-4 — Empty and contradictory manifests · MEDIUM
`PLAYBOOK.a2ml` has **zero uncommented keys**; `NEUROSYM.a2ml` is nearly all
commented placeholders; `AGENTIC.a2ml` (2026-04-11) is comment-only and states
*"Never place state files in repository root"* while four sit at root.
Two different `0-AI-MANIFEST.a2ml` files exist (root = S-expression,
`6a2/` = Markdown) with contradictory content.
`svc/README.adoc` documents a `k9/` directory that does not exist (the file lives
in `self-validating/`).
**Next:** fill or delete; deduplicate the manifest.

### M-5 — Contractile drift · LOW
`contractiles/README.adoc` claims its `Justfile` is *"hardlinked from the repo
root"* — inodes differ (746328 vs 746300), and the copy still carries the
`@echo` stubs that `STATE.a2ml` records as resolved. `Intentfile.a2ml` and both
`bust/*.a2ml` mark already-resolved failure modes as `status: declared`.
**Next:** re-link or regenerate.

---

## What this pass changed

Fixed in the accompanying documentation PR:

- **Licence ambiguity**: `PALIMPSEST.adoc` rewritten as a historical note — it
  had asserted the repo was licensed PMPL-1.0, contradicting `LICENSE`.
- **`CONTRIBUTING.md`**: was structurally broken (rendered as an H1 mid-code-block,
  ended unterminated, carried two conflicting SPDX ids). Rewritten with the real
  repository layout, the real signing requirement (`required_signatures` is
  enforced by the branch ruleset — not DCO), and a proof-obligations section.
- **`ROADMAP.adoc`**: was a mint-time placeholder (*"Initial development phase"*,
  all boxes unchecked) contradicting a 0.2.0 tree with 221 passing tests.
  Replaced with the real rung state and an explicit *Not planned* section.
- **`README.adoc`**: stale duplicate contradicting `README.md` on test count,
  proof phase and dialect structure → short accurate pointer.
- **`EXPLAINME.adoc`**: corrected the false *"`.hypatia-baseline.json` is an
  intentionally empty array"* claim (it has 7 expiring entries), the stale test
  count, the four-co-equal-dialects framing, a dead `contractiles/` path, and the
  ReScript→AffineScript survivals.
- **Wiki**: preservation status corrected on three pages; aspirational pages
  bannered; the 26-page in-tree wiki published to the previously-stub GitHub wiki.
- **Repo metadata**: description and topics replaced (see below).

## Wiki publication

`docs/wiki/` is the source of truth; the GitHub wiki is generated from it. Do not
edit the GitHub wiki directly — edits there are overwritten.

## Repository metadata

Description and topics were rewritten on 2026-08-07. The previous topics
(`development`, `hyperpolymath`, `open-source`, `rust`, `software`, `tooling`)
duplicated what GitHub already derives from the language list, or named the
owner. The current nine are concept-level and chosen for discoverability:
`quantitative-type-theory`, `affine-types`, `linear-types`,
`mechanized-metatheory`, `session-types`, `verified-compiler`,
`programming-language-design`, `proof-engineering`, `type-systems`.
