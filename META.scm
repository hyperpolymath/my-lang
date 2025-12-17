;; META.scm - Project metadata
;; SPDX-License-Identifier: AGPL-3.0-or-later

(meta
  (project "my-lang")
  (repository "https://github.com/hyperpolymath/my-lang"))

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

