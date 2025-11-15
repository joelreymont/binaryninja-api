# PowerPC-VLE SPE Instruction Lifting Investigation

**Date:** 2025-11-15
**Issue:** #7218 - Integrate lifting of PowerPC-VLE SPE instructions
**Status:** Decoder complete, IL lifting needed

## Problem Description

PowerPC Signal Processing Engine (SPE) instructions can be disassembled but are not lifted to IL, resulting in `Unimplemented` IL nodes that break decompilation.

## Current Status

### Disassembly Support: Complete
**Location:** `arch/powerpc/decode/`

**SPE Instructions Defined:** 253 instructions including:
- EFD* (Embedded Floating-Point Double) - 29 instructions
- EFS* (Embedded Floating-Point Single) - 21 instructions
- EV* (Embedded Vector) - 203+ instructions
  - EVLDD, EVLDDX (loads)
  - EVSTDD, EVSTDDX (stores)
  - EVMHOSSF, EVMHOSSFA (multiply-accumulate)
  - EVADD, EVSUB (arithmetic)
  - Many more vector/SIMD operations

All instructions have:
- Instruction ID enum (`PPC_ID_SPE_*`)
- Mnemonic strings
- Operand parsing
- Disassembly support

### IL Lifting Support: Missing
**Location:** `arch/powerpc/il.cpp`

**Current count:** 0 SPE instruction cases

When SPE instructions are encountered:
```cpp
case PPC_OP_NONE:
default:
    MYLOG("ERROR: don't know how to convert operand to IL\n");
    res = il.Unimplemented();  // ← SPE instructions hit this
```

## Reference Implementation

**External Plugin:** Martyx00/PowerPC-VLE-Extension
- Repository: https://github.com/Martyx00/PowerPC-VLE-Extension
- Written in C++ for Binary Ninja
- Implements SPE lifting
- Uses custom libvle submodule

**Issue:** Plugin source code structure unclear from web fetch, need direct repository access to extract lifting logic.

## What Needs To Be Done

### Phase 1: Extract Reference Implementation (2-4 hours)
1. Clone Martyx00/PowerPC-VLE-Extension repository
2. Locate SPE instruction lifting code (likely in vle_ext.cpp)
3. Identify IL generation patterns for SPE instruction classes
4. Document register usage and operand patterns

### Phase 2: Integrate SPE Lifting (8-16 hours)

**For each SPE instruction class:**

1. **Embedded Floating-Point (EFS/EFD)**
   - Add, subtract, multiply, divide operations
   - Conversions (integer/float, single/double)
   - Comparisons and tests
   - Absolute value, negate
   - Estimated: 50 instructions, 6-8 hours

2. **Embedded Vector Loads/Stores (EVLD*/EVST*)**
   - Double-word loads and stores
   - Indexed addressing modes
   - Half-word loads with extension
   - Estimated: 30 instructions, 2-3 hours

3. **Embedded Vector Arithmetic (EVAD*/EVSUB*/etc)**
   - Vector add, subtract
   - Multiply-accumulate operations
   - Shift and rotate operations
   - Estimated: 50 instructions, 4-5 hours

4. **Embedded Vector Multiply/MAC (EVMH*)**
   - Complex multiply-accumulate variants
   - Signed/unsigned operations
   - Accumulator management
   - Estimated: 80 instructions, 5-6 hours

5. **Miscellaneous (BRINC, merge ops, etc)**
   - Bit reverse increment
   - Merge operations
   - Select operations
   - Estimated: 43 instructions, 3-4 hours

**Total estimated:** 253 instructions, 20-26 hours of implementation

### Phase 3: Testing (4-6 hours)
1. Create test cases for each instruction class
2. Validate IL generation correctness
3. Test with real PowerPC VLE binaries
4. Verify decompilation output

### Total Estimated Effort: 32-46 hours

## Complexity Analysis

**Why This Is Large:**

1. **Volume:** 253 instructions to implement
2. **Specialization:** SPE instructions have unique semantics
   - Vector operations on 64-bit register pairs
   - Accumulator registers
   - Saturation arithmetic
   - Special rounding modes

3. **Documentation:** SPE architecture reference required
   - Freescale/NXP documentation
   - Instruction encoding details
   - Flag behavior for each instruction

4. **Testing:** Need comprehensive validation
   - PowerPC VLE binaries are uncommon
   - Each instruction needs test coverage
   - IL correctness verification challenging

## Blocking Issues

1. **Reference Implementation Access**
   - Cannot directly view Martyx00's SPE lifting code via WebFetch
   - Need to clone repository and examine source

2. **SPE Specification**
   - Need official Freescale/NXP SPE programming reference
   - Some instructions have complex semantics

3. **Test Binaries**
   - Need PowerPC VLE binaries with SPE instructions
   - Validation requires known-good output

## Recommendation

Given the scope (32-46 hours), this exceeds the "Low Effort" (<1 week) estimate in issue #7218.

**Options:**

**Option 1: Full Implementation (32-46 hours)**
- Clone Martyx00's plugin
- Port all 253 SPE instruction liftings
- Comprehensive testing
- Pros: Complete solution
- Cons: Exceeds time budget significantly

**Option 2: Partial Implementation (8-12 hours)**
- Implement most common SPE instructions first:
  - Floating-point operations (EFS*/EFD*)
  - Basic loads/stores (EVLDD/EVSTDD)
  - Simple arithmetic (EVADD/EVSUB)
- Leave complex vector multiply-accumulate for later
- Pros: Provides value sooner
- Cons: Incomplete

**Option 3: Defer Pending Resources (0 hours now)**
- Request Martyx00's cooperation to contribute code
- Wait for test binaries from enterprise customer
- Document requirements for future implementation
- Pros: Ensures quality, avoids duplication
- Cons: No immediate progress

## Decision

**Deferring full implementation** due to:
1. Effort (32-46 hours) significantly exceeds estimate (<1 week)
2. Need direct access to reference implementation
3. Lack of test binaries for validation
4. Issue marked "Future" milestone (not urgent)

**Recommended next step:** Contact Martyx00 to request code contribution or detailed implementation guide.

---

**Investigation by:** Joel Reymont
**Conclusion:** Decoder complete, IL lifting deferred pending reference implementation access and test binaries
