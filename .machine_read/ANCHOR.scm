;; SPDX-License-Identifier: AGPL-3.0-or-later
;; Repo: hyperpolymath/mylang-playground
;; Machine-readable anchor schema for AI superintendents

(define anchor
  '((schema . "hyperpolymath.anchor/1")
    (repo . "hyperpolymath/mylang-playground")
    (date . "2026-01-01")
    (authority . "repo-superintendent")
    (purpose
      . ("Scope arrest: this repo is an orchestrator playground for the My-Lang dialect family."
         "Keep the dialect boundary crisp (Me/Solo/Duet/Ensemble)."
         "No random experiments: everything must serve dialect learning, tooling, or demos."
         "Ensure recursive clone workflow stays correct and pinned."))

    (identity
      . ((project . "My-Lang Playground")
         (kind . "playground-orchestrator")
         (one-sentence . "Playground for the My-Lang progressive dialect family (Me -> Solo -> Duet -> Ensemble).")
         (upstream . "hyperpolymath/my-lang")))

    (semantic-anchor
      . ((policy . "downstream")
         (upstream-authority
           . ("Dialect definitions/spec live upstream (or in explicitly designated hive repos)."
              "This repo aggregates hives and provides learning paths + demo harnesses."))))

    (implementation-policy
      . ((allowed . ("Deno" "ReScript" "Scheme" "Shell" "Just" "Markdown" "AsciiDoc"))
         (quarantined . ("New dialect invention (must be an upstream decision)"))
         (forbidden
           . ("Turning this into a second standalone language implementation"
              "Unpinned submodule drift (no floating refs)"
              "Network-required execution for demos"))))

    (golden-path
      . ((smoke-test-command
           . ("git submodule update --init --recursive"
              "just --list"
              "just demo   ;; must exist; runs a canonical Me/Solo demo"
              "just test"))
         (success-criteria
           . ("A recursive clone works and produces consistent hive checkouts."
              "Demo works offline and clearly indicates which dialect is being run."
              "Repo-level docs clearly route newcomers to the correct dialect."))))

    (mandatory-files
      . ("./.machine_read/LLM_SUPERINTENDENT.scm"
         "./.machine_read/ROADMAP.f0.scm"
         "./.machine_read/SPEC.playground.scm"
         "./hives/"))

    (first-pass-directives
      . ("SPEC.playground.scm must define: hive layout contract, dialect selection rules, demo command contract."
         "Add a tiny top-level harness that calls into hive demos; do not duplicate language logic."
         "Add a 'dialect chooser' command or doc index that is machine-readable."))

    (rsr . ((target-tier . "bronze-now")
            (upgrade-path . "silver-after-f1 (CI for submodule integrity + demo matrix)")))))
