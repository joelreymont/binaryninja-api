# TriCore and nanoMIPS Architecture Issues - Blockers

**Date:** 2025-11-16
**Investigator:** Joel Reymont
**Status:** BLOCKED - Cannot implement due to source code access limitations

---

## Executive Summary

Two architecture improvement requests cannot be implemented in the open-source Binary Ninja API repository:
- **TriCore** (#7131, #6618) - Source code not accessible (Ultimate Only)
- **nanoMIPS** (#6972) - Source code not accessible + LLVM dependency missing

Both architectures are **"Ultimate Only"** features with proprietary/closed-source implementations not available in the `binaryninja-api` open-source repository.

---

## TriCore Architecture Issues

### Issue #7131 - Global Register Configuration
### Issue #6618 - Architecture Hook Registration

**Status:** CANNOT FIX - Source code not accessible

### Investigation Findings

1. **Ultimate Only Architecture**
   ```
   From: docs/guide/settings.md:266
   |corePlugins.architectures|TriCore Architecture|
   Enable the built-in TriCore architecture module. (Ultimate Only)|
   ```

2. **Source Code Location**
   - ✗ Not in `/arch/tricore/` (directory does not exist)
   - ✗ Not in open-source repository
   - ✓ Capstone includes TriCore support (in PowerPC's capstone submodule)
   - ✓ But Binary Ninja's TriCore architecture is proprietary

3. **Architecture Directory Listing**
   ```bash
   $ ls /home/user/binaryninja-api/arch/
   arm64  armv7  mips  msp430  powerpc  riscv  x86
   ```

   **TriCore is missing** - confirms it's closed-source.

### Why These Issues Cannot Be Fixed

**#7131 - Global Register Configuration**
- Requires modifying TriCore architecture class
- Source code in closed-source Binary Ninja Ultimate
- Cannot access or modify without license

**#6618 - Architecture Hook Registration**
- Requires modifying TriCore plugin initialization
- Source code not in open-source repository
- Would need Binary Ninja internal development access

### Recommendations

1. **File internal ticket** with Binary Ninja development team
2. **Provide issue details** from GitHub #7131 and #6618
3. **Request priority** if TriCore support is business-critical
4. **Alternative:** Contact Binary Ninja support for paid development

---

## nanoMIPS Assembler Issue

### Issue #6972 - nanoMIPS Assembler Functionality Missing

**Status:** BLOCKED - No mainline LLVM support

### Investigation Findings

#### 1. nanoMIPS is Ultimate Only

```
From: docs/guide/settings.md:263
|corePlugins.architectures|nanoMIPS Architecture|
Enable the built-in nanoMIPS architecture module. (Ultimate Only)|
```

- Source code not in `/arch/nanomips/` (directory does not exist)
- Proprietary/closed-source implementation
- Cannot access architecture class to add assembler support

#### 2. LLVM Mainline Lacks nanoMIPS Support

**Verified:** LLVM mainline repository (as of 2025-11-16)

Source: https://github.com/llvm/llvm-project/blob/main/llvm/include/llvm/TargetParser/Triple.h

**MIPS Architectures in LLVM ArchType enum:**
```cpp
mips,      // MIPS: mips, mipsallegrex, mipsr6
mipsel,    // MIPSEL: mipsel, mipsallegrexe, mipsr6el
mips64,    // MIPS64: mips64, mips64r6, mipsn32, mipsn32r6
mips64el,  // MIPS64EL: mips64el, mips64r6el, mipsn32el, mipsn32r6el
```

**Result:** ✗ No `nanomips` or `nanomipsel` architecture type exists

#### 3. MediaTek's Out-of-Tree Implementation

**Findings:**
- MediaTek maintains custom LLVM patches for nanoMIPS
- nanoMIPS support is **not upstreamed** to LLVM mainline
- Latest release: nanoMIPS-2024.11-02 (January 2025)
- Uses custom toolchain prefix: `nanomips-elf`

**Impact:**
- Binary Ninja's embedded LLVM services do not support nanoMIPS
- `BNLlvmServicesAssemble()` with nanoMIPS triple would fail
- Cannot implement assembler via LLVM (the recommended approach)

#### 4. Previous Investigation Summary

From `NANOMIPS_ASSEMBLER_INVESTIGATION.md`:

**Proposed Solutions:**
1. ✗ **LLVM Services** - Blocked (nanoMIPS not in mainline LLVM)
2. ✗ **GNU Binutils** - Requires external dependency, 20-30 hours
3. ✗ **Custom Assembler** - Very high effort (40-60 hours), custom encoding

**All solutions blocked:**
- Option 1: Requires LLVM mainline support (does not exist)
- Option 2: Requires access to nanoMIPS architecture class (closed-source)
- Option 3: Requires access to nanoMIPS architecture class (closed-source)

### Technical Details

**Why Standard MIPS Approach Fails:**

```cpp
// From arch/mips/arch_mips.cpp - This pattern CANNOT be used for nanoMIPS
bool Assemble(const string& code, uint64_t addr, DataBuffer& result, string& errors) {
    BNLlvmServicesInit();

    const char* triple = "mips-pc-none-o32";      // Standard MIPS
    // const char* triple = "nanomips-pc-none";    // NOT in LLVM mainline!

    assembleResult = BNLlvmServicesAssemble(code.c_str(), LLVM_SVCS_DIALECT_UNSPEC,
        triple, LLVM_SVCS_CM_DEFAULT, LLVM_SVCS_RM_STATIC,
        &instrBytes, &instrBytesLen, &err, &errLen);
    // This would FAIL because LLVM doesn't recognize "nanomips" triple
}
```

**Error that would occur:**
```
error: unknown target triple 'nanomips-pc-none'
```

### Root Cause Analysis

**Two-fold blocker:**

1. **Architectural Blocker**
   - nanoMIPS plugin source code is closed-source (Ultimate Only)
   - Cannot add `Assemble()` method to nanoMIPS architecture class
   - Cannot override `CanAssemble()` to return `true`

2. **LLVM Blocker**
   - Even if source code were accessible, LLVM doesn't support nanoMIPS
   - Binary Ninja's embedded LLVM lacks nanoMIPS target backend
   - Would require Binary Ninja to:
     - Update embedded LLVM to include MediaTek's patches, OR
     - Implement custom assembler in closed-source plugin

### Recommendations

#### For Binary Ninja Development Team

1. **Upstream LLVM Support**
   - Work with MediaTek to upstream nanoMIPS to LLVM mainline
   - OR bundle MediaTek's custom LLVM patches in Binary Ninja
   - Update embedded LLVM services to include nanoMIPS target

2. **Custom Assembler Implementation**
   - Implement nanoMIPS assembler in closed-source plugin
   - Use MediaTek's nanomips-gnu-toolchain as reference
   - Handle variable-length instruction encoding

3. **External Tool Integration**
   - Integrate with MediaTek's assembler via subprocess
   - Provide configuration option for toolchain path
   - Similar to how external disassemblers can be integrated

#### For Users (Workaround)

1. **Use MediaTek Toolchain Directly**
   ```bash
   # Install MediaTek nanoMIPS toolchain
   wget https://github.com/MediaTek-Labs/nanomips-gnu-toolchain/releases/...

   # Assemble manually
   nanomips-elf-as input.s -o output.o
   nanomips-elf-objcopy -O binary output.o output.bin

   # Patch binary in Binary Ninja
   ```

2. **Online Assemblers**
   - Use online MIPS/nanoMIPS assemblers
   - Copy assembled bytes into Binary Ninja
   - Manual but functional for small patches

---

## Impact Assessment

### Issues Affected

| Issue | Title | Architecture | Blocker Type | Can Fix? |
|-------|-------|--------------|--------------|----------|
| #7131 | TriCore global register configuration | TriCore | Source Access | ✗ No |
| #6618 | TriCore architecture hook registration | TriCore | Source Access | ✗ No |
| #6972 | nanoMIPS assembler functionality | nanoMIPS | Source Access + LLVM | ✗ No |

### Scope of Limitation

**Ultimate Only Architectures (Confirmed):**
- ✗ TriCore
- ✗ nanoMIPS

**Open Source Architectures (Can Modify):**
- ✓ ARM64 (AArch64)
- ✓ ARMv7 (ARM/Thumb)
- ✓ MIPS/MIPS64
- ✓ MSP430
- ✓ PowerPC
- ✓ RISC-V
- ✓ x86/x86-64

### Business Impact

**For Ultimate License Holders:**
- Cannot benefit from open-source community fixes for TriCore/nanoMIPS
- Must rely on Binary Ninja internal development for these architectures
- Feature requests have longer implementation timeline

**For Community Contributors:**
- Cannot contribute improvements to Ultimate Only architectures
- Open-source contribution limited to 7 architectures
- Reduces community involvement in specialized architectures

---

## Alternative Issues Available

Since TriCore and nanoMIPS are blocked, here are **open-source architecture issues** that CAN be fixed:

### ARM64 (AArch64) - `/arch/arm64/`

- **#6037** - System register write IL lifting
  - Effort: Medium (8-12 hours)
  - Impact: Better kernel/hypervisor code analysis

### ARMv7 (ARM/Thumb) - `/arch/armv7/`

- **#5527** - IT (If-Then) conditional block lifting
  - Effort: Medium (12-16 hours)
  - Impact: Better Thumb decompilation

- **#5153** - Branch-with-link patching
  - Effort: Low-Medium (6-10 hours)
  - Impact: Improved function boundary detection

- **#5097** - Logical NOT pattern recognition
  - Effort: Low (4-6 hours)
  - Impact: Cleaner decompilation output

### x86/x86-64 - `/arch/x86/`

- **#4920** - Flag operations simplification
  - Effort: Medium (10-15 hours)
  - Impact: Cleaner IL and decompilation

### C-SKY - (Architecture location unknown, but mentioned in backlog)

- **#5912** - Float conversion signedness
- **#5911** - Carry flag handling

---

## Conclusions

### TriCore Issues (#7131, #6618)

**Status:** BLOCKED - Closed-source architecture

**Recommendation:**
- File internal ticket with Binary Ninja development team
- Cannot be fixed by open-source community
- Requires Binary Ninja internal development resources

### nanoMIPS Issue (#6972)

**Status:** BLOCKED - Closed-source + Missing LLVM support

**Recommendation:**
- Work with MediaTek to upstream nanoMIPS to LLVM mainline, OR
- Implement custom assembler in closed-source nanoMIPS plugin, OR
- Provide external toolchain integration option

**Workaround:**
- Users can manually assemble with MediaTek toolchain
- Copy assembled bytes into Binary Ninja for patching

### Going Forward

**Focus on open-source architectures** where community contributions can make impact:
- ARM64, ARMv7, MIPS, MSP430, PowerPC, RISC-V, x86

**Document Ultimate Only limitations** to set proper expectations for:
- Open-source contributors
- Binary Ninja Ultimate users
- Binary Ninja development team

---

**Investigation Complete:** 2025-11-16
**Investigator:** Joel Reymont
**Next Steps:** Focus on fixable open-source architecture issues
