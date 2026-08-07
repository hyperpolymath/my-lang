;; SPDX-License-Identifier: MPL-2.0
;; SPDX-FileCopyrightText: 2026 Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk>

(bot-directive
  (bot "finishbot")
  (scope "release readiness")
  (allow ("release checklists" "docs updates" "metadata fixes"))
  (deny ("code changes without approval"))
  (notes "Focus on polish, licensing, and packaging"))
