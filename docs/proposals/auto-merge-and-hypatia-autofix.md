<!--
SPDX-License-Identifier: MPL-2.0
SPDX-FileCopyrightText: 2026 Jonathan D.A. Jewell (hyperpolymath)
-->

# Proposal: self-driving PR merge via Hypatia autofix + gitbots

**Status:** proposal (no automation enabled by this document)
**Motivation:** PR #84 demonstrated the gap. Every push re-tripped the *same*
pre-existing, mechanical CI findings, none caused by the PR's diff, and a human
had to reason about each one before merge. This proposal specifies how the
Hypatia ruleset and the estate gitbots can drive that loop themselves in
future — safely, with a human kept in the loop for anything semantic.

This is a **design**. Most of the moving parts live outside `my-lang`
(`hyperpolymath/standards` owns the reusable workflows + the Hypatia ruleset;
the gitbots live in the bot-directive repo), so landing it is a cross-repo
effort. This doc is the `my-lang`-side anchor and the checklist.

> **Scope note (post-#83).** The *one-off* governance fixes this proposal
> originally also covered are already implemented in **PR #83**: the
> `.hypatia-baseline.json` array-shape fix and the `frontier-practices/*.res`
> `.hypatia-ignore` exemption. So this document is now scoped to the part #83
> does **not** address — the repeatable **autofix + auto-merge automation** so
> the *next* PR doesn't need a human to re-resolve the same mechanical findings.
> Items already done in #83 are marked as such below rather than restated as
> work.

## 1. What "do this itself" has to mean

Two distinct capabilities, often conflated:

1. **Auto-fix** — mechanically resolve the recurring, deterministic findings so
   they stop blocking (timeouts, SHA-pins, baseline-shape drift).
2. **Auto-merge** — once required checks are green, merge without a human
   pressing the button.

Auto-merge without auto-fix just blocks forever on the same findings.
Auto-fix without auto-merge still needs a human to push the button. We want
both, **gated** so only low-risk PRs ever merge unattended.

## 2. The recurring findings, classified

From PR #84's scans, every blocking item fell into one of three buckets:

| Finding | Bucket | Where the fix lives | Auto-fixable? |
|---|---|---|---|
| `missing_timeout_minutes` on `cflite_*`, `checker-scaling`, `codeql`, `governance`, `hypatia-scan`, `mirror` | mechanical | `my-lang/.github/workflows/*` | **Yes** — add `timeout-minutes:` |
| `unpinned_action` → `standards/...governance-reusable.yml@main` | mechanical-ish | `my-lang/.github/workflows/governance.yml` | Yes, but needs a resolver (pin to SHA + keep updated via Renovate/Dependabot) |
| `Validate Hypatia baseline` shape mismatch (object vs array) | contract drift | ✅ **fixed in #83** (`.hypatia-baseline.json` → array) | n/a — done |
| `Language / package anti-pattern` on `frontier-practices/*.res` | policy decision | ✅ **decided in #83** (rationale'd `.hypatia-ignore` exemption) | n/a — done |
| `scaling (ubuntu-latest)` linearity guard | flaky/threshold | `my-lang/tests/checker_alloc_scaling.rs` | **No** — needs robustness work (issue #85) |

**Rule of thumb:** the top three are safe for a bot; the bottom two must never
be auto-resolved (auto-exempting a policy or muting a flaky guard is exactly the
silent-divergence failure mode we want to avoid).

## 3. Hypatia ruleset changes (in `standards`)

The Hypatia finding schema already carries an `action` field (`flag`,
`pin_sha`, …). The proposal:

1. Promote the **mechanical** rules from `action: flag` to `action: autofix`,
   each with a deterministic, idempotent fixer:
   - `missing_timeout_minutes` → insert a default `timeout-minutes:` (repo-
     configurable; e.g. 30 for build/test jobs) on jobs that lack one.
   - `unpinned_action` → resolve `@<ref>` to the current `@<sha> # <ref>` and
     register the action with the repo's updater so it stays current.
   - `hypatia_baseline_shape` → normalise `.hypatia-baseline.json` to the
     canonical schema (see §4).
2. Keep **policy** and **flaky** rules at `action: flag` (never autofix):
   `banned_language_file`, allocation/perf guards, anything `severity: high|critical`.
3. Emit an autofix **patch artifact** (not a direct push) so the gitbot, not
   the scanner, owns the commit — keeps provenance and review trail clean.

## 4. Baseline contract — fixed in #83 (estate follow-up only)

The `Validate Hypatia baseline` red (committed `.hypatia-baseline.json` was an
*object*; the `standards` validator asserts an *array*) is **already resolved in
PR #83**, which converts the baseline to a valid array. No further `my-lang`
action.

The remaining (estate-level, optional) follow-up is to make the **canonical
baseline schema** authoritative in `standards` so consumers can't drift again —
but that is no longer a blocker here.

## 5. Gitbot behaviour (bot-directive repo)

A bot (the existing `finishbot` / `seambot` directive style fits) on
`pull_request` + `check_suite` events:

```
on PR open / synchronize:
  1. pull Hypatia autofix patch artifact (§3)
  2. if patch non-empty AND PR is bot/low-risk tier:
       apply patch as a commit, push to PR branch
  3. label the PR with its risk tier (see §6)
  4. if tier == auto AND no unresolved `flag`/policy findings:
       enable GitHub native auto-merge (squash)   # merges when checks go green
  5. else:
       leave for human review; post a one-line summary of blocking flags
on check_suite completed (success):
  GitHub native auto-merge does the merge; nothing else to do
```

Use **GitHub-native auto-merge** as the merge primitive (it already respects
branch protection and required checks) rather than a bot calling the merge API —
that keeps the safety rail (protection) authoritative.

## 6. Risk tiers — what may merge unattended

```
tier: auto      → dependency bumps, SHA-pin/timeout autofixes, doc-only,
                  generated-file refreshes. Auto-merge on green.
tier: assisted  → bot applies mechanical autofixes, but a human approves merge.
                  (default for normal feature PRs like #84)
tier: manual    → touches policy, security gates, allocation/perf guards,
                  or carries any `flag`/high/critical finding. No automation.
```

Tier is derived from changed paths + finding severities; never self-selected by
the PR author. This is what stops a feature PR from silently auto-merging.

## 7. `my-lang`-side changes (remaining; the only ones landable here)

Done in #83: baseline array-shape fix and the `frontier-practices/*.res`
exemption. Still open in this repo:

- [ ] Add `timeout-minutes:` to `cflite_batch.yml`, `cflite_pr.yml`,
      `checker-scaling.yml`, `codeql.yml`, `governance.yml`, `hypatia-scan.yml`,
      `mirror.yml` (matches the `coverage.yml` precedent from #84).
- [ ] Harden or tier the `#14` allocation-scaling guard so it isn't flaky on CI
      runners (issue #85).
- [ ] After branch protection trusts the green set, enable repo setting
      **"Allow auto-merge"** so the gitbot can use native auto-merge.

## 8. Cross-repo dependencies (out of `my-lang` scope)

- `hyperpolymath/standards`: Hypatia ruleset `flag→autofix` promotion + fixers
  (§3); baseline schema/validator reconciliation (§4); a reusable
  `autofix`/auto-merge workflow consumers can call.
- bot-directive repo: the gitbot logic (§5) + risk-tiering (§6).

## 9. Sequencing

0. ✅ Baseline shape + `.res` exemption — **landed in #83**.
1. Land the `my-lang` mechanical fixes (§7, timeouts) — clears the
   `missing_timeout_minutes` noise.
2. Promote mechanical Hypatia rules to `autofix` in `standards` (§3).
3. Ship the gitbot tiering + native auto-merge enablement (§5, §6).
4. Resolve the remaining human-decision item (the flaky scaling guard) so the
   estate's required set is permanently green — only then is unattended merge of
   `assisted`-tier PRs safe to consider.

Nothing here auto-merges anything until steps 1–3 are in place and a human has
signed off on the tiering policy.
