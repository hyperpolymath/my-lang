;; META.scm - Project metadata
;; SPDX-License-Identifier: AGPL-3.0-or-later

(meta
  (project "my-lang")
  (version "0.1.0")
  (repository "https://github.com/hyperpolymath/my-lang")
  (description "A programming language with first-class AI integration"))

;; ============================================================================
;; COMPONENTS (Updated 2025-12-17)
;; ============================================================================

(components
  (my-lang
    (type "language-runtime")
    (path ".")
    (status "active")
    (description "Core language: lexer, parser, type-checker, interpreter"))

  (my-ssg
    (type "static-site-generator")
    (path "my-ssg")
    (status "new")
    (description "Static site generator powered by My Language templates")))

;; ============================================================================
;; SECURITY REVIEW (2025-12-17)
;; ============================================================================

(security
  (last-review "2025-12-17")
  (status "secure")
  (findings
    (no-shell-execution "✓ No shell/command execution in core code")
    (no-unsafe-code "✓ No unsafe blocks in production code (only in docs as examples)")
    (no-sql-injection "✓ No SQL operations")
    (env-access "⚠ stdlib provides env() function - returns empty on missing vars")
    (file-io "⚠ SSG performs file I/O - sandboxed to project directories")
    (eval-safety "✓ Language eval is sandboxed interpreter, not system eval"))

  (recommendations
    "1. Consider sandboxing env() function in production builds"
    "2. SSG file operations limited to content/templates/static/output dirs"
    "3. All user input is parsed through the lexer - no raw execution"))

;; ============================================================================
;; CROSS-PLATFORM STATUS (Added 2025-12-17)
;; ============================================================================
;; This repo exists on multiple platforms. GitHub is the primary/source of truth.

(cross-platform-status
  (generated "2025-12-17")
  (primary-platform "github")
  (gitlab-mirror
    (path "hyperpolymath/maaf/4a-languages/my-lang")
    (url "https://gitlab.com/hyperpolymath/maaf/4a-languages/my-lang")
    (last-gitlab-activity "2025-11-13")
    (sync-status "gh-primary")
    (notes "GitHub newer by 1 month. Safe to sync GH→GL."))

  (reconciliation-instructions
    ";; To fetch and compare GitLab content:"
    ";; git remote add gitlab https://gitlab.com/hyperpolymath/maaf/4a-languages/my-lang.git"
    ";; git fetch gitlab"
    ";; git log gitlab/main --oneline"
    ";; git diff main gitlab/main"
    ";;"
    ";; To merge if GitLab has unique content:"
    ";; git merge gitlab/main --allow-unrelated-histories"
    ";;"
    ";; After reconciliation, GitHub mirrors to GitLab automatically.")

  (action-required "gh-primary"))

