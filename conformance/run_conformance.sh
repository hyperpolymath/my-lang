#!/usr/bin/env bash
# SPDX-License-Identifier: PMPL-1.0-or-later
# Conformance test runner for My-Lang
#
# Invokes the My-Lang parser (Rust) on every file in valid/ and invalid/,
# asserting success for valid files and failure for invalid files.
#
# Usage: ./run_conformance.sh [path-to-my-lang-binary]

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# my-lang is a Rust workspace at the monorepo level AND at /var$REPOS_DIR/my-lang
MY_LANG_ROOT="${SCRIPT_DIR}/.."
PARSER="${1:-cargo run --manifest-path "${MY_LANG_ROOT}/Cargo.toml" -p my-cli --quiet -- --parse-only}"

PASS=0
FAIL=0
TOTAL=0

# --- Valid programs: parser MUST succeed ---
for f in "${SCRIPT_DIR}"/valid/*.my; do
    TOTAL=$((TOTAL + 1))
    name="$(basename "$f")"
    if eval "${PARSER}" "$f" >/dev/null 2>&1; then
        echo "  PASS  valid/${name}"
        PASS=$((PASS + 1))
    else
        echo "  FAIL  valid/${name}  (expected success, got failure)"
        FAIL=$((FAIL + 1))
    fi
done

# --- Invalid programs: parser MUST fail ---
for f in "${SCRIPT_DIR}"/invalid/*.my; do
    TOTAL=$((TOTAL + 1))
    name="$(basename "$f")"
    if eval "${PARSER}" "$f" >/dev/null 2>&1; then
        echo "  FAIL  invalid/${name}  (expected failure, got success)"
        FAIL=$((FAIL + 1))
    else
        echo "  PASS  invalid/${name}"
        PASS=$((PASS + 1))
    fi
done

echo ""
echo "Results: ${PASS}/${TOTAL} passed, ${FAIL} failed"

if [ "${FAIL}" -gt 0 ]; then
    exit 1
fi
