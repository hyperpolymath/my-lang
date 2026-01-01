# SPDX-License-Identifier: AGPL-3.0-or-later
# Justfile - hyperpolymath standard task runner

default:
    @just --list

# Build the project
build:
    @echo "Building..."

# Run tests
test:
    @echo "Testing..."

# Run lints
lint:
    @echo "Linting..."

# Clean build artifacts
clean:
    @echo "Cleaning..."

# Format code
fmt:
    @echo "Formatting..."

# Run all checks
check: lint test

# Prepare a release
release VERSION:
    @echo "Releasing {{VERSION}}..."

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
        # When hives are available, iterate:
        # for d in me solo duet ensemble; do
        #     run_demo "$d" || true
        # done
        echo "Run 'just demo <dialect>' once hives are installed."
        echo "Dialects: me, solo, duet, ensemble"
    else
        run_demo "{{dialect}}"
    fi

# Initialize submodules (part of golden-path)
init:
    git submodule update --init --recursive

# Verify playground health
verify:
    @echo "Verifying playground structure..."
    @test -d .machine_read && echo "[OK] .machine_read/ exists" || echo "[FAIL] .machine_read/ missing"
    @test -f .machine_read/ANCHOR.scm && echo "[OK] ANCHOR.scm exists" || echo "[FAIL] ANCHOR.scm missing"
    @test -f .machine_read/SPEC.playground.scm && echo "[OK] SPEC.playground.scm exists" || echo "[FAIL] SPEC.playground.scm missing"
    @test -d hives && echo "[OK] hives/ exists" || echo "[FAIL] hives/ missing"
    @echo "Verification complete."
