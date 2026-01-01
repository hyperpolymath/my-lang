;; SPDX-License-Identifier: AGPL-3.0-or-later
;; ANCHOR.scope-arrest.2026-01-01.Jewell.scm  (my-lang)
(define anchor
  '((schema . "hyperpolymath.anchor/1")
    (repo . "hyperpolymath/my-lang")
    (date . "2026-01-01")
    (authority . "repo-superintendent")
    (purpose . ("Contain scope; make one dialect real; quarantine the rest."))
    (identity
      . ((project . "My")
         (kind . "progressive-reveal language")
         (domain . "multi-dialect (Me/Solo/Duet/Ensemble)")
         (one-sentence . "A language family with progressive complexity levels; only one level is authoritative in f0.")))

    (semantic-anchor
      . ((policy . "dual")
         (reference-impl . ("Solo dialect interpreter is authoritative in f0"))
         (formal-spec . ("SPEC.core.scm defines Solo syntax/semantics only"))
         (dialect-policy
           . ("In f0: only SOLO is in-scope."
              "Other dialects must be tagged 'planned' and MUST NOT ship partial semantics."))))

    (allowed-implementation-languages
      . ("Rust"))
    (quarantined-optional
      . ("AI integration hooks (must be stubbed/offline in tests)"
         "SSG and extra tooling"))
    (forbidden
      . ("Implementing multiple dialects in f0"
         "Network-required builds"
         "Adding new host languages"))

    (golden-path
      . ((smoke-test-command . "cargo test && cargo run -- run examples/hello.ml")
         (success-criteria . ("Solo example parses+runs"
                              "invalid Solo programs fail deterministically"
                              "no network access required"))))

    (first-pass-directives
      . ("Freeze a minimal Solo grammar and evaluator."
         "Move other dialect docs into ROADMAP only (no partial code)."
         "Create conformance corpus for Solo."))

    (rsr
      . ((target-tier . "bronze-now") (upgrade-path . "silver-after-f1")))))
