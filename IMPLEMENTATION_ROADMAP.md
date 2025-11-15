# Binary Ninja Processor Architecture Improvement Roadmap

**Date:** 2025-11-15
**Status:** Implementation Plan
**Author:** Joel Reymont

---

## Overview

This roadmap addresses identified pain points in Binary Ninja processor architectures through a phased approach. Each phase contains small, self-contained features with complete test coverage.

**Key Principles:**
- One commit per logical feature with tests
- Small, self-contained changes
- Complete test coverage in existing style
- Build and test validation before commit
- Short, succinct commit messages

---

## Phase 1: Critical Bug Fixes

**Goal:** Fix high-impact bugs affecting analysis accuracy
**Duration:** 1-2 weeks
**Impact:** Immediate improvement to existing functionality

### Task 1.1: RISC-V JALR Branch Detection Fix

**Issue:** #6273 - Indirect calls via JALR not reported as branches

**Files Modified:**
- `arch/riscv/src/lib.rs` (lines 712-722)

**Changes Required:**
1. Replace JALR branch detection logic
2. Add branch reporting for all JALR variants:
   - `rd == 0 && rs1 == 1` -> FunctionReturn
   - `rd == 0 && rs1 != 1` -> Unresolved
   - `rd != 0` -> Call (indirect call with link)

**Testing:**
Create test file: `arch/riscv/tests/test_jalr_branches.rs`

Test cases:
- `jalr x0, x1, 0` -> FunctionReturn branch
- `jalr x0, x5, 0` -> Unresolved branch
- `jalr x1, x5, 0` -> Call branch
- `jalr x5, x10, 8` -> Call branch with offset
- Verify IL generation still correct

**Build Validation:**
```bash
cd arch/riscv
cargo build
cargo test
```

**Complexity:** Low
**Time Estimate:** 2-3 hours (including tests)
**Priority:** 1 (highest impact, lowest effort)

---

### Task 1.2: MIPS64R6 Return Instruction Recognition

**Issue:** #7355 - MIPS64R6 return instruction variants not recognized

**Prerequisites:**
- Obtain MIPS64R6 ISA specification
- Identify exact opcode encodings for R6 return variants
- Determine if disassembler needs updates

**Files Modified:**
- `arch/mips/mips/mips.h` (add new instruction enum values)
- `arch/mips/arch_mips.cpp` (lines 384-390, branch detection)
- `arch/mips/il.cpp` (lines 1436-1442, IL generation)
- Possibly: `arch/mips/disassembler/` (if new opcodes need decoder support)

**Changes Required:**
1. Add MIPS64R6 instruction variants to enum:
   ```cpp
   MIPS_JR_R6,      // MIPS64R6 return variant
   MIPS_JR_HB_R6,   // MIPS64R6 return with hazard barrier
   ```

2. Update branch detection:
   ```cpp
   case MIPS_JR:
   case MIPS_JR_HB:
   case MIPS_JR_R6:
   case MIPS_JR_HB_R6:
       if (instr.operands[0].reg == REG_RA)
           result.AddBranch(FunctionReturn, 0, nullptr, hasBranchDelay);
       else
           result.AddBranch(UnresolvedBranch, 0, nullptr, hasBranchDelay);
       break;
   ```

3. Update IL generation similarly

4. Update disassembler if needed (depends on investigation)

**Testing:**
Extend `arch/mips/test_lifting.py` or create new test file:

Test cases:
- MIPS64R6 binary with return instructions
- Verify FunctionReturn branch emitted
- Verify IL shows `LLIL_RETURN(LLIL_REG.d(ra))`
- Verify standard MIPS still works (regression)

**Build Validation:**
```bash
cd arch/mips
mkdir build && cd build
cmake ..
make
python3 ../test_lifting.py
```

**Complexity:** Medium
**Time Estimate:** 4-6 hours (including spec research, testing)
**Priority:** 2

**Note:** May require binutils or QEMU as reference for R6 encodings

---

### Task 1.3: x86 BEXTR Instruction Lifting Improvement

**Issue:** #6287 - BEXTR instruction semantics not decoded in IL

