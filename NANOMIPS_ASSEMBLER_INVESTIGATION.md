# nanoMIPS Assembler Support Investigation

**Date:** 2025-11-15
**Issue:** #6972 - nanoMIPS assembler functionality missing
**Status:** Technically feasible via LLVM, implementation straightforward

## Problem Description

nanoMIPS architecture lacks assembler support in Binary Ninja, preventing users from patching binaries with nanoMIPS instructions.

**Current status:**
- Disassembly: ✓ Working
- Lifting: ✓ Working
- Assembler: ✗ Missing

## Investigation Results

### MIPS Assembler Pattern

**Location:** `arch/mips/arch_mips.cpp:464-494`

MIPS uses LLVM services for assembly via `BNLlvmServicesAssemble()`:

```cpp
bool Assemble(const string& code, uint64_t addr, DataBuffer& result, string& errors) {
    BNLlvmServicesInit();

    const char* triple = "mips-pc-none-o32";      // Big-endian
    if (m_endian == LittleEndian)
        triple = "mipsel-pc-none-o32";             // Little-endian

    assembleResult = BNLlvmServicesAssemble(code.c_str(), LLVM_SVCS_DIALECT_UNSPEC,
        triple, LLVM_SVCS_CM_DEFAULT, LLVM_SVCS_RM_STATIC,
        &instrBytes, &instrBytesLen, &err, &errLen);

    // Handle result and errors
}
```

**Key components:**
- LLVM target triple specifies architecture
- Dialect, code model, relocation model parameters
- Returns assembled bytes or error messages

### nanoMIPS in LLVM

**Research findings:**

1. **LLVM Support Confirmed**
   - IEEE publications document nanoMIPS support in LLVM
   - "Adding support for integrated nanoMIPS assembler to LLVM" (2022)
   - "Implementation of Machine Outliner for nanoMIPS in LLVM" (2022)
   - Support includes variable-length instruction handling

2. **Target Triple Unknown**
   - Standard MIPS uses: `mips*-vendor-os-abi`
   - nanoMIPS likely uses: `nanomips-*` or `mipsnano-*` (unconfirmed)
   - Need to verify with LLVM documentation or testing

3. **Binary Ninja LLVM Version**
   - Unclear if Binary Ninja's LLVM services include nanoMIPS support
   - May depend on LLVM version embedded in Binary Ninja
   - Could require Binary Ninja update to newer LLVM

## Implementation Approach

### Option 1: LLVM Services (Recommended)

**Effort:** Low (4-8 hours)

**Implementation:**

```cpp
bool Assemble(const string& code, uint64_t addr, DataBuffer& result, string& errors) override {
    BNLlvmServicesInit();

    // TODO: Verify correct nanoMIPS triple
    const char* triple = "nanomips-pc-none";  // Placeholder, needs verification
    if (m_endian == LittleEndian)
        triple = "nanomipsel-pc-none";

    assembleResult = BNLlvmServicesAssemble(code.c_str(), LLVM_SVCS_DIALECT_UNSPEC,
        triple, LLVM_SVCS_CM_DEFAULT, LLVM_SVCS_RM_STATIC,
        &instrBytes, &instrBytesLen, &err, &errLen);

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

**Steps:**
1. Determine correct nanoMIPS LLVM target triple
2. Add `CanAssemble()` returning `true` to nanoMIPS architecture
3. Add `Assemble()` method following MIPS pattern
4. Test with nanoMIPS assembly code
5. Handle any LLVM version compatibility issues

**Risks:**
- Binary Ninja's LLVM may not include nanoMIPS target
- Incorrect triple will cause assembly failures
- LLVM error messages may be cryptic

### Option 2: GNU Binutils Integration

**Effort:** High (20-30 hours)

**Implementation:**
- Integrate MediaTek nanomips-gnu-toolchain
- Create subprocess wrapper around `as` assembler
- Parse assembler output and errors
- Handle temporary files and cleanup

**Pros:** Guaranteed nanoMIPS support
**Cons:** External dependency, platform-specific, slower performance

### Option 3: Custom Assembler

**Effort:** Very High (40-60 hours)

**Implementation:**
- Implement nanoMIPS instruction encoding from scratch
- Handle variable-length instruction encoding
- Support all addressing modes
- Comprehensive error handling

**Pros:** Full control, no dependencies
**Cons:** Massive effort, maintenance burden

## Recommended Solution

**Implement Option 1 (LLVM Services)** because:
1. Follows established pattern (MIPS, PowerPC already use this)
2. Minimal code changes required
3. Leverages existing infrastructure
4. Low maintenance burden

**Prerequisites:**
1. Determine nanoMIPS LLVM triple (can test various formats)
2. Verify Binary Ninja's LLVM version supports nanoMIPS
3. If not, request Binary Ninja team update embedded LLVM

## Testing Strategy

```cpp
// Test cases for nanoMIPS assembler
const char* tests[] = {
    "li $t0, 42",           // Load immediate
    "addiu $t1, $t0, 10",   // Add unsigned immediate
    "sw $t1, 0($sp)",       // Store word
    "lw $t2, 0($sp)",       // Load word
    "b 0x1000",             // Branch
    "nop",                  // No operation
};

for (auto& test : tests) {
    DataBuffer result;
    string errors;
    bool success = Assemble(test, 0x1000, result, errors);
    // Verify result matches expected encoding
}
```

## Effort Estimate

**Option 1 (LLVM):**
- Research triple format: 1-2 hours
- Implementation: 2-3 hours
- Testing: 1-2 hours
- Documentation: 1 hour
- **Total: 5-8 hours**

**Contingencies:**
- If LLVM doesn't support nanoMIPS: Coordinate with Binary Ninja team (unknown timeline)
- If triple format non-standard: May need LLVM source code review (2-4 hours)

## Next Steps

1. **Determine nanoMIPS LLVM triple**
   - Check LLVM source code
   - Test various triple formats
   - Contact LLVM/Binary Ninja communities if needed

2. **Verify Binary Ninja LLVM support**
   - Test BNLlvmServicesAssemble with nanoMIPS triple
   - If fails, contact Binary Ninja team

3. **Implement if feasible**
   - Add CanAssemble() override returning true
   - Add Assemble() method with nanoMIPS triple
   - Test and validate

4. **Document if blocked**
   - If Binary Ninja LLVM lacks nanoMIPS, file enhancement request
   - Provide fallback options (binutils integration)

## Decision

**Defer implementation pending triple verification** because:
- Cannot implement without knowing correct LLVM target triple
- Testing required to verify Binary Ninja's LLVM supports nanoMIPS
- Estimated 5-8 hours but uncertainty on prerequisites

**Recommended approach:**
1. Research LLVM nanoMIPS triple format (may require LLVM source access)
2. Test with Binary Ninja's LLVM services
3. If successful, implement following MIPS pattern
4. If blocked, request Binary Ninja team assistance

---

**Investigation by:** Joel Reymont
**Conclusion:** Implementation straightforward via LLVM if prerequisites verified
