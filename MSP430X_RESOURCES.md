# MSP430X Resources

This document contains links and information for MSP430X architecture development, including documentation, test binaries, and tools.

## Official Documentation

### Primary Reference
- **SLAU391F** - MSP430X CPU User's Guide (August 2012, Revised March 2018)
  - URL: https://www.ti.com/lit/ug/slau391f/slau391f.pdf
  - Contains complete MSP430X instruction set description and instruction maps

### Family User Guides
- **SLAU144J** - MSP430x2xx Family User's Guide
  - URL: https://courses.cs.washington.edu/courses/cse466/13au/pdfs/slau144j.pdf
  - Sections on MSP430 and MSP430X Instructions, MSP430X Extended Instructions

- **SLAU208G** - MSP430x5xx/MSP430x6xx Family User's Guide
  - Contains MSP430 and MSP430X Instructions, CPUX architecture details
  - Chapter 6 covers CPUX (extended CPU architecture)

- **SLAU445G** - MSP430FR4xx and MSP430FR2xx Family User's Guide
  - URL: https://web.eng.fiu.edu/watsonh/intromicros/m0-index/msp430fr2433/referencemanualslau445g.pdf
  - Contains MSP430 and MSP430X Instructions, Extended Instruction Binary Descriptions

### Application Notes
- **SLAA801** - MSP430X Application Note (October 2017)
  - URL: https://www.ti.com/lit/an/slaa801/slaa801.pdf
  - Explains 20-bit addressing and extended instruction set

- **MSP430 EABI Application Report**
  - URL: https://schaumont.dyn.wpi.edu/ece4530f19/pdf/msp430applicationbinaryinterface.pdf
  - Embedded Application Binary Interface specification

### Compiler and Tools Documentation
- **SLAU131R** - MSP430 Assembly Language Tools v18.1.0.LTS User's Guide
  - URL: https://www.ti.com/lit/ug/slau131r/slau131r.pdf
  - ELF object file format details

- **MSP430 Optimizing C/C++ Compiler User's Guide**
  - Contains section on "Compiling for 20-Bit MSP430X Devices"

## Test Binaries and Examples

### TI Official Examples
- **MSP430 Function Code Examples**
  - URL: https://www.ti.com/tool/MSP430-FUNCTION-CODE-EXAMPLES
  - Download: https://software-dl.ti.com/msp430/msp430_public_sw/mcu/msp430/MSP430-FUNCTION-CODE-EXAMPLES/latest/index_FDS.html
  - Collection of code examples for various MSP430 devices including MSP430X

### Open Source Firmware Projects

#### OpenChronos
- **Repository**: https://github.com/BenjaminSoelberg/openchronos-ng-elf
- **Description**: Fully modular opensource firmware for the eZ430 Chronos using TI's msp430-elf-gcc
- **Value**: Real-world MSP430X firmware with ELF binaries

#### MSP430 Examples
- **Repository**: https://github.com/ticepd/msp430-examples
- **Description**: Code Composer Studio examples for MSP430

- **Repository**: https://github.com/ab2tech/msp430
- **Description**: Collection of publicly-released MSP430 libraries and examples

### Creating Custom Test Binaries

#### Compiler Setup
Install msp430-elf-gcc toolchain:
- **TI MSP430 GCC**: https://www.ti.com/tool/MSP430-GCC-OPENSOURCE
- **Source Archive**: https://github.com/pabigot/msp430-elf

#### Example Test Program
Create a file `test_msp430x.c`:

