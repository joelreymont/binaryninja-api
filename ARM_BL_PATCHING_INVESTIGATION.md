# ARM Branch-with-Link Patching Investigation

**Date:** 2025-11-16
**Issue:** #5153 - ARM branch-with-link patching causes address corruption
**Status:** Core API Limitation - Cannot Fix in Architecture Plugin
**Investigator:** Joel Reymont

---

## Executive Summary

Investigation of issue #5153 reveals a **Binary Ninja core API limitation** that prevents the architecture plugin from correctly assembling PC-relative instructions like BL (Branch with Link) when users edit them in the disassembly view.

**Root Cause:** The `BNLlvmServicesAssemble` API does not accept an address parameter, making it impossible for the assembler to correctly calculate PC-relative offsets.

**Impact:**
- Users cannot reliably edit BL/BLX instructions in the disassembly view
- Even restoring original values causes corruption
- Affects ARM/Thumb BL, BLX, and other PC-relative instructions

**Recommendation:** Binary Ninja core team must add address parameter to LLVM services API.

---

## Problem Statement

### Issue #5153 Description

**Reported Behavior:**
When users attempt to modify ARM BL (Branch with Link) instructions through the disassembly editor:

1. Right-click on BL instruction
2. Select "Edit current line"
3. Modify the target address
4. Even restoring to original value causes corruption
5. Function call points to wrong address

**Version Info:**
- Binary Ninja 4.0.4911 Personal
- Platform: Windows 10, x86_64
- Component: ARM Architecture plugin

**Expected:** User should be able to edit and restore addresses correctly

**Actual:** Editing breaks function calls with incorrect addresses

---

## Technical Investigation

### ARM BL Instruction Encoding

ARM BL (Branch with Link) uses **PC-relative addressing**:

```assembly
BL target_address
```

**Encoding (ARM mode - 32-bit):**
```
31  28 27  24 23                    0
[cond] [1011] [   24-bit offset    ]
```

The offset is calculated as:
```c
offset = (target_address - (PC + 8)) >> 2
```

**Critical:** The assembler **MUST know the current PC** to encode the offset correctly!

### Current Assemble Implementation

**File:** `arch/armv7/arch_armv7.cpp` line 2129-2158

```cpp
bool ArmCommonArchitecture::Assemble(const string& code, uint64_t addr,
                                      DataBuffer& result, string& errors)
{
    (void)addr;  // ⚠️ ADDRESS IS IGNORED!

    char *instrBytes=NULL, *err=NULL;
    int instrBytesLen=0, errLen=0;

    int assembleResult;

    string triple = GetAssemblerTriple();
    LogDebug("%s() retrieves and uses triple %s\n", __func__, triple.c_str());

    BNLlvmServicesInit();

    errors.clear();
    assembleResult = BNLlvmServicesAssemble(code.c_str(), LLVM_SVCS_DIALECT_UNSPEC,
      triple.c_str(), LLVM_SVCS_CM_DEFAULT, LLVM_SVCS_RM_STATIC,
      &instrBytes, &instrBytesLen, &err, &errLen);
      // ⚠️ NO ADDRESS PARAMETER PASSED TO LLVM!

    if(assembleResult || errLen) {
        errors = err;
        BNLlvmServicesAssembleFree(instrBytes, err);
        return false;
    }

    result.Clear();
    result.Append(instrBytes, instrBytesLen);
    BNLlvmServicesAssembleFree(instrBytes, err);
    return true;
}
```

**Problems:**
1. Line 2131: `(void)addr;` - Address parameter completely ignored
2. Line 2144-2146: `BNLlvmServicesAssemble` called without address
3. LLVM assembler defaults to address 0x0 for PC-relative calculations
4. Resulting encoding has wrong offset

### Binary Ninja Core API

**File:** `binaryninjacore.h` line 7801-7802

