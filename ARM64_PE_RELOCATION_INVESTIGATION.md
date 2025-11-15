# ARM64 PE Relocation Support Investigation

**Date:** 2025-11-15
**Issue:** #6208 - Implement ARM64 PE Relocations
**Status:** Deferred - requires PE view changes, not architecture plugin alone

## Problem Description

ARM64 EFI binaries fail to load correctly due to unimplemented PE base relocations. Important data structures (e.g., 19-entry jump tables) are being replaced with null pointers when IMAGE_REL_BASED_DIR64 relocations are not applied.

## Current Implementation

### PE Relocation Handler Status

**Location:** `arch/arm64/arch_arm64.cpp:3327-3346`

```cpp
class Arm64PeRelocationHandler : public RelocationHandler
{
public:
    virtual bool GetRelocationInfo(
        Ref<BinaryView> view, Ref<Architecture> arch, vector<BNRelocationInfo>& result) override
    {
        (void)view;
        (void)arch;
        set<uint64_t> relocTypes;
        for (auto& reloc : result)
        {
            reloc.type = UnhandledRelocation;
            relocTypes.insert(reloc.nativeType);
        }
        for (auto& reloc : relocTypes)
            LogWarn("Unsupported PE relocation type: %s", GetRelocationString((PeArm64RelocationType)reloc));
        return false;
    }
};
```

**Current behavior:** Stub implementation that marks all relocations as unhandled and logs warnings.

### COFF Relocation Handler (For Comparison)

**Location:** `arch/arm64/arch_arm64.cpp:3349-3500`

The COFF relocation handler IS fully implemented, handling:
- `IMAGE_REL_ARM64_REL21` - PC-relative 21-bit
- `IMAGE_REL_ARM64_PAGEBASE_REL21` - Page-based addressing
- `IMAGE_REL_ARM64_PAGEOFFSET_12A` - 12-bit page offset (ADD/SUB)
- `IMAGE_REL_ARM64_PAGEOFFSET_12L` - 12-bit page offset (load/store)
- `IMAGE_REL_ARM64_BRANCH26` - 26-bit branch
- `IMAGE_REL_ARM64_BRANCH19` - 19-bit conditional branch
- `IMAGE_REL_ARM64_BRANCH14` - 14-bit test and branch
- `IMAGE_REL_ARM64_SECTION` - Section index
- `IMAGE_REL_ARM64_SECREL` - Section-relative
- `IMAGE_REL_ARM64_ADDR32NB` - 32-bit image-relative address
- `IMAGE_REL_ARM64_ADDR64` - 64-bit absolute address

## Relocation Type Confusion

### Two Different Relocation Systems

**COFF/Object File Relocations:** `IMAGE_REL_ARM64_*`
- Architecture-specific relocations
- Used in object files (.obj) and COFF archives
- Already implemented in Arm64COFFRelocationHandler

**PE Base Relocations:** `IMAGE_REL_BASED_*`
- Generic PE base relocations (not architecture-specific)
- Used in executable PE files (.exe, .efi, .dll)
- Currently unimplemented for ARM64

### Required PE Base Relocation Types

From the issue, these generic PE base relocations need support:

1. **IMAGE_REL_BASED_HIGH** (0x1) - High 16 bits of 32-bit address
2. **IMAGE_REL_BASED_LOW** (0x2) - Low 16 bits of 32-bit address
3. **IMAGE_REL_BASED_HIGHLOW** (0x3) - Full 32-bit address
4. **IMAGE_REL_BASED_HIGHADJ** (0x4) - High 16 bits with adjustment
5. **IMAGE_REL_BASED_DIR64** (0xA) - 64-bit direct address

**Note:** These are NOT ARM64-specific, they're generic PE relocations that could apply to any architecture.

## Implementation Challenges

### Architecture vs View Responsibility

**Issue author's observation:** "These relocation types do not appear to be architecture-dependent"

**Implication:** These should likely be handled in the PE view code, not in each architecture plugin.

**Current architecture:**
- PE view parses base relocation table
- Passes relocations to architecture-specific handler
- ARM64 handler currently rejects all relocations

### PE Base Relocation Format

