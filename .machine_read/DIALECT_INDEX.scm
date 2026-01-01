;; SPDX-License-Identifier: AGPL-3.0-or-later
;; Dialect Index - Machine-readable dialect routing table
;; Used by the dialect chooser and demo harness

(define dialect-index
  '((version . "1.0.0")
    (schema . "hyperpolymath.dialect-index/1")
    (last-updated . "2026-01-01")

    ;; Available dialects
    (dialects
      . ((me
           . ((name . "Me")
              (hive-path . "./hives/me-hive")
              (status . "coming-soon")
              (demo-command . "just demo me")
              (description . "Visual block-based dialect for young learners (ages 8-12)")
              (learning-resources
                . ("./docs/me-getting-started.adoc"
                   "./hives/me-hive/tutorials/"))))
         (solo
           . ((name . "Solo")
              (hive-path . "./hives/solo-hive")
              (status . "coming-soon")
              (demo-command . "just demo solo")
              (description . "Text-based human-first dialect with AI assistance")
              (learning-resources
                . ("./docs/solo-getting-started.adoc"
                   "./hives/solo-hive/tutorials/"))))
         (duet
           . ((name . "Duet")
              (hive-path . "./hives/duet-hive")
              (status . "coming-soon")
              (demo-command . "just demo duet")
              (description . "Balanced human-AI collaborative dialect")
              (learning-resources
                . ("./docs/duet-getting-started.adoc"
                   "./hives/duet-hive/tutorials/"))))
         (ensemble
           . ((name . "Ensemble")
              (hive-path . "./hives/ensemble-hive")
              (status . "coming-soon")
              (demo-command . "just demo ensemble")
              (description . "Multi-agent orchestration dialect with Newtonian spectrum")
              (learning-resources
                . ("./docs/ensemble-getting-started.adoc"
                   "./hives/ensemble-hive/tutorials/"))))))))
