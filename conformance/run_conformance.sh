#!/usr/bin/env bash
# SPDX-License-Identifier: MPL-2.0
# SPDX-FileCopyrightText: 2026 Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk>

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

# Build PARSER_CMD as an array so we never need `eval`. If the caller passes
# a parser command as $1, split it on whitespace into array elements; otherwise
# use the default cargo invocation.
if [[ -n "${1:-}" ]]; then
    # shellcheck disable=SC2206 # deliberate word-splitting of the caller's string
    PARSER_CMD=(${1})
else
    PARSER_CMD=(cargo run --manifest-path "${MY_LANG_ROOT}/Cargo.toml" -p my-cli --quiet -- --parse-only)
fi

PASS=0
FAIL=0
TOTAL=0

# --- Valid programs: parser MUST succeed ---
for f in "${SCRIPT_DIR}"/valid/*.my; do
    TOTAL=$((TOTAL + 1))
    name="$(basename "$f")"
    if "${PARSER_CMD[@]}" "$f" >/dev/null 2>&1; then
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
    if "${PARSER_CMD[@]}" "$f" >/dev/null 2>&1; then
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
