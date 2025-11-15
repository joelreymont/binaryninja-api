# Architecture Improvement Testing Guide

This document describes the automated tests for Binary Ninja architecture improvements.

## Overview

Automated tests have been created for all implemented fixes:

1. **RISC-V JALR Branch Detection** (#6273)
2. **MIPS64R6 JALR Return Recognition** (#7355)
3. **x86 BEXTR Semantic Lifting** (#6287)
4. **ARM64 LSE Atomic MIN/MAX Intrinsics** (#6599)

## Requirements

- Binary Ninja installation (Commercial, Enterprise, or evaluation)
- Python 3.6 or higher
- Binary Ninja Python API module

### Verifying Binary Ninja Python API

```bash
python3 -c "import binaryninja; print('Binary Ninja found')"
```

If this fails, consult the Binary Ninja documentation:
https://docs.binary.ninja/dev/batch.html

## Running Tests

### Run All Tests

```bash
./run_architecture_tests.sh
```

This will run all test suites and provide a summary.

### Run Individual Test Suites

#### RISC-V JALR Tests
```bash
python3 arch/riscv/test_riscv_jalr.py
```

**What it tests:**
- JALR instructions with rd != 0 are detected as indirect calls
- JALR with rd = 0, rs1 = x1 is detected as function return
- JALR with rd = 0, rs1 != x1 is detected as unresolved branch
- IL lifting generates correct LLIL_CALL, LLIL_RET, or LLIL_JUMP

**Expected output:**
```
Testing RISC-V JALR branch detection (8 tests)...
PASS Test 1: jalr x5, x10, 0 - indirect call
PASS Test 2: jalr x10, x5, 4 - indirect call
...
Results: 8 passed, 0 failed out of 8 tests
```

#### MIPS64R6 JALR Tests
```bash
python3 arch/mips/test_mips64r6_jalr.py
```

**What it tests:**
- `jalr $zero, $ra` is detected as function return (MIPS R6 encoding of `jr $ra`)
- `jalr $zero, $rs` (rs != $ra) is detected as unresolved branch
- `jalr $rd, $rs` (rd != $zero) is detected as indirect call
- JALR.HB variants work correctly
- IL lifting generates correct LLIL_RET, LLIL_JUMP, or LLIL_CALL

**Expected output:**
```
Testing MIPS64R6 JALR branch detection (8 tests)...
Using architecture: mips64
PASS Test 1: jalr $zero, $ra - function return (R6 jr $ra)
...
Results: 8 passed, 0 failed out of 8 tests
```

#### x86 BEXTR Tests
```bash
python3 arch/x86/test_bextr_lifting.py
```

**What it tests:**
- BEXTR instructions are lifted to semantic IL (not opaque intrinsic)
- IL includes logical shift right (LLIL_LSR)
- IL includes bitwise AND (LLIL_AND)
- IL includes shift left for mask generation (LLIL_SHIFT_LEFT)
- BEXTR is not lifted as LLIL_INTRINSIC

**Expected output:**
```
Testing x86-64 BEXTR semantic lifting (3 tests)...
PASS Test 1: bextr eax, ebx, ecx
  Semantic IL generated (not opaque intrinsic)
...
Results: 3 passed, 0 failed out of 3 tests
```

#### ARM64 Atomic MIN/MAX Tests
```bash
python3 arch/arm64/test_atomic_minmax.py
```

**What it tests:**
- All 24 LSE atomic MIN/MAX intrinsic variants are recognized
- LDSMAX, LDSMIN, LDUMAX, LDUMIN (load variants)
- STSMAX, STSMIN, STUMAX, STUMIN (store-only variants)
- Byte (B), halfword (H), and word sizes
- LD* variants have output registers (SET_REG)
- ST* variants do NOT have output registers

**Expected output:**
```
Testing ARM64 LSE atomic MIN/MAX intrinsics (30 tests)...
PASS Test 1: __ldsmax
PASS Test 2: __ldsmax
...
Results: 30 passed, 0 failed out of 30 tests
```

## Test File Locations

- `arch/riscv/test_riscv_jalr.py` - RISC-V JALR tests
- `arch/mips/test_mips64r6_jalr.py` - MIPS64R6 JALR tests
- `arch/x86/test_bextr_lifting.py` - x86 BEXTR tests
- `arch/arm64/test_atomic_minmax.py` - ARM64 atomic tests

## Test Structure

All tests follow a common pattern:

1. **Import Binary Ninja API** - Load the binaryninja module
2. **Encode test instructions** - Generate binary instruction bytes
3. **Create binary view** - Load instruction bytes into Binary Ninja
4. **Get instruction info** - Retrieve branch information
5. **Verify branch type** - Check that the correct branch type is detected
6. **Lift to IL** - Generate Low Level IL
7. **Verify IL** - Check for expected IL operations
8. **Report results** - Pass/fail for each test case

## Adding New Tests

To add tests for a new architecture fix:

1. Create a test file in the architecture directory (e.g., `arch/myarch/test_myfix.py`)
2. Follow the existing test pattern:
   - Import binaryninja module
   - Create test cases (instruction bytes + expected results)
   - Implement test functions
   - Add main execution block
3. Add the test to `run_architecture_tests.sh`
4. Update this documentation

### Test Template

```python
#!/usr/bin/env python3
"""
Architecture: My Fix Tests

Description of what the fix does and what the tests validate.
"""

import sys
try:
    import binaryninja
    from binaryninja import binaryview, Architecture
except ImportError:
    print("ERROR: Binary Ninja not found", file=sys.stderr)
    sys.exit(1)

# Test cases
test_cases = [
    (b'\x00\x00\x00\x00', "description", "expected_result"),
]

def test_my_fix():
    arch = Architecture['myarch']
    # ... test implementation
    return passed == len(test_cases)

if __name__ == '__main__':
    sys.exit(0 if test_my_fix() else 1)
```

## Troubleshooting

### "Binary Ninja not found"

The Binary Ninja Python API is not available. Ensure:
- Binary Ninja is installed
- You're using the correct Python environment
- The `binaryninja` module is in your Python path

### "Architecture not found"

The architecture plugin is not loaded. This could mean:
- The architecture is not built
- The architecture is not installed
- You're using a Binary Ninja edition that doesn't include that architecture

### "Function not created"

Binary Ninja could not create a function from the test bytes. This usually means:
- The instruction bytes are invalid
- The architecture doesn't recognize the instruction
- There's an issue with the binary view setup

### Test failures

If tests fail after making changes:
1. Check that the fix is actually implemented
2. Verify the instruction encoding is correct
3. Review the expected IL patterns
4. Check Binary Ninja version compatibility

## Continuous Integration

These tests can be integrated into CI/CD pipelines:

```yaml
# Example GitHub Actions workflow
- name: Run Architecture Tests
  run: |
    ./run_architecture_tests.sh
```

**Note:** CI environments need Binary Ninja license and installation.

## Test Coverage

Current test coverage by fix:

| Fix | Branch Detection | IL Lifting | Edge Cases |
|-----|-----------------|------------|------------|
| RISC-V JALR | ✓ | ✓ | ✓ |
| MIPS64R6 JALR | ✓ | ✓ | ✓ |
| x86 BEXTR | N/A | ✓ | ✓ |
| ARM64 Atomics | N/A | ✓ | ✓ |

## Performance

Test execution times (approximate, on development machine):

- RISC-V JALR: ~2 seconds
- MIPS64R6 JALR: ~2 seconds
- x86 BEXTR: ~2 seconds
- ARM64 Atomics: ~3 seconds

**Total:** ~10 seconds for all tests

## Future Enhancements

Potential test improvements:

1. **Execution validation** - Run test binaries and verify actual behavior
2. **Symbolic evaluation** - Validate semantic correctness symbolically
3. **HLIL testing** - Test high-level IL decompilation
4. **Cross-architecture tests** - Test architectures side-by-side
5. **Regression tests** - Ensure fixes don't break existing functionality
6. **Performance benchmarks** - Track lifting performance over time

---

**Note:** These tests require Binary Ninja to be installed. They cannot run in environments without Binary Ninja access.
