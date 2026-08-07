#!/usr/bin/env bash
# SPDX-License-Identifier: MPL-2.0
# SPDX-FileCopyrightText: 2026 Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk>

#
# Run Ensemble examples
# Usage: ./run_example.sh <example_name>

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
EXAMPLES_DIR="${SCRIPT_DIR}/../examples"
HIVE_ROOT="${SCRIPT_DIR}/.."

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

print_usage() {
    echo "Usage: $0 [options] <example_name>"
    echo ""
    echo "Options:"
    echo "  -l, --list           List available examples"
    echo "  -a, --all            Run all examples"
    echo "  -d, --distributed    Run in distributed mode"
    echo "  -s, --single         Run in single-node mode (default)"
    echo "  --nodes <n>          Number of simulated nodes (default: 3)"
    echo "  --audit              Enable audit logging"
    echo "  -h, --help           Show this help message"
    echo ""
    echo "Examples:"
    echo "  $0 01_agent_basics"
    echo "  $0 --distributed 05_distributed_agents"
    echo "  $0 --nodes 5 --audit 02_multi_agent_workflow"
}

list_examples() {
    echo -e "${CYAN}Available Ensemble examples:${NC}"
    echo ""
    for file in "${EXAMPLES_DIR}"/*.ensemble; do
        if [[ -f "$file" ]]; then
            local name
            name=$(basename "$file" .ensemble)
            echo "  ${name}"
        fi
    done
}

run_example() {
    local example_name="$1"
    local distributed="${2:-false}"
    local nodes="${3:-3}"
    local audit="${4:-false}"
    local example_file="${EXAMPLES_DIR}/${example_name}.ensemble"

    if [[ ! -f "$example_file" ]]; then
        echo -e "${RED}Error: Example '${example_name}' not found${NC}"
        echo "Use --list to see available examples"
        exit 1
    fi

    echo -e "${CYAN}Running Ensemble: ${example_name}${NC}"
    echo "----------------------------------------"

    local flags=""
    if [[ "$distributed" == "true" ]]; then
        flags="${flags} --distributed --nodes ${nodes}"
        echo -e "${GREEN}Mode: Distributed (${nodes} nodes)${NC}"
    else
        echo -e "${GREEN}Mode: Single-node${NC}"
    fi

    if [[ "$audit" == "true" ]]; then
        flags="${flags} --audit"
        echo -e "${GREEN}Audit: enabled${NC}"
    fi

    # Note: This is a placeholder - actual Ensemble compiler not yet implemented
    # Once implemented, replace with: ensemble run ${flags} "${example_file}"
    echo -e "${YELLOW}[Stub] Would run: ensemble run ${flags} ${example_file}${NC}"
    echo ""
    echo "File contents (first 40 lines):"
    head -40 "${example_file}"
    echo "..."
    echo ""
}

run_all_examples() {
    echo "Running all Ensemble examples..."
    echo ""

    for file in "${EXAMPLES_DIR}"/*.ensemble; do
        if [[ -f "$file" ]]; then
            local name
            name=$(basename "$file" .ensemble)
            run_example "$name" "false" "3" "false"
            echo ""
        fi
    done

    echo -e "${GREEN}All examples completed!${NC}"
}

# Parse arguments
DISTRIBUTED="false"
NODES="3"
AUDIT="false"
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
        -d|--distributed)
            DISTRIBUTED="true"
            shift
            ;;
        -s|--single)
            DISTRIBUTED="false"
            shift
            ;;
        --nodes)
            NODES="$2"
            shift 2
            ;;
        --audit)
            AUDIT="true"
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
        run_example "$EXAMPLE" "$DISTRIBUTED" "$NODES" "$AUDIT"
        ;;
esac
