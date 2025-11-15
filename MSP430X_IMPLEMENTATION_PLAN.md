# MSP430X Architecture Module Implementation Plan

## Overview

This document outlines the implementation plan for adding MSP430X (20-bit extended) architecture support to Binary Ninja's MSP430 architecture module.

## Background

The current MSP430 architecture module in Binary Ninja supports the base MSP430 instruction set (16-bit architecture). The MSP430X extension adds 20-bit addressing capabilities and an extended instruction set to support operations on 20-bit addresses.

Reference Issue: https://github.com/Vector35/binaryninja-api/issues/7620

## Current State

The existing MSP430 module is located at `arch/msp430` and is implemented in Rust using:
- Binary Ninja Rust API
- `msp430-asm` crate (v0.2) for instruction decoding
- Supports base MSP430 instruction set (27 core instructions)
- 16-bit address space
- ELF binary format support (e_machine = 105)

Key files:
- `arch/msp430/src/lib.rs` - Plugin initialization and calling conventions
- `arch/msp430/src/architecture.rs` - Architecture implementation (address_size: 2 bytes, max_instr_len: 6 bytes)
- `arch/msp430/src/lift.rs` - LLIL lifting implementation
- `arch/msp430/src/register.rs` - Register definitions
- `arch/msp430/src/flag.rs` - Flag definitions

## MSP430X Extended Architecture Features

### Address Space
- 20-bit addressing (up to 1MB address space vs 64KB for base MSP430)
- Addresses from 0x00000 to 0xFFFFF
- Extension word format for 20-bit immediate values and addresses

### New Instructions

Based on research, the MSP430X extended instruction set includes:

#### Address Instructions
- MOVA - Move address (20-bit)
- ADDA - Add to address register
- SUBA - Subtract from address register
- CMPA - Compare address

#### Branch and Call Instructions
- CALLA - Call subroutine (20-bit address)
- BRA - Branch (20-bit address)
- RETA - Return from subroutine (20-bit)

#### Rotate/Shift Instructions
- RRCM - Rotate right through carry multiple bits
- RRAM - Rotate right arithmetic multiple bits
- RLAM - Rotate left arithmetic multiple bits
- RRUM - Rotate right unsigned multiple bits

#### Stack Instructions
- PUSHM.A - Push multiple registers (20-bit/address mode)
- PUSHM.W - Push multiple registers (16-bit/word mode)
- POPM.A - Pop multiple registers (20-bit/address mode)
- POPM.W - Pop multiple registers (16-bit/word mode)

### Extension Word Format

MSP430X instructions can include an extension word that precedes the instruction to provide:
- 20-bit source and destination addresses
- 20-bit immediate values
- Additional addressing mode information

Instructions with extension words can be up to 3 words (6 bytes) in length.

## Implementation Requirements

### Phase 1: Research and Dependencies

1. Review the `msp430-asm` crate to determine current MSP430X support
   - If not supported, we need to either:
     - Fork and extend the crate
     - Find an alternative crate
     - Implement our own decoder

2. Obtain test binaries for MSP430X
   - Sources identified:
     - TI MSP430 Function Code Examples: https://www.ti.com/tool/MSP430-FUNCTION-CODE-EXAMPLES
     - TI Code Composer Studio examples
     - Compile custom test cases using msp430-elf-gcc
     - OpenChronos firmware: https://github.com/BenjaminSoelberg/openchronos-ng-elf
     - Check microcorruption.com for MSP430X challenges

3. Documentation sources:
   - SLAU391F - MSP430X CPU User's Guide (primary reference)
   - SLAU144J - MSP430x2xx Family User's Guide
   - SLAU208G - MSP430x5xx/MSP430x6xx Family User's Guide
   - SLAA801 - Application note on MSP430X

### Phase 2: Architecture Updates

1. Update architecture properties in `architecture.rs`:
   - Change `address_size()` to return 3 bytes (20-bit) or make it configurable
   - Consider creating separate MSP430 (16-bit) and MSP430X (20-bit) architecture variants
   - Update `max_instr_len()` if needed (currently 6 bytes, which should accommodate extension words)

2. Register updates in `register.rs`:
   - Extend registers to support 20-bit values where applicable
   - R0 (PC) - 20-bit program counter
   - R1 (SP) - 20-bit stack pointer
   - General purpose registers may remain 16-bit or support 20-bit depending on instruction

3. Update calling conventions in `lib.rs`:
   - Review EABI calling convention for 20-bit address space
   - Update stack operations for 20-bit addresses

### Phase 3: Instruction Decoding

1. Evaluate `msp430-asm` crate capabilities:
   - Check if it supports MSP430X extended instructions
   - If not, determine effort to add support

2. Implement/extend decoder for:
   - Extension word parsing
   - Extended instruction decoding (MOVA, CMPA, ADDA, SUBA, etc.)
   - 20-bit address calculation
   - Multiple register push/pop instructions

3. Update `instruction_info()` in `architecture.rs`:
   - Add branch handling for CALLA, BRA, RETA
   - Handle 20-bit branch targets
   - Support for PUSHM/POPM stack operations

4. Update `instruction_text()` in `architecture.rs`:
   - Generate tokens for extended instructions
   - Format 20-bit addresses properly
   - Display extension word information