**Files Modified:**
- `arch/x86/il.cpp` (add special case before line 4250)

**Changes Required:**

Add BEXTR handler before default intrinsic case:

```cpp
case XED_ICLASS_BEXTR: {
    // BEXTR dst, src, control
    // control = [LENGTH(15:8)][START(7:0)]

    ExprId src = ReadILOperand(il, xedd, addr, 1, opTwoLen);
    ExprId control = ReadILOperand(il, xedd, addr, 2, opTreLen);

    // Extract START[7:0]
    ExprId start = il.LowPart(1, control);

    // Extract LENGTH[15:8]
    ExprId controlLow = il.LowPart(2, control);
    ExprId len = il.LogicalShiftRight(2, controlLow, il.Const(1, 8));

    // Compute: (src >> start) & ((1 << len) - 1)
    ExprId shifted = il.LogicalShiftRight(opTwoLen, src, start);
    ExprId mask_base = il.ShiftLeft(opTwoLen, il.Const(opTwoLen, 1), len);
    ExprId mask = il.Sub(opTwoLen, mask_base, il.Const(opTwoLen, 1));
    ExprId result = il.And(opTwoLen, shifted, mask);

    // Write to destination
    il.AddInstruction(WriteILOperand(il, xedd, addr, 0, 0, result));

    // Set flags (ZF based on result, CF/OF cleared)
    // BEXTR clears OF and CF, sets ZF if result is zero
    il.AddInstruction(il.SetFlag(IL_FLAG_Z, il.CompareEqual(opTwoLen, result, il.Const(opTwoLen, 0))));
    il.AddInstruction(il.SetFlag(IL_FLAG_C, il.Const(0, 0)));
    il.AddInstruction(il.SetFlag(IL_FLAG_O, il.Const(0, 0)));

    break;
}
```

**Testing:**
Extend `arch/x86/test_lifting.py`:

```python
tests_bmi = [
    # bextr eax, ebx, ecx (START in CL, LEN in CH)
    (
        b'\xC4\xE2\x70\xF7\xC3',
        'LLIL_SET_REG.d(eax,LLIL_AND.d(LLIL_LSR.d(LLIL_REG.d(ebx),LLIL_LOW_PART.b(LLIL_REG.d(ecx))),LLIL_SUB.d(LLIL_LSL.d(LLIL_CONST.d(0x1),LLIL_LSR.w(LLIL_LOW_PART.w(LLIL_REG.d(ecx)),LLIL_CONST.b(0x8))),LLIL_CONST.d(0x1)))); LLIL_SET_FLAG(z,...); LLIL_SET_FLAG(c,LLIL_CONST.b(0x0)); LLIL_SET_FLAG(o,LLIL_CONST.b(0x0))'
    ),
]

# Add to test_cases list
test_cases = ... + tests_bmi
```

**Build Validation:**
```bash
cd arch/x86
mkdir build && cd build
cmake ..
make
python3 ../test_lifting.py
```

**Complexity:** Medium
**Time Estimate:** 3-4 hours (including IL string testing)
**Priority:** 3

---

### Task 1.4: ARM/Thumb Calling Convention Investigation

**Issue:** #6615 - Extra arguments in function calls

**Phase 1: Investigation (2-3 hours)**

1. Obtain test case from GitHub issue #6615
2. Load binary in Binary Ninja
3. Identify function with extra arguments
4. Trace parameter inference:
   - Check function signature
   - Check IL for function calls
   - Check stack analysis
   - Check calling convention application

5. Search codebase for parameter detection logic:
   ```bash
   cd arch/armv7
   grep -r "parameter" .
   grep -r "GetIntegerArgumentRegisters" .
   grep -r "InferParameters" .
   ```

6. Locate actual bug (not in CallingConvention definition)

**Phase 2: Fix (1-2 hours after investigation)**

Will depend on investigation results. Likely candidates:
- Stack argument detection heuristics
- Thumb mode convention switching
- Function boundary detection

**Phase 3: Testing**

