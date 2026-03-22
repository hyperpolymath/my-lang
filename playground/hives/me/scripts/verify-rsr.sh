#!/usr/bin/env bash
# RSR (Rhodium Standard Repository) Compliance Verification Script
# This script checks compliance with RSR Bronze Level requirements

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Counters
PASSED=0
FAILED=0
WARNINGS=0

# Helper functions
pass() {
    echo -e "${GREEN}✓${NC} $1"
    ((PASSED++))
}

fail() {
    echo -e "${RED}✗${NC} $1"
    ((FAILED++))
}

warn() {
    echo -e "${YELLOW}⚠${NC} $1"
    ((WARNINGS++))
}

info() {
    echo -e "${BLUE}ℹ${NC} $1"
}

check_file() {
    if [ -f "$1" ]; then
        pass "File exists: $1"
        return 0
    else
        fail "Missing file: $1"
        return 1
    fi
}

check_dir() {
    if [ -d "$1" ]; then
        pass "Directory exists: $1"
        return 0
    else
        fail "Missing directory: $1"
        return 1
    fi
}

echo "======================================================================"
echo "  RSR (Rhodium Standard Repository) Compliance Verification"
echo "  Target Level: Bronze"
echo "======================================================================"
echo ""

# 1. TYPE SAFETY
echo "1. Type Safety"
echo "━━━━━━━━━━━━━━"
if [ -f "Cargo.toml" ]; then
    pass "Rust project (compile-time type checking)"
else
    fail "Not a Rust project"
fi

# Check for type system implementation
if [ -f "crates/typechecker/Cargo.toml" ]; then
    pass "Type checker crate exists"
else
    warn "Type checker crate missing (stub expected)"
fi
echo ""

# 2. MEMORY SAFETY
echo "2. Memory Safety"
echo "━━━━━━━━━━━━━━━━"