```c
// MSP430X Extended Instruction Test Cases
// Compile with: msp430-elf-gcc -mmcu=msp430f5529 -o test.elf test_msp430x.c

#include <msp430.h>

// Test CALLA instruction (20-bit call)
void __attribute__((noinline)) test_calla(void) {
    asm volatile("calla #0x10000");
}

// Test MOVA instruction (move 20-bit address)
void __attribute__((noinline)) test_mova(void) {
    asm volatile("mova #0x12345, r15");
}

// Test ADDA instruction (add to address)
void __attribute__((noinline)) test_adda(void) {
    asm volatile("adda r14, r15");
}

// Test SUBA instruction (subtract from address)
void __attribute__((noinline)) test_suba(void) {
    asm volatile("suba #0x1000, r15");
}

// Test CMPA instruction (compare address)
void __attribute__((noinline)) test_cmpa(void) {
    asm volatile("cmpa #0x10000, r15");
}

// Test PUSHM.A instruction (push multiple registers, address mode)
void __attribute__((noinline)) test_pushm_a(void) {
    asm volatile("pushm.a #4, r15");
}

// Test POPM.A instruction (pop multiple registers, address mode)
void __attribute__((noinline)) test_popm_a(void) {
    asm volatile("popm.a #4, r15");
}

// Test PUSHM.W instruction (push multiple registers, word mode)
void __attribute__((noinline)) test_pushm_w(void) {
    asm volatile("pushm.w #3, r15");
}

// Test POPM.W instruction (pop multiple registers, word mode)
void __attribute__((noinline)) test_popm_w(void) {
    asm volatile("popm.w #3, r15");
}

// Test RRCM instruction (rotate right through carry, multiple bits)
void __attribute__((noinline)) test_rrcm(void) {
    asm volatile("rrcm.a #2, r15");
}

// Test RRAM instruction (rotate right arithmetic, multiple bits)
void __attribute__((noinline)) test_rram(void) {
    asm volatile("rram.a #3, r14");
}

// Test RLAM instruction (rotate left arithmetic, multiple bits)
void __attribute__((noinline)) test_rlam(void) {
    asm volatile("rlam.a #2, r13");
}

// Test RRUM instruction (rotate right unsigned, multiple bits)
void __attribute__((noinline)) test_rrum(void) {
    asm volatile("rrum.a #4, r12");
}

// Test BRA instruction (branch, 20-bit address)
void __attribute__((noinline)) test_bra(void) {
    asm volatile("bra #test_calla");
}

// Test RETA instruction (return from subroutine, 20-bit)
void __attribute__((noinline)) test_reta(void) {
    asm volatile("reta");
}

// Main function to call all tests
int main(void) {
    WDTCTL = WDTPW | WDTHOLD;  // Stop watchdog timer

    test_calla();
    test_mova();
    test_adda();
    test_suba();
    test_cmpa();
    test_pushm_a();
    test_popm_a();
    test_pushm_w();
    test_popm_w();
    test_rrcm();
    test_rram();
    test_rlam();
    test_rrum();
    test_bra();
    test_reta();

    while(1);
    return 0;
}
```

#### Compilation Commands
```bash
# Compile for MSP430F5529 (MSP430X device)
msp430-elf-gcc -mmcu=msp430f5529 -o test_msp430x.elf test_msp430x.c

# Generate disassembly for verification
msp430-elf-objdump -d test_msp430x.elf > test_msp430x.dis

# Generate binary
msp430-elf-objcopy -O binary test_msp430x.elf test_msp430x.bin

# Display ELF header
msp430-elf-readelf -h test_msp430x.elf
```

#### Common MSP430X Devices
Use these with `-mmcu` flag:
- msp430f5529 (USB MSP430F5529 LaunchPad)
- msp430f5438 (MSP430F5438 Experimenter Board)
- msp430f5510
- msp430fr5969
- msp430fr5994
- msp430fr6989

## Tools and Utilities

### Binary Analysis Tools
- **python-msp430-tools**
  - Repository: https://github.com/zsquareplusc/python-msp430-tools
  - Works with ELF, ihex, titext, hexdump formats
  - Useful for binary conversion and inspection

- **BSL430.NET**
  - Repository: https://github.com/parezj/BSL430.NET
  - Support for Intel-HEX, TI-TXT, ELF and SREC formats
  - Can convert, combine, and hex-edit firmware

### Development Tools
- **Code Composer Studio (CCS)**
  - IDE for MSP430 development
  - Includes debugger and project examples