Create test based on issue #6615 test case:
- Binary with known function signature
- Verify correct number of parameters detected
- Verify correct parameter types
- Test both ARM and Thumb2 modes

**Complexity:** High (investigation required)
**Time Estimate:** 4-6 hours total
**Priority:** 4

**Status:** Requires investigation before implementation plan

---

## Phase 2: Missing Instruction Set Extensions

**Goal:** Add support for missing instruction set variants
**Duration:** 2-3 weeks
**Impact:** Enables analysis of previously unsupported binaries

### Task 2.1: MSP430X Extension Support

**Issue:** #7620 - MSP430X extension support needed

**Investigation Required:**
1. Identify MSP430X instruction differences from MSP430
2. Determine if new opcodes or just extended addressing
3. Check if Rust disassembler needs updates

**Files Affected:**
- `arch/msp430/` (Rust implementation)
- Instruction decoder
- IL generation

**Approach:**
1. Research MSP430X specification
2. Identify missing instructions
3. Add to instruction enum and decoder
4. Add IL lifting for new instructions
5. Create comprehensive test suite

**Testing:**
- MSP430X binaries with extended instructions
- Verify disassembly correct
- Verify IL generation correct
- Regression test standard MSP430

**Complexity:** Medium-High
**Time Estimate:** 8-12 hours
**Priority:** 5

---

### Task 2.2: PowerPC-VLE SPE Instruction Support

**Issue:** #7218 - PowerPC-VLE SPE instruction integration needed

**Background:**
- VLE (Variable Length Encoding) for embedded PowerPC
- SPE (Signal Processing Engine) instructions
- Currently uses Capstone disassembler

**Investigation Required:**
1. Check if Capstone supports VLE SPE
2. If not, may need custom decoder
3. Identify missing instruction patterns

**Files Affected:**
- `arch/powerpc/disassembler.cpp`
- `arch/powerpc/il.cpp`
- Possibly custom VLE decoder

**Approach:**
1. Research PowerPC VLE and SPE specifications
2. Test Capstone support for VLE SPE
3. Add missing instruction support
4. Update IL generation
5. Create test suite

**Testing:**
- PowerPC VLE binaries with SPE instructions
- Verify disassembly
- Verify IL generation
- Regression test standard PowerPC

**Complexity:** High
**Time Estimate:** 12-16 hours
**Priority:** 6

---

### Task 2.3: nanoMIPS Assembler Support

**Issue:** #6972 - nanoMIPS assembler functionality missing

**Background:**
- Disassembly works
- Assembler (for patching) missing

**Investigation Required:**
1. Check existing MIPS assembler implementation
2. Identify nanoMIPS encoding differences
3. Determine scope (full assembler vs basic patching)

**Files Affected:**
- `arch/mips/` (new assembler component)
- May follow pattern from PowerPC assembler

**Approach:**
1. Research nanoMIPS encoding
2. Implement instruction assembler
3. Add encoding tables
4. Create assembler API
5. Test with patch operations

**Testing:**
- Assemble common nanoMIPS instructions
- Round-trip test: disassemble -> assemble -> compare
- Test patching in real binaries

**Complexity:** High
**Time Estimate:** 16-20 hours
**Priority:** 7

---

## Phase 3: Feature Parity and Enhancements

**Goal:** Bring all architectures to consistent feature level
**Duration:** 3-4 weeks
**Impact:** Better analysis for modern instruction sets

### Task 3.1: ARM64 Atomic Operation Intrinsics

**Issue:** #6599 - Aarch64 atomic operation intrinsics needed

**Background:**
- ARM64 has extensive atomic operations (LSE extension)
- Currently may be lifted as generic intrinsics
- Should have semantic intrinsics

**Files Affected:**
- `arch/arm64/il.cpp`
- `arch/arm64/neon_intrinsics.cpp` (or new atomics file)

**Atomic Operations to Support:**
- LDADD, LDCLR, LDEOR, LDSET (atomic fetch-and-op)
- LDMAX, LDMIN, LDUMAX, LDUMIN (atomic min/max)
- SWP (atomic swap)
- CAS (compare and swap)

