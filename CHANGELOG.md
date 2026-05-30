<!--
SPDX-License-Identifier: MPL-2.0
SPDX-FileCopyrightText: 2026 Jonathan D.A. Jewell (hyperpolymath)
-->

# Changelog

All notable changes to `my-lang` will be documented in this file.

This file is generated from conventional commits by the
[`changelog-reusable.yml`](https://github.com/hyperpolymath/standards/blob/main/.github/workflows/changelog-reusable.yml)
workflow (`hyperpolymath/standards#206`). Adopt the workflow in this repo's CI to keep this file in sync automatically — see
[`templates/cliff.toml`](https://github.com/hyperpolymath/standards/blob/main/templates/cliff.toml)
for the canonical config.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/);
this project aims to follow [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- feat(solo/stdlib): fs_list_dir(path) -> Array<String> (#56)
- feat(solo/stdlib): date_today() -> ISO YYYY-MM-DD (UTC) (#54)
- feat(solo/stdlib): json_parse / json_stringify + fix string escapes (#53)
- feat(solo/stdlib): string-keyed Map/dict builtins (#49)
- feat(parser): add MAX_PARSE_EXPR_DEPTH guard + recover partial AST (closes #15) (#21)
- feat(stdlib): add fs_*/env_args/format builtins + my-cli argv forwarding (#10)
- feat(stdlib): add fs_*/env_args/format builtins + my-cli argv forwarding (#8)
- feat(stdlib): add fs_*/env_args/format builtins + my-cli argv forwarding (#2)
- feat(my-lang): add Ensemble dialect grammar and implement Variant B types

### Fixed

- fix(checker): self-driving stack-depth measurement + CI guard on Linux + Windows; harden survives() (closes #37) (#80)
- fix(licence): clear scaffold-placeholder leak (isolated; dirty repo) (#64)
- fix(vscode): make the packaged .vsix actually load (Refs #62) (#63)
- fix(vscode-ext): pass real callbacks to registerCommand (affinescript#35 P2) (#60)
- fix(vscode-ext): correct affinescript compile invocation + document build (#58)
- fix(ci): pin upload-artifact to valid SHA in hypatia-scan.yml (Refs standards#48) (#50)
- fix(checker): general non-overflowing AST teardown + measured MAX_EXPR_DEPTH (#37) (#43)
- fix(ci): sync hypatia-scan.yml to canonical (kill cd-scanner build drift) (#44)
- fix(ci): Phase-2 fleet submission must not fail the security gate (#42)
- fix(#34): resolve Hypatia backlog at source; finalize baseline (#35)
- fix(lib/common): two latent correctness bugs in stdin/radix paths (refs #22) (#24)

### Changed

- perf(checker): memoise environment-independent subexpressions (closes #16) (#31)
- refactor: wire up dead fields, derive Default, clarify package-mgr stubs (refs #22) (#26)
- refactor: rename 7-tentacles → frontier-practices, sideline dialects/me

### Documentation

- docs: mark Maps/JSON/Date stdlib rows Shipped in ecosystem roadmap (#59)
- docs: replace never-built stdlib .my-module/@ffi sketch with reality (#52)
- docs(checker): reframe MAX_EXPR_DEPTH as a stack-recursion bound (closes #14) (#36)
- docs(#14) record + fix repeated Hypatia scan noise (#33)
- docs(readme): add SPDX header, OSSF and GWF badges

### CI

- ci(spark): adopt estate SPARK Theatre Gate (standards#135) (#65)
- ci(codeql): scan `actions` instead of javascript-typescript (#32)
- ci(secret-scanner): drop duplicate --fail from trufflehog extra_args (#17)
- ci(antipattern): fix top-level dir matching + benchmarks/lsp/bench filename allowlists (#7)
- ci(antipattern): TS check reads .claude/CLAUDE.md exemption table (#6)

## Pre-history

Prior commits to this file's introduction are recorded in git history but not formally classified into Keep-a-Changelog sections. To backfill, run `git cliff -o CHANGELOG.md` locally using the canonical [`cliff.toml`](https://github.com/hyperpolymath/standards/blob/main/templates/cliff.toml) — this is one-shot mechanical work.

---

<!-- This file was seeded by the 2026-05-26 estate tech-debt audit follow-up (Row-2 Phase 3); see [`hyperpolymath/standards/docs/audits/2026-05-26-estate-documentation-debt.md`](https://github.com/hyperpolymath/standards/blob/main/docs/audits/2026-05-26-estate-documentation-debt.md). -->
