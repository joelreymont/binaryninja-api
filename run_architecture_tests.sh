#!/bin/bash
#
# Architecture Test Runner
#
# Runs all architecture-specific tests for Binary Ninja fixes.
#
# Usage: ./run_architecture_tests.sh
#
# Requirements:
#   - Binary Ninja installation
#   - Python 3 with binaryninja module available
#
# Exit codes:
#   0 - All tests passed
#   1 - Some tests failed
#   2 - Binary Ninja not found

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Check if Binary Ninja is available
echo "Checking for Binary Ninja installation..."
if ! python3 -c "import binaryninja" 2>/dev/null; then
    echo -e "${RED}ERROR: Binary Ninja Python module not found${NC}"
    echo "Please ensure Binary Ninja is installed and the Python API is available."
    echo "See: https://docs.binary.ninja/dev/batch.html"
    exit 2
fi

echo -e "${GREEN}Binary Ninja found${NC}"
echo ""

# Track overall results
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0

# Function to run a test
run_test() {
    local test_file="$1"
    local test_name="$2"

    echo -e "${YELLOW}========================================${NC}"
    echo -e "${YELLOW}Running: $test_name${NC}"
    echo -e "${YELLOW}========================================${NC}"

    if [ ! -f "$test_file" ]; then
        echo -e "${RED}SKIP: Test file not found: $test_file${NC}"
        return 0
    fi

    TOTAL_TESTS=$((TOTAL_TESTS + 1))

    if python3 "$test_file"; then
        echo -e "${GREEN}PASS: $test_name${NC}"
        PASSED_TESTS=$((PASSED_TESTS + 1))
        return 0
    else
        echo -e "${RED}FAIL: $test_name${NC}"
        FAILED_TESTS=$((FAILED_TESTS + 1))
        return 1
    fi
}

# Run all tests
echo "Starting architecture tests..."
echo ""

# RISC-V JALR tests
run_test "arch/riscv/test_riscv_jalr.py" "RISC-V JALR Branch Detection"
echo ""

# MIPS64R6 JALR tests
run_test "arch/mips/test_mips64r6_jalr.py" "MIPS64R6 JALR Return Recognition"
echo ""

# x86 BEXTR tests
run_test "arch/x86/test_bextr_lifting.py" "x86 BEXTR Semantic Lifting"
echo ""

# ARM64 atomic intrinsics tests
run_test "arch/arm64/test_atomic_minmax.py" "ARM64 LSE Atomic MIN/MAX Intrinsics"
echo ""

# Summary
echo -e "${YELLOW}========================================${NC}"
echo -e "${YELLOW}Test Summary${NC}"
echo -e "${YELLOW}========================================${NC}"
echo "Total test suites: $TOTAL_TESTS"
echo -e "Passed: ${GREEN}$PASSED_TESTS${NC}"
echo -e "Failed: ${RED}$FAILED_TESTS${NC}"
echo ""

if [ $FAILED_TESTS -eq 0 ]; then
    echo -e "${GREEN}All tests passed!${NC}"
    exit 0
else
    echo -e "${RED}Some tests failed${NC}"
    exit 1
fi