PE base relocations are simple address adjustments for ASLR:
- Stored in `.reloc` section
- Grouped by 4KB pages
- Each entry: type (4 bits) + offset (12 bits)
- Applied when image loads at different base address

**Formula:** `*address += (actual_base - preferred_base)`

### Why Not in ARM64 Handler?

1. **Not architecture-dependent** - Same logic for x86, x64, ARM, ARM64, etc.
2. **Simple address patching** - No instruction decoding required
3. **Already in PE view** - PE view already parses relocation table
4. **Code duplication** - Would duplicate logic across all architectures

## Alternative Implementations

### Option 1: Centralized PE View Handler

**Best approach:** Handle base relocations in PE view before passing to architecture

**Pros:**
- Single implementation for all architectures
- Avoids code duplication
- Matches PE specification (base relocations are format-level, not architecture-level)

**Cons:**
- Requires PE view changes
- Outside scope of architecture plugin

**Effort:** 8-12 hours (PE view modification)

### Option 2: ARM64 Handler with Generic Logic

**Implementation:** Add base relocation handling to Arm64PeRelocationHandler

**Code sketch:**
```cpp
virtual bool ApplyRelocation(...) override
{
    BNRelocationInfo info = reloc->GetInfo();
    uint64_t delta = view->GetStart() - view->GetDefaultBase();
    uint64_t* dest64 = (uint64_t*)dest;
    uint32_t* dest32 = (uint32_t*)dest;
    uint16_t* dest16 = (uint16_t*)dest;

    switch (info.nativeType)
    {
    case IMAGE_REL_BASED_DIR64:
        if (len >= 8)
            dest64[0] += delta;
        break;
    case IMAGE_REL_BASED_HIGHLOW:
        if (len >= 4)
            dest32[0] += (uint32_t)delta;
        break;
    case IMAGE_REL_BASED_HIGH:
        if (len >= 2)
            dest16[0] += (uint16_t)(delta >> 16);
        break;
    case IMAGE_REL_BASED_LOW:
        if (len >= 2)
            dest16[0] += (uint16_t)(delta & 0xFFFF);
        break;
    // ...
    }
}
```

**Pros:**
- Fixes ARM64 PE loading immediately
- Self-contained in architecture plugin

**Cons:**
- Logic would need to be duplicated for x64, x86, etc.
- Not following proper separation of concerns
- May conflict with future centralized implementation

**Effort:** 4-6 hours (quick fix, but architecturally wrong)

### Option 3: Hybrid Approach

**Implementation:** Add generic base relocation handler to Binary Ninja core, called before architecture handler

**Pros:**
- Clean separation
- No code duplication
- Future-proof

**Cons:**
- Requires core changes
- Larger scope

**Effort:** 12-16 hours (core API extension)

## Testing Considerations

**Test binaries needed:**
- ARM64 UEFI/EFI binaries (most common use case)
- ARM64 PE executables
- ARM64 DLLs with relocations

**Validation:**
- Jump tables resolve correctly
- Global pointers update properly
- Functions load at relocated addresses
- No null pointer corruption

## Recommendation

**Defer to Binary Ninja team** for decision on approach:

1. **Preferred:** Implement centralized base relocation handling in PE view (Option 1)
2. **Quick fix:** Implement in ARM64 handler with TODO to centralize (Option 2)
3. **Long-term:** Core API extension for generic relocations (Option 3)

## Issue with Current Approach

The issue conflates two different types of relocations:
- PE base relocations (IMAGE_REL_BASED_*) - needed for ASLR
- COFF object relocations (IMAGE_REL_ARM64_*) - already implemented

The stub PE relocation handler is checking for the wrong relocation types. It's looking for ARM64-specific types when it should be handling generic base relocations.

## Next Steps

1. **Clarify scope** with Binary Ninja team: Architecture plugin or PE view?
2. **Gather test binaries** with ARM64 PE base relocations
3. **Prototype** Option 2 (quick ARM64 fix) if architectural change is not feasible
4. **Submit PR** for centralized implementation if Option 1 is approved

---

**Investigation by:** Joel Reymont
**Conclusion:** PE base relocations are generic (not ARM64-specific) and should likely be handled in PE view, not architecture plugin. Requires Binary Ninja team decision on implementation approach.