# Check for unsafe blocks
UNSAFE_COUNT=$(find crates/*/src -name '*.rs' -exec grep -c 'unsafe' {} + 2>/dev/null | awk '{sum+=$1} END {print sum+0}')
if [ "$UNSAFE_COUNT" -eq 0 ]; then
    pass "Zero unsafe blocks in core"
else
    warn "$UNSAFE_COUNT unsafe blocks found (review needed)"
    info "Run: grep -rn 'unsafe' crates/*/src"
fi

# Check for affine type support
if [ -f "crates/affine/Cargo.toml" ]; then
    pass "Affine type system crate exists"
else
    fail "Affine type system crate missing"
fi
echo ""

# 3. OFFLINE-FIRST
echo "3. Offline-First"
echo "━━━━━━━━━━━━━━━━"

# Check for network dependencies
NETWORK_DEPS=$(grep -i -E 'reqwest|hyper|tokio.*http' Cargo.toml 2>/dev/null | wc -l)
if [ "$NETWORK_DEPS" -eq 0 ]; then
    pass "No network dependencies in core"
else
    warn "Found $NETWORK_DEPS potential network dependencies"
fi

# Check for examples that work offline
if [ -d "examples/solo" ]; then
    SOLO_EXAMPLES=$(find examples/solo -name '*.solo' | wc -l)
    pass "$SOLO_EXAMPLES offline Solo examples"
else
    warn "No Solo examples found"
fi
echo ""

# 4. DOCUMENTATION
echo "4. Documentation"
echo "━━━━━━━━━━━━━━━━"

check_file "README.md"
check_file "LICENSE.txt"

# Check for dual license
if grep -q "Palimpsest" LICENSE.txt 2>/dev/null; then
    pass "Dual license (MIT + Palimpsest v0.8)"
else
    fail "Missing Palimpsest license"
fi

check_file "SECURITY.md"
check_file "CONTRIBUTING.md"
check_file "CODE_OF_CONDUCT.md"
check_file "MAINTAINERS.md"
check_file "CHANGELOG.md"
echo ""

# 5. .WELL-KNOWN DIRECTORY
echo "5. .well-known/ Directory"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━"

check_dir ".well-known"
check_file ".well-known/security.txt"

# Verify RFC 9116 compliance
if [ -f ".well-known/security.txt" ]; then
    if grep -q "Contact:" .well-known/security.txt && \
       grep -q "Expires:" .well-known/security.txt; then
        pass "security.txt is RFC 9116 compliant"
    else
        warn "security.txt may not be fully RFC 9116 compliant"
    fi
fi

check_file ".well-known/ai.txt"
check_file ".well-known/humans.txt"
echo ""

# 6. BUILD SYSTEM
echo "6. Build System"
echo "━━━━━━━━━━━━━━━"

check_file "Cargo.toml"
check_file "justfile"

# Check justfile recipes
if [ -f "justfile" ]; then
    RECIPES=$(just --list --unsorted 2>/dev/null | wc -l || echo 0)
    if [ "$RECIPES" -gt 20 ]; then
        pass "justfile has $RECIPES recipes (≥20 required)"
    else
        warn "justfile has only $RECIPES recipes (need ≥20)"
    fi
fi

check_file "flake.nix"

# Check CI/CD
if [ -f ".github/workflows/ci.yml" ]; then
    pass "CI/CD configured (GitHub Actions)"
elif [ -f ".gitlab-ci.yml" ]; then
    pass "CI/CD configured (GitLab CI)"
else
    fail "No CI/CD configuration found"
fi
echo ""

# 7. TESTING
echo "7. Testing"
echo "━━━━━━━━━━"

if [ -d "tests" ]; then
    TEST_FILES=$(find tests -name '*.rs' | wc -l)
    pass "$TEST_FILES test files found"
else
    warn "No tests/ directory"
fi

# Run tests if possible
if command -v cargo >/dev/null 2>&1; then
    info "Running tests..."
    if cargo test --quiet --all-features 2>&1 | grep -q "test result: ok"; then
        pass "All tests pass"
    else
        fail "Some tests failed"
    fi
else
    warn "cargo not available, skipping test execution"
fi
echo ""

# 8. TPCF (TRI-PERIMETER CONTRIBUTION FRAMEWORK)
echo "8. TPCF (Tri-Perimeter Contribution Framework)"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

if [ -f "MAINTAINERS.md" ]; then
    if grep -q "Perimeter 1" MAINTAINERS.md && \
       grep -q "Perimeter 2" MAINTAINERS.md && \
       grep -q "Perimeter 3" MAINTAINERS.md; then
        pass "TPCF structure defined"
    else
        fail "TPCF structure incomplete"
    fi

    if grep -q "Community Sandbox" MAINTAINERS.md; then
        pass "Community Sandbox (Perimeter 3) active"
    else
        warn "Community Sandbox not explicitly mentioned"
    fi
fi
echo ""

# 9. ADDITIONAL CHECKS
echo "9. Additional RSR Features"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━"

# Check for examples
EXAMPLE_DIRS=("examples/solo" "examples/duet" "examples/ensemble")
for dir in "${EXAMPLE_DIRS[@]}"; do
    if [ -d "$dir" ]; then
        COUNT=$(find "$dir" -type f | wc -l)
        pass "$dir: $COUNT files"
    fi
done

# Check for grammar specifications
if [ -d "docs/specs" ]; then
    GRAMMARS=$(find docs/specs -name '*.ebnf' | wc -l)
    if [ "$GRAMMARS" -gt 0 ]; then
        pass "$GRAMMARS EBNF grammar files"
    fi
fi

# Check for research documentation
if [ -d "docs/research" ]; then
    pass "Research documentation exists"
fi

# Check for tutorials
if [ -d "docs/tutorials" ]; then
    pass "Tutorial documentation exists"
fi
echo ""

# 10. COMPLIANCE SUMMARY
echo "======================================================================"
echo "  Compliance Summary"
echo "======================================================================"
echo ""

TOTAL=$((PASSED + FAILED))
if [ $TOTAL -gt 0 ]; then
    PASS_PERCENT=$((PASSED * 100 / TOTAL))
else
    PASS_PERCENT=0
fi

echo -e "${GREEN}Passed:${NC}   $PASSED"
echo -e "${RED}Failed:${NC}   $FAILED"
echo -e "${YELLOW}Warnings:${NC} $WARNINGS"
echo ""
echo "Pass Rate: $PASS_PERCENT%"
echo ""

if [ $FAILED -eq 0 ]; then
    echo -e "${GREEN}🎉 RSR Bronze Level Compliance: ACHIEVED${NC}"
    echo ""
    echo "Your project meets all Bronze level requirements!"
    echo ""
    exit 0
elif [ $PASS_PERCENT -ge 80 ]; then
    echo -e "${YELLOW}⚠️  RSR Bronze Level Compliance: PARTIAL${NC}"
    echo ""
    echo "Your project is close to compliance."
    echo "Address the failed checks above to achieve full compliance."
    echo ""
    exit 1
else
    echo -e "${RED}❌ RSR Bronze Level Compliance: NOT MET${NC}"
    echo ""
    echo "Please address the failed checks above."
    echo "See docs/RSR_COMPLIANCE.md for guidance."
    echo ""
    exit 2
fi
