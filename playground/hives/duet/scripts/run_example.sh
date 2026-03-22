#!/usr/bin/env bash
# SPDX-License-Identifier: PMPL-1.0-or-later
#
# Run Duet examples
# Usage: ./run_example.sh <example_name>

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
EXAMPLES_DIR="${SCRIPT_DIR}/../examples"
HIVE_ROOT="${SCRIPT_DIR}/.."

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
PURPLE='\033[0;35m'
NC='\033[0m' # No Color

print_usage() {
    echo "Usage: $0 [options] <example_name>"
    echo ""
    echo "Options:"
    echo "  -l, --list        List available examples"
    echo "  -a, --all         Run all examples"
    echo "  -v, --verify      Run with verification enabled"
    echo "  --no-ai           Run without AI assistance"
    echo "  -h, --help        Show this help message"
    echo ""
    echo "Examples:"
    echo "  $0 01_synthesis_basic"
    echo "  $0 --verify 03_verification"
    echo "  $0 --list"
}

list_examples() {
    echo -e "${PURPLE}Available Duet examples:${NC}"
    echo ""
    for file in "${EXAMPLES_DIR}"/*.duet; do
        if [[ -f "$file" ]]; then
            local name
            name=$(basename "$file" .duet)
            echo "  ${name}"
        fi
    done
}

run_example() {
    local example_name="$1"
    local verify="${2:-false}"
    local ai="${3:-true}"
    local example_file="${EXAMPLES_DIR}/${example_name}.duet"

    if [[ ! -f "$example_file" ]]; then
        echo -e "${RED}Error: Example '${example_name}' not found${NC}"
        echo "Use --list to see available examples"
        exit 1
    fi

    echo -e "${PURPLE}Running Duet: ${example_name}${NC}"
    echo "----------------------------------------"

    local flags=""
    if [[ "$verify" == "true" ]]; then
        flags="${flags} --verify"
        echo -e "${GREEN}Verification: enabled${NC}"
    fi
    if [[ "$ai" == "false" ]]; then
        flags="${flags} --no-ai"
        echo -e "${YELLOW}AI Assistance: disabled${NC}"
    fi

    # Note: This is a placeholder - actual Duet compiler not yet implemented
    # Once implemented, replace with: duet run ${flags} "${example_file}"
    echo -e "${YELLOW}[Stub] Would run: duet run ${flags} ${example_file}${NC}"
    echo ""
    echo "File contents (first 30 lines):"
    head -30 "${example_file}"
    echo "..."
    echo ""
}

run_all_examples() {
    echo "Running all Duet examples..."
    echo ""

    for file in "${EXAMPLES_DIR}"/*.duet; do
        if [[ -f "$file" ]]; then
            local name
            name=$(basename "$file" .duet)
            run_example "$name" "false" "true"
            echo ""
        fi
    done

    echo -e "${GREEN}All examples completed!${NC}"
}

# Parse arguments
VERIFY="false"
AI="true"
ACTION=""
EXAMPLE=""

while [[ $# -gt 0 ]]; do
    case "$1" in
        -l|--list)
            ACTION="list"
            shift
            ;;
        -a|--all)
            ACTION="all"
            shift
            ;;
        -v|--verify)
            VERIFY="true"
            shift
            ;;
        --no-ai)
            AI="false"
            shift
            ;;
        -h|--help)
            print_usage
            exit 0
            ;;
        *)
            EXAMPLE="$1"
            shift
            ;;
    esac
done

# Execute
case "$ACTION" in
    list)
        list_examples
        ;;
    all)
        run_all_examples
        ;;
    *)
        if [[ -z "$EXAMPLE" ]]; then
            print_usage
            exit 1
        fi
        run_example "$EXAMPLE" "$VERIFY" "$AI"
        ;;
esac