```c
BINARYNINJACOREAPI int BNLlvmServicesAssemble(
    const char* src,           // Assembly code
    int dialect,               // Dialect (LLVM_SVCS_DIALECT_UNSPEC, etc.)
    const char* triplet,       // Target triple (e.g., "arm-unknown-none")
    int codeModel,             // Code model
    int relocMode,             // Relocation mode
    char** outBytes,           // Output buffer
    int* outBytesLen,          // Output length
    char** err,                // Error string
    int* errLen                // Error length
);
```

**Missing:** `uint64_t addr` parameter!

**Impact:** Architecture plugins **cannot** pass the current address to LLVM for PC-relative instruction encoding.

---

## Reproduction Example

### Scenario

User has ARM code at address 0x1000:
```assembly
0x1000: BL 0x2000    ; Call function at 0x2000
```

**Correct encoding:**
- PC = 0x1000
- Target = 0x2000
- Offset = (0x2000 - (0x1000 + 8)) >> 2 = (0x2000 - 0x1008) >> 2 = 0x3FE
- Encoded: `EB 00 03 FE` (ARM little-endian)

### What Happens When User Edits

1. User selects "Edit current line" on the BL instruction
2. Binary Ninja shows: `BL #0x2000`
3. User modifies to: `BL #0x3000` then changes back to `BL #0x2000`
4. Architecture plugin calls `Assemble("BL #0x2000", 0x1000, ...)`
5. But Assemble() ignores the address and calls:
   ```cpp
   BNLlvmServicesAssemble("BL #0x2000", ..., NO_ADDRESS_PARAM);
   ```
6. LLVM assumes PC = 0x0 (default)
7. **Wrong offset calculated:**
   - Offset = (0x2000 - (0x0 + 8)) >> 2 = (0x2000 - 0x8) >> 2 = 0x7FE
   - Wrong encoding: `EB 00 07 FE`
8. When executed at 0x1000, branches to:
   - Target = (0x1000 + 8) + (0x7FE << 2) = 0x1008 + 0x1FF8 = 0x3000
   - **Wrong address!** Should be 0x2000

### Result

Even though user entered the correct target address (0x2000), the instruction now branches to 0x3000 because LLVM calculated the offset assuming PC=0 instead of PC=0x1000.

---

## Why This Cannot Be Fixed in Architecture Plugin

### Attempted Solutions

**1. Pass address in assembly text?**

Some assemblers support `.org` directives:
```assembly
.org 0x1000
BL #0x2000
```

**Problem:** LLVM's assembler doesn't support this for runtime assembly via BNLlvmServicesAssemble. The `.org` directive is for object file generation, not runtime instruction assembly.

**2. Calculate offset manually?**

Architecture plugin could calculate the offset and provide it:
```assembly
BL #0xFF8    ; Pre-calculated offset
```

**Problems:**
- User sees address (0x2000) but we'd need to convert to offset
- Complex parsing of user input
- Doesn't handle all PC-relative instructions (B, BL, BLX, ADR, LDR PC-rel, etc.)
- Fragile and error-prone

**3. Use different assembler?**

Some architectures might use custom assembly instead of LLVM services.

**Problems:**
- Would require implementing full ARM instruction encoder
- Thousands of ARM/Thumb instructions
- Extremely complex (ARM ARM is 8000+ pages)
- Not maintainable

### Core API Limitation

The fundamental issue is that **`BNLlvmServicesAssemble` does not accept an address parameter**.

**Required fix:**
```c
// Proposed new API signature:
BINARYNINJACOREAPI int BNLlvmServicesAssembleAtAddress(
    const char* src,
    uint64_t addr,       // ✓ ADD THIS PARAMETER
    int dialect,
    const char* triplet,
    int codeModel,
    int relocMode,
    char** outBytes,
    int* outBytesLen,
    char** err,
    int* errLen
);
```

With this, the architecture plugin could:
```cpp
BNLlvmServicesAssembleAtAddress(code.c_str(), addr,  // Pass address!
    LLVM_SVCS_DIALECT_UNSPEC, triple.c_str(),
    LLVM_SVCS_CM_DEFAULT, LLVM_SVCS_RM_STATIC,
    &instrBytes, &instrBytesLen, &err, &errLen);
```