**Approach:**
1. Research ARM64 LSE (Large System Extensions)
2. Define intrinsic names following LLVM/GCC conventions
3. Add IL generation for atomic operations
4. Create comprehensive test suite

**Testing:**
Follow `arch/arm64/arm64test.py` pattern:
```python
tests_atomics = [
    # ldadd x0, x1, [x2]
    (b'\x41\x00\x20\xF8', 'LLIL_INTRINSIC([x1],_ldadd_64,[x0,x2])'),
    # cas x0, x1, [x2]
    (b'\x41\x7C\xA0\xC8', 'LLIL_INTRINSIC([x1],_cas_64,[x0,x2])'),
]
```

**Complexity:** Medium
**Time Estimate:** 6-8 hours
**Priority:** 8

---

### Task 3.2: ARM64 Pointer Authentication Optimization

**Issue:** #6702 - Pointer authentication check optimization

**Background:**
- ARM64 PAC (Pointer Authentication) adds authentication codes to pointers
- Current implementation may be slow or incomplete
- Need optimized detection and handling

**Investigation Required:**
1. Profile current PAC handling performance
2. Identify optimization opportunities
3. Check if caching or early detection possible

**Files Affected:**
- `arch/arm64/il.cpp` (PAC instruction handling)
- Possibly analysis framework

**Approach:**
1. Benchmark current PAC performance
2. Identify bottlenecks
3. Implement optimizations (caching, early checks)
4. Verify correctness maintained
5. Benchmark improvements

**Testing:**
- Binaries with heavy PAC usage
- Performance benchmarks
- Correctness regression tests

**Complexity:** Medium-High
**Time Estimate:** 8-10 hours
**Priority:** 9

---

### Task 3.3: ARM BE8 Support Outside ELF

**Issue:** #7217 - ARM BE8 support outside ELF format

**Background:**
- BE8 (Byte-Invariant Big-Endian) is ARM big-endian mode
- Currently only works with ELF
- Should work with raw binaries, Mach-O, etc.

**Investigation Required:**
1. Understand current BE8 detection mechanism
2. Identify ELF-specific dependencies
3. Design format-independent detection

**Files Affected:**
- `arch/armv7/arch_armv7.cpp`
- Binary format loaders

**Approach:**
1. Research BE8 detection heuristics
2. Move BE8 logic from ELF loader to architecture
3. Add configuration options
4. Test with multiple formats

**Testing:**
- BE8 raw binaries
- BE8 Mach-O files
- Regression test ELF BE8

**Complexity:** Medium
**Time Estimate:** 6-8 hours
**Priority:** 10

---

## Phase 4: Testing Infrastructure Improvements

**Goal:** Comprehensive test coverage and better testing tools
**Duration:** 2-3 weeks
**Impact:** Prevents regressions, easier to add features

### Task 4.1: Standardized IL Test Framework

**Goal:** Consistent IL testing across all architectures

**Deliverables:**
1. Python module: `arch/common/il_test_utils.py`
   - `il2str()` converter (normalized across architectures)
   - `instr_to_il()` harness
   - Assertion helpers
   - Test data structures

2. Documentation: `arch/TESTING_GUIDE.md`
   - How to write IL tests
   - Test patterns and examples
   - Best practices

3. Template: `arch/common/test_template.py`
   - Boilerplate for new test files

**Testing:**
- Migrate existing tests to use common framework
- Verify all architectures work

**Complexity:** Medium
**Time Estimate:** 8-12 hours
**Priority:** 11

---

### Task 4.2: Edge Case Test Suite

**Goal:** Comprehensive coverage of edge cases from issues

**Approach:**
For each architecture, add tests for:
1. All branch types (calls, returns, indirect, conditional)
2. All addressing modes
3. Boundary conditions (max immediates, edge registers)
4. Mode switches (ARM/Thumb, x86/x64)
5. Known issues from GitHub

**Deliverables:**
- `arch/{arch}/test_edge_cases.py` for each architecture
- Minimum 50+ edge cases per architecture

**Testing:**
- Run new edge case suites
- Verify no false positives
- Document any expected failures

