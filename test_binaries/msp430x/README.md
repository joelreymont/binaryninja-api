# MSP430X Test Binaries

This directory contains test programs for validating MSP430X architecture support in Binary Ninja.

## Prerequisites

Install the MSP430 GCC toolchain:

### Option 1: TI Official Toolchain
Download from: https://www.ti.com/tool/MSP430-GCC-OPENSOURCE

### Option 2: Package Manager (Linux)
```bash
# Debian/Ubuntu
sudo apt-get install gcc-msp430

# Note: Package manager versions may be older
```

### Verify Installation
```bash
msp430-elf-gcc --version
```

## Building Test Binaries

### Build All Tests
```bash
chmod +x build_tests.sh
./build_tests.sh
```

This will:
1. Compile all .c files in the directory
2. Generate ELF binaries in `binaries/`
3. Generate disassembly in `disassembly/`
4. Create both raw binary and Intel HEX formats

### Build Individual Test
```bash
msp430-elf-gcc -mmcu=msp430f5529 -O0 -g -Wall -o test.elf test_all_extended.c
```

## Test Files

### test_all_extended.c
Comprehensive test covering all MSP430X extended instructions:

- **Address Instructions**: MOVA, CMPA, ADDA, SUBA
- **Control Flow**: CALLA, BRA, RETA
- **Rotates/Shifts**: RRCM, RRAM, RLAM, RRUM (both .A and .W modes)
- **Stack Operations**: PUSHM.A, PUSHM.W, POPM.A, POPM.W
- **Mixed**: Combination of base MSP430 and MSP430X instructions

### test_simple.c (TODO)
Minimal test with just a few key instructions for quick validation.

## Output Files

After building, you'll find:

```
test_binaries/msp430x/
├── binaries/
│   ├── test_all_extended.elf    # ELF binary (load this in Binary Ninja)
│   ├── test_all_extended.bin    # Raw binary
│   └── test_all_extended.hex    # Intel HEX format
├── disassembly/
│   ├── test_all_extended.dis    # Disassembly (compare with Binary Ninja)
│   └── test_all_extended.lst    # Full listing with headers
└── ...
```

## Using with Binary Ninja

### Load ELF Binary
1. Open Binary Ninja
2. File -> Open -> Select `binaries/test_all_extended.elf`
3. Binary Ninja should auto-detect MSP430 architecture
4. Verify e_machine = 105 (MSP430)

### Verify Disassembly
Compare Binary Ninja's disassembly with the reference:
```bash
diff <disassembly_from_binja> disassembly/test_all_extended.dis
```

### Expected Behavior (After MSP430X Support)

Binary Ninja should:
- Recognize all MSP430X extended instructions
- Disassemble with correct mnemonics (MOVA, CALLA, etc.)
- Handle 20-bit addresses properly
- Lift instructions to correct LLIL
- Identify function boundaries correctly

## Target Device

Tests are compiled for **MSP430F5529** which is an MSP430X device with:
- 20-bit address bus (1MB address space)
- Extended instruction set support
- USB capability
- Commonly available on MSP-EXP430F5529 LaunchPad

## MSP430X Instruction Encoding Examples

### Extension Word Format
MSP430X instructions can include an extension word (18xx pattern):
```
Word 0 (Extension): 18xx xxxx xxxx xxxx
Word 1 (Instruction): xxxx xxxx xxxx xxxx
Word 2 (Optional Data): xxxx xxxx xxxx xxxx
```

### MOVA Encoding Example
```
MOVA #0x12345, R15

Bytes: [extension word] [opcode word] [immediate word]
Encoding varies by addressing mode
```

Refer to TI documentation (SLAU391F) for complete encoding details.

## Troubleshooting

### Compiler Not Found
```
Error: msp430-elf-gcc not found
```
Solution: Install MSP430 GCC toolchain (see Prerequisites)

### Unknown MCU
```
Error: unknown MCU 'msp430f5529'
```
Solution: Your toolchain may not include this device. Try:
- msp430f5438
- msp430fr5969
- msp430fr5994

### Inline Assembly Issues
If inline assembly fails, the toolchain may not support MSP430X instructions.
Verify you have a recent version (8.3.0+).

## References

- MSP430X CPU User's Guide: SLAU391F
- MSP430 GCC Documentation: https://gcc.gnu.org/onlinedocs/gcc/MSP430-Options.html
- Binary Ninja MSP430 Module: arch/msp430/