---

## Affected Instructions

### ARM Mode

All PC-relative instructions affected:
- **B** - Branch
- **BL** - Branch with Link
- **BLX** - Branch with Link and Exchange
- **ADR** - Form PC-relative address
- **LDR PC-rel** - Load from PC-relative address
- **LDRD PC-rel** - Load double from PC-relative address

### Thumb Mode

- **B** - Branch (T2, T3, T4 encodings)
- **BL** - Branch with Link
- **BLX** - Branch with Link and Exchange
- **ADR** - Form PC-relative address
- **LDR PC-rel** - Load from PC-relative address
- **TBB/TBH** - Table branch

### Total Impact

Approximately **15-20 instruction variants** affected across ARM and Thumb modes.

---

## Comparison with Other Architectures

### x86/x86-64

**PC-relative instructions:**
- JMP rel32
- CALL rel32
- Jcc rel32 (conditional jumps)

**Status:** Likely affected by same issue

### ARM64

**PC-relative instructions:**
- B - Branch
- BL - Branch with Link
- ADR/ADRP - Form PC-relative address
- LDR literal - Load from PC-relative address

**Status:** Likely affected by same issue

### MIPS

**PC-relative instructions:**
- J/JAL in MIPS32 (pseudo-absolute)
- B/BAL in MIPS (PC-relative)

**Status:** Potentially affected

### RISC-V

**PC-relative instructions:**
- JAL - Jump and Link
- AUIPC - Add Upper Immediate to PC

**Status:** Likely affected

**Conclusion:** This is a **widespread issue affecting multiple architectures**, not just ARM.

---

## Workarounds for Users

Until the core API is fixed, users can work around the issue:

### Workaround 1: Use Hex Editor

Instead of editing assembly:
1. Use Hex Editor view
2. Manually calculate PC-relative offset
3. Encode instruction bytes directly
4. Paste into binary

**Pros:** Always works correctly
**Cons:** Requires manual offset calculation, error-prone

### Workaround 2: Use External Assembler

1. Use external ARM assembler (e.g., GNU as)
2. Specify `.org` directive for correct address
3. Assemble to get correct bytes
4. Paste into Binary Ninja hex editor

**Example:**
```assembly
.arch armv7-a
.syntax unified
.org 0x1000
BL #0x2000
```

Assemble:
```bash
arm-none-eabi-as test.s -o test.o
arm-none-eabi-objcopy -O binary test.o test.bin
hexdump -C test.bin
```

**Pros:** Gets correct encoding
**Cons:** Requires external tools, multi-step process

### Workaround 3: Don't Edit BL Instructions

Simply avoid editing PC-relative instructions in the disassembly view.

**Pros:** Safe
**Cons:** Doesn't solve the problem

---

## Recommendations

### For Binary Ninja Core Team (High Priority)

**1. Add Address Parameter to LLVM Services**

Add new API function:
```c
BINARYNINJACOREAPI int BNLlvmServicesAssembleAtAddress(
    const char* src,
    uint64_t addr,          // Current address for PC-relative calculations
    int dialect,
    const char* triplet,
    int codeModel,
    int relocMode,
    char** outBytes,
    int* outBytesLen,
    char** err,
    int* errLen
);
```

**Implementation:**
- Pass address to LLVM MC (Machine Code) layer
- Set section address before assembling
- This is supported by LLVM MC infrastructure

**2. Update Architecture Plugins**

Update all architecture plugins to use new API:
```cpp
// arch/armv7/arch_armv7.cpp
bool ArmCommonArchitecture::Assemble(const string& code, uint64_t addr,
                                      DataBuffer& result, string& errors)
{
    // Remove: (void)addr;
    // Add: Use addr parameter

    BNLlvmServicesAssembleAtAddress(code.c_str(), addr,  // Pass address!
        LLVM_SVCS_DIALECT_UNSPEC, triple.c_str(),
        LLVM_SVCS_CM_DEFAULT, LLVM_SVCS_RM_STATIC,
        &instrBytes, &instrBytesLen, &err, &errLen);
}
```