**Complexity:** Medium-High
**Time Estimate:** 16-20 hours (across all architectures)
**Priority:** 12

---

### Task 4.3: Automated Regression Test Suite

**Goal:** Prevent regressions when adding features

**Deliverables:**
1. Script: `run_all_tests.sh`
   - Runs all Python tests
   - Runs all Rust tests
   - Runs all C++ standalone tests
   - Generates coverage report

2. CI/CD configuration (if applicable)
   - GitHub Actions workflow
   - Test on multiple platforms

3. Documentation: Update with test running instructions

**Testing:**
- Verify script runs all tests
- Verify failures detected correctly

**Complexity:** Low-Medium
**Time Estimate:** 4-6 hours
**Priority:** 13

---

## Phase 5: Documentation and Standardization

**Goal:** Make architecture development accessible to contributors
**Duration:** 1-2 weeks
**Impact:** Lower barrier to entry, faster development

### Task 5.1: Architecture Developer Guide

**Deliverable:** `docs/ARCHITECTURE_DEVELOPER_GUIDE.md`

**Contents:**
1. Architecture Plugin Structure
   - Required components
   - Optional components
   - File organization

2. Disassembler Integration
   - Custom disassembler pattern
   - External library pattern (Capstone, XED)
   - Pros/cons of each approach

3. Instruction Info
   - Branch detection
   - Instruction length
   - Operand parsing

4. IL Generation
   - IL operation overview
   - Common patterns (loads, stores, arithmetic)
   - Flag handling
   - Intrinsics vs semantic IL

5. Calling Conventions
   - Defining register conventions
   - Multiple ABIs per architecture
   - Platform-specific variants

6. Testing
   - Disassembly tests
   - IL tests
   - Test patterns to follow

7. Build System
   - CMake integration
   - Rust/Cargo integration
   - Dependencies

**Complexity:** Medium
**Time Estimate:** 12-16 hours
**Priority:** 14

---

### Task 5.2: IL Generation Best Practices

**Deliverable:** `docs/IL_GENERATION_GUIDE.md`

**Contents:**
1. IL Philosophy
   - What belongs in IL
   - Semantic vs opaque (intrinsics)
   - Precision vs simplicity tradeoffs

2. Common Patterns
   - Loads and stores
   - Arithmetic operations
   - Shifts and rotates
   - Conditional operations
   - Branch and call handling

3. Flag Modeling
   - Flag groups
   - Flag semantics
   - Undefined flags

4. Register Handling
   - Register hierarchies
   - Partial register updates
   - Special registers

5. Intrinsics
   - When to use intrinsics
   - Naming conventions
   - Parameter passing

6. Debugging IL
   - IL visualization
   - Common mistakes
   - Testing strategies

**Complexity:** Medium
**Time Estimate:** 8-12 hours
**Priority:** 15

---

### Task 5.3: Architecture Template Generator

**Deliverable:** `tools/arch_template_generator.py`

**Functionality:**
```bash
./tools/arch_template_generator.py --name myarch --lang cpp
# Creates arch/myarch/ with skeleton files:
#   arch_myarch.cpp
#   il.cpp / il.h
#   il_macros.h
#   CMakeLists.txt
#   README.md
#   test_lifting.py
```

**Template includes:**
- Boilerplate Architecture class
- IL generation skeleton
- Test harness
- Build configuration
- README with TODOs

**Complexity:** Low-Medium
**Time Estimate:** 6-8 hours
**Priority:** 16

---

## Phase 6: Long-Term Vision

**Goal:** Reduce duplication and improve maintainability
**Duration:** 4-6 weeks
**Impact:** Easier to add new architectures

### Task 6.1: Common IL Utilities Library

**Deliverable:** `arch/common/il_utils.cpp` + `il_utils.h`

**Contents:**
- Common flag computation functions
- Standard load/store helpers
- Shift/rotate helpers
- Arithmetic with flags helpers
- Conditional operation builders

**Approach:**
1. Identify common patterns across architectures
2. Extract to shared library
3. Refactor architectures to use common code
4. Document usage patterns

