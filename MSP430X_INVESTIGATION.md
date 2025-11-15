# MSP430X Extension Support Investigation

**Date:** 2025-11-15
**Issue:** #7620 - MSP430X extension support needed
**Status:** Blocked on external library support

## Problem Description

Binary Ninja's MSP430 architecture plugin lacks support for MSP430X extended instructions and 20-bit addressing.

## MSP430X Key Differences

### Address Space
- **MSP430:** 16-bit addressing, 64KB address space
- **MSP430X:** 20-bit addressing, 1MB address space

### Extension Words
MSP430X instructions require an additional 16-bit extension word that:
- Contains the upper 4 bits of 20-bit addresses
- Precedes the main instruction word
- Extends immediate values to 20 bits

### New Instructions
- **CALLA** - Call subroutine in full 1MB address space
- **RETA** - Return from subroutine in full 1MB address space
- **PUSHM/POPM** - Multiple register push/pop
- **ADDA/SUBA/CMPA** - 20-bit address arithmetic
- Extended versions of existing instructions with .A suffix

### Modified Instructions
- All instructions can use 20-bit operands when preceded by extension word
- CALL/RET limited to 64KB, must use CALLA/RETA for full range

## Current Implementation

**Architecture:** Rust-based using msp430-asm library
**Location:** `/home/user/binaryninja-api/arch/msp430/`

**Key constraints:**
```rust
fn address_size(&self) -> usize {
    2  // Hard-coded 16-bit
}

fn max_instr_len(&self) -> usize {
    6  // Would need 8 for MSP430X with extension word
}
```

**Disassembly:** Uses external crate `msp430-asm = "^0.2"`

## Investigation Results

### msp430-asm Library Support

**Current version:** 0.2.x
**MSP430X support:** No evidence found

Searched for MSP430X indicators:
- No references to "msp430x", "MSP430X", "CALLA", "RETA"
- No extension word handling
- No 20-bit addressing mode support

### Implementation Blockers

1. **External Dependency:** msp430-asm library does not support MSP430X
2. **Library Modification Required:** Would need to fork/extend msp430-asm or write custom decoder
3. **Architecture Changes:** Address size, instruction length, register handling
4. **Complexity:** Medium estimate (8-12 hours) is insufficient

### What Would Be Required

**Phase 1: Disassembler Support**
- Fork msp430-asm or implement custom decoder
- Add extension word parsing
- Add MSP430X instruction table
- Support 20-bit operand encoding
- Estimated effort: 20-30 hours

**Phase 2: Architecture Plugin Updates**
- Update address_size() to handle 20-bit
- Increase max_instr_len to 8
- Add MSP430X-specific registers (if any)
- Update instruction_info for new branch types
- Estimated effort: 4-6 hours

**Phase 3: IL Generation**
- Add lifting for CALLA/RETA/PUSHM/POPM
- Handle 20-bit address operations
- Test with MSP430X binaries
- Estimated effort: 8-12 hours

**Total estimated effort:** 32-48 hours (significantly more than initial estimate)

## Recommendations

### Option 1: External Library Extension
Contact msp430-asm maintainer to request MSP430X support or submit PR to that project first.

**Pros:** Maintains consistent architecture, community benefit
**Cons:** Depends on external maintainer acceptance, timeline uncertain

### Option 2: Custom Decoder
Implement MSP430X decoder directly in Binary Ninja plugin without msp430-asm dependency.

**Pros:** Full control, can implement exactly what's needed
**Cons:** More maintenance burden, duplicates effort

### Option 3: Defer Implementation
Document requirements and defer until:
1. msp430-asm gains MSP430X support, OR
2. User provides test binaries and use cases to justify effort

**Pros:** Focuses effort on more tractable improvements
**Cons:** Issue remains unresolved

## Decision

**Deferring MSP430X implementation** due to:
1. External library dependency blocker
2. Actual effort (32-48 hours) significantly exceeds estimate (8-12 hours)
3. No test binaries available to validate implementation
4. No immediate user demand beyond initial issue

## Next Steps

1. Comment on GitHub issue #7620 explaining blocker
2. Request community input:
   - Test binaries for validation
   - MSP430X use cases
   - msp430-asm library support timeline
3. Move to other Phase 2 tasks (PowerPC-VLE, nanoMIPS)

---

**Investigation by:** Joel Reymont
**Conclusion:** Blocked on msp430-asm library MSP430X support
