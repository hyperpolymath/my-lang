;; SPDX-License-Identifier: MPL-2.0
;; SPDX-FileCopyrightText: 2026 Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk>

(bot-directive
  (bot "rhodibot")
  (scope "rsr-compliance")
  (allow ("metadata" "docs" "repo-structure checks"))
  (deny ("destructive edits without approval"))
  (notes "Auto-fix allowed only for formatting in docs and metadata"))
