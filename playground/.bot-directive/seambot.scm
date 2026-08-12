;; SPDX-License-Identifier: MPL-2.0
;; SPDX-FileCopyrightText: 2026 Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk>

(bot-directive
  (bot "seambot")
  (scope "integration health")
  (allow ("analysis" "contract checks" "docs updates"))
  (deny ("code changes without approval"))
  (notes "May add integration test suggestions"))
