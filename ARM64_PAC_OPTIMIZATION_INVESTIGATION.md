# ARM64 Pointer Authentication Optimization Investigation

**Date:** 2025-11-15
**Issue:** #6702 - Hide explicit pointer authentication checks before tail calls
**Status:** Deferred - requires complex pattern matching

## Problem Description

ARM64 macOS binaries with Pointer Authentication Code (PAC) enabled include explicit validation checks before tail calls. These validation patterns clutter the HLIL decompilation output with incomplete conditionals that obscure control flow.

## Current Behavior

**Example Assembly Pattern:**
```
eor x16, x30, x30, lsl #0x1    ; XOR lr with lr shifted left by 1
tbz x16, #0x3e, <target>       ; Test bit 62, branch if zero
; ... tail call follows
```

**Current HLIL Output:**
```
if (((x30 ^ x30 << 1) & 0x40000000) == 0)
; ... tail call
```

This validation checks that the high bits of the pointer are consistent (valid PAC signature), but adds noise to decompilation.

## Technical Analysis

### Validation Pattern: HighBitsNoTBI

From LLVM's `AArch64PointerAuth.h`, the `HighBitsNoTBI` pattern:
1. XORs the pointer with itself shifted left by 1
2. Tests bit 62 to verify consistency
3. Branches if the check passes

**Purpose:** Validates pointer authentication without stripping the PAC

**Variants:** Multiple validation patterns exist across LLVM versions

### Current IL Lifting

**Location:** `arch/arm64/il.cpp`

- **EOR** (line 1806): Lifted as standard XOR operation
- **TBZ** (line 3887): Lifted as conditional branch

No special handling for PAC validation patterns exists.

## Implementation Challenges

### Pattern Detection

Would require:
1. **Lookahead analysis** - detecting multi-instruction patterns during IL lifting
2. **Instruction context** - tracking that EOR output feeds into TBZ
3. **Pattern matching** - recognizing specific register and immediate combinations

### IL Suppression

Would require:
1. **Conditional IL generation** - suppressing IL for matched patterns
2. **Control flow preservation** - ensuring branch targets remain valid
3. **Analysis correctness** - not breaking other analysis passes

### Multiple Patterns

LLVM has evolved PAC validation over time, requiring:
- Pattern database maintenance
- Version-specific detection
- Fallback handling for unknown patterns

## Existing Architecture

### PAC Instruction Lifting

**Location:** `arch/arm64/il.cpp:1248-1280`

PAC instructions (AUTIA, AUTDA, PACIA, PACDA, etc.) are already lifted as intrinsics:
```cpp
case ARM64_AUTIA:
case ARM64_AUTIA1716:
case ARM64_AUTIASP:
    return ARM64_INTRIN_AUTIA;
```

**Note:** PAC instructions themselves are preserved, issue only affects validation patterns.

## Recommendation

**Defer implementation** due to:

1. **Complexity:** Multi-instruction pattern matching requires significant architecture changes
2. **Scope:** Issue marked "Medium" effort but requires extensive testing
3. **Maintenance:** Pattern database needs ongoing updates for LLVM changes
4. **Core dependency:** Likely better handled in core analysis rather than architecture plugin

## Alternative Approaches

### Option 1: Core Analysis Pass

Implement pattern suppression as a separate analysis pass after IL lifting:
- **Pros:** Cleaner separation, easier to maintain
- **Cons:** Requires core Binary Ninja changes

### Option 2: HLIL Simplification

Add HLIL optimization pass to recognize and remove PAC validation idioms:
- **Pros:** Works at higher IL level where patterns are clearer
- **Cons:** Still requires pattern matching logic

### Option 3: User Documentation

Document the validation pattern and recommend manual analysis:
- **Pros:** Zero implementation effort
- **Cons:** Doesn't solve user problem

## Next Steps

1. **Escalate to core team:** Determine if core analysis pass is feasible
2. **Pattern database:** Collect all known PAC validation patterns from LLVM
3. **Prototype:** Test pattern detection in standalone tool before integration
4. **Benchmark:** Measure impact on analysis performance

## Decision

**Deferred pending core team input** on whether this should be:
- Architecture plugin responsibility (complex pattern matching in IL lifting)
- Core analysis responsibility (separate optimization pass)
- HLIL optimization responsibility (higher-level simplification)

---

**Investigation by:** Joel Reymont
**Conclusion:** Requires complex multi-instruction pattern matching and IL suppression, better suited for core analysis pass than architecture plugin