### Phase 4: LLIL Lifting

1. Update `lift.rs` to lift extended instructions:
   - MOVA, ADDA, SUBA, CMPA - 20-bit address operations
   - CALLA, BRA, RETA - 20-bit control flow
   - RRCM, RRAM, RLAM, RRUM - multi-bit rotations
   - PUSHM, POPM - multiple register stack operations

2. Handle 20-bit address arithmetic:
   - Ensure proper sign extension and masking
   - Handle address wrapping behavior

3. Update flag handling for extended instructions:
   - Extended instructions may have different flag update semantics

### Phase 5: Testing

1. Create test suite:
   - Disassembly tests for all extended instructions
   - LLIL lifting verification
   - Binary analysis workflow tests

2. Test with real-world binaries:
   - TI example code
   - OpenChronos firmware
   - Custom test programs

3. Verify ELF loading:
   - Ensure 20-bit addresses are handled correctly
   - Test relocations and symbols

### Phase 6: Documentation

1. Update architecture documentation
2. Add code comments for extended instruction handling
3. Create example analysis scripts if needed

## Technical Challenges

1. Decoder implementation:
   - The `msp430-asm` crate may not support MSP430X
   - May need to implement our own decoder or fork the crate

2. Address size handling:
   - Binary Ninja expects consistent address sizes
   - Need to determine if we need separate MSP430/MSP430X architectures or a unified one

3. Extension word parsing:
   - Need to correctly decode and interpret extension words
   - Handle variable-length instruction encoding

4. Register size flexibility:
   - Some instructions operate on 16-bit registers, others on 20-bit
   - Need to model this correctly in LLIL

5. Binary format:
   - Determine if MSP430X uses same ELF e_machine value (105) or different
   - May need separate binary view registration

## Test Binary Sources

### Option 1: TI Official Examples
- MSP430 Function Code Examples: https://www.ti.com/tool/MSP430-FUNCTION-CODE-EXAMPLES
- Download and compile with msp430-elf-gcc targeting MSP430X devices

### Option 2: Open Source Firmware
- OpenChronos: https://github.com/BenjaminSoelberg/openchronos-ng-elf
- Various MSP430 projects on GitHub with ELF outputs

### Option 3: Custom Test Cases
Create minimal test programs with msp430-elf-gcc:
```c
// test_msp430x.c - compile with -mmcu=msp430f5529 or similar MSP430X device
void test_calla(void) {
    // CALLA instruction test
    asm("calla #0x10000");
}

void test_mova(void) {
    // MOVA instruction test
    asm("mova #0x12345, r15");
}

void test_pushm(void) {
    // PUSHM instruction test
    asm("pushm.a #4, r15");
}
```

Compile with:
```bash
msp430-elf-gcc -mmcu=msp430f5529 -o test.elf test_msp430x.c
```

### Option 4: Examine Existing Binaries
- Search GitHub for compiled MSP430X ELF files
- Use python-msp430-tools to work with various binary formats

## Recommended Approach

### Initial Investigation
1. Check if `msp430-asm` v0.2 supports MSP430X instructions
   - Clone https://github.com/jrozner/msp430-asm
   - Review source code for extended instruction support
   - Check issues/PRs for MSP430X work

2. Obtain at least 2-3 test binaries with MSP430X instructions
   - Compile simple test cases using msp430-elf-gcc
   - Download OpenChronos or other open-source MSP430X firmware

3. Analyze test binaries with objdump
   - Verify they contain extended instructions
   - Document instruction encodings found

### Development Strategy

Option A: If `msp430-asm` supports MSP430X
- Upgrade to newer version if needed
- Proceed with architecture updates and lifting

Option B: If `msp430-asm` doesn't support MSP430X
- Fork the crate and add extended instruction support
- Submit upstream PR to benefit community
- Update Cargo.toml to use forked version

Option C: Implement custom decoder
- More work but gives full control
- Consider if `msp430-asm` is abandoned or unsuitable

### Phased Rollout
1. Start with subset of most common extended instructions
   - MOVA, CALLA, BRA, RETA (control flow)
   - ADDA, SUBA (address arithmetic)

2. Add remaining extended instructions incrementally

3. Test thoroughly at each phase

## Effort Estimate

Based on the complexity analysis:

- Phase 1 (Research): 1-2 days
- Phase 2 (Architecture updates): 1-2 days
- Phase 3 (Instruction decoding): 3-5 days (depends on `msp430-asm` state)
- Phase 4 (LLIL lifting): 3-5 days
- Phase 5 (Testing): 2-3 days
- Phase 6 (Documentation): 1 day

Total: 11-18 days of focused development

This aligns with the "less than one month" estimate mentioned in the issue.

## Success Criteria

1. All MSP430X extended instructions are recognized and disassembled correctly
2. LLIL lifting produces correct semantics for extended instructions
3. 20-bit addressing is handled properly throughout
4. Test binaries can be loaded and analyzed successfully
5. No regressions in base MSP430 support

## Next Steps

1. Investigate `msp430-asm` crate MSP430X support status
2. Compile or obtain 2-3 MSP430X test binaries
3. Set up development environment with test cases
4. Begin implementation based on decoder availability
5. Create tracking issues for individual tasks
