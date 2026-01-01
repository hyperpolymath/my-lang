;; SPDX-License-Identifier: AGPL-3.0-or-later
;; Playground Specification
;; Machine-readable spec for My-Lang Playground orchestration

(define spec-playground
  '((version . "1.0.0")
    (schema . "hyperpolymath.spec.playground/1")
    (last-updated . "2026-01-01")

    ;; =========================================
    ;; HIVE LAYOUT CONTRACT
    ;; =========================================
    (hive-layout-contract
      . ((description . "Defines the expected structure and interface for hives")
         (root-path . "./hives/")
         (required-structure
           . ((each-hive-must-have
                . ("README.md or README.adoc"
                   "demo/ directory with runnable examples"
                   "justfile or Mustfile with 'demo' target"
                   ".hive-meta.scm with hive metadata"))))
         (naming-convention
           . ((pattern . "{dialect}-hive")
              (examples . ("me-hive" "solo-hive" "duet-hive" "ensemble-hive"))))
         (submodule-rules
           . ("All hives MUST be git submodules"
              "Submodule refs MUST be pinned to specific commits (no floating HEAD)"
              "Submodule URLs MUST use HTTPS"))))

    ;; =========================================
    ;; DIALECT SELECTION RULES
    ;; =========================================
    (dialect-selection-rules
      . ((description . "Rules for selecting and routing to appropriate dialect")
         (dialect-hierarchy
           . ((me
                . ((level . 1)
                   (audience . "ages 8-12")
                   (mode . "visual/block-based")
                   (prerequisites . ())))
              (solo
                . ((level . 2)
                   (audience . "ages 13+")
                   (mode . "text/human-first")
                   (prerequisites . ("me"))))
              (duet
                . ((level . 3)
                   (audience . "collaborative teams")
                   (mode . "balanced co-creation")
                   (prerequisites . ("solo"))))
              (ensemble
                . ((level . 4)
                   (audience . "architects/orchestrators")
                   (mode . "ai-leads/human-refines")
                   (prerequisites . ("duet"))))))
         (selection-algorithm
           . ("1. If user specifies dialect explicitly, use that dialect"
              "2. If user's skill indicators suggest a level, recommend that dialect"
              "3. Default to 'me' for newcomers"
              "4. Always allow progression to next dialect"
              "5. Never force regression to lower dialect"))))

    ;; =========================================
    ;; DEMO COMMAND CONTRACT
    ;; =========================================
    (demo-command-contract
      . ((description . "Contract for the 'just demo' command")
         (interface
           . ((command . "just demo")
              (optional-args . ("[dialect]"))
              (environment . "offline-capable")))
         (behavior
           . ((without-args
                . ("Run demos for all available hives in sequence"
                   "Print clear delimiter between each dialect demo"
                   "Print dialect name before each demo"))
              (with-dialect-arg
                . ("Run demo for specified dialect only"
                   "Error gracefully if dialect not available"
                   "Suggest available dialects on error"))))
         (output-requirements
           . ("MUST print 'Running [DIALECT] demo...' before each demo"
              "MUST work without network access"
              "MUST exit 0 on success, non-zero on failure"
              "SHOULD complete in under 10 seconds"))
         (example-invocations
           . (("just demo" . "Run all dialect demos")
              ("just demo me" . "Run Me dialect demo only")
              ("just demo solo" . "Run Solo dialect demo only")))))

    ;; =========================================
    ;; DIALECT CHOOSER INDEX
    ;; =========================================
    (dialect-chooser
      . ((description . "Machine-readable index for dialect routing")
         (format . "s-expression")
         (location . "./.machine_read/DIALECT_INDEX.scm")
         (schema
           . ((dialects
                . ((name . "string")
                   (hive-path . "relative-path")
                   (status . "available|coming-soon|deprecated")
                   (demo-command . "string")
                   (learning-resources . "list-of-paths")))))))))
