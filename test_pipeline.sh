#!/usr/bin/env bash
# SPDX-License-Identifier: PMPL-1.0-or-later
# Test the complete My Language compilation pipeline

set -euo pipefail

echo "=== My Language Pipeline Test ==="
echo

# Build the project
echo "1. Building project..."
cargo build --release -p my-lang -p my-hir -p my-mir 2>&1 | grep -E "(Compiling|Finished)" || true
echo

# Test source file
SOURCE="examples/simple.my"
echo "2. Source program ($SOURCE):"
cat "$SOURCE"
echo

# Parse and run with interpreter
echo "3. Running with interpreter..."
cargo run --release --bin my -- interpret "$SOURCE"

echo
echo "=== Pipeline Test Complete ==="
