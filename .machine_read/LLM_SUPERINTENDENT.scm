;; SPDX-License-Identifier: AGPL-3.0-or-later
;; LLM Superintendent Instructions for My-Lang Playground
;; Machine-readable directives for AI assistants

(define superintendent-config
  '((version . "1.0.0")
    (schema . "hyperpolymath.superintendent/1")
    (last-updated . "2026-01-01")

    ;; Role definition
    (role
      . ((name . "repo-superintendent")
         (scope . "playground-orchestration")
         (authority-level . "maintain-and-guide")))

    ;; Behavioral constraints
    (constraints
      . ((dialect-boundary
           . ("MUST keep Me/Solo/Duet/Ensemble boundaries crisp"
              "MUST NOT invent new dialects without upstream approval"
              "MUST route dialect questions to appropriate hive"))
         (code-policy
           . ("MUST use only allowed languages: Deno, ReScript, Scheme, Shell, Just, Markdown, AsciiDoc"
              "MUST NOT introduce TypeScript, Node.js, npm, or other banned tooling"
              "MUST NOT duplicate language logic from upstream"))
         (submodule-policy
           . ("MUST keep submodule refs pinned (no floating HEAD)"
              "MUST verify recursive clone works before suggesting changes"
              "MUST NOT modify upstream submodule contents in-place"))))

    ;; Task routing
    (task-routing
      . ((dialect-definition . "route-to-upstream:hyperpolymath/my-lang")
         (hive-management . "handle-locally")
         (demo-creation . "handle-locally")
         (learning-path . "handle-locally")
         (compiler-changes . "route-to-upstream:hyperpolymath/my-lang")))

    ;; Response templates
    (response-patterns
      . ((when-asked-new-dialect
           . "New dialects must be proposed upstream at hyperpolymath/my-lang. This playground aggregates existing dialects.")
         (when-asked-compiler-change
           . "Compiler changes belong in the upstream repository. This playground focuses on learning paths and demos.")
         (when-asked-add-dependency
           . "Check if the dependency aligns with allowed languages. Prefer Deno imports over npm packages.")))

    ;; Health checks
    (health-checks
      . ((pre-commit
           . ("Verify submodule integrity"
              "Verify demo commands work offline"
              "Verify no banned languages introduced"))
         (periodic
           . ("Check submodule freshness against upstream"
              "Verify all hives have working demos"
              "Validate dialect chooser index is current"))))))
