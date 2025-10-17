#!/usr/bin/env bash
#
# Run E2E tests for bwenv against real Bitwarden Secrets Manager
#
# Prerequisites:
# 1. Create .env.test with BITWARDEN_ACCESS_TOKEN=your_token
# 2. Have a test project in Bitwarden (default: "E2E-Test")
#
# Usage:
#   ./scripts/run-e2e-tests.sh [OPTIONS]
#
# Options:
#   --local     Run tests locally without Docker
#   --docker    Run tests in Docker (default)
#   --cleanup   Only run cleanup (delete test secrets)
#   --help      Show this help message

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# Default options
RUN_MODE="docker"
CLEANUP_ONLY=false

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --local)
            RUN_MODE="local"
            shift
            ;;
        --docker)
            RUN_MODE="docker"
            shift
            ;;
        --cleanup)
            CLEANUP_ONLY=true
            shift
            ;;
        --help)
            head -n 15 "$0" | tail -n +3 | sed 's/^# //'
            exit 0
            ;;
        *)
            echo -e "${RED}Unknown option: $1${NC}"
            exit 1
            ;;
    esac
done

# Check for .env.test file
if [ ! -f "$PROJECT_ROOT/.env.test" ]; then
    echo -e "${RED}Error: .env.test file not found${NC}"
    echo ""
    echo "Create .env.test with your Bitwarden Secrets Manager access token:"
    echo ""
    echo "  BITWARDEN_ACCESS_TOKEN=your_token_here"
    echo "  BITWARDEN_TEST_PROJECT=E2E-Test"
    echo ""
    echo "See .env.test.example for template"
    exit 1
fi

# Load environment variables
set -a
source "$PROJECT_ROOT/.env.test"
set +a

# Verify access token is set
if [ -z "${BITWARDEN_ACCESS_TOKEN:-}" ]; then
    echo -e "${RED}Error: BITWARDEN_ACCESS_TOKEN not set in .env.test${NC}"
    exit 1
fi

echo -e "${BLUE}=== bwenv E2E Test Runner ===${NC}"
echo ""
echo "Mode: $RUN_MODE"
echo "Project: ${BITWARDEN_TEST_PROJECT:-E2E-Test}"
echo ""

# Cleanup function
cleanup() {
    echo -e "${YELLOW}Cleaning up...${NC}"
    if [ "$RUN_MODE" = "docker" ]; then
        docker-compose -f "$PROJECT_ROOT/tests/docker/docker-compose.e2e.yml" down -v 2>/dev/null || true
    fi
}

# Register cleanup trap
trap cleanup EXIT INT TERM

# Run cleanup only
if [ "$CLEANUP_ONLY" = true ]; then
    echo -e "${BLUE}Running cleanup only...${NC}"
    cd "$PROJECT_ROOT"
    cargo run --release -- list --project "${BITWARDEN_TEST_PROJECT:-E2E-Test}" || true
    echo -e "${GREEN}Cleanup completed${NC}"
    exit 0
fi

# Run tests based on mode
if [ "$RUN_MODE" = "docker" ]; then
    echo -e "${BLUE}Building and running E2E tests in Docker...${NC}"
    echo ""

    cd "$PROJECT_ROOT"

    # Build and run with docker-compose
    docker-compose -f tests/docker/docker-compose.e2e.yml up \
        --build \
        --abort-on-container-exit \
        --exit-code-from e2e-tests

    TEST_EXIT_CODE=$?

    echo ""
    if [ $TEST_EXIT_CODE -eq 0 ]; then
        echo -e "${GREEN}✓ All E2E tests passed!${NC}"
    else
        echo -e "${RED}✗ E2E tests failed (exit code: $TEST_EXIT_CODE)${NC}"
    fi

    exit $TEST_EXIT_CODE

else
    echo -e "${BLUE}Running E2E tests locally...${NC}"
    echo ""

    cd "$PROJECT_ROOT"

    # Build release binary
    echo -e "${YELLOW}Building release binary...${NC}"
    cargo build --release

    # Export environment variables for tests
    export BWENV_BINARY="$PROJECT_ROOT/target/release/bwenv"
    export RUST_BACKTRACE=1
    export RUST_LOG="${RUST_LOG:-info}"

    # Run E2E tests
    echo -e "${YELLOW}Running E2E tests...${NC}"
    cargo test --test e2e -- --test-threads=1 --nocapture

    TEST_EXIT_CODE=$?

    echo ""
    if [ $TEST_EXIT_CODE -eq 0 ]; then
        echo -e "${GREEN}✓ All E2E tests passed!${NC}"
    else
        echo -e "${RED}✗ E2E tests failed (exit code: $TEST_EXIT_CODE)${NC}"
    fi

    exit $TEST_EXIT_CODE
fi