**Complexity:** High
**Time Estimate:** 20-24 hours
**Priority:** 17

---

### Task 6.2: Disassembler Integration Framework

**Deliverable:** Standardized disassembler plugin interface

**Contents:**
- Abstract disassembler interface
- Capstone wrapper
- XED wrapper
- Custom disassembler helpers
- Documentation for adding new disassemblers

**Complexity:** High
**Time Estimate:** 24-30 hours
**Priority:** 18

---

### Task 6.3: Architecture Validation Suite

**Deliverable:** `tools/validate_architecture.py`

**Functionality:**
- Checks architecture completeness
- Validates instruction coverage
- Checks test coverage
- Identifies missing functionality
- Generates coverage reports

**Output:**
```
Architecture: arm64
  Instruction Coverage: 95%
  IL Coverage: 92%
  Test Coverage: 87%
  Missing: 15 SVE2 instructions
  Warnings: 3 intrinsics without tests
```

**Complexity:** Medium-High
**Time Estimate:** 12-16 hours
**Priority:** 19

---

## Implementation Schedule

### Month 1: Critical Fixes
- Week 1-2: Tasks 1.1, 1.2, 1.3 (RISC-V, MIPS, x86 fixes)
- Week 3-4: Task 1.4 (ARM investigation and fix)

### Month 2: Extensions and Features
- Week 1-2: Tasks 2.1, 2.2 (MSP430X, PowerPC VLE)
- Week 3-4: Tasks 2.3, 3.1 (nanoMIPS, ARM64 atomics)

### Month 3: Testing and Documentation
- Week 1-2: Tasks 4.1, 4.2, 4.3 (Testing infrastructure)
- Week 3-4: Tasks 5.1, 5.2, 5.3 (Documentation)

### Month 4+: Long-term improvements
- Tasks 6.1, 6.2, 6.3 (Common libraries, validation)
- Additional features as needed

---

## Success Metrics

### Immediate (Phase 1 complete)
- All 4 critical bugs fixed
- Test coverage for all fixes
- No regressions in existing tests

### Medium-term (Phases 1-3 complete)
- 3+ missing instruction sets supported
- Feature parity across major architectures
- 90%+ test coverage for core functionality

### Long-term (All phases)
- Comprehensive developer documentation
- Standardized patterns across architectures
- New architecture can be added in < 1 week

---

## Risk Mitigation

### Technical Risks
1. **Specification availability**: Some ISA specs may be proprietary
   - Mitigation: Use public docs, reverse engineer from binutils/gcc

2. **Test data acquisition**: Need binaries for testing
   - Mitigation: Use assemblers to create test binaries

3. **Build system complexity**: Multiple languages/toolchains
   - Mitigation: Document requirements, use Docker for consistency

### Process Risks
1. **Scope creep**: Many potential improvements
   - Mitigation: Stick to phased approach, one feature at a time

2. **Breaking changes**: IL changes may affect users
   - Mitigation: Extensive regression testing, careful reviews

---

## Next Steps

1. Review and approve this roadmap
2. Set up development environment
3. Begin with Task 1.1 (RISC-V JALR fix - highest priority)
4. Create feature branch for each task
5. One commit per task with tests
6. Review and merge systematically

---

## Appendix: File Locations Quick Reference

### Critical Bug Files
- RISC-V JALR: `arch/riscv/src/lib.rs:712-722`
- MIPS64R6: `arch/mips/arch_mips.cpp:384-390`, `arch/mips/il.cpp:1436-1442`
- x86 BEXTR: `arch/x86/il.cpp:4250`
- ARM/Thumb: TBD (investigation needed)

### Test File Patterns
- IL tests: `arch/{arch}/test_lifting.py`
- Disasm tests: `arch/{arch}/test_disasm.py`
- Rust tests: `arch/riscv/tests/*.rs`, `rust/tests/*.rs`

### Build Files
- CMake: `arch/{arch}/CMakeLists.txt`
- Cargo: `arch/riscv/Cargo.toml`, `rust/Cargo.toml`
