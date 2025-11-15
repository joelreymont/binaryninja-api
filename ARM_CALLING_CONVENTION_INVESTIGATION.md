# ARM/Thumb Calling Convention Issue #6615 - Investigation Report

**Date:** 2025-11-15
**Issue:** Extra arguments added to function in THUMB
**Status:** Cannot fix in architecture plugin - requires core analysis changes

## Problem Description

Binary Ninja incorrectly identifies function parameters when analyzing ARM Thumb code with a specific compiler optimization pattern:

```thumb
push {r0, r1, r2, r3, r4, lr}   ; Push 6 registers
movs r4, #0
str  r4, [sp, #4]                ; Use stack slots as locals
str  r4, [sp, #8]
add  sp, #0x10
pop  {r4, pc}                    ; Only restore 2 registers
```

**Current behavior:** Function incorrectly declared with 4 parameters
**Expected behavior:** Function should have 0 parameters (or actual parameter count)

## Root Cause

The issue is in the **core Binary Ninja analysis framework**, not the architecture plugin:

1. The ARM calling convention is correctly defined (r0-r3 are argument registers)
2. The parameter inference heuristic incorrectly assumes pushed registers are arguments
3. When registers are pushed but not popped, they're stack allocation, not parameters
4. The architecture plugin has no hooks to override this heuristic

## Investigation Results

### Architecture Plugin Code Review

**Calling Convention Definition:** `arch/armv7/arch_armv7.cpp:2146-2177`
```cpp
class ArmCallingConvention: public CallingConvention {
    virtual vector<uint32_t> GetIntegerArgumentRegisters() override {
        return vector<uint32_t>{ REG_R0, REG_R1, REG_R2, REG_R3 };
    }
};
```

This is **correct** per ARM EABI specification.

### What Would Be Needed

The fix requires changes to Binary Ninja core analysis:

1. Heuristic to compare push/pop instructions in function prologue/epilogue
2. If register pushed but not popped -> stack allocation, not parameter
3. Only count registers that are both pushed AND popped as potential parameters

### Available Architecture Plugin Hooks

Searched for parameter-related hooks:
- `GetIncomingRegisterValue` - not overridden in ARM plugin
- `GetIncomingVariableForRegister` - not available
- No hook for "ShouldRegisterBeParameter" or similar

**Conclusion:** Architecture plugin cannot override parameter inference.

## Maintainer Comments

From GitHub issue #6615:
> "this appears to be an analysis problem rather than an instruction lifting issue, requiring investigation from the architecture plugin perspective"

The issue was marked "Awaiting Triage" for the core analysis team.

## Proposed Solution (for Binary Ninja core team)

Add heuristic in function parameter analysis:

```cpp
bool IsActualParameter(uint32_t reg, Function* func) {
    // Check if register is in calling convention argument list
    if (!IsArgumentRegister(reg))
        return false;

    // Check prologue: is register pushed?
    bool isPushed = IsPushedInPrologue(func, reg);

    // Check epilogue: is register popped?
    bool isPopped = IsPoppedInEpilogue(func, reg);

    // Only treat as parameter if pushed AND popped
    // (or if never pushed - direct use)
    return !isPushed || isPopped;
}
```

## Recommendation

**This issue cannot be fixed in the architecture plugin.** It requires changes to Binary Ninja's core analysis framework.

The issue should be:
1. Escalated to the core Binary Ninja analysis team
2. Implemented in the function parameter inference code
3. Tested with the provided test case

## Test Case for Future Fix

The issue reporter should provide a binary exhibiting this behavior for regression testing once the core analysis fix is implemented.

---

**Investigation by:** Joel Reymont
**Conclusion:** Moving to Phase 2 (architecture plugin improvements that CAN be implemented)