- **MSP430 GCC Toolchain**
  - Repository: https://github.com/cjacker/opensource-toolchain-msp430
  - Open source toolchain guide

## Rust Crates

### msp430-asm
- **Repository**: https://github.com/jrozner/msp430-asm
- **Current Version**: 0.2
- **Description**: Disassembly library for MSP430 machine code
- **Status**: Need to verify MSP430X extended instruction support

### Related Crates
- **msp430**: https://github.com/rust-embedded/msp430
  - Low-level access to MSP430 microcontrollers

- **msp430-rt**: https://lib.rs/crates/msp430-rt
  - Minimal runtime/startup for MSP430 microcontrollers

## Educational Resources

### Tutorials and Guides
- **WPI MSP430 Course Materials**
  - Lecture 2 - The MSP-430 core: https://schaumont.dyn.wpi.edu/ece4530f19/lectures/lecture02-notes.html
  - Lecture 3 - The MSP-430 Software Design Flow: https://schaumont.dyn.wpi.edu/ece4530f19/lectures/lecture03-notes.html
  - Contains practical examples and disassembly

- **Simply Embedded Tutorial**
  - MSP430 Architecture: http://www.simplyembedded.org/tutorials/msp430-architecture/

- **Full MSP430 Instruction Set Reference**
  - URL: https://sites.google.com/site/arch1utep/home/reference/full-msp430-instruction-set

### Wikipedia
- **TI MSP430 Wikipedia**
  - URL: https://en.wikipedia.org/wiki/TI_MSP430
  - Overview of MSP430 and MSP430X architectures

## LLVM MSP430X Support

### Patches and Development
- **LLVM Review D110723**
  - URL: https://reviews.llvm.org/D110723
  - MSP430X shift instructions assembler and MC support
  - Shows instruction encoding details

## Online CTF Platforms

### Microcorruption
- **URL**: https://microcorruption.com
- **Description**: CTF platform with MSP430 challenges
- **Note**: Check if any challenges use MSP430X extended instructions

## ELF Binary Format

### e_machine Value
- **MSP430**: e_machine = 105 (0x69)
- **Endianness**: Little Endian
- **Address Size**: 16-bit (base MSP430) or 20-bit (MSP430X)

### Binary View Registration
Current registration in Binary Ninja (from lib.rs):
```rust
if let Ok(bvt) = BinaryViewType::by_name("ELF") {
    bvt.register_arch(105, Endianness::LittleEndian, arch);
}
```

## MSP430X Instruction Set Summary

### Extended Instructions by Category

#### Address Instructions (20-bit operations)
- MOVA - Move address
- CMPA - Compare address
- ADDA - Add to address
- SUBA - Subtract from address

#### Control Flow (20-bit)
- CALLA - Call subroutine
- BRA - Branch
- RETA - Return from subroutine

#### Rotate/Shift (multiple bits)
- RRCM - Rotate right through carry, multiple bits
- RRAM - Rotate right arithmetic, multiple bits
- RLAM - Rotate left arithmetic, multiple bits
- RRUM - Rotate right unsigned, multiple bits

#### Stack Operations
- PUSHM.A - Push multiple registers (address/20-bit mode)
- PUSHM.W - Push multiple registers (word/16-bit mode)
- POPM.A - Pop multiple registers (address/20-bit mode)
- POPM.W - Pop multiple registers (word/16-bit mode)

### Instruction Encoding
- Extension word format provides 20-bit addressing
- Instructions can be 1-3 words (2-6 bytes)
- Extension word precedes the instruction word
- All addresses, indexes, and immediate numbers have 20-bit values when preceded by extension word

## Next Steps

1. Download or compile at least 2-3 test binaries with MSP430X instructions
2. Verify binaries contain extended instructions using objdump
3. Test current Binary Ninja MSP430 support with MSP430X binaries to identify gaps
4. Begin implementation following the plan in MSP430X_IMPLEMENTATION_PLAN.md
