#!/usr/bin/env bash
# SPDX-License-Identifier: MPL-2.0
#
# Run Solo examples
# Usage: ./run_example.sh <example_name>

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
EXAMPLES_DIR="${SCRIPT_DIR}/../examples"
HIVE_ROOT="${SCRIPT_DIR}/.."

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

print_usage() {
    echo "Usage: $0 [options] <example_name>"
    echo ""
    echo "Options:"
    echo "  -l, --list     List available examples"
    echo "  -a, --all      Run all examples"
    echo "  -h, --help     Show this help message"
    echo ""
    echo "Examples:"
    echo "  $0 01_hello_world"
    echo "  $0 --list"
    echo "  $0 --all"
}

list_examples() {
    echo "Available Solo examples:"
    echo ""
    for file in "${EXAMPLES_DIR}"/*.solo; do
        if [[ -f "$file" ]]; then
            basename "$file" .solo
        fi
    done
}

run_example() {
    local example_name="$1"
    local example_file="${EXAMPLES_DIR}/${example_name}.solo"

    if [[ ! -f "$example_file" ]]; then
        echo -e "${RED}Error: Example '${example_name}' not found${NC}"
        echo "Use --list to see available examples"
        exit 1
    fi

    echo -e "${GREEN}Running: ${example_name}${NC}"
    echo "----------------------------------------"

    # Note: This is a placeholder - actual Solo compiler not yet implemented
    # Once implemented, replace with: solo run "${example_file}"
    echo -e "${YELLOW}[Stub] Would run: solo run ${example_file}${NC}"
    echo ""
    echo "File contents:"
    head -20 "${example_file}"
    echo "..."
    echo ""
}

run_all_examples() {
    echo "Running all Solo examples..."
    echo ""

    for file in "${EXAMPLES_DIR}"/*.solo; do
        if [[ -f "$file" ]]; then
            local name
            name=$(basename "$file" .solo)
            run_example "$name"
            echo ""
        fi
    done

    echo -e "${GREEN}All examples completed!${NC}"
}

# Main
case "${1:-}" in
    -l|--list)
        list_examples
        ;;
    -a|--all)
        run_all_examples
        ;;
    -h|--help|"")
        print_usage
        ;;
    *)
        run_example "$1"
        ;;
esac
