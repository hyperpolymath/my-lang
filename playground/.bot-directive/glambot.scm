;; SPDX-License-Identifier: MPL-2.0
;; SPDX-FileCopyrightText: 2026 Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk>

(bot-directive
  (bot "glambot")
  (scope "presentation + accessibility")
  (allow ("docs" "readme badges" "ui/accessibility suggestions"))
  (deny ("logic changes"))
  (notes "Edits limited to presentation layers"))
