# msp430-asm Crate Analysis

## Investigation Results

Date: 2025-11-15
Crate Repository: https://github.com/jrozner/msp430-asm

### Current State

**Version**: 0.3.0 (latest on GitHub)
- Binary Ninja's MSP430 module currently uses v0.2
- Upgrade to v0.3.0 may be beneficial but won't add MSP430X support

**License**: MIT

**Edition**: 2024

### Supported Instructions

The crate currently supports only the base MSP430 instruction set (27 core instructions + emulated):

#### Single Operand Instructions
- RRC - Rotate right through carry
- SWPB - Swap bytes
- RRA - Rotate right arithmetic
- SXT - Sign extend
- PUSH - Push value onto stack
- CALL - Call subroutine
- RETI - Return from interrupt

#### Jump Instructions (Jxx)
- JNZ - Jump if not zero
- JZ - Jump if zero
- JLO - Jump if lower (unsigned)
- JC - Jump if carry
- JN - Jump if negative
- JGE - Jump if greater or equal (signed)
- JL - Jump if less than (signed)
- JMP - Jump unconditionally

#### Two Operand Instructions
- MOV - Move
- ADD - Add
- ADDC - Add with carry
- SUBC - Subtract with carry
- SUB - Subtract
- CMP - Compare
- DADD - Decimal add
- BIT - Bit test
- BIC - Bit clear
- BIS - Bit set
- XOR - Exclusive OR
- AND - Logical AND

#### Emulated Instructions
The crate also recognizes emulated instructions (combinations of base instructions that are commonly used):
- ADC, BR, CLR, CLRC, CLRN, CLRZ, DADC, DEC, DECD, DINT, EINT, INC, INCD, INV, NOP, POP, RET, RLA, RLC, SBC, SETC, SETN, SETZ, TST

### Missing MSP430X Support

The crate does NOT support any MSP430X extended instructions:

#### Missing Address Instructions
- MOVA - Move address (20-bit)
- CMPA - Compare address
- ADDA - Add to address
- SUBA - Subtract from address

#### Missing Control Flow Instructions
- CALLA - Call subroutine (20-bit)
- BRA - Branch (20-bit)
- RETA - Return from subroutine (20-bit)

#### Missing Rotate/Shift Instructions
- RRCM - Rotate right through carry, multiple bits
- RRAM - Rotate right arithmetic, multiple bits
- RLAM - Rotate left arithmetic, multiple bits
- RRUM - Rotate right unsigned, multiple bits

#### Missing Stack Instructions
- PUSHM.A - Push multiple registers (address mode)
- PUSHM.W - Push multiple registers (word mode)
- POPM.A - Pop multiple registers (address mode)
- POPM.W - Pop multiple registers (word mode)

### Decoder Architecture

The decoder in `src/lib.rs` uses a pattern-matching approach:

1. Reads first 16-bit word
2. Checks instruction type bits (INST_TYPE_MASK: 0b1110_0000_0000_0000)
3. Routes to appropriate decoder:
   - SINGLE_OPERAND_INSTRUCTION (0b0000): Single operand instructions
   - JMP_INSTRUCTION (0b0010): Jump instructions
   - Default: Two operand instructions

**No handling for MSP430X instruction encoding**:
- No extension word parsing
- No support for 0xxx range extended instructions
- No support for 13xx/14xx range (CALLA, PUSHM/POPM)

### Address Size Limitation

Current operand types (from examining the instruction definitions):
- All addresses are 16-bit (i16 for offsets, u16 for absolute)
- No support for 20-bit addresses or operands
- Extension word mechanism not implemented

### Implementation Gap Summary

To add MSP430X support, the crate needs:

1. **Extension Word Support**
   - Parse extension word (18xx encoding prefix)
   - Extract 20-bit source and destination addresses
   - Handle extended immediate values

2. **New Instruction Variants**
   - Add MOVA, CMPA, ADDA, SUBA (0xxx range)
   - Add CALLA (13xx range)
   - Add PUSHM/POPM (14xx range)
   - Add RRCM, RRAM, RLAM, RRUM (0xxx range)
   - Add BRA, RETA

3. **Operand Type Extensions**
   - Support 20-bit addresses (u32 instead of u16 for some operands)
   - Add .A (address/20-bit) and .W (word/16-bit) suffix support
   - Handle multi-register count for PUSHM/POPM

4. **Decoder Logic Updates**
   - Detect and parse extension words
   - Add new opcode ranges (0xxx for most extended instructions)
   - Handle variable instruction lengths (up to 3 words)

### File Structure

```
msp430-asm/
├── src/
│   ├── lib.rs              # Main decoder logic
│   ├── instruction.rs      # Instruction enum
│   ├── single_operand.rs   # Single operand instruction types
│   ├── two_operand.rs      # Two operand instruction types
│   ├── jxx.rs              # Jump instruction types
│   ├── emulate.rs          # Emulated instruction trait/logic
│   ├── operand.rs          # Operand types and parsing
│   └── decode_error.rs     # Error types
├── Cargo.toml
└── README.md
```

### Recommendations

#### Option 1: Fork and Extend (Recommended)
Fork the msp430-asm crate and add MSP430X support:

**Advantages**:
- Clean separation of concerns
- Can submit PR upstream to benefit community
- Maintains existing MSP430 functionality
- Well-structured codebase to build upon

**Effort**: Moderate (3-5 days)

**Steps**:
1. Fork jrozner/msp430-asm
2. Add extension word parsing to operand.rs
3. Add new instruction types for MSP430X instructions
4. Update decoder in lib.rs to handle new opcode ranges
5. Add tests for all new instructions
6. Update Binary Ninja's Cargo.toml to use forked version

#### Option 2: Separate MSP430X Decoder
Create a new msp430x-asm crate:

**Advantages**:
- Full control over implementation
- Can optimize specifically for MSP430X
- No concerns about breaking base MSP430 compatibility

**Disadvantages**:
- Code duplication
- Need to maintain both decoders if supporting both architectures

**Effort**: Higher (5-7 days)

#### Option 3: Implement Decoder Inline
Implement MSP430X decoding directly in Binary Ninja's architecture.rs:

**Advantages**:
- No external dependency changes
- Complete control

**Disadvantages**:
- Significant code to add to architecture module
- Harder to test in isolation
- Mixing decoding and architecture logic

**Effort**: Moderate (4-6 days)

### Decision

**Recommended approach**: Option 1 (Fork and Extend)

Rationale:
- msp430-asm has a clean, well-tested structure
- Fork allows contributing back to community
- Separation of decoding logic from Binary Ninja architecture logic is good design
- The codebase is straightforward to extend
- MIT license is compatible with Apache-2.0 used by Binary Ninja module

### Next Steps

1. Fork https://github.com/jrozner/msp430-asm to our namespace
2. Create a branch for MSP430X support
3. Implement extension word parsing
4. Add MSP430X instructions incrementally (starting with most common: MOVA, CALLA)
5. Add comprehensive tests
6. Update Binary Ninja arch/msp430 to use forked version
7. Test with real MSP430X binaries
8. Submit PR to upstream if maintainer is receptive

### Test Coverage

The existing crate has excellent test coverage in lib.rs with unit tests for:
- Each instruction variant
- Different addressing modes
- Byte vs word operations
- Edge cases (negative offsets, constant generation, etc.)

This same testing approach should be followed for MSP430X instructions.
