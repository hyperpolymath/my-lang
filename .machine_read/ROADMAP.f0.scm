;; SPDX-License-Identifier: AGPL-3.0-or-later
;; Roadmap Fragment 0 (f0) - Initial Setup Phase
;; Machine-readable roadmap for My-Lang Playground

(define roadmap-f0
  '((version . "0.1.0")
    (schema . "hyperpolymath.roadmap/1")
    (fragment . "f0")
    (title . "Foundation Phase")
    (status . "in-progress")
    (date-created . "2026-01-01")

    ;; RSR Target
    (rsr
      . ((current-tier . "bronze-now")
         (target-tier . "bronze")
         (next-tier . "silver-after-f1")))

    ;; Milestones
    (milestones
      . ((m1
           . ((id . "m1-anchor-schema")
              (title . "Anchor Schema Setup")
              (status . "in-progress")
              (tasks
                . (("Create .machine_read/ directory" . "done")
                   ("Create ANCHOR.scm" . "done")
                   ("Create LLM_SUPERINTENDENT.scm" . "done")
                   ("Create SPEC.playground.scm" . "in-progress")
                   ("Create hives/ directory structure" . "pending")))))
         (m2
           . ((id . "m2-golden-path")
              (title . "Golden Path Verification")
              (status . "pending")
              (tasks
                . (("Implement just demo command" . "pending")
                   ("Verify recursive clone workflow" . "pending")
                   ("Add dialect chooser" . "pending")
                   ("Verify offline operation" . "pending")))))
         (m3
           . ((id . "m3-hive-integration")
              (title . "Hive Integration")
              (status . "pending")
              (tasks
                . (("Add Me dialect hive reference" . "pending")
                   ("Add Solo dialect hive reference" . "pending")
                   ("Create hive aggregation harness" . "pending")
                   ("Document hive layout contract" . "pending")))))))

    ;; Dependencies
    (dependencies
      . ((upstream . "hyperpolymath/my-lang")
         (hives . ("me-hive" "solo-hive"))))

    ;; Success criteria for f0 completion
    (success-criteria
      . ("All mandatory .machine_read files exist and are valid Scheme"
         "just demo runs without network access"
         "git submodule update --init --recursive succeeds"
         "Dialect chooser documentation exists"))))
