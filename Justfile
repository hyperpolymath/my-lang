# SPDX-License-Identifier: MPL-2.0
# Owner: Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk>
# Justfile — hyperpolymath standard task runner
#
# NOTE: every comment here must start with `#`. A `//` C-style comment makes
# the whole file parse-dead (`error: unknown start of token '.'`) and every
# recipe unavailable — which is exactly what happened to this file until
# 2026-07-27, silently breaking the README's documented golden path.
#
# my-llvm needs a system LLVM 21 (LLVM_SYS_211_PREFIX). The default build/test
# recipes exclude it so the golden path works on a clean checkout; use the
# `-all` variants when you have the toolchain. CI does the same (coverage.yml).

default:
    @just --list

# Build the workspace (excludes my-llvm — needs system LLVM 21)
build:
    cargo build --workspace --exclude my-llvm

# Build everything including the LLVM back end (requires LLVM 21)
build-all:
    cargo build --workspace

# Run unit + conformance tests (excludes my-llvm)
test:
    cargo test --workspace --exclude my-llvm

# Run the full test suite including the LLVM back end (requires LLVM 21)
test-all:
    cargo test --workspace

# Run lints
lint:
    cargo clippy --workspace --exclude my-llvm --all-targets

# Format code
fmt:
    cargo fmt --all

# Check formatting without writing (CI-style)
fmt-check:
    cargo fmt --all -- --check

# Clean build artifacts
clean:
    cargo clean

# Run all checks
check: fmt-check lint test

# Machine-check the Coq solo-core (the authoritative proof track)
proofs-coq:
    cd proofs/verification/coq/solo-core && \
      coq_makefile -f _CoqProject -o CoqMakefile && \
      make -f CoqMakefile

# Machine-check the Idris2 solo-core
proofs-idris:
    cd proofs/verification/idris/solo-core && idris2 --build solo-core.ipkg

# Both proof tracks
proofs: proofs-coq proofs-idris

# End-to-end pipeline smoke test (build -> parse -> interpret an example)
pipeline:
    ./test_pipeline.sh

# Prepare a release
release VERSION:
    @echo "Releasing {{VERSION}}..."
    @echo "Set version = \"{{VERSION}}\" in Cargo.toml [workspace.package] and .machine_readable/6a2/STATE.a2ml, then tag."

# Run dialect demos (per golden-path contract)
# Usage: just demo [dialect]
demo dialect="all":
    #!/usr/bin/env bash
    set -euo pipefail

    run_demo() {
        local dialect="$1"
        local hive_path="./hives/${dialect}-hive"

        if [[ -d "$hive_path" ]]; then
            echo "=========================================="
            echo "Running ${dialect^^} demo..."
            echo "=========================================="
            if [[ -f "$hive_path/justfile" ]]; then
                just -f "$hive_path/justfile" demo
            elif [[ -f "$hive_path/Mustfile" ]]; then
                must -f "$hive_path/Mustfile" demo
            else
                echo "No demo harness found for $dialect"
                return 1
            fi
        else
            echo "Hive not available: $dialect (path: $hive_path)"
            echo "Available dialects: me, solo, duet, ensemble"
            return 1
        fi
    }

    if [[ "{{dialect}}" == "all" ]]; then
        echo "My-Lang Playground Demo"
        echo "========================"
        echo ""
        echo "Note: Hives are coming soon. Add submodules to ./hives/"
        echo ""
        echo "Run 'just demo <dialect>' once hives are installed."
        echo "Dialects: me, solo, duet, ensemble"
    else
        run_demo "{{dialect}}"
    fi

# Initialize submodules (part of golden-path)
init:
    git submodule update --init --recursive

# (The previous version checked a `.machine_read/` path that has never existed
# and swallowed every failure with `||`, so it always reported success.)
# Verify repository structure — exits non-zero when something is missing
verify:
    #!/usr/bin/env bash
    set -uo pipefail
    fail=0
    for p in .machine_readable .machine_readable/6a2/STATE.a2ml \
             .hypatia-baseline.json .hypatia-ignore \
             proofs/STATUS.md Cargo.toml; do
        if [[ -e "$p" ]]; then echo "[OK]   $p"
        else echo "[FAIL] $p missing"; fail=1; fi
    done
    exit "$fail"

# Run panic-attacker pre-commit scan
assail:
    @command -v panic-attack >/dev/null 2>&1 && panic-attack assail . || echo "panic-attack not found — install from https://github.com/hyperpolymath/panic-attacker"

secret-scan-trufflehog:
    @command -v trufflehog >/dev/null && trufflehog filesystem . --only-verified || true
