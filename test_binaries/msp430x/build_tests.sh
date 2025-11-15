#!/bin/bash

# MSP430X Test Binary Compilation Script
# This script compiles test programs containing MSP430X extended instructions

set -e

# Configuration
MCU="msp430f5529"
CC="msp430-elf-gcc"
OBJDUMP="msp430-elf-objdump"
OBJCOPY="msp430-elf-objcopy"
READELF="msp430-elf-readelf"
SIZE="msp430-elf-size"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Output directory
OUTPUT_DIR="binaries"
DISASM_DIR="disassembly"

echo "MSP430X Test Binary Build Script"
echo "================================="
echo ""

# Check if compiler is available
if ! command -v $CC &> /dev/null; then
    echo -e "${RED}Error: $CC not found${NC}"
    echo "Please install the MSP430 GCC toolchain:"
    echo "  - TI MSP430 GCC: https://www.ti.com/tool/MSP430-GCC-OPENSOURCE"
    echo "  - Or via package manager (may vary by distro)"
    exit 1
fi

echo -e "${GREEN}Found toolchain:${NC}"
$CC --version | head -1
echo ""

# Create output directories
mkdir -p "$OUTPUT_DIR"
mkdir -p "$DISASM_DIR"

# Compilation function
compile_test() {
    local source=$1
    local name=$(basename "$source" .c)
    local elf="$OUTPUT_DIR/${name}.elf"
    local bin="$OUTPUT_DIR/${name}.bin"
    local hex="$OUTPUT_DIR/${name}.hex"
    local dis="$DISASM_DIR/${name}.dis"
    local lst="$DISASM_DIR/${name}.lst"

    echo -e "${YELLOW}Compiling: $source${NC}"

    # Compile with MSP430X support
    # -mmcu specifies the device (MSP430X capable)
    # -O0 for no optimization to keep instructions clear
    # -g for debug symbols
    # -Wall for all warnings
    $CC -mmcu=$MCU -O0 -g -Wall -o "$elf" "$source"

    if [ $? -eq 0 ]; then
        echo -e "${GREEN}  Successfully compiled $elf${NC}"

        # Generate disassembly
        $OBJDUMP -d -S "$elf" > "$dis"
        echo "  Generated disassembly: $dis"

        # Generate listing with headers
        $OBJDUMP -x -d -S "$elf" > "$lst"
        echo "  Generated listing: $lst"

        # Generate binary
        $OBJCOPY -O binary "$elf" "$bin"
        echo "  Generated binary: $bin"

        # Generate Intel HEX
        $OBJCOPY -O ihex "$elf" "$hex"
        echo "  Generated hex: $hex"

        # Show ELF header
        echo "  ELF Info:"
        $READELF -h "$elf" | grep -E "(Machine|Entry point)"

        # Show size
        echo "  Size:"
        $SIZE "$elf"

        echo ""
    else
        echo -e "${RED}  Failed to compile $source${NC}"
        return 1
    fi
}

# Compile all test files
for test_file in *.c; do
    if [ -f "$test_file" ]; then
        compile_test "$test_file"
    fi
done

echo -e "${GREEN}Build complete!${NC}"
echo ""
echo "Output locations:"
echo "  ELF binaries: $OUTPUT_DIR/*.elf"
echo "  Raw binaries: $OUTPUT_DIR/*.bin"
echo "  Intel HEX: $OUTPUT_DIR/*.hex"
echo "  Disassembly: $DISASM_DIR/*.dis"
echo "  Listings: $DISASM_DIR/*.lst"
echo ""
echo "To analyze in Binary Ninja:"
echo "  1. Open the .elf files directly"
echo "  2. Binary Ninja should auto-detect MSP430 architecture"
echo "  3. Compare disassembly with $DISASM_DIR/*.dis"
