# ARM BE8 Support Outside ELF Investigation

**Date:** 2025-11-15
**Issue:** #7217 - Add support for ARM BE8 outside of an ELF
**Status:** Deferred - requires core architecture changes

## Problem Description

Binary Ninja supports ARM BE8 (big-endian data, little-endian code) only for ELF binaries with the `EF_ARM_BE8` flag. Users analyzing non-ELF firmware cannot enable this mixed-endianness mode.

## BE8 Explained

**BE8 (Big-Endian Word Invariant):**
- **Instructions:** Little-endian
- **Data:** Big-endian

**Purpose:** Allows big-endian data on ARM without byte-swapping instructions

**Example:**
```
Instruction: 0xE3A01042  (stored as: 42 10 A0 E3 in memory)
Data word:   0x12345678  (stored as: 12 34 56 78 in memory)
```

## Current Implementation

### ELF BE8 Detection

**Location:** `view/elf/elfview.cpp:3025-3052`

```cpp
bool checkForARMBE8 = Settings::Instance()->Get<bool>("files.elf.detectARMBE8Binary");
if (checkForARMBE8)
    endianness = ((commonHeader.arch == EM_ARM) && (header.flags & EF_ARM_BE8)) ? BigEndian : endianness;

// ...later...

BNEndianness codeEndianness = endianness;
if (checkForARMBE8 && (commonHeader.arch == EM_ARM) && (header.flags & EF_ARM_BE8))
    codeEndianness = LittleEndian;

*arch = g_elfViewType->GetArchitecture(commonHeader.arch, codeEndianness);
```

**Mechanism:**
1. Detect `EF_ARM_BE8` flag (0x00800000) in ELF header
2. Set data endianness to `BigEndian`
3. Set code endianness to `LittleEndian`
4. Architecture selected based on code endianness

**Limitation:** Only works for ELF files

### ARM Architecture Registration

**Location:** `arch/armv7/arch_armv7.cpp:3323-3324`

```cpp
RegisterArmArchitecture("armv7", "thumb2", LittleEndian);
RegisterArmArchitecture("armv7eb", "thumb2eb", BigEndian);
```

**Current architectures:**
- `armv7` - Little-endian code and data
- `armv7eb` - Big-endian code and data

**Missing:** BE8 variant (little-endian code, big-endian data)

## Implementation Approaches

### Option 1: Register BE8 Architecture Variant

**Implementation:**
```cpp
RegisterArmArchitecture("armv7", "thumb2", LittleEndian);
RegisterArmArchitecture("armv7eb", "thumb2eb", BigEndian);
RegisterArmArchitecture("armv7-be8", "thumb2-be8", ??? );  // Problem: single endianness parameter
```

**Blocker:** `RegisterArmArchitecture` accepts single endianness parameter

**Required changes:**
1. Modify architecture API to support mixed endianness
2. Update instruction fetch logic to use code endianness
3. Update data access logic to use data endianness
4. Create new architecture variant registration

**Effort:** 12-16 hours (core API changes required)

### Option 2: Architecture Setting/Flag

**Implementation:**
Add BE8 mode as architecture-specific setting:

```cpp
class ArmArchitecture : public Architecture {
    bool m_be8Mode;

    BNEndianness GetInstructionEndianness() override {
        return m_be8Mode ? LittleEndian : GetEndianness();
    }

    BNEndianness GetDataEndianness() override {
        return GetEndianness();
    }
};
```

**Required changes:**
1. Add BE8 mode flag to architecture class
2. Split endianness queries into instruction vs data
3. Update all memory access and instruction fetch call sites
4. Add UI for toggling BE8 mode

**Effort:** 16-20 hours (extensive architecture changes)

### Option 3: Decouple Endianness in UI

**Implementation:**
Modify "Open with Options" dialog to allow:
- **Code endianness:** Little | Big
- **Data endianness:** Little | Big

**Required changes:**
1. Extend platform/architecture selection UI
2. Pass dual endianness through view creation
3. Update architecture API to accept both
4. Modify all architectures to support split endianness

**Effort:** 20-30 hours (UI and core changes)

## Workaround Analysis

**Current user workaround:**
1. Load binary with `armv7eb` (fully big-endian)
2. Hold analysis
3. Switch architecture to little-endian via console
4. Resume analysis

**Why it works:** Code is disassembled with little-endian, data remains big-endian from initial load

**Problems:**
- Manual, error-prone process
- Not discoverable
- Analysis results may be inconsistent

## Blocking Issues

1. **Architecture API limitation:** Single endianness parameter
2. **Core dependency:** Requires changes to core architecture handling
3. **Testing:** Need BE8 firmware binaries for validation
4. **Scope:** Affects all architecture plugins, not just ARM

## Use Cases

**Affected users:**
- Firmware reverse engineers
- Embedded systems analysts
- Network appliance researchers

**Common BE8 targets:**
- ARM big-endian embedded firmware
- Legacy ARM network devices
- Industrial control systems

## Recommendation

**Defer implementation** due to:

1. **Core dependency:** Requires core Binary Ninja architecture API changes
2. **Scope:** Not an architecture plugin issue alone
3. **Effort:** 12-30 hours depending on approach chosen
4. **Testing:** Lack of BE8 test binaries

## Next Steps

1. **Request from Binary Ninja team:** Core API support for mixed endianness
2. **Collect test cases:** Gather BE8 firmware samples
3. **Design review:** Determine best approach for mixed endianness support
4. **Prototype:** Test approach with minimal architecture changes

## Suggested Core API Enhancement

```cpp
// Proposed API extension
struct BNArchitectureEndianness {
    BNEndianness instruction;
    BNEndianness data;
};

// Extended registration
BNRegisterArchitecture(
    const char* name,
    BNArchitectureEndianness endianness,
    BNArchitecture* arch
);
```

This would allow architectures to specify different endianness for instructions vs data without breaking existing single-endianness architectures.

---

**Investigation by:** Joel Reymont
**Conclusion:** BE8 support requires core Binary Ninja API changes to support mixed endianness, cannot be implemented in architecture plugin alone