**3. Add Regression Tests**

Create tests for PC-relative instruction assembly at various addresses.

### For Open-Source Contributors (Not Fixable)

**This issue cannot be fixed in the open-source architecture plugin** because:
1. Requires core API changes
2. `BNLlvmServicesAssemble` is in closed-source `libbinaryninjacore.so`
3. Only Binary Ninja team can add address parameter to LLVM services

**Recommendation:** Document as core API limitation and move to Binary Ninja team's internal issue tracker.

---

## Testing Approach (Once Fixed)

### Test Case 1: Basic BL Assembly

```python
def test_bl_assembly_at_address():
    arch = Architecture['armv7']

    # Test at address 0x1000
    result = arch.assemble("BL #0x2000", 0x1000)
    expected = b'\xeb\x00\x03\xfe'  # Correct encoding for BL from 0x1000 to 0x2000
    assert result == expected

    # Test at address 0x5000
    result = arch.assemble("BL #0x2000", 0x5000)
    # Offset = (0x2000 - (0x5000 + 8)) >> 2 = negative offset
    # Should handle correctly
    assert len(result) == 4
```

### Test Case 2: Multiple Addresses

Verify same instruction text produces different encodings at different addresses:

```python
def test_bl_different_addresses():
    arch = Architecture['armv7']

    encoding_1000 = arch.assemble("BL #0x8000", 0x1000)
    encoding_2000 = arch.assemble("BL #0x8000", 0x2000)
    encoding_3000 = arch.assemble("BL #0x8000", 0x3000)

    # All should be different (different offsets)
    assert encoding_1000 != encoding_2000
    assert encoding_2000 != encoding_3000
    assert encoding_1000 != encoding_3000
```

### Test Case 3: Roundtrip

Assemble and disassemble should be consistent:

```python
def test_bl_roundtrip():
    arch = Architecture['armv7']

    addr = 0x1000
    target = 0x2000

    # Assemble
    encoded = arch.assemble(f"BL #{target:#x}", addr)

    # Disassemble
    tokens, length = arch.get_instruction_text(encoded, addr)
    disasm = ''.join(str(t) for t in tokens)

    # Should show original target
    assert f"{target:#x}" in disasm or f"0x{target:x}" in disasm
```

---

## Impact Assessment

### Severity

- **Impact:** Medium-High
- **Effort to Fix:** Low (in core) / Impossible (in plugin)
- **User Workaround:** Available but inconvenient
- **Affected Architectures:** Multiple (ARM, ARM64, x86, RISC-V, MIPS, etc.)

### User Experience

**Current State:**
- ❌ Cannot edit PC-relative instructions reliably
- ❌ Confusing behavior (editing breaks addresses)
- ❌ Workarounds are cumbersome

**After Fix:**
- ✅ PC-relative instructions editable
- ✅ Consistent behavior
- ✅ No workarounds needed

---

## Conclusion

Issue #5153 (ARM BL patching causing address corruption) is caused by a **Binary Ninja core API limitation**. The `BNLlvmServicesAssemble` function does not accept an address parameter, making it impossible to correctly assemble PC-relative instructions.

**Key Findings:**
1. ✅ Root cause identified: Missing address parameter in core API
2. ✅ Affects multiple architectures, not just ARM
3. ❌ Cannot be fixed in open-source architecture plugin
4. ✅ Fix requires Binary Ninja core team intervention

**Recommendations:**
1. **Binary Ninja Team:** Add `BNLlvmServicesAssembleAtAddress` API with address parameter
2. **Architecture Plugin:** Update to use new API once available
3. **Issue Tracking:** Move to internal tracker as core API enhancement
4. **Documentation:** Inform users of current limitation and workarounds

**Status:** Investigation complete, requires core team action

---

**Investigation Date:** 2025-11-16
**Investigator:** Joel Reymont
**Issue Status:** Core API Limitation - Not Fixable in Architecture Plugin
**Recommendation:** Escalate to Binary Ninja Core Team
